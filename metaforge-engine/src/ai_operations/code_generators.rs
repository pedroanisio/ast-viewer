use anyhow::Result;
use std::collections::HashMap;
use crate::ai_operations::{AbstractBlockSpec, BlockSynthesisRequest, DesignPattern, PatternLibrary, BlockType, ParameterSpec, TypeSpec};

pub trait LanguageGenerator: Send + Sync {
    fn generate_from_pattern(
        &self,
        pattern: &DesignPattern,
        spec: &AbstractBlockSpec,
    ) -> Result<String>;
    
    fn language_name(&self) -> &str;
    fn file_extension(&self) -> &str;
}

pub struct CodeGenerator {
    generators: HashMap<String, Box<dyn LanguageGenerator>>,
    pattern_library: PatternLibrary,
}

impl CodeGenerator {
    pub fn new() -> Self {
        let mut generators: HashMap<String, Box<dyn LanguageGenerator>> = HashMap::new();
        
        generators.insert("python".to_string(), Box::new(PythonGenerator::new()));
        generators.insert("typescript".to_string(), Box::new(TypeScriptGenerator::new()));
        generators.insert("rust".to_string(), Box::new(RustGenerator::new()));
        
        Self {
            generators,
            pattern_library: PatternLibrary::new(),
        }
    }

    pub fn generate_from_spec(
        &self,
        spec: &AbstractBlockSpec,
        request: &BlockSynthesisRequest,
    ) -> Result<String> {
        // Select appropriate pattern
        let pattern = self.pattern_library.select_pattern(spec)?;
        
        // Default to Python if no language specified
        let language = request.constraints
            .iter()
            .find(|c| c.constraint_type == "target_language")
            .and_then(|c| c.value.as_str())
            .unwrap_or("python");
        
        // Get language-specific generator
        let generator = self.generators
            .get(language)
            .ok_or_else(|| anyhow::anyhow!("Unsupported language: {}", language))?;
        
        // Generate code
        generator.generate_from_pattern(&pattern, spec)
    }

    pub fn add_generator(&mut self, language: String, generator: Box<dyn LanguageGenerator>) {
        self.generators.insert(language, generator);
    }

    pub fn supported_languages(&self) -> Vec<&str> {
        self.generators.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

// Python Code Generator
pub struct PythonGenerator {
    indent: String,
}

impl PythonGenerator {
    pub fn new() -> Self {
        Self {
            indent: "    ".to_string(),
        }
    }

    fn format_parameters(&self, parameters: &[ParameterSpec]) -> String {
        if parameters.is_empty() {
            return String::new();
        }

        let formatted: Vec<String> = parameters
            .iter()
            .map(|param| {
                let mut result = param.name.clone();
                
                // Add type annotation
                if !param.param_type.name.is_empty() {
                    result.push_str(": ");
                    result.push_str(&self.format_type(&param.param_type));
                }
                
                // Add default value
                if let Some(default) = &param.default_value {
                    result.push_str(" = ");
                    result.push_str(default);
                }
                
                result
            })
            .collect();

        formatted.join(", ")
    }

    fn format_type(&self, type_spec: &TypeSpec) -> String {
        let mut result = type_spec.name.clone();
        
        if !type_spec.generics.is_empty() {
            result.push('[');
            let generics: Vec<String> = type_spec.generics
                .iter()
                .map(|t| self.format_type(t))
                .collect();
            result.push_str(&generics.join(", "));
            result.push(']');
        }
        
        if type_spec.nullable {
            result = format!("Optional[{}]", result);
        }
        
        result
    }

    fn generate_function_body(&self, spec: &AbstractBlockSpec) -> String {
        if spec.behaviors.is_empty() {
            return format!("{}pass", self.indent);
        }

        let mut body = Vec::new();
        
        for behavior in &spec.behaviors {
            body.push(format!("{}# {}", self.indent, behavior.description));
            
            // Generate basic implementation based on behavior
            match behavior.name.as_str() {
                "validate" => body.push(format!("{}if not data:", self.indent)),
                "process" => body.push(format!("{}result = self._process_data(data)", self.indent)),
                "save" => body.push(format!("{}self._save_to_storage(data)", self.indent)),
                _ => body.push(format!("{}# TODO: Implement {}", self.indent, behavior.name)),
            }
        }
        
        if body.is_empty() {
            body.push(format!("{}pass", self.indent));
        }
        
        body.join("\n")
    }

    fn generate_class_methods(&self, spec: &AbstractBlockSpec) -> String {
        if spec.behaviors.is_empty() {
            return format!("{}pass", self.indent);
        }

        let mut methods = Vec::new();
        
        for behavior in &spec.behaviors {
            let method_name = behavior.name.replace(" ", "_").to_lowercase();
            let mut method = format!("{}def {}(self):", self.indent, method_name);
            method.push_str(&format!("\n{}{}\"\"\"{}\"\"\"", self.indent, self.indent, behavior.description));
            method.push_str(&format!("\n{}{}# TODO: Implement {}", self.indent, self.indent, behavior.name));
            method.push_str(&format!("\n{}{}pass", self.indent, self.indent));
            methods.push(method);
        }
        
        methods.join("\n\n")
    }
}

impl LanguageGenerator for PythonGenerator {
    fn generate_from_pattern(
        &self,
        pattern: &DesignPattern,
        spec: &AbstractBlockSpec,
    ) -> Result<String> {
        let variant = pattern.language_variants
            .get("python")
            .ok_or_else(|| anyhow::anyhow!("Python variant not found for pattern: {}", pattern.name))?;

        let mut template = variant.template.clone();
        let mut imports = variant.imports.clone();

        // Replace placeholders based on block type
        match spec.block_type {
            BlockType::Function => {
                template = template.replace("{{function_name}}", &spec.semantic_name);
                template = template.replace("{{description}}", &spec.description);
                template = template.replace("{{parameters}}", &self.format_parameters(&spec.properties.parameters));
                template = template.replace("{{body}}", &self.generate_function_body(spec));
                template = template.replace("{{return_value}}", "None");
                
                if let Some(return_type) = &spec.properties.return_type {
                    template = template.replace("{{return_type}}", &self.format_type(return_type));
                }
            }
            BlockType::Class => {
                template = template.replace("{{class_name}}", &spec.semantic_name);
                template = template.replace("{{description}}", &spec.description);
                
                let init_params = if spec.properties.parameters.is_empty() {
                    String::new()
                } else {
                    format!(", {}", self.format_parameters(&spec.properties.parameters))
                };
                template = template.replace("{{init_parameters}}", &init_params);
                
                template = template.replace("{{init_body}}", &format!("{}pass", self.indent));
                template = template.replace("{{methods}}", &self.generate_class_methods(spec));
                template = template.replace("{{fields}}", "");
            }
            _ => {
                // Default handling
                template = template.replace("{{description}}", &spec.description);
                template = template.replace("{{code}}", "pass");
            }
        }

        // Add type annotations if needed
        if !spec.properties.parameters.is_empty() || spec.properties.return_type.is_some() {
            if !imports.iter().any(|i| i.contains("typing")) {
                imports.insert(0, "from typing import Optional, List, Dict, Any".to_string());
            }
        }

        // Combine imports and code
        let mut result = String::new();
        if !imports.is_empty() {
            result.push_str(&imports.join("\n"));
            result.push_str("\n\n");
        }
        result.push_str(&template);

        Ok(result)
    }

    fn language_name(&self) -> &str {
        "python"
    }

    fn file_extension(&self) -> &str {
        "py"
    }
}

// TypeScript Code Generator
pub struct TypeScriptGenerator;

impl TypeScriptGenerator {
    pub fn new() -> Self {
        Self
    }

    fn format_parameters(&self, parameters: &[ParameterSpec]) -> String {
        if parameters.is_empty() {
            return String::new();
        }

        let formatted: Vec<String> = parameters
            .iter()
            .map(|param| {
                let mut result = param.name.clone();
                
                if !param.param_type.name.is_empty() {
                    result.push_str(": ");
                    result.push_str(&self.format_type(&param.param_type));
                }
                
                if let Some(default) = &param.default_value {
                    result.push_str(" = ");
                    result.push_str(default);
                }
                
                result
            })
            .collect();

        formatted.join(", ")
    }

    fn format_type(&self, type_spec: &TypeSpec) -> String {
        let mut result = match type_spec.name.as_str() {
            "str" => "string".to_string(),
            "int" => "number".to_string(),
            "float" => "number".to_string(),
            "bool" => "boolean".to_string(),
            "list" => "Array".to_string(),
            "dict" => "Record<string, any>".to_string(),
            _ => type_spec.name.clone(),
        };
        
        if !type_spec.generics.is_empty() {
            result.push('<');
            let generics: Vec<String> = type_spec.generics
                .iter()
                .map(|t| self.format_type(t))
                .collect();
            result.push_str(&generics.join(", "));
            result.push('>');
        }
        
        if type_spec.nullable {
            result = format!("{} | null", result);
        }
        
        result
    }
}

impl LanguageGenerator for TypeScriptGenerator {
    fn generate_from_pattern(
        &self,
        pattern: &DesignPattern,
        spec: &AbstractBlockSpec,
    ) -> Result<String> {
        let variant = pattern.language_variants
            .get("typescript")
            .ok_or_else(|| anyhow::anyhow!("TypeScript variant not found for pattern: {}", pattern.name))?;

        let mut template = variant.template.clone();

        // Replace placeholders
        match spec.block_type {
            BlockType::Function => {
                template = template.replace("{{function_name}}", &spec.semantic_name);
                template = template.replace("{{description}}", &spec.description);
                template = template.replace("{{parameters}}", &self.format_parameters(&spec.properties.parameters));
                template = template.replace("{{body}}", "    // TODO: Implement function body");
                template = template.replace("{{return_value}}", "null");
                
                if let Some(return_type) = &spec.properties.return_type {
                    template = template.replace("{{return_type}}", &self.format_type(return_type));
                } else {
                    template = template.replace("{{return_type}}", "void");
                }
            }
            BlockType::Class => {
                template = template.replace("{{class_name}}", &spec.semantic_name);
                template = template.replace("{{description}}", &spec.description);
                template = template.replace("{{init_parameters}}", &self.format_parameters(&spec.properties.parameters));
                template = template.replace("{{init_body}}", "        // TODO: Initialize class");
                template = template.replace("{{methods}}", "    // TODO: Add methods");
            }
            _ => {
                template = template.replace("{{description}}", &spec.description);
            }
        }

        Ok(template)
    }

    fn language_name(&self) -> &str {
        "typescript"
    }

    fn file_extension(&self) -> &str {
        "ts"
    }
}

// Rust Code Generator
pub struct RustGenerator;

impl RustGenerator {
    pub fn new() -> Self {
        Self
    }

    fn format_parameters(&self, parameters: &[ParameterSpec]) -> String {
        if parameters.is_empty() {
            return String::new();
        }

        let formatted: Vec<String> = parameters
            .iter()
            .map(|param| {
                let mut result = param.name.clone();
                result.push_str(": ");
                result.push_str(&self.format_type(&param.param_type));
                result
            })
            .collect();

        formatted.join(", ")
    }

    fn format_type(&self, type_spec: &TypeSpec) -> String {
        let mut result = match type_spec.name.as_str() {
            "str" => "String".to_string(),
            "int" => "i32".to_string(),
            "float" => "f64".to_string(),
            "bool" => "bool".to_string(),
            "list" => "Vec".to_string(),
            "dict" => "HashMap<String, String>".to_string(),
            _ => type_spec.name.clone(),
        };
        
        if !type_spec.generics.is_empty() {
            result.push('<');
            let generics: Vec<String> = type_spec.generics
                .iter()
                .map(|t| self.format_type(t))
                .collect();
            result.push_str(&generics.join(", "));
            result.push('>');
        }
        
        if type_spec.nullable {
            result = format!("Option<{}>", result);
        }
        
        result
    }
}

impl LanguageGenerator for RustGenerator {
    fn generate_from_pattern(
        &self,
        pattern: &DesignPattern,
        spec: &AbstractBlockSpec,
    ) -> Result<String> {
        let variant = pattern.language_variants
            .get("rust")
            .ok_or_else(|| anyhow::anyhow!("Rust variant not found for pattern: {}", pattern.name))?;

        let mut template = variant.template.clone();

        // Replace placeholders
        match spec.block_type {
            BlockType::Function => {
                template = template.replace("{{function_name}}", &spec.semantic_name);
                template = template.replace("{{description}}", &spec.description);
                template = template.replace("{{parameters}}", &self.format_parameters(&spec.properties.parameters));
                template = template.replace("{{body}}", "    // TODO: Implement function body");
                template = template.replace("{{return_value}}", "()");
                
                if let Some(return_type) = &spec.properties.return_type {
                    template = template.replace("{{return_type}}", &self.format_type(return_type));
                } else {
                    template = template.replace("{{return_type}}", "()");
                }
            }
            BlockType::Class => {
                template = template.replace("{{class_name}}", &spec.semantic_name);
                template = template.replace("{{description}}", &spec.description);
                template = template.replace("{{init_parameters}}", &self.format_parameters(&spec.properties.parameters));
                template = template.replace("{{init_body}}", "        Self { /* TODO: Initialize fields */ }");
                template = template.replace("{{methods}}", "    // TODO: Add methods");
                template = template.replace("{{fields}}", "    // TODO: Add fields");
            }
            _ => {
                template = template.replace("{{description}}", &spec.description);
            }
        }

        Ok(template)
    }

    fn language_name(&self) -> &str {
        "rust"
    }

    fn file_extension(&self) -> &str {
        "rs"
    }
}
