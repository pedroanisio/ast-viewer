use std::collections::HashMap;
use uuid::Uuid;
use anyhow::Result;
use crate::database::{Database, Block};

pub struct HierarchicalGenerator {
    blocks: Vec<Block>,
    root_blocks: Vec<Uuid>,
    children_map: HashMap<Uuid, Vec<Uuid>>,
    language: String,
}

#[allow(dead_code)]
pub struct GenerationContext {
    language: String,
    indent_size: usize,
    current_imports: Vec<String>,
}

impl HierarchicalGenerator {
    pub async fn from_container(db: &Database, container_id: Uuid) -> Result<Self> {
        let blocks = db.get_blocks_by_container(container_id).await?;
        
        // Get container to extract language
        let container = db.get_container_by_id(container_id).await?;
        let language = container.language.unwrap_or_else(|| "unknown".to_string());
        
        let mut root_blocks = Vec::new();
        let mut children_map: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        
        // Build hierarchy
        for block in &blocks {
            if let Some(parent_id) = block.parent_block_id {
                children_map.entry(parent_id)
                    .or_insert_with(Vec::new)
                    .push(block.id);
            } else {
                root_blocks.push(block.id);
            }
        }
        
        // Sort children by position
        for children in children_map.values_mut() {
            children.sort_by_key(|&id| {
                blocks.iter()
                    .find(|b| b.id == id)
                    .map(|b| b.position_in_parent)
                    .unwrap_or(0)
            });
        }
        
        Ok(Self {
            blocks,
            root_blocks,
            children_map,
            language,
        })
    }
    
    pub fn generate(&self) -> Result<String> {
        let mut output = Vec::new();
        let mut context = GenerationContext::new(&self.language);
        
        // Phase 1: Collect and group imports
        let imports = self.collect_by_type("Import");
        if !imports.is_empty() {
            output.push(self.generate_imports(&imports, &mut context)?);
            output.push(String::new()); // Empty line after imports
        }
        
        // Phase 2: Generate top-level code
        for &root_id in &self.root_blocks {
            if let Some(block) = self.find_block(root_id) {
                if block.block_type != "Import" {
                    self.generate_recursive(block, 0, &mut output, &mut context)?;
                    output.push(String::new()); // Empty line between top-level blocks
                }
            }
        }
        
        // Remove trailing empty lines
        while output.last() == Some(&String::new()) {
            output.pop();
        }
        
        Ok(output.join("\n"))
    }
    
    fn generate_recursive(&self, block: &Block, depth: usize, output: &mut Vec<String>, _ctx: &mut GenerationContext) -> Result<()> {
        // Generate block opening
        let indent = self.get_indent(depth);
        let opening = self.generate_block_opening(block, &indent, _ctx)?;
        if !opening.is_empty() {
            output.push(opening);
        }
        
        // Generate children
        if let Some(children_ids) = self.children_map.get(&block.id) {
            for &child_id in children_ids {
                if let Some(child) = self.find_block(child_id) {
                    self.generate_recursive(child, depth + 1, output, _ctx)?;
                }
            }
        }
        
        // Generate block closing (if needed)
        if let Some(closing) = self.generate_block_closing(block, &indent)? {
            output.push(closing);
        }
        
        Ok(())
    }
    
    fn generate_block_opening(&self, block: &Block, indent: &str, _ctx: &mut GenerationContext) -> Result<String> {
        match self.language.as_str() {
            "python" => self.generate_python_opening(block, indent, _ctx),
            "javascript" | "typescript" | "tsx" => self.generate_js_opening(block, indent, _ctx),
            "rust" => self.generate_rust_opening(block, indent, _ctx),
            _ => Ok(format!("{}// {}: {}", indent, block.block_type, block.semantic_name.as_ref().unwrap_or(&"unknown".to_string())))
        }
    }
    
    fn generate_block_closing(&self, block: &Block, indent: &str) -> Result<Option<String>> {
        match self.language.as_str() {
            "javascript" | "typescript" | "tsx" => {
                match block.block_type.as_str() {
                    "Function" | "Class" => Ok(Some(format!("{}}}", indent))),
                    _ => Ok(None)
                }
            },
            "rust" => {
                match block.block_type.as_str() {
                    "Function" | "Class" => Ok(Some(format!("{}}}", indent))),
                    _ => Ok(None)
                }
            },
            _ => Ok(None)
        }
    }
    
    fn generate_python_opening(&self, block: &Block, indent: &str, _ctx: &mut GenerationContext) -> Result<String> {
        match block.block_type.as_str() {
            "Function" => {
                let params = self.extract_parameters(block)?;
                let default_name = "unnamed".to_string();
                let name = block.semantic_name.as_ref().unwrap_or(&default_name);
                Ok(format!("{}def {}({}):", indent, name, params))
            },
            "Class" => {
                let default_name = "UnnamedClass".to_string();
                let name = block.semantic_name.as_ref().unwrap_or(&default_name);
                let bases = self.extract_base_classes(block)?;
                if bases.is_empty() {
                    Ok(format!("{}class {}:", indent, name))
                } else {
                    Ok(format!("{}class {}({}):", indent, name, bases.join(", ")))
                }
            },
            "Import" => {
                // Use original text for imports to preserve exact syntax
                let original = self.extract_original_text(block)?;
                Ok(format!("{}{}", indent, original.trim()))
            },
            "Variable" => {
                let original = self.extract_original_text(block)?;
                Ok(format!("{}{}", indent, original.trim()))
            },
            _ => {
                let original = self.extract_original_text(block)?;
                Ok(format!("{}{}", indent, original.trim()))
            }
        }
    }
    
    fn generate_js_opening(&self, block: &Block, indent: &str, _ctx: &mut GenerationContext) -> Result<String> {
        match block.block_type.as_str() {
            "Function" => {
                let params = self.extract_parameters(block)?;
                let default_name = "anonymous".to_string();
                let name = block.semantic_name.as_ref().unwrap_or(&default_name);
                let modifiers = self.extract_modifiers(block)?;
                let modifier_str = if modifiers.is_empty() { String::new() } else { format!("{} ", modifiers.join(" ")) };
                
                if name == "anonymous" {
                    Ok(format!("{}{}({}) {{", indent, modifier_str, params))
                } else {
                    Ok(format!("{}{}function {}({}) {{", indent, modifier_str, name, params))
                }
            },
            "Class" => {
                let default_name = "UnnamedClass".to_string();
                let name = block.semantic_name.as_ref().unwrap_or(&default_name);
                let extends = self.extract_extends_clause(block)?;
                if extends.is_empty() {
                    Ok(format!("{}class {} {{", indent, name))
                } else {
                    Ok(format!("{}class {} extends {} {{", indent, name, extends))
                }
            },
            "Import" => {
                let original = self.extract_original_text(block)?;
                Ok(format!("{}{}", indent, original.trim()))
            },
            "Export" => {
                let original = self.extract_original_text(block)?;
                Ok(format!("{}{}", indent, original.trim()))
            },
            "Variable" => {
                let original = self.extract_original_text(block)?;
                Ok(format!("{}{}", indent, original.trim()))
            },
            _ => {
                let original = self.extract_original_text(block)?;
                Ok(format!("{}{}", indent, original.trim()))
            }
        }
    }
    
    fn generate_rust_opening(&self, block: &Block, indent: &str, _ctx: &mut GenerationContext) -> Result<String> {
        match block.block_type.as_str() {
            "Function" => {
                let params = self.extract_parameters(block)?;
                let default_name = "unnamed".to_string();
                let name = block.semantic_name.as_ref().unwrap_or(&default_name);
                let return_type = self.extract_return_type(block)?;
                let return_str = if return_type.is_empty() { String::new() } else { format!(" -> {}", return_type) };
                Ok(format!("{}fn {}({}){} {{", indent, name, params, return_str))
            },
            "Class" => {
                let default_name = "UnnamedStruct".to_string();
                let name = block.semantic_name.as_ref().unwrap_or(&default_name);
                if name.starts_with("impl ") {
                    Ok(format!("{}{} {{", indent, name))
                } else {
                    Ok(format!("{}struct {} {{", indent, name))
                }
            },
            "Import" => {
                let original = self.extract_original_text(block)?;
                Ok(format!("{}{}", indent, original.trim()))
            },
            _ => {
                let original = self.extract_original_text(block)?;
                Ok(format!("{}{}", indent, original.trim()))
            }
        }
    }
    
    fn generate_imports(&self, imports: &[&Block], _ctx: &mut GenerationContext) -> Result<String> {
        let mut import_lines = Vec::new();
        
        for import in imports {
            let original = self.extract_original_text(import)?;
            import_lines.push(original.trim().to_string());
        }
        
        // Sort imports for consistency
        import_lines.sort();
        
        Ok(import_lines.join("\n"))
    }
    
    fn collect_by_type(&self, block_type: &str) -> Vec<&Block> {
        self.blocks.iter()
            .filter(|b| b.block_type == block_type)
            .collect()
    }
    
    fn find_block(&self, id: Uuid) -> Option<&Block> {
        self.blocks.iter().find(|b| b.id == id)
    }
    
    fn get_indent(&self, depth: usize) -> String {
        match self.language.as_str() {
            "python" => "    ".repeat(depth), // 4 spaces
            "javascript" | "typescript" | "tsx" => "  ".repeat(depth), // 2 spaces
            "rust" => "    ".repeat(depth), // 4 spaces
            _ => "    ".repeat(depth)
        }
    }
    
    // Helper methods to extract information from blocks
    
    fn extract_parameters(&self, block: &Block) -> Result<String> {
        if let Some(params) = &block.parameters {
            if let Some(param_array) = params.as_array() {
                let param_strings: Vec<String> = param_array.iter()
                    .filter_map(|p| {
                        p.get("name").and_then(|n| n.as_str()).map(|s| s.to_string())
                    })
                    .collect();
                return Ok(param_strings.join(", "));
            }
        }
        Ok(String::new())
    }
    
    fn extract_return_type(&self, block: &Block) -> Result<String> {
        Ok(block.return_type.as_ref().unwrap_or(&String::new()).clone())
    }
    
    fn extract_modifiers(&self, block: &Block) -> Result<Vec<String>> {
        if let Some(modifiers) = &block.modifiers {
            Ok(modifiers.clone())
        } else {
            Ok(Vec::new())
        }
    }
    
    fn extract_base_classes(&self, _block: &Block) -> Result<Vec<String>> {
        // This would need to be extracted from relationships or metadata
        // For now, return empty
        Ok(Vec::new())
    }
    
    fn extract_extends_clause(&self, _block: &Block) -> Result<String> {
        // This would need to be extracted from relationships or metadata
        // For now, return empty
        Ok(String::new())
    }
    
    fn extract_original_text(&self, block: &Block) -> Result<String> {
        // ✅ ENHANCED: First priority - use preserved implementation data
        if let Some(implementation) = block.abstract_syntax.get("implementation") {
            // For variables, use the preserved variable assignments
            if block.block_type == "Variable" {
                if let Some(assignments) = implementation.get("variable_assignments") {
                    let semantic_name = block.semantic_name.as_deref().unwrap_or("unnamed");
                    if let Some(assignment_info) = assignments.get(semantic_name) {
                        if let Some(literal_value) = assignment_info.get("literal_value") {
                            // Use the actual preserved value
                            let value_str = match literal_value {
                                serde_json::Value::String(s) => format!("\"{}\"", s),
                                serde_json::Value::Number(n) => n.to_string(),
                                serde_json::Value::Bool(b) => if *b { "True".to_string() } else { "False".to_string() },
                                serde_json::Value::Null => "None".to_string(),
                                _ => assignment_info.get("expression")
                                    .and_then(|e| e.as_str())
                                    .unwrap_or("None")
                                    .to_string(),
                            };
                            return Ok(format!("{} = {}", semantic_name, value_str));
                        } else if let Some(expression) = assignment_info.get("expression") {
                            // Use the preserved expression
                            let expr_str = expression.as_str().unwrap_or("None");
                            return Ok(format!("{} = {}", semantic_name, expr_str));
                        }
                    }
                }
            }
            // For functions, use the preserved original body
            else if block.block_type == "Function" {
                if let Some(original_body) = implementation.get("original_body") {
                    if let Some(body_str) = original_body.as_str() {
                        if !body_str.trim().is_empty() {
                            return Ok(body_str.to_string());
                        }
                    }
                }
            }
            // For other types, use original_text if available
            if let Some(original_text) = implementation.get("original_text") {
                if let Some(text_str) = original_text.as_str() {
                    if !text_str.trim().is_empty() {
                        return Ok(text_str.to_string());
                    }
                }
            }
        }
        
        // Try to get original text from abstract_syntax (legacy)
        if let Some(raw_text) = block.abstract_syntax.get("raw_text") {
            if let Some(text) = raw_text.as_str() {
                return Ok(text.to_string());
            }
        }
        
        // Try to get from metadata if available (legacy)
        if let Some(metadata) = &block.metadata {
            if let Some(source) = metadata.get("source_code") {
                if let Some(text) = source.as_str() {
                    return Ok(text.to_string());
                }
            }
        }
        
        // Fallback to reconstructing from semantic name and parameters
        let default_name = "unnamed".to_string();
        let name = block.semantic_name.as_ref().unwrap_or(&default_name);
        
        match block.block_type.as_str() {
            "Function" => {
                let params = self.extract_parameters(block).unwrap_or_default();
                Ok(format!("def {}({}):\n    pass", name, params))
            },
            "Class" => {
                Ok(format!("class {}:\n    pass", name))
            },
            "Variable" => {
                Ok(format!("{} = None", name))
            },
            "Import" => {
                Ok(format!("import {}", name))
            },
            _ => Ok(format!("pass  # {}: {}", block.block_type, name))
        }
    }
}

impl GenerationContext {
    pub fn new(language: &str) -> Self {
        Self {
            language: language.to_string(),
            indent_size: match language {
                "python" => 4,
                "javascript" | "typescript" | "tsx" => 2,
                "rust" => 4,
                _ => 4,
            },
            current_imports: Vec::new(),
        }
    }
}
