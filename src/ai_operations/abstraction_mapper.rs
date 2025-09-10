use anyhow::Result;
use std::collections::HashMap;
use crate::ai_operations::{AbstractBlockSpec, PatternLibrary, LanguageGenerator, code_generators::CodeGenerator};

pub struct AbstractionMapper {
    pattern_library: PatternLibrary,
    language_generators: HashMap<String, Box<dyn LanguageGenerator>>,
    code_generator: CodeGenerator,
}

impl AbstractionMapper {
    pub fn new() -> Self {
        Self {
            pattern_library: PatternLibrary::new(),
            language_generators: HashMap::new(),
            code_generator: CodeGenerator::new(),
        }
    }

    pub fn map_abstraction_to_code(
        &self,
        abstraction: &AbstractBlockSpec,
        target_language: &str,
    ) -> Result<String> {
        // 1. Select appropriate design pattern
        let pattern = self.pattern_library.select_pattern(abstraction)?;
        
        // 2. Get language-specific generator
        let generator = self.language_generators
            .get(target_language)
            .ok_or_else(|| anyhow::anyhow!("Unsupported language: {}", target_language))?;
        
        // 3. Generate concrete implementation
        let code = generator.generate_from_pattern(&pattern, abstraction)?;
        
        // 4. Apply constraints and optimizations
        let optimized = self.apply_constraints(code, &abstraction.invariants)?;
        
        Ok(optimized)
    }

    pub fn map_multiple_abstractions(
        &self,
        abstractions: &[AbstractBlockSpec],
        target_language: &str,
    ) -> Result<HashMap<String, String>> {
        let mut results = HashMap::new();
        
        for abstraction in abstractions {
            let code = self.map_abstraction_to_code(abstraction, target_language)?;
            let filename = format!("{}.{}", 
                abstraction.semantic_name.to_lowercase(),
                self.get_file_extension(target_language)?
            );
            results.insert(filename, code);
        }
        
        Ok(results)
    }

    pub fn create_module_structure(
        &self,
        abstractions: &[AbstractBlockSpec],
        target_language: &str,
        module_name: &str,
    ) -> Result<ModuleStructure> {
        let mut files = HashMap::new();
        let dependencies = Vec::new();
        let mut exports = Vec::new();

        // Generate individual files
        for abstraction in abstractions {
            let code = self.map_abstraction_to_code(abstraction, target_language)?;
            let filename = format!("{}.{}", 
                abstraction.semantic_name.to_lowercase(),
                self.get_file_extension(target_language)?
            );
            files.insert(filename.clone(), code);
            exports.push(abstraction.semantic_name.clone());
        }

        // Create module index/init file
        let index_file = self.create_module_index(target_language, module_name, &exports)?;
        let index_filename = match target_language {
            "python" => "__init__.py".to_string(),
            "typescript" => "index.ts".to_string(),
            "rust" => "mod.rs".to_string(),
            _ => "index.txt".to_string(),
        };
        files.insert(index_filename, index_file);

        Ok(ModuleStructure {
            name: module_name.to_string(),
            files,
            dependencies,
            exports,
        })
    }

    fn apply_constraints(
        &self,
        mut code: String,
        constraints: &[crate::ai_operations::Invariant],
    ) -> Result<String> {
        for constraint in constraints {
            code = match constraint.name.as_str() {
                "thread_safe" => self.apply_thread_safety(&code)?,
                "immutable" => self.apply_immutability(&code)?,
                "async_compatible" => self.apply_async_compatibility(&code)?,
                "error_handling" => self.apply_error_handling(&code)?,
                _ => code, // Unknown constraint, skip
            };
        }
        
        Ok(code)
    }

    fn apply_thread_safety(&self, code: &str) -> Result<String> {
        // Add thread safety annotations/imports
        if code.contains("class ") && code.contains("python") {
            Ok(format!("import threading\n\n{}", code))
        } else {
            Ok(code.to_string())
        }
    }

    fn apply_immutability(&self, code: &str) -> Result<String> {
        // Apply immutability patterns
        let mut result = code.to_string();
        
        // For Python, add @dataclass(frozen=True) if it's a class
        if result.contains("class ") && !result.contains("@dataclass") {
            result = result.replace("class ", "@dataclass(frozen=True)\nclass ");
            result = format!("from dataclasses import dataclass\n\n{}", result);
        }
        
        Ok(result)
    }

    fn apply_async_compatibility(&self, code: &str) -> Result<String> {
        // Ensure async compatibility
        let mut result = code.to_string();
        
        if result.contains("def ") && !result.contains("async def") {
            // Convert to async if needed
            result = result.replace("def ", "async def ");
            if !result.contains("import asyncio") {
                result = format!("import asyncio\n\n{}", result);
            }
        }
        
        Ok(result)
    }

    fn apply_error_handling(&self, code: &str) -> Result<String> {
        // Add comprehensive error handling
        let mut result = code.to_string();
        
        // Add try-catch blocks around function bodies
        if result.contains("def ") && !result.contains("try:") {
            // This is a simplified implementation
            // In practice, you'd want more sophisticated AST manipulation
            result = result.replace("pass", "try:\n        pass\n    except Exception as e:\n        raise");
        }
        
        Ok(result)
    }

    fn get_file_extension(&self, language: &str) -> Result<&str> {
        match language {
            "python" => Ok("py"),
            "typescript" => Ok("ts"),
            "javascript" => Ok("js"),
            "rust" => Ok("rs"),
            "java" => Ok("java"),
            "go" => Ok("go"),
            _ => Err(anyhow::anyhow!("Unknown language: {}", language)),
        }
    }

    fn create_module_index(
        &self,
        language: &str,
        module_name: &str,
        exports: &[String],
    ) -> Result<String> {
        match language {
            "python" => {
                let mut content = format!("\"\"\"{}\"\"\"", module_name);
                content.push_str("\n\n");
                for export in exports {
                    content.push_str(&format!("from .{} import {}\n", 
                        export.to_lowercase(), export));
                }
                content.push_str("\n__all__ = [\n");
                for export in exports {
                    content.push_str(&format!("    \"{}\",\n", export));
                }
                content.push_str("]\n");
                Ok(content)
            }
            "typescript" => {
                let mut content = format!("// {}\n\n", module_name);
                for export in exports {
                    content.push_str(&format!("export {{ {} }} from './{}';\n", 
                        export, export.to_lowercase()));
                }
                Ok(content)
            }
            "rust" => {
                let mut content = format!("//! {}\n\n", module_name);
                for export in exports {
                    content.push_str(&format!("pub mod {};\n", export.to_lowercase()));
                }
                content.push_str("\n");
                for export in exports {
                    content.push_str(&format!("pub use {}::*;\n", export.to_lowercase()));
                }
                Ok(content)
            }
            _ => Ok(format!("// Module: {}\n", module_name)),
        }
    }

    pub fn add_language_generator(&mut self, language: String, generator: Box<dyn LanguageGenerator>) {
        self.language_generators.insert(language, generator);
    }

    pub fn supported_languages(&self) -> Vec<&str> {
        self.language_generators.keys().map(|s| s.as_str()).collect()
    }

    pub fn analyze_abstraction_complexity(&self, abstraction: &AbstractBlockSpec) -> ComplexityAnalysis {
        let mut score = 0;
        let mut factors = Vec::new();

        // Analyze parameters
        if abstraction.properties.parameters.len() > 5 {
            score += 2;
            factors.push("High parameter count".to_string());
        }

        // Analyze behaviors
        if abstraction.behaviors.len() > 10 {
            score += 3;
            factors.push("Many behaviors".to_string());
        }

        // Analyze invariants
        if abstraction.invariants.len() > 3 {
            score += 2;
            factors.push("Complex constraints".to_string());
        }

        // Check for async requirements
        if abstraction.properties.is_async {
            score += 1;
            factors.push("Async complexity".to_string());
        }

        let level = match score {
            0..=2 => ComplexityLevel::Simple,
            3..=5 => ComplexityLevel::Moderate,
            6..=8 => ComplexityLevel::Complex,
            _ => ComplexityLevel::VeryComplex,
        };

        ComplexityAnalysis {
            level: level.clone(),
            score,
            factors: factors.clone(),
            recommendations: self.generate_complexity_recommendations(&level, &factors),
        }
    }

    fn generate_complexity_recommendations(&self, level: &ComplexityLevel, factors: &[String]) -> Vec<String> {
        let mut recommendations = Vec::new();

        match level {
            ComplexityLevel::Simple => {
                recommendations.push("Consider adding more comprehensive documentation".to_string());
            }
            ComplexityLevel::Moderate => {
                recommendations.push("Consider breaking into smaller components".to_string());
                recommendations.push("Add comprehensive unit tests".to_string());
            }
            ComplexityLevel::Complex => {
                recommendations.push("Strongly recommend decomposition into smaller blocks".to_string());
                recommendations.push("Implement comprehensive error handling".to_string());
                recommendations.push("Add integration tests".to_string());
            }
            ComplexityLevel::VeryComplex => {
                recommendations.push("CRITICAL: This abstraction is too complex".to_string());
                recommendations.push("Must be decomposed into multiple simpler abstractions".to_string());
                recommendations.push("Consider architectural patterns (facade, strategy, etc.)".to_string());
            }
        }

        // Add factor-specific recommendations
        for factor in factors {
            if factor.contains("parameter") {
                recommendations.push("Consider using configuration objects instead of many parameters".to_string());
            }
            if factor.contains("behaviors") {
                recommendations.push("Group related behaviors into separate interfaces".to_string());
            }
        }

        recommendations
    }
}

impl Default for AbstractionMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ModuleStructure {
    pub name: String,
    pub files: HashMap<String, String>,
    pub dependencies: Vec<String>,
    pub exports: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ComplexityAnalysis {
    pub level: ComplexityLevel,
    pub score: u32,
    pub factors: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComplexityLevel {
    Simple,
    Moderate,
    Complex,
    VeryComplex,
}
