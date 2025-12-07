use anyhow::{Result, anyhow};
use tree_sitter::Node;
use crate::core::*;
use crate::parser::extraction_context::{ExtractionContext, ParseResult, LanguageExtractor};

pub struct JavaScriptExtractor {
    pub is_typescript: bool,
}

impl LanguageExtractor for JavaScriptExtractor {
    fn extract_with_context(&self, root: Node, source: &str, _file_path: &str) -> Result<ParseResult> {
        let mut context = ExtractionContext::new();
        self.visit_with_context(root, source, &mut context)?;
        Ok(context.finish())
    }
}

impl JavaScriptExtractor {
    #[allow(dead_code)]
    pub fn extract_blocks(&self, root: Node, source: &str, file_path: &str) -> Result<Vec<SemanticBlock>> {
        let result = self.extract_with_context(root, source, file_path)?;
        Ok(result.blocks)
    }
    
    fn visit_with_context(&self, node: Node, source: &str, ctx: &mut ExtractionContext) -> Result<()> {
        match node.kind() {
            "function_declaration" | "function_expression" | "arrow_function" => {
                if let Ok(block) = self.extract_function_block(node, source) {
                    let block_id = ctx.enter_block(block);
                    self.visit_children(node, source, ctx)?;
                    ctx.exit_block(block_id);
                }
            },
            "class_declaration" => {
                if let Ok(block) = self.extract_class_block(node, source) {
                    let block_id = ctx.enter_block(block);
                    self.visit_children(node, source, ctx)?;
                    ctx.exit_block(block_id);
                }
            },
            "method_definition" => {
                if let Ok(block) = self.extract_method_block(node, source) {
                    let block_id = ctx.enter_block(block);
                    self.visit_children(node, source, ctx)?;
                    ctx.exit_block(block_id);
                }
            },
            "import_statement" => {
                if let Ok(block) = self.extract_import_block(node, source) {
                    ctx.enter_block(block);
                }
            },
            _ => {
                self.visit_children(node, source, ctx)?;
            }
        }
        Ok(())
    }

    fn visit_children(&self, node: Node, source: &str, ctx: &mut ExtractionContext) -> Result<()> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_with_context(child, source, ctx)?;
        }
        Ok(())
    }
    
    fn extract_function_block(&self, node: Node, source: &str) -> Result<SemanticBlock> {
        let name = self.extract_function_name(node, source).unwrap_or_else(|_| "anonymous".to_string());
        let text = node.utf8_text(source.as_bytes())?;
        
        let mut block = SemanticBlock::new(
            BlockType::Function,
            name,
            text.to_string(),
            if self.is_typescript { "typescript" } else { "javascript" }.to_string(),
        );
        
        let start = node.start_position();
        let end = node.end_position();
        block.position = BlockPosition {
            start_line: start.row,
            end_line: end.row,
            start_column: start.column,
            end_column: end.column,
            index: 0,
        };
        
        Ok(block)
    }
    
    fn extract_class_block(&self, node: Node, source: &str) -> Result<SemanticBlock> {
        let name = self.extract_class_name(node, source)?;
        let text = node.utf8_text(source.as_bytes())?;
        
        let mut block = SemanticBlock::new(
            BlockType::Class,
            name,
            text.to_string(),
            if self.is_typescript { "typescript" } else { "javascript" }.to_string(),
        );
        
        let start = node.start_position();
        let end = node.end_position();
        block.position = BlockPosition {
            start_line: start.row,
            end_line: end.row,
            start_column: start.column,
            end_column: end.column,
            index: 0,
        };
        
        Ok(block)
    }
    
    fn extract_method_block(&self, node: Node, source: &str) -> Result<SemanticBlock> {
        let name = self.extract_method_name(node, source)?;
        let text = node.utf8_text(source.as_bytes())?;
        
        let mut block = SemanticBlock::new(
            BlockType::Function,
            name,
            text.to_string(),
            if self.is_typescript { "typescript" } else { "javascript" }.to_string(),
        );
        
        let start = node.start_position();
        let end = node.end_position();
        block.position = BlockPosition {
            start_line: start.row,
            end_line: end.row,
            start_column: start.column,
            end_column: end.column,
            index: 0,
        };
        
        Ok(block)
    }
    
    fn extract_import_block(&self, node: Node, source: &str) -> Result<SemanticBlock> {
        let text = node.utf8_text(source.as_bytes())?;
        let import_name = self.extract_import_name(node, source).unwrap_or_else(|_| "unknown_import".to_string());
        
        let mut block = SemanticBlock::new(
            BlockType::Import,
            import_name,
            text.to_string(),
            if self.is_typescript { "typescript" } else { "javascript" }.to_string(),
        );
        
        let start = node.start_position();
        let end = node.end_position();
        block.position = BlockPosition {
            start_line: start.row,
            end_line: end.row,
            start_column: start.column,
            end_column: end.column,
            index: 0,
        };
        
        Ok(block)
    }
    
    fn extract_function_name(&self, node: Node, source: &str) -> Result<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                return Ok(child.utf8_text(source.as_bytes())?.to_string());
            }
        }
        Err(anyhow!("Function name not found"))
    }

    fn extract_class_name(&self, node: Node, source: &str) -> Result<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                return Ok(child.utf8_text(source.as_bytes())?.to_string());
            }
        }
        Err(anyhow!("Class name not found"))
    }
    
    fn extract_method_name(&self, node: Node, source: &str) -> Result<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "property_identifier" || child.kind() == "identifier" {
                return Ok(child.utf8_text(source.as_bytes())?.to_string());
            }
        }
        Err(anyhow!("Method name not found"))
    }
    
    fn extract_import_name(&self, node: Node, source: &str) -> Result<String> {
        let text = node.utf8_text(source.as_bytes())?;
        if let Some(from_pos) = text.find(" from ") {
            if let Some(module_start) = text[from_pos + 6..].find('"') {
                let module_part = &text[from_pos + 6 + module_start + 1..];
                if let Some(module_end) = module_part.find('"') {
                    return Ok(module_part[..module_end].to_string());
                }
            }
        }
        Ok("unknown_import".to_string())
    }
}