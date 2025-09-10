use anyhow::Result;
use ast_extractor::traits::SemanticBlock;

use crate::components::*;

/// Trait for language-specific component mappers
pub trait ComponentMapper: Send + Sync {
    /// Map a semantic block to code components
    fn map_semantic_block(&self, block: &SemanticBlock) -> Result<Vec<CodeComponent>>;
    
    /// Get the language this mapper supports
    fn language(&self) -> &'static str;
}

/// Python-specific component mapper
pub struct PythonMapper;

impl PythonMapper {
    pub fn new() -> Self {
        Self
    }

    fn map_function(&self, block: &SemanticBlock) -> Result<Vec<CodeComponent>> {
        let mut components = Vec::new();

        // Extract function signature
        let params = block.ast_node.attributes.get("parameters")
            .and_then(|p| p.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|p| p.as_str())
                    .map(|name| Parameter::new(name.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let decorators = block.ast_node.attributes.get("decorators")
            .and_then(|d| d.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|d| d.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();

        let return_type = block.ast_node.attributes.get("return_type")
            .and_then(|r| r.as_str())
            .map(|s| s.to_string());

        let is_async = block.ast_node.attributes.get("is_async")
            .and_then(|a| a.as_bool())
            .unwrap_or(false);

        let signature = FunctionSignature {
            name: block.semantic_name.clone(),
            parameters: params,
            return_type,
            is_async,
            decorators,
            type_parameters: vec![],
        };

        components.push(CodeComponent::FunctionSignature(signature));

        // Extract function body from expression AST
        if let Some(expr_ast) = &block.expression_ast {
            let body = FunctionBody {
                statements: vec![], // TODO: Extract from expression AST
                expressions: vec![expr_ast.clone()],
                local_variables: expr_ast.variables.clone(),
                called_functions: expr_ast.function_calls.iter()
                    .map(|fc| fc.name.clone())
                    .collect(),
            };
            components.push(CodeComponent::FunctionBody(body));
        }

        Ok(components)
    }

    fn map_class(&self, block: &SemanticBlock) -> Result<Vec<CodeComponent>> {
        let mut components = Vec::new();

        // Extract class declaration
        let base_classes = block.ast_node.attributes.get("base_classes")
            .and_then(|b| b.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|b| b.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();

        let decorators = block.ast_node.attributes.get("decorators")
            .and_then(|d| d.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|d| d.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();

        let declaration = ClassDeclaration {
            name: block.semantic_name.clone(),
            base_classes,
            decorators,
            type_parameters: vec![],
            is_abstract: false, // TODO: Detect abstract classes
        };

        components.push(CodeComponent::ClassDeclaration(declaration));

        // Extract class body
        let methods = block.ast_node.attributes.get("methods")
            .and_then(|m| m.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|m| m.as_str())
                    .map(|name| FunctionSignature {
                        name: name.to_string(),
                        parameters: vec![],
                        return_type: None,
                        is_async: false,
                        decorators: vec![],
                        type_parameters: vec![],
                    })
                    .collect()
            })
            .unwrap_or_default();

        let attributes = block.ast_node.attributes.get("attributes")
            .and_then(|a| a.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|a| a.as_str())
                    .map(|name| VariableDeclaration {
                        name: name.to_string(),
                        type_annotation: None,
                        initial_value: None,
                        is_constant: false,
                        is_static: false,
                    })
                    .collect()
            })
            .unwrap_or_default();

        let body = ClassBody {
            methods,
            attributes,
            properties: vec![],
            static_methods: vec![],
            class_methods: vec![],
        };

        components.push(CodeComponent::ClassBody(body));

        Ok(components)
    }

    fn map_variable(&self, block: &SemanticBlock) -> Result<Vec<CodeComponent>> {
        let type_annotation = block.ast_node.attributes.get("type_annotation")
            .and_then(|t| t.as_str())
            .map(|t| TypeAnnotation {
                base_type: t.to_string(),
                type_parameters: vec![],
                is_optional: false,
                is_union: false,
                union_types: vec![],
            });

        let variable = VariableDeclaration {
            name: block.semantic_name.clone(),
            type_annotation,
            initial_value: block.expression_ast.clone(),
            is_constant: false,
            is_static: false,
        };

        Ok(vec![CodeComponent::Variable(variable)])
    }
}

impl ComponentMapper for PythonMapper {
    fn map_semantic_block(&self, block: &SemanticBlock) -> Result<Vec<CodeComponent>> {
        match block.block_type.as_str() {
            "Function" | "function_definition" => self.map_function(block),
            "Class" | "class_definition" => self.map_class(block),
            "Variable" | "assignment" => self.map_variable(block),
            _ => {
                // Generic mapping for unknown types
                if let Some(expr_ast) = &block.expression_ast {
                    Ok(vec![CodeComponent::Expression(expr_ast.clone())])
                } else {
                    Ok(vec![])
                }
            }
        }
    }

    fn language(&self) -> &'static str {
        "python"
    }
}

/// Rust-specific component mapper
pub struct RustMapper;

impl RustMapper {
    pub fn new() -> Self {
        Self
    }
}

impl ComponentMapper for RustMapper {
    fn map_semantic_block(&self, block: &SemanticBlock) -> Result<Vec<CodeComponent>> {
        // TODO: Implement Rust-specific mapping
        match block.block_type.as_str() {
            "function_item" => {
                // Map Rust function
                let signature = FunctionSignature {
                    name: block.semantic_name.clone(),
                    parameters: vec![],
                    return_type: None,
                    is_async: false,
                    decorators: vec![],
                    type_parameters: vec![],
                };
                Ok(vec![CodeComponent::FunctionSignature(signature)])
            }
            "struct_item" => {
                // Map Rust struct
                let declaration = ClassDeclaration {
                    name: block.semantic_name.clone(),
                    base_classes: vec![],
                    decorators: vec![],
                    type_parameters: vec![],
                    is_abstract: false,
                };
                Ok(vec![CodeComponent::ClassDeclaration(declaration)])
            }
            _ => Ok(vec![]),
        }
    }

    fn language(&self) -> &'static str {
        "rust"
    }
}

/// TypeScript/JavaScript component mapper
pub struct TypeScriptMapper;

impl TypeScriptMapper {
    pub fn new() -> Self {
        Self
    }
}

impl ComponentMapper for TypeScriptMapper {
    fn map_semantic_block(&self, block: &SemanticBlock) -> Result<Vec<CodeComponent>> {
        // TODO: Implement TypeScript-specific mapping
        match block.block_type.as_str() {
            "function_declaration" | "method_definition" => {
                // Map TypeScript function
                let signature = FunctionSignature {
                    name: block.semantic_name.clone(),
                    parameters: vec![],
                    return_type: None,
                    is_async: false,
                    decorators: vec![],
                    type_parameters: vec![],
                };
                Ok(vec![CodeComponent::FunctionSignature(signature)])
            }
            "class_declaration" => {
                // Map TypeScript class
                let declaration = ClassDeclaration {
                    name: block.semantic_name.clone(),
                    base_classes: vec![],
                    decorators: vec![],
                    type_parameters: vec![],
                    is_abstract: false,
                };
                Ok(vec![CodeComponent::ClassDeclaration(declaration)])
            }
            _ => Ok(vec![]),
        }
    }

    fn language(&self) -> &'static str {
        "typescript"
    }
}
