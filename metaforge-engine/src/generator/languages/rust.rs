use super::super::*;

pub struct RustGenerator;

impl RustGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl LanguageGenerator for RustGenerator {
    fn generate(
        &self,
        _container: &Container,
        blocks: &[GenerationBlock],
        config: &GenerationConfig,
    ) -> Result<String> {
        let mut lines = Vec::new();
        
        // Group blocks
        let mut uses = Vec::new();
        let mut mods = Vec::new();
        let mut structs = Vec::new();
        let mut enums = Vec::new();
        let mut traits = Vec::new();
        let mut impls = Vec::new();
        let mut functions = Vec::new();
        let mut statics = Vec::new();
        
        for block in blocks {
            match block.block_type.as_str() {
                "Import" => uses.push(block),
                "Module" => mods.push(block),
                "Struct" | "Class" => structs.push(block),
                "Enum" => enums.push(block),
                "Trait" | "Interface" => traits.push(block),
                "Impl" => impls.push(block),
                "Function" => functions.push(block),
                "Static" | "Const" => statics.push(block),
                _ => {}
            }
        }
        
        // Generate use statements
        for use_stmt in &uses {
            lines.push(self.generate_use(use_stmt)?);
        }
        if !uses.is_empty() {
            lines.push(String::new());
        }
        
        // Generate modules
        for module in &mods {
            lines.push(self.generate_module(module)?);
        }
        
        // Generate structs
        for struct_block in &structs {
            lines.push(self.generate_struct(struct_block)?);
            lines.push(String::new());
        }
        
        // Generate enums
        for enum_block in &enums {
            lines.push(self.generate_enum(enum_block)?);
            lines.push(String::new());
        }
        
        // Generate traits
        for trait_block in &traits {
            lines.push(self.generate_trait(trait_block)?);
            lines.push(String::new());
        }
        
        // Generate implementations
        for impl_block in &impls {
            lines.push(self.generate_impl(impl_block)?);
            lines.push(String::new());
        }
        
        // Generate functions
        for function in &functions {
            lines.push(self.generate_function(function, config)?);
            lines.push(String::new());
        }
        
        Ok(lines.join("\n"))
    }
    
    fn format(&self, code: String, _config: &GenerationConfig) -> Result<String> {
        // Would use rustfmt
        Ok(code)
    }
}

impl RustGenerator {
    fn generate_use(&self, block: &GenerationBlock) -> Result<String> {
        if !block.abstract_syntax.raw_text.is_empty() {
            return Ok(block.abstract_syntax.raw_text.clone());
        }
        
        if let Some(path) = block.metadata.get("path") {
            Ok(format!("use {};", path.as_str().unwrap_or("unknown")))
        } else {
            Ok("// Unknown use statement".to_string())
        }
    }
    
    fn generate_function(&self, block: &GenerationBlock, config: &GenerationConfig) -> Result<String> {
        let simplified = &block.abstract_syntax.simplified;
        let mut func = String::new();
        
        // Add visibility
        if let Some(vis) = block.metadata.get("visibility") {
            func.push_str(vis.as_str().unwrap_or(""));
            func.push(' ');
        }
        
        // Add async if needed
        if simplified.modifiers.contains(&"async".to_string()) {
            func.push_str("async ");
        }
        
        func.push_str("fn ");
        func.push_str(&simplified.name.clone().unwrap_or_else(|| "unnamed".to_string()));
        
        // Add generics
        if let Some(generics) = block.metadata.get("generics") {
            func.push_str(generics.as_str().unwrap_or(""));
        }
        
        // Add parameters
        func.push('(');
        func.push_str(&simplified.params.join(", "));
        func.push(')');
        
        // Add return type
        if let Some(return_type) = block.metadata.get("return_type") {
            let ret = return_type.as_str().unwrap_or("()");
            if ret != "()" && ret != "" {
                func.push_str(" -> ");
                func.push_str(ret);
            }
        }
        
        func.push_str(" {\n");
        
        // Add body
        if !block.abstract_syntax.raw_text.is_empty() {
            let body = self.extract_function_body(&block.abstract_syntax.raw_text)?;
            func.push_str(&body);
        } else {
            func.push_str("    todo!()\n");
        }
        
        func.push('}');
        
        Ok(func)
    }
    
    fn generate_struct(&self, block: &GenerationBlock) -> Result<String> {
        if !block.abstract_syntax.raw_text.is_empty() {
            return Ok(block.abstract_syntax.raw_text.clone());
        }
        
        let simplified = &block.abstract_syntax.simplified;
        let mut struct_def = String::new();
        
        // Add derives
        if let Some(derives) = block.metadata.get("derives") {
            if let Some(derive_list) = derives.as_array() {
                if !derive_list.is_empty() {
                    let derives_str: Vec<String> = derive_list.iter()
                        .filter_map(|d| d.as_str().map(|s| s.to_string()))
                        .collect();
                    struct_def.push_str(&format!("#[derive({})]\n", derives_str.join(", ")));
                }
            }
        }
        
        // Add visibility
        if let Some(vis) = block.metadata.get("visibility") {
            struct_def.push_str(vis.as_str().unwrap_or(""));
            struct_def.push(' ');
        }
        
        struct_def.push_str("struct ");
        struct_def.push_str(&simplified.name.clone().unwrap_or_else(|| "Struct".to_string()));
        
        // Add generics
        if let Some(generics) = block.metadata.get("generics") {
            struct_def.push_str(generics.as_str().unwrap_or(""));
        }
        
        // Add fields
        if let Some(fields) = block.metadata.get("fields") {
            struct_def.push_str(" {\n");
            if let Some(field_list) = fields.as_array() {
                for field in field_list {
                    if let Some(field_obj) = field.as_object() {
                        let name = field_obj.get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("field");
                        let field_type = field_obj.get("type")
                            .and_then(|t| t.as_str())
                            .unwrap_or("Unknown");
                        let vis = field_obj.get("visibility")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        
                        struct_def.push_str(&format!("    {}{}: {},\n", vis, name, field_type));
                    }
                }
            }
            struct_def.push('}');
        } else {
            struct_def.push(';');
        }
        
        Ok(struct_def)
    }
    
    fn generate_enum(&self, block: &GenerationBlock) -> Result<String> {
        // Similar structure to generate_struct
        let mut enum_def = String::new();
        let visibility = if block.public { "pub " } else { "" };
        enum_def.push_str(&format!("{}enum {} {{\n", visibility, block.name));
        
        // Add enum variants from semantic data
        if let Some(variants) = &block.body_ast {
            if let Some(variant_list) = variants.get("variants") {
                if let Some(variants_array) = variant_list.as_array() {
                    for variant in variants_array {
                        if let Some(variant_name) = variant.as_str() {
                            enum_def.push_str(&format!("    {},\n", variant_name));
                        }
                    }
                }
            }
        }
        
        enum_def.push_str("}\n");
        Ok(enum_def)
    }
    
    fn generate_trait(&self, block: &GenerationBlock) -> Result<String> {
        // Generate trait definition
        let mut trait_def = String::new();
        let visibility = if block.public { "pub " } else { "" };
        trait_def.push_str(&format!("{}trait {} {{\n", visibility, block.name));
        
        // Add trait methods from semantic data
        if let Some(methods) = &block.body_ast {
            if let Some(method_list) = methods.get("methods") {
                if let Some(methods_array) = method_list.as_array() {
                    for method in methods_array {
                        if let Some(method_obj) = method.as_object() {
                            if let Some(signature) = method_obj.get("signature").and_then(|s| s.as_str()) {
                                trait_def.push_str(&format!("    {};\n", signature));
                            }
                        }
                    }
                }
            }
        }
        
        trait_def.push_str("}\n");
        Ok(trait_def)
    }
    
    fn generate_impl(&self, block: &GenerationBlock) -> Result<String> {
        // Generate impl block
        let mut impl_def = String::new();
        
        // Extract impl target and trait
        let target = block.name.clone();
        if let Some(impl_info) = &block.body_ast {
            if let Some(trait_name) = impl_info.get("trait").and_then(|t| t.as_str()) {
                impl_def.push_str(&format!("impl {} for {} {{\n", trait_name, target));
            } else {
                impl_def.push_str(&format!("impl {} {{\n", target));
            }
            
            // Add impl methods
            if let Some(methods) = impl_info.get("methods") {
                if let Some(methods_array) = methods.as_array() {
                    for method in methods_array {
                        if let Some(method_str) = method.as_str() {
                            impl_def.push_str(&format!("    {}\n", method_str));
                        }
                    }
                }
            }
        }
        
        impl_def.push_str("}\n");
        Ok(impl_def)
    }
    
    fn generate_module(&self, block: &GenerationBlock) -> Result<String> {
        let simplified = &block.abstract_syntax.simplified;
        let name = simplified.name.clone().unwrap_or_else(|| "module".to_string());
        
        Ok(format!("mod {};", name))
    }
    
    fn extract_function_body(&self, raw_text: &str) -> Result<String> {
        if let Some(start) = raw_text.find('{') {
            if let Some(end) = raw_text.rfind('}') {
                return Ok(raw_text[start + 1..end].to_string());
            }
        }
        Ok("    todo!()".to_string())
    }
}
