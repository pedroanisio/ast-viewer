use super::super::*;

pub struct PythonGenerator;

impl PythonGenerator {
    pub fn new() -> Self {
        Self
    }
    
    fn indent(&self, level: usize, config: &GenerationConfig) -> String {
        if config.use_tabs {
            "\t".repeat(level)
        } else {
            " ".repeat(level * config.indent_size)
        }
    }
}

impl LanguageGenerator for PythonGenerator {
    fn generate(
        &self,
        _container: &Container,
        blocks: &[GenerationBlock],
        config: &GenerationConfig,
    ) -> Result<String> {
        let mut lines = Vec::new();
        
        // Group blocks by type for better organization
        let mut imports = Vec::new();
        let mut globals = Vec::new();
        let mut functions = Vec::new();
        let mut classes = Vec::new();
        let mut main_code = Vec::new();
        
        for block in blocks {
            match block.block_type.as_str() {
                "Import" => imports.push(block),
                "Class" => classes.push(block),
                "Function" => functions.push(block),
                "Variable" if block.indent_level == 0 => globals.push(block),
                _ => main_code.push(block),
            }
        }
        
        // Generate imports
        if !imports.is_empty() {
            if config.group_imports {
                // Group imports by type
                let mut standard_imports = Vec::new();
                let mut third_party_imports = Vec::new();
                let mut local_imports = Vec::new();
                
                for import in imports {
                    let import_line = self.generate_import(import)?;
                    if self.is_standard_library(&import.semantic_name) {
                        standard_imports.push(import_line);
                    } else if import_line.starts_with("from .") {
                        local_imports.push(import_line);
                    } else {
                        third_party_imports.push(import_line);
                    }
                }
                
                // Add grouped imports
                if !standard_imports.is_empty() {
                    lines.extend(standard_imports);
                    lines.push(String::new());
                }
                if !third_party_imports.is_empty() {
                    lines.extend(third_party_imports);
                    lines.push(String::new());
                }
                if !local_imports.is_empty() {
                    lines.extend(local_imports);
                    lines.push(String::new());
                }
            } else {
                for import in imports {
                    lines.push(self.generate_import(import)?);
                }
                lines.push(String::new());
            }
        }
        
        // Generate global variables
        for global in &globals {
            lines.push(self.generate_variable(global, config)?);
        }
        if !globals.is_empty() {
            lines.push(String::new());
        }
        
        // Generate classes
        for class in &classes {
            lines.push(self.generate_class(class, config)?);
            lines.push(String::new());
        }
        
        // Generate functions
        for function in &functions {
            lines.push(self.generate_function(function, config)?);
            lines.push(String::new());
        }
        
        // Generate main code
        if !main_code.is_empty() {
            lines.push("if __name__ == \"__main__\":".to_string());
            for block in &main_code {
                let mut code = self.generate_block(block, config)?;
                // Indent main code
                code = code.lines()
                    .map(|line| format!("    {}", line))
                    .collect::<Vec<_>>()
                    .join("\n");
                lines.push(code);
            }
        }
        
        Ok(lines.join("\n"))
    }
    
    fn format(&self, code: String, _config: &GenerationConfig) -> Result<String> {
        // Use black or autopep8 for Python formatting
        // For now, return as-is
        Ok(code)
    }
}

impl PythonGenerator {
    fn generate_import(&self, block: &GenerationBlock) -> Result<String> {
        // Try to use raw text if available
        if !block.abstract_syntax.raw_text.is_empty() {
            return Ok(block.abstract_syntax.raw_text.clone());
        }
        
        // Otherwise generate from simplified syntax
        let simplified = &block.abstract_syntax.simplified;
        if let Some(name) = &simplified.name {
            Ok(format!("import {}", name))
        } else {
            Ok("# Unknown import".to_string())
        }
    }
    
    fn generate_function(&self, block: &GenerationBlock, config: &GenerationConfig) -> Result<String> {
        let indent = self.indent(block.indent_level, config);
        
        // Get the function name from semantic_name (database field)
        let func_name = block.semantic_name.as_ref().unwrap_or(&"unknown_function".to_string());
        
        // ✅ ENHANCED: First priority - use preserved implementation from enhanced extraction
        // The enhanced implementation data is stored directly in abstract_syntax (from SemanticBlock.syntax_preservation.normalized_ast)
        if let Some(implementation) = block.abstract_syntax.get("implementation") {
            if let Some(original_body) = implementation.get("original_body") {
                if let Some(body_str) = original_body.as_str() {
                    if !body_str.trim().is_empty() {
                        // Extract parameters from metadata or use empty params
                        let params = if let Some(metadata) = &block.metadata {
                            metadata.get("parameters")
                                .and_then(|p| p.as_array())
                                .map(|arr| arr.iter()
                                    .filter_map(|v| v.as_str())
                                    .collect::<Vec<_>>()
                                    .join(", "))
                                .unwrap_or_else(|| "".to_string())
                        } else {
                            "".to_string()
                        };
                            
                        // Build complete function with preserved implementation
                        let mut result = String::new();
                        
                        // Add decorators if present
                        if let Some(metadata) = &block.metadata {
                            if let Some(decorators) = metadata.get("decorators") {
                                if let Some(decorator_list) = decorators.as_array() {
                                    for decorator in decorator_list {
                                        result.push_str(&format!("{}@{}\n", indent, decorator.as_str().unwrap_or("decorator")));
                                    }
                                }
                            }
                        }
                        
                        // Add async if needed (check metadata for modifiers)
                        let is_async = if let Some(metadata) = &block.metadata {
                            metadata.get("modifiers")
                                .and_then(|m| m.as_array())
                                .map(|arr| arr.iter().any(|v| v.as_str() == Some("async")))
                                .unwrap_or(false)
                        } else {
                            false
                        };
                        
                        if is_async {
                            result.push_str(&format!("{}async def {}({}):\n", indent, func_name, params));
                        } else {
                            result.push_str(&format!("{}def {}({}):\n", indent, func_name, params));
                        }
                        
                        // Add preserved body with proper indentation
                        let formatted_body = body_str.lines()
                            .map(|line| {
                                if line.trim().is_empty() {
                                    String::new()
                                } else {
                                    format!("{}{}", self.indent(block.indent_level + 1, config), line)
                                }
                            })
                            .collect::<Vec<_>>()
                            .join("\n");
                        
                        result.push_str(&formatted_body);
                        return Ok(result);
                    }
                }
            }
        }
        
        // Final fallback - template generation using semantic_name and metadata
        
        // Build function signature
        let mut signature = String::new();
        
        // Add decorators if present
        if let Some(metadata) = &block.metadata {
            if let Some(decorators) = metadata.get("decorators") {
                if let Some(decorator_list) = decorators.as_array() {
                    for decorator in decorator_list {
                        signature.push_str(&format!("{}@{}\n", indent, decorator.as_str().unwrap_or("decorator")));
                    }
                }
            }
        }
        
        // Add async if needed
        let is_async = if let Some(metadata) = &block.metadata {
            metadata.get("modifiers")
                .and_then(|m| m.as_array())
                .map(|arr| arr.iter().any(|v| v.as_str() == Some("async")))
                .unwrap_or(false)
        } else {
            false
        };
        
        if is_async {
            signature.push_str("async ");
        }
        
        signature.push_str("def ");
        signature.push_str(func_name);
        
        // Add parameters
        signature.push('(');
        let params = if let Some(metadata) = &block.metadata {
            metadata.get("parameters")
                .and_then(|p| p.as_array())
                .map(|arr| arr.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join(", "))
                .unwrap_or_else(|| "".to_string())
        } else {
            "".to_string()
        };
        signature.push_str(&params);
        signature.push_str(")");
        
        // Add return type if present
        if let Some(metadata) = &block.metadata {
            if let Some(return_type) = metadata.get("return_type") {
                signature.push_str(&format!(" -> {}", return_type.as_str().unwrap_or("Any")));
            }
        }
        
        signature.push(':');
        
        // Add body - simple fallback
        let body = format!("{}    pass", self.indent(block.indent_level + 1, config));
        
        Ok(format!("{}{}\n{}", indent, signature, body))
    }
    
    fn generate_class(&self, block: &GenerationBlock, config: &GenerationConfig) -> Result<String> {
        // ALWAYS prefer complete original if available
        if !block.abstract_syntax.raw_text.is_empty() {
            return Ok(block.abstract_syntax.raw_text.clone());
        }
        
        let simplified = &block.abstract_syntax.simplified;
        let indent = self.indent(block.indent_level, config);
        
        let mut class_code = String::new();
        
        // Add class declaration
        class_code.push_str(&format!("{}class {}", indent, 
            simplified.name.clone().unwrap_or_else(|| "UnnamedClass".to_string())));
        
        // Add base classes if present
        if let Some(bases) = block.metadata.get("base_classes") {
            if let Some(base_list) = bases.as_array() {
                if !base_list.is_empty() {
                    let bases_str: Vec<String> = base_list.iter()
                        .filter_map(|b| b.as_str().map(|s| s.to_string()))
                        .collect();
                    class_code.push_str(&format!("({})", bases_str.join(", ")));
                }
            }
        }
        
        class_code.push_str(":\n");
        
        // Add class body
        if !block.abstract_syntax.raw_text.is_empty() {
            let body = self.extract_class_body(&block.abstract_syntax.raw_text)?;
            class_code.push_str(&body);
        } else {
            class_code.push_str(&format!("{}    pass", self.indent(block.indent_level + 1, config)));
        }
        
        Ok(class_code)
    }
    
    fn generate_variable(&self, block: &GenerationBlock, config: &GenerationConfig) -> Result<String> {
        let indent = self.indent(block.indent_level, config);
        
        // Get the variable name from semantic_name (database field)
        let var_name = block.semantic_name.as_ref().unwrap_or(&"unnamed_var".to_string());
        
        // ✅ ENHANCED: First priority - use preserved implementation with actual values
        // The enhanced implementation data is stored directly in abstract_syntax (from SemanticBlock.syntax_preservation.normalized_ast)
        if let Some(implementation) = block.abstract_syntax.get("implementation") {
            if let Some(assignments) = implementation.get("variable_assignments") {
                if let Some(assignment_info) = assignments.get(var_name) {
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
                        return Ok(format!("{}{} = {}", indent, var_name, value_str));
                    } else if let Some(expression) = assignment_info.get("expression") {
                        // Use the preserved expression
                        return Ok(format!("{}{} = {}", indent, var_name, expression.as_str().unwrap_or("None")));
                    }
                }
            }
        }
        
        // Second priority - try to use raw text if available
        // Note: raw_text is not a field in the database Block struct, so this is likely not available
        
        // Generate fallback using semantic_name
        let mut var_decl = var_name.clone();
        
        // Add type annotation if present in metadata
        if let Some(metadata) = &block.metadata {
            if let Some(var_type) = metadata.get("type") {
                var_decl = format!("{}: {}", var_name, var_type.as_str().unwrap_or("Any"));
            }
        }
        
        // Add value if present in metadata
        if let Some(metadata) = &block.metadata {
            if let Some(value) = metadata.get("initial_value") {
                var_decl = format!("{} = {}", var_decl, value);
            } else {
                var_decl = format!("{} = None", var_decl);
            }
        } else {
            var_decl = format!("{} = None", var_decl);
        }
        
        Ok(format!("{}{}", indent, var_decl))
    }
    
    fn generate_block(&self, block: &GenerationBlock, config: &GenerationConfig) -> Result<String> {
        match block.block_type.as_str() {
            "Function" => self.generate_function(block, config),
            "Class" => self.generate_class(block, config),
            "Variable" => self.generate_variable(block, config),
            "Import" => self.generate_import(block),
            _ => {
                // Use raw text if available
                if !block.abstract_syntax.raw_text.is_empty() {
                    Ok(format!("{}{}", 
                        self.indent(block.indent_level, config),
                        block.abstract_syntax.raw_text))
                } else {
                    // Generate semantic block based on type and content
                    let semantic_content = if let Some(content) = &block.body_ast {
                        if let Some(body) = content.get("body").and_then(|b| b.as_str()) {
                            body.to_string()
                        } else {
                            format!("# Implementation for {}", block.block_type)
                        }
                    } else {
                        format!("# Implementation for {}", block.block_type)
                    };
                    Ok(format!("{}{}", 
                        self.indent(block.indent_level, config),
                        block.block_type))
                }
            }
        }
    }
    
    fn extract_function_body(&self, raw_text: &str) -> Result<String> {
        // Find the function body after the colon
        if let Some(colon_pos) = raw_text.find(':') {
            Ok(raw_text[colon_pos + 1..].to_string())
        } else {
            Ok("    pass".to_string())
        }
    }
    
    fn extract_class_body(&self, raw_text: &str) -> Result<String> {
        // Find the class body after the colon
        if let Some(colon_pos) = raw_text.find(':') {
            Ok(raw_text[colon_pos + 1..].to_string())
        } else {
            Ok("    pass".to_string())
        }
    }
    
    fn is_standard_library(&self, module: &str) -> bool {
        const STANDARD_LIBS: &[&str] = &[
            "os", "sys", "re", "json", "math", "random", "datetime", "collections",
            "itertools", "functools", "typing", "pathlib", "io", "time", "copy",
            "threading", "multiprocessing", "subprocess", "socket", "http", "urllib",
        ];
        
        STANDARD_LIBS.contains(&module)
    }
}
