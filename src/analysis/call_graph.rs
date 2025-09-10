use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;
use anyhow::Result;
use serde::{Serialize, Deserialize};
use crate::database::{Database, Block};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallGraphNode {
    pub block_id: Uuid,
    pub function_name: String,
    pub file_path: Option<String>,
    pub line_number: Option<u32>,
    pub calls: Vec<Uuid>,
    pub called_by: Vec<Uuid>,
    pub complexity_score: f64,
    pub is_recursive: bool,
    pub is_entry_point: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallPath {
    pub path: Vec<Uuid>,
    pub depth: usize,
    pub has_cycles: bool,
    pub total_complexity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallGraphAnalysis {
    pub nodes: HashMap<Uuid, CallGraphNode>,
    pub entry_points: Vec<Uuid>,
    pub cycles: Vec<Vec<Uuid>>,
    pub max_depth: usize,
    pub total_functions: usize,
    pub recursive_functions: usize,
    pub unreachable_functions: Vec<Uuid>,
    pub critical_paths: Vec<CallPath>,
}

pub struct CallGraphAnalyzer {
    db: Database,
}

impl CallGraphAnalyzer {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn analyze_container(&self, container_id: Uuid) -> Result<CallGraphAnalysis> {
        let blocks = self.db.get_blocks_by_container(container_id).await?;
        let relationships = self.db.get_relationships_by_container(container_id).await?;

        // Build call graph nodes
        let mut nodes = HashMap::new();
        let function_blocks: Vec<_> = blocks.iter()
            .filter(|b| b.block_type == "Function")
            .collect();

        for block in function_blocks {
            let node = CallGraphNode {
                block_id: block.id,
                function_name: block.semantic_name.clone().unwrap_or_else(|| "anonymous".to_string()),
                file_path: self.extract_file_path(block),
                line_number: self.extract_line_number(block),
                calls: Vec::new(),
                called_by: Vec::new(),
                complexity_score: self.calculate_complexity_score(block),
                is_recursive: false,
                is_entry_point: false,
            };
            nodes.insert(block.id, node);
        }

        // Build call relationships
        for relationship in relationships {
            if relationship.relationship_type == "Calls" {
                // Check if both nodes exist first
                if nodes.contains_key(&relationship.source_block_id) && 
                   nodes.contains_key(&relationship.target_block_id) {
                    
                    // Update caller
                    if let Some(caller) = nodes.get_mut(&relationship.source_block_id) {
                        caller.calls.push(relationship.target_block_id);
                    }
                    
                    // Update callee
                    if let Some(callee) = nodes.get_mut(&relationship.target_block_id) {
                        callee.called_by.push(relationship.source_block_id);
                    }
                }
            }
        }

        // Detect recursion - collect IDs first to avoid borrow checker issues
        let node_ids: Vec<_> = nodes.keys().cloned().collect();
        for id in node_ids {
            let is_recursive = self.is_recursive_function(id, &nodes);
            if let Some(node) = nodes.get_mut(&id) {
                node.is_recursive = is_recursive;
            }
        }

        // Identify entry points (functions not called by others)
        let entry_points: Vec<_> = nodes.iter()
            .filter(|(_, node)| node.called_by.is_empty())
            .map(|(id, _)| *id)
            .collect();

        // Mark entry points
        for entry_id in &entry_points {
            if let Some(node) = nodes.get_mut(entry_id) {
                node.is_entry_point = true;
            }
        }

        // Detect cycles
        let cycles = self.detect_cycles(&nodes);

        // Calculate max depth
        let max_depth = self.calculate_max_depth(&nodes, &entry_points);

        // Find unreachable functions
        let unreachable_functions = self.find_unreachable_functions(&nodes, &entry_points);

        // Identify critical paths
        let critical_paths = self.find_critical_paths(&nodes, &entry_points);

        Ok(CallGraphAnalysis {
            nodes: nodes.clone(),
            entry_points,
            cycles,
            max_depth,
            total_functions: blocks.iter().filter(|b| b.block_type == "Function").count(),
            recursive_functions: nodes.values().filter(|n| n.is_recursive).count(),
            unreachable_functions,
            critical_paths,
        })
    }

    pub async fn get_function_dependencies(&self, function_id: Uuid) -> Result<Vec<Uuid>> {
        let analysis = self.analyze_container(
            self.db.get_container_id_by_block(function_id).await?
        ).await?;

        if let Some(_node) = analysis.nodes.get(&function_id) {
            Ok(self.get_all_dependencies(function_id, &analysis.nodes))
        } else {
            Ok(Vec::new())
        }
    }

    pub async fn get_function_dependents(&self, function_id: Uuid) -> Result<Vec<Uuid>> {
        let analysis = self.analyze_container(
            self.db.get_container_id_by_block(function_id).await?
        ).await?;

        if let Some(_node) = analysis.nodes.get(&function_id) {
            Ok(self.get_all_dependents(function_id, &analysis.nodes))
        } else {
            Ok(Vec::new())
        }
    }

    pub fn generate_dot_graph(&self, analysis: &CallGraphAnalysis) -> String {
        let mut dot = String::from("digraph CallGraph {\n");
        dot.push_str("  rankdir=TB;\n");
        dot.push_str("  node [shape=box];\n\n");

        // Add nodes
        for (id, node) in &analysis.nodes {
            let color = if node.is_entry_point {
                "lightgreen"
            } else if node.is_recursive {
                "lightcoral"
            } else {
                "lightblue"
            };

            dot.push_str(&format!(
                "  \"{}\" [label=\"{}\\nComplexity: {:.1}\" fillcolor={} style=filled];\n",
                id, node.function_name, node.complexity_score, color
            ));
        }

        dot.push_str("\n");

        // Add edges
        for (_, node) in &analysis.nodes {
            for &callee_id in &node.calls {
                dot.push_str(&format!("  \"{}\" -> \"{}\";\n", node.block_id, callee_id));
            }
        }

        // Highlight cycles
        for cycle in &analysis.cycles {
            if cycle.len() > 1 {
                dot.push_str(&format!("\n  // Cycle: {:?}\n", cycle));
                for i in 0..cycle.len() {
                    let current = cycle[i];
                    let next = cycle[(i + 1) % cycle.len()];
                    dot.push_str(&format!(
                        "  \"{}\" -> \"{}\" [color=red weight=2];\n",
                        current, next
                    ));
                }
            }
        }

        dot.push_str("}\n");
        dot
    }

    fn extract_file_path(&self, block: &Block) -> Option<String> {
        block.metadata.as_ref()
            .and_then(|m| m.get("file_path"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    fn extract_line_number(&self, block: &Block) -> Option<u32> {
        block.metadata.as_ref()
            .and_then(|m| m.get("line_number"))
            .and_then(|v| v.as_u64())
            .map(|n| n as u32)
    }

    fn calculate_complexity_score(&self, block: &Block) -> f64 {
        // Extract complexity from stored metrics or calculate basic score
        if let Some(metrics) = &block.complexity_metrics {
            if let Some(score) = metrics.get("cyclomatic_complexity") {
                return score.as_f64().unwrap_or(1.0);
            }
        }
        
        // Fallback: estimate based on parameters and modifiers
        let param_count = block.parameters.as_ref()
            .and_then(|p| p.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0) as f64;
        
        let modifier_count = block.modifiers.as_ref()
            .map(|m| m.len())
            .unwrap_or(0) as f64;
        
        1.0 + (param_count * 0.5) + (modifier_count * 0.3)
    }

    fn is_recursive_function(&self, function_id: Uuid, nodes: &HashMap<Uuid, CallGraphNode>) -> bool {
        if let Some(_node) = nodes.get(&function_id) {
            self.has_path_to_self(function_id, function_id, nodes, &mut HashSet::new())
        } else {
            false
        }
    }

    fn has_path_to_self(&self, start: Uuid, target: Uuid, nodes: &HashMap<Uuid, CallGraphNode>, visited: &mut HashSet<Uuid>) -> bool {
        if visited.contains(&start) {
            return start == target;
        }
        
        visited.insert(start);
        
        if let Some(node) = nodes.get(&start) {
            for &callee in &node.calls {
                if callee == target || self.has_path_to_self(callee, target, nodes, visited) {
                    return true;
                }
            }
        }
        
        false
    }

    fn detect_cycles(&self, nodes: &HashMap<Uuid, CallGraphNode>) -> Vec<Vec<Uuid>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for &node_id in nodes.keys() {
            if !visited.contains(&node_id) {
                self.detect_cycle_dfs(node_id, nodes, &mut visited, &mut rec_stack, &mut Vec::new(), &mut cycles);
            }
        }

        cycles
    }

    fn detect_cycle_dfs(
        &self,
        node_id: Uuid,
        nodes: &HashMap<Uuid, CallGraphNode>,
        visited: &mut HashSet<Uuid>,
        rec_stack: &mut HashSet<Uuid>,
        path: &mut Vec<Uuid>,
        cycles: &mut Vec<Vec<Uuid>>,
    ) {
        visited.insert(node_id);
        rec_stack.insert(node_id);
        path.push(node_id);

        if let Some(node) = nodes.get(&node_id) {
            for &callee in &node.calls {
                if !visited.contains(&callee) {
                    self.detect_cycle_dfs(callee, nodes, visited, rec_stack, path, cycles);
                } else if rec_stack.contains(&callee) {
                    // Found a cycle
                    if let Some(cycle_start) = path.iter().position(|&id| id == callee) {
                        let cycle = path[cycle_start..].to_vec();
                        cycles.push(cycle);
                    }
                }
            }
        }

        path.pop();
        rec_stack.remove(&node_id);
    }

    fn calculate_max_depth(&self, nodes: &HashMap<Uuid, CallGraphNode>, entry_points: &[Uuid]) -> usize {
        let mut max_depth = 0;
        
        for &entry_id in entry_points {
            let depth = self.calculate_depth_from(entry_id, nodes, &mut HashSet::new());
            max_depth = max_depth.max(depth);
        }
        
        max_depth
    }

    fn calculate_depth_from(&self, node_id: Uuid, nodes: &HashMap<Uuid, CallGraphNode>, visited: &mut HashSet<Uuid>) -> usize {
        if visited.contains(&node_id) {
            return 0; // Avoid infinite recursion
        }
        
        visited.insert(node_id);
        
        let mut max_child_depth = 0;
        if let Some(node) = nodes.get(&node_id) {
            for &callee in &node.calls {
                let child_depth = self.calculate_depth_from(callee, nodes, visited);
                max_child_depth = max_child_depth.max(child_depth);
            }
        }
        
        visited.remove(&node_id);
        1 + max_child_depth
    }

    fn find_unreachable_functions(&self, nodes: &HashMap<Uuid, CallGraphNode>, entry_points: &[Uuid]) -> Vec<Uuid> {
        let mut reachable = HashSet::new();
        
        for &entry_id in entry_points {
            self.mark_reachable(entry_id, nodes, &mut reachable);
        }
        
        nodes.keys()
            .filter(|&&id| !reachable.contains(&id))
            .copied()
            .collect()
    }

    fn mark_reachable(&self, node_id: Uuid, nodes: &HashMap<Uuid, CallGraphNode>, reachable: &mut HashSet<Uuid>) {
        if reachable.contains(&node_id) {
            return;
        }
        
        reachable.insert(node_id);
        
        if let Some(node) = nodes.get(&node_id) {
            for &callee in &node.calls {
                self.mark_reachable(callee, nodes, reachable);
            }
        }
    }

    fn find_critical_paths(&self, nodes: &HashMap<Uuid, CallGraphNode>, entry_points: &[Uuid]) -> Vec<CallPath> {
        let mut critical_paths = Vec::new();
        
        for &entry_id in entry_points {
            let paths = self.find_paths_from(entry_id, nodes, 10); // Limit to top 10 paths
            critical_paths.extend(paths);
        }
        
        // Sort by complexity and return top paths
        critical_paths.sort_by(|a, b| b.total_complexity.partial_cmp(&a.total_complexity).unwrap());
        critical_paths.truncate(20);
        
        critical_paths
    }

    fn find_paths_from(&self, start_id: Uuid, nodes: &HashMap<Uuid, CallGraphNode>, max_paths: usize) -> Vec<CallPath> {
        let mut paths = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back((vec![start_id], 0.0, HashSet::new()));
        
        while let Some((path, complexity, visited)) = queue.pop_front() {
            if paths.len() >= max_paths {
                break;
            }
            
            let current_id = *path.last().unwrap();
            
            if let Some(node) = nodes.get(&current_id) {
                if node.calls.is_empty() {
                    // End of path
                    paths.push(CallPath {
                        path: path.clone(),
                        depth: path.len(),
                        has_cycles: visited.len() != path.len(),
                        total_complexity: complexity + node.complexity_score,
                    });
                } else {
                    // Continue exploring
                    for &callee in &node.calls {
                        if path.len() < 20 { // Prevent infinite paths
                            let mut new_path = path.clone();
                            let mut new_visited = visited.clone();
                            new_path.push(callee);
                            new_visited.insert(current_id);
                            
                            queue.push_back((
                                new_path,
                                complexity + node.complexity_score,
                                new_visited,
                            ));
                        }
                    }
                }
            }
        }
        
        paths
    }

    fn get_all_dependencies(&self, function_id: Uuid, nodes: &HashMap<Uuid, CallGraphNode>) -> Vec<Uuid> {
        let mut dependencies = Vec::new();
        let mut visited = HashSet::new();
        self.collect_dependencies(function_id, nodes, &mut dependencies, &mut visited);
        dependencies
    }

    fn collect_dependencies(&self, function_id: Uuid, nodes: &HashMap<Uuid, CallGraphNode>, dependencies: &mut Vec<Uuid>, visited: &mut HashSet<Uuid>) {
        if visited.contains(&function_id) {
            return;
        }
        visited.insert(function_id);

        if let Some(node) = nodes.get(&function_id) {
            for &callee in &node.calls {
                dependencies.push(callee);
                self.collect_dependencies(callee, nodes, dependencies, visited);
            }
        }
    }

    fn get_all_dependents(&self, function_id: Uuid, nodes: &HashMap<Uuid, CallGraphNode>) -> Vec<Uuid> {
        let mut dependents = Vec::new();
        let mut visited = HashSet::new();
        self.collect_dependents(function_id, nodes, &mut dependents, &mut visited);
        dependents
    }

    fn collect_dependents(&self, function_id: Uuid, nodes: &HashMap<Uuid, CallGraphNode>, dependents: &mut Vec<Uuid>, visited: &mut HashSet<Uuid>) {
        if visited.contains(&function_id) {
            return;
        }
        visited.insert(function_id);

        if let Some(node) = nodes.get(&function_id) {
            for &caller in &node.called_by {
                dependents.push(caller);
                self.collect_dependents(caller, nodes, dependents, visited);
            }
        }
    }
}

// These methods will be added to the Database implementation in schema.rs
