use anyhow::{Result, anyhow};
use serde_json::Value;
use std::collections::HashMap;
// use crate::core::*;
use crate::database::{Container, Block};
use crate::generator::formatters::LanguageFormatters;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TemplateEngine {
    templates: HashMap<String, LanguageTemplate>,
    formatters: LanguageFormatters,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LanguageTemplate {
    pub function_template: String,
    pub class_template: String,
    pub variable_template: String,
    pub import_template: String,
    pub comment_template: String,
    pub file_header_template: String,
    pub file_footer_template: String,
    
    // Phase 1B: Enhanced template coverage
    pub method_template: String,
    pub constructor_template: String,
    pub interface_template: String,
    pub enum_template: String,
    pub struct_template: String,
    pub trait_template: String,
    pub module_template: String,
    pub namespace_template: String,
    
    // Control flow templates
    pub if_template: String,
    pub for_template: String,
    pub while_template: String,
    pub try_catch_template: String,
    pub switch_template: String,
    pub loop_template: String,
    
    // Advanced language features
    pub generic_template: String,
    pub decorator_template: String,
    pub annotation_template: String,
    pub macro_template: String,
    pub lambda_template: String,
    pub closure_template: String,
}

#[allow(dead_code)]
impl TemplateEngine {
    pub fn new() -> Self {
        let mut templates = HashMap::new();
        
        // Rust templates - Phase 1B Complete Coverage
        templates.insert("rust".to_string(), LanguageTemplate {
            function_template: "{{visibility}}{{modifiers}}fn {{name}}{{generics}}({{params}}){{return_type}}{{where_clause}} {\n{{body}}\n}".to_string(),
            class_template: "{{visibility}}{{modifiers}}struct {{name}}{{generics}}{{where_clause}} {\n{{fields}}\n}".to_string(),
            variable_template: "{{modifiers}}let {{mutability}}{{name}}: {{type}} = {{value}};".to_string(),
            import_template: "use {{path}}{{alias}};".to_string(),
            comment_template: "// {{content}}".to_string(),
            file_header_template: "#![allow(unused)]\n// Generated from semantic blocks\n\n".to_string(),
            file_footer_template: "".to_string(),
            
            // Phase 1B: Enhanced template coverage
            method_template: "    {{visibility}}{{modifiers}}fn {{name}}{{generics}}({{self_param}}{{params}}){{return_type}}{{where_clause}} {\n{{body}}\n    }".to_string(),
            constructor_template: "    {{visibility}}fn new({{params}}) -> Self {\n{{body}}\n    }".to_string(),
            interface_template: "{{visibility}}trait {{name}}{{generics}}{{where_clause}} {\n{{methods}}\n}".to_string(),
            enum_template: "{{visibility}}{{modifiers}}enum {{name}}{{generics}}{{where_clause}} {\n{{variants}}\n}".to_string(),
            struct_template: "{{visibility}}{{modifiers}}struct {{name}}{{generics}}{{where_clause}} {\n{{fields}}\n}".to_string(),
            trait_template: "{{visibility}}trait {{name}}{{generics}}{{bounds}}{{where_clause}} {\n{{associated_types}}\n{{methods}}\n}".to_string(),
            module_template: "{{visibility}}mod {{name}} {\n{{content}}\n}".to_string(),
            namespace_template: "// Rust uses modules, not namespaces\nmod {{name}} {\n{{content}}\n}".to_string(),
            
            // Control flow templates
            if_template: "if {{condition}} {\n{{then_body}}\n}{{else_clause}}".to_string(),
            for_template: "for {{pattern}} in {{iterator}} {\n{{body}}\n}".to_string(),
            while_template: "while {{condition}} {\n{{body}}\n}".to_string(),
            try_catch_template: "match {{expression}} {\n    Ok({{ok_pattern}}) => {\n{{ok_body}}\n    },\n    Err({{err_pattern}}) => {\n{{err_body}}\n    }\n}".to_string(),
            switch_template: "match {{expression}} {\n{{arms}}\n}".to_string(),
            loop_template: "loop {\n{{body}}\n}".to_string(),
            
            // Advanced language features
            generic_template: "<{{type_params}}>".to_string(),
            decorator_template: "#[{{name}}{{args}}]".to_string(),
            annotation_template: "#[{{name}}{{args}}]".to_string(),
            macro_template: "macro_rules! {{name}} {\n{{rules}}\n}".to_string(),
            lambda_template: "|{{params}}| {{body}}".to_string(),
            closure_template: "{{move_keyword}}|{{params}}|{{return_type}} {\n{{body}}\n}".to_string(),
        });

        // Python templates - Phase 1B Complete Coverage
        templates.insert("python".to_string(), LanguageTemplate {
            function_template: "{{decorators}}{{async_keyword}}def {{name}}({{params}}){{return_type}}:\n{{docstring}}{{body}}".to_string(),
            class_template: "{{decorators}}class {{name}}({{bases}}):\n{{docstring}}{{body}}".to_string(),
            variable_template: "{{name}}{{type_hint}} = {{value}}".to_string(),
            import_template: "{{import_type}} {{path}}{{alias}}".to_string(),
            comment_template: "# {{content}}".to_string(),
            file_header_template: "#!/usr/bin/env python3\n# -*- coding: utf-8 -*-\n\"\"\"Generated from semantic blocks\"\"\"\n\n".to_string(),
            file_footer_template: "\n\nif __name__ == '__main__':\n    pass".to_string(),
            
            // Phase 1B: Enhanced template coverage
            method_template: "    {{decorators}}{{async_keyword}}def {{name}}({{params}}){{return_type}}:\n{{docstring}}{{body}}".to_string(),
            constructor_template: "    def __init__(self{{params}}):\n{{docstring}}{{body}}".to_string(),
            interface_template: "{{decorators}}class {{name}}({{bases}}):\n{{docstring}}    \"\"\"Interface: {{name}}\"\"\"\n{{methods}}".to_string(),
            enum_template: "class {{name}}({{enum_base}}):\n{{docstring}}{{values}}".to_string(),
            struct_template: "@dataclass\nclass {{name}}:\n{{docstring}}{{fields}}".to_string(),
            trait_template: "class {{name}}(ABC):\n{{docstring}}{{abstract_methods}}".to_string(),
            module_template: "# Module: {{name}}\n{{docstring}}\n{{content}}".to_string(),
            namespace_template: "# Namespace: {{name}}\n{{content}}".to_string(),
            
            // Control flow templates
            if_template: "if {{condition}}:\n{{then_body}}{{else_clause}}".to_string(),
            for_template: "for {{target}} in {{iterable}}:\n{{body}}".to_string(),
            while_template: "while {{condition}}:\n{{body}}".to_string(),
            try_catch_template: "try:\n{{try_body}}\nexcept {{exception_types}} as {{exception_var}}:\n{{except_body}}{{finally_clause}}".to_string(),
            switch_template: "match {{expression}}:\n{{cases}}".to_string(),  // Python 3.10+ match statement
            loop_template: "while True:\n{{body}}".to_string(),
            
            // Advanced language features
            generic_template: "[{{type_vars}}]".to_string(),
            decorator_template: "@{{name}}{{args}}".to_string(),
            annotation_template: "{{name}}: {{type}}".to_string(),
            macro_template: "# Python doesn't have macros - using function\ndef {{name}}({{params}}):\n{{body}}".to_string(),
            lambda_template: "lambda {{params}}: {{body}}".to_string(),
            closure_template: "def {{name}}({{outer_params}}):\n    def inner({{inner_params}}):\n{{body}}\n    return inner".to_string(),
        });

        // JavaScript templates - Phase 1B Complete Coverage
        templates.insert("javascript".to_string(), LanguageTemplate {
            function_template: "{{export_keyword}}{{async_keyword}}function {{name}}({{params}}) {\n{{body}}\n}".to_string(),
            class_template: "{{export_keyword}}class {{name}}{{extends}} {\n{{body}}\n}".to_string(),
            variable_template: "{{export_keyword}}{{declaration_type}} {{name}} = {{value}};".to_string(),
            import_template: "import {{imports}} from '{{path}}';".to_string(),
            comment_template: "// {{content}}".to_string(),
            file_header_template: "// Generated from semantic blocks\n'use strict';\n\n".to_string(),
            file_footer_template: "".to_string(),
            
            // Phase 1B: Enhanced template coverage
            method_template: "  {{async_keyword}}{{name}}({{params}}) {\n{{body}}\n  }".to_string(),
            constructor_template: "  constructor({{params}}) {\n{{body}}\n  }".to_string(),
            interface_template: "// JavaScript interfaces are implemented through classes\nclass {{name}} {\n{{methods}}\n}".to_string(),
            enum_template: "const {{name}} = Object.freeze({\n{{values}}\n});".to_string(),
            struct_template: "class {{name}} {\n  constructor({{params}}) {\n{{assignments}}\n  }\n}".to_string(),
            trait_template: "// JavaScript traits are implemented through mixins\nconst {{name}} = {\n{{methods}}\n};".to_string(),
            module_template: "// Module: {{name}}\n{{content}}".to_string(),
            namespace_template: "const {{name}} = {\n{{content}}\n};".to_string(),
            
            // Control flow templates
            if_template: "if ({{condition}}) {\n{{then_body}}\n}{{else_clause}}".to_string(),
            for_template: "for ({{initialization}}; {{condition}}; {{increment}}) {\n{{body}}\n}".to_string(),
            while_template: "while ({{condition}}) {\n{{body}}\n}".to_string(),
            try_catch_template: "try {\n{{try_body}}\n} catch ({{error_var}}) {\n{{catch_body}}\n}{{finally_clause}}".to_string(),
            switch_template: "switch ({{expression}}) {\n{{cases}}\n}".to_string(),
            loop_template: "while (true) {\n{{body}}\n}".to_string(),
            
            // Advanced language features
            generic_template: "// JavaScript doesn't have generics".to_string(),
            decorator_template: "@{{name}}{{args}}".to_string(),
            annotation_template: "// {{name}}: {{type}}".to_string(),
            macro_template: "// JavaScript doesn't have macros".to_string(),
            lambda_template: "({{params}}) => {{body}}".to_string(),
            closure_template: "({{outer_params}}) => {\n  return ({{inner_params}}) => {\n{{body}}\n  };\n}".to_string(),
        });

        // TypeScript templates - Phase 1B Complete Coverage
        templates.insert("typescript".to_string(), LanguageTemplate {
            function_template: "{{export_keyword}}{{async_keyword}}function {{name}}{{generics}}({{params}}){{return_type}} {\n{{body}}\n}".to_string(),
            class_template: "{{export_keyword}}{{abstract_keyword}}class {{name}}{{generics}}{{extends}}{{implements}} {\n{{body}}\n}".to_string(),
            variable_template: "{{export_keyword}}{{modifiers}} {{name}}{{type_annotation}} = {{value}};".to_string(),
            import_template: "import {{imports}} from '{{path}}';".to_string(),
            comment_template: "// {{content}}".to_string(),
            file_header_template: "// Generated from semantic blocks\n// TypeScript\n\n".to_string(),
            file_footer_template: "".to_string(),
            
            // Phase 1B: Enhanced template coverage
            method_template: "  {{visibility}}{{abstract_keyword}}{{async_keyword}}{{name}}{{generics}}({{params}}){{return_type}} {\n{{body}}\n  }".to_string(),
            constructor_template: "  constructor({{params}}) {\n{{body}}\n  }".to_string(),
            interface_template: "{{export_keyword}}interface {{name}}{{generics}}{{extends}} {\n{{members}}\n}".to_string(),
            enum_template: "{{export_keyword}}enum {{name}} {\n{{values}}\n}".to_string(),
            struct_template: "{{export_keyword}}type {{name}} = {\n{{fields}}\n};".to_string(),
            trait_template: "{{export_keyword}}interface {{name}}{{generics}}{{extends}} {\n{{methods}}\n}".to_string(),
            module_template: "namespace {{name}} {\n{{content}}\n}".to_string(),
            namespace_template: "namespace {{name}} {\n{{content}}\n}".to_string(),
            
            // Control flow templates
            if_template: "if ({{condition}}) {\n{{then_body}}\n}{{else_clause}}".to_string(),
            for_template: "for ({{initialization}}; {{condition}}; {{increment}}) {\n{{body}}\n}".to_string(),
            while_template: "while ({{condition}}) {\n{{body}}\n}".to_string(),
            try_catch_template: "try {\n{{try_body}}\n} catch ({{error_var}}{{error_type}}) {\n{{catch_body}}\n}{{finally_clause}}".to_string(),
            switch_template: "switch ({{expression}}) {\n{{cases}}\n}".to_string(),
            loop_template: "while (true) {\n{{body}}\n}".to_string(),
            
            // Advanced language features
            generic_template: "<{{type_params}}>".to_string(),
            decorator_template: "@{{name}}{{args}}".to_string(),
            annotation_template: "{{name}}: {{type}}".to_string(),
            macro_template: "// TypeScript doesn't have macros - use functions or generics".to_string(),
            lambda_template: "({{params}}){{return_type}} => {{body}}".to_string(),
            closure_template: "({{outer_params}}) => {\n  return ({{inner_params}}){{return_type}} => {\n{{body}}\n  };\n}".to_string(),
        });

        // Go templates - Phase 1B New Language Support
        templates.insert("go".to_string(), LanguageTemplate {
            function_template: "func {{name}}{{generics}}({{params}}){{return_type}} {\n{{body}}\n}".to_string(),
            class_template: "type {{name}} struct {\n{{fields}}\n}".to_string(),
            variable_template: "{{modifiers}} {{name}} {{type}} = {{value}}".to_string(),
            import_template: "import {{alias}} \"{{path}}\"".to_string(),
            comment_template: "// {{content}}".to_string(),
            file_header_template: "package {{package_name}}\n\n// Generated from semantic blocks\n\n".to_string(),
            file_footer_template: "".to_string(),
            
            method_template: "func ({{receiver}}) {{name}}{{generics}}({{params}}){{return_type}} {\n{{body}}\n}".to_string(),
            constructor_template: "func New{{name}}({{params}}) *{{name}} {\n{{body}}\n}".to_string(),
            interface_template: "type {{name}} interface {\n{{methods}}\n}".to_string(),
            enum_template: "type {{name}} {{underlying_type}}\n\nconst (\n{{values}}\n)".to_string(),
            struct_template: "type {{name}} struct {\n{{fields}}\n}".to_string(),
            trait_template: "type {{name}} interface {\n{{methods}}\n}".to_string(),
            module_template: "package {{name}}\n\n{{content}}".to_string(),
            namespace_template: "// Go uses packages, not namespaces\npackage {{name}}\n\n{{content}}".to_string(),
            
            if_template: "if {{condition}} {\n{{then_body}}\n}{{else_clause}}".to_string(),
            for_template: "for {{condition}} {\n{{body}}\n}".to_string(),
            while_template: "for {{condition}} {\n{{body}}\n}".to_string(),
            try_catch_template: "// Go uses error values, not exceptions\nif err := {{expression}}; err != nil {\n{{error_handling}}\n}".to_string(),
            switch_template: "switch {{expression}} {\n{{cases}}\n}".to_string(),
            loop_template: "for {\n{{body}}\n}".to_string(),
            
            generic_template: "[{{type_params}}]".to_string(),
            decorator_template: "// Go doesn't have decorators".to_string(),
            annotation_template: "// {{name}}: {{type}}".to_string(),
            macro_template: "//go:generate {{command}}".to_string(),
            lambda_template: "func({{params}}){{return_type}} { {{body}} }".to_string(),
            closure_template: "func({{outer_params}}) func({{inner_params}}){{return_type}} {\n  return func({{inner_params}}){{return_type}} {\n{{body}}\n  }\n}".to_string(),
        });

        // Java templates - Phase 1B New Language Support
        templates.insert("java".to_string(), LanguageTemplate {
            function_template: "{{visibility}} {{modifiers}} {{return_type}} {{name}}{{generics}}({{params}}){{throws}} {\n{{body}}\n}".to_string(),
            class_template: "{{visibility}} {{modifiers}} class {{name}}{{generics}}{{extends}}{{implements}} {\n{{body}}\n}".to_string(),
            variable_template: "{{visibility}} {{modifiers}} {{type}} {{name}} = {{value}};".to_string(),
            import_template: "import {{static_keyword}}{{path}};".to_string(),
            comment_template: "// {{content}}".to_string(),
            file_header_template: "// Generated from semantic blocks\npackage {{package_name}};\n\n".to_string(),
            file_footer_template: "".to_string(),
            
            method_template: "    {{visibility}} {{modifiers}} {{return_type}} {{name}}{{generics}}({{params}}){{throws}} {\n{{body}}\n    }".to_string(),
            constructor_template: "    {{visibility}} {{name}}({{params}}){{throws}} {\n{{body}}\n    }".to_string(),
            interface_template: "{{visibility}} interface {{name}}{{generics}}{{extends}} {\n{{methods}}\n}".to_string(),
            enum_template: "{{visibility}} enum {{name}}{{implements}} {\n{{values}};\n{{body}}\n}".to_string(),
            struct_template: "{{visibility}} class {{name}}{{generics}} {\n{{fields}}\n{{constructor}}\n}".to_string(),
            trait_template: "{{visibility}} interface {{name}}{{generics}}{{extends}} {\n{{methods}}\n}".to_string(),
            module_template: "package {{name}};\n\n{{content}}".to_string(),
            namespace_template: "package {{name}};\n\n{{content}}".to_string(),
            
            if_template: "if ({{condition}}) {\n{{then_body}}\n}{{else_clause}}".to_string(),
            for_template: "for ({{initialization}}; {{condition}}; {{increment}}) {\n{{body}}\n}".to_string(),
            while_template: "while ({{condition}}) {\n{{body}}\n}".to_string(),
            try_catch_template: "try {\n{{try_body}}\n}{{catch_blocks}}{{finally_clause}}".to_string(),
            switch_template: "switch ({{expression}}) {\n{{cases}}\n}".to_string(),
            loop_template: "while (true) {\n{{body}}\n}".to_string(),
            
            generic_template: "<{{type_params}}>".to_string(),
            decorator_template: "@{{name}}{{args}}".to_string(),
            annotation_template: "@{{name}}{{args}}".to_string(),
            macro_template: "// Java doesn't have macros - use annotations or code generation".to_string(),
            lambda_template: "({{params}}) -> {{body}}".to_string(),
            closure_template: "// Java closures are implemented via lambda expressions and method references".to_string(),
        });

        // C# templates - Phase 1B New Language Support
        templates.insert("csharp".to_string(), LanguageTemplate {
            function_template: "{{visibility}} {{modifiers}} {{return_type}} {{name}}{{generics}}({{params}}){{where_clause}} {\n{{body}}\n}".to_string(),
            class_template: "{{visibility}} {{modifiers}} class {{name}}{{generics}}{{inheritance}}{{where_clause}} {\n{{body}}\n}".to_string(),
            variable_template: "{{visibility}} {{modifiers}} {{type}} {{name}} = {{value}};".to_string(),
            import_template: "using {{alias}}{{path}};".to_string(),
            comment_template: "// {{content}}".to_string(),
            file_header_template: "// Generated from semantic blocks\nusing System;\n\nnamespace {{namespace}} {\n\n".to_string(),
            file_footer_template: "\n}".to_string(),
            
            method_template: "    {{visibility}} {{modifiers}} {{return_type}} {{name}}{{generics}}({{params}}){{where_clause}} {\n{{body}}\n    }".to_string(),
            constructor_template: "    {{visibility}} {{name}}({{params}}) {{base_call}} {\n{{body}}\n    }".to_string(),
            interface_template: "{{visibility}} interface {{name}}{{generics}}{{inheritance}}{{where_clause}} {\n{{members}}\n}".to_string(),
            enum_template: "{{visibility}} enum {{name}} {{underlying_type}} {\n{{values}}\n}".to_string(),
            struct_template: "{{visibility}} struct {{name}}{{generics}}{{inheritance}}{{where_clause}} {\n{{members}}\n}".to_string(),
            trait_template: "{{visibility}} interface {{name}}{{generics}}{{inheritance}}{{where_clause}} {\n{{methods}}\n}".to_string(),
            module_template: "namespace {{name}} {\n{{content}}\n}".to_string(),
            namespace_template: "namespace {{name}} {\n{{content}}\n}".to_string(),
            
            if_template: "if ({{condition}}) {\n{{then_body}}\n}{{else_clause}}".to_string(),
            for_template: "for ({{initialization}}; {{condition}}; {{increment}}) {\n{{body}}\n}".to_string(),
            while_template: "while ({{condition}}) {\n{{body}}\n}".to_string(),
            try_catch_template: "try {\n{{try_body}}\n}{{catch_blocks}}{{finally_clause}}".to_string(),
            switch_template: "switch ({{expression}}) {\n{{cases}}\n}".to_string(),
            loop_template: "while (true) {\n{{body}}\n}".to_string(),
            
            generic_template: "<{{type_params}}>".to_string(),
            decorator_template: "[{{name}}{{args}}]".to_string(),
            annotation_template: "[{{name}}{{args}}]".to_string(),
            macro_template: "#define {{name}}{{params}} {{body}}".to_string(),
            lambda_template: "({{params}}) => {{body}}".to_string(),
            closure_template: "({{outer_params}}) => {\n  return ({{inner_params}}) => {\n{{body}}\n  };\n}".to_string(),
        });

        // C++ templates - Phase 1B New Language Support
        templates.insert("cpp".to_string(), LanguageTemplate {
            function_template: "{{template_decl}}{{return_type}} {{name}}({{params}}){{specifiers}} {\n{{body}}\n}".to_string(),
            class_template: "{{template_decl}}class {{name}}{{inheritance}} {\n{{access_specifiers}}\n{{body}}\n};".to_string(),
            variable_template: "{{modifiers}} {{type}} {{name}} = {{value}};".to_string(),
            import_template: "#include {{angle_brackets}}{{path}}{{angle_brackets}}".to_string(),
            comment_template: "// {{content}}".to_string(),
            file_header_template: "// Generated from semantic blocks\n#pragma once\n\n".to_string(),
            file_footer_template: "".to_string(),
            
            method_template: "    {{return_type}} {{name}}({{params}}){{specifiers}} {\n{{body}}\n    }".to_string(),
            constructor_template: "    {{name}}({{params}}){{initializer_list}} {\n{{body}}\n    }".to_string(),
            interface_template: "class {{name}} {\npublic:\n{{pure_virtual_methods}}\n};".to_string(),
            enum_template: "{{enum_class}} {{name}}{{underlying_type}} {\n{{values}}\n};".to_string(),
            struct_template: "{{template_decl}}struct {{name}}{{inheritance}} {\n{{members}}\n};".to_string(),
            trait_template: "// C++ uses concepts for traits\n{{template_decl}}\nconcept {{name}} = {{constraints}};".to_string(),
            module_template: "// C++20 module\nmodule {{name}};\n\n{{content}}".to_string(),
            namespace_template: "namespace {{name}} {\n{{content}}\n}".to_string(),
            
            if_template: "if ({{condition}}) {\n{{then_body}}\n}{{else_clause}}".to_string(),
            for_template: "for ({{initialization}}; {{condition}}; {{increment}}) {\n{{body}}\n}".to_string(),
            while_template: "while ({{condition}}) {\n{{body}}\n}".to_string(),
            try_catch_template: "try {\n{{try_body}}\n}{{catch_blocks}}".to_string(),
            switch_template: "switch ({{expression}}) {\n{{cases}}\n}".to_string(),
            loop_template: "while (true) {\n{{body}}\n}".to_string(),
            
            generic_template: "template<{{template_params}}>".to_string(),
            decorator_template: "// C++ doesn't have decorators - use attributes\n[[{{name}}{{args}}]]".to_string(),
            annotation_template: "[[{{name}}{{args}}]]".to_string(),
            macro_template: "#define {{name}}{{params}} {{body}}".to_string(),
            lambda_template: "[{{capture}}]({{params}}){{return_type}} { {{body}} }".to_string(),
            closure_template: "[{{capture}}]({{outer_params}}) {\n  return [{{inner_capture}}]({{inner_params}}){{return_type}} {\n{{body}}\n  };\n}".to_string(),
        });

        // Ruby templates - Phase 1B New Language Support
        templates.insert("ruby".to_string(), LanguageTemplate {
            function_template: "def {{name}}{{params}}\n{{body}}\nend".to_string(),
            class_template: "class {{name}}{{inheritance}}\n{{body}}\nend".to_string(),
            variable_template: "{{name}} = {{value}}".to_string(),
            import_template: "require '{{path}}'".to_string(),
            comment_template: "# {{content}}".to_string(),
            file_header_template: "#!/usr/bin/env ruby\n# Generated from semantic blocks\n\n".to_string(),
            file_footer_template: "".to_string(),
            
            method_template: "  def {{name}}{{params}}\n{{body}}\n  end".to_string(),
            constructor_template: "  def initialize{{params}}\n{{body}}\n  end".to_string(),
            interface_template: "module {{name}}\n{{methods}}\nend".to_string(),
            enum_template: "module {{name}}\n{{constants}}\nend".to_string(),
            struct_template: "{{name}} = Struct.new({{fields}}) do\n{{methods}}\nend".to_string(),
            trait_template: "module {{name}}\n{{methods}}\nend".to_string(),
            module_template: "module {{name}}\n{{content}}\nend".to_string(),
            namespace_template: "module {{name}}\n{{content}}\nend".to_string(),
            
            if_template: "if {{condition}}\n{{then_body}}\n{{else_clause}}end".to_string(),
            for_template: "{{iterable}}.each do |{{variable}}|\n{{body}}\nend".to_string(),
            while_template: "while {{condition}}\n{{body}}\nend".to_string(),
            try_catch_template: "begin\n{{try_body}}\nrescue {{exception_types}} => {{exception_var}}\n{{rescue_body}}\n{{ensure_clause}}end".to_string(),
            switch_template: "case {{expression}}\n{{when_clauses}}\nend".to_string(),
            loop_template: "loop do\n{{body}}\nend".to_string(),
            
            generic_template: "# Ruby doesn't have generics".to_string(),
            decorator_template: "# Ruby uses method decorators\n{{name}} {{args}}".to_string(),
            annotation_template: "# {{name}}: {{type}}".to_string(),
            macro_template: "# Ruby uses metaprogramming instead of macros".to_string(),
            lambda_template: "lambda { |{{params}}| {{body}} }".to_string(),
            closure_template: "proc { |{{outer_params}}| proc { |{{inner_params}}| {{body}} } }".to_string(),
        });

        // PHP templates - Phase 1B New Language Support
        templates.insert("php".to_string(), LanguageTemplate {
            function_template: "{{visibility}} function {{name}}({{params}}){{return_type}} {\n{{body}}\n}".to_string(),
            class_template: "{{modifiers}} class {{name}}{{extends}}{{implements}} {\n{{body}}\n}".to_string(),
            variable_template: "{{visibility}} {{modifiers}} ${{name}}{{type_hint}} = {{value}};".to_string(),
            import_template: "{{use_type}} {{path}}{{alias}};".to_string(),
            comment_template: "// {{content}}".to_string(),
            file_header_template: "<?php\n// Generated from semantic blocks\n\n".to_string(),
            file_footer_template: "".to_string(),
            
            method_template: "    {{visibility}} {{modifiers}} function {{name}}({{params}}){{return_type}} {\n{{body}}\n    }".to_string(),
            constructor_template: "    {{visibility}} function __construct({{params}}) {\n{{body}}\n    }".to_string(),
            interface_template: "interface {{name}}{{extends}} {\n{{methods}}\n}".to_string(),
            enum_template: "enum {{name}}{{backing_type}} {\n{{cases}}\n}".to_string(),
            struct_template: "class {{name}} {\n{{properties}}\n{{constructor}}\n}".to_string(),
            trait_template: "trait {{name}} {\n{{methods}}\n}".to_string(),
            module_template: "namespace {{name}} {\n{{content}}\n}".to_string(),
            namespace_template: "namespace {{name}} {\n{{content}}\n}".to_string(),
            
            if_template: "if ({{condition}}) {\n{{then_body}}\n}{{else_clause}}".to_string(),
            for_template: "for ({{initialization}}; {{condition}}; {{increment}}) {\n{{body}}\n}".to_string(),
            while_template: "while ({{condition}}) {\n{{body}}\n}".to_string(),
            try_catch_template: "try {\n{{try_body}}\n}{{catch_blocks}}{{finally_clause}}".to_string(),
            switch_template: "switch ({{expression}}) {\n{{cases}}\n}".to_string(),
            loop_template: "while (true) {\n{{body}}\n}".to_string(),
            
            generic_template: "{{template_syntax}}".to_string(),
            decorator_template: "#[{{name}}{{args}}]".to_string(),
            annotation_template: "#[{{name}}{{args}}]".to_string(),
            macro_template: "// PHP doesn't have macros - use functions or classes".to_string(),
            lambda_template: "function({{params}}){{return_type}} { {{body}} }".to_string(),
            closure_template: "function({{outer_params}}) use ({{use_vars}}) { return function({{inner_params}}){{return_type}} { {{body}} }; }".to_string(),
        });

        Self { 
            templates,
            formatters: LanguageFormatters::new(),
        }
    }

    pub fn get_template(&self, language: &str) -> Result<&LanguageTemplate> {
        self.templates.get(language)
            .ok_or_else(|| anyhow!("No template found for language: {}", language))
    }

    pub fn render_block(&self, block: &Block, language: &str) -> Result<String> {
        let template = self.get_template(language)?;
        let empty_map = serde_json::Map::new();
        let metadata = block.metadata.as_ref()
            .and_then(|m| m.as_object())
            .unwrap_or(&empty_map);

        let block_type = block.block_type.as_str();
        let _semantic_name = block.semantic_name.as_deref().unwrap_or("unnamed");
        
        match block_type {
            "Function" => self.render_function(template, block, metadata),
            "Class" => self.render_class(template, block, metadata),
            "Variable" => self.render_variable(template, block, metadata),
            "Import" => self.render_import(template, block, metadata),
            "Comment" => self.render_comment(template, block, metadata),
            
            // Phase 1B: Enhanced block type support
            "Method" => self.render_method(template, block, metadata),
            "Constructor" => self.render_constructor(template, block, metadata),
            "Interface" => self.render_interface(template, block, metadata),
            "Enum" => self.render_enum(template, block, metadata),
            "Struct" => self.render_struct(template, block, metadata),
            "Trait" => self.render_trait(template, block, metadata),
            "Module" => self.render_module(template, block, metadata),
            "Namespace" => self.render_namespace(template, block, metadata),
            
            // Control flow blocks
            "If" => self.render_if(template, block, metadata),
            "For" => self.render_for(template, block, metadata),
            "While" => self.render_while(template, block, metadata),
            "TryCatch" => self.render_try_catch(template, block, metadata),
            "Switch" => self.render_switch(template, block, metadata),
            "Loop" => self.render_loop(template, block, metadata),
            
            // Advanced language features
            "Generic" => self.render_generic(template, block, metadata),
            "Decorator" => self.render_decorator(template, block, metadata),
            "Annotation" => self.render_annotation(template, block, metadata),
            "Macro" => self.render_macro(template, block, metadata),
            "Lambda" => self.render_lambda(template, block, metadata),
            "Closure" => self.render_closure(template, block, metadata),
            
            _ => Ok(format!("// Unknown block type: {}\n", block_type)),
        }
    }

    fn render_function(&self, template: &LanguageTemplate, block: &Block, metadata: &serde_json::Map<String, Value>) -> Result<String> {
        // ✅ ENHANCED: First priority - use preserved implementation if available
        // ✅ ENHANCED: Check for preserved implementation data in abstract_syntax
        if let Some(implementation) = block.abstract_syntax.get("implementation") {
            if let Some(original_body) = implementation.get("original_body") {
                if let Some(body_str) = original_body.as_str() {
                    if !body_str.trim().is_empty() {
                        // Use actual preserved implementation instead of template
                        let semantic_name = block.semantic_name.as_deref().unwrap_or("unnamed");
                        let params = self.extract_parameters(block, metadata)?;
                        let return_type = self.extract_return_type(block, metadata)?;
                        let modifiers = self.extract_modifiers(block)?;
                        
                        // Build function with preserved body
                        let mut rendered = template.function_template.clone();
                        rendered = rendered.replace("{{name}}", semantic_name);
                        rendered = rendered.replace("{{params}}", &params);
                        rendered = rendered.replace("{{return_type}}", &return_type);
                        rendered = rendered.replace("{{modifiers}}", &modifiers);
                        rendered = rendered.replace("{{body}}", body_str); // Use preserved body directly
                        
                        return Ok(rendered);
                    }
                }
            }
        }
        
        // Fallback to template generation
        let mut rendered = template.function_template.clone();
        
        let semantic_name = block.semantic_name.as_deref().unwrap_or("unnamed");
        
        // Extract parameters from semantic fields (preferred) or metadata (fallback)
        let params = self.extract_parameters(block, metadata)?;
        
        // Extract return type from semantic fields (preferred) or metadata (fallback)
        let return_type = self.extract_return_type(block, metadata)?;
        
        // Extract modifiers from semantic fields
        let modifiers = self.extract_modifiers(block)?;
        
        // Extract body from semantic AST (no raw_text fallback)
        let body = self.extract_function_body(block)?;

        // Replace template variables
        rendered = rendered.replace("{{name}}", semantic_name);
        rendered = rendered.replace("{{params}}", &params);
        rendered = rendered.replace("{{return_type}}", &return_type);
        rendered = rendered.replace("{{modifiers}}", &modifiers);
        rendered = rendered.replace("{{body}}", &body);

        Ok(rendered)
    }

    fn render_class(&self, template: &LanguageTemplate, block: &Block, metadata: &serde_json::Map<String, Value>) -> Result<String> {
        let mut rendered = template.class_template.clone();
        
        let semantic_name = block.semantic_name.as_deref().unwrap_or("unnamed");
        
        // Extract inheritance chain
        let bases = metadata.get("inheritance_chain")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().map(|b| b.as_str().unwrap_or("")).collect::<Vec<_>>().join(", "))
            .unwrap_or_default();

        // Extract fields and methods from metadata
        let fields = metadata.get("fields")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().map(|f| format!("    {}", f.as_str().unwrap_or("field"))).collect::<Vec<_>>().join("\n"))
            .unwrap_or_default();

        // Replace template variables
        rendered = rendered.replace("{{name}}", semantic_name);
        rendered = rendered.replace("{{bases}}", &bases);
        rendered = rendered.replace("{{fields}}", &fields);
        let extends_str = if bases.is_empty() { 
            String::new() 
        } else { 
            format!(" extends {}", bases) 
        };
        rendered = rendered.replace("{{extends}}", &extends_str);
        rendered = rendered.replace("{{generics}}", &self.extract_generics(block)?);
        rendered = rendered.replace("{{impl_blocks}}", &self.extract_impl_blocks(block)?);
        rendered = rendered.replace("{{modifiers}}", &self.extract_modifiers(block)?);

        Ok(rendered)
    }

    fn render_variable(&self, template: &LanguageTemplate, block: &Block, _metadata: &serde_json::Map<String, Value>) -> Result<String> {
        // ✅ ENHANCED: First priority - use preserved implementation with actual values
        // ✅ ENHANCED: Check for preserved implementation data in abstract_syntax
        if let Some(implementation) = block.abstract_syntax.get("implementation") {
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
                        
                        // Build variable declaration with preserved value
                        let mut rendered = template.variable_template.clone();
                        rendered = rendered.replace("{{name}}", semantic_name);
                        rendered = rendered.replace("{{value}}", &value_str);
                        rendered = rendered.replace("{{type}}", &self.extract_type_info(block)?);
                        rendered = rendered.replace("{{modifiers}}", &self.extract_modifiers(block)?);
                        rendered = rendered.replace("{{mutability}}", "");
                        rendered = rendered.replace("{{type_hint}}", "");
                        
                        return Ok(rendered);
                    } else if let Some(expression) = assignment_info.get("expression") {
                        // Use the preserved expression
                        let expr_str = expression.as_str().unwrap_or("None");
                        
                        let mut rendered = template.variable_template.clone();
                        rendered = rendered.replace("{{name}}", semantic_name);
                        rendered = rendered.replace("{{value}}", expr_str);
                        rendered = rendered.replace("{{type}}", &self.extract_type_info(block)?);
                        rendered = rendered.replace("{{modifiers}}", &self.extract_modifiers(block)?);
                        rendered = rendered.replace("{{mutability}}", "");
                        rendered = rendered.replace("{{type_hint}}", "");
                        
                        return Ok(rendered);
                    }
                }
            }
        }
        
        // Fallback to template generation
        let mut rendered = template.variable_template.clone();
        
        let semantic_name = block.semantic_name.as_deref().unwrap_or("unnamed");
        
        // Extract value from semantic AST
        let value = self.extract_variable_value(block)?;
        
        // Extract type information from semantic fields
        let type_info = self.extract_type_info(block)?;
        
        // Extract modifiers from semantic fields
        let modifiers = self.extract_modifiers(block)?;

        // Replace template variables
        rendered = rendered.replace("{{name}}", semantic_name);
        rendered = rendered.replace("{{value}}", &value);
        rendered = rendered.replace("{{type}}", &type_info);
        rendered = rendered.replace("{{modifiers}}", &modifiers);

        Ok(rendered)
    }

    fn render_import(&self, template: &LanguageTemplate, block: &Block, _metadata: &serde_json::Map<String, Value>) -> Result<String> {
        let mut rendered = template.import_template.clone();
        
        // Extract import path from semantic AST
        let path = self.extract_import_path(block)?;

        // Replace template variables
        rendered = rendered.replace("{{path}}", &path);

        Ok(rendered)
    }

    fn render_comment(&self, template: &LanguageTemplate, block: &Block, _metadata: &serde_json::Map<String, Value>) -> Result<String> {
        let mut rendered = template.comment_template.clone();
        
        // Extract comment content from semantic structure
        let content = self.extract_comment_content(block)?;

        // Replace template variables
        rendered = rendered.replace("{{content}}", &content);

        Ok(rendered)
    }

    pub fn render_file(&self, _container: &Container, blocks: &[Block], language: &str) -> Result<String> {
        let template = self.get_template(language)?;
        let mut content = template.file_header_template.clone();
        
        // Sort blocks by position
        let mut sorted_blocks = blocks.to_vec();
        sorted_blocks.sort_by_key(|b| b.position);
        
        // Render each block
        for block in sorted_blocks {
            let rendered_block = self.render_block(&block, language)?;
            content.push_str(&rendered_block);
            content.push('\n');
        }
        
        content.push_str(&template.file_footer_template);
        
        // Phase 1B: Format the generated code
        self.format_code(&content, language)
    }

    /// Format generated code using appropriate language formatter
    pub fn format_code(&self, code: &str, language: &str) -> Result<String> {
        self.formatters.format_code(code, language)
    }

    // Semantic extraction methods - replace raw_text dependencies
    
    fn extract_parameters(&self, block: &Block, metadata: &serde_json::Map<String, Value>) -> Result<String> {
        // First try semantic fields from database
        if let Some(params) = &block.parameters {
            if let Some(param_array) = params.as_array() {
                let param_strings: Vec<String> = param_array.iter()
                    .filter_map(|p| {
                        if let Some(obj) = p.as_object() {
                            let name = obj.get("name")?.as_str()?;
                            let type_hint = obj.get("type_hint")
                                .and_then(|t| t.as_str())
                                .map(|t| format!(": {}", t))
                                .unwrap_or_default();
                            Some(format!("{}{}", name, type_hint))
                        } else {
                            None
                        }
                    })
                    .collect();
                return Ok(param_strings.join(", "));
            }
        }
        
        // Fallback to metadata
        Ok(metadata.get("parameters")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().map(|p| p.as_str().unwrap_or("param")).collect::<Vec<_>>().join(", "))
            .unwrap_or_default())
    }
    
    fn extract_return_type(&self, block: &Block, metadata: &serde_json::Map<String, Value>) -> Result<String> {
        // First try semantic fields from database
        if let Some(return_type) = &block.return_type {
            if !return_type.is_empty() {
                return Ok(format!(" -> {}", return_type));
            }
        }
        
        // Fallback to metadata
        Ok(metadata.get("return_type")
            .and_then(|v| v.as_str())
            .map(|t| if t.is_empty() { String::new() } else { format!(" -> {}", t) })
            .unwrap_or_default())
    }
    
    fn extract_modifiers(&self, block: &Block) -> Result<String> {
        if let Some(modifiers) = &block.modifiers {
            let modifier_str = modifiers.join(" ");
            if !modifier_str.is_empty() {
                return Ok(format!("{} ", modifier_str));
            }
        }
        Ok(String::new())
    }
    
    fn extract_generics(&self, block: &Block) -> Result<String> {
        // Extract generics from language_features or abstract_syntax
        if let Some(features) = &block.language_features {
            if let Some(generics) = features.get("generics") {
                if let Some(generic_params) = generics.as_array() {
                    let params: Vec<String> = generic_params.iter()
                        .filter_map(|g| g.as_str().map(|s| s.to_string()))
                        .collect();
                    if !params.is_empty() {
                        return Ok(format!("<{}>", params.join(", ")));
                    }
                }
            }
        }
        Ok(String::new())
    }
    
    fn extract_impl_blocks(&self, _block: &Block) -> Result<String> {
        // For now, return empty - impl blocks would be separate blocks in the hierarchical system
        Ok(String::new())
    }
    
    fn extract_function_body(&self, block: &Block) -> Result<String> {
        // Extract body from AST structure, not raw text
        if let Some(body_ast) = &block.body_ast {
            if let Some(statements) = body_ast.get("statements") {
                if let Some(stmt_array) = statements.as_array() {
                    let body_lines: Vec<String> = stmt_array.iter()
                        .filter_map(|stmt| {
                            stmt.get("code").and_then(|c| c.as_str().map(|s| format!("    {}", s)))
                        })
                        .collect();
                    if !body_lines.is_empty() {
                        return Ok(body_lines.join("\n"));
                    }
                }
            }
        }
        
        // Extract from abstract_syntax with semantic structure
        if let Some(body) = block.abstract_syntax.get("body") {
            if let Some(body_str) = body.as_str() {
                return Ok(format!("    {}", body_str));
            }
        }
        
        // Generate implementation from semantic structure
        Ok("    // Implementation generated from semantic blocks".to_string())
    }
    
    fn extract_variable_value(&self, block: &Block) -> Result<String> {
        // Extract from semantic AST structure
        if let Some(value) = block.abstract_syntax.get("value") {
            if let Some(value_str) = value.as_str() {
                return Ok(value_str.to_string());
            }
        }
        
        // Extract from initialization expression
        if let Some(init) = block.abstract_syntax.get("initializer") {
            if let Some(init_str) = init.as_str() {
                return Ok(init_str.to_string());
            }
        }
        
        Ok("undefined".to_string())
    }
    
    fn extract_type_info(&self, block: &Block) -> Result<String> {
        // Extract from language_features type annotations
        if let Some(features) = &block.language_features {
            if let Some(type_info) = features.get("type_annotation") {
                if let Some(type_str) = type_info.as_str() {
                    return Ok(format!(": {}", type_str));
                }
            }
        }
        
        // Extract from abstract_syntax
        if let Some(type_info) = block.abstract_syntax.get("type") {
            if let Some(type_str) = type_info.as_str() {
                return Ok(format!(": {}", type_str));
            }
        }
        
        Ok(String::new())
    }
    
    fn extract_import_path(&self, block: &Block) -> Result<String> {
        // Extract from semantic structure
        if let Some(path) = block.abstract_syntax.get("module_path") {
            if let Some(path_str) = path.as_str() {
                return Ok(path_str.to_string());
            }
        }
        
        if let Some(path) = block.abstract_syntax.get("path") {
            if let Some(path_str) = path.as_str() {
                return Ok(path_str.to_string());
            }
        }
        
        Ok("module".to_string())
    }
    
    fn extract_comment_content(&self, block: &Block) -> Result<String> {
        // Extract comment content from semantic structure
        if let Some(content) = block.abstract_syntax.get("content") {
            if let Some(content_str) = content.as_str() {
                return Ok(content_str.to_string());
            }
        }
        
        if let Some(text) = block.abstract_syntax.get("text") {
            if let Some(text_str) = text.as_str() {
                return Ok(text_str.to_string());
            }
        }
        
        Ok("Comment".to_string())
    }

    // Phase 1B: Enhanced render methods for complete template coverage
    
    fn render_method(&self, template: &LanguageTemplate, block: &Block, metadata: &serde_json::Map<String, Value>) -> Result<String> {
        let mut rendered = template.method_template.clone();
        
        let semantic_name = block.semantic_name.as_deref().unwrap_or("unnamed");
        let params = self.extract_parameters(block, metadata)?;
        let return_type = self.extract_return_type(block, metadata)?;
        let modifiers = self.extract_modifiers(block)?;
        let body = self.extract_function_body(block)?;
        let visibility = self.extract_visibility(block)?;
        let generics = self.extract_generics(block)?;
        
        rendered = rendered.replace("{{name}}", semantic_name);
        rendered = rendered.replace("{{params}}", &params);
        rendered = rendered.replace("{{return_type}}", &return_type);
        rendered = rendered.replace("{{modifiers}}", &modifiers);
        rendered = rendered.replace("{{body}}", &body);
        rendered = rendered.replace("{{visibility}}", &visibility);
        rendered = rendered.replace("{{generics}}", &generics);
        rendered = rendered.replace("{{self_param}}", &self.extract_self_param(block)?);
        rendered = rendered.replace("{{where_clause}}", &self.extract_where_clause(block)?);
        rendered = rendered.replace("{{async_keyword}}", &self.extract_async_keyword(block)?);
        
        Ok(rendered)
    }
    
    fn render_constructor(&self, template: &LanguageTemplate, block: &Block, metadata: &serde_json::Map<String, Value>) -> Result<String> {
        let mut rendered = template.constructor_template.clone();
        
        let params = self.extract_parameters(block, metadata)?;
        let body = self.extract_function_body(block)?;
        let visibility = self.extract_visibility(block)?;
        
        rendered = rendered.replace("{{params}}", &params);
        rendered = rendered.replace("{{body}}", &body);
        rendered = rendered.replace("{{visibility}}", &visibility);
        
        Ok(rendered)
    }
    
    fn render_interface(&self, template: &LanguageTemplate, block: &Block, metadata: &serde_json::Map<String, Value>) -> Result<String> {
        let mut rendered = template.interface_template.clone();
        
        let semantic_name = block.semantic_name.as_deref().unwrap_or("unnamed");
        let methods = self.extract_interface_methods(block, metadata)?;
        let generics = self.extract_generics(block)?;
        let extends = self.extract_extends(block)?;
        let visibility = self.extract_visibility(block)?;
        
        rendered = rendered.replace("{{name}}", semantic_name);
        rendered = rendered.replace("{{methods}}", &methods);
        rendered = rendered.replace("{{generics}}", &generics);
        rendered = rendered.replace("{{extends}}", &extends);
        rendered = rendered.replace("{{visibility}}", &visibility);
        rendered = rendered.replace("{{members}}", &methods);
        rendered = rendered.replace("{{where_clause}}", &self.extract_where_clause(block)?);
        
        Ok(rendered)
    }
    
    fn render_enum(&self, template: &LanguageTemplate, block: &Block, metadata: &serde_json::Map<String, Value>) -> Result<String> {
        let mut rendered = template.enum_template.clone();
        
        let semantic_name = block.semantic_name.as_deref().unwrap_or("unnamed");
        let values = self.extract_enum_values(block, metadata)?;
        let visibility = self.extract_visibility(block)?;
        let modifiers = self.extract_modifiers(block)?;
        
        rendered = rendered.replace("{{name}}", semantic_name);
        rendered = rendered.replace("{{values}}", &values);
        rendered = rendered.replace("{{visibility}}", &visibility);
        rendered = rendered.replace("{{modifiers}}", &modifiers);
        rendered = rendered.replace("{{where_clause}}", &self.extract_where_clause(block)?);
        rendered = rendered.replace("{{variants}}", &values);
        rendered = rendered.replace("{{generics}}", &self.extract_generics(block)?);
        
        Ok(rendered)
    }
    
    fn render_struct(&self, template: &LanguageTemplate, block: &Block, metadata: &serde_json::Map<String, Value>) -> Result<String> {
        let mut rendered = template.struct_template.clone();
        
        let semantic_name = block.semantic_name.as_deref().unwrap_or("unnamed");
        let fields = self.extract_struct_fields(block, metadata)?;
        let visibility = self.extract_visibility(block)?;
        let modifiers = self.extract_modifiers(block)?;
        let generics = self.extract_generics(block)?;
        
        rendered = rendered.replace("{{name}}", semantic_name);
        rendered = rendered.replace("{{fields}}", &fields);
        rendered = rendered.replace("{{visibility}}", &visibility);
        rendered = rendered.replace("{{modifiers}}", &modifiers);
        rendered = rendered.replace("{{generics}}", &generics);
        rendered = rendered.replace("{{where_clause}}", &self.extract_where_clause(block)?);
        rendered = rendered.replace("{{members}}", &fields);
        
        Ok(rendered)
    }
    
    fn render_trait(&self, template: &LanguageTemplate, block: &Block, metadata: &serde_json::Map<String, Value>) -> Result<String> {
        let mut rendered = template.trait_template.clone();
        
        let semantic_name = block.semantic_name.as_deref().unwrap_or("unnamed");
        let methods = self.extract_interface_methods(block, metadata)?;
        let visibility = self.extract_visibility(block)?;
        let generics = self.extract_generics(block)?;
        
        rendered = rendered.replace("{{name}}", semantic_name);
        rendered = rendered.replace("{{methods}}", &methods);
        rendered = rendered.replace("{{visibility}}", &visibility);
        rendered = rendered.replace("{{generics}}", &generics);
        rendered = rendered.replace("{{where_clause}}", &self.extract_where_clause(block)?);
        rendered = rendered.replace("{{bounds}}", &self.extract_trait_bounds(block)?);
        rendered = rendered.replace("{{associated_types}}", &self.extract_associated_types(block)?);
        
        Ok(rendered)
    }
    
    fn render_module(&self, template: &LanguageTemplate, block: &Block, _metadata: &serde_json::Map<String, Value>) -> Result<String> {
        let mut rendered = template.module_template.clone();
        
        let semantic_name = block.semantic_name.as_deref().unwrap_or("unnamed");
        let content = self.extract_module_content(block)?;
        let visibility = self.extract_visibility(block)?;
        
        rendered = rendered.replace("{{name}}", semantic_name);
        rendered = rendered.replace("{{content}}", &content);
        rendered = rendered.replace("{{visibility}}", &visibility);
        
        Ok(rendered)
    }
    
    fn render_namespace(&self, template: &LanguageTemplate, block: &Block, _metadata: &serde_json::Map<String, Value>) -> Result<String> {
        let mut rendered = template.namespace_template.clone();
        
        let semantic_name = block.semantic_name.as_deref().unwrap_or("unnamed");
        let content = self.extract_module_content(block)?;
        
        rendered = rendered.replace("{{name}}", semantic_name);
        rendered = rendered.replace("{{content}}", &content);
        
        Ok(rendered)
    }
    
    // Control flow render methods
    
    fn render_if(&self, template: &LanguageTemplate, block: &Block, metadata: &serde_json::Map<String, Value>) -> Result<String> {
        let mut rendered = template.if_template.clone();
        
        let condition = self.extract_condition(block, metadata)?;
        let then_body = self.extract_then_body(block)?;
        let else_clause = self.extract_else_clause(block)?;
        
        rendered = rendered.replace("{{condition}}", &condition);
        rendered = rendered.replace("{{then_body}}", &then_body);
        rendered = rendered.replace("{{else_clause}}", &else_clause);
        
        Ok(rendered)
    }
    
    fn render_for(&self, template: &LanguageTemplate, block: &Block, metadata: &serde_json::Map<String, Value>) -> Result<String> {
        let mut rendered = template.for_template.clone();
        
        let initialization = self.extract_for_initialization(block, metadata)?;
        let condition = self.extract_condition(block, metadata)?;
        let increment = self.extract_for_increment(block, metadata)?;
        let body = self.extract_function_body(block)?;
        
        rendered = rendered.replace("{{initialization}}", &initialization);
        rendered = rendered.replace("{{condition}}", &condition);
        rendered = rendered.replace("{{increment}}", &increment);
        rendered = rendered.replace("{{body}}", &body);
        rendered = rendered.replace("{{pattern}}", &self.extract_for_pattern(block)?);
        rendered = rendered.replace("{{iterator}}", &self.extract_for_iterator(block)?);
        rendered = rendered.replace("{{target}}", &self.extract_for_target(block)?);
        rendered = rendered.replace("{{iterable}}", &self.extract_for_iterable(block)?);
        rendered = rendered.replace("{{variable}}", &self.extract_for_variable(block)?);
        
        Ok(rendered)
    }
    
    fn render_while(&self, template: &LanguageTemplate, block: &Block, metadata: &serde_json::Map<String, Value>) -> Result<String> {
        let mut rendered = template.while_template.clone();
        
        let condition = self.extract_condition(block, metadata)?;
        let body = self.extract_function_body(block)?;
        
        rendered = rendered.replace("{{condition}}", &condition);
        rendered = rendered.replace("{{body}}", &body);
        
        Ok(rendered)
    }
    
    fn render_try_catch(&self, template: &LanguageTemplate, block: &Block, metadata: &serde_json::Map<String, Value>) -> Result<String> {
        let mut rendered = template.try_catch_template.clone();
        
        let try_body = self.extract_try_body(block)?;
        let catch_body = self.extract_catch_body(block)?;
        let exception_var = self.extract_exception_var(block, metadata)?;
        let exception_types = self.extract_exception_types(block, metadata)?;
        
        rendered = rendered.replace("{{try_body}}", &try_body);
        rendered = rendered.replace("{{catch_body}}", &catch_body);
        rendered = rendered.replace("{{except_body}}", &catch_body);
        rendered = rendered.replace("{{rescue_body}}", &catch_body);
        rendered = rendered.replace("{{exception_var}}", &exception_var);
        rendered = rendered.replace("{{exception_types}}", &exception_types);
        rendered = rendered.replace("{{error_var}}", &exception_var);
        rendered = rendered.replace("{{error_type}}", &exception_types);
        rendered = rendered.replace("{{finally_clause}}", &self.extract_finally_clause(block)?);
        rendered = rendered.replace("{{ensure_clause}}", &self.extract_finally_clause(block)?);
        rendered = rendered.replace("{{catch_blocks}}", &self.extract_catch_blocks(block)?);
        rendered = rendered.replace("{{expression}}", &self.extract_try_expression(block)?);
        rendered = rendered.replace("{{error_handling}}", &catch_body);
        
        Ok(rendered)
    }
    
    fn render_switch(&self, template: &LanguageTemplate, block: &Block, metadata: &serde_json::Map<String, Value>) -> Result<String> {
        let mut rendered = template.switch_template.clone();
        
        let expression = self.extract_switch_expression(block, metadata)?;
        let cases = self.extract_switch_cases(block)?;
        
        rendered = rendered.replace("{{expression}}", &expression);
        rendered = rendered.replace("{{cases}}", &cases);
        rendered = rendered.replace("{{arms}}", &cases);
        rendered = rendered.replace("{{when_clauses}}", &cases);
        
        Ok(rendered)
    }
    
    fn render_loop(&self, template: &LanguageTemplate, block: &Block, _metadata: &serde_json::Map<String, Value>) -> Result<String> {
        let mut rendered = template.loop_template.clone();
        
        let body = self.extract_function_body(block)?;
        
        rendered = rendered.replace("{{body}}", &body);
        
        Ok(rendered)
    }
    
    // Advanced language feature render methods
    
    fn render_generic(&self, template: &LanguageTemplate, block: &Block, metadata: &serde_json::Map<String, Value>) -> Result<String> {
        let mut rendered = template.generic_template.clone();
        
        let type_params = self.extract_generic_type_params(block, metadata)?;
        
        rendered = rendered.replace("{{type_params}}", &type_params);
        rendered = rendered.replace("{{type_vars}}", &type_params);
        rendered = rendered.replace("{{template_params}}", &type_params);
        
        Ok(rendered)
    }
    
    fn render_decorator(&self, template: &LanguageTemplate, block: &Block, metadata: &serde_json::Map<String, Value>) -> Result<String> {
        let mut rendered = template.decorator_template.clone();
        
        let semantic_name = block.semantic_name.as_deref().unwrap_or("decorator");
        let args = self.extract_decorator_args(block, metadata)?;
        
        rendered = rendered.replace("{{name}}", semantic_name);
        rendered = rendered.replace("{{args}}", &args);
        
        Ok(rendered)
    }
    
    fn render_annotation(&self, template: &LanguageTemplate, block: &Block, metadata: &serde_json::Map<String, Value>) -> Result<String> {
        let mut rendered = template.annotation_template.clone();
        
        let semantic_name = block.semantic_name.as_deref().unwrap_or("annotation");
        let type_info = self.extract_type_info(block)?;
        let args = self.extract_decorator_args(block, metadata)?;
        
        rendered = rendered.replace("{{name}}", semantic_name);
        rendered = rendered.replace("{{type}}", &type_info);
        rendered = rendered.replace("{{args}}", &args);
        
        Ok(rendered)
    }
    
    fn render_macro(&self, template: &LanguageTemplate, block: &Block, metadata: &serde_json::Map<String, Value>) -> Result<String> {
        let mut rendered = template.macro_template.clone();
        
        let semantic_name = block.semantic_name.as_deref().unwrap_or("macro");
        let params = self.extract_parameters(block, metadata)?;
        let body = self.extract_function_body(block)?;
        
        rendered = rendered.replace("{{name}}", semantic_name);
        rendered = rendered.replace("{{params}}", &params);
        rendered = rendered.replace("{{body}}", &body);
        rendered = rendered.replace("{{rules}}", &body);
        rendered = rendered.replace("{{command}}", &body);
        
        Ok(rendered)
    }
    
    fn render_lambda(&self, template: &LanguageTemplate, block: &Block, metadata: &serde_json::Map<String, Value>) -> Result<String> {
        let mut rendered = template.lambda_template.clone();
        
        let params = self.extract_parameters(block, metadata)?;
        let body = self.extract_function_body(block)?;
        let return_type = self.extract_return_type(block, metadata)?;
        
        rendered = rendered.replace("{{params}}", &params);
        rendered = rendered.replace("{{body}}", &body);
        rendered = rendered.replace("{{return_type}}", &return_type);
        
        Ok(rendered)
    }
    
    fn render_closure(&self, template: &LanguageTemplate, block: &Block, metadata: &serde_json::Map<String, Value>) -> Result<String> {
        let mut rendered = template.closure_template.clone();
        
        let outer_params = self.extract_outer_params(block, metadata)?;
        let inner_params = self.extract_inner_params(block, metadata)?;
        let body = self.extract_function_body(block)?;
        let return_type = self.extract_return_type(block, metadata)?;
        
        rendered = rendered.replace("{{outer_params}}", &outer_params);
        rendered = rendered.replace("{{inner_params}}", &inner_params);
        rendered = rendered.replace("{{body}}", &body);
        rendered = rendered.replace("{{return_type}}", &return_type);
        rendered = rendered.replace("{{capture}}", &self.extract_closure_capture(block)?);
        rendered = rendered.replace("{{inner_capture}}", &self.extract_inner_closure_capture(block)?);
        rendered = rendered.replace("{{move_keyword}}", &self.extract_move_keyword(block)?);
        rendered = rendered.replace("{{use_vars}}", &self.extract_use_vars(block)?);
        
        Ok(rendered)
    }

    // Helper extraction methods for Phase 1B features
    
    fn extract_visibility(&self, block: &Block) -> Result<String> {
        if let Some(modifiers) = &block.modifiers {
            for modifier in modifiers {
                if ["public", "private", "protected", "internal", "pub", "priv"].contains(&modifier.as_str()) {
                    return Ok(format!("{} ", modifier));
                }
            }
        }
        Ok(String::new())
    }
    
    fn extract_self_param(&self, block: &Block) -> Result<String> {
        if let Some(params) = &block.parameters {
            if let Some(param_array) = params.as_array() {
                if let Some(first_param) = param_array.first() {
                    if let Some(obj) = first_param.as_object() {
                        if let Some(name) = obj.get("name").and_then(|n| n.as_str()) {
                            if ["self", "&self", "&mut self", "this"].contains(&name) {
                                return Ok(format!("{}, ", name));
                            }
                        }
                    }
                }
            }
        }
        Ok(String::new())
    }
    
    fn extract_where_clause(&self, block: &Block) -> Result<String> {
        if let Some(features) = &block.language_features {
            if let Some(where_clause) = features.get("where_clause") {
                if let Some(clause_str) = where_clause.as_str() {
                    return Ok(format!(" where {}", clause_str));
                }
            }
        }
        Ok(String::new())
    }
    
    fn extract_async_keyword(&self, block: &Block) -> Result<String> {
        if let Some(modifiers) = &block.modifiers {
            if modifiers.contains(&"async".to_string()) {
                return Ok("async ".to_string());
            }
        }
        Ok(String::new())
    }
    
    fn extract_interface_methods(&self, block: &Block, _metadata: &serde_json::Map<String, Value>) -> Result<String> {
        if let Some(body_ast) = &block.body_ast {
            if let Some(methods) = body_ast.get("methods") {
                if let Some(method_array) = methods.as_array() {
                    let method_strings: Vec<String> = method_array.iter()
                        .filter_map(|m| m.get("signature").and_then(|s| s.as_str().map(|s| format!("    {};", s))))
                        .collect();
                    return Ok(method_strings.join("\n"));
                }
            }
        }
        Ok("    // TODO: Define interface methods".to_string())
    }
    
    fn extract_enum_values(&self, block: &Block, _metadata: &serde_json::Map<String, Value>) -> Result<String> {
        if let Some(body_ast) = &block.body_ast {
            if let Some(values) = body_ast.get("values") {
                if let Some(value_array) = values.as_array() {
                    let value_strings: Vec<String> = value_array.iter()
                        .filter_map(|v| v.as_str().map(|s| format!("    {}", s)))
                        .collect();
                    return Ok(value_strings.join(",\n"));
                }
            }
        }
        Ok("    // TODO: Define enum values".to_string())
    }
    
    fn extract_struct_fields(&self, block: &Block, _metadata: &serde_json::Map<String, Value>) -> Result<String> {
        if let Some(body_ast) = &block.body_ast {
            if let Some(fields) = body_ast.get("fields") {
                if let Some(field_array) = fields.as_array() {
                    let field_strings: Vec<String> = field_array.iter()
                        .filter_map(|f| {
                            if let Some(obj) = f.as_object() {
                                let name = obj.get("name")?.as_str()?;
                                let type_info = obj.get("type")?.as_str()?;
                                Some(format!("    {}: {},", name, type_info))
                            } else {
                                None
                            }
                        })
                        .collect();
                    return Ok(field_strings.join("\n"));
                }
            }
        }
        Ok("    // TODO: Define struct fields".to_string())
    }
    
    fn extract_extends(&self, block: &Block) -> Result<String> {
        if let Some(inheritance) = block.abstract_syntax.get("inheritance") {
            if let Some(extends) = inheritance.get("extends") {
                if let Some(parent) = extends.as_str() {
                    return Ok(format!(" extends {}", parent));
                }
            }
        }
        Ok(String::new())
    }
    
    fn extract_trait_bounds(&self, _block: &Block) -> Result<String> {
        // Extract trait bounds for generic constraints
        Ok(String::new())
    }
    
    fn extract_associated_types(&self, _block: &Block) -> Result<String> {
        // Extract associated types for traits
        Ok(String::new())
    }
    
    fn extract_module_content(&self, block: &Block) -> Result<String> {
        if let Some(content) = block.abstract_syntax.get("content") {
            if let Some(content_str) = content.as_str() {
                return Ok(content_str.to_string());
            }
        }
        Ok("    // TODO: Define module content".to_string())
    }
    
    // Control flow helper methods
    
    fn extract_condition(&self, block: &Block, _metadata: &serde_json::Map<String, Value>) -> Result<String> {
        if let Some(condition) = block.abstract_syntax.get("condition") {
            if let Some(condition_str) = condition.as_str() {
                return Ok(condition_str.to_string());
            }
        }
        Ok("true".to_string())
    }
    
    fn extract_then_body(&self, block: &Block) -> Result<String> {
        if let Some(then_body) = block.abstract_syntax.get("then_body") {
            if let Some(body_str) = then_body.as_str() {
                return Ok(format!("    {}", body_str));
            }
        }
        Ok("    // TODO: Implement then branch".to_string())
    }
    
    fn extract_else_clause(&self, block: &Block) -> Result<String> {
        if let Some(else_body) = block.abstract_syntax.get("else_body") {
            if let Some(body_str) = else_body.as_str() {
                return Ok(format!(" else {{\n    {}\n}}", body_str));
            }
        }
        Ok(String::new())
    }
    
    fn extract_for_initialization(&self, block: &Block, _metadata: &serde_json::Map<String, Value>) -> Result<String> {
        if let Some(init) = block.abstract_syntax.get("initialization") {
            if let Some(init_str) = init.as_str() {
                return Ok(init_str.to_string());
            }
        }
        Ok("int i = 0".to_string())
    }
    
    fn extract_for_increment(&self, block: &Block, _metadata: &serde_json::Map<String, Value>) -> Result<String> {
        if let Some(increment) = block.abstract_syntax.get("increment") {
            if let Some(increment_str) = increment.as_str() {
                return Ok(increment_str.to_string());
            }
        }
        Ok("i++".to_string())
    }
    
    fn extract_for_pattern(&self, block: &Block) -> Result<String> {
        if let Some(pattern) = block.abstract_syntax.get("pattern") {
            if let Some(pattern_str) = pattern.as_str() {
                return Ok(pattern_str.to_string());
            }
        }
        Ok("item".to_string())
    }
    
    fn extract_for_iterator(&self, block: &Block) -> Result<String> {
        if let Some(iterator) = block.abstract_syntax.get("iterator") {
            if let Some(iterator_str) = iterator.as_str() {
                return Ok(iterator_str.to_string());
            }
        }
        Ok("items.iter()".to_string())
    }
    
    fn extract_for_target(&self, block: &Block) -> Result<String> {
        if let Some(target) = block.abstract_syntax.get("target") {
            if let Some(target_str) = target.as_str() {
                return Ok(target_str.to_string());
            }
        }
        Ok("item".to_string())
    }
    
    fn extract_for_iterable(&self, block: &Block) -> Result<String> {
        if let Some(iterable) = block.abstract_syntax.get("iterable") {
            if let Some(iterable_str) = iterable.as_str() {
                return Ok(iterable_str.to_string());
            }
        }
        Ok("items".to_string())
    }
    
    fn extract_for_variable(&self, block: &Block) -> Result<String> {
        if let Some(variable) = block.abstract_syntax.get("variable") {
            if let Some(variable_str) = variable.as_str() {
                return Ok(variable_str.to_string());
            }
        }
        Ok("item".to_string())
    }
    
    fn extract_try_body(&self, block: &Block) -> Result<String> {
        if let Some(try_body) = block.abstract_syntax.get("try_body") {
            if let Some(body_str) = try_body.as_str() {
                return Ok(format!("    {}", body_str));
            }
        }
        Ok("    // TODO: Implement try block".to_string())
    }
    
    fn extract_catch_body(&self, block: &Block) -> Result<String> {
        if let Some(catch_body) = block.abstract_syntax.get("catch_body") {
            if let Some(body_str) = catch_body.as_str() {
                return Ok(format!("    {}", body_str));
            }
        }
        Ok("    // TODO: Handle exception".to_string())
    }
    
    fn extract_exception_var(&self, block: &Block, _metadata: &serde_json::Map<String, Value>) -> Result<String> {
        if let Some(exception_var) = block.abstract_syntax.get("exception_var") {
            if let Some(var_str) = exception_var.as_str() {
                return Ok(var_str.to_string());
            }
        }
        Ok("e".to_string())
    }
    
    fn extract_exception_types(&self, block: &Block, _metadata: &serde_json::Map<String, Value>) -> Result<String> {
        if let Some(exception_types) = block.abstract_syntax.get("exception_types") {
            if let Some(types_str) = exception_types.as_str() {
                return Ok(types_str.to_string());
            }
        }
        Ok("Exception".to_string())
    }
    
    fn extract_finally_clause(&self, block: &Block) -> Result<String> {
        if let Some(finally_body) = block.abstract_syntax.get("finally_body") {
            if let Some(body_str) = finally_body.as_str() {
                return Ok(format!("\nfinally {{\n    {}\n}}", body_str));
            }
        }
        Ok(String::new())
    }
    
    fn extract_catch_blocks(&self, block: &Block) -> Result<String> {
        if let Some(catch_blocks) = block.abstract_syntax.get("catch_blocks") {
            if let Some(blocks_str) = catch_blocks.as_str() {
                return Ok(blocks_str.to_string());
            }
        }
        Ok("catch (Exception e) {\n    // Handle exception\n}".to_string())
    }
    
    fn extract_try_expression(&self, block: &Block) -> Result<String> {
        if let Some(expression) = block.abstract_syntax.get("expression") {
            if let Some(expr_str) = expression.as_str() {
                return Ok(expr_str.to_string());
            }
        }
        Ok("some_operation()".to_string())
    }
    
    fn extract_switch_expression(&self, block: &Block, _metadata: &serde_json::Map<String, Value>) -> Result<String> {
        if let Some(expression) = block.abstract_syntax.get("expression") {
            if let Some(expr_str) = expression.as_str() {
                return Ok(expr_str.to_string());
            }
        }
        Ok("value".to_string())
    }
    
    fn extract_switch_cases(&self, block: &Block) -> Result<String> {
        if let Some(cases) = block.abstract_syntax.get("cases") {
            if let Some(cases_array) = cases.as_array() {
                let case_strings: Vec<String> = cases_array.iter()
                    .filter_map(|c| c.as_str().map(|s| format!("    {}", s)))
                    .collect();
                return Ok(case_strings.join("\n"));
            }
        }
        Ok("    default:\n        break;".to_string())
    }
    
    fn extract_generic_type_params(&self, block: &Block, _metadata: &serde_json::Map<String, Value>) -> Result<String> {
        if let Some(features) = &block.language_features {
            if let Some(generics) = features.get("generics") {
                if let Some(generic_params) = generics.as_array() {
                    let params: Vec<String> = generic_params.iter()
                        .filter_map(|g| g.as_str().map(|s| s.to_string()))
                        .collect();
                    return Ok(params.join(", "));
                }
            }
        }
        Ok("T".to_string())
    }
    
    fn extract_decorator_args(&self, block: &Block, _metadata: &serde_json::Map<String, Value>) -> Result<String> {
        if let Some(args) = block.abstract_syntax.get("arguments") {
            if let Some(args_str) = args.as_str() {
                return Ok(format!("({})", args_str));
            }
        }
        Ok(String::new())
    }
    
    fn extract_outer_params(&self, block: &Block, _metadata: &serde_json::Map<String, Value>) -> Result<String> {
        if let Some(outer_params) = block.abstract_syntax.get("outer_params") {
            if let Some(params_str) = outer_params.as_str() {
                return Ok(params_str.to_string());
            }
        }
        Ok("x".to_string())
    }
    
    fn extract_inner_params(&self, block: &Block, _metadata: &serde_json::Map<String, Value>) -> Result<String> {
        if let Some(inner_params) = block.abstract_syntax.get("inner_params") {
            if let Some(params_str) = inner_params.as_str() {
                return Ok(params_str.to_string());
            }
        }
        Ok("y".to_string())
    }
    
    fn extract_closure_capture(&self, block: &Block) -> Result<String> {
        if let Some(capture) = block.abstract_syntax.get("capture") {
            if let Some(capture_str) = capture.as_str() {
                return Ok(capture_str.to_string());
            }
        }
        Ok("".to_string())
    }
    
    fn extract_inner_closure_capture(&self, block: &Block) -> Result<String> {
        if let Some(capture) = block.abstract_syntax.get("inner_capture") {
            if let Some(capture_str) = capture.as_str() {
                return Ok(capture_str.to_string());
            }
        }
        Ok("".to_string())
    }
    
    fn extract_move_keyword(&self, block: &Block) -> Result<String> {
        if let Some(modifiers) = &block.modifiers {
            if modifiers.contains(&"move".to_string()) {
                return Ok("move ".to_string());
            }
        }
        Ok(String::new())
    }
    
    fn extract_use_vars(&self, block: &Block) -> Result<String> {
        if let Some(use_vars) = block.abstract_syntax.get("use_vars") {
            if let Some(vars_str) = use_vars.as_str() {
                return Ok(vars_str.to_string());
            }
        }
        Ok("$var".to_string())
    }
}
