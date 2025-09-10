use anyhow::Result;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};

/// Dependency analysis for code blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub source_id: Uuid,
    pub target_id: Uuid,
    pub dependency_type: DependencyType,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    Import,
    FunctionCall,
    ClassInheritance,
    VariableReference,
    TypeReference,
    ModuleReference,
}

/// Dependency analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub dependencies: Vec<Dependency>,
    pub circular_dependencies: Vec<Vec<Uuid>>,
    pub dependency_layers: Vec<HashSet<Uuid>>,
}

/// Dependency analyzer
pub struct DependencyAnalyzer {
    db: crate::database::Database,
}

impl DependencyAnalyzer {
    pub fn new(db: crate::database::Database) -> Self {
        Self { db }
    }
    
    pub async fn analyze_dependencies(&self, block_ids: &[Uuid]) -> Result<DependencyGraph> {
        let mut dependencies = Vec::new();
        
        // Collect all dependencies for the given blocks
        for &block_id in block_ids {
            let block_dependencies = self.get_block_dependencies(block_id).await?;
            dependencies.extend(block_dependencies);
        }
        
        // Detect circular dependencies
        let circular_dependencies = self.detect_circular_dependencies(&dependencies).await?;
        
        // Calculate dependency layers (topological ordering)
        let dependency_layers = self.calculate_dependency_layers(&dependencies).await?;
        
        Ok(DependencyGraph {
            dependencies,
            circular_dependencies,
            dependency_layers,
        })
    }
    
    async fn get_block_dependencies(&self, block_id: Uuid) -> Result<Vec<Dependency>> {
        let relationships = sqlx::query!(
            r#"
            SELECT 
                source_block_id,
                target_block_id,
                relationship_type,
                metadata
            FROM block_relationships 
            WHERE source_block_id = $1 OR target_block_id = $1
            "#,
            block_id
        )
        .fetch_all(self.db.pool())
        .await?;

        let mut dependencies = Vec::new();
        for row in relationships {
            let dependency_type = match row.relationship_type.as_str() {
                "imports" => DependencyType::Import,
                "calls" => DependencyType::FunctionCall,
                "inherits" => DependencyType::ClassInheritance,
                "references" => DependencyType::VariableReference,
                "uses_type" => DependencyType::TypeReference,
                "module_ref" => DependencyType::ModuleReference,
                _ => DependencyType::VariableReference, // Default
            };

            dependencies.push(Dependency {
                source_id: row.source_block_id,
                target_id: row.target_block_id,
                dependency_type,
                metadata: row.metadata
                    .and_then(|v| serde_json::from_value(v).ok())
                    .unwrap_or_default(),
            });
        }

        Ok(dependencies)
    }
    
    pub async fn detect_circular_dependencies(&self, dependencies: &[Dependency]) -> Result<Vec<Vec<Uuid>>> {
        // Build adjacency list
        let mut graph: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        let mut all_nodes = HashSet::new();
        
        for dep in dependencies {
            graph.entry(dep.source_id).or_default().push(dep.target_id);
            all_nodes.insert(dep.source_id);
            all_nodes.insert(dep.target_id);
        }
        
        // Use Tarjan's algorithm to find strongly connected components
        let mut index_counter = 0;
        let mut stack = Vec::new();
        let mut indices: HashMap<Uuid, usize> = HashMap::new();
        let mut lowlinks: HashMap<Uuid, usize> = HashMap::new();
        let mut on_stack: HashSet<Uuid> = HashSet::new();
        let mut sccs = Vec::new();
        
        for &node in &all_nodes {
            if !indices.contains_key(&node) {
                self.tarjan_strongconnect(
                    node,
                    &graph,
                    &mut index_counter,
                    &mut stack,
                    &mut indices,
                    &mut lowlinks,
                    &mut on_stack,
                    &mut sccs,
                );
            }
        }
        
        // Filter out single-node SCCs (not circular)
        let circular_deps: Vec<Vec<Uuid>> = sccs
            .into_iter()
            .filter(|scc| scc.len() > 1)
            .collect();
            
        Ok(circular_deps)
    }
    
    fn tarjan_strongconnect(
        &self,
        v: Uuid,
        graph: &HashMap<Uuid, Vec<Uuid>>,
        index_counter: &mut usize,
        stack: &mut Vec<Uuid>,
        indices: &mut HashMap<Uuid, usize>,
        lowlinks: &mut HashMap<Uuid, usize>,
        on_stack: &mut HashSet<Uuid>,
        sccs: &mut Vec<Vec<Uuid>>,
    ) {
        // Set the depth index for v to the smallest unused index
        indices.insert(v, *index_counter);
        lowlinks.insert(v, *index_counter);
        *index_counter += 1;
        stack.push(v);
        on_stack.insert(v);
        
        // Consider successors of v
        if let Some(successors) = graph.get(&v) {
            for &w in successors {
                if !indices.contains_key(&w) {
                    // Successor w has not yet been visited; recurse on it
                    self.tarjan_strongconnect(w, graph, index_counter, stack, indices, lowlinks, on_stack, sccs);
                    lowlinks.insert(v, lowlinks[&v].min(lowlinks[&w]));
                } else if on_stack.contains(&w) {
                    // Successor w is in stack and hence in the current SCC
                    lowlinks.insert(v, lowlinks[&v].min(indices[&w]));
                }
            }
        }
        
        // If v is a root node, pop the stack and create an SCC
        if lowlinks[&v] == indices[&v] {
            let mut scc = Vec::new();
            loop {
                let w = stack.pop().unwrap();
                on_stack.remove(&w);
                scc.push(w);
                if w == v {
                    break;
                }
            }
            sccs.push(scc);
        }
    }
    
    async fn calculate_dependency_layers(&self, dependencies: &[Dependency]) -> Result<Vec<HashSet<Uuid>>> {
        // Build adjacency list and in-degree count
        let mut graph: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        let mut in_degree: HashMap<Uuid, usize> = HashMap::new();
        let mut all_nodes = HashSet::new();
        
        for dep in dependencies {
            graph.entry(dep.source_id).or_default().push(dep.target_id);
            *in_degree.entry(dep.target_id).or_insert(0) += 1;
            in_degree.entry(dep.source_id).or_insert(0);
            all_nodes.insert(dep.source_id);
            all_nodes.insert(dep.target_id);
        }
        
        // Kahn's algorithm for topological sorting
        let mut layers = Vec::new();
        let mut queue: Vec<Uuid> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(&node, _)| node)
            .collect();
        
        while !queue.is_empty() {
            let current_layer: HashSet<Uuid> = queue.iter().cloned().collect();
            layers.push(current_layer.clone());
            
            let mut next_queue = Vec::new();
            
            for &node in &queue {
                if let Some(neighbors) = graph.get(&node) {
                    for &neighbor in neighbors {
                        if let Some(degree) = in_degree.get_mut(&neighbor) {
                            *degree -= 1;
                            if *degree == 0 {
                                next_queue.push(neighbor);
                            }
                        }
                    }
                }
            }
            
            queue = next_queue;
        }
        
        Ok(layers)
    }
    
    /// Calculate dependency metrics for a block
    pub async fn calculate_dependency_metrics(&self, block_id: Uuid) -> Result<DependencyMetrics> {
        let dependencies = self.get_block_dependencies(block_id).await?;
        
        let efferent_coupling = dependencies
            .iter()
            .filter(|dep| dep.source_id == block_id)
            .count();
            
        let afferent_coupling = dependencies
            .iter()
            .filter(|dep| dep.target_id == block_id)
            .count();
            
        let instability = if efferent_coupling + afferent_coupling == 0 {
            0.0
        } else {
            efferent_coupling as f64 / (efferent_coupling + afferent_coupling) as f64
        };
        
        Ok(DependencyMetrics {
            efferent_coupling,
            afferent_coupling,
            instability,
            abstractness: 0.0, // TODO: Calculate based on abstract classes/interfaces
        })
    }
    
    /// Find dependency paths between two blocks
    pub async fn find_dependency_path(&self, from: Uuid, to: Uuid) -> Result<Option<Vec<Uuid>>> {
        let dependencies = self.get_block_dependencies(from).await?;
        let mut graph: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        
        for dep in dependencies {
            graph.entry(dep.source_id).or_default().push(dep.target_id);
        }
        
        // BFS to find shortest path
        let mut queue = std::collections::VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent: HashMap<Uuid, Uuid> = HashMap::new();
        
        queue.push_back(from);
        visited.insert(from);
        
        while let Some(current) = queue.pop_front() {
            if current == to {
                // Reconstruct path
                let mut path = Vec::new();
                let mut node = to;
                path.push(node);
                
                while let Some(&p) = parent.get(&node) {
                    path.push(p);
                    node = p;
                }
                
                path.reverse();
                return Ok(Some(path));
            }
            
            if let Some(neighbors) = graph.get(&current) {
                for &neighbor in neighbors {
                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        parent.insert(neighbor, current);
                        queue.push_back(neighbor);
                    }
                }
            }
        }
        
        Ok(None)
    }
}

/// Dependency metrics for a block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyMetrics {
    pub efferent_coupling: usize,  // Ce - outgoing dependencies
    pub afferent_coupling: usize,  // Ca - incoming dependencies  
    pub instability: f64,          // I = Ce / (Ca + Ce)
    pub abstractness: f64,         // A - ratio of abstract classes
}

// Note: Default implementation removed since DependencyAnalyzer now requires a Database parameter


