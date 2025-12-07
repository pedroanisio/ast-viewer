use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tree_sitter::Node;
use uuid::Uuid;

use crate::{ASTNode, ExpressionAST};

/// Core trait for AST extraction from different languages
pub trait ASTExtractor {
    /// Extract AST from a tree-sitter node
    fn extract(&self, node: Node, source: &str, context: &ExtractionContext) -> Result<ExtractionResult>;
    
    /// Get supported language identifier
    fn language(&self) -> &'static str;
    
    /// Check if this extractor can handle the given file extension
    fn supports_extension(&self, extension: &str) -> bool;
    
    /// Extract semantic metadata for a specific node type
    fn extract_semantic_metadata(&self, node: Node, source: &str) -> Result<HashMap<String, serde_json::Value>>;
}

/// Context information for AST extraction
#[derive(Debug, Clone)]
pub struct ExtractionContext {
    pub file_path: String,
    pub language: String,
    pub container_id: Uuid,
    pub migration_id: Uuid,
    pub extract_expressions: bool,
    pub max_depth: Option<usize>,
    pub include_comments: bool,
}

impl ExtractionContext {
    pub fn new(file_path: String, language: String, container_id: Uuid, migration_id: Uuid) -> Self {
        Self {
            file_path,
            language,
            container_id,
            migration_id,
            extract_expressions: true,
            max_depth: None,
            include_comments: false,
        }
    }

    pub fn with_expression_extraction(mut self, extract: bool) -> Self {
        self.extract_expressions = extract;
        self
    }

    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }

    pub fn with_comments(mut self, include: bool) -> Self {
        self.include_comments = include;
        self
    }
}

/// Result of AST extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    pub root_node: ASTNode,
    pub semantic_blocks: Vec<SemanticBlock>,
    pub dependencies: Vec<Dependency>,
    pub exports: Vec<Export>,
    pub metadata: ExtractionMetadata,
}

/// Semantic block representing a meaningful code unit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticBlock {
    pub id: Uuid,
    pub block_type: String,
    pub semantic_name: String,
    pub ast_node: ASTNode,
    pub expression_ast: Option<ExpressionAST>,
    pub dependencies: Vec<String>,
    pub exports: Vec<String>,
    pub complexity_score: u32,
    pub generation_ready: bool,
}

/// Dependency relationship between code units
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub from_block_id: Uuid,
    pub to_identifier: String,
    pub dependency_type: DependencyType,
    pub is_external: bool,
    pub module_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    Import,
    FunctionCall,
    VariableReference,
    TypeReference,
    Inheritance,
    Composition,
}

/// Export information for code units
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Export {
    pub identifier: String,
    pub export_type: ExportType,
    pub block_id: Uuid,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportType {
    Function,
    Class,
    Variable,
    Constant,
    Type,
    Module,
}

/// Metadata about the extraction process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionMetadata {
    pub total_nodes: usize,
    pub semantic_blocks_count: usize,
    pub dependencies_count: usize,
    pub exports_count: usize,
    pub extraction_time_ms: u64,
    pub language_specific: HashMap<String, serde_json::Value>,
}

impl SemanticBlock {
    pub fn new(
        block_type: String,
        semantic_name: String,
        ast_node: ASTNode,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            block_type,
            semantic_name,
            ast_node,
            expression_ast: None,
            dependencies: Vec::new(),
            exports: Vec::new(),
            complexity_score: 1,
            generation_ready: false,
        }
    }

    /// Check if this block is ready for code generation
    pub fn is_generation_ready(&self) -> bool {
        self.generation_ready && 
        self.expression_ast.is_some() &&
        !self.semantic_name.is_empty()
    }

    /// Mark this block as ready for generation
    pub fn mark_generation_ready(&mut self) {
        self.generation_ready = self.expression_ast.is_some() && !self.semantic_name.is_empty();
    }

    /// Get all transitive dependencies
    pub fn get_transitive_dependencies(&self) -> Vec<String> {
        // This would be implemented with graph traversal in practice
        self.dependencies.clone()
    }
}

impl ExtractionResult {
    pub fn new(root_node: ASTNode) -> Self {
        Self {
            root_node,
            semantic_blocks: Vec::new(),
            dependencies: Vec::new(),
            exports: Vec::new(),
            metadata: ExtractionMetadata {
                total_nodes: 0,
                semantic_blocks_count: 0,
                dependencies_count: 0,
                exports_count: 0,
                extraction_time_ms: 0,
                language_specific: HashMap::new(),
            },
        }
    }

    pub fn add_semantic_block(&mut self, block: SemanticBlock) {
        self.semantic_blocks.push(block);
        self.metadata.semantic_blocks_count = self.semantic_blocks.len();
    }

    pub fn add_dependency(&mut self, dependency: Dependency) {
        self.dependencies.push(dependency);
        self.metadata.dependencies_count = self.dependencies.len();
    }

    pub fn add_export(&mut self, export: Export) {
        self.exports.push(export);
        self.metadata.exports_count = self.exports.len();
    }

    /// Get semantic blocks by type
    pub fn get_blocks_by_type(&self, block_type: &str) -> Vec<&SemanticBlock> {
        self.semantic_blocks
            .iter()
            .filter(|block| block.block_type == block_type)
            .collect()
    }

    /// Get generation-ready blocks
    pub fn get_generation_ready_blocks(&self) -> Vec<&SemanticBlock> {
        self.semantic_blocks
            .iter()
            .filter(|block| block.is_generation_ready())
            .collect()
    }
}

/// Trait for language-specific semantic analysis
pub trait SemanticAnalyzer {
    /// Analyze semantic relationships in the AST
    fn analyze_semantics(&self, ast: &ASTNode, source: &str) -> Result<Vec<SemanticBlock>>;
    
    /// Extract dependencies from the AST
    fn extract_dependencies(&self, ast: &ASTNode, source: &str) -> Result<Vec<Dependency>>;
    
    /// Extract exports from the AST
    fn extract_exports(&self, ast: &ASTNode, source: &str) -> Result<Vec<Export>>;
    
    /// Calculate complexity metrics
    fn calculate_complexity(&self, block: &SemanticBlock) -> u32;
}
