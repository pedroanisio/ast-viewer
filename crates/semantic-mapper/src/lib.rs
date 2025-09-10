//! # Semantic Mapper
//! 
//! Maps AST nodes to semantic code components for generation.
//! This crate bridges the gap between raw AST extraction and code generation
//! by providing semantic understanding of code structures.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use ast_extractor::{ASTNode, ExpressionAST, traits::{SemanticBlock, Dependency, Export}};

pub mod components;
pub mod mappers;
pub mod relationships;

pub use components::{
    CodeComponent, FunctionSignature, FunctionBody, ClassDeclaration, ClassBody,
    VariableDeclaration, ImportStatement, Statement, Parameter, TypeAnnotation
};
pub use mappers::{ComponentMapper, PythonMapper, RustMapper, TypeScriptMapper};
pub use relationships::{RelationshipAnalyzer, ComponentRelationship, RelationshipType};

/// Main semantic mapper that orchestrates component extraction
pub struct SemanticMapper {
    mappers: HashMap<String, Box<dyn ComponentMapper>>,
    relationship_analyzer: RelationshipAnalyzer,
}

impl SemanticMapper {
    pub fn new() -> Self {
        let mut mappers: HashMap<String, Box<dyn ComponentMapper>> = HashMap::new();
        mappers.insert("python".to_string(), Box::new(PythonMapper::new()));
        mappers.insert("rust".to_string(), Box::new(RustMapper::new()));
        mappers.insert("typescript".to_string(), Box::new(TypeScriptMapper::new()));
        mappers.insert("javascript".to_string(), Box::new(TypeScriptMapper::new())); // JS uses TS mapper

        Self {
            mappers,
            relationship_analyzer: RelationshipAnalyzer::new(),
        }
    }

    /// Map a semantic block to code components
    pub fn map_block_to_components(&self, block: &SemanticBlock, language: &str) -> Result<Vec<CodeComponent>> {
        let mapper = self.mappers.get(language)
            .ok_or_else(|| anyhow::anyhow!("Unsupported language: {}", language))?;

        mapper.map_semantic_block(block)
    }

    /// Map AST directly to components (backward compatibility)
    pub fn map_ast_to_components(&self, ast: &serde_json::Value) -> Result<Vec<CodeComponent>> {
        let mut components = Vec::new();
        
        // Extract block type
        let block_type = ast.get("type")
            .and_then(|t| t.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing block type"))?;
        
        // Determine language from context or default to Python
        let language = ast.get("language")
            .and_then(|l| l.as_str())
            .unwrap_or("python");
        
        let mapper = self.mappers.get(language)
            .ok_or_else(|| anyhow::anyhow!("Unsupported language: {}", language))?;

        // Create a temporary semantic block from JSON
        let temp_block = self.json_to_semantic_block(ast)?;
        
        components.extend(mapper.map_semantic_block(&temp_block)?);
        
        Ok(components)
    }

    /// Analyze relationships between components
    pub fn analyze_relationships(&self, components: &[CodeComponent]) -> Result<Vec<ComponentRelationship>> {
        self.relationship_analyzer.analyze(components)
    }

    /// Convert JSON AST to semantic block (for backward compatibility)
    fn json_to_semantic_block(&self, ast: &serde_json::Value) -> Result<SemanticBlock> {
        let block_type = ast.get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("unknown")
            .to_string();
        
        let semantic_name = ast.get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("unnamed")
            .to_string();

        // Create minimal AST node
        let ast_node = ASTNode::new(
            block_type.clone(),
            ast_extractor::SourceRange {
                start_line: 0,
                start_column: 0,
                end_line: 0,
                end_column: 0,
                byte_start: 0,
                byte_end: 0,
            }
        );

        let mut semantic_block = SemanticBlock::new(
            block_type,
            semantic_name,
            ast_node,
        );

        // Extract expression AST if available
        if let Some(expr_ast_json) = ast.get("expression_ast") {
            if let Ok(expr_ast) = serde_json::from_value::<ExpressionAST>(expr_ast_json.clone()) {
                semantic_block.expression_ast = Some(expr_ast);
            }
        }

        // Mark as generation ready if we have expression data
        if semantic_block.expression_ast.is_some() {
            semantic_block.mark_generation_ready();
        }

        Ok(semantic_block)
    }
}

impl Default for SemanticMapper {
    fn default() -> Self {
        Self::new()
    }
}

/// Mapping context for additional information during mapping
#[derive(Debug, Clone)]
pub struct MappingContext {
    pub language: String,
    pub file_path: Option<String>,
    pub container_id: Uuid,
    pub preserve_formatting: bool,
    pub extract_comments: bool,
}

impl MappingContext {
    pub fn new(language: String, container_id: Uuid) -> Self {
        Self {
            language,
            file_path: None,
            container_id,
            preserve_formatting: true,
            extract_comments: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_function_block() {
        let mapper = SemanticMapper::new();
        
        let ast = serde_json::json!({
            "type": "function",
            "name": "calculate_sum",
            "parameters": [
                {"name": "a", "type": "int"},
                {"name": "b", "type": "int"}
            ],
            "return_type": "int",
            "expression_ast": {
                "expression_type": "binary_operator",
                "operator": "+",
                "operands": [
                    {"expression_type": "identifier", "variables": ["a"]},
                    {"expression_type": "identifier", "variables": ["b"]}
                ],
                "source_text": "a + b"
            }
        });

        let components = mapper.map_ast_to_components(&ast).unwrap();
        
        assert!(!components.is_empty());
        
        // Should have at least function signature and body
        assert!(components.iter().any(|c| matches!(c, CodeComponent::FunctionSignature(_))));
        assert!(components.iter().any(|c| matches!(c, CodeComponent::FunctionBody(_))));
    }

    #[test]
    fn test_analyze_relationships() {
        let mapper = SemanticMapper::new();
        
        let components = vec![
            CodeComponent::FunctionSignature(FunctionSignature {
                name: "process_data".to_string(),
                parameters: vec![],
                return_type: None,
                is_async: false,
                decorators: vec![],
                type_parameters: vec![],
            }),
            CodeComponent::FunctionBody(FunctionBody {
                statements: vec![],
                expressions: vec![],
                local_variables: vec!["result".to_string()],
                called_functions: vec!["calculate".to_string()],
            }),
        ];

        let relationships = mapper.analyze_relationships(&components).unwrap();
        
        // Should detect function call relationship
        assert!(!relationships.is_empty());
    }
}