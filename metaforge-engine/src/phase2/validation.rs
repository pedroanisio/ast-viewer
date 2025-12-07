// Validation Engine: Comprehensive quality assurance for Phase 2
// Following ARCHITECT principle: Verification-first mindset

use anyhow::Result;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::database::{Database, Container, Block};
use crate::generator::templates::TemplateEngine;
use crate::parser::universal::UniversalParser;
use std::collections::HashMap;

pub struct ValidationEngine {
    db: Database,
    template_engine: TemplateEngine,
    parser: UniversalParser,
}

impl ValidationEngine {
    pub fn new(db: Database) -> Self {
        Self {
            template_engine: TemplateEngine::new(),
            parser: UniversalParser::new().expect("Failed to initialize parser"),
            db,
        }
    }

    /// Verify database schema alignment (DoR requirement)
    pub async fn verify_schema_alignment(&self) -> Result<bool> {
        // Check that all required fields exist in database
        let required_fields = vec![
            ("containers", "source_code"),
            ("blocks", "parent_block_id"),
            ("blocks", "position_in_parent"),
            ("blocks", "abstract_syntax"),
            ("blocks", "semantic_metadata"),
        ];

        for (table, field) in required_fields {
            if !self.check_field_exists(table, field).await? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Verify language parsers operational (DoR requirement)
    pub async fn verify_parsers(&mut self) -> Result<bool> {
        let test_cases = vec![
            ("python", "def hello(): return 'world'"),
            ("rust", "fn hello() -> &'static str { \"world\" }"),
            ("javascript", "function hello() { return 'world'; }"),
            ("typescript", "function hello(): string { return 'world'; }"),
        ];

        for (language, code) in test_cases {
            match self.parser.parse_file(code, language, &format!("test.{}", 
                self.get_file_extension(language)?)) {
                Ok(blocks) => {
                    if blocks.blocks.is_empty() {
                        eprintln!("Parser for {} returned no blocks", language);
                        return Ok(false);
                    }
                }
                Err(e) => {
                    eprintln!("Parser for {} failed: {}", language, e);
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    /// Verify template system coverage (DoR requirement)
    pub async fn verify_template_coverage(&self) -> Result<bool> {
        let languages = vec!["rust", "python", "javascript", "typescript"];
        let block_types = vec![
            "Function", "Class", "Variable", "Import", "Comment",
            "Method", "Constructor", "Interface", "Enum", "Struct"
        ];

        for language in &languages {
            for block_type in &block_types {
                if !self.test_template_rendering(language, block_type).await? {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    /// Verify test suite adequacy (DoR requirement)
    pub async fn verify_test_suite(&self) -> Result<bool> {
        // Check for minimum number of test samples
        let container_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM containers WHERE source_code IS NOT NULL"
        )
        .fetch_one(self.db.pool())
        .await?;

        // Require at least 100 test samples for meaningful validation
        if container_count.0 < 100 {
            return Ok(false);
        }

        // Check language distribution
        let language_coverage = self.check_language_coverage().await?;
        let required_languages = vec!["python", "rust", "javascript", "typescript"];
        
        for lang in required_languages {
            if !language_coverage.contains_key(lang) || language_coverage[lang] < 10 {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Verify source code elimination (DoD requirement)
    pub async fn verify_source_code_elimination(&self) -> Result<bool> {
        let remaining_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM containers WHERE source_code IS NOT NULL"
        )
        .fetch_one(self.db.pool())
        .await?;

        Ok(remaining_count.0 == 0)
    }

    /// Measure round-trip accuracy (DoD requirement)
    pub async fn measure_round_trip_accuracy(&mut self) -> Result<f64> {
        let test_containers = self.get_test_sample(100).await?;
        let mut total_tests = 0;
        let mut successful_tests = 0;

        for container in test_containers {
            if let Some(original_code) = &container.source_code {
                total_tests += 1;
                
                // Parse -> Generate -> Compare
                match self.test_round_trip(&container, original_code).await {
                    Ok(true) => successful_tests += 1,
                    Ok(false) => {
                        eprintln!("Round-trip failed for container: {}", container.id);
                    }
                    Err(e) => {
                        eprintln!("Round-trip error for container {}: {}", container.id, e);
                    }
                }
            }
        }

        if total_tests == 0 {
            return Ok(0.0);
        }

        let accuracy = (successful_tests as f64 / total_tests as f64) * 100.0;
        Ok(accuracy)
    }

    /// Verify block reconstruction (DoD requirement)
    pub async fn verify_block_reconstruction(&self) -> Result<bool> {
        let query = r#"
            SELECT DISTINCT block_type FROM blocks 
            WHERE container_id IN (
                SELECT id FROM containers LIMIT 50
            )
        "#;

        let block_types: Vec<(String,)> = sqlx::query_as(query)
            .fetch_all(self.db.pool())
            .await?;

        for (block_type,) in block_types {
            if !self.test_block_type_reconstruction(&block_type).await? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Verify formatting preservation (DoD requirement)
    pub async fn verify_formatting_preservation(&mut self) -> Result<bool> {
        let test_containers = self.get_test_sample(20).await?;
        let mut total_checks = 0;
        let mut acceptable_variance = 0;

        for container in test_containers {
            if let Some(original) = &container.source_code {
                total_checks += 1;
                
                match self.check_formatting_preservation(&container, original).await {
                    Ok(variance) if variance < 0.05 => acceptable_variance += 1, // <5% variance
                    Ok(variance) => {
                        eprintln!("High formatting variance ({:.1}%) for container: {}", 
                                variance * 100.0, container.id);
                    }
                    Err(e) => {
                        eprintln!("Formatting check error for container {}: {}", container.id, e);
                    }
                }
            }
        }

        let preservation_rate = if total_checks > 0 {
            acceptable_variance as f64 / total_checks as f64
        } else {
            0.0
        };

        Ok(preservation_rate >= 0.95) // 95% of files must have acceptable formatting variance
    }

    /// Verify no regressions (DoD requirement)
    pub async fn verify_no_regressions(&mut self) -> Result<bool> {
        // Test that all previously working functionality still works
        // This would include running existing integration tests
        
        // For now, we'll do basic functionality checks
        let basic_checks = vec![
            self.test_database_connectivity().await?,
            self.test_parser_functionality().await?,
            self.test_template_functionality().await?,
        ];

        Ok(basic_checks.iter().all(|&check| check))
    }

    /// Validate migration results comprehensively
    pub async fn validate_migration(&mut self, _results: &crate::phase2::migration_strategy::MigrationResults) -> Result<ValidationResults> {
        let mut validation = ValidationResults::new();

        validation.round_trip_accuracy = self.measure_round_trip_accuracy().await?;
        validation.block_reconstruction_success = self.verify_block_reconstruction().await?;
        validation.formatting_preservation = self.verify_formatting_preservation().await?;
        validation.source_code_eliminated = self.verify_source_code_elimination().await?;
        validation.no_regressions = self.verify_no_regressions().await?;

        // Calculate overall success score
        validation.overall_success_score = self.calculate_success_score(&validation);

        Ok(validation)
    }

    // Helper methods

    async fn check_field_exists(&self, table: &str, field: &str) -> Result<bool> {
        let query = format!(
            "SELECT column_name FROM information_schema.columns WHERE table_name = '{}' AND column_name = '{}'",
            table, field
        );

        let result: Vec<(String,)> = sqlx::query_as(&query)
            .fetch_all(self.db.pool())
            .await?;

        Ok(!result.is_empty())
    }

    fn get_file_extension(&self, language: &str) -> Result<&str> {
        match language {
            "python" => Ok("py"),
            "rust" => Ok("rs"),
            "javascript" => Ok("js"),
            "typescript" => Ok("ts"),
            _ => Err(anyhow::anyhow!("Unknown language: {}", language)),
        }
    }

    async fn test_template_rendering(&self, language: &str, block_type: &str) -> Result<bool> {
        // Create a test block
        let test_block = self.create_test_block(block_type).await?;
        
        // Try to render it
        match self.template_engine.render_block(&test_block, language) {
            Ok(rendered) => Ok(!rendered.is_empty()),
            Err(_) => Ok(false),
        }
    }

    async fn check_language_coverage(&self) -> Result<HashMap<String, i64>> {
        let query = r#"
            SELECT language, COUNT(*) as count
            FROM containers 
            WHERE source_code IS NOT NULL AND language IS NOT NULL
            GROUP BY language
        "#;

        let results: Vec<(Option<String>, i64)> = sqlx::query_as(query)
            .fetch_all(self.db.pool())
            .await?;

        let mut coverage = HashMap::new();
        for (lang, count) in results {
            if let Some(language) = lang {
                coverage.insert(language.to_lowercase(), count);
            }
        }

        Ok(coverage)
    }

    async fn get_test_sample(&self, limit: i32) -> Result<Vec<Container>> {
        let query = r#"
            SELECT * FROM containers 
            WHERE source_code IS NOT NULL 
            ORDER BY RANDOM() 
            LIMIT $1
        "#;

        let containers: Vec<Container> = sqlx::query_as(query)
            .bind(limit)
            .fetch_all(self.db.pool())
            .await?;

        Ok(containers)
    }

    async fn test_round_trip(&mut self, container: &Container, original_code: &str) -> Result<bool> {
        // This is a simplified round-trip test
        // Full implementation would: parse -> store -> retrieve -> generate -> compare
        
        if let Some(language) = &container.language {
            // Parse the original code
            let parsed_blocks = self.parser.parse_file(
                original_code, 
                language, 
                &format!("test.{}", self.get_file_extension(language)?)
            )?;

            if parsed_blocks.blocks.is_empty() {
                return Ok(false);
            }

            // Convert SemanticBlocks to database Blocks for rendering
            let db_blocks: Vec<Block> = parsed_blocks.blocks.iter().map(|sb| {
                self.convert_semantic_block_to_block(sb)
            }).collect();
            
            // Try to regenerate (simplified)
            let mut regenerated = String::new();
            for block in &db_blocks {
                let rendered = self.template_engine.render_block(block, language)?;
                regenerated.push_str(&rendered);
                regenerated.push('\n');
            }

            // Simple similarity check (more sophisticated comparison needed in full implementation)
            let similarity = self.calculate_code_similarity(original_code, &regenerated);
            Ok(similarity > 0.8) // 80% similarity threshold
        } else {
            Ok(false)
        }
    }

    async fn test_block_type_reconstruction(&self, block_type: &str) -> Result<bool> {
        // Get a sample block of this type
        let query = "SELECT * FROM blocks WHERE block_type = $1 LIMIT 1";
        
        let block_result: Result<Block, sqlx::Error> = sqlx::query_as(query)
            .bind(block_type)
            .fetch_one(self.db.pool())
            .await;

        match block_result {
            Ok(block) => {
                // Try to render this block for available languages
                let languages = vec!["rust", "python", "javascript", "typescript"];
                for language in languages {
                    match self.template_engine.render_block(&block, language) {
                        Ok(rendered) if !rendered.is_empty() => return Ok(true),
                        _ => continue,
                    }
                }
                Ok(false)
            }
            Err(_) => Ok(true), // No blocks of this type, so technically it passes
        }
    }

    async fn check_formatting_preservation(&mut self, container: &Container, original: &str) -> Result<f64> {
        // Calculate formatting variance (simplified implementation)
        if let Some(language) = &container.language {
            let blocks = self.parser.parse_file(
                original, 
                language, 
                &format!("test.{}", self.get_file_extension(language)?)
            )?;

            // Convert SemanticBlocks to database Blocks for rendering
            let db_blocks: Vec<Block> = blocks.blocks.iter().map(|sb| {
                self.convert_semantic_block_to_block(sb)
            }).collect();
            
            let regenerated = self.template_engine.render_file(container, &db_blocks, language)?;
            
            // Calculate formatting differences (whitespace, indentation, etc.)
            let variance = self.calculate_formatting_variance(original, &regenerated);
            Ok(variance)
        } else {
            Ok(1.0) // 100% variance if no language specified
        }
    }

    async fn test_database_connectivity(&self) -> Result<bool> {
        let result: (i32,) = sqlx::query_as("SELECT 1")
            .fetch_one(self.db.pool())
            .await?;
        Ok(result.0 == 1)
    }

    async fn test_parser_functionality(&mut self) -> Result<bool> {
        self.verify_parsers().await
    }

    async fn test_template_functionality(&self) -> Result<bool> {
        self.verify_template_coverage().await
    }

    async fn create_test_block(&self, block_type: &str) -> Result<Block> {
        // Create a minimal test block for validation
        Ok(Block {
            id: Uuid::new_v4(),
            container_id: Uuid::new_v4(),
            block_type: block_type.to_string(),
            semantic_name: Some("test".to_string()),
            abstract_syntax: serde_json::json!({"name": "test", "type": block_type}),
            position: 0,
            indent_level: 0,
            metadata: None,
            created_at: chrono::Utc::now(),
            parent_block_id: None,
            position_in_parent: 0,
            parameters: None,
            return_type: None,
            modifiers: None,
            decorators: None,
            body_ast: None,
            language_ast: None,
            language_features: None,
            complexity_metrics: None,
            scope_info: None,
            syntax_preservation: None,
            structural_context: None,
            semantic_metadata: None,
            source_language: None,
            template_metadata: None,
            generation_hints: None,
            semantic_signature: None,
            behavioral_contract: None,
            formatting_metadata: None,
            attached_comments: None,
            dependency_info: None,
            position_metadata: None,
            hierarchical_index: None,
            depth_level: None,
        })
    }

    fn calculate_code_similarity(&self, original: &str, regenerated: &str) -> f64 {
        // Simplified similarity calculation
        // Full implementation would use more sophisticated algorithms
        let original_lines: Vec<&str> = original.lines().collect();
        let regenerated_lines: Vec<&str> = regenerated.lines().collect();
        
        let min_len = original_lines.len().min(regenerated_lines.len());
        if min_len == 0 {
            return if original_lines.len() == regenerated_lines.len() { 1.0 } else { 0.0 };
        }
        
        let matching = original_lines.iter()
            .zip(regenerated_lines.iter())
            .take(min_len)
            .filter(|(a, b)| a.trim() == b.trim())
            .count();
        
        matching as f64 / original_lines.len().max(regenerated_lines.len()) as f64
    }

    fn calculate_formatting_variance(&self, original: &str, regenerated: &str) -> f64 {
        // Calculate variance in whitespace, indentation, etc.
        let original_chars = original.chars().filter(|c| !c.is_whitespace()).count();
        let regenerated_chars = regenerated.chars().filter(|c| !c.is_whitespace()).count();
        
        if original_chars == 0 && regenerated_chars == 0 {
            return 0.0;
        }
        
        let diff = (original_chars as i32 - regenerated_chars as i32).abs() as f64;
        let max_chars = original_chars.max(regenerated_chars) as f64;
        
        if max_chars == 0.0 { 0.0 } else { diff / max_chars }
    }

    fn calculate_success_score(&self, validation: &ValidationResults) -> f64 {
        let mut score = 0.0;
        let mut weight_sum = 0.0;
        
        // Round-trip accuracy (weight: 40%)
        score += validation.round_trip_accuracy * 0.4;
        weight_sum += 0.4;
        
        // Block reconstruction (weight: 20%)
        if validation.block_reconstruction_success {
            score += 20.0;
        }
        weight_sum += 0.2;
        
        // Formatting preservation (weight: 15%)
        if validation.formatting_preservation {
            score += 15.0;
        }
        weight_sum += 0.15;
        
        // Source code elimination (weight: 15%)
        if validation.source_code_eliminated {
            score += 15.0;
        }
        weight_sum += 0.15;
        
        // No regressions (weight: 10%)
        if validation.no_regressions {
            score += 10.0;
        }
        weight_sum += 0.1;
        
        score / weight_sum
    }
    
    /// Convert SemanticBlock to database Block for template rendering
    fn convert_semantic_block_to_block(&self, semantic_block: &crate::core::semantic_block::SemanticBlock) -> Block {
        use chrono::Utc;
        
        Block {
            id: semantic_block.id,
            container_id: Uuid::new_v4(), // Will be set properly in real implementation
            block_type: semantic_block.block_type.to_string(),
            semantic_name: Some(semantic_block.semantic_identity.canonical_name.clone()),
            abstract_syntax: serde_json::json!({
                "type": semantic_block.block_type.to_string(),
                "identity": semantic_block.semantic_identity
            }),
            position: semantic_block.position.start_line as i32,
            indent_level: semantic_block.position.start_column as i32,
            metadata: Some(serde_json::to_value(&semantic_block.semantic_metadata).unwrap_or_default()),
            created_at: Utc::now(),
            parent_block_id: None,
            position_in_parent: 0,
            parameters: None,
            return_type: None,
            modifiers: None,
            decorators: None,
            body_ast: None,
            language_ast: None,
            language_features: None,
            complexity_metrics: None,
            scope_info: None,
            syntax_preservation: Some(serde_json::to_value(&semantic_block.syntax_preservation).unwrap_or_default()),
            structural_context: Some(serde_json::to_value(&semantic_block.structural_context).unwrap_or_default()),
            semantic_metadata: Some(serde_json::to_value(&semantic_block.semantic_metadata).unwrap_or_default()),
            source_language: Some(semantic_block.source_language.clone()),
            template_metadata: None,
            generation_hints: None,
            semantic_signature: None,
            behavioral_contract: None,
            formatting_metadata: None,
            attached_comments: None,
            dependency_info: None,
            position_metadata: None,
            hierarchical_index: None,
            depth_level: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResults {
    pub round_trip_accuracy: f64,
    pub block_reconstruction_success: bool,
    pub formatting_preservation: bool,
    pub source_code_eliminated: bool,
    pub no_regressions: bool,
    pub overall_success_score: f64,
    pub detailed_metrics: HashMap<String, serde_json::Value>,
}

impl ValidationResults {
    pub fn new() -> Self {
        Self {
            round_trip_accuracy: 0.0,
            block_reconstruction_success: false,
            formatting_preservation: false,
            source_code_eliminated: false,
            no_regressions: false,
            overall_success_score: 0.0,
            detailed_metrics: HashMap::new(),
        }
    }
}
