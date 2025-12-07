use async_graphql::*;

/// GraphQL-compatible semantic block representation
#[derive(SimpleObject, Clone, Debug)]
pub struct GqlSemanticBlock {
    pub id: ID,
    pub block_type: String,
    pub semantic_name: Option<String>,
    pub source_language: String,
    pub abstract_syntax: String, // JSON as string for GraphQL
    pub position: i32,
    pub indent_level: i32,
    pub metadata: Option<String>, // JSON as string
    pub parent_block_id: Option<ID>,
    pub position_in_parent: i32,
    pub parameters: Option<String>, // JSON as string
    pub return_type: Option<String>,
    pub modifiers: Vec<String>,
}

/// Pattern matching for semantic search
#[derive(InputObject, Clone, Debug)]
pub struct CodePattern {
    /// Pattern type (function, class, async_function, etc.)
    pub pattern_type: String,
    /// Language to search in
    pub language: Option<String>,
    /// Specific attributes to match
    pub attributes: Option<String>, // JSON string
    /// Minimum complexity threshold
    pub min_complexity: Option<i32>,
    /// Maximum complexity threshold  
    pub max_complexity: Option<i32>,
}

/// Result of pattern matching
#[derive(SimpleObject, Clone, Debug)]
pub struct PatternMatch {
    pub block: GqlSemanticBlock,
    pub confidence: f64,
    pub matched_attributes: Vec<String>,
}

/// Dependency graph representation
#[derive(SimpleObject, Clone, Debug)]
pub struct DependencyGraph {
    pub nodes: Vec<DependencyNode>,
    pub edges: Vec<DependencyEdge>,
    pub metrics: DependencyMetrics,
}

#[derive(SimpleObject, Clone, Debug)]
pub struct DependencyNode {
    pub id: ID,
    pub name: String,
    pub node_type: String,
    pub language: String,
    pub complexity: Option<i32>,
}

#[derive(SimpleObject, Clone, Debug)]
pub struct DependencyEdge {
    pub source: ID,
    pub target: ID,
    pub relationship_type: String,
    pub weight: f64,
}

#[derive(SimpleObject, Clone, Debug)]
pub struct DependencyMetrics {
    pub total_nodes: i32,
    pub total_edges: i32,
    pub cyclic_dependencies: i32,
    pub max_depth: i32,
    pub average_complexity: f64,
}

/// Refactoring patterns for AI agents
#[derive(InputObject, Clone, Debug)]
pub struct RefactoringPattern {
    pub pattern_type: RefactoringType,
    pub target_blocks: Vec<ID>,
    pub parameters: Option<String>, // JSON parameters
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum RefactoringType {
    ExtractFunction,
    InlineVariable,
    RenameSymbol,
    MoveMethod,
    ExtractClass,
    InlineMethod,
}

/// Scope for refactoring operations
#[derive(InputObject, Clone, Debug)]
pub struct RefactoringScope {
    pub container_ids: Option<Vec<ID>>,
    pub language: Option<String>,
    pub include_tests: bool,
}

/// Result of refactoring operation
#[derive(SimpleObject, Clone, Debug)]
pub struct RefactoringResult {
    pub success: bool,
    pub modified_blocks: Vec<ID>,
    pub new_blocks: Vec<GqlSemanticBlock>,
    pub removed_blocks: Vec<ID>,
    pub generated_code: Option<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

/// Specification for synthesizing new blocks
#[derive(InputObject, Clone, Debug)]
pub struct BlockSpecification {
    pub block_type: String,
    pub name: String,
    pub language: String,
    pub description: String,
    pub parameters: Option<String>, // JSON string
    pub return_type: Option<String>,
    pub constraints: Option<String>, // JSON string
}

/// Search filters for semantic queries
#[derive(InputObject, Clone, Debug)]
pub struct SearchFilters {
    pub languages: Option<Vec<String>>,
    pub block_types: Option<Vec<String>>,
    pub has_tests: Option<bool>,
    pub min_complexity: Option<i32>,
    pub max_complexity: Option<i32>,
    pub has_documentation: Option<bool>,
}

/// Relationship analysis result
#[derive(SimpleObject, Clone, Debug)]
pub struct RelationshipAnalysis {
    pub source_block: GqlSemanticBlock,
    pub target_block: GqlSemanticBlock,
    pub relationship_type: String,
    pub strength: f64,
    pub bidirectional: bool,
    pub metadata: Option<String>,
}

/// Code quality metrics
#[derive(SimpleObject, Clone, Debug)]
pub struct QualityMetrics {
    pub cyclomatic_complexity: Option<i32>,
    pub cognitive_complexity: Option<i32>,
    pub lines_of_code: i32,
    pub test_coverage: Option<f64>,
    pub maintainability_index: Option<f64>,
    pub technical_debt_ratio: Option<f64>,
}

/// Container information for GraphQL
#[derive(SimpleObject, Clone, Debug)]
pub struct GqlContainer {
    pub id: ID,
    pub name: String,
    pub container_type: String,
    pub language: Option<String>,
    pub original_path: Option<String>,
    pub block_count: i32,
    pub quality_metrics: Option<QualityMetrics>,
}

// Helper functions for type conversion
impl From<crate::database::Block> for GqlSemanticBlock {
    fn from(block: crate::database::Block) -> Self {
        Self {
            id: ID::from(block.id.to_string()),
            block_type: block.block_type,
            semantic_name: block.semantic_name,
            source_language: "unknown".to_string(), // Will be populated from container
            abstract_syntax: block.abstract_syntax.to_string(),
            position: block.position,
            indent_level: block.indent_level,
            metadata: block.metadata.map(|m| m.to_string()),
            parent_block_id: block.parent_block_id.map(|id| ID::from(id.to_string())),
            position_in_parent: block.position_in_parent,
            parameters: block.parameters.map(|p| p.to_string()),
            return_type: block.return_type,
            modifiers: block.modifiers.unwrap_or_default(),
        }
    }
}

impl From<crate::database::Container> for GqlContainer {
    fn from(container: crate::database::Container) -> Self {
        Self {
            id: ID::from(container.id.to_string()),
            name: container.name,
            container_type: container.container_type,
            language: container.language,
            original_path: container.original_path,
            block_count: 0, // Will be populated by resolver
            quality_metrics: None, // Will be calculated by resolver
        }
    }
}

// ============================================================================
// Block Synthesis Types
// ============================================================================

/// Input for synthesizing a new semantic block
#[derive(InputObject, Clone, Debug)]
pub struct BlockSynthesisInput {
    /// Type of block to create
    pub block_type: String,
    /// Semantic name for the block
    pub semantic_name: String,
    /// Description of the block's purpose
    pub description: String,
    /// Block properties
    pub properties: Option<BlockPropertiesInput>,
    /// Behavioral specifications
    pub behaviors: Option<Vec<BehaviorInput>>,
    /// Constraints and invariants
    pub constraints: Option<Vec<String>>,
    /// Target container ID
    pub target_container: Option<ID>,
    /// Target language for code generation
    pub target_language: Option<String>,
}

/// Block properties input
#[derive(InputObject, Clone, Debug)]
pub struct BlockPropertiesInput {
    /// Function/method parameters
    pub parameters: Option<Vec<ParameterInput>>,
    /// Return type specification
    pub return_type: Option<TypeSpecInput>,
    /// Modifiers (public, private, static, etc.)
    pub modifiers: Option<Vec<String>>,
    /// Complexity target (1-10)
    pub complexity: Option<i32>,
    /// Is asynchronous
    pub is_async: Option<bool>,
    /// Visibility level
    pub visibility: Option<String>,
}

/// Parameter specification input
#[derive(InputObject, Clone, Debug)]
pub struct ParameterInput {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: TypeSpecInput,
    /// Parameter description
    pub description: Option<String>,
    /// Default value
    pub default_value: Option<String>,
    /// Is optional parameter
    pub is_optional: Option<bool>,
}

/// Type specification input
#[derive(InputObject, Clone, Debug)]
pub struct TypeSpecInput {
    /// Type name
    pub name: String,
    /// Generic type parameters
    pub generics: Option<Vec<TypeSpecInput>>,
    /// Is nullable/optional
    pub nullable: Option<bool>,
    /// Type constraints
    pub constraints: Option<Vec<String>>,
}

/// Behavior specification input
#[derive(InputObject, Clone, Debug)]
pub struct BehaviorInput {
    /// Behavior name
    pub name: String,
    /// Behavior description
    pub description: String,
    /// Preconditions
    pub preconditions: Option<Vec<String>>,
    /// Postconditions
    pub postconditions: Option<Vec<String>>,
    /// Side effects
    pub side_effects: Option<Vec<String>>,
}

/// Result of block synthesis
#[derive(SimpleObject, Clone, Debug)]
pub struct BlockSynthesisResult {
    /// Generated block ID
    pub block_id: ID,
    /// The synthesized semantic block
    pub semantic_block: GqlSemanticBlock,
    /// Generated source code
    pub generated_code: String,
    /// Created relationships
    pub relationships: Vec<GqlRelationship>,
    /// Warnings during synthesis
    pub warnings: Vec<String>,
}

/// Relationship for GraphQL
#[derive(SimpleObject, Clone, Debug)]
pub struct GqlRelationship {
    /// Source block ID
    pub source_id: ID,
    /// Target block ID  
    pub target_id: ID,
    /// Relationship type
    pub relationship_type: String,
    /// Additional properties
    pub properties: Option<String>, // JSON string
}

/// Input for module synthesis
#[derive(InputObject, Clone, Debug)]
pub struct ModuleSynthesisInput {
    /// Module name
    pub module_name: String,
    /// Component block specifications
    pub components: Vec<BlockSynthesisInput>,
    /// Module-level relationships
    pub relationships: Option<Vec<RelationshipInput>>,
    /// Architectural pattern
    pub architecture: Option<ModuleArchitectureInput>,
    /// Target language
    pub target_language: Option<String>,
}

/// Relationship specification input
#[derive(InputObject, Clone, Debug)]
pub struct RelationshipInput {
    /// Relationship type (uses, extends, implements, etc.)
    pub relationship_type: String,
    /// Target block identifier
    pub target_block: String,
    /// Additional properties (JSON string)
    pub properties: Option<String>,
}

/// Module architecture specification
#[derive(InputObject, Clone, Debug)]
pub struct ModuleArchitectureInput {
    /// Architectural pattern (facade, mvc, layered, etc.)
    pub pattern: String,
    /// Entry point methods/functions
    pub entry_points: Option<Vec<String>>,
    /// Dependencies
    pub dependencies: Option<Vec<String>>,
    /// Error handling strategy
    pub error_handling: Option<String>,
}

/// Result of module synthesis
#[derive(SimpleObject, Clone, Debug)]
pub struct ModuleSynthesisResult {
    /// Module ID
    pub module_id: ID,
    /// Generated component blocks
    pub components: Vec<GqlSemanticBlock>,
    /// Module relationships
    pub relationships: Vec<GqlRelationship>,
    /// Generated files (filename -> content)
    pub generated_files: Vec<GeneratedFile>,
    /// Module structure description
    pub module_structure: String,
    /// Warnings during synthesis
    pub warnings: Vec<String>,
}

/// Generated file representation
#[derive(SimpleObject, Clone, Debug)]
pub struct GeneratedFile {
    /// File name
    pub filename: String,
    /// File content
    pub content: String,
}

/// Input for code generation from existing blocks
#[derive(InputObject, Clone, Debug)]
pub struct GenerationInput {
    /// Block IDs to generate code for
    pub block_ids: Vec<ID>,
    /// Target language
    pub language: String,
    /// Generation options
    pub options: Option<GenerationOptionsInput>,
}

/// Code generation options
#[derive(InputObject, Clone, Debug)]
pub struct GenerationOptionsInput {
    /// Include type annotations
    pub include_types: Option<bool>,
    /// Include documentation
    pub include_docs: Option<bool>,
    /// Code style (clean, compact, verbose)
    pub style: Option<String>,
    /// Format generated code
    pub format_code: Option<bool>,
}

/// Result of code generation
#[derive(SimpleObject, Clone, Debug)]
pub struct GenerationResult {
    /// Generated files
    pub files: Vec<GeneratedFile>,
    /// Generation statistics
    pub stats: GenerationStats,
    /// Warnings during generation
    pub warnings: Vec<String>,
}

/// Generation statistics
#[derive(SimpleObject, Clone, Debug)]
pub struct GenerationStats {
    /// Number of blocks processed
    pub blocks_processed: i32,
    /// Lines of code generated
    pub lines_generated: i32,
    /// Files created
    pub files_created: i32,
    /// Generation time in milliseconds
    pub generation_time_ms: i32,
}
