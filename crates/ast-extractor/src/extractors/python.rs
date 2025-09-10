use anyhow::Result;
use std::collections::HashMap;
use tree_sitter::{Node, TreeCursor};
use uuid::Uuid;

use crate::{
    ASTNode, SourceRange, ASTMetadata, ExpressionExtractor,
    traits::{ASTExtractor, ExtractionContext, ExtractionResult, SemanticBlock, Dependency, Export, DependencyType, ExportType},
};

pub struct PythonASTExtractor {
    expression_extractor: ExpressionExtractor,
}

impl PythonASTExtractor {
    pub fn new() -> Self {
        Self {
            expression_extractor: ExpressionExtractor::new(),
        }
    }

    fn extract_function_definition(&self, node: Node, source: &str, context: &ExtractionContext) -> Result<SemanticBlock> {
        let mut cursor = node.walk();
        let mut function_name = String::new();
        let mut parameters = Vec::new();
        let mut decorators = Vec::new();
        let mut return_type = None;
        let mut docstring = None;
        let mut body_nodes = Vec::new();

        // Extract function components
        for child in node.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    if function_name.is_empty() {
                        function_name = child.utf8_text(source.as_bytes())?.to_string();
                    }
                }
                "parameters" => {
                    parameters = self.extract_parameters(child, source)?;
                }
                "type" => {
                    return_type = Some(child.utf8_text(source.as_bytes())?.to_string());
                }
                "block" => {
                    body_nodes = self.extract_body_statements(child, source)?;
                    
                    // Check for docstring
                    if let Some(first_stmt) = body_nodes.first() {
                        if first_stmt.contains("\"\"\"") || first_stmt.contains("'''") {
                            docstring = Some(first_stmt.clone());
                        }
                    }
                }
                "decorator" => {
                    decorators.push(child.utf8_text(source.as_bytes())?.to_string());
                }
                _ => {}
            }
        }

        // Create AST node
        let source_range = self.node_to_source_range(node, source)?;
        let mut ast_node = ASTNode::new("function_definition".to_string(), source_range);
        ast_node.semantic_name = Some(function_name.clone());

        // Add function-specific attributes
        ast_node.attributes.insert("parameters".to_string(), serde_json::json!(parameters));
        ast_node.attributes.insert("decorators".to_string(), serde_json::json!(decorators));
        if let Some(ret_type) = return_type {
            ast_node.attributes.insert("return_type".to_string(), serde_json::json!(ret_type));
        }
        if let Some(doc) = docstring {
            ast_node.attributes.insert("docstring".to_string(), serde_json::json!(doc));
        }

        // Extract expression AST for the entire function body
        if context.extract_expressions {
            ast_node.expression_ast = Some(self.expression_extractor.extract_expression(node, source)?);
        }

        // Create semantic block
        let mut semantic_block = SemanticBlock::new(
            "Function".to_string(),
            function_name,
            ast_node,
        );

        // Calculate complexity based on body statements
        semantic_block.complexity_score = self.calculate_function_complexity(&body_nodes);
        semantic_block.mark_generation_ready();

        Ok(semantic_block)
    }

    fn extract_class_definition(&self, node: Node, source: &str, context: &ExtractionContext) -> Result<SemanticBlock> {
        let mut cursor = node.walk();
        let mut class_name = String::new();
        let mut base_classes = Vec::new();
        let mut decorators = Vec::new();
        let mut methods = Vec::new();
        let mut attributes = Vec::new();

        for child in node.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    if class_name.is_empty() {
                        class_name = child.utf8_text(source.as_bytes())?.to_string();
                    }
                }
                "argument_list" => {
                    // Base classes
                    base_classes = self.extract_base_classes(child, source)?;
                }
                "block" => {
                    // Extract methods and class attributes
                    let (class_methods, class_attrs) = self.extract_class_body(child, source, context)?;
                    methods = class_methods;
                    attributes = class_attrs;
                }
                "decorator" => {
                    decorators.push(child.utf8_text(source.as_bytes())?.to_string());
                }
                _ => {}
            }
        }

        let source_range = self.node_to_source_range(node, source)?;
        let mut ast_node = ASTNode::new("class_definition".to_string(), source_range);
        ast_node.semantic_name = Some(class_name.clone());

        ast_node.attributes.insert("base_classes".to_string(), serde_json::json!(base_classes));
        ast_node.attributes.insert("decorators".to_string(), serde_json::json!(decorators));
        ast_node.attributes.insert("methods".to_string(), serde_json::json!(methods));
        ast_node.attributes.insert("attributes".to_string(), serde_json::json!(attributes));

        if context.extract_expressions {
            ast_node.expression_ast = Some(self.expression_extractor.extract_expression(node, source)?);
        }

        let mut semantic_block = SemanticBlock::new(
            "Class".to_string(),
            class_name,
            ast_node,
        );

        semantic_block.complexity_score = methods.len() as u32 + attributes.len() as u32 + 1;
        semantic_block.mark_generation_ready();

        Ok(semantic_block)
    }

    fn extract_assignment(&self, node: Node, source: &str, context: &ExtractionContext) -> Result<SemanticBlock> {
        let mut cursor = node.walk();
        let mut variable_name = String::new();
        let mut value_expression = None;
        let mut type_annotation = None;

        for child in node.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    if variable_name.is_empty() {
                        variable_name = child.utf8_text(source.as_bytes())?.to_string();
                    }
                }
                "type" => {
                    type_annotation = Some(child.utf8_text(source.as_bytes())?.to_string());
                }
                _ => {
                    // Assume this is the value expression
                    if context.extract_expressions {
                        value_expression = Some(self.expression_extractor.extract_expression(child, source)?);
                    }
                }
            }
        }

        let source_range = self.node_to_source_range(node, source)?;
        let mut ast_node = ASTNode::new("assignment".to_string(), source_range);
        ast_node.semantic_name = Some(variable_name.clone());

        if let Some(type_ann) = type_annotation {
            ast_node.attributes.insert("type_annotation".to_string(), serde_json::json!(type_ann));
        }

        if let Some(expr_ast) = value_expression {
            ast_node.expression_ast = Some(expr_ast);
        }

        let mut semantic_block = SemanticBlock::new(
            "Variable".to_string(),
            variable_name,
            ast_node,
        );

        semantic_block.complexity_score = 1;
        semantic_block.mark_generation_ready();

        Ok(semantic_block)
    }

    fn extract_import_statement(&self, node: Node, source: &str) -> Result<Vec<Dependency>> {
        let mut dependencies = Vec::new();
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            match child.kind() {
                "dotted_name" | "identifier" => {
                    let module_name = child.utf8_text(source.as_bytes())?.to_string();
                    dependencies.push(Dependency {
                        from_block_id: Uuid::new_v4(), // Will be set by caller
                        to_identifier: module_name.clone(),
                        dependency_type: DependencyType::Import,
                        is_external: !module_name.starts_with('.'), // Relative imports start with '.'
                        module_path: Some(module_name),
                    });
                }
                "import_from_statement" => {
                    // Handle 'from module import name' statements
                    let from_deps = self.extract_import_statement(child, source)?;
                    dependencies.extend(from_deps);
                }
                _ => {}
            }
        }

        Ok(dependencies)
    }

    // Helper methods
    fn extract_parameters(&self, node: Node, source: &str) -> Result<Vec<String>> {
        let mut parameters = Vec::new();
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                parameters.push(child.utf8_text(source.as_bytes())?.to_string());
            }
        }

        Ok(parameters)
    }

    fn extract_body_statements(&self, node: Node, source: &str) -> Result<Vec<String>> {
        let mut statements = Vec::new();
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if !matches!(child.kind(), "indent" | "dedent" | "\n") {
                statements.push(child.utf8_text(source.as_bytes())?.to_string());
            }
        }

        Ok(statements)
    }

    fn extract_base_classes(&self, node: Node, source: &str) -> Result<Vec<String>> {
        let mut base_classes = Vec::new();
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if matches!(child.kind(), "identifier" | "attribute") {
                base_classes.push(child.utf8_text(source.as_bytes())?.to_string());
            }
        }

        Ok(base_classes)
    }

    fn extract_class_body(&self, node: Node, source: &str, context: &ExtractionContext) -> Result<(Vec<String>, Vec<String>)> {
        let mut methods = Vec::new();
        let mut attributes = Vec::new();
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            match child.kind() {
                "function_definition" => {
                    if let Ok(func_block) = self.extract_function_definition(child, source, context) {
                        methods.push(func_block.semantic_name);
                    }
                }
                "assignment" => {
                    if let Ok(var_block) = self.extract_assignment(child, source, context) {
                        attributes.push(var_block.semantic_name);
                    }
                }
                _ => {}
            }
        }

        Ok((methods, attributes))
    }

    fn calculate_function_complexity(&self, statements: &[String]) -> u32 {
        let mut complexity = 1; // Base complexity

        for statement in statements {
            // Add complexity for control flow statements
            if statement.contains("if ") || statement.contains("elif ") {
                complexity += 1;
            }
            if statement.contains("for ") || statement.contains("while ") {
                complexity += 2;
            }
            if statement.contains("try:") || statement.contains("except ") {
                complexity += 1;
            }
            if statement.contains("with ") {
                complexity += 1;
            }
        }

        complexity
    }

    fn node_to_source_range(&self, node: Node, _source: &str) -> Result<SourceRange> {
        Ok(SourceRange {
            start_line: node.start_position().row,
            start_column: node.start_position().column,
            end_line: node.end_position().row,
            end_column: node.end_position().column,
            byte_start: node.start_byte(),
            byte_end: node.end_byte(),
        })
    }
}

impl ASTExtractor for PythonASTExtractor {
    fn extract(&self, node: Node, source: &str, context: &ExtractionContext) -> Result<ExtractionResult> {
        let start_time = std::time::Instant::now();
        
        let source_range = self.node_to_source_range(node, source)?;
        let root_node = ASTNode::new("module".to_string(), source_range);
        let mut result = ExtractionResult::new(root_node);

        let mut cursor = node.walk();
        
        // Extract all semantic blocks from the module
        for child in node.children(&mut cursor) {
            match child.kind() {
                "function_definition" => {
                    let block = self.extract_function_definition(child, source, context)?;
                    result.add_semantic_block(block);
                }
                "class_definition" => {
                    let block = self.extract_class_definition(child, source, context)?;
                    result.add_semantic_block(block);
                }
                "assignment" => {
                    let block = self.extract_assignment(child, source, context)?;
                    result.add_semantic_block(block);
                }
                "import_statement" | "import_from_statement" => {
                    let dependencies = self.extract_import_statement(child, source)?;
                    for dep in dependencies {
                        result.add_dependency(dep);
                    }
                }
                _ => {
                    // Handle other node types generically
                    if context.max_depth.map_or(true, |max| cursor.depth() < max) {
                        // Recursively process unknown nodes
                        let nested_result = self.extract(child, source, context)?;
                        result.semantic_blocks.extend(nested_result.semantic_blocks);
                        result.dependencies.extend(nested_result.dependencies);
                        result.exports.extend(nested_result.exports);
                    }
                }
            }
        }

        // Update metadata
        result.metadata.total_nodes = self.count_nodes(node);
        result.metadata.extraction_time_ms = start_time.elapsed().as_millis() as u64;
        result.metadata.language_specific.insert(
            "python_version".to_string(),
            serde_json::json!("3.8+")
        );

        Ok(result)
    }

    fn language(&self) -> &'static str {
        "python"
    }

    fn supports_extension(&self, extension: &str) -> bool {
        matches!(extension, "py" | "pyx" | "pyi")
    }

    fn extract_semantic_metadata(&self, node: Node, source: &str) -> Result<HashMap<String, serde_json::Value>> {
        let mut metadata = HashMap::new();
        
        metadata.insert("node_kind".to_string(), serde_json::json!(node.kind()));
        metadata.insert("source_text".to_string(), serde_json::json!(node.utf8_text(source.as_bytes())?));
        metadata.insert("line_count".to_string(), serde_json::json!(node.end_position().row - node.start_position().row + 1));
        
        // Add Python-specific metadata
        if matches!(node.kind(), "function_definition" | "class_definition") {
            metadata.insert("is_async".to_string(), serde_json::json!(
                node.utf8_text(source.as_bytes())?.contains("async ")
            ));
        }

        Ok(metadata)
    }
}

impl PythonASTExtractor {
    fn count_nodes(&self, node: Node) -> usize {
        let mut count = 1;
        let mut cursor = node.walk();
        
        for child in node.children(&mut cursor) {
            count += self.count_nodes(child);
        }
        
        count
    }
}

impl Default for PythonASTExtractor {
    fn default() -> Self {
        Self::new()
    }
}
