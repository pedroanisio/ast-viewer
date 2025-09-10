use serde::{Serialize, Deserialize};
use uuid::Uuid;
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use crate::database::{Database, Block};

/// Comprehensive property graph for semantic code analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodePropertyGraph {
    pub nodes: HashMap<Uuid, GraphNode>,
    pub edges: HashMap<Uuid, GraphEdge>,
    pub metadata: GraphMetadata,
    pub indices: GraphIndices,
}

/// Node in the property graph representing a code entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: Uuid,
    pub node_type: NodeType,
    pub properties: HashMap<String, PropertyValue>,
    pub labels: HashSet<String>,
    pub source_location: Option<SourceLocation>,
}

/// Edge in the property graph representing a relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: Uuid,
    pub source_id: Uuid,
    pub target_id: Uuid,
    pub edge_type: EdgeType,
    pub properties: HashMap<String, PropertyValue>,
    pub weight: f64,
    pub bidirectional: bool,
}

/// Types of nodes in the property graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum NodeType {
    // Code structure nodes
    Function,
    Class,
    Module,
    Interface,
    Struct,
    Enum,
    Variable,
    Parameter,
    Field,
    Method,
    
    // Type system nodes
    Type,
    GenericType,
    UnionType,
    
    // Control flow nodes
    IfStatement,
    LoopStatement,
    TryBlock,
    CatchBlock,
    
    // Data flow nodes
    Assignment,
    FunctionCall,
    MethodCall,
    Return,
    
    // Architectural nodes
    Package,
    Namespace,
    Component,
    Service,
    
    // Quality nodes
    Test,
    Documentation,
    Comment,
    
    // Security nodes
    AuthenticationPoint,
    AuthorizationCheck,
    InputValidation,
    OutputSanitization,
    
    // Performance nodes
    DatabaseQuery,
    NetworkCall,
    FileOperation,
    CacheAccess,
}

/// Types of edges in the property graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EdgeType {
    // Structural relationships
    Contains,
    Extends,
    Implements,
    Uses,
    Imports,
    
    // Call relationships
    Calls,
    CallsAsync,
    Invokes,
    
    // Data flow relationships
    Reads,
    Writes,
    Modifies,
    Passes,
    Returns,
    
    // Control flow relationships
    ControlsFlow,
    Branches,
    Loops,
    Throws,
    Catches,
    
    // Dependency relationships
    DependsOn,
    RequiredBy,
    OptionalDependency,
    
    // Type relationships
    HasType,
    IsInstanceOf,
    Casts,
    
    // Testing relationships
    Tests,
    TestedBy,
    Mocks,
    
    // Documentation relationships
    Documents,
    DocumentedBy,
    
    // Security relationships
    Authenticates,
    Authorizes,
    Validates,
    Sanitizes,
    
    // Performance relationships
    Optimizes,
    Caches,
    Indexes,
    
    // Temporal relationships
    Before,
    After,
    Concurrent,
}

/// Property values in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PropertyValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<PropertyValue>),
    Object(HashMap<String, PropertyValue>),
    Null,
}

/// Source location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file_path: String,
    pub line_start: u32,
    pub line_end: u32,
    pub column_start: u32,
    pub column_end: u32,
}

/// Graph metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub version: String,
    pub migration_id: Uuid,
    pub node_count: usize,
    pub edge_count: usize,
    pub languages: HashSet<String>,
    pub complexity_metrics: ComplexityMetrics,
}

/// Indices for fast graph queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphIndices {
    pub by_type: HashMap<NodeType, HashSet<Uuid>>,
    pub by_label: HashMap<String, HashSet<Uuid>>,
    pub by_property: HashMap<String, HashMap<String, HashSet<Uuid>>>,
    pub adjacency_list: HashMap<Uuid, Vec<Uuid>>,
    pub reverse_adjacency: HashMap<Uuid, Vec<Uuid>>,
}

/// Complexity metrics for the entire graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityMetrics {
    pub cyclomatic_complexity: f64,
    pub cognitive_complexity: f64,
    pub coupling_metrics: CouplingMetrics,
    pub cohesion_metrics: CohesionMetrics,
    pub maintainability_index: f64,
    pub technical_debt_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouplingMetrics {
    pub afferent_coupling: HashMap<Uuid, u32>,
    pub efferent_coupling: HashMap<Uuid, u32>,
    pub instability: HashMap<Uuid, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohesionMetrics {
    pub lcom: HashMap<Uuid, f64>, // Lack of Cohesion of Methods
    pub cohesion_score: HashMap<Uuid, f64>,
}

/// Query result from the property graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQueryResult {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub paths: Vec<GraphPath>,
    pub aggregations: HashMap<String, PropertyValue>,
    pub execution_time_ms: u64,
}

/// Path through the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphPath {
    pub nodes: Vec<Uuid>,
    pub edges: Vec<Uuid>,
    pub length: usize,
    pub weight: f64,
}

/// Property graph engine implementation
pub struct PropertyGraphEngine {
    db: Database,
    graph_cache: Option<CodePropertyGraph>,
}

impl PropertyGraphEngine {
    pub fn new(db: Database) -> Self {
        Self {
            db,
            graph_cache: None,
        }
    }

    /// Build comprehensive property graph from migration data
    pub async fn build_graph(&mut self, migration_id: Uuid) -> Result<CodePropertyGraph> {
        let start_time = std::time::Instant::now();
        
        // Get all blocks for this migration
        let blocks = self.db.get_blocks_by_migration(migration_id).await?;
        let relationships = self.db.get_relationships_by_migration(migration_id).await?;
        
        let mut graph = CodePropertyGraph {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            metadata: GraphMetadata {
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                version: "1.0".to_string(),
                migration_id,
                node_count: 0,
                edge_count: 0,
                languages: HashSet::new(),
                complexity_metrics: ComplexityMetrics {
                    cyclomatic_complexity: 0.0,
                    cognitive_complexity: 0.0,
                    coupling_metrics: CouplingMetrics {
                        afferent_coupling: HashMap::new(),
                        efferent_coupling: HashMap::new(),
                        instability: HashMap::new(),
                    },
                    cohesion_metrics: CohesionMetrics {
                        lcom: HashMap::new(),
                        cohesion_score: HashMap::new(),
                    },
                    maintainability_index: 0.0,
                    technical_debt_ratio: 0.0,
                },
            },
            indices: GraphIndices {
                by_type: HashMap::new(),
                by_label: HashMap::new(),
                by_property: HashMap::new(),
                adjacency_list: HashMap::new(),
                reverse_adjacency: HashMap::new(),
            },
        };

        // Convert blocks to nodes
        for block in blocks {
            let node = self.block_to_node(&block)?;
            self.add_node_to_graph(&mut graph, node);
        }

        // Convert relationships to edges
        for relationship in relationships {
            let edge = self.relationship_to_edge(&relationship)?;
            self.add_edge_to_graph(&mut graph, edge);
        }

        // Analyze and add derived relationships
        self.analyze_derived_relationships(&mut graph).await?;
        
        // Calculate complexity metrics
        self.calculate_complexity_metrics(&mut graph);
        
        // Build indices for fast querying
        self.build_indices(&mut graph);
        
        // Update metadata
        graph.metadata.node_count = graph.nodes.len();
        graph.metadata.edge_count = graph.edges.len();
        graph.metadata.updated_at = chrono::Utc::now();
        
        let build_time = start_time.elapsed();
        println!("Built property graph in {:?}: {} nodes, {} edges", 
                 build_time, graph.nodes.len(), graph.edges.len());

        // Cache the graph
        self.graph_cache = Some(graph.clone());
        
        Ok(graph)
    }

    /// Execute Cypher-like queries on the property graph
    pub async fn query(&self, query: &str) -> Result<GraphQueryResult> {
        let start_time = std::time::Instant::now();
        
        let graph = self.graph_cache.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Graph not built. Call build_graph() first."))?;

        // Parse and execute the query
        let result = self.execute_query(graph, query).await?;
        
        let execution_time = start_time.elapsed().as_millis() as u64;
        
        Ok(GraphQueryResult {
            nodes: result.nodes,
            edges: result.edges,
            paths: result.paths,
            aggregations: result.aggregations,
            execution_time_ms: execution_time,
        })
    }

    /// Find all paths between two nodes
    pub fn find_paths(&self, from: Uuid, to: Uuid, max_depth: usize) -> Result<Vec<GraphPath>> {
        let graph = self.graph_cache.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Graph not built"))?;

        let mut paths = Vec::new();
        let mut visited = HashSet::new();
        let mut current_path = Vec::new();
        
        self.dfs_paths(graph, from, to, max_depth, &mut visited, &mut current_path, &mut paths);
        
        Ok(paths)
    }

    /// Find nodes by pattern matching
    pub fn find_nodes_by_pattern(&self, pattern: &NodePattern) -> Result<Vec<GraphNode>> {
        let graph = self.graph_cache.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Graph not built"))?;

        let mut matching_nodes = Vec::new();
        
        for node in graph.nodes.values() {
            if self.matches_pattern(node, pattern) {
                matching_nodes.push(node.clone());
            }
        }
        
        Ok(matching_nodes)
    }

    /// Analyze security vulnerabilities in the graph
    pub fn analyze_security_vulnerabilities(&self) -> Result<Vec<SecurityVulnerability>> {
        let graph = self.graph_cache.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Graph not built"))?;

        let mut vulnerabilities = Vec::new();

        // Find unvalidated inputs
        for node in graph.nodes.values() {
            if node.node_type == NodeType::Parameter || node.node_type == NodeType::Variable {
                if !self.has_validation_path(graph, node.id) {
                    vulnerabilities.push(SecurityVulnerability {
                        vulnerability_type: VulnerabilityType::UnvalidatedInput,
                        node_id: node.id,
                        severity: Severity::High,
                        description: "Input parameter lacks validation".to_string(),
                        recommendation: "Add input validation before use".to_string(),
                    });
                }
            }
        }

        // Find SQL injection risks
        for node in graph.nodes.values() {
            if node.node_type == NodeType::DatabaseQuery {
                if self.has_dynamic_query_construction(graph, node.id) {
                    vulnerabilities.push(SecurityVulnerability {
                        vulnerability_type: VulnerabilityType::SqlInjection,
                        node_id: node.id,
                        severity: Severity::Critical,
                        description: "Dynamic SQL query construction detected".to_string(),
                        recommendation: "Use parameterized queries or ORM".to_string(),
                    });
                }
            }
        }

        Ok(vulnerabilities)
    }

    /// Analyze performance bottlenecks
    pub fn analyze_performance_bottlenecks(&self) -> Result<Vec<PerformanceIssue>> {
        let graph = self.graph_cache.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Graph not built"))?;

        let mut issues = Vec::new();

        // Find N+1 query problems
        for node in graph.nodes.values() {
            if node.node_type == NodeType::LoopStatement {
                if self.has_database_query_in_loop(graph, node.id) {
                    issues.push(PerformanceIssue {
                        issue_type: PerformanceIssueType::NPlus1Query,
                        node_id: node.id,
                        severity: Severity::High,
                        description: "Database query inside loop detected".to_string(),
                        recommendation: "Use batch queries or eager loading".to_string(),
                        estimated_impact: ImpactLevel::High,
                    });
                }
            }
        }

        // Find uncached expensive operations
        for node in graph.nodes.values() {
            if matches!(node.node_type, NodeType::NetworkCall | NodeType::FileOperation) {
                if !self.has_caching(graph, node.id) {
                    issues.push(PerformanceIssue {
                        issue_type: PerformanceIssueType::UncachedOperation,
                        node_id: node.id,
                        severity: Severity::Medium,
                        description: "Expensive operation without caching".to_string(),
                        recommendation: "Add caching layer".to_string(),
                        estimated_impact: ImpactLevel::Medium,
                    });
                }
            }
        }

        Ok(issues)
    }

    // Helper methods
    fn block_to_node(&self, block: &Block) -> Result<GraphNode> {
        let node_type = match block.block_type.as_str() {
            "Function" => NodeType::Function,
            "Class" => NodeType::Class,
            "Module" => NodeType::Module,
            "Variable" => NodeType::Variable,
            _ => NodeType::Function, // Default
        };

        let mut properties = HashMap::new();
        properties.insert("name".to_string(), PropertyValue::String(
            block.semantic_name.clone().unwrap_or_else(|| "unnamed".to_string())
        ));
        properties.insert("complexity".to_string(), PropertyValue::Integer(
            block.metadata.as_ref()
                .and_then(|m| m.get("cyclomatic_complexity"))
                .and_then(|v| v.as_i64())
                .unwrap_or(1)
        ));

        let mut labels = HashSet::new();
        labels.insert(block.block_type.clone());
        if let Some(metadata) = &block.metadata {
            if let Some(tested) = metadata.get("has_tests").and_then(|v| v.as_bool()) {
                if tested {
                    labels.insert("tested".to_string());
                } else {
                    labels.insert("untested".to_string());
                }
            }
        }

        Ok(GraphNode {
            id: block.id,
            node_type,
            properties,
            labels,
            source_location: Some(SourceLocation {
                file_path: "unknown".to_string(), // Would be populated from container
                line_start: block.position as u32,
                line_end: block.position as u32,
                column_start: 0,
                column_end: 0,
            }),
        })
    }

    fn relationship_to_edge(&self, relationship: &crate::database::schema::BlockRelationship) -> Result<GraphEdge> {
        let edge_type = match relationship.relationship_type.as_str() {
            "calls" => EdgeType::Calls,
            "uses" => EdgeType::Uses,
            "extends" => EdgeType::Extends,
            "implements" => EdgeType::Implements,
            "contains" => EdgeType::Contains,
            _ => EdgeType::Uses, // Default
        };

        Ok(GraphEdge {
            id: Uuid::new_v4(),
            source_id: relationship.source_block_id,
            target_id: relationship.target_block_id,
            edge_type,
            properties: HashMap::new(),
            weight: 1.0,
            bidirectional: false,
        })
    }

    fn add_node_to_graph(&self, graph: &mut CodePropertyGraph, node: GraphNode) {
        // Add to main collection
        graph.nodes.insert(node.id, node.clone());
        
        // Update indices
        graph.indices.by_type
            .entry(node.node_type.clone())
            .or_insert_with(HashSet::new)
            .insert(node.id);
            
        for label in &node.labels {
            graph.indices.by_label
                .entry(label.clone())
                .or_insert_with(HashSet::new)
                .insert(node.id);
        }
    }

    fn add_edge_to_graph(&self, graph: &mut CodePropertyGraph, edge: GraphEdge) {
        // Add to main collection
        graph.edges.insert(edge.id, edge.clone());
        
        // Update adjacency lists
        graph.indices.adjacency_list
            .entry(edge.source_id)
            .or_insert_with(Vec::new)
            .push(edge.target_id);
            
        graph.indices.reverse_adjacency
            .entry(edge.target_id)
            .or_insert_with(Vec::new)
            .push(edge.source_id);
    }

    async fn analyze_derived_relationships(&self, _graph: &mut CodePropertyGraph) -> Result<()> {
        // TODO: Implement analysis of derived relationships
        // - Transitive dependencies
        // - Implicit relationships
        // - Pattern-based relationships
        Ok(())
    }

    fn calculate_complexity_metrics(&self, _graph: &mut CodePropertyGraph) {
        // TODO: Implement complexity calculations
        // - Cyclomatic complexity
        // - Cognitive complexity
        // - Coupling metrics
        // - Cohesion metrics
    }

    fn build_indices(&self, _graph: &mut CodePropertyGraph) {
        // Indices are built incrementally in add_node_to_graph and add_edge_to_graph
    }

    async fn execute_query(&self, _graph: &CodePropertyGraph, _query: &str) -> Result<GraphQueryResult> {
        // TODO: Implement query parser and executor
        // For now, return empty result
        Ok(GraphQueryResult {
            nodes: vec![],
            edges: vec![],
            paths: vec![],
            aggregations: HashMap::new(),
            execution_time_ms: 0,
        })
    }

    fn dfs_paths(
        &self,
        graph: &CodePropertyGraph,
        current: Uuid,
        target: Uuid,
        max_depth: usize,
        visited: &mut HashSet<Uuid>,
        current_path: &mut Vec<Uuid>,
        paths: &mut Vec<GraphPath>,
    ) {
        if current_path.len() >= max_depth {
            return;
        }

        visited.insert(current);
        current_path.push(current);

        if current == target {
            paths.push(GraphPath {
                nodes: current_path.clone(),
                edges: vec![], // TODO: Collect edge IDs
                length: current_path.len(),
                weight: 1.0,
            });
        } else {
            if let Some(neighbors) = graph.indices.adjacency_list.get(&current) {
                for &neighbor in neighbors {
                    if !visited.contains(&neighbor) {
                        self.dfs_paths(graph, neighbor, target, max_depth, visited, current_path, paths);
                    }
                }
            }
        }

        current_path.pop();
        visited.remove(&current);
    }

    fn matches_pattern(&self, _node: &GraphNode, _pattern: &NodePattern) -> bool {
        // TODO: Implement pattern matching
        true
    }

    fn has_validation_path(&self, _graph: &CodePropertyGraph, _node_id: Uuid) -> bool {
        // TODO: Check if there's a path to validation
        false
    }

    fn has_dynamic_query_construction(&self, _graph: &CodePropertyGraph, _node_id: Uuid) -> bool {
        // TODO: Check for dynamic SQL construction
        false
    }

    fn has_database_query_in_loop(&self, _graph: &CodePropertyGraph, _node_id: Uuid) -> bool {
        // TODO: Check for database queries in loops
        false
    }

    fn has_caching(&self, _graph: &CodePropertyGraph, _node_id: Uuid) -> bool {
        // TODO: Check for caching mechanisms
        false
    }
}

// Supporting types for analysis
#[derive(Debug, Clone)]
pub struct NodePattern {
    pub node_type: Option<NodeType>,
    pub labels: Vec<String>,
    pub properties: HashMap<String, PropertyValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityVulnerability {
    pub vulnerability_type: VulnerabilityType,
    pub node_id: Uuid,
    pub severity: Severity,
    pub description: String,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VulnerabilityType {
    UnvalidatedInput,
    SqlInjection,
    XssVulnerability,
    InsecureDeserialization,
    WeakCryptography,
    AuthenticationBypass,
    AuthorizationFlaws,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceIssue {
    pub issue_type: PerformanceIssueType,
    pub node_id: Uuid,
    pub severity: Severity,
    pub description: String,
    pub recommendation: String,
    pub estimated_impact: ImpactLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceIssueType {
    NPlus1Query,
    UncachedOperation,
    SynchronousBlocking,
    MemoryLeak,
    InefficiientAlgorithm,
    UnindexedQuery,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpactLevel {
    Low,
    Medium,
    High,
    Critical,
}
