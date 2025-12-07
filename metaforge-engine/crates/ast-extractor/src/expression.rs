use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tree_sitter::{Node, TreeCursor};

/// Enhanced expression analysis for complex code patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpressionAST {
    pub expression_type: String,
    pub operator: Option<String>,
    pub operands: Vec<Value>,
    pub literal_value: Option<Value>,
    pub function_calls: Vec<FunctionCall>,
    pub attribute_access: Vec<AttributeAccess>,
    pub variables: Vec<String>,
    pub complexity_score: u32,
    pub source_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: Vec<Value>,
    pub module_path: Option<String>,
    pub is_method: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeAccess {
    pub object: String,
    pub attribute: String,
    pub chain: Vec<String>,
    pub is_assignment: bool,
}

pub struct ExpressionExtractor;

impl ExpressionExtractor {
    pub fn new() -> Self {
        Self
    }

    /// Extract comprehensive AST for any expression
    pub fn extract_expression(&self, node: Node, source: &str) -> Result<ExpressionAST> {
        let source_text = node.utf8_text(source.as_bytes())?.to_string();
        
        let mut ast = ExpressionAST {
            expression_type: node.kind().to_string(),
            operator: None,
            operands: Vec::new(),
            literal_value: None,
            function_calls: Vec::new(),
            attribute_access: Vec::new(),
            variables: Vec::new(),
            complexity_score: 1,
            source_text,
        };

        match node.kind() {
            "binary_operator" | "comparison_operator" => {
                self.extract_binary_operation(node, source, &mut ast)?;
            }
            "call" => {
                self.extract_function_call(node, source, &mut ast)?;
            }
            "attribute" => {
                self.extract_attribute_access(node, source, &mut ast)?;
            }
            "identifier" => {
                self.extract_identifier(node, source, &mut ast)?;
            }
            "string" | "integer" | "float" | "true" | "false" | "none" => {
                self.extract_literal(node, source, &mut ast)?;
            }
            "assignment" => {
                self.extract_assignment(node, source, &mut ast)?;
            }
            _ => {
                // Generic extraction for unknown node types
                self.extract_generic(node, source, &mut ast)?;
            }
        }

        ast.complexity_score = self.calculate_complexity(&ast);
        Ok(ast)
    }

    fn extract_binary_operation(&self, node: Node, source: &str, ast: &mut ExpressionAST) -> Result<()> {
        let mut cursor = node.walk();
        
        // Find operator and operands
        for child in node.children(&mut cursor) {
            match child.kind() {
                "==" | "!=" | "<" | ">" | "<=" | ">=" | "and" | "or" | "+" | "-" | "*" | "/" | "%" | "**" => {
                    ast.operator = Some(child.utf8_text(source.as_bytes())?.to_string());
                }
                _ => {
                    // Extract operands recursively
                    let operand_ast = self.extract_expression(child, source)?;
                    ast.operands.push(json!(operand_ast));
                    
                    // Collect variables and function calls from operands
                    ast.variables.extend(operand_ast.variables);
                    ast.function_calls.extend(operand_ast.function_calls);
                    ast.attribute_access.extend(operand_ast.attribute_access);
                }
            }
        }
        Ok(())
    }

    fn extract_function_call(&self, node: Node, source: &str, ast: &mut ExpressionAST) -> Result<()> {
        let mut cursor = node.walk();
        let mut function_name = String::new();
        let mut arguments = Vec::new();
        let mut module_path = None;
        let mut is_method = false;

        for child in node.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    function_name = child.utf8_text(source.as_bytes())?.to_string();
                }
                "attribute" => {
                    // Handle module.function() or object.method() calls
                    let attr_ast = self.extract_expression(child, source)?;
                    if let Some(attr) = attr_ast.attribute_access.first() {
                        module_path = Some(attr.object.clone());
                        function_name = attr.attribute.clone();
                        is_method = true; // Assume method call for attribute access
                    }
                    ast.attribute_access.extend(attr_ast.attribute_access);
                }
                "argument_list" => {
                    // Extract all arguments
                    let mut arg_cursor = child.walk();
                    for arg_child in child.children(&mut arg_cursor) {
                        if !matches!(arg_child.kind(), "," | "(" | ")") {
                            let arg_ast = self.extract_expression(arg_child, source)?;
                            arguments.push(json!(arg_ast));
                            
                            // Collect nested data
                            ast.variables.extend(arg_ast.variables);
                            ast.function_calls.extend(arg_ast.function_calls);
                            ast.attribute_access.extend(arg_ast.attribute_access);
                        }
                    }
                }
                _ => {}
            }
        }

        ast.function_calls.push(FunctionCall {
            name: function_name,
            arguments,
            module_path,
            is_method,
        });

        Ok(())
    }

    fn extract_attribute_access(&self, node: Node, source: &str, ast: &mut ExpressionAST) -> Result<()> {
        let mut cursor = node.walk();
        let mut object = String::new();
        let mut attribute = String::new();
        let mut chain = Vec::new();

        for child in node.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    if object.is_empty() {
                        object = child.utf8_text(source.as_bytes())?.to_string();
                        chain.push(object.clone());
                    } else {
                        attribute = child.utf8_text(source.as_bytes())?.to_string();
                        chain.push(attribute.clone());
                    }
                }
                "attribute" => {
                    // Nested attribute access (e.g., a.b.c)
                    let nested_ast = self.extract_expression(child, source)?;
                    if let Some(nested_attr) = nested_ast.attribute_access.first() {
                        object = format!("{}.{}", nested_attr.object, nested_attr.attribute);
                        chain.extend(nested_attr.chain.clone());
                    }
                }
                _ => {}
            }
        }

        if attribute.is_empty() && chain.len() >= 2 {
            // Handle case where we have a chain like [obj, attr]
            object = chain[0].clone();
            attribute = chain[1].clone();
        }

        ast.attribute_access.push(AttributeAccess {
            object: object.clone(),
            attribute: attribute.clone(),
            chain,
            is_assignment: false, // Will be set by parent context if needed
        });

        ast.variables.push(object);
        Ok(())
    }

    fn extract_assignment(&self, node: Node, source: &str, ast: &mut ExpressionAST) -> Result<()> {
        let mut cursor = node.walk();
        
        for child in node.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    let var_name = child.utf8_text(source.as_bytes())?.to_string();
                    ast.variables.push(var_name);
                }
                "attribute" => {
                    // Assignment to attribute (e.g., self.x = value)
                    let mut attr_ast = self.extract_expression(child, source)?;
                    // Mark as assignment
                    for attr in &mut attr_ast.attribute_access {
                        attr.is_assignment = true;
                    }
                    ast.attribute_access.extend(attr_ast.attribute_access);
                    ast.variables.extend(attr_ast.variables);
                }
                _ => {
                    // Extract the right-hand side expression
                    let rhs_ast = self.extract_expression(child, source)?;
                    ast.operands.push(json!(rhs_ast));
                    
                    // Merge RHS data
                    ast.variables.extend(rhs_ast.variables);
                    ast.function_calls.extend(rhs_ast.function_calls);
                    ast.attribute_access.extend(rhs_ast.attribute_access);
                }
            }
        }
        
        Ok(())
    }

    fn extract_identifier(&self, node: Node, source: &str, ast: &mut ExpressionAST) -> Result<()> {
        let identifier = node.utf8_text(source.as_bytes())?.to_string();
        ast.variables.push(identifier);
        Ok(())
    }

    fn extract_literal(&self, node: Node, source: &str, ast: &mut ExpressionAST) -> Result<()> {
        let text = node.utf8_text(source.as_bytes())?;
        
        ast.literal_value = Some(match node.kind() {
            "string" => {
                // Remove quotes and handle escape sequences
                let content = text.trim_matches('"').trim_matches('\'');
                json!(content)
            }
            "integer" => {
                json!(text.parse::<i64>().unwrap_or(0))
            }
            "float" => {
                json!(text.parse::<f64>().unwrap_or(0.0))
            }
            "true" => json!(true),
            "false" => json!(false),
            "none" | "null" => json!(null),
            _ => json!(text)
        });

        Ok(())
    }

    fn extract_generic(&self, node: Node, source: &str, ast: &mut ExpressionAST) -> Result<()> {
        // For unknown node types, extract text and try to parse recursively
        let text = node.utf8_text(source.as_bytes())?.to_string();
        
        // Recursively process children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let child_ast = self.extract_expression(child, source)?;
            
            // Merge child data
            ast.variables.extend(child_ast.variables);
            ast.function_calls.extend(child_ast.function_calls);
            ast.attribute_access.extend(child_ast.attribute_access);
            
            if child_ast.literal_value.is_some() {
                ast.literal_value = child_ast.literal_value;
            }
            
            // Merge operands
            ast.operands.push(json!(child_ast));
        }

        // If no children provided useful data, store as raw text
        if ast.literal_value.is_none() && ast.variables.is_empty() && 
           ast.function_calls.is_empty() && ast.attribute_access.is_empty() {
            ast.literal_value = Some(json!(text));
        }

        Ok(())
    }

    fn calculate_complexity(&self, ast: &ExpressionAST) -> u32 {
        let mut score = 1;
        
        // Add complexity for each component
        score += ast.operands.len() as u32;
        score += ast.function_calls.len() as u32 * 2; // Function calls are more complex
        score += ast.attribute_access.len() as u32;
        
        // Nested expressions add exponential complexity
        for operand in &ast.operands {
            if let Ok(nested_ast) = serde_json::from_value::<ExpressionAST>(operand.clone()) {
                score += nested_ast.complexity_score;
            }
        }
        
        // Operator complexity
        if let Some(ref op) = ast.operator {
            score += match op.as_str() {
                "and" | "or" => 2,  // Logical operators add complexity
                "**" => 2,          // Power operator
                _ => 1,
            };
        }
        
        score
    }
}

impl Default for ExpressionExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;

    #[test]
    fn test_extract_simple_comparison() {
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_python::language()).unwrap();
        
        let code = "os.name == 'nt'";
        let tree = parser.parse(code, None).unwrap();
        
        let extractor = ExpressionExtractor::new();
        let ast = extractor.extract_expression(tree.root_node().child(0).unwrap(), code).unwrap();
        
        assert_eq!(ast.expression_type, "comparison_operator");
        assert_eq!(ast.operator, Some("==".to_string()));
        assert_eq!(ast.operands.len(), 2);
        
        // Should detect os.name attribute access
        assert!(!ast.attribute_access.is_empty());
        assert_eq!(ast.attribute_access[0].object, "os");
        assert_eq!(ast.attribute_access[0].attribute, "name");
    }

    #[test]
    fn test_extract_function_call() {
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_python::language()).unwrap();
        
        let code = "len(items)";
        let tree = parser.parse(code, None).unwrap();
        
        let extractor = ExpressionExtractor::new();
        let ast = extractor.extract_expression(tree.root_node().child(0).unwrap(), code).unwrap();
        
        assert_eq!(ast.expression_type, "call");
        assert!(!ast.function_calls.is_empty());
        assert_eq!(ast.function_calls[0].name, "len");
        assert_eq!(ast.function_calls[0].arguments.len(), 1);
        assert!(!ast.function_calls[0].is_method);
    }

    #[test]
    fn test_extract_method_call() {
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_python::language()).unwrap();
        
        let code = "path.exists()";
        let tree = parser.parse(code, None).unwrap();
        
        let extractor = ExpressionExtractor::new();
        let ast = extractor.extract_expression(tree.root_node().child(0).unwrap(), code).unwrap();
        
        assert_eq!(ast.expression_type, "call");
        assert!(!ast.function_calls.is_empty());
        assert_eq!(ast.function_calls[0].name, "exists");
        assert_eq!(ast.function_calls[0].module_path, Some("path".to_string()));
        assert!(ast.function_calls[0].is_method);
    }

    #[test]
    fn test_complex_expression() {
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_python::language()).unwrap();
        
        let code = "platform.system() == 'Windows' and os.path.exists(file_path)";
        let tree = parser.parse(code, None).unwrap();
        
        let extractor = ExpressionExtractor::new();
        let ast = extractor.extract_expression(tree.root_node().child(0).unwrap(), code).unwrap();
        
        assert_eq!(ast.operator, Some("and".to_string()));
        assert!(ast.complexity_score > 5); // Should be complex
        
        // Should contain multiple function calls and attribute accesses
        assert!(ast.function_calls.len() >= 2);
        assert!(ast.attribute_access.len() >= 2);
    }
}
