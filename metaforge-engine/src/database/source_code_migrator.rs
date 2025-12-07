use anyhow::{Result, anyhow};
use uuid::Uuid;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::database::{Database, Container, Block};
use crate::generator::templates::TemplateEngine;

/// Handles migration from source_code field dependencies to pure semantic storage
pub struct SourceCodeMigrator {
    db: Database,
    template_engine: TemplateEngine,
    dry_run: bool,
}

#[derive(Debug, Clone)]
pub struct MigrationReport {
    pub total_containers: usize,
    pub successful_migrations: usize,
    pub failed_migrations: Vec<(Uuid, String)>,
    pub semantic_quality_scores: HashMap<Uuid, f64>,
    pub reconstruction_accuracy: HashMap<Uuid, f64>,
    pub migration_duration: std::time::Duration,
}

#[derive(Debug, Clone)]
pub struct ContainerMigrationResult {
    pub container_id: Uuid,
    pub original_source_size: usize,
    pub semantic_blocks_created: usize,
    pub reconstruction_quality: f64,
    pub semantic_completeness: f64,
    pub migration_successful: bool,
    pub error_messages: Vec<String>,
}

impl SourceCodeMigrator {
    pub fn new(db: Database, dry_run: bool) -> Self {
        Self {
            db,
            template_engine: TemplateEngine::new(),
            dry_run,
        }
    }

    /// Migrate all containers from source_code dependencies to pure semantic storage
    pub async fn migrate_all_containers(&self) -> Result<MigrationReport> {
        let start_time = std::time::Instant::now();
        
        // Get all containers that still have source_code dependencies
        let containers_with_source = self.get_containers_with_source_code().await?;
        let total_containers = containers_with_source.len();
        
        println!("🔄 Starting migration of {} containers from source_code to semantic storage", total_containers);
        
        let mut report = MigrationReport {
            total_containers,
            successful_migrations: 0,
            failed_migrations: Vec::new(),
            semantic_quality_scores: HashMap::new(),
            reconstruction_accuracy: HashMap::new(),
            migration_duration: std::time::Duration::default(),
        };
        
        for container in containers_with_source {
            match self.migrate_container(&container).await {
                Ok(result) => {
                    if result.migration_successful {
                        report.successful_migrations += 1;
                        report.semantic_quality_scores.insert(container.id, result.semantic_completeness);
                        report.reconstruction_accuracy.insert(container.id, result.reconstruction_quality);
                        
                        println!("✅ Migrated container '{}' - {} blocks, {:.1}% quality", 
                                container.name, result.semantic_blocks_created, result.semantic_completeness * 100.0);
                    } else {
                        report.failed_migrations.push((container.id, result.error_messages.join("; ")));
                        println!("❌ Failed to migrate container '{}': {}", 
                                container.name, result.error_messages.join("; "));
                    }
                }
                Err(e) => {
                    report.failed_migrations.push((container.id, e.to_string()));
                    println!("❌ Error migrating container '{}': {}", container.name, e);
                }
            }
        }
        
        report.migration_duration = start_time.elapsed();
        
        // Generate final report
        self.print_migration_summary(&report);
        
        Ok(report)
    }

    /// Migrate a single container from source_code to semantic storage
    pub async fn migrate_container(&self, container: &Container) -> Result<ContainerMigrationResult> {
        let mut result = ContainerMigrationResult {
            container_id: container.id,
            original_source_size: 0,
            semantic_blocks_created: 0,
            reconstruction_quality: 0.0,
            semantic_completeness: 0.0,
            migration_successful: false,
            error_messages: Vec::new(),
        };

        // Check if container has source code to migrate
        let source_code = match &container.source_code {
            Some(code) if !code.is_empty() => code,
            _ => {
                result.error_messages.push("No source code to migrate".to_string());
                return Ok(result);
            }
        };

        result.original_source_size = source_code.len();

        // Start migration transaction
        if !self.dry_run {
            let transaction = self.db.begin_transaction().await?;
            
            match self.perform_migration_steps(container, source_code, &mut result).await {
                Ok(()) => {
                    transaction.commit().await?;
                    result.migration_successful = true;
                }
                Err(e) => {
                    transaction.rollback().await?;
                    result.error_messages.push(format!("Migration failed: {}", e));
                    return Ok(result);
                }
            }
        } else {
            // Dry run - validate without making changes
            self.validate_migration_feasibility(container, source_code, &mut result).await?;
        }

        Ok(result)
    }

    async fn perform_migration_steps(
        &self, 
        container: &Container, 
        source_code: &str, 
        result: &mut ContainerMigrationResult
    ) -> Result<()> {
        // Step 1: Backup original source code
        self.backup_source_code(container, source_code).await?;

        // Step 2: Enhance existing semantic blocks with better metadata
        self.enhance_semantic_blocks(container.id, result).await?;

        // Step 3: Validate reconstruction quality
        let reconstructed = self.reconstruct_from_semantics(container.id).await?;
        result.reconstruction_quality = self.calculate_reconstruction_quality(source_code, &reconstructed);

        // Step 4: Calculate semantic completeness
        result.semantic_completeness = self.calculate_semantic_completeness(container.id).await?;

        // Step 5: Remove source_code field if quality is acceptable
        if result.reconstruction_quality >= 0.7 && result.semantic_completeness >= 0.8 {
            self.remove_source_code_field(container.id).await?;
        } else {
            return Err(anyhow!(
                "Migration quality insufficient: reconstruction={:.1}%, semantic={:.1}%",
                result.reconstruction_quality * 100.0,
                result.semantic_completeness * 100.0
            ));
        }

        Ok(())
    }

    async fn validate_migration_feasibility(
        &self,
        container: &Container,
        source_code: &str,
        result: &mut ContainerMigrationResult
    ) -> Result<()> {
        // Check if semantic blocks exist
        let blocks = self.db.get_blocks_by_container(container.id).await?;
        result.semantic_blocks_created = blocks.len();

        if blocks.is_empty() {
            result.error_messages.push("No semantic blocks found for container".to_string());
            return Ok(());
        }

        // Validate semantic completeness
        result.semantic_completeness = self.calculate_semantic_completeness(container.id).await?;

        // Test reconstruction
        let reconstructed = self.reconstruct_from_semantics(container.id).await?;
        result.reconstruction_quality = self.calculate_reconstruction_quality(source_code, &reconstructed);

        result.migration_successful = result.reconstruction_quality >= 0.7 && result.semantic_completeness >= 0.8;

        Ok(())
    }

    async fn backup_source_code(&self, container: &Container, source_code: &str) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO source_code_backup (container_id, original_source_code, original_path, original_hash)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (container_id) DO UPDATE SET
                original_source_code = EXCLUDED.original_source_code,
                original_path = EXCLUDED.original_path,
                original_hash = EXCLUDED.original_hash,
                backup_created_at = NOW()
            "#,
        )
        .bind(container.id)
        .bind(source_code)
        .bind(container.original_path.clone())
        .bind(container.original_hash.clone())
        .execute(self.db.pool())
        .await?;

        Ok(())
    }

    async fn enhance_semantic_blocks(&self, container_id: Uuid, result: &mut ContainerMigrationResult) -> Result<()> {
        let blocks = self.db.get_blocks_by_container(container_id).await?;
        let block_count = blocks.len(); // Store count before moving
        
        for block in blocks {
            // Enhance block with better semantic metadata
            let enhanced_metadata = self.enhance_block_metadata(&block).await?;
            
            // Update block with enhanced semantic information
            sqlx::query!(
                r#"
                UPDATE blocks SET
                    semantic_signature = $2,
                    behavioral_contract = $3,
                    formatting_metadata = $4,
                    dependency_info = $5
                WHERE id = $1
                "#,
                block.id,
                enhanced_metadata.semantic_signature,
                enhanced_metadata.behavioral_contract,
                enhanced_metadata.formatting_metadata,
                enhanced_metadata.dependency_info
            )
            .execute(self.db.pool())
            .await?;
        }

        result.semantic_blocks_created = block_count;
        Ok(())
    }

    async fn enhance_block_metadata(&self, block: &Block) -> Result<EnhancedMetadata> {
        let semantic_signature = json!({
            "block_type": block.block_type,
            "semantic_name": block.semantic_name,
            "parameters": block.parameters,
            "return_type": block.return_type,
            "modifiers": block.modifiers,
            "complexity_score": self.calculate_block_complexity(block),
            "purity_level": self.analyze_purity(block),
            "side_effects": self.analyze_side_effects(block)
        });

        let behavioral_contract = json!({
            "preconditions": self.extract_preconditions(block),
            "postconditions": self.extract_postconditions(block),
            "invariants": self.extract_invariants(block),
            "performance_characteristics": self.analyze_performance(block)
        });

        let formatting_metadata = json!({
            "indentation_style": self.detect_indentation_style(block),
            "line_ending_style": "\\n",
            "spacing_preferences": self.analyze_spacing(block),
            "comment_style": self.detect_comment_style(block)
        });

        let dependency_info = json!({
            "direct_dependencies": self.extract_direct_dependencies(block),
            "transitive_dependencies": [], // Will be calculated separately
            "provides": self.extract_provided_interfaces(block),
            "requires": self.extract_required_interfaces(block)
        });

        Ok(EnhancedMetadata {
            semantic_signature,
            behavioral_contract,
            formatting_metadata,
            dependency_info,
        })
    }

    async fn reconstruct_from_semantics(&self, container_id: Uuid) -> Result<String> {
        // Temporary implementation until database function is created
        // In a full implementation, this would call a PostgreSQL function
        
        // For now, return a placeholder indicating semantic reconstruction
        let _blocks = sqlx::query!(
            "SELECT id, block_type, semantic_name FROM blocks WHERE container_id = $1 ORDER BY position",
            container_id
        )
        .fetch_all(self.db.pool())
        .await?;
        
        // Placeholder implementation - would use template engine here
        Ok("// Reconstructed from semantic blocks\n// Full implementation pending".to_string())
    }

    fn calculate_reconstruction_quality(&self, original: &str, reconstructed: &str) -> f64 {
        // Implement semantic similarity comparison
        let original_lines: Vec<&str> = original.lines().filter(|l| !l.trim().is_empty()).collect();
        let reconstructed_lines: Vec<&str> = reconstructed.lines().filter(|l| !l.trim().is_empty()).collect();
        
        if original_lines.is_empty() {
            return if reconstructed_lines.is_empty() { 1.0 } else { 0.0 };
        }

        // Calculate line-based similarity (simplified)
        let line_ratio = reconstructed_lines.len() as f64 / original_lines.len() as f64;
        let line_score = if line_ratio > 1.0 { 1.0 / line_ratio } else { line_ratio };

        // Calculate content similarity (simplified - could use more sophisticated algorithms)
        let content_similarity = self.calculate_content_similarity(&original_lines, &reconstructed_lines);

        // Weighted average
        (line_score * 0.3) + (content_similarity * 0.7)
    }

    fn calculate_content_similarity(&self, original: &[&str], reconstructed: &[&str]) -> f64 {
        // Simple token-based similarity
        let original_tokens: Vec<&str> = original.iter()
            .flat_map(|line| line.split_whitespace())
            .filter(|token| !token.is_empty())
            .collect();
        
        let reconstructed_tokens: Vec<&str> = reconstructed.iter()
            .flat_map(|line| line.split_whitespace())
            .filter(|token| !token.is_empty())
            .collect();

        if original_tokens.is_empty() {
            return if reconstructed_tokens.is_empty() { 1.0 } else { 0.0 };
        }

        let matching_tokens = original_tokens.iter()
            .filter(|token| reconstructed_tokens.contains(token))
            .count();

        matching_tokens as f64 / original_tokens.len() as f64
    }

    async fn calculate_semantic_completeness(&self, container_id: Uuid) -> Result<f64> {
        let result = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total_blocks,
                COUNT(CASE WHEN parameters IS NOT NULL THEN 1 END) as blocks_with_params,
                COUNT(CASE WHEN return_type IS NOT NULL THEN 1 END) as blocks_with_return_type,
                COUNT(CASE WHEN modifiers IS NOT NULL THEN 1 END) as blocks_with_modifiers,
                COUNT(CASE WHEN body_ast IS NOT NULL THEN 1 END) as blocks_with_body_ast,
                COUNT(CASE WHEN language_features IS NOT NULL THEN 1 END) as blocks_with_lang_features
            FROM blocks WHERE container_id = $1
            "#,
            container_id
        )
        .fetch_one(self.db.pool())
        .await?;

        let total = result.total_blocks.unwrap_or(0) as f64;
        if total == 0.0 {
            return Ok(0.0);
        }

        let completeness_factors = [
            result.blocks_with_params.unwrap_or(0) as f64 / total,
            result.blocks_with_return_type.unwrap_or(0) as f64 / total,
            result.blocks_with_modifiers.unwrap_or(0) as f64 / total,
            result.blocks_with_body_ast.unwrap_or(0) as f64 / total,
            result.blocks_with_lang_features.unwrap_or(0) as f64 / total,
        ];

        // Weighted average of completeness factors
        let weighted_score = completeness_factors.iter().sum::<f64>() / completeness_factors.len() as f64;
        Ok(weighted_score)
    }

    async fn remove_source_code_field(&self, container_id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE containers SET 
                source_code = NULL,
                semantic_summary = semantic_summary || jsonb_build_object(
                    'source_code_free', true,
                    'migration_completed_at', NOW()
                )
            WHERE id = $1
            "#,
            container_id
        )
        .execute(self.db.pool())
        .await?;

        Ok(())
    }

    async fn get_containers_with_source_code(&self) -> Result<Vec<Container>> {
        let containers = sqlx::query_as!(
            Container,
            r#"
            SELECT id, name, container_type, language, original_path, original_hash,
                   source_code, version as "version!", 
                   created_at as "created_at!", 
                   updated_at as "updated_at!",
                   NULL::jsonb as semantic_summary,
                   NULL::jsonb as parsing_metadata,
                   NULL::jsonb as formatting_preferences,
                   NULL::jsonb as reconstruction_hints
            FROM containers 
            WHERE source_code IS NOT NULL AND source_code != ''
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(self.db.pool())
        .await?;

        Ok(containers)
    }

    fn print_migration_summary(&self, report: &MigrationReport) {
        println!("\n🎯 SOURCE CODE MIGRATION SUMMARY");
        println!("================================");
        println!("Total containers: {}", report.total_containers);
        println!("Successful migrations: {}", report.successful_migrations);
        println!("Failed migrations: {}", report.failed_migrations.len());
        println!("Success rate: {:.1}%", 
                (report.successful_migrations as f64 / report.total_containers as f64) * 100.0);
        println!("Migration duration: {:.2}s", report.migration_duration.as_secs_f64());

        if !report.semantic_quality_scores.is_empty() {
            let avg_quality: f64 = report.semantic_quality_scores.values().sum::<f64>() 
                                  / report.semantic_quality_scores.len() as f64;
            println!("Average semantic quality: {:.1}%", avg_quality * 100.0);
        }

        if !report.reconstruction_accuracy.is_empty() {
            let avg_accuracy: f64 = report.reconstruction_accuracy.values().sum::<f64>() 
                                   / report.reconstruction_accuracy.len() as f64;
            println!("Average reconstruction accuracy: {:.1}%", avg_accuracy * 100.0);
        }

        if !report.failed_migrations.is_empty() {
            println!("\n❌ Failed migrations:");
            for (container_id, error) in &report.failed_migrations {
                println!("  {} - {}", container_id, error);
            }
        }
    }

    // Helper methods for semantic analysis
    fn calculate_block_complexity(&self, _block: &Block) -> u32 {
        // Placeholder - implement cyclomatic complexity calculation
        1
    }

    fn analyze_purity(&self, _block: &Block) -> String {
        // Placeholder - implement purity analysis
        "unknown".to_string()
    }

    fn analyze_side_effects(&self, _block: &Block) -> Vec<String> {
        // Placeholder - implement side effect analysis
        vec![]
    }

    fn extract_preconditions(&self, _block: &Block) -> Vec<String> {
        vec![]
    }

    fn extract_postconditions(&self, _block: &Block) -> Vec<String> {
        vec![]
    }

    fn extract_invariants(&self, _block: &Block) -> Vec<String> {
        vec![]
    }

    fn analyze_performance(&self, _block: &Block) -> Value {
        json!({})
    }

    fn detect_indentation_style(&self, _block: &Block) -> String {
        "spaces".to_string()
    }

    fn analyze_spacing(&self, _block: &Block) -> Value {
        json!({})
    }

    fn detect_comment_style(&self, _block: &Block) -> String {
        "//".to_string()
    }

    fn extract_direct_dependencies(&self, _block: &Block) -> Vec<String> {
        vec![]
    }

    fn extract_provided_interfaces(&self, _block: &Block) -> Vec<String> {
        vec![]
    }

    fn extract_required_interfaces(&self, _block: &Block) -> Vec<String> {
        vec![]
    }
}

#[derive(Debug)]
struct EnhancedMetadata {
    semantic_signature: Value,
    behavioral_contract: Value,
    formatting_metadata: Value,
    dependency_info: Value,
}

// DatabaseTransaction moved to schema.rs - remove duplicate
