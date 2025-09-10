use anyhow::{Result, anyhow};
use tree_sitter::Node;
use std::collections::HashMap;
use crate::core::*;
use crate::parser::extraction_context::{ExtractionContext, ParseResult, LanguageExtractor};

pub struct RustExtractor;

impl LanguageExtractor for RustExtractor {
    fn extract_with_context(&self, root: Node, source: &str, _file_path: &str) -> Result<ParseResult> {
        let mut context = ExtractionContext::new();
        self.visit_with_context(root, source, &mut context)?;
        Ok(context.finish())
    }
}

impl RustExtractor {
    #[allow(dead_code)]
    pub fn extract_blocks(&self, root: Node, source: &str, file_path: &str) -> Result<Vec<SemanticBlock>> {
        let result = self.extract_with_context(root, source, file_path)?;
        Ok(result.blocks)
    }
    
    fn visit_with_context(&self, node: Node, source: &str, ctx: &mut ExtractionContext) -> Result<()> {
        match node.kind() {
            "function_item" => {
                if let Ok(block) = self.extract_function_block(node, source) {
                    let block_id = ctx.enter_block(block);
                    self.visit_children(node, source, ctx)?;
                    ctx.exit_block(block_id);
                }
            },
            "struct_item" | "enum_item" => {
                if let Ok(block) = self.extract_struct_block(node, source) {
                    let block_id = ctx.enter_block(block);
                    self.visit_children(node, source, ctx)?;
                    ctx.exit_block(block_id);
                }
            },
            "impl_item" => {
                if let Ok(block) = self.extract_impl_block(node, source) {
                    let block_id = ctx.enter_block(block);
                    self.visit_children(node, source, ctx)?;
                    ctx.exit_block(block_id);
                }
            },
            "use_declaration" => {
                if let Ok(block) = self.extract_use_block(node, source) {
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
        let name = self.extract_function_name(node, source)?;
        let text = node.utf8_text(source.as_bytes())?;
        
        let mut block = SemanticBlock::new(
            BlockType::Function,
            name,
            text.to_string(),
            "rust".to_string(),
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
    
    fn extract_struct_block(&self, node: Node, source: &str) -> Result<SemanticBlock> {
        let name = self.extract_struct_name(node, source)?;
        let text = node.utf8_text(source.as_bytes())?;
        
        let mut block = SemanticBlock::new(
            BlockType::Class,
            name,
            text.to_string(),
            "rust".to_string(),
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
    
    fn extract_impl_block(&self, node: Node, source: &str) -> Result<SemanticBlock> {
        let name = self.extract_impl_name(node, source)?;
        let text = node.utf8_text(source.as_bytes())?;
        
        let mut block = SemanticBlock::new(
            BlockType::Class,
            name,
            text.to_string(),
            "rust".to_string(),
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
    
    fn extract_use_block(&self, node: Node, source: &str) -> Result<SemanticBlock> {
        let text = node.utf8_text(source.as_bytes())?;
        let use_name = self.extract_use_name(node, source)?;
        
        let mut block = SemanticBlock::new(
            BlockType::Import,
            use_name,
            text.to_string(),
            "rust".to_string(),
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
    
    fn extract_struct_name(&self, node: Node, source: &str) -> Result<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "type_identifier" {
                return Ok(child.utf8_text(source.as_bytes())?.to_string());
            }
        }
        Err(anyhow!("Struct name not found"))
    }
    
    fn extract_impl_name(&self, node: Node, source: &str) -> Result<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "type_identifier" {
                return Ok(format!("impl {}", child.utf8_text(source.as_bytes())?));
            }
        }
        Ok("impl".to_string())
    }
    
    fn extract_use_name(&self, node: Node, source: &str) -> Result<String> {
        let text = node.utf8_text(source.as_bytes())?;
        if let Some(use_part) = text.strip_prefix("use ") {
            if let Some(semicolon_pos) = use_part.find(';') {
                return Ok(use_part[..semicolon_pos].trim().to_string());
            }
        }
        Ok("unknown_use".to_string())
    }
    
    fn _extract_blocks_legacy(&self, root: Node, source: &str, _file_path: &str) -> Result<Vec<SemanticBlock>> {
        let mut visitor = RustVisitor::new(source);
        let _ = visitor.visit(root);
        visitor.into_blocks()
    }
}

#[allow(dead_code)]
struct RustVisitor<'a> {
    source: &'a str,
    blocks: Vec<SemanticBlock>,
    current_module: Option<String>,
}

#[allow(dead_code)]
impl<'a> RustVisitor<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            blocks: Vec::new(),
            current_module: None,
        }
    }

    fn string_to_modifier(&self, s: &str) -> Modifier {
        match s {
            "async" => Modifier::Async,
            "static" => Modifier::Static,
            "const" => Modifier::Const,
            "final" => Modifier::Final,
            "abstract" => Modifier::Abstract,
            "override" => Modifier::Override,
            _ => Modifier::Static, // default fallback
        }
    }

    fn visit(&mut self, node: Node) -> Result<()> {
        match node.kind() {
            "function_item" => self.visit_function(node)?,
            "struct_item" => self.visit_struct(node)?,
            "enum_item" => self.visit_enum(node)?,
            "impl_item" => self.visit_impl(node)?,
            "trait_item" => self.visit_trait(node)?,
            "macro_definition" => self.visit_macro(node)?,
            "use_declaration" => self.visit_use(node)?,
            "mod_item" => self.visit_module(node)?,
            _ => {
                // Recursively visit children
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.visit(child)?;
                }
            }
        }
        Ok(())
    }

    fn visit_function(&mut self, node: Node) -> Result<()> {
        let name = self.extract_function_name(node)?;
        let params = self.extract_parameters(node)?;
        let original_text = node.utf8_text(self.source.as_bytes())?;
        
        let mut block = SemanticBlock::new(
            BlockType::Function,
            name.clone(),
            original_text.to_string(),
            "rust".to_string(),
        );

        // Set semantic identity
        block.semantic_identity.canonical_name = name.clone();
        block.semantic_identity.fully_qualified_name = self.build_qualified_name(&name);
        block.semantic_identity.signature_hash = self.hash_signature(&original_text);

        // Set syntax preservation
        block.syntax_preservation.original_text = original_text.to_string();
        block.syntax_preservation.normalized_ast = self.node_to_json(node)?;
        block.syntax_preservation.reconstruction_hints = ReconstructionHints {
            prefer_original: true,
            template: Some(original_text.to_string()),
            parameter_positions: params.iter().map(|p| p.to_info()).collect(),
            body_extraction: BodyExtraction {
                method: "between_braces".to_string(),
                start_marker: "{".to_string(),
                preserve_indentation: true,
            },
        };

        // Set structural context
        block.structural_context.scope = if let Some(ref module) = self.current_module {
            ScopeInfo::Module(module.clone())
        } else {
            ScopeInfo::Module("main".to_string())
        };

        // Set semantic metadata
        block.semantic_metadata.parameters = params.clone();
        block.semantic_metadata.visibility = self.determine_visibility(node);
        block.semantic_metadata.modifiers = self.extract_modifiers(node)?.into_iter()
            .map(|s| self.string_to_modifier(&s))
            .collect();
        
        // Enhanced semantic metadata (Phase 1.2)
        block.semantic_metadata.parameter_details = Some(self.extract_parameter_details(node, &params)?);
        block.semantic_metadata.side_effect_analysis = Some(self.analyze_side_effects(node, &original_text)?);
        block.semantic_metadata.complexity_metrics = Some(self.calculate_complexity_metrics(node, &original_text)?);
        block.semantic_metadata.generics = Some(self.extract_generics(node)?);
        block.semantic_metadata.macros = Some(self.extract_macros(node)?);

        // Set position
        block.position = BlockPosition {
            start_line: node.start_position().row,
            end_line: node.end_position().row,
            start_column: node.start_position().column,
            end_column: node.end_position().column,
            index: self.blocks.len(),
        };

        self.blocks.push(block);
        Ok(())
    }

    fn visit_struct(&mut self, node: Node) -> Result<()> {
        let name = self.extract_struct_name(node)?;
        let original_text = node.utf8_text(self.source.as_bytes())?;
        
        let mut block = SemanticBlock::new(
            BlockType::Class,
            name.clone(),
            original_text.to_string(),
            "rust".to_string(),
        );

        // Set semantic identity
        block.semantic_identity.canonical_name = name.clone();
        block.semantic_identity.fully_qualified_name = self.build_qualified_name(&name);

        // Set syntax preservation
        block.syntax_preservation.original_text = original_text.to_string();
        block.syntax_preservation.normalized_ast = self.node_to_json(node)?;
        block.syntax_preservation.reconstruction_hints = ReconstructionHints {
            prefer_original: true,
            template: Some(original_text.to_string()),
            parameter_positions: vec![],
            body_extraction: BodyExtraction {
                method: "between_braces".to_string(),
                start_marker: "{".to_string(),
                preserve_indentation: true,
            },
        };

        // Set structural context
        block.structural_context.scope = if let Some(ref module) = self.current_module {
            ScopeInfo::Module(module.clone())
        } else {
            ScopeInfo::Module("main".to_string())
        };

        // Set semantic metadata
        block.semantic_metadata.visibility = self.determine_visibility(node);
        block.semantic_metadata.modifiers = self.extract_modifiers(node)?.into_iter()
            .map(|s| self.string_to_modifier(&s))
            .collect();
        
        // Enhanced semantic metadata (Phase 1.2)
        block.semantic_metadata.generics = Some(self.extract_generics(node)?);
        block.semantic_metadata.complexity_metrics = Some(self.calculate_complexity_metrics(node, &original_text)?);

        // Set position
        block.position = BlockPosition {
            start_line: node.start_position().row,
            end_line: node.end_position().row,
            start_column: node.start_position().column,
            end_column: node.end_position().column,
            index: self.blocks.len(),
        };

        self.blocks.push(block);
        Ok(())
    }

    fn visit_enum(&mut self, node: Node) -> Result<()> {
        let name = self.extract_enum_name(node)?;
        let original_text = node.utf8_text(self.source.as_bytes())?;
        
        let mut block = SemanticBlock::new(
            BlockType::Class,
            name.clone(),
            original_text.to_string(),
            "rust".to_string(),
        );

        // Set semantic identity
        block.semantic_identity.canonical_name = name.clone();
        block.semantic_identity.fully_qualified_name = self.build_qualified_name(&name);

        // Set syntax preservation
        block.syntax_preservation.original_text = original_text.to_string();
        block.syntax_preservation.normalized_ast = self.node_to_json(node)?;
        block.syntax_preservation.reconstruction_hints = ReconstructionHints {
            prefer_original: true,
            template: Some(original_text.to_string()),
            parameter_positions: vec![],
            body_extraction: BodyExtraction {
                method: "between_braces".to_string(),
                start_marker: "{".to_string(),
                preserve_indentation: true,
            },
        };

        // Set structural context
        block.structural_context.scope = if let Some(ref module) = self.current_module {
            ScopeInfo::Module(module.clone())
        } else {
            ScopeInfo::Module("main".to_string())
        };

        // Set semantic metadata
        block.semantic_metadata.visibility = self.determine_visibility(node);
        block.semantic_metadata.modifiers = self.extract_modifiers(node)?.into_iter()
            .map(|s| self.string_to_modifier(&s))
            .collect();
        
        // Enhanced semantic metadata (Phase 1.2)
        block.semantic_metadata.generics = Some(self.extract_generics(node)?);
        block.semantic_metadata.complexity_metrics = Some(self.calculate_complexity_metrics(node, &original_text)?);

        // Set position
        block.position = BlockPosition {
            start_line: node.start_position().row,
            end_line: node.end_position().row,
            start_column: node.start_position().column,
            end_column: node.end_position().column,
            index: self.blocks.len(),
        };

        self.blocks.push(block);
        Ok(())
    }

    fn visit_impl(&mut self, node: Node) -> Result<()> {
        let name = self.extract_impl_name(node)?;
        let original_text = node.utf8_text(self.source.as_bytes())?;
        
        let mut block = SemanticBlock::new(
            BlockType::Class,
            name.clone(),
            original_text.to_string(),
            "rust".to_string(),
        );

        // Set semantic identity
        block.semantic_identity.canonical_name = name.clone();
        block.semantic_identity.fully_qualified_name = self.build_qualified_name(&name);

        // Set syntax preservation
        block.syntax_preservation.original_text = original_text.to_string();
        block.syntax_preservation.normalized_ast = self.node_to_json(node)?;
        block.syntax_preservation.reconstruction_hints = ReconstructionHints {
            prefer_original: true,
            template: Some(original_text.to_string()),
            parameter_positions: vec![],
            body_extraction: BodyExtraction {
                method: "between_braces".to_string(),
                start_marker: "{".to_string(),
                preserve_indentation: true,
            },
        };

        // Set structural context
        block.structural_context.scope = if let Some(ref module) = self.current_module {
            ScopeInfo::Module(module.clone())
        } else {
            ScopeInfo::Module("main".to_string())
        };

        // Set semantic metadata
        block.semantic_metadata.visibility = self.determine_visibility(node);
        block.semantic_metadata.modifiers = self.extract_modifiers(node)?.into_iter()
            .map(|s| self.string_to_modifier(&s))
            .collect();
        
        // Enhanced semantic metadata (Phase 1.2)
        block.semantic_metadata.generics = Some(self.extract_generics(node)?);
        block.semantic_metadata.complexity_metrics = Some(self.calculate_complexity_metrics(node, &original_text)?);

        // Set position
        block.position = BlockPosition {
            start_line: node.start_position().row,
            end_line: node.end_position().row,
            start_column: node.start_position().column,
            end_column: node.end_position().column,
            index: self.blocks.len(),
        };

        self.blocks.push(block);
        Ok(())
    }

    fn visit_trait(&mut self, node: Node) -> Result<()> {
        let name = self.extract_trait_name(node)?;
        let original_text = node.utf8_text(self.source.as_bytes())?;
        
        let mut block = SemanticBlock::new(
            BlockType::Interface,
            name.clone(),
            original_text.to_string(),
            "rust".to_string(),
        );

        // Set semantic identity
        block.semantic_identity.canonical_name = name.clone();
        block.semantic_identity.fully_qualified_name = self.build_qualified_name(&name);

        // Set syntax preservation
        block.syntax_preservation.original_text = original_text.to_string();
        block.syntax_preservation.normalized_ast = self.node_to_json(node)?;
        block.syntax_preservation.reconstruction_hints = ReconstructionHints {
            prefer_original: true,
            template: Some(original_text.to_string()),
            parameter_positions: vec![],
            body_extraction: BodyExtraction {
                method: "between_braces".to_string(),
                start_marker: "{".to_string(),
                preserve_indentation: true,
            },
        };

        // Set structural context
        block.structural_context.scope = if let Some(ref module) = self.current_module {
            ScopeInfo::Module(module.clone())
        } else {
            ScopeInfo::Module("main".to_string())
        };

        // Set semantic metadata
        block.semantic_metadata.visibility = self.determine_visibility(node);
        block.semantic_metadata.modifiers = self.extract_modifiers(node)?.into_iter()
            .map(|s| self.string_to_modifier(&s))
            .collect();
        
        // Enhanced semantic metadata (Phase 1.2)
        block.semantic_metadata.generics = Some(self.extract_generics(node)?);
        block.semantic_metadata.complexity_metrics = Some(self.calculate_complexity_metrics(node, &original_text)?);

        // Set position
        block.position = BlockPosition {
            start_line: node.start_position().row,
            end_line: node.end_position().row,
            start_column: node.start_position().column,
            end_column: node.end_position().column,
            index: self.blocks.len(),
        };

        self.blocks.push(block);
        Ok(())
    }

    fn visit_macro(&mut self, node: Node) -> Result<()> {
        let name = self.extract_macro_name(node)?;
        let original_text = node.utf8_text(self.source.as_bytes())?;
        
        let mut block = SemanticBlock::new(
            BlockType::Function,
            name.clone(),
            original_text.to_string(),
            "rust".to_string(),
        );

        // Set semantic identity
        block.semantic_identity.canonical_name = name.clone();
        block.semantic_identity.fully_qualified_name = self.build_qualified_name(&name);

        // Set syntax preservation
        block.syntax_preservation.original_text = original_text.to_string();
        block.syntax_preservation.normalized_ast = self.node_to_json(node)?;
        block.syntax_preservation.reconstruction_hints = ReconstructionHints {
            prefer_original: true,
            template: Some(original_text.to_string()),
            parameter_positions: vec![],
            body_extraction: BodyExtraction {
                method: "between_braces".to_string(),
                start_marker: "{".to_string(),
                preserve_indentation: true,
            },
        };

        // Set structural context
        block.structural_context.scope = if let Some(ref module) = self.current_module {
            ScopeInfo::Module(module.clone())
        } else {
            ScopeInfo::Module("main".to_string())
        };

        // Set semantic metadata
        block.semantic_metadata.visibility = self.determine_visibility(node);
        block.semantic_metadata.modifiers = self.extract_modifiers(node)?.into_iter()
            .map(|s| self.string_to_modifier(&s))
            .collect();
        
        // Enhanced semantic metadata (Phase 1.2)
        block.semantic_metadata.macros = Some(self.extract_macro_info(node)?);
        block.semantic_metadata.complexity_metrics = Some(self.calculate_complexity_metrics(node, &original_text)?);

        // Set position
        block.position = BlockPosition {
            start_line: node.start_position().row,
            end_line: node.end_position().row,
            start_column: node.start_position().column,
            end_column: node.end_position().column,
            index: self.blocks.len(),
        };

        self.blocks.push(block);
        Ok(())
    }

    fn visit_use(&mut self, node: Node) -> Result<()> {
        let original_text = node.utf8_text(self.source.as_bytes())?;
        let import_name = self.extract_use_name(node)?;
        
        let mut block = SemanticBlock::new(
            BlockType::Import,
            import_name.clone(),
            original_text.to_string(),
            "rust".to_string(),
        );

        // Set semantic identity
        block.semantic_identity.canonical_name = import_name;
        block.semantic_identity.fully_qualified_name = Some(original_text.to_string());

        // Set syntax preservation
        block.syntax_preservation.original_text = original_text.to_string();
        block.syntax_preservation.normalized_ast = self.node_to_json(node)?;
        block.syntax_preservation.reconstruction_hints = ReconstructionHints {
            prefer_original: true,
            template: Some(original_text.to_string()),
            parameter_positions: vec![],
            body_extraction: BodyExtraction {
                method: "original".to_string(),
                start_marker: "".to_string(),
                preserve_indentation: true,
            },
        };

        // Set position
        block.position = BlockPosition {
            start_line: node.start_position().row,
            end_line: node.end_position().row,
            start_column: node.start_position().column,
            end_column: node.end_position().column,
            index: self.blocks.len(),
        };

        self.blocks.push(block);
        Ok(())
    }

    fn visit_module(&mut self, node: Node) -> Result<()> {
        let name = self.extract_module_name(node)?;
        let original_text = node.utf8_text(self.source.as_bytes())?;
        
        // Set current module context
        self.current_module = Some(name.clone());
        
        let mut block = SemanticBlock::new(
            BlockType::Module,
            name.clone(),
            original_text.to_string(),
            "rust".to_string(),
        );

        // Set semantic identity
        block.semantic_identity.canonical_name = name.clone();
        block.semantic_identity.fully_qualified_name = self.build_qualified_name(&name);

        // Set syntax preservation
        block.syntax_preservation.original_text = original_text.to_string();
        block.syntax_preservation.normalized_ast = self.node_to_json(node)?;
        block.syntax_preservation.reconstruction_hints = ReconstructionHints {
            prefer_original: true,
            template: Some(original_text.to_string()),
            parameter_positions: vec![],
            body_extraction: BodyExtraction {
                method: "between_braces".to_string(),
                start_marker: "{".to_string(),
                preserve_indentation: true,
            },
        };

        // Set structural context
        block.structural_context.scope = ScopeInfo::Module("main".to_string());

        // Set semantic metadata
        block.semantic_metadata.visibility = self.determine_visibility(node);
        block.semantic_metadata.modifiers = self.extract_modifiers(node)?.into_iter()
            .map(|s| self.string_to_modifier(&s))
            .collect();

        // Set position
        block.position = BlockPosition {
            start_line: node.start_position().row,
            end_line: node.end_position().row,
            start_column: node.start_position().column,
            end_column: node.end_position().column,
            index: self.blocks.len(),
        };

        self.blocks.push(block);
        Ok(())
    }

    // Extraction helper methods
    fn extract_function_name(&self, node: Node) -> Result<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                return Ok(child.utf8_text(self.source.as_bytes())?.to_string());
            }
        }
        Err(anyhow!("Could not find function name"))
    }

    fn extract_struct_name(&self, node: Node) -> Result<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "type_identifier" {
                return Ok(child.utf8_text(self.source.as_bytes())?.to_string());
            }
        }
        Err(anyhow!("Could not find struct name"))
    }

    fn extract_enum_name(&self, node: Node) -> Result<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "type_identifier" {
                return Ok(child.utf8_text(self.source.as_bytes())?.to_string());
            }
        }
        Err(anyhow!("Could not find enum name"))
    }

    fn extract_impl_name(&self, node: Node) -> Result<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "type_identifier" {
                return Ok(child.utf8_text(self.source.as_bytes())?.to_string());
            }
        }
        Err(anyhow!("Could not find impl name"))
    }

    fn extract_trait_name(&self, node: Node) -> Result<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "type_identifier" {
                return Ok(child.utf8_text(self.source.as_bytes())?.to_string());
            }
        }
        Err(anyhow!("Could not find trait name"))
    }

    fn extract_macro_name(&self, node: Node) -> Result<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                return Ok(child.utf8_text(self.source.as_bytes())?.to_string());
            }
        }
        Err(anyhow!("Could not find macro name"))
    }

    fn extract_use_name(&self, node: Node) -> Result<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "scoped_identifier" || child.kind() == "identifier" {
                return Ok(child.utf8_text(self.source.as_bytes())?.to_string());
            }
        }
        Err(anyhow!("Could not find use name"))
    }

    fn extract_module_name(&self, node: Node) -> Result<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                return Ok(child.utf8_text(self.source.as_bytes())?.to_string());
            }
        }
        Err(anyhow!("Could not find module name"))
    }

    fn extract_parameters(&self, node: Node) -> Result<Vec<Parameter>> {
        let mut params = Vec::new();
        let mut cursor = node.walk();
        
        for child in node.children(&mut cursor) {
            if child.kind() == "parameters" {
                let mut param_cursor = child.walk();
                for param_node in child.children(&mut param_cursor) {
                    if param_node.kind() == "parameter" {
                        let name = self.extract_parameter_name(param_node)?;
                        let type_hint = self.extract_parameter_type(param_node);
                        params.push(Parameter {
                            name,
                            type_hint,
                            default_value: None, // Rust doesn't have default parameters
                            is_optional: false,
                            position: params.len(),
                        });
                    }
                }
            }
        }
        
        Ok(params)
    }

    fn extract_parameter_name(&self, node: Node) -> Result<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                return Ok(child.utf8_text(self.source.as_bytes())?.to_string());
            }
        }
        Err(anyhow!("Could not find parameter name"))
    }

    fn extract_parameter_type(&self, node: Node) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "type_identifier" || child.kind() == "reference_type" {
                return Some(child.utf8_text(self.source.as_bytes()).unwrap_or("").to_string());
            }
        }
        None
    }

    fn determine_visibility(&self, node: Node) -> Visibility {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "visibility_modifier" {
                let text = child.utf8_text(self.source.as_bytes()).unwrap_or("");
                if text.contains("pub") {
                    return Visibility::Public;
                }
            }
        }
        Visibility::Private
    }

    fn extract_modifiers(&self, node: Node) -> Result<Vec<String>> {
        let mut modifiers = Vec::new();
        let mut cursor = node.walk();
        
        for child in node.children(&mut cursor) {
            match child.kind() {
                "visibility_modifier" => {
                    let text = child.utf8_text(self.source.as_bytes())?;
                    if text.contains("pub") {
                        modifiers.push("public".to_string());
                    }
                }
                "async" => modifiers.push("async".to_string()),
                "unsafe" => modifiers.push("unsafe".to_string()),
                "const" => modifiers.push("const".to_string()),
                "static" => modifiers.push("static".to_string()),
                _ => {}
            }
        }
        
        Ok(modifiers)
    }

    fn build_qualified_name(&self, name: &str) -> Option<String> {
        if let Some(ref module) = self.current_module {
            Some(format!("{}::{}", module, name))
        } else {
            Some(name.to_string())
        }
    }

    fn hash_signature(&self, text: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    fn node_to_json(&self, node: Node) -> Result<serde_json::Value> {
        let text = node.utf8_text(self.source.as_bytes())?;
        let start = node.start_position();
        let end = node.end_position();
        
        Ok(serde_json::json!({
            "type": node.kind(),
            "start": {
                "row": start.row,
                "column": start.column
            },
            "end": {
                "row": end.row,
                "column": end.column
            },
            "text": text,
        }))
    }

    fn into_blocks(self) -> Result<Vec<SemanticBlock>> {
        Ok(self.blocks)
    }
    
    // Enhanced extraction methods (Phase 1.2)
    
    fn extract_parameter_details(&self, _node: Node, params: &[Parameter]) -> Result<ParameterDetails> {
        let mut detailed_params = Vec::new();
        let default_values = HashMap::new();
        let mut type_constraints = HashMap::new();
        
        for (_i, param) in params.iter().enumerate() {
            let detailed_param = DetailedParameter {
                name: param.name.clone(),
                type_annotation: param.type_hint.as_ref().map(|hint| TypeInfo {
                    representation: hint.clone(),
                    is_generic: hint.contains('<'),
                    generic_args: self.extract_generic_args(hint),
                }),
                default_value: None, // Rust doesn't have default parameters
                is_optional: false,
                is_variadic: false, // Rust doesn't have variadic functions
                is_keyword_only: false,
                is_positional_only: false,
                documentation: None,
                validation_rules: vec![],
            };
            
            detailed_params.push(detailed_param);
            
            if let Some(hint) = &param.type_hint {
                type_constraints.insert(param.name.clone(), vec![hint.clone()]);
            }
        }
        
        Ok(ParameterDetails {
            parameters: detailed_params,
            variadic_info: None, // Rust doesn't have variadic functions
            default_values,
            type_constraints,
        })
    }
    
    fn extract_generic_args(&self, hint: &str) -> Vec<String> {
        // Extract generic arguments from Rust types
        if let Some(start) = hint.find('<') {
            if let Some(end) = hint.rfind('>') {
                let args_str = &hint[start + 1..end];
                return args_str.split(',').map(|s| s.trim().to_string()).collect();
            }
        }
        vec![]
    }
    
    fn analyze_side_effects(&self, node: Node, original_text: &str) -> Result<SideEffectAnalysis> {
        let mut side_effects = Vec::new();
        let mut dependencies = Vec::new();
        
        // Analyze the function body for side effects
        let body_text = self.extract_function_body_text(node, original_text)?;
        
        // Check for common side effects in Rust
        if body_text.contains("println!") || body_text.contains("eprintln!") {
            side_effects.push(SideEffect {
                effect_type: SideEffectType::ConsoleIO,
                description: "Console I/O operation".to_string(),
                severity: EffectSeverity::Low,
                line_number: None,
                confidence: 0.9,
            });
        }
        
        if body_text.contains("std::fs::") || body_text.contains("File::") {
            side_effects.push(SideEffect {
                effect_type: SideEffectType::FileIO,
                description: "File I/O operation".to_string(),
                severity: EffectSeverity::Medium,
                line_number: None,
                confidence: 0.9,
            });
        }
        
        if body_text.contains("reqwest::") || body_text.contains("hyper::") {
            side_effects.push(SideEffect {
                effect_type: SideEffectType::NetworkIO,
                description: "Network I/O operation".to_string(),
                severity: EffectSeverity::High,
                line_number: None,
                confidence: 0.9,
            });
        }
        
        if body_text.contains("std::thread::") || body_text.contains("tokio::") {
            side_effects.push(SideEffect {
                effect_type: SideEffectType::AsyncOperation,
                description: "Async operation".to_string(),
                severity: EffectSeverity::Medium,
                line_number: None,
                confidence: 0.9,
            });
        }
        
        if body_text.contains("panic!") || body_text.contains("unwrap()") {
            side_effects.push(SideEffect {
                effect_type: SideEffectType::ExceptionThrowing,
                description: "Exception throwing".to_string(),
                severity: EffectSeverity::High,
                line_number: None,
                confidence: 0.9,
            });
        }
        
        // Extract dependencies from use statements
        self.extract_dependencies_from_text(original_text, &mut dependencies);
        
        // Determine purity level
        let purity_level = if side_effects.is_empty() {
            PurityLevel::Pure
        } else if side_effects.iter().any(|se| matches!(se.effect_type, SideEffectType::FileIO | SideEffectType::NetworkIO)) {
            PurityLevel::Impure
        } else {
            PurityLevel::MostlyPure
        };
        
        Ok(SideEffectAnalysis {
            purity_level,
            side_effects,
            dependencies,
            mutability: self.analyze_mutability(node, original_text),
            resource_usage: self.analyze_resource_usage(node, original_text),
        })
    }
    
    fn extract_function_body_text(&self, _node: Node, original_text: &str) -> Result<String> {
        // Find the opening brace and extract everything after it
        if let Some(brace_pos) = original_text.find('{') {
            Ok(original_text[brace_pos + 1..].to_string())
        } else {
            Ok(String::new())
        }
    }
    
    fn extract_dependencies_from_text(&self, text: &str, dependencies: &mut Vec<Dependency>) {
        // Extract use statements
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with("use ") {
                if let Some(semicolon_pos) = line.find(';') {
                    let use_path = &line[4..semicolon_pos].trim();
                    if !use_path.is_empty() {
                        dependencies.push(Dependency {
                            name: use_path.to_string(),
                            version: None,
                            dependency_type: DependencyType::Import,
                            is_optional: false,
                        });
                    }
                }
            }
        }
    }
    
    fn analyze_mutability(&self, _node: Node, original_text: &str) -> MutabilityInfo {
        let is_mutable = original_text.contains("mut ") || original_text.contains("&mut");
        let mutates_self = original_text.contains("&mut self");
        let mutates_parameters = original_text.contains("mut ") && !original_text.contains("mut self");
        let mutates_globals = original_text.contains("static mut");
        
        MutabilityInfo {
            is_mutable,
            mutates_self,
            mutates_parameters,
            mutates_globals,
            mutates_externals: false,
        }
    }
    
    fn analyze_resource_usage(&self, _node: Node, original_text: &str) -> ResourceUsage {
        let mut io_operations = Vec::new();
        
        if original_text.contains("std::fs::") {
            io_operations.push(IoOperation {
                operation_type: IoType::Read,
                resource: "file".to_string(),
                is_blocking: true,
                estimated_latency: Some(1.0),
            });
        }
        
        if original_text.contains("reqwest::") {
            io_operations.push(IoOperation {
                operation_type: IoType::Network,
                resource: "http".to_string(),
                is_blocking: false, // async by default
                estimated_latency: Some(100.0),
            });
        }
        
        ResourceUsage {
            memory_usage: Some(MemoryUsage {
                estimated_bytes: Some(original_text.len()),
                allocation_type: AllocationType::Stack,
                is_leaked: false,
            }),
            cpu_usage: Some(CpuUsage {
                complexity: ComplexityLevel::Linear,
                estimated_operations: Some(original_text.lines().count()),
                is_optimizable: true,
            }),
            io_operations,
        }
    }
    
    fn calculate_complexity_metrics(&self, node: Node, original_text: &str) -> Result<ComplexityMetrics> {
        let lines_of_code = original_text.lines().count();
        let cyclomatic_complexity = self.calculate_cyclomatic_complexity(node);
        let cognitive_complexity = self.calculate_cognitive_complexity(node);
        let nesting_depth = self.calculate_nesting_depth(node);
        let branching_factor = self.calculate_branching_factor(node);
        
        // Simple maintainability index calculation
        let maintainability_index = 171.0 - 5.2 * (cyclomatic_complexity as f64).ln() 
            - 0.23 * (cognitive_complexity as f64).ln() 
            - 16.2 * (lines_of_code as f64).ln();
        
        Ok(ComplexityMetrics {
            cyclomatic_complexity,
            cognitive_complexity,
            lines_of_code,
            number_of_parameters: self.count_parameters(node),
            nesting_depth,
            branching_factor,
            maintainability_index: maintainability_index.max(0.0).min(100.0),
        })
    }
    
    fn calculate_cyclomatic_complexity(&self, node: Node) -> usize {
        let mut complexity = 1; // Base complexity
        let mut cursor = node.walk();
        
        for child in node.children(&mut cursor) {
            match child.kind() {
                "if_expression" | "while_expression" | "for_expression" | "loop_expression" => {
                    complexity += 1;
                }
                "match_arm" => {
                    complexity += 1;
                }
                _ => {}
            }
        }
        
        complexity
    }
    
    fn calculate_cognitive_complexity(&self, node: Node) -> usize {
        let mut complexity = 0;
        let mut nesting_level = 0;
        let mut cursor = node.walk();
        
        for child in node.children(&mut cursor) {
            match child.kind() {
                "if_expression" | "while_expression" | "for_expression" | "loop_expression" => {
                    complexity += 1 + nesting_level;
                    nesting_level += 1;
                }
                "match_arm" => {
                    complexity += 1;
                }
                _ => {}
            }
        }
        
        complexity
    }
    
    fn calculate_nesting_depth(&self, node: Node) -> usize {
        let mut max_depth = 0;
        let mut current_depth = 0;
        let mut cursor = node.walk();
        
        for child in node.children(&mut cursor) {
            match child.kind() {
                "if_expression" | "while_expression" | "for_expression" | "loop_expression" => {
                    current_depth += 1;
                    max_depth = max_depth.max(current_depth);
                }
                _ => {}
            }
        }
        
        max_depth
    }
    
    fn calculate_branching_factor(&self, node: Node) -> usize {
        let mut branches = 0;
        let mut cursor = node.walk();
        
        for child in node.children(&mut cursor) {
            match child.kind() {
                "if_expression" => {
                    branches += 1; // if
                    // Count else if and else
                    let mut child_cursor = child.walk();
                    for grandchild in child.children(&mut child_cursor) {
                        if grandchild.kind() == "else_clause" {
                            branches += 1;
                        }
                    }
                }
                "while_expression" | "for_expression" | "loop_expression" => {
                    branches += 1;
                }
                "match_expression" => {
                    let mut match_cursor = child.walk();
                    for arm in child.children(&mut match_cursor) {
                        if arm.kind() == "match_arm" {
                            branches += 1;
                        }
                    }
                }
                _ => {}
            }
        }
        
        branches
    }
    
    fn count_parameters(&self, node: Node) -> usize {
        let mut count = 0;
        let mut cursor = node.walk();
        
        for child in node.children(&mut cursor) {
            if child.kind() == "parameters" {
                let mut param_cursor = child.walk();
                for param in child.children(&mut param_cursor) {
                    if param.kind() == "parameter" {
                        count += 1;
                    }
                }
            }
        }
        
        count
    }
    
    fn extract_generics(&self, node: Node) -> Result<GenericInfo> {
        let mut generic_params = Vec::new();
        let mut constraints = Vec::new();
        let mut cursor = node.walk();
        
        for child in node.children(&mut cursor) {
            if child.kind() == "type_parameters" {
                let mut param_cursor = child.walk();
                for param in child.children(&mut param_cursor) {
                    if param.kind() == "type_parameter" {
                        let name = self.extract_generic_param_name(param)?;
                        let param_constraints = self.extract_generic_constraints(param);
                        
                        generic_params.push(GenericParameter {
                            name: name.clone(),
                            bounds: param_constraints.iter().map(|c| c.constraint_value.clone()).collect(),
                            constraints: param_constraints.iter().map(|c| c.constraint_value.clone()).collect(),
                            default_type: None,
                            variance: Some(Variance::Invariant),
                        });
                        
                        constraints.extend(param_constraints);
                    }
                }
            }
        }
        
        Ok(GenericInfo {
            generic_parameters: generic_params.clone(),
            parameters: generic_params,
            constraints,
            variance: Some(Variance::Invariant),
        })
    }
    
    fn extract_generic_param_name(&self, node: Node) -> Result<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "type_identifier" {
                return Ok(child.utf8_text(self.source.as_bytes())?.to_string());
            }
        }
        Err(anyhow!("Could not find generic parameter name"))
    }
    
    fn extract_generic_constraints(&self, node: Node) -> Vec<GenericConstraint> {
        let mut constraints = Vec::new();
        let mut cursor = node.walk();
        let param_name = self.extract_generic_param_name(node).unwrap_or_else(|_| "T".to_string());
        
        for child in node.children(&mut cursor) {
            if child.kind() == "trait_bounds" {
                let mut bound_cursor = child.walk();
                for bound in child.children(&mut bound_cursor) {
                    if bound.kind() == "trait_bound" {
                        let constraint_type = self.extract_constraint_type(bound);
                        constraints.push(GenericConstraint {
                            constraint_type,
                            parameter: param_name.clone(),
                            parameters: vec![param_name.clone()],
                            constraint_value: bound.utf8_text(self.source.as_bytes()).unwrap_or("").to_string(),
                        });
                    }
                }
            }
        }
        
        constraints
    }
    
    fn extract_constraint_type(&self, node: Node) -> ConstraintType {
        let text = node.utf8_text(self.source.as_bytes()).unwrap_or("");
        if text.contains("Clone") {
            ConstraintType::Clone
        } else if text.contains("Copy") {
            ConstraintType::Copy
        } else if text.contains("Send") {
            ConstraintType::Send
        } else if text.contains("Sync") {
            ConstraintType::Sync
        } else {
            ConstraintType::Other(text.to_string())
        }
    }
    
    fn extract_macros(&self, node: Node) -> Result<MacroInfo> {
        let mut macros = Vec::new();
        let mut cursor = node.walk();
        
        for child in node.children(&mut cursor) {
            if child.kind() == "macro_definition" {
                let name = self.extract_macro_name(child)?;
                let _macro_type = self.determine_macro_type(child);
                let _parameters = self.extract_macro_parameters(child);
                
                macros.push(MacroParameter {
                    name,
                    parameter_type: MacroParamType::Expr,
                    is_repeatable: false,
                    is_optional: false,
                    default_value: None,
                    separator: None,
                });
            }
        }
        
        Ok(MacroInfo {
            macro_name: "unknown".to_string(),
            macro_type: MacroType::Declarative,
            parameters: macros,
            expansion: None,
            hygiene: HygieneLevel::Hygienic,
            hygiene_level: HygieneLevel::Hygienic,
            is_procedural: false,
        })
    }
    
    fn determine_macro_type(&self, _node: Node) -> MacroType {
        MacroType::Declarative
    }
    
    fn extract_macro_parameters(&self, node: Node) -> Vec<MacroParameter> {
        let mut params = Vec::new();
        let mut cursor = node.walk();
        
        for child in node.children(&mut cursor) {
            if child.kind() == "macro_parameter" {
                let name = child.utf8_text(self.source.as_bytes()).unwrap_or("").to_string();
                params.push(MacroParameter {
                    name,
                    parameter_type: MacroParamType::Expression,
                    is_repeatable: false,
                    is_optional: false,
                    default_value: None,
                    separator: None,
                });
            }
        }
        
        params
    }
    
    fn extract_macro_info(&self, node: Node) -> Result<MacroInfo> {
        let name = self.extract_macro_name(node)?;
        let _macro_type = self.determine_macro_type(node);
        let parameters = self.extract_macro_parameters(node);
        
        Ok(MacroInfo {
            macro_name: name,
            macro_type: MacroType::Declarative,
            parameters,
            expansion: None,
            hygiene: HygieneLevel::Hygienic,
            hygiene_level: HygieneLevel::Hygienic,
            is_procedural: false,
        })
    }
}
