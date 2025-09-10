use super::super::*;

pub struct TypeScriptGenerator {
    is_typescript: bool,
}

impl TypeScriptGenerator {
    pub fn new() -> Self {
        Self { is_typescript: true }
    }
}

pub struct JavaScriptGenerator;

impl JavaScriptGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl LanguageGenerator for TypeScriptGenerator {
    fn generate(
        &self,
        container: &Container,
        blocks: &[GenerationBlock],
        config: &GenerationConfig,
    ) -> Result<String> {
        let mut lines = Vec::new();
        
        // Check if this is a React component
        let is_react = blocks.iter().any(|b| 
            b.metadata.get("is_react_component").and_then(|v| v.as_bool()).unwrap_or(false)
        );
        
        // Group blocks
        let mut imports = Vec::new();
        let mut interfaces = Vec::new();
        let mut types = Vec::new();
        let mut functions = Vec::new();
        let mut classes = Vec::new();
        let mut components = Vec::new();
        let mut exports = Vec::new();
        let mut other = Vec::new();
        
        for block in blocks {
            match block.block_type.as_str() {
                "Import" => imports.push(block),
                "Interface" => interfaces.push(block),
                "TypeDef" => types.push(block),
                "Function" if is_react && self.is_component(block) => components.push(block),
                "Function" => functions.push(block),
                "Class" => classes.push(block),
                "Export" => exports.push(block),
                _ => other.push(block),
            }
        }
        
        // Generate imports
        for import in &imports {
            lines.push(self.generate_import(import)?);
        }
        if !imports.is_empty() {
            lines.push(String::new());
        }
        
        // Generate type definitions
        for typedef in &types {
            lines.push(self.generate_type_def(typedef)?);
        }
        if !types.is_empty() {
            lines.push(String::new());
        }
        
        // Generate interfaces
        for interface in &interfaces {
            lines.push(self.generate_interface(interface)?);
        }
        if !interfaces.is_empty() {
            lines.push(String::new());
        }
        
        // Generate functions/components
        for function in &functions {
            lines.push(self.generate_function(function, config)?);
            lines.push(String::new());
        }
        
        for component in &components {
            lines.push(self.generate_component(component, config)?);
            lines.push(String::new());
        }
        
        // Generate classes
        for class in &classes {
            lines.push(self.generate_class(class, config)?);
            lines.push(String::new());
        }
        
        // Generate exports
        for export in &exports {
            lines.push(self.generate_export(export)?);
        }
        
        Ok(lines.join("\n"))
    }
    
    fn format(&self, code: String, _config: &GenerationConfig) -> Result<String> {
        // Would use prettier or similar
        Ok(code)
    }
}

impl TypeScriptGenerator {
    fn generate_import(&self, block: &GenerationBlock) -> Result<String> {
        if !block.abstract_syntax.raw_text.is_empty() {
            return Ok(block.abstract_syntax.raw_text.clone());
        }
        
        // Generate from metadata
        if let Some(source) = block.metadata.get("source") {
            let source_str = source.as_str().unwrap_or("unknown");
            
            if let Some(specifiers) = block.metadata.get("specifiers") {
                // Named imports
                let specs: Vec<String> = specifiers.as_array()
                    .map(|arr| arr.iter()
                        .filter_map(|s| s.as_str().map(|st| st.to_string()))
                        .collect())
                    .unwrap_or_default();
                
                if !specs.is_empty() {
                    return Ok(format!("import {{ {} }} from '{}';", specs.join(", "), source_str));
                }
            }
            
            // Default import
            if let Some(name) = &block.abstract_syntax.simplified.name {
                return Ok(format!("import {} from '{}';", name, source_str));
            }
        }
        
        Ok("// Unknown import".to_string())
    }
    
    fn generate_function(&self, block: &GenerationBlock, config: &GenerationConfig) -> Result<String> {
        // ALWAYS prefer complete original if available
        if !block.abstract_syntax.raw_text.is_empty() {
            return Ok(block.abstract_syntax.raw_text.clone());
        }
        
        let simplified = &block.abstract_syntax.simplified;
        let mut func = String::new();
        
        // Add export if needed
        if block.metadata.get("is_exported").and_then(|v| v.as_bool()).unwrap_or(false) {
            func.push_str("export ");
        }
        
        // Add async if needed
        if simplified.modifiers.contains(&"async".to_string()) {
            func.push_str("async ");
        }
        
        func.push_str("function ");
        func.push_str(&simplified.name.clone().unwrap_or_else(|| "unnamed".to_string()));
        
        // Add generic parameters if present
        if let Some(generics) = block.metadata.get("generics") {
            func.push_str(&format!("<{}>", generics.as_str().unwrap_or("T")));
        }
        
        // Add parameters
        func.push('(');
        if self.is_typescript {
            // Add typed parameters
            func.push_str(&self.generate_typed_params(&simplified.params, block)?);
        } else {
            func.push_str(&simplified.params.join(", "));
        }
        func.push(')');
        
        // Add return type for TypeScript
        if self.is_typescript {
            if let Some(return_type) = block.metadata.get("return_type") {
                func.push_str(&format!(": {}", return_type.as_str().unwrap_or("any")));
            }
        }
        
        func.push_str(" {\n");
        
        // Add body
        if !block.abstract_syntax.raw_text.is_empty() {
            let body = self.extract_function_body(&block.abstract_syntax.raw_text)?;
            func.push_str(&body);
        } else {
            if let Some(body_ast) = &block.body_ast {
                if let Some(body) = body_ast.get("body").and_then(|b| b.as_str()) {
                    func.push_str(&format!("  {}\n", body));
                } else {
                    func.push_str("  // Implementation\n");
                }
            } else {
                func.push_str("  // Implementation\n");
            }
        }
        
        func.push('}');
        
        Ok(func)
    }
    
    fn generate_component(&self, block: &GenerationBlock, config: &GenerationConfig) -> Result<String> {
        let simplified = &block.abstract_syntax.simplified;
        let name = simplified.name.clone().unwrap_or_else(|| "Component".to_string());
        
        let mut component = String::new();
        
        // Check if it's a functional component
        let is_functional = !block.metadata.get("is_class_component")
            .and_then(|v| v.as_bool()).unwrap_or(false);
        
        if is_functional {
            // Export if needed
            if block.metadata.get("is_exported").and_then(|v| v.as_bool()).unwrap_or(false) {
                component.push_str("export ");
            }
            
            component.push_str("const ");
            component.push_str(&name);
            
            // Add props type for TypeScript
            if self.is_typescript {
                if let Some(props_type) = block.metadata.get("props_type") {
                    component.push_str(&format!(": React.FC<{}>", props_type.as_str().unwrap_or("any")));
                } else {
                    component.push_str(": React.FC");
                }
            }
            
            component.push_str(" = (");
            
            // Add props parameter
            if let Some(props) = simplified.params.first() {
                component.push_str(props);
            } else {
                component.push_str("props");
            }
            
            component.push_str(") => {\n");
            
            // Add body
            if !block.abstract_syntax.raw_text.is_empty() {
                let body = self.extract_component_body(&block.abstract_syntax.raw_text)?;
                component.push_str(&body);
            } else {
                if let Some(component_body) = &block.body_ast {
                    if let Some(jsx) = component_body.get("jsx").and_then(|j| j.as_str()) {
                        component.push_str(&format!("  return {};\n", jsx));
                    } else {
                        component.push_str("  return <div>Component</div>;\n");
                    }
                } else {
                    component.push_str("  return <div>Component</div>;\n");
                }
            }
            
            component.push_str("};");
        } else {
            // Class component
            component.push_str(&self.generate_class_component(block, config)?);
        }
        
        Ok(component)
    }
    
    fn generate_interface(&self, block: &GenerationBlock) -> Result<String> {
        if !block.abstract_syntax.raw_text.is_empty() {
            return Ok(block.abstract_syntax.raw_text.clone());
        }
        
        let simplified = &block.abstract_syntax.simplified;
        let name = simplified.name.clone().unwrap_or_else(|| "Interface".to_string());
        
        let mut interface = String::new();
        
        // Export if needed
        if block.metadata.get("is_exported").and_then(|v| v.as_bool()).unwrap_or(false) {
            interface.push_str("export ");
        }
        
        interface.push_str("interface ");
        interface.push_str(&name);
        
        // Add extends if present
        if let Some(extends) = block.metadata.get("extends") {
            if let Some(base_interfaces) = extends.as_array() {
                if !base_interfaces.is_empty() {
                    let bases: Vec<String> = base_interfaces.iter()
                        .filter_map(|b| b.as_str().map(|s| s.to_string()))
                        .collect();
                    interface.push_str(&format!(" extends {}", bases.join(", ")));
                }
            }
        }
        
        interface.push_str(" {\n");
        
        // Add members
        if let Some(members) = block.metadata.get("members") {
            if let Some(member_list) = members.as_array() {
                for member in member_list {
                    if let Some(member_obj) = member.as_object() {
                        let name = member_obj.get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("unknown");
                        let member_type = member_obj.get("type")
                            .and_then(|t| t.as_str())
                            .unwrap_or("any");
                        let optional = member_obj.get("optional")
                            .and_then(|o| o.as_bool())
                            .unwrap_or(false);
                        
                        interface.push_str(&format!("  {}{}: {};\n", 
                            name, 
                            if optional { "?" } else { "" },
                            member_type
                        ));
                    }
                }
            }
        }
        
        interface.push('}');
        
        Ok(interface)
    }
    
    fn generate_type_def(&self, block: &GenerationBlock) -> Result<String> {
        if !block.abstract_syntax.raw_text.is_empty() {
            return Ok(block.abstract_syntax.raw_text.clone());
        }
        
        let simplified = &block.abstract_syntax.simplified;
        let name = simplified.name.clone().unwrap_or_else(|| "Type".to_string());
        
        let mut typedef = String::new();
        
        // Export if needed
        if block.metadata.get("is_exported").and_then(|v| v.as_bool()).unwrap_or(false) {
            typedef.push_str("export ");
        }
        
        typedef.push_str("type ");
        typedef.push_str(&name);
        
        // Add generic parameters if present
        if let Some(generics) = block.metadata.get("generics") {
            typedef.push_str(&format!("<{}>", generics.as_str().unwrap_or("T")));
        }
        
        typedef.push_str(" = ");
        
        // Add type definition
        if let Some(definition) = block.metadata.get("definition") {
            typedef.push_str(definition.as_str().unwrap_or("unknown"));
        } else {
            typedef.push_str("any");
        }
        
        typedef.push(';');
        
        Ok(typedef)
    }
    
    fn generate_class(&self, block: &GenerationBlock, config: &GenerationConfig) -> Result<String> {
        // ALWAYS prefer complete original if available
        if !block.abstract_syntax.raw_text.is_empty() {
            return Ok(block.abstract_syntax.raw_text.clone());
        }
        
        let simplified = &block.abstract_syntax.simplified;
        let name = simplified.name.clone().unwrap_or_else(|| "Class".to_string());
        
        let mut class = String::new();
        
        // Export if needed
        if block.metadata.get("is_exported").and_then(|v| v.as_bool()).unwrap_or(false) {
            class.push_str("export ");
        }
        
        class.push_str("class ");
        class.push_str(&name);
        
        // Add extends if present
        if let Some(extends) = block.metadata.get("extends") {
            class.push_str(&format!(" extends {}", extends.as_str().unwrap_or("Object")));
        }
        
        // Add implements if present
        if let Some(implements) = block.metadata.get("implements") {
            if let Some(interfaces) = implements.as_array() {
                if !interfaces.is_empty() {
                    let ifaces: Vec<String> = interfaces.iter()
                        .filter_map(|i| i.as_str().map(|s| s.to_string()))
                        .collect();
                    class.push_str(&format!(" implements {}", ifaces.join(", ")));
                }
            }
        }
        
        class.push_str(" {\n");
        
        // Add body
        if !block.abstract_syntax.raw_text.is_empty() {
            let body = self.extract_class_body(&block.abstract_syntax.raw_text)?;
            class.push_str(&body);
        } else {
            if let Some(class_body) = &block.body_ast {
                if let Some(methods) = class_body.get("methods").and_then(|m| m.as_array()) {
                    for method in methods {
                        if let Some(method_str) = method.as_str() {
                            class.push_str(&format!("  {}\n", method_str));
                        }
                    }
                } else {
                    class.push_str("  // Class implementation\n");
                }
            } else {
                class.push_str("  // Class implementation\n");
            }
        }
        
        class.push('}');
        
        Ok(class)
    }
    
    fn generate_export(&self, block: &GenerationBlock) -> Result<String> {
        if !block.abstract_syntax.raw_text.is_empty() {
            return Ok(block.abstract_syntax.raw_text.clone());
        }
        
        // Generate export statement
        if let Some(default) = block.metadata.get("is_default") {
            if default.as_bool().unwrap_or(false) {
                if let Some(name) = &block.abstract_syntax.simplified.name {
                    return Ok(format!("export default {};", name));
                }
            }
        }
        
        // Named exports
        if let Some(exports) = block.metadata.get("exports") {
            if let Some(export_list) = exports.as_array() {
                let names: Vec<String> = export_list.iter()
                    .filter_map(|e| e.as_str().map(|s| s.to_string()))
                    .collect();
                if !names.is_empty() {
                    return Ok(format!("export {{ {} }};", names.join(", ")));
                }
            }
        }
        
        Ok("// Unknown export".to_string())
    }
    
    fn is_component(&self, block: &GenerationBlock) -> bool {
        // Check if function name starts with uppercase (React convention)
        if let Some(name) = &block.abstract_syntax.simplified.name {
            if name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                return true;
            }
        }
        
        // Check metadata
        block.metadata.get("is_react_component")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }
    
    fn generate_typed_params(&self, params: &[String], block: &GenerationBlock) -> Result<String> {
        // Try to get typed parameters from metadata
        if let Some(typed_params) = block.metadata.get("typed_parameters") {
            if let Some(param_list) = typed_params.as_array() {
                let typed: Vec<String> = param_list.iter()
                    .filter_map(|p| p.as_str().map(|s| s.to_string()))
                    .collect();
                return Ok(typed.join(", "));
            }
        }
        
        // Fall back to untyped parameters
        Ok(params.join(", "))
    }
    
    fn extract_function_body(&self, raw_text: &str) -> Result<String> {
        // Find body between braces
        if let Some(start) = raw_text.find('{') {
            if let Some(end) = raw_text.rfind('}') {
                return Ok(raw_text[start + 1..end].to_string());
            }
        }
        Ok("  // Function body".to_string())
    }
    
    fn extract_component_body(&self, raw_text: &str) -> Result<String> {
        // Similar to function body extraction
        self.extract_function_body(raw_text)
    }
    
    fn extract_class_body(&self, raw_text: &str) -> Result<String> {
        // Similar to function body extraction
        self.extract_function_body(raw_text)
    }
    
    fn generate_class_component(&self, block: &GenerationBlock, config: &GenerationConfig) -> Result<String> {
        // Generate React class component
        let simplified = &block.abstract_syntax.simplified;
        let name = simplified.name.clone().unwrap_or_else(|| "Component".to_string());
        
        let mut component = String::new();
        
        if block.metadata.get("is_exported").and_then(|v| v.as_bool()).unwrap_or(false) {
            component.push_str("export ");
        }
        
        component.push_str("class ");
        component.push_str(&name);
        component.push_str(" extends React.Component");
        
        // Add props and state types for TypeScript
        if self.is_typescript {
            let props_type = block.metadata.get("props_type")
                .and_then(|t| t.as_str())
                .unwrap_or("any");
            let state_type = block.metadata.get("state_type")
                .and_then(|t| t.as_str())
                .unwrap_or("any");
            component.push_str(&format!("<{}, {}>", props_type, state_type));
        }
        
        component.push_str(" {\n");
        
        // Add body
        if !block.abstract_syntax.raw_text.is_empty() {
            let body = self.extract_class_body(&block.abstract_syntax.raw_text)?;
            component.push_str(&body);
        } else {
            component.push_str("  render() {\n");
            if let Some(component_body) = &block.body_ast {
                if let Some(jsx) = component_body.get("jsx").and_then(|j| j.as_str()) {
                    component.push_str(&format!("    return {};\n", jsx));
                } else {
                    component.push_str("    return <div>Component</div>;\n");
                }
            } else {
                component.push_str("    return <div>Component</div>;\n");
            }
            component.push_str("  }\n");
        }
        
        component.push('}');
        
        Ok(component)
    }
}

impl LanguageGenerator for JavaScriptGenerator {
    fn generate(
        &self,
        container: &Container,
        blocks: &[GenerationBlock],
        config: &GenerationConfig,
    ) -> Result<String> {
        // Use TypeScript generator with is_typescript = false
        let ts_gen = TypeScriptGenerator { is_typescript: false };
        ts_gen.generate(container, blocks, config)
    }
    
    fn format(&self, code: String, config: &GenerationConfig) -> Result<String> {
        // Use TypeScript formatter
        let ts_gen = TypeScriptGenerator { is_typescript: false };
        ts_gen.format(code, config)
    }
}
