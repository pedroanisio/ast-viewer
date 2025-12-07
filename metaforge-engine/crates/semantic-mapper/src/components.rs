use serde::{Deserialize, Serialize};
use ast_extractor::ExpressionAST;

/// Semantic code components that can be generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CodeComponent {
    FunctionSignature(FunctionSignature),
    FunctionBody(FunctionBody),
    ClassDeclaration(ClassDeclaration),
    ClassBody(ClassBody),
    Variable(VariableDeclaration),
    Import(ImportStatement),
    Expression(ExpressionAST),
    Statement(Statement),
    Comment(Comment),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSignature {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<String>,
    pub is_async: bool,
    pub decorators: Vec<String>,
    pub type_parameters: Vec<String>, // Generic type parameters
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionBody {
    pub statements: Vec<Statement>,
    pub expressions: Vec<ExpressionAST>,
    pub local_variables: Vec<String>,
    pub called_functions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassDeclaration {
    pub name: String,
    pub base_classes: Vec<String>,
    pub decorators: Vec<String>,
    pub type_parameters: Vec<String>,
    pub is_abstract: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassBody {
    pub methods: Vec<FunctionSignature>,
    pub attributes: Vec<VariableDeclaration>,
    pub properties: Vec<Property>,
    pub static_methods: Vec<String>,
    pub class_methods: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableDeclaration {
    pub name: String,
    pub type_annotation: Option<TypeAnnotation>,
    pub initial_value: Option<ExpressionAST>,
    pub is_constant: bool,
    pub is_static: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportStatement {
    pub module_path: String,
    pub imported_names: Vec<ImportedName>,
    pub is_relative: bool,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedName {
    pub original: String,
    pub alias: Option<String>,
    pub is_type: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub type_hint: Option<String>,
    pub default_value: Option<ExpressionAST>,
    pub is_variadic: bool,
    pub is_keyword_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statement {
    pub statement_type: StatementType,
    pub expression: Option<ExpressionAST>,
    pub nested_statements: Vec<Statement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatementType {
    Return,
    If,
    While,
    For,
    Try,
    With,
    Assignment,
    Expression,
    Pass,
    Break,
    Continue,
    Raise,
    Assert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeAnnotation {
    pub base_type: String,
    pub type_parameters: Vec<String>,
    pub is_optional: bool,
    pub is_union: bool,
    pub union_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    pub name: String,
    pub getter: Option<String>,
    pub setter: Option<String>,
    pub deleter: Option<String>,
    pub type_annotation: Option<TypeAnnotation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub content: String,
    pub comment_type: CommentType,
    pub associated_element: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommentType {
    SingleLine,
    MultiLine,
    DocString,
    TypeComment,
}

impl CodeComponent {
    /// Get the semantic name of this component
    pub fn semantic_name(&self) -> Option<&str> {
        match self {
            CodeComponent::FunctionSignature(f) => Some(&f.name),
            CodeComponent::ClassDeclaration(c) => Some(&c.name),
            CodeComponent::Variable(v) => Some(&v.name),
            _ => None,
        }
    }

    /// Check if this component is ready for generation
    pub fn is_generation_ready(&self) -> bool {
        match self {
            CodeComponent::FunctionSignature(sig) => !sig.name.is_empty(),
            CodeComponent::FunctionBody(body) => !body.statements.is_empty() || !body.expressions.is_empty(),
            CodeComponent::ClassDeclaration(decl) => !decl.name.is_empty(),
            CodeComponent::ClassBody(_) => true,
            CodeComponent::Variable(var) => !var.name.is_empty(),
            CodeComponent::Import(imp) => !imp.module_path.is_empty(),
            CodeComponent::Expression(_) => true,
            CodeComponent::Statement(_) => true,
            CodeComponent::Comment(_) => true,
        }
    }

    /// Get complexity score for this component
    pub fn complexity_score(&self) -> u32 {
        match self {
            CodeComponent::FunctionSignature(sig) => sig.parameters.len() as u32 + sig.decorators.len() as u32,
            CodeComponent::FunctionBody(body) => {
                body.statements.len() as u32 * 2 + 
                body.expressions.len() as u32 +
                body.called_functions.len() as u32
            }
            CodeComponent::ClassDeclaration(decl) => decl.base_classes.len() as u32 + 1,
            CodeComponent::ClassBody(body) => body.methods.len() as u32 + body.attributes.len() as u32,
            CodeComponent::Variable(_) => 1,
            CodeComponent::Import(imp) => imp.imported_names.len() as u32,
            CodeComponent::Expression(expr) => expr.complexity_score,
            CodeComponent::Statement(stmt) => 1 + stmt.nested_statements.len() as u32,
            CodeComponent::Comment(_) => 0,
        }
    }
}

impl Parameter {
    pub fn new(name: String) -> Self {
        Self {
            name,
            type_hint: None,
            default_value: None,
            is_variadic: false,
            is_keyword_only: false,
        }
    }

    pub fn with_type(mut self, type_hint: String) -> Self {
        self.type_hint = Some(type_hint);
        self
    }

    pub fn with_default(mut self, default: ExpressionAST) -> Self {
        self.default_value = Some(default);
        self
    }
}

impl Statement {
    pub fn new(statement_type: StatementType) -> Self {
        Self {
            statement_type,
            expression: None,
            nested_statements: Vec::new(),
        }
    }

    pub fn with_expression(mut self, expr: ExpressionAST) -> Self {
        self.expression = Some(expr);
        self
    }

    pub fn with_nested(mut self, statements: Vec<Statement>) -> Self {
        self.nested_statements = statements;
        self
    }
}
