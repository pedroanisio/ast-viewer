use anyhow::{Result, anyhow};
use tree_sitter::Node;
use crate::core::*;
use crate::parser::extraction_context::{ExtractionContext, ParseResult, RelationshipType, LanguageExtractor};

pub struct PythonExtractor;

impl LanguageExtractor for PythonExtractor {
    fn extract_with_context(&self, root: Node, source: &str, _file_path: &str) -> Result<ParseResult> {
        let mut context = ExtractionContext::new();
        self.visit_with_context(root, source, &mut context)?;
        Ok(context.finish())
    }
}

impl PythonExtractor {
    #[allow(dead_code)]
    pub fn extract_blocks(&self, root: Node, source: &str, file_path: &str) -> Result<Vec<SemanticBlock>> {
        let result = self.extract_with_context(root, source, file_path)?;
        Ok(result.blocks)
    }
    
    fn visit_with_context(&self, node: Node, source: &str, ctx: &mut ExtractionContext) -> Result<()> {
        match node.kind() {
            "function_definition" => {
                let block = self.extract_function_block(node, source)?;
                let block_id = ctx.enter_block(block);
                
                // Extract function body and find calls
                self.extract_function_calls(node, source, block_id, ctx)?;
                
                // Visit children for nested functions
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "block" {
                        self.visit_with_context(child, source, ctx)?;
                    }
                }
                
                ctx.exit_block(block_id);
            },
            "class_definition" => {
                let block = self.extract_class_block(node, source)?;
                let block_id = ctx.enter_block(block);
                
                // Extract inheritance relationships
                if let Some(bases) = self.extract_base_classes(node, source)? {
                    for base in bases {
                        ctx.add_relationship(block_id, &base, RelationshipType::Inherits);
                    }
                }
                
                // Visit class body
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "block" {
                        self.visit_with_context(child, source, ctx)?;
                    }
                }
                
                ctx.exit_block(block_id);
            },
            "import_statement" | "import_from_statement" => {
                let block = self.extract_import_block(node, source)?;
                ctx.enter_block(block);
            },
            "assignment" => {
                if let Some(block) = self.extract_variable_block(node, source)? {
                    ctx.enter_block(block);
                }
            },
            _ => {
                // Visit children
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.visit_with_context(child, source, ctx)?;
                }
            }
        }
        Ok(())
    }

    fn extract_function_block(&self, node: Node, source: &str) -> Result<SemanticBlock> {
        let name = self.extract_function_name(node, source)?;
        let text = node.utf8_text(source.as_bytes())?;
        let parameters = self.extract_function_parameters(node, source)?;
        let return_type = self.extract_return_type(node, source)?;
        let decorators = self.extract_decorators(node, source)?;
        
        let mut block = SemanticBlock::new(
            BlockType::Function,
            name.clone(),
            text.to_string(),
            "python".to_string(),
        );

        // Set parameters
        block.semantic_metadata.parameters = parameters;
        
        // Set return type
        if let Some(ret_type) = return_type {
            block.semantic_metadata.return_type = Some(TypeInfo {
                representation: ret_type,
                is_generic: false,
                generic_args: vec![],
            });
        }
        
        // Set decorators
        block.structural_context.decorators = decorators;

        // Set position
        let start = node.start_position();
        let end = node.end_position();
        block.position = BlockPosition {
            start_line: start.row,
            end_line: end.row,
            start_column: start.column,
            end_column: end.column,
            index: 0, // Will be set by context
        };
        
        // Extract modifiers
        if self.is_async_function(node, source)? {
            block.semantic_metadata.modifiers.push(Modifier::Async);
        }
        
        // ✅ ENHANCED: Preserve implementation details in normalized_ast
        let implementation_details = self.extract_implementation_details(node, source)?;
        block.syntax_preservation.normalized_ast = serde_json::json!({
            "implementation": implementation_details
        });
        
        Ok(block)
    }
    
    fn extract_class_block(&self, node: Node, source: &str) -> Result<SemanticBlock> {
        let name = self.extract_class_name(node, source)?;
        let text = node.utf8_text(source.as_bytes())?;
        let decorators = self.extract_decorators(node, source)?;
        
        let mut block = SemanticBlock::new(
            BlockType::Class,
            name.clone(),
            text.to_string(),
            "python".to_string(),
        );

        // Set decorators
        block.structural_context.decorators = decorators;

        // Set position
        let start = node.start_position();
        let end = node.end_position();
        block.position = BlockPosition {
            start_line: start.row,
            end_line: end.row,
            start_column: start.column,
            end_column: end.column,
            index: 0, // Will be set by context
        };
        
        Ok(block)
    }
    
    fn extract_import_block(&self, node: Node, source: &str) -> Result<SemanticBlock> {
        let text = node.utf8_text(source.as_bytes())?;
        let import_name = self.extract_import_name(node, source)?;
        
        let mut block = SemanticBlock::new(
            BlockType::Import,
            import_name,
            text.to_string(),
            "python".to_string(),
        );

        // Set position
        let start = node.start_position();
        let end = node.end_position();
        block.position = BlockPosition {
            start_line: start.row,
            end_line: end.row,
            start_column: start.column,
            end_column: end.column,
            index: 0, // Will be set by context
        };
        
        Ok(block)
    }
    
    fn extract_variable_block(&self, node: Node, source: &str) -> Result<Option<SemanticBlock>> {
        // Only extract top-level assignments
        let text = node.utf8_text(source.as_bytes())?;
        if let Some(name) = self.extract_assignment_name(node, source)? {
            let mut block = SemanticBlock::new(
                BlockType::Variable,
                name.clone(),
                text.to_string(),
                "python".to_string(),
            );
            
            // Set position
            let start = node.start_position();
            let end = node.end_position();
            block.position = BlockPosition {
                start_line: start.row,
                end_line: end.row,
                start_column: start.column,
                end_column: end.column,
                index: 0, // Will be set by context
            };
            
            // ✅ ENHANCED: Preserve variable assignment details
            let variable_details = self.extract_variable_implementation_details(node, source, &name)?;
            block.syntax_preservation.normalized_ast = serde_json::json!({
                "implementation": variable_details
            });
            
            Ok(Some(block))
        } else {
            Ok(None)
        }
    }
    
    fn extract_function_calls(&self, node: Node, source: &str, caller_id: uuid::Uuid, ctx: &mut ExtractionContext) -> Result<()> {
        // Walk the function body looking for calls
        self.find_calls_recursive(node, source, caller_id, ctx)?;
        Ok(())
    }

    fn find_calls_recursive(&self, node: Node, source: &str, caller_id: uuid::Uuid, ctx: &mut ExtractionContext) -> Result<()> {
        if node.kind() == "call" {
            if let Some(name) = self.extract_call_name(node, source)? {
                ctx.add_relationship(caller_id, &name, RelationshipType::Calls);
            }
        }
        
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.find_calls_recursive(child, source, caller_id, ctx)?;
        }
        
        Ok(())
    }

    // Helper methods for extracting specific information
    
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
    
    fn extract_import_name(&self, node: Node, source: &str) -> Result<String> {
        let text = node.utf8_text(source.as_bytes())?;
        // Extract the main import name
        if text.starts_with("from ") {
            if let Some(module) = text.split_whitespace().nth(1) {
                return Ok(module.to_string());
            }
        } else if text.starts_with("import ") {
            if let Some(module) = text.split_whitespace().nth(1) {
                return Ok(module.split('.').next().unwrap_or(module).to_string());
            }
        }
        Ok("unknown_import".to_string())
    }
    
    fn extract_assignment_name(&self, node: Node, source: &str) -> Result<Option<String>> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                return Ok(Some(child.utf8_text(source.as_bytes())?.to_string()));
            }
        }
        Ok(None)
    }

    fn extract_call_name(&self, node: Node, source: &str) -> Result<Option<String>> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" || child.kind() == "attribute" {
                return Ok(Some(child.utf8_text(source.as_bytes())?.to_string()));
            }
        }
        Ok(None)
    }

    fn extract_function_parameters(&self, node: Node, source: &str) -> Result<Vec<Parameter>> {
        let mut parameters = Vec::new();
        let mut cursor = node.walk();
        
        for child in node.children(&mut cursor) {
            if child.kind() == "parameters" {
                let mut param_cursor = child.walk();
                for param_child in child.children(&mut param_cursor) {
                    if param_child.kind() == "identifier" {
                        let name = param_child.utf8_text(source.as_bytes())?.to_string();
                        parameters.push(Parameter {
                            name,
                            type_hint: None,
                            default_value: None,
                            is_optional: false,
                            position: parameters.len(),
                        });
                    }
                }
                break;
            }
        }
        
        Ok(parameters)
    }
    
    fn extract_return_type(&self, node: Node, source: &str) -> Result<Option<String>> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "type" {
                return Ok(Some(child.utf8_text(source.as_bytes())?.to_string()));
            }
        }
        Ok(None)
    }

    fn extract_decorators(&self, node: Node, source: &str) -> Result<Vec<Decorator>> {
        let mut decorators = Vec::new();
        let mut cursor = node.walk();
        
        for child in node.children(&mut cursor) {
            if child.kind() == "decorator" {
                let text = child.utf8_text(source.as_bytes())?;
                let name = text.trim_start_matches('@').to_string();
                decorators.push(Decorator {
                    name,
                    arguments: vec![],
                    line_number: child.start_position().row,
                });
            }
        }
        
        Ok(decorators)
    }
    
    fn extract_base_classes(&self, node: Node, source: &str) -> Result<Option<Vec<String>>> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "argument_list" {
                let mut bases = Vec::new();
                let mut base_cursor = child.walk();
                for base_child in child.children(&mut base_cursor) {
                    if base_child.kind() == "identifier" {
                        bases.push(base_child.utf8_text(source.as_bytes())?.to_string());
                    }
                }
                if !bases.is_empty() {
                    return Ok(Some(bases));
                }
            }
        }
        Ok(None)
    }
    
    fn is_async_function(&self, node: Node, source: &str) -> Result<bool> {
        let text = node.utf8_text(source.as_bytes())?;
        Ok(text.trim_start().starts_with("async def"))
    }
    
    // ✅ NEW: Extract comprehensive implementation details
    fn extract_implementation_details(&self, node: Node, source: &str) -> Result<serde_json::Value> {
        let original_body = self.extract_function_body(node, source)?;
        let variable_assignments = self.extract_variable_assignments(node, source)?;
        let control_flow = self.extract_control_flow_info(node, source)?;
        let return_statements = self.extract_return_statements(node, source)?;
        let function_calls = self.extract_function_calls_detailed(node, source)?;
        
        Ok(serde_json::json!({
            "original_body": original_body,
            "variable_assignments": variable_assignments,
            "control_flow": control_flow,
            "return_statements": return_statements,
            "function_calls": function_calls
        }))
    }
    
    fn extract_function_body(&self, node: Node, source: &str) -> Result<String> {
        // Find the function body (block after the colon)
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "block" {
                let body_text = child.utf8_text(source.as_bytes())?;
                // Remove the leading indentation to get clean body
                let lines: Vec<&str> = body_text.lines().collect();
                if lines.is_empty() {
                    return Ok(String::new());
                }
                
                // Find common indentation
                let mut min_indent = usize::MAX;
                for line in &lines {
                    if !line.trim().is_empty() {
                        let indent = line.len() - line.trim_start().len();
                        min_indent = min_indent.min(indent);
                    }
                }
                
                if min_indent == usize::MAX {
                    min_indent = 0;
                }
                
                // Remove common indentation
                let cleaned_lines: Vec<String> = lines.iter()
                    .map(|line| {
                        if line.len() >= min_indent {
                            line[min_indent..].to_string()
                        } else {
                            line.to_string()
                        }
                    })
                    .collect();
                
                return Ok(cleaned_lines.join("\n"));
            }
        }
        Ok(String::new())
    }
    
    fn extract_variable_assignments(&self, node: Node, source: &str) -> Result<serde_json::Value> {
        let mut assignments = serde_json::Map::new();
        
        // Walk through the function body looking for assignments
        self.find_assignments_recursive(node, source, &mut assignments)?;
        
        Ok(serde_json::Value::Object(assignments))
    }
    
    fn find_assignments_recursive(&self, node: Node, source: &str, assignments: &mut serde_json::Map<String, serde_json::Value>) -> Result<()> {
        if node.kind() == "assignment" {
            if let Some(assignment_info) = self.extract_assignment_info(node, source)? {
                assignments.insert(assignment_info.0, assignment_info.1);
            }
        }
        
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.find_assignments_recursive(child, source, assignments)?;
        }
        
        Ok(())
    }
    
    fn extract_assignment_info(&self, node: Node, source: &str) -> Result<Option<(String, serde_json::Value)>> {
        let mut cursor = node.walk();
        let mut target_name = None;
        let mut value_expr = None;
        
        for child in node.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    if target_name.is_none() {
                        target_name = Some(child.utf8_text(source.as_bytes())?.to_string());
                    }
                }
                _ => {
                    // This is likely the value expression
                    let value_text = child.utf8_text(source.as_bytes())?;
                    let literal_value = self.extract_literal_value(child, source)?;
                    value_expr = Some(serde_json::json!({
                        "expression": value_text,
                        "literal_value": literal_value,
                        "line_number": child.start_position().row
                    }));
                }
            }
        }
        
        if let (Some(name), Some(value)) = (target_name, value_expr) {
            Ok(Some((name, value)))
        } else {
            Ok(None)
        }
    }
    
    fn extract_literal_value(&self, node: Node, source: &str) -> Result<Option<serde_json::Value>> {
        match node.kind() {
            "string" => {
                let text = node.utf8_text(source.as_bytes())?;
                // Remove quotes
                let unquoted = text.trim_matches('"').trim_matches('\'');
                Ok(Some(serde_json::Value::String(unquoted.to_string())))
            }
            "integer" => {
                let text = node.utf8_text(source.as_bytes())?;
                if let Ok(num) = text.parse::<i64>() {
                    Ok(Some(serde_json::Value::Number(serde_json::Number::from(num))))
                } else {
                    Ok(None)
                }
            }
            "float" => {
                let text = node.utf8_text(source.as_bytes())?;
                if let Ok(num) = text.parse::<f64>() {
                    if let Some(json_num) = serde_json::Number::from_f64(num) {
                        Ok(Some(serde_json::Value::Number(json_num)))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            "true" => Ok(Some(serde_json::Value::Bool(true))),
            "false" => Ok(Some(serde_json::Value::Bool(false))),
            "none" => Ok(Some(serde_json::Value::Null)),
            _ => Ok(None), // Complex expressions
        }
    }
    
    fn extract_control_flow_info(&self, node: Node, source: &str) -> Result<serde_json::Value> {
        let mut control_flow = serde_json::Map::new();
        
        // Find control flow statements
        self.find_control_flow_recursive(node, source, &mut control_flow)?;
        
        Ok(serde_json::Value::Object(control_flow))
    }
    
    fn find_control_flow_recursive(&self, node: Node, source: &str, control_flow: &mut serde_json::Map<String, serde_json::Value>) -> Result<()> {
        match node.kind() {
            "if_statement" => {
                let condition = self.extract_condition(node, source)?;
                let entry = control_flow.entry("if_statements".to_string())
                    .or_insert_with(|| serde_json::Value::Array(vec![]));
                if let serde_json::Value::Array(ref mut arr) = entry {
                    arr.push(serde_json::json!({
                        "condition": condition,
                        "line": node.start_position().row
                    }));
                }
            }
            "for_statement" => {
                let entry = control_flow.entry("for_loops".to_string())
                    .or_insert_with(|| serde_json::Value::Array(vec![]));
                if let serde_json::Value::Array(ref mut arr) = entry {
                    arr.push(serde_json::json!({
                        "line": node.start_position().row
                    }));
                }
            }
            "while_statement" => {
                let condition = self.extract_condition(node, source)?;
                let entry = control_flow.entry("while_loops".to_string())
                    .or_insert_with(|| serde_json::Value::Array(vec![]));
                if let serde_json::Value::Array(ref mut arr) = entry {
                    arr.push(serde_json::json!({
                        "condition": condition,
                        "line": node.start_position().row
                    }));
                }
            }
            _ => {}
        }
        
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.find_control_flow_recursive(child, source, control_flow)?;
        }
        
        Ok(())
    }
    
    fn extract_condition(&self, node: Node, source: &str) -> Result<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() != "if" && child.kind() != "while" && child.kind() != ":" {
                return Ok(child.utf8_text(source.as_bytes())?.to_string());
            }
        }
        Ok(String::new())
    }
    
    fn extract_return_statements(&self, node: Node, source: &str) -> Result<serde_json::Value> {
        let mut returns = Vec::new();
        
        self.find_return_statements_recursive(node, source, &mut returns)?;
        
        Ok(serde_json::Value::Array(returns))
    }
    
    fn find_return_statements_recursive(&self, node: Node, source: &str, returns: &mut Vec<serde_json::Value>) -> Result<()> {
        if node.kind() == "return_statement" {
            let return_text = node.utf8_text(source.as_bytes())?;
            let expression = return_text.strip_prefix("return").unwrap_or("").trim();
            
            returns.push(serde_json::json!({
                "expression": expression,
                "line": node.start_position().row
            }));
        }
        
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.find_return_statements_recursive(child, source, returns)?;
        }
        
        Ok(())
    }
    
    fn extract_function_calls_detailed(&self, node: Node, source: &str) -> Result<serde_json::Value> {
        let mut calls = Vec::new();
        
        self.find_function_calls_detailed_recursive(node, source, &mut calls)?;
        
        Ok(serde_json::Value::Array(calls))
    }
    
    fn find_function_calls_detailed_recursive(&self, node: Node, source: &str, calls: &mut Vec<serde_json::Value>) -> Result<()> {
        if node.kind() == "call" {
            let call_text = node.utf8_text(source.as_bytes())?;
            let function_name = self.extract_call_name(node, source)?.unwrap_or_default();
            
            calls.push(serde_json::json!({
                "function_name": function_name,
                "full_expression": call_text,
                "line": node.start_position().row
            }));
        }
        
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.find_function_calls_detailed_recursive(child, source, calls)?;
        }
        
        Ok(())
    }
    
    fn extract_variable_implementation_details(&self, node: Node, source: &str, var_name: &str) -> Result<serde_json::Value> {
        let mut assignments = serde_json::Map::new();
        
        // Extract the assignment info for this specific variable
        if let Some(assignment_info) = self.extract_assignment_info(node, source)? {
            assignments.insert(var_name.to_string(), assignment_info.1);
        }
        
        Ok(serde_json::json!({
            "variable_assignments": assignments,
            "original_text": node.utf8_text(source.as_bytes())?,
            "line_number": node.start_position().row
        }))
    }
}