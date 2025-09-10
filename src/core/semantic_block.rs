use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticBlock {
    pub id: Uuid,
    pub block_type: BlockType,
    pub semantic_identity: SemanticIdentity,
    pub syntax_preservation: SyntaxPreservation,
    pub structural_context: StructuralContext,
    pub semantic_metadata: SemanticMetadata,
    pub position: BlockPosition,
    pub source_language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticIdentity {
    pub canonical_name: String,
    pub aliases: Vec<String>,
    pub fully_qualified_name: Option<String>,
    pub signature_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxPreservation {
    pub original_text: String,
    pub normalized_ast: serde_json::Value,
    pub reconstruction_hints: ReconstructionHints,
    pub formatting_preserved: FormattingInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconstructionHints {
    pub prefer_original: bool,
    pub template: Option<String>,
    pub parameter_positions: Vec<ParameterInfo>,
    pub body_extraction: BodyExtraction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyExtraction {
    pub method: String,
    pub start_marker: String,
    pub preserve_indentation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormattingInfo {
    pub indentation: String,
    pub line_endings: String,
    pub spacing: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralContext {
    pub parent_block: Option<Uuid>,
    pub child_blocks: Vec<Uuid>,
    pub inheritance_chain: Vec<String>,
    pub implements: Vec<String>,
    pub decorators: Vec<Decorator>,
    pub scope: ScopeInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decorator {
    pub name: String,
    pub arguments: Vec<String>,
    pub line_number: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScopeInfo {
    Module(String),
    Class(String),
    Function(String),
    Block,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticMetadata {
    pub parameters: Vec<Parameter>,
    pub return_type: Option<TypeInfo>,
    pub throws: Vec<String>,
    pub side_effects: Vec<String>,
    pub purity: PurityLevel,
    pub visibility: Visibility,
    pub modifiers: Vec<Modifier>,
    
    // Advanced language features (Phase 1.1 enhancements)
    pub generics: Option<GenericInfo>,
    pub macros: Option<MacroInfo>,
    pub decorators: Option<DecoratorInfo>,
    pub type_annotations: Option<AdvancedTypeInfo>,
    pub side_effect_analysis: Option<SideEffectAnalysis>,
    pub parameter_details: Option<ParameterDetails>,
    pub complexity_metrics: Option<ComplexityMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub type_hint: Option<String>,
    pub default_value: Option<String>,
    pub is_optional: bool,
    pub position: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterInfo {
    pub name: String,
    pub start_pos: usize,
    pub end_pos: usize,
    pub type_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeInfo {
    pub representation: String,
    pub is_generic: bool,
    pub generic_args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PurityLevel {
    Pure,
    MostlyPure,
    Impure,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Modifier {
    Async,
    Static,
    Const,
    Final,
    Abstract,
    Override,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockPosition {
    pub start_line: usize,
    pub end_line: usize,
    pub start_column: usize,
    pub end_column: usize,
    pub index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BlockType {
    Function,
    Class,
    Interface,
    Variable,
    Import,
    Export,
    Conditional,
    Loop,
    TryCatch,
    Comment,
    TypeDef,
    Component,
    Query,
    Config,
    Module,
}

impl std::fmt::Display for BlockType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockType::Function => write!(f, "Function"),
            BlockType::Class => write!(f, "Class"),
            BlockType::Interface => write!(f, "Interface"),
            BlockType::Variable => write!(f, "Variable"),
            BlockType::Import => write!(f, "Import"),
            BlockType::Export => write!(f, "Export"),
            BlockType::Conditional => write!(f, "Conditional"),
            BlockType::Loop => write!(f, "Loop"),
            BlockType::TryCatch => write!(f, "TryCatch"),
            BlockType::Comment => write!(f, "Comment"),
            BlockType::TypeDef => write!(f, "TypeDef"),
            BlockType::Component => write!(f, "Component"),
            BlockType::Query => write!(f, "Query"),
            BlockType::Config => write!(f, "Config"),
            BlockType::Module => write!(f, "Module"),
        }
    }
}

// Advanced language features (Phase 1.1 enhancements)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericInfo {
    pub generic_parameters: Vec<GenericParameter>,
    pub parameters: Vec<GenericParameter>, // alias for compatibility
    pub constraints: Vec<GenericConstraint>,
    pub variance: Option<Variance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericParameter {
    pub name: String,
    pub bounds: Vec<String>,
    pub constraints: Vec<String>, // alias for bounds
    pub default_type: Option<String>,
    pub variance: Option<Variance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericConstraint {
    pub parameter: String,
    pub parameters: Vec<String>, // for multiple parameters
    pub constraint_type: ConstraintType,
    pub constraint_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    Trait,      // Rust: T: Display
    Type,       // TypeScript: T extends string
    Lifetime,   // Rust: 'a
    Where,      // Rust: where T: Clone
    Clone,      // Rust: T: Clone
    Copy,       // Rust: T: Copy
    Send,       // Rust: T: Send
    Sync,       // Rust: T: Sync
    Other(String), // Custom constraints
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Variance {
    Covariant,    // +T
    Contravariant, // -T
    Invariant,    // T
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroInfo {
    pub macro_name: String,
    pub macro_type: MacroType,
    pub parameters: Vec<MacroParameter>,
    pub expansion: Option<String>,
    pub hygiene: HygieneLevel,
    pub hygiene_level: HygieneLevel, // alias for compatibility
    pub is_procedural: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MacroType {
    Declarative,  // macro_rules!
    Procedural,   // proc_macro
    Builtin,      // println!, vec!
    Custom,       // User-defined
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroParameter {
    pub name: String,
    pub parameter_type: MacroParamType,
    pub is_repeatable: bool,
    pub is_optional: bool,
    pub default_value: Option<String>,
    pub separator: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MacroParamType {
    Ident,        // identifier
    Literal,      // literal
    Expr,         // expression
    Expression,   // alias for expression
    Stmt,         // statement
    Block,        // block
    Ty,           // type
    Pat,          // pattern
    Path,         // path
    Meta,         // meta
    Vis,          // visibility
    Lifetime,     // lifetime
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HygieneLevel {
    Hygienic,     // Variables don't leak
    Unhygienic,   // Variables can leak
    Mixed,        // Some variables leak
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecoratorInfo {
    pub decorators: Vec<AdvancedDecorator>,
    pub execution_order: Vec<usize>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedDecorator {
    pub name: String,
    pub arguments: Vec<DecoratorArgument>,
    pub line_number: usize,
    pub column_number: usize,
    pub is_builtin: bool,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecoratorArgument {
    pub name: Option<String>,
    pub value: serde_json::Value,
    pub argument_type: DecoratorArgType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecoratorArgType {
    Positional,
    Keyword,
    Starred,      // *args
    DoubleStarred, // **kwargs
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedTypeInfo {
    pub union_types: Vec<TypeInfo>,
    pub intersection_types: Vec<TypeInfo>,
    pub optional_types: Vec<TypeInfo>,
    pub array_types: Vec<TypeInfo>,
    pub tuple_types: Vec<TypeInfo>,
    pub function_types: Vec<FunctionTypeInfo>,
    pub generic_types: Vec<GenericTypeInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionTypeInfo {
    pub parameters: Vec<TypeInfo>,
    pub return_type: TypeInfo,
    pub is_async: bool,
    pub is_generator: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericTypeInfo {
    pub base_type: String,
    pub type_arguments: Vec<TypeInfo>,
    pub constraints: Vec<GenericConstraint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideEffectAnalysis {
    pub purity_level: PurityLevel,
    pub side_effects: Vec<SideEffect>,
    pub dependencies: Vec<Dependency>,
    pub mutability: MutabilityInfo,
    pub resource_usage: ResourceUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideEffect {
    pub effect_type: SideEffectType,
    pub description: String,
    pub severity: EffectSeverity,
    pub line_number: Option<usize>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SideEffectType {
    FileIO,
    NetworkIO,
    DatabaseIO,
    ConsoleIO,
    MemoryAllocation,
    SystemCall,
    GlobalStateMutation,
    ExceptionThrowing,
    AsyncOperation,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: Option<String>,
    pub dependency_type: DependencyType,
    pub is_optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    Import,
    Require,
    Include,
    Use,
    From,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutabilityInfo {
    pub is_mutable: bool,
    pub mutates_self: bool,
    pub mutates_parameters: bool,
    pub mutates_globals: bool,
    pub mutates_externals: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub memory_usage: Option<MemoryUsage>,
    pub cpu_usage: Option<CpuUsage>,
    pub io_operations: Vec<IoOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsage {
    pub estimated_bytes: Option<usize>,
    pub allocation_type: AllocationType,
    pub is_leaked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocationType {
    Stack,
    Heap,
    Static,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuUsage {
    pub complexity: ComplexityLevel,
    pub estimated_operations: Option<usize>,
    pub is_optimizable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplexityLevel {
    Constant,
    Logarithmic,
    Linear,
    Linearithmic,
    Quadratic,
    Cubic,
    Exponential,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoOperation {
    pub operation_type: IoType,
    pub resource: String,
    pub is_blocking: bool,
    pub estimated_latency: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IoType {
    Read,
    Write,
    Append,
    Delete,
    Network,
    Database,
    Cache,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDetails {
    pub parameters: Vec<DetailedParameter>,
    pub variadic_info: Option<VariadicInfo>,
    pub default_values: HashMap<String, serde_json::Value>,
    pub type_constraints: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedParameter {
    pub name: String,
    pub type_annotation: Option<TypeInfo>,
    pub default_value: Option<serde_json::Value>,
    pub is_optional: bool,
    pub is_variadic: bool,
    pub is_keyword_only: bool,
    pub is_positional_only: bool,
    pub documentation: Option<String>,
    pub validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariadicInfo {
    pub variadic_type: VariadicType,
    pub parameter_name: String,
    pub separator: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariadicType {
    Args,        // *args
    Kwargs,      // **kwargs
    Rest,        // ...rest
    Spread,      // ...spread
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub rule_type: ValidationType,
    pub rule_value: serde_json::Value,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationType {
    MinLength,
    MaxLength,
    MinValue,
    MaxValue,
    Pattern,
    Required,
    Optional,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityMetrics {
    pub cyclomatic_complexity: usize,
    pub cognitive_complexity: usize,
    pub lines_of_code: usize,
    pub number_of_parameters: usize,
    pub nesting_depth: usize,
    pub branching_factor: usize,
    pub maintainability_index: f64,
}

// Helper implementations
impl Parameter {
    pub fn to_info(&self) -> ParameterInfo {
        ParameterInfo {
            name: self.name.clone(),
            start_pos: 0, // Will be filled during extraction
            end_pos: 0,   // Will be filled during extraction
            type_hint: self.type_hint.clone(),
        }
    }
}

impl SemanticBlock {
    pub fn new(
        block_type: BlockType,
        canonical_name: String,
        original_text: String,
        source_language: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            block_type,
            semantic_identity: SemanticIdentity {
                canonical_name: canonical_name.clone(),
                aliases: vec![],
                fully_qualified_name: Some(canonical_name),
                signature_hash: String::new(), // Will be filled during extraction
            },
            syntax_preservation: SyntaxPreservation {
                original_text: original_text.clone(),
                normalized_ast: serde_json::Value::Null,
                reconstruction_hints: ReconstructionHints {
                    prefer_original: true,
                    template: Some(original_text),
                    parameter_positions: vec![],
                    body_extraction: BodyExtraction {
                        method: "original".to_string(),
                        start_marker: "".to_string(),
                        preserve_indentation: true,
                    },
                },
                formatting_preserved: FormattingInfo {
                    indentation: "    ".to_string(),
                    line_endings: "\n".to_string(),
                    spacing: HashMap::new(),
                },
            },
            structural_context: StructuralContext {
                parent_block: None,
                child_blocks: vec![],
                inheritance_chain: vec![],
                implements: vec![],
                decorators: vec![],
                scope: ScopeInfo::Module("main".to_string()),
            },
            semantic_metadata: SemanticMetadata {
                parameters: vec![],
                return_type: None,
                throws: vec![],
                side_effects: vec![],
                purity: PurityLevel::Unknown,
                visibility: Visibility::Public,
                modifiers: vec![],
                
                // Advanced language features (Phase 1.1 enhancements)
                generics: None,
                macros: None,
                decorators: None,
                type_annotations: None,
                side_effect_analysis: None,
                parameter_details: None,
                complexity_metrics: None,
            },
            position: BlockPosition {
                start_line: 0,
                end_line: 0,
                start_column: 0,
                end_column: 0,
                index: 0,
            },
            source_language,
        }
    }
    
    // Helper methods for advanced language features (Phase 1.1)
    
    /// Add generic information to the block
    #[allow(dead_code)]
    pub fn with_generics(mut self, generics: GenericInfo) -> Self {
        self.semantic_metadata.generics = Some(generics);
        self
    }
    
    /// Add macro information to the block
    #[allow(dead_code)]
    pub fn with_macros(mut self, macros: MacroInfo) -> Self {
        self.semantic_metadata.macros = Some(macros);
        self
    }
    
    /// Add advanced decorator information to the block
    #[allow(dead_code)]
    pub fn with_decorators(mut self, decorators: DecoratorInfo) -> Self {
        self.semantic_metadata.decorators = Some(decorators);
        self
    }
    
    /// Add advanced type annotations to the block
    #[allow(dead_code)]
    pub fn with_type_annotations(mut self, type_annotations: AdvancedTypeInfo) -> Self {
        self.semantic_metadata.type_annotations = Some(type_annotations);
        self
    }
    
    /// Add side effect analysis to the block
    #[allow(dead_code)]
    pub fn with_side_effect_analysis(mut self, analysis: SideEffectAnalysis) -> Self {
        self.semantic_metadata.side_effect_analysis = Some(analysis);
        self
    }
    
    /// Add detailed parameter information to the block
    #[allow(dead_code)]
    pub fn with_parameter_details(mut self, details: ParameterDetails) -> Self {
        self.semantic_metadata.parameter_details = Some(details);
        self
    }
    
    /// Add complexity metrics to the block
    #[allow(dead_code)]
    pub fn with_complexity_metrics(mut self, metrics: ComplexityMetrics) -> Self {
        self.semantic_metadata.complexity_metrics = Some(metrics);
        self
    }
    
    /// Check if the block has generic parameters
    #[allow(dead_code)]
    pub fn has_generics(&self) -> bool {
        self.semantic_metadata.generics.is_some()
    }
    
    /// Check if the block has side effects
    #[allow(dead_code)]
    pub fn has_side_effects(&self) -> bool {
        self.semantic_metadata.side_effect_analysis
            .as_ref()
            .map(|analysis| !analysis.side_effects.is_empty())
            .unwrap_or(false)
    }
    
    /// Get the purity level of the block
    #[allow(dead_code)]
    pub fn get_purity_level(&self) -> &PurityLevel {
        self.semantic_metadata.side_effect_analysis
            .as_ref()
            .map(|analysis| &analysis.purity_level)
            .unwrap_or(&self.semantic_metadata.purity)
    }
    
    /// Get the complexity score of the block
    #[allow(dead_code)]
    pub fn get_complexity_score(&self) -> Option<usize> {
        self.semantic_metadata.complexity_metrics
            .as_ref()
            .map(|metrics| metrics.cyclomatic_complexity)
    }
    
    /// Check if the block is a pure function
    #[allow(dead_code)]
    pub fn is_pure(&self) -> bool {
        matches!(self.get_purity_level(), PurityLevel::Pure)
    }
    
    /// Get all side effects of the block
    #[allow(dead_code)]
    pub fn get_side_effects(&self) -> Vec<&SideEffect> {
        self.semantic_metadata.side_effect_analysis
            .as_ref()
            .map(|analysis| analysis.side_effects.iter().collect())
            .unwrap_or_default()
    }
    
    /// Get all dependencies of the block
    #[allow(dead_code)]
    pub fn get_dependencies(&self) -> Vec<&Dependency> {
        self.semantic_metadata.side_effect_analysis
            .as_ref()
            .map(|analysis| analysis.dependencies.iter().collect())
            .unwrap_or_default()
    }
}
