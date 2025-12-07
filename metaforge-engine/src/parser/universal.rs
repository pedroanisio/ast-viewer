use tree_sitter::{Parser, Tree, Node};
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use super::extractors::{PythonExtractor, JavaScriptExtractor, RustExtractor};
use super::extraction_context::{ParseResult, LanguageExtractor};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalBlock {
    pub id: Uuid,
    pub block_type: BlockType,
    pub semantic_name: String,
    pub abstract_syntax: AbstractSyntax,
    pub position: usize,
    pub indent_level: usize,
    pub source_language: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbstractSyntax {
    pub semantic_type: String,
    pub raw_text: String,
    pub simplified: SimplifiedNode,
    pub tree_sitter_ast: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplifiedNode {
    pub kind: String,
    pub name: Option<String>,
    pub params: Vec<String>,
    pub body: Option<String>,
    pub modifiers: Vec<String>,
}

pub struct UniversalParser {
    parsers: HashMap<String, Parser>,
    python_extractor: PythonExtractor,
    javascript_extractor: JavaScriptExtractor,
    rust_extractor: RustExtractor,
}

#[allow(dead_code)]
impl UniversalParser {
    pub fn new() -> Result<Self> {
        let mut parsers = HashMap::new();
        
        // Initialize core language parsers for MVP
        let languages = vec![
            ("python", tree_sitter_python::language()),
            ("javascript", tree_sitter_javascript::language()),
            ("typescript", tree_sitter_typescript::language_typescript()),
            ("tsx", tree_sitter_typescript::language_tsx()),
            ("rust", tree_sitter_rust::language()),
        ];
        
        for (name, language) in languages {
            let mut parser = Parser::new();
            parser.set_language(language)?;
            parsers.insert(name.to_string(), parser);
        }
        
        Ok(Self { 
            parsers,
            python_extractor: PythonExtractor,
            javascript_extractor: JavaScriptExtractor { is_typescript: false },
            rust_extractor: RustExtractor,
        })
    }
    
    pub fn parse_file(&mut self, content: &str, language: &str, file_path: &str) -> Result<ParseResult> {
        let parser = self.parsers.get_mut(language)
            .ok_or_else(|| anyhow!("Unsupported language: {}", language))?;
        
        let tree = parser.parse(content, None)
            .ok_or_else(|| anyhow!("Failed to parse file"))?;
        
        // Single extraction path - no duplication
        let mut extraction_result = match language {
            "python" => self.python_extractor.extract_with_context(tree.root_node(), content, file_path)?,
            "javascript" => self.javascript_extractor.extract_with_context(tree.root_node(), content, file_path)?,
            "typescript" => {
                let ts_extractor = JavaScriptExtractor { is_typescript: true };
                ts_extractor.extract_with_context(tree.root_node(), content, file_path)?
            },
            "tsx" => {
                let tsx_extractor = JavaScriptExtractor { is_typescript: true };
                tsx_extractor.extract_with_context(tree.root_node(), content, file_path)?
            },
            "rust" => self.rust_extractor.extract_with_context(tree.root_node(), content, file_path)?,
            _ => return Err(anyhow!("No extractor for language: {}", language))
        };
        
        // Second pass: resolve relationships
        extraction_result.resolve_relationships();
        
        Ok(extraction_result)
    }
    
    // Legacy method for backward compatibility - converts to old format
    pub fn parse_file_legacy(&mut self, content: &str, language: &str, file_path: &str) -> Result<Vec<UniversalBlock>> {
        let parse_result = self.parse_file(content, language, file_path)?;
        
        // Convert SemanticBlock to UniversalBlock for compatibility
        let universal_blocks: Vec<UniversalBlock> = parse_result.blocks.into_iter().map(|sb| {
            let block_type_str = format!("{:?}", sb.block_type);
            let canonical_name = sb.semantic_identity.canonical_name.clone();
            let source_language = sb.source_language.clone();
            let fqn = sb.semantic_identity.fully_qualified_name.clone();
            let inheritance_chain = sb.structural_context.inheritance_chain.clone();
            let decorators = sb.structural_context.decorators.clone();
            
            UniversalBlock {
                id: sb.id,
                block_type: self.convert_block_type_back(sb.block_type),
                semantic_name: canonical_name.clone(),
                abstract_syntax: AbstractSyntax {
                    semantic_type: block_type_str.clone(),
                    raw_text: sb.syntax_preservation.original_text,
                    simplified: SimplifiedNode {
                        kind: block_type_str,
                        name: Some(canonical_name.clone()),
                        params: sb.semantic_metadata.parameters.iter().map(|p| p.name.clone()).collect(),
                        body: None,
                        modifiers: sb.semantic_metadata.modifiers.iter().map(|m| format!("{:?}", m)).collect(),
                    },
                    tree_sitter_ast: Some(sb.syntax_preservation.normalized_ast),
                },
                position: sb.position.index,
                indent_level: 0,
                source_language: source_language.clone(),
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("language".to_string(), serde_json::Value::String(source_language));
                    meta.insert("canonical_name".to_string(), serde_json::Value::String(canonical_name));
                    if let Some(fqn) = fqn {
                        meta.insert("fully_qualified_name".to_string(), serde_json::Value::String(fqn));
                    }
                    meta.insert("inheritance_chain".to_string(), serde_json::to_value(inheritance_chain).unwrap());
                    meta.insert("decorators".to_string(), serde_json::to_value(decorators).unwrap());
                    meta
                },
            }
        }).collect();
        
        Ok(universal_blocks)
    }
    
    fn extract_blocks(&self, tree: &Tree, source: &str, language: &str, _file_path: &str) -> Result<Vec<UniversalBlock>> {
        let mut blocks = Vec::new();
        let mut position = 0;
        
        self.visit_node(tree.root_node(), source, language, &mut blocks, &mut position, 0)?;
        
        Ok(blocks)
    }
    
    fn visit_node(
        &self,
        node: Node,
        source: &str,
        language: &str,
        blocks: &mut Vec<UniversalBlock>,
        position: &mut usize,
        indent_level: usize,
    ) -> Result<()> {
        // Extract semantic block if this node represents one
        if let Some(block) = self.node_to_block(node, source, language, *position, indent_level)? {
            blocks.push(block);
            *position += 1;
        }
        
        // Recursively visit children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_node(child, source, language, blocks, position, indent_level + 1)?;
        }
        
        Ok(())
    }
    
    fn node_to_block(
        &self,
        node: Node,
        source: &str,
        language: &str,
        position: usize,
        indent_level: usize,
    ) -> Result<Option<UniversalBlock>> {
        let node_kind = node.kind();
        
        // Map node types to semantic blocks based on language
        let block_type = match (language, node_kind) {
            // Python
            ("python", "function_definition") => Some(BlockType::Function),
            ("python", "class_definition") => Some(BlockType::Class),
            ("python", "import_statement") | ("python", "import_from_statement") => Some(BlockType::Import),
            
            // JavaScript/TypeScript
            ("javascript" | "typescript" | "tsx", "function_declaration") => Some(BlockType::Function),
            ("javascript" | "typescript" | "tsx", "arrow_function") => Some(BlockType::Function),
            ("javascript" | "typescript" | "tsx", "class_declaration") => Some(BlockType::Class),
            ("javascript" | "typescript" | "tsx", "import_statement") => Some(BlockType::Import),
            
            // Rust
            ("rust", "function_item") => Some(BlockType::Function),
            ("rust", "struct_item") => Some(BlockType::Class),
            ("rust", "enum_item") => Some(BlockType::Class),
            ("rust", "impl_item") => Some(BlockType::Class),
            ("rust", "trait_item") => Some(BlockType::Interface),
            ("rust", "macro_definition") => Some(BlockType::Function),
            ("rust", "use_declaration") => Some(BlockType::Import),
            ("rust", "mod_item") => Some(BlockType::Module),
            ("javascript" | "typescript" | "tsx", "export_statement") => Some(BlockType::Export),
            ("typescript" | "tsx", "interface_declaration") => Some(BlockType::Interface),
            ("typescript" | "tsx", "type_alias_declaration") => Some(BlockType::TypeDef),
            
            // Common control structures
            (_, "if_statement") | (_, "if_expression") => Some(BlockType::Conditional),
            (_, "for_statement") | (_, "while_statement") | (_, "loop_expression") => Some(BlockType::Loop),
            (_, "try_statement") | (_, "try_expression") => Some(BlockType::TryCatch),
            
            _ => None,
        };
        
        if let Some(block_type) = block_type {
            let text = node.utf8_text(source.as_bytes())?;
            let semantic_name = self.extract_name(node, source, language);
            
            let block = UniversalBlock {
                id: Uuid::new_v4(),
                block_type,
                semantic_name,
                abstract_syntax: AbstractSyntax {
                    semantic_type: node_kind.to_string(),
                    raw_text: text.to_string(),
                    simplified: self.simplify_node(node, source, language)?,
                    tree_sitter_ast: Some(self.node_to_json(node, source)?),
                },
                position,
                indent_level,
                source_language: language.to_string(),
                metadata: self.extract_metadata(node, source, language),
            };
            
            Ok(Some(block))
        } else {
            Ok(None)
        }
    }
    
    fn extract_name(&self, node: Node, source: &str, language: &str) -> String {
        // Language-specific name extraction
        let mut cursor = node.walk();
        
        for child in node.children(&mut cursor) {
            match (language, child.kind()) {
                (_, "identifier") | (_, "property_identifier") => {
                    if let Ok(name) = child.utf8_text(source.as_bytes()) {
                        return name.to_string();
                    }
                }
                _ => {}
            }
        }
        
        "anonymous".to_string()
    }
    
    fn simplify_node(&self, node: Node, source: &str, language: &str) -> Result<SimplifiedNode> {
        let mut params = Vec::new();
        let mut modifiers = Vec::new();
        
        // Extract parameters for functions
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind().contains("parameter") || child.kind() == "formal_parameters" {
                let param_text = child.utf8_text(source.as_bytes())?;
                params.push(param_text.to_string());
            }
            
            // Extract modifiers (async, public, static, etc.)
            if child.kind() == "async" || child.kind() == "static" || 
               child.kind() == "public" || child.kind() == "private" {
                modifiers.push(child.kind().to_string());
            }
        }
        
        Ok(SimplifiedNode {
            kind: node.kind().to_string(),
            name: Some(self.extract_name(node, source, language)),
            params,
            body: None,  // We store the full text separately
            modifiers,
        })
    }
    
    fn node_to_json(&self, node: Node, source: &str) -> Result<serde_json::Value> {
        let text = node.utf8_text(source.as_bytes())?;
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
    
    fn extract_metadata(&self, node: Node, source: &str, language: &str) -> HashMap<String, serde_json::Value> {
        let mut metadata = HashMap::new();
        
        // Add language-specific metadata
        metadata.insert("language".to_string(), serde_json::Value::String(language.to_string()));
        metadata.insert("node_kind".to_string(), serde_json::Value::String(node.kind().to_string()));
        
        // Check for specific patterns
        if let Ok(text) = node.utf8_text(source.as_bytes()) {
            // React component detection
            if language == "javascript" || language == "typescript" || language == "tsx" {
                if text.contains("React.Component") || text.contains("useState") || text.contains("useEffect") {
                    metadata.insert("is_react_component".to_string(), serde_json::Value::Bool(true));
                }
            }
            
            // Async detection
            if text.contains("async") || text.contains("await") {
                metadata.insert("is_async".to_string(), serde_json::Value::Bool(true));
            }
        }
        
        metadata
    }
    
    fn convert_block_type(&self, block_type: BlockType) -> crate::core::BlockType {
        match block_type {
            BlockType::Function => crate::core::BlockType::Function,
            BlockType::Class => crate::core::BlockType::Class,
            BlockType::Interface => crate::core::BlockType::Interface,
            BlockType::Variable => crate::core::BlockType::Variable,
            BlockType::Import => crate::core::BlockType::Import,
            BlockType::Export => crate::core::BlockType::Export,
            BlockType::Conditional => crate::core::BlockType::Conditional,
            BlockType::Loop => crate::core::BlockType::Loop,
            BlockType::TryCatch => crate::core::BlockType::TryCatch,
            BlockType::Comment => crate::core::BlockType::Comment,
            BlockType::TypeDef => crate::core::BlockType::TypeDef,
            BlockType::Component => crate::core::BlockType::Component,
            BlockType::Query => crate::core::BlockType::Query,
            BlockType::Config => crate::core::BlockType::Config,
            BlockType::Module => crate::core::BlockType::Module,
        }
    }
    
    fn convert_block_type_back(&self, block_type: crate::core::BlockType) -> BlockType {
        match block_type {
            crate::core::BlockType::Function => BlockType::Function,
            crate::core::BlockType::Class => BlockType::Class,
            crate::core::BlockType::Interface => BlockType::Interface,
            crate::core::BlockType::Variable => BlockType::Variable,
            crate::core::BlockType::Import => BlockType::Import,
            crate::core::BlockType::Export => BlockType::Export,
            crate::core::BlockType::Conditional => BlockType::Conditional,
            crate::core::BlockType::Loop => BlockType::Loop,
            crate::core::BlockType::TryCatch => BlockType::TryCatch,
            crate::core::BlockType::Comment => BlockType::Comment,
            crate::core::BlockType::TypeDef => BlockType::TypeDef,
            crate::core::BlockType::Component => BlockType::Component,
            crate::core::BlockType::Query => BlockType::Query,
            crate::core::BlockType::Config => BlockType::Config,
            crate::core::BlockType::Module => BlockType::Module,
        }
    }
}
