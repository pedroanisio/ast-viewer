//! # AST Extractor
//! 
//! Advanced Abstract Syntax Tree extraction for semantic code analysis.
//! This crate provides comprehensive AST extraction capabilities that go beyond
//! simple parsing to capture semantic meaning and relationships.

pub mod expression;
pub mod traits;
pub mod extractors;

pub use expression::{ExpressionAST, ExpressionExtractor, FunctionCall, AttributeAccess};
pub use traits::{ASTExtractor, ExtractionContext, ExtractionResult};
pub use extractors::{PythonASTExtractor, RustASTExtractor, JavaScriptASTExtractor};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Core AST node representing any semantic unit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ASTNode {
    pub id: Uuid,
    pub node_type: String,
    pub semantic_name: Option<String>,
    pub source_range: SourceRange,
    pub attributes: HashMap<String, serde_json::Value>,
    pub children: Vec<ASTNode>,
    pub expression_ast: Option<ExpressionAST>,
    pub metadata: ASTMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRange {
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub byte_start: usize,
    pub byte_end: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ASTMetadata {
    pub complexity_score: u32,
    pub dependencies: Vec<String>,
    pub exports: Vec<String>,
    pub language_specific: HashMap<String, serde_json::Value>,
}

impl ASTNode {
    pub fn new(node_type: String, source_range: SourceRange) -> Self {
        Self {
            id: Uuid::new_v4(),
            node_type,
            semantic_name: None,
            source_range,
            attributes: HashMap::new(),
            children: Vec::new(),
            expression_ast: None,
            metadata: ASTMetadata {
                complexity_score: 1,
                dependencies: Vec::new(),
                exports: Vec::new(),
                language_specific: HashMap::new(),
            },
        }
    }

    /// Find child nodes by type
    pub fn find_children_by_type(&self, node_type: &str) -> Vec<&ASTNode> {
        self.children
            .iter()
            .filter(|child| child.node_type == node_type)
            .collect()
    }

    /// Get all dependencies recursively
    pub fn get_all_dependencies(&self) -> Vec<String> {
        let mut deps = self.metadata.dependencies.clone();
        for child in &self.children {
            deps.extend(child.get_all_dependencies());
        }
        deps.sort();
        deps.dedup();
        deps
    }

    /// Calculate total complexity score
    pub fn total_complexity(&self) -> u32 {
        self.metadata.complexity_score + 
        self.children.iter().map(|c| c.total_complexity()).sum::<u32>()
    }
}
