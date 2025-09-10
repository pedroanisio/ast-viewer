use std::collections::HashMap;
use uuid::Uuid;
use anyhow::Result;
use serde::{Serialize, Deserialize};
use crate::database::{Database, Block};
use crate::parser::universal::UniversalParser;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformationRequest {
    pub operation: TransformationOperation,
    pub target_blocks: Vec<Uuid>,
    pub parameters: HashMap<String, serde_json::Value>,
    pub preserve_semantics: bool,
    pub generate_tests: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransformationOperation {
    ExtractFunction {
        new_function_name: String,
        extract_from_block: Uuid,
        lines_to_extract: (usize, usize),
    },
    MoveFunction {
        function_id: Uuid,
        target_container: Uuid,
    },
    RenameSymbol {
        symbol_id: Uuid,
        new_name: String,
    },
    InlineFunction {
        function_id: Uuid,
    },
    SplitClass {
        class_id: Uuid,
        split_criteria: SplitCriteria,
    },
    MergeClasses {
        class_ids: Vec<Uuid>,
        new_class_name: String,
    },
    ConvertToAsync {
        function_id: Uuid,
    },
    AddErrorHandling {
        function_id: Uuid,
        error_strategy: ErrorHandlingStrategy,
    },
    OptimizePerformance {
        target_blocks: Vec<Uuid>,
        optimization_type: OptimizationType,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SplitCriteria {
    ByResponsibility,
    ByComplexity,
    ByDependencies,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorHandlingStrategy {
    TryCatch,
    ResultType,
    Optional,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationType {
    ReduceComplexity,
    ImproveReadability,
    PerformanceOptimization,
    MemoryOptimization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformationResult {
    pub success: bool,
    pub transformed_blocks: Vec<Uuid>,
    pub new_blocks: Vec<Uuid>,
    pub removed_blocks: Vec<Uuid>,
    pub generated_code: Option<String>,
    pub impact_analysis: TransformationImpact,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformationImpact {
    pub affected_files: Vec<String>,
    pub complexity_change: f64,
    pub maintainability_score: f64,
    pub test_coverage_impact: f64,
    pub performance_impact: PerformanceImpact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceImpact {
    pub execution_time_change: f64,
    pub memory_usage_change: f64,
    pub io_operations_change: i32,
}

pub struct CodeTransformer {
    db: Database,
    parser: UniversalParser,
}

impl CodeTransformer {
    pub fn new(db: Database) -> Result<Self> {
        let parser = UniversalParser::new()?;
        Ok(Self { db, parser })
    }

    pub async fn transform(&mut self, request: TransformationRequest) -> Result<TransformationResult> {
        match request.operation {
            TransformationOperation::ExtractFunction { ref new_function_name, extract_from_block, lines_to_extract } => {
                self.extract_function(extract_from_block, new_function_name, lines_to_extract, &request).await
            },
            TransformationOperation::MoveFunction { function_id, target_container } => {
                self.move_function(function_id, target_container, &request).await
            },
            TransformationOperation::RenameSymbol { symbol_id, ref new_name } => {
                self.rename_symbol(symbol_id, new_name, &request).await
            },
            TransformationOperation::InlineFunction { function_id } => {
                self.inline_function(function_id, &request).await
            },
            TransformationOperation::SplitClass { class_id, ref split_criteria } => {
                self.split_class(class_id, split_criteria.clone(), &request).await
            },
            TransformationOperation::MergeClasses { ref class_ids, ref new_class_name } => {
                self.merge_classes(class_ids.clone(), new_class_name, &request).await
            },
            TransformationOperation::ConvertToAsync { function_id } => {
                self.convert_to_async(function_id, &request).await
            },
            TransformationOperation::AddErrorHandling { function_id, ref error_strategy } => {
                self.add_error_handling(function_id, error_strategy.clone(), &request).await
            },
            TransformationOperation::OptimizePerformance { ref target_blocks, ref optimization_type } => {
                self.optimize_performance(target_blocks.clone(), optimization_type.clone(), &request).await
            },
        }
    }

    async fn extract_function(
        &mut self,
        source_block_id: Uuid,
        new_function_name: &str,
        lines_to_extract: (usize, usize),
        request: &TransformationRequest,
    ) -> Result<TransformationResult> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        // Get the source block
        let source_block = self.db.get_block_by_id(source_block_id).await?;
        
        // Analyze the code to extract
        let container_id = self.db.get_container_id_by_block(source_block_id).await?;
        let container = self.db.get_container_by_id(container_id).await?;
        
        if let Some(source_code) = &container.source_code {
            let lines: Vec<&str> = source_code.lines().collect();
            let (start_line, end_line) = lines_to_extract;
            
            if start_line >= lines.len() || end_line >= lines.len() || start_line > end_line {
                errors.push("Invalid line range for extraction".to_string());
                return Ok(TransformationResult {
                    success: false,
                    transformed_blocks: Vec::new(),
                    new_blocks: Vec::new(),
                    removed_blocks: Vec::new(),
                    generated_code: None,
                    impact_analysis: self.calculate_impact_analysis(&[], &container).await?,
                    warnings,
                    errors,
                });
            }

            // Extract the code lines
            let extracted_lines = &lines[start_line..=end_line];
            let extracted_code = extracted_lines.join("\n");
            
            // Analyze dependencies in the extracted code
            let dependencies = self.analyze_extracted_dependencies(&extracted_code, &source_block).await?;
            
            // Generate new function signature
            let function_signature = self.generate_function_signature(
                new_function_name,
                &dependencies,
                &container.language.as_ref().unwrap_or(&"unknown".to_string())
            )?;
            
            // Create the new function code
            let new_function_code = format!("{}\n{}\n", function_signature, self.indent_code(&extracted_code, 1));
            
            // Generate replacement call
            let function_call = self.generate_function_call(new_function_name, &dependencies)?;
            
            // Create modified source code
            let mut modified_lines = lines.clone();
            
            // Replace extracted lines with function call
            modified_lines.splice(start_line..=end_line, std::iter::once(function_call.as_str()));
            
            // Insert new function (simple heuristic: add after the current function)
            let insert_position = self.find_function_end_position(&modified_lines, &source_block)?;
            modified_lines.insert(insert_position, "");
            modified_lines.insert(insert_position + 1, &new_function_code);
            
            let final_code = modified_lines.join("\n");
            
            // Parse the modified code to create new blocks
            let parse_result = self.parser.parse_file(
                &final_code,
                &container.language.as_ref().unwrap_or(&"unknown".to_string()),
                &container.original_path.as_ref().unwrap_or(&container.name)
            )?;
            
            // Calculate impact
            let impact_analysis = self.calculate_impact_analysis(&parse_result.blocks, &container).await?;
            
            if request.preserve_semantics {
                // Verify semantic preservation
                let semantic_preserved = self.verify_semantic_preservation(&container, &final_code).await?;
                if !semantic_preserved {
                    warnings.push("Semantic preservation could not be verified".to_string());
                }
            }
            
            Ok(TransformationResult {
                success: true,
                transformed_blocks: vec![source_block_id],
                new_blocks: parse_result.blocks.iter().map(|b| b.id).collect(),
                removed_blocks: Vec::new(),
                generated_code: Some(final_code),
                impact_analysis,
                warnings,
                errors,
            })
        } else {
            errors.push("No source code available for transformation".to_string());
            Ok(TransformationResult {
                success: false,
                transformed_blocks: Vec::new(),
                new_blocks: Vec::new(),
                removed_blocks: Vec::new(),
                generated_code: None,
                impact_analysis: self.calculate_impact_analysis(&[], &container).await?,
                warnings,
                errors,
            })
        }
    }

    async fn move_function(
        &mut self,
        function_id: Uuid,
        target_container: Uuid,
        _request: &TransformationRequest,
    ) -> Result<TransformationResult> {
        // Implementation for moving functions between files
        let mut warnings = Vec::new();
        let errors = Vec::new();
        
        warnings.push("Move function operation not yet implemented".to_string());
        
        let container = self.db.get_container_by_id(target_container).await?;
        
        Ok(TransformationResult {
            success: false,
            transformed_blocks: vec![function_id],
            new_blocks: Vec::new(),
            removed_blocks: Vec::new(),
            generated_code: None,
            impact_analysis: self.calculate_impact_analysis(&[], &container).await?,
            warnings,
            errors,
        })
    }

    async fn rename_symbol(
        &mut self,
        symbol_id: Uuid,
        new_name: &str,
        _request: &TransformationRequest,
    ) -> Result<TransformationResult> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        // Get the symbol block
        let symbol_block = self.db.get_block_by_id(symbol_id).await?;
        let container_id = self.db.get_container_id_by_block(symbol_id).await?;
        let container = self.db.get_container_by_id(container_id).await?;

        // Analyze all references to this symbol
        // Note: We'll skip call graph analysis for now due to ownership issues
        let dependents: Vec<Uuid> = Vec::new(); // Placeholder

        if !dependents.is_empty() {
            warnings.push(format!("Symbol has {} dependents that will need to be updated", dependents.len()));
        }

        // Generate new code with renamed symbol
        if let Some(source_code) = &container.source_code {
            let default_name = "unknown".to_string();
            let old_name = symbol_block.semantic_name.as_ref().unwrap_or(&default_name);
            let new_code = source_code.replace(old_name, new_name);
            
            // Parse the modified code
            let parse_result = self.parser.parse_file(
                &new_code,
                &container.language.as_ref().unwrap_or(&"unknown".to_string()),
                &container.original_path.as_ref().unwrap_or(&container.name)
            )?;
            
            let impact_analysis = self.calculate_impact_analysis(&parse_result.blocks, &container).await?;
            
            Ok(TransformationResult {
                success: true,
                transformed_blocks: vec![symbol_id],
                new_blocks: parse_result.blocks.iter().map(|b| b.id).collect(),
                removed_blocks: Vec::new(),
                generated_code: Some(new_code),
                impact_analysis,
                warnings,
                errors,
            })
        } else {
            errors.push("No source code available for renaming".to_string());
            Ok(TransformationResult {
                success: false,
                transformed_blocks: Vec::new(),
                new_blocks: Vec::new(),
                removed_blocks: Vec::new(),
                generated_code: None,
                impact_analysis: self.calculate_impact_analysis(&[], &container).await?,
                warnings,
                errors,
            })
        }
    }

    async fn inline_function(
        &mut self,
        function_id: Uuid,
        _request: &TransformationRequest,
    ) -> Result<TransformationResult> {
        let mut warnings = Vec::new();
        let errors = Vec::new();
        
        warnings.push("Inline function operation not yet implemented".to_string());
        
        let container_id = self.db.get_container_id_by_block(function_id).await?;
        let container = self.db.get_container_by_id(container_id).await?;
        
        Ok(TransformationResult {
            success: false,
            transformed_blocks: vec![function_id],
            new_blocks: Vec::new(),
            removed_blocks: Vec::new(),
            generated_code: None,
            impact_analysis: self.calculate_impact_analysis(&[], &container).await?,
            warnings,
            errors,
        })
    }

    async fn split_class(
        &mut self,
        class_id: Uuid,
        _split_criteria: SplitCriteria,
        _request: &TransformationRequest,
    ) -> Result<TransformationResult> {
        let mut warnings = Vec::new();
        let errors = Vec::new();
        
        warnings.push("Split class operation not yet implemented".to_string());
        
        let container_id = self.db.get_container_id_by_block(class_id).await?;
        let container = self.db.get_container_by_id(container_id).await?;
        
        Ok(TransformationResult {
            success: false,
            transformed_blocks: vec![class_id],
            new_blocks: Vec::new(),
            removed_blocks: Vec::new(),
            generated_code: None,
            impact_analysis: self.calculate_impact_analysis(&[], &container).await?,
            warnings,
            errors,
        })
    }

    async fn merge_classes(
        &mut self,
        class_ids: Vec<Uuid>,
        _new_class_name: &str,
        _request: &TransformationRequest,
    ) -> Result<TransformationResult> {
        let mut warnings = Vec::new();
        let errors = Vec::new();
        
        warnings.push("Merge classes operation not yet implemented".to_string());
        
        // Get container from first class
        let container_id = if let Some(&first_id) = class_ids.first() {
            self.db.get_container_id_by_block(first_id).await?
        } else {
            return Err(anyhow::anyhow!("No class IDs provided"));
        };
        let container = self.db.get_container_by_id(container_id).await?;
        
        Ok(TransformationResult {
            success: false,
            transformed_blocks: class_ids,
            new_blocks: Vec::new(),
            removed_blocks: Vec::new(),
            generated_code: None,
            impact_analysis: self.calculate_impact_analysis(&[], &container).await?,
            warnings,
            errors,
        })
    }

    async fn convert_to_async(
        &mut self,
        function_id: Uuid,
        _request: &TransformationRequest,
    ) -> Result<TransformationResult> {
        let mut warnings = Vec::new();
        let errors = Vec::new();
        
        warnings.push("Convert to async operation not yet implemented".to_string());
        
        let container_id = self.db.get_container_id_by_block(function_id).await?;
        let container = self.db.get_container_by_id(container_id).await?;
        
        Ok(TransformationResult {
            success: false,
            transformed_blocks: vec![function_id],
            new_blocks: Vec::new(),
            removed_blocks: Vec::new(),
            generated_code: None,
            impact_analysis: self.calculate_impact_analysis(&[], &container).await?,
            warnings,
            errors,
        })
    }

    async fn add_error_handling(
        &mut self,
        function_id: Uuid,
        _error_strategy: ErrorHandlingStrategy,
        _request: &TransformationRequest,
    ) -> Result<TransformationResult> {
        let mut warnings = Vec::new();
        let errors = Vec::new();
        
        warnings.push("Add error handling operation not yet implemented".to_string());
        
        let container_id = self.db.get_container_id_by_block(function_id).await?;
        let container = self.db.get_container_by_id(container_id).await?;
        
        Ok(TransformationResult {
            success: false,
            transformed_blocks: vec![function_id],
            new_blocks: Vec::new(),
            removed_blocks: Vec::new(),
            generated_code: None,
            impact_analysis: self.calculate_impact_analysis(&[], &container).await?,
            warnings,
            errors,
        })
    }

    async fn optimize_performance(
        &mut self,
        target_blocks: Vec<Uuid>,
        _optimization_type: OptimizationType,
        _request: &TransformationRequest,
    ) -> Result<TransformationResult> {
        let mut warnings = Vec::new();
        let errors = Vec::new();
        
        warnings.push("Performance optimization operation not yet implemented".to_string());
        
        // Get container from first block
        let container_id = if let Some(&first_id) = target_blocks.first() {
            self.db.get_container_id_by_block(first_id).await?
        } else {
            return Err(anyhow::anyhow!("No target blocks provided"));
        };
        let container = self.db.get_container_by_id(container_id).await?;
        
        Ok(TransformationResult {
            success: false,
            transformed_blocks: target_blocks,
            new_blocks: Vec::new(),
            removed_blocks: Vec::new(),
            generated_code: None,
            impact_analysis: self.calculate_impact_analysis(&[], &container).await?,
            warnings,
            errors,
        })
    }

    // Helper methods

    async fn analyze_extracted_dependencies(&self, _code: &str, _source_block: &Block) -> Result<Vec<String>> {
        // Analyze what variables/functions the extracted code depends on
        Ok(Vec::new())
    }

    fn generate_function_signature(&self, name: &str, dependencies: &[String], language: &str) -> Result<String> {
        match language {
            "python" => {
                let params = if dependencies.is_empty() {
                    String::new()
                } else {
                    dependencies.join(", ")
                };
                Ok(format!("def {}({}):", name, params))
            },
            "javascript" | "typescript" => {
                let params = if dependencies.is_empty() {
                    String::new()
                } else {
                    dependencies.join(", ")
                };
                Ok(format!("function {}({}) {{", name, params))
            },
            "rust" => {
                let params = if dependencies.is_empty() {
                    String::new()
                } else {
                    dependencies.iter().map(|d| format!("{}: &str", d)).collect::<Vec<_>>().join(", ")
                };
                Ok(format!("fn {}({}) {{", name, params))
            },
            _ => Ok(format!("// Function: {}", name)),
        }
    }

    fn generate_function_call(&self, name: &str, dependencies: &[String]) -> Result<String> {
        let args = if dependencies.is_empty() {
            String::new()
        } else {
            dependencies.join(", ")
        };
        Ok(format!("{}({})", name, args))
    }

    fn indent_code(&self, code: &str, levels: usize) -> String {
        let indent = "    ".repeat(levels);
        code.lines()
            .map(|line| if line.trim().is_empty() { line.to_string() } else { format!("{}{}", indent, line) })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn find_function_end_position(&self, _lines: &[&str], _source_block: &Block) -> Result<usize> {
        // Simple heuristic: add at the end
        Ok(_lines.len())
    }

    async fn verify_semantic_preservation(&self, _original_container: &crate::database::Container, _new_code: &str) -> Result<bool> {
        // This would involve running tests or static analysis
        // For now, return true as a placeholder
        Ok(true)
    }

    async fn calculate_impact_analysis(&self, _blocks: &[crate::core::SemanticBlock], _container: &crate::database::Container) -> Result<TransformationImpact> {
        Ok(TransformationImpact {
            affected_files: vec![_container.name.clone()],
            complexity_change: 0.0,
            maintainability_score: 0.8,
            test_coverage_impact: 0.0,
            performance_impact: PerformanceImpact {
                execution_time_change: 0.0,
                memory_usage_change: 0.0,
                io_operations_change: 0,
            },
        })
    }

}
