use anyhow::Result;
use std::collections::HashMap;
use tree_sitter::Node;

use crate::traits::{ASTExtractor, ExtractionContext, ExtractionResult};

pub struct JavaScriptASTExtractor;

impl JavaScriptASTExtractor {
    pub fn new() -> Self {
        Self
    }
}

impl ASTExtractor for JavaScriptASTExtractor {
    fn extract(&self, _node: Node, _source: &str, _context: &ExtractionContext) -> Result<ExtractionResult> {
        // TODO: Implement JavaScript AST extraction
        anyhow::bail!("JavaScript AST extraction not yet implemented")
    }

    fn language(&self) -> &'static str {
        "javascript"
    }

    fn supports_extension(&self, extension: &str) -> bool {
        matches!(extension, "js" | "jsx" | "ts" | "tsx")
    }

    fn extract_semantic_metadata(&self, _node: Node, _source: &str) -> Result<HashMap<String, serde_json::Value>> {
        Ok(HashMap::new())
    }
}

impl Default for JavaScriptASTExtractor {
    fn default() -> Self {
        Self::new()
    }
}
