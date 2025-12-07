use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use anyhow::Result;
use serde::{Serialize, Deserialize};
use crate::database::{Database, Container};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDependency {
    pub source_file: String,
    pub target_file: String,
    pub dependency_type: DependencyType,
    pub import_statement: Option<String>,
    pub symbols_used: Vec<String>,
    pub is_circular: bool,
    pub strength: f64, // 0.0 to 1.0, based on usage frequency
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DependencyType {
    Import,
    Require,
    Include,
    Use,
    From,
    Inheritance,
    Composition,
    FunctionCall,
    TypeReference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub files: HashMap<String, FileNode>,
    pub dependencies: Vec<FileDependency>,
    pub circular_dependencies: Vec<Vec<String>>,
    pub dependency_layers: Vec<Vec<String>>,
    pub orphaned_files: Vec<String>,
    pub entry_points: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    pub file_path: String,
    pub container_id: Uuid,
    pub language: String,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub functions: Vec<String>,
    pub classes: Vec<String>,
    pub variables: Vec<String>,
    pub complexity_score: f64,
    pub lines_of_code: usize,
    pub dependencies_count: usize,
    pub dependents_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAnalysis {
    pub affected_files: Vec<String>,
    pub impact_score: f64,
    pub change_propagation: Vec<ChangePropagation>,
    pub test_files_affected: Vec<String>,
    pub build_order_changes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePropagation {
    pub file_path: String,
    pub change_type: ChangeType,
    pub confidence: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    RequiresUpdate,
    RequiresTesting,
    RequiresRecompilation,
    RequiresDocumentation,
    MayBreak,
}

pub struct DependencyTracker {
    db: Database,
}

impl DependencyTracker {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn analyze_repository(&self, migration_id: Uuid) -> Result<DependencyGraph> {
        let containers = self.db.get_containers_by_migration(migration_id).await?;
        
        let mut files = HashMap::new();
        let mut dependencies = Vec::new();

        // Build file nodes
        for container in &containers {
            let file_node = self.build_file_node(container).await?;
            files.insert(container.original_path.clone().unwrap_or_else(|| container.name.clone()), file_node);
        }

        // Analyze dependencies between files
        for container in &containers {
            let file_deps = self.analyze_file_dependencies(container, &containers).await?;
            dependencies.extend(file_deps);
        }

        // Detect circular dependencies
        let circular_dependencies = self.detect_circular_dependencies(&files, &dependencies);

        // Calculate dependency layers (topological sort)
        let dependency_layers = self.calculate_dependency_layers(&files, &dependencies);

        // Find orphaned files and entry points
        let (orphaned_files, entry_points) = self.find_orphaned_and_entry_files(&files, &dependencies);

        // Update dependency counts
        let mut files = files;
        for file_node in files.values_mut() {
            file_node.dependencies_count = dependencies.iter()
                .filter(|d| d.source_file == file_node.file_path)
                .count();
            file_node.dependents_count = dependencies.iter()
                .filter(|d| d.target_file == file_node.file_path)
                .count();
        }

        Ok(DependencyGraph {
            files,
            dependencies,
            circular_dependencies,
            dependency_layers,
            orphaned_files,
            entry_points,
        })
    }

    pub async fn analyze_impact(&self, file_path: &str, migration_id: Uuid) -> Result<ImpactAnalysis> {
        let dependency_graph = self.analyze_repository(migration_id).await?;
        
        let mut affected_files = Vec::new();
        let mut visited = HashSet::new();
        
        // Find all files that depend on the changed file
        self.find_dependent_files(file_path, &dependency_graph, &mut affected_files, &mut visited);
        
        // Calculate impact score based on number of affected files and their importance
        let impact_score = self.calculate_impact_score(&affected_files, &dependency_graph);
        
        // Analyze change propagation
        let change_propagation = self.analyze_change_propagation(&affected_files, &dependency_graph);
        
        // Find affected test files
        let test_files_affected = affected_files.iter()
            .filter(|f| self.is_test_file(f))
            .cloned()
            .collect();
        
        // Determine build order changes
        let build_order_changes = self.calculate_build_order_changes(&affected_files, &dependency_graph);
        
        Ok(ImpactAnalysis {
            affected_files,
            impact_score,
            change_propagation,
            test_files_affected,
            build_order_changes,
        })
    }

    pub async fn suggest_refactoring_opportunities(&self, migration_id: Uuid) -> Result<Vec<RefactoringOpportunity>> {
        let dependency_graph = self.analyze_repository(migration_id).await?;
        let mut opportunities = Vec::new();

        // Detect circular dependencies
        for cycle in &dependency_graph.circular_dependencies {
            opportunities.push(RefactoringOpportunity {
                opportunity_type: RefactoringType::BreakCircularDependency,
                files_involved: cycle.clone(),
                description: format!("Break circular dependency between {} files", cycle.len()),
                impact_score: 0.8,
                effort_estimate: EstimateEffort::Medium,
                benefits: vec![
                    "Improved modularity".to_string(),
                    "Easier testing".to_string(),
                    "Reduced coupling".to_string(),
                ],
            });
        }

        // Detect highly coupled files
        let highly_coupled = self.find_highly_coupled_files(&dependency_graph);
        for (file1, file2, coupling_score) in highly_coupled {
            opportunities.push(RefactoringOpportunity {
                opportunity_type: RefactoringType::ReduceCoupling,
                files_involved: vec![file1.clone(), file2.clone()],
                description: format!("Reduce coupling between {} and {}", file1, file2),
                impact_score: coupling_score,
                effort_estimate: EstimateEffort::Medium,
                benefits: vec![
                    "Improved maintainability".to_string(),
                    "Better separation of concerns".to_string(),
                ],
            });
        }

        // Detect large files that could be split
        for (file_path, node) in &dependency_graph.files {
            if node.lines_of_code > 500 && (node.functions.len() > 20 || node.classes.len() > 10) {
                opportunities.push(RefactoringOpportunity {
                    opportunity_type: RefactoringType::SplitLargeFile,
                    files_involved: vec![file_path.clone()],
                    description: format!("Split large file {} ({} LOC)", file_path, node.lines_of_code),
                    impact_score: 0.6,
                    effort_estimate: EstimateEffort::High,
                    benefits: vec![
                        "Improved readability".to_string(),
                        "Better organization".to_string(),
                        "Easier maintenance".to_string(),
                    ],
                });
            }
        }

        // Sort by impact score
        opportunities.sort_by(|a, b| b.impact_score.partial_cmp(&a.impact_score).unwrap());
        
        Ok(opportunities)
    }

    pub fn generate_dependency_matrix(&self, dependency_graph: &DependencyGraph) -> String {
        let files: Vec<_> = dependency_graph.files.keys().collect();
        let mut matrix = String::new();
        
        // Header
        matrix.push_str("Dependency Matrix:\n");
        matrix.push_str("Rows depend on Columns\n\n");
        
        // Column headers
        matrix.push_str("     ");
        for (i, _) in files.iter().enumerate() {
            matrix.push_str(&format!("{:3} ", i));
        }
        matrix.push('\n');
        
        // Rows
        for (i, file1) in files.iter().enumerate() {
            matrix.push_str(&format!("{:3}: ", i));
            for file2 in &files {
                let has_dependency = dependency_graph.dependencies.iter()
                    .any(|d| d.source_file == **file1 && d.target_file == **file2);
                matrix.push_str(if has_dependency { " X  " } else { " .  " });
            }
            matrix.push_str(&format!(" {}\n", file1));
        }
        
        matrix
    }

    async fn build_file_node(&self, container: &Container) -> Result<FileNode> {
        let blocks = self.db.get_blocks_by_container(container.id).await?;
        
        let functions: Vec<_> = blocks.iter()
            .filter(|b| b.block_type == "Function")
            .filter_map(|b| b.semantic_name.clone())
            .collect();
        
        let classes: Vec<_> = blocks.iter()
            .filter(|b| b.block_type == "Class")
            .filter_map(|b| b.semantic_name.clone())
            .collect();
        
        let variables: Vec<_> = blocks.iter()
            .filter(|b| b.block_type == "Variable")
            .filter_map(|b| b.semantic_name.clone())
            .collect();
        
        let imports: Vec<_> = blocks.iter()
            .filter(|b| b.block_type == "Import")
            .filter_map(|b| b.semantic_name.clone())
            .collect();
        
        // Calculate complexity score
        let complexity_score = blocks.iter()
            .filter_map(|b| b.complexity_metrics.as_ref())
            .filter_map(|m| m.get("cyclomatic_complexity"))
            .filter_map(|v| v.as_f64())
            .sum::<f64>() / blocks.len().max(1) as f64;
        
        // Estimate lines of code
        let lines_of_code = container.source_code.as_ref()
            .map(|s| s.lines().count())
            .unwrap_or(0);
        
        Ok(FileNode {
            file_path: container.original_path.clone().unwrap_or_else(|| container.name.clone()),
            container_id: container.id,
            language: container.language.clone().unwrap_or_else(|| "unknown".to_string()),
            imports,
            exports: Vec::new(), // TODO: Extract exports
            functions,
            classes,
            variables,
            complexity_score,
            lines_of_code,
            dependencies_count: 0, // Will be calculated later
            dependents_count: 0,   // Will be calculated later
        })
    }

    async fn analyze_file_dependencies(&self, container: &Container, all_containers: &[Container]) -> Result<Vec<FileDependency>> {
        let blocks = self.db.get_blocks_by_container(container.id).await?;
        let relationships = self.db.get_relationships_by_container(container.id).await?;
        
        let mut dependencies = Vec::new();
        let source_file = container.original_path.clone().unwrap_or_else(|| container.name.clone());
        
        // Analyze import blocks
        for block in blocks.iter().filter(|b| b.block_type == "Import") {
            if let Some(import_name) = &block.semantic_name {
                // Try to resolve import to actual file
                if let Some(target_file) = self.resolve_import_to_file(import_name, all_containers) {
                    dependencies.push(FileDependency {
                        source_file: source_file.clone(),
                        target_file,
                        dependency_type: DependencyType::Import,
                        import_statement: Some(import_name.clone()),
                        symbols_used: vec![import_name.clone()],
                        is_circular: false, // Will be determined later
                        strength: 1.0,
                    });
                }
            }
        }
        
        // Analyze function call relationships that cross file boundaries
        for relationship in relationships {
            if relationship.relationship_type == "Calls" {
                // Check if the target is in a different file
                if let Ok(target_container_id) = self.db.get_container_id_by_block(relationship.target_block_id).await {
                    if target_container_id != container.id {
                        if let Ok(target_container) = self.db.get_container_by_id(target_container_id).await {
                            let target_file = target_container.original_path.unwrap_or_else(|| target_container.name);
                            
                            dependencies.push(FileDependency {
                                source_file: source_file.clone(),
                                target_file,
                                dependency_type: DependencyType::FunctionCall,
                                import_statement: None,
                                symbols_used: Vec::new(),
                                is_circular: false,
                                strength: 0.7,
                            });
                        }
                    }
                }
            }
        }
        
        Ok(dependencies)
    }

    fn resolve_import_to_file(&self, import_name: &str, containers: &[Container]) -> Option<String> {
        // Simple heuristic: try to match import name to file names
        for container in containers {
            let file_path = container.original_path.as_ref().unwrap_or(&container.name);
            
            // Extract filename without extension
            if let Some(filename) = std::path::Path::new(file_path).file_stem() {
                if let Some(filename_str) = filename.to_str() {
                    if import_name.contains(filename_str) || filename_str.contains(import_name) {
                        return Some(file_path.clone());
                    }
                }
            }
        }
        
        None
    }

    fn detect_circular_dependencies(&self, files: &HashMap<String, FileNode>, dependencies: &[FileDependency]) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        
        for file_path in files.keys() {
            if !visited.contains(file_path) {
                self.detect_cycle_dfs(
                    file_path,
                    dependencies,
                    &mut visited,
                    &mut rec_stack,
                    &mut Vec::new(),
                    &mut cycles,
                );
            }
        }
        
        cycles
    }

    fn detect_cycle_dfs(
        &self,
        file_path: &str,
        dependencies: &[FileDependency],
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        visited.insert(file_path.to_string());
        rec_stack.insert(file_path.to_string());
        path.push(file_path.to_string());

        // Find all files this file depends on
        for dep in dependencies.iter().filter(|d| d.source_file == file_path) {
            if !visited.contains(&dep.target_file) {
                self.detect_cycle_dfs(&dep.target_file, dependencies, visited, rec_stack, path, cycles);
            } else if rec_stack.contains(&dep.target_file) {
                // Found a cycle
                if let Some(cycle_start) = path.iter().position(|f| f == &dep.target_file) {
                    let cycle = path[cycle_start..].to_vec();
                    cycles.push(cycle);
                }
            }
        }

        path.pop();
        rec_stack.remove(file_path);
    }

    fn calculate_dependency_layers(&self, files: &HashMap<String, FileNode>, dependencies: &[FileDependency]) -> Vec<Vec<String>> {
        let mut layers = Vec::new();
        let mut remaining_files: HashSet<_> = files.keys().cloned().collect();
        
        while !remaining_files.is_empty() {
            let mut current_layer = Vec::new();
            
            // Find files with no dependencies to remaining files
            for file in &remaining_files {
                let has_dependencies = dependencies.iter()
                    .any(|d| d.source_file == *file && remaining_files.contains(&d.target_file));
                
                if !has_dependencies {
                    current_layer.push(file.clone());
                }
            }
            
            if current_layer.is_empty() {
                // Circular dependency - add all remaining files
                current_layer.extend(remaining_files.iter().cloned());
            }
            
            for file in &current_layer {
                remaining_files.remove(file);
            }
            
            layers.push(current_layer);
        }
        
        layers
    }

    fn find_orphaned_and_entry_files(&self, files: &HashMap<String, FileNode>, dependencies: &[FileDependency]) -> (Vec<String>, Vec<String>) {
        let mut orphaned = Vec::new();
        let mut entry_points = Vec::new();
        
        for file_path in files.keys() {
            let has_dependents = dependencies.iter().any(|d| d.target_file == *file_path);
            let has_dependencies = dependencies.iter().any(|d| d.source_file == *file_path);
            
            if !has_dependents && !has_dependencies {
                orphaned.push(file_path.clone());
            } else if !has_dependencies {
                entry_points.push(file_path.clone());
            }
        }
        
        (orphaned, entry_points)
    }

    fn find_dependent_files(&self, file_path: &str, graph: &DependencyGraph, affected: &mut Vec<String>, visited: &mut HashSet<String>) {
        if visited.contains(file_path) {
            return;
        }
        visited.insert(file_path.to_string());
        
        for dep in &graph.dependencies {
            if dep.target_file == file_path {
                affected.push(dep.source_file.clone());
                self.find_dependent_files(&dep.source_file, graph, affected, visited);
            }
        }
    }

    fn calculate_impact_score(&self, affected_files: &[String], graph: &DependencyGraph) -> f64 {
        let total_files = graph.files.len() as f64;
        let affected_count = affected_files.len() as f64;
        
        // Base score from percentage of files affected
        let base_score = affected_count / total_files;
        
        // Weight by complexity of affected files
        let complexity_weight = affected_files.iter()
            .filter_map(|f| graph.files.get(f))
            .map(|node| node.complexity_score)
            .sum::<f64>() / affected_files.len().max(1) as f64;
        
        (base_score + complexity_weight * 0.3).min(1.0)
    }

    fn analyze_change_propagation(&self, affected_files: &[String], graph: &DependencyGraph) -> Vec<ChangePropagation> {
        let mut propagations = Vec::new();
        
        for file_path in affected_files {
            if let Some(node) = graph.files.get(file_path) {
                let change_type = if self.is_test_file(file_path) {
                    ChangeType::RequiresTesting
                } else if node.dependents_count > 5 {
                    ChangeType::MayBreak
                } else {
                    ChangeType::RequiresUpdate
                };
                
                propagations.push(ChangePropagation {
                    file_path: file_path.clone(),
                    change_type,
                    confidence: 0.8,
                    reason: format!("File has {} dependents", node.dependents_count),
                });
            }
        }
        
        propagations
    }

    fn is_test_file(&self, file_path: &str) -> bool {
        file_path.contains("test") || file_path.contains("spec") || file_path.ends_with("_test.py") || file_path.ends_with(".test.js")
    }

    fn calculate_build_order_changes(&self, affected_files: &[String], graph: &DependencyGraph) -> Vec<String> {
        // Simple implementation: return files in dependency order
        let mut build_order = Vec::new();
        
        for layer in &graph.dependency_layers {
            for file in layer {
                if affected_files.contains(file) {
                    build_order.push(file.clone());
                }
            }
        }
        
        build_order
    }

    fn find_highly_coupled_files(&self, graph: &DependencyGraph) -> Vec<(String, String, f64)> {
        let mut coupled_pairs = Vec::new();
        
        let files: Vec<_> = graph.files.keys().collect();
        for i in 0..files.len() {
            for j in i + 1..files.len() {
                let file1 = files[i];
                let file2 = files[j];
                
                let coupling_score = self.calculate_coupling_score(file1, file2, graph);
                if coupling_score > 0.7 {
                    coupled_pairs.push((file1.clone(), file2.clone(), coupling_score));
                }
            }
        }
        
        coupled_pairs.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        coupled_pairs
    }

    fn calculate_coupling_score(&self, file1: &str, file2: &str, graph: &DependencyGraph) -> f64 {
        let deps_1_to_2 = graph.dependencies.iter()
            .filter(|d| d.source_file == file1 && d.target_file == file2)
            .count();
        
        let deps_2_to_1 = graph.dependencies.iter()
            .filter(|d| d.source_file == file2 && d.target_file == file1)
            .count();
        
        let total_deps = deps_1_to_2 + deps_2_to_1;
        
        // Normalize by file sizes
        let file1_size = graph.files.get(file1).map(|n| n.lines_of_code).unwrap_or(1);
        let file2_size = graph.files.get(file2).map(|n| n.lines_of_code).unwrap_or(1);
        let avg_size = (file1_size + file2_size) as f64 / 2.0;
        
        (total_deps as f64 / avg_size * 100.0).min(1.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringOpportunity {
    pub opportunity_type: RefactoringType,
    pub files_involved: Vec<String>,
    pub description: String,
    pub impact_score: f64,
    pub effort_estimate: EstimateEffort,
    pub benefits: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RefactoringType {
    BreakCircularDependency,
    ReduceCoupling,
    SplitLargeFile,
    ExtractCommonCode,
    MergeSmallFiles,
    MoveToAppropriateModule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EstimateEffort {
    Low,
    Medium,
    High,
}
