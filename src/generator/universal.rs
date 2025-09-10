use anyhow::Result;
use crate::database::{Database, Container, Block};
use super::templates::TemplateEngine;
use super::validation::{ReconstructionValidator, ValidationResult};

#[derive(Debug, Clone)]
pub struct GenerationConfig {
    pub output_dir: std::path::PathBuf,
    #[allow(dead_code)]
    pub format_code: bool,
    #[allow(dead_code)]
    pub group_imports: bool,
    #[allow(dead_code)]
    pub add_markers: bool,
    #[allow(dead_code)]
    pub validate_output: bool,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            output_dir: std::path::PathBuf::from("generated"),
            format_code: true,
            group_imports: true,
            add_markers: true,
            validate_output: true,
        }
    }
}

#[allow(dead_code)]
pub struct UniversalGenerator {
    template_engine: TemplateEngine,
    validator: ReconstructionValidator,
}

#[allow(dead_code)]
impl UniversalGenerator {
    pub fn new() -> Self {
        Self {
            template_engine: TemplateEngine::new(),
            validator: ReconstructionValidator::new(),
        }
    }

    pub async fn generate_repository(
        &self,
        db: &Database,
        migration_id: uuid::Uuid,
        config: &GenerationConfig,
    ) -> Result<GenerationResult> {
        // Get all containers for this migration
        let containers = db.get_containers_by_migration(migration_id).await?;
        
        let mut results = Vec::new();
        let mut total_blocks = 0;
        let mut total_files = 0;
        let mut validation_results = Vec::new();
        
        for container in containers {
            // Get blocks for this container
            let blocks = db.get_blocks_by_container(container.id).await?;
            total_blocks += blocks.len();
            
            if blocks.is_empty() {
                continue;
            }
            
            // Determine language from container metadata
            let language = self.determine_language(&container, &blocks)?;
            
            // Generate file content
            let generated_content = self.template_engine.render_file(&container, &blocks, &language)?;
            
            // Create output file path
            let original_path = container.original_path.as_ref().unwrap_or(&container.name);
            let output_path = config.output_dir.join(original_path);
            
            // Ensure output directory exists
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            
            // Write generated content
            std::fs::write(&output_path, &generated_content)?;
            total_files += 1;
            
            // Validate if requested
            if config.validate_output {
                let original_content = container.source_code.as_ref().unwrap_or(&String::new()).clone();
                let validation = self.validator.validate_reconstruction(
                    &original_content,
                    &generated_content,
                    &language,
                )?;
                validation_results.push(validation);
            }
            
            results.push(FileGenerationResult {
                original_path: container.original_path.clone().unwrap_or(container.name.clone()),
                generated_path: output_path,
                language,
                block_count: blocks.len(),
                content_length: generated_content.len(),
            });
        }
        
        // Calculate overall metrics
        let overall_validation = if !validation_results.is_empty() {
            self.aggregate_validation_results(&validation_results)
        } else {
            ValidationResult {
                is_valid: true,
                errors: Vec::new(),
                warnings: Vec::new(),
                metrics: super::validation::ValidationMetrics {
                    syntax_valid: true,
                    semantic_coverage: 1.0,
                    reconstruction_fidelity: 1.0,
                    block_count: total_blocks,
                    file_count: total_files,
                },
            }
        };
        
        Ok(GenerationResult {
            migration_id,
            total_files,
            total_blocks,
            file_results: results,
            validation: overall_validation,
        })
    }
    
    fn determine_language(&self, container: &Container, blocks: &[Block]) -> Result<String> {
        // Try to determine language from container language field first
        if let Some(lang) = &container.language {
            return Ok(lang.clone());
        }
        
        // Fall back to checking block metadata
        for block in blocks {
            if let Some(metadata) = &block.metadata {
                if let Some(lang) = metadata.get("language").and_then(|v| v.as_str()) {
                    return Ok(lang.to_string());
                }
            }
        }
        
        // Default fallback
        Ok("unknown".to_string())
    }
    
    fn aggregate_validation_results(&self, results: &[ValidationResult]) -> ValidationResult {
        let mut all_errors = Vec::new();
        let mut all_warnings = Vec::new();
        let mut total_coverage = 0.0;
        let mut total_fidelity = 0.0;
        let mut total_blocks = 0;
        let mut total_files = 0;
        let mut all_syntax_valid = true;
        
        for result in results {
            all_errors.extend(result.errors.clone());
            all_warnings.extend(result.warnings.clone());
            total_coverage += result.metrics.semantic_coverage;
            total_fidelity += result.metrics.reconstruction_fidelity;
            total_blocks += result.metrics.block_count;
            total_files += result.metrics.file_count;
            if !result.metrics.syntax_valid {
                all_syntax_valid = false;
            }
        }
        
        let count = results.len() as f64;
        
        ValidationResult {
            is_valid: all_syntax_valid && all_errors.is_empty(),
            errors: all_errors,
            warnings: all_warnings,
            metrics: super::validation::ValidationMetrics {
                syntax_valid: all_syntax_valid,
                semantic_coverage: if count > 0.0 { total_coverage / count } else { 0.0 },
                reconstruction_fidelity: if count > 0.0 { total_fidelity / count } else { 0.0 },
                block_count: total_blocks,
                file_count: total_files,
            },
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GenerationResult {
    pub migration_id: uuid::Uuid,
    pub total_files: usize,
    pub total_blocks: usize,
    #[allow(dead_code)]
    pub file_results: Vec<FileGenerationResult>,
    pub validation: ValidationResult,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FileGenerationResult {
    #[allow(dead_code)]
    pub original_path: String,
    #[allow(dead_code)]
    pub generated_path: std::path::PathBuf,
    #[allow(dead_code)]
    pub language: String,
    #[allow(dead_code)]
    pub block_count: usize,
    #[allow(dead_code)]
    pub content_length: usize,
}
