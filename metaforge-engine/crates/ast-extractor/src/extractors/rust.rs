use anyhow::Result;
use std::collections::HashMap;
use tree_sitter::Node;

use crate::traits::{ASTExtractor, ExtractionContext, ExtractionResult};

pub struct RustASTExtractor;

impl RustASTExtractor {
    pub fn new() -> Self {
        Self
    }
}

impl ASTExtractor for RustASTExtractor {
    fn extract(&self, _node: Node, _source: &str, _context: &ExtractionContext) -> Result<ExtractionResult> {
        // TODO: Implement Rust AST extraction
        anyhow::bail!("Rust AST extraction not yet implemented")
    }

    fn language(&self) -> &'static str {
        "rust"
    }

    fn supports_extension(&self, extension: &str) -> bool {
        matches!(extension, "rs")
    }

    fn extract_semantic_metadata(&self, _node: Node, _source: &str) -> Result<HashMap<String, serde_json::Value>> {
        Ok(HashMap::new())
    }
}

impl Default for RustASTExtractor {
    fn default() -> Self {
        Self::new()
    }
}
