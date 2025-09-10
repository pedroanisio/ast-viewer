// Phase 2: Source-Code Elimination Implementation
// Following ARCHITECT principles: verification-first, incremental development

pub mod migration_strategy;
pub mod hierarchical_generation;
pub mod validation;
pub mod backup_system;
pub mod metrics;

use anyhow::Result;
use uuid::Uuid;
use crate::database::Database;

/// Phase 2 orchestrator implementing DoR/DoD compliance
pub struct Phase2Orchestrator {
    db: Database,
    validation: validation::ValidationEngine,
    backup: backup_system::BackupManager,
    migration: migration_strategy::MigrationManager,
    hierarchical: hierarchical_generation::HierarchicalGenerator,
    metrics: metrics::MetricsCollector,
}

impl Phase2Orchestrator {
    pub fn new(db: Database) -> Self {
        Self {
            validation: validation::ValidationEngine::new(db.clone()),
            backup: backup_system::BackupManager::new(db.clone()),
            migration: migration_strategy::MigrationManager::new(db.clone()),
            hierarchical: hierarchical_generation::HierarchicalGenerator::new(db.clone()),
            metrics: metrics::MetricsCollector::new(),
            db,
        }
    }

    /// Verify all Definition of Ready criteria before proceeding
    pub async fn verify_readiness(&mut self) -> Result<ReadinessReport> {
        let mut report = ReadinessReport::new();
        
        // DoR Check 1: Compilation errors resolved
        report.compilation_clean = self.check_compilation().await?;
        
        // DoR Check 2: Database schema alignment
        report.schema_aligned = self.validation.verify_schema_alignment().await?;
        
        // DoR Check 3: Language parsers operational
        report.parsers_operational = self.validation.verify_parsers().await?;
        
        // DoR Check 4: Template system coverage
        report.template_coverage = self.validation.verify_template_coverage().await?;
        
        // DoR Check 5: Test suite exists
        report.test_suite_adequate = self.validation.verify_test_suite().await?;
        
        // DoR Check 6: Backup system ready
        report.backup_system_ready = self.backup.verify_backup_capability().await?;
        
        // DoR Check 7: Rollback mechanism implemented
        report.rollback_ready = self.backup.verify_rollback_capability().await?;
        
        // DoR Check 8: Performance benchmarks established
        report.benchmarks_established = self.metrics.establish_baselines().await?;
        
        Ok(report)
    }

    /// Execute Phase 2 with comprehensive validation
    pub async fn execute_phase2(&mut self) -> Result<Phase2Results> {
        // Step 1: Verify readiness
        let readiness = self.verify_readiness().await?;
        if !readiness.is_ready() {
            return Err(anyhow::anyhow!("DoR not satisfied: {:?}", readiness.blocking_issues()));
        }

        // Step 2: Create backup
        let backup_id = self.backup.create_full_backup().await?;
        
        // Step 3: Execute migration
        let migration_results = self.migration.execute_source_code_elimination().await?;
        
        // Step 4: Validate results
        let validation_results = self.validation.validate_migration(&migration_results).await?;
        
        // Step 5: Verify DoD criteria
        let dod_compliance = self.verify_definition_of_done(&migration_results).await?;
        
        if !dod_compliance.is_complete() {
            // Rollback if DoD not met
            self.backup.restore_from_backup(backup_id).await?;
            return Err(anyhow::anyhow!("DoD not satisfied, rolled back: {:?}", dod_compliance.failures()));
        }

        Ok(Phase2Results {
            backup_id,
            migration_results,
            validation_results,
            dod_compliance,
        })
    }

    async fn check_compilation(&self) -> Result<bool> {
        // Use cargo check to verify compilation
        let output = std::process::Command::new("cargo")
            .args(&["check", "--quiet"])
            .current_dir("/apps/dev-tools/block-migrate")
            .output()?;
        
        Ok(output.status.success())
    }

    async fn verify_definition_of_done(&mut self, results: &migration_strategy::MigrationResults) -> Result<DoDCompliance> {
        let mut compliance = DoDCompliance::new();
        
        // DoD Check 1: 100% of containers have source_code set to NULL
        compliance.source_code_eliminated = self.validation.verify_source_code_elimination().await?;
        
        // DoD Check 2: Round-trip accuracy ≥99.5%
        compliance.round_trip_accuracy = self.validation.measure_round_trip_accuracy().await?;
        
        // DoD Check 3: All block types reconstruct successfully
        compliance.block_types_reconstruct = self.validation.verify_block_reconstruction().await?;
        
        // DoD Check 4: Formatting preservation verified
        compliance.formatting_preserved = self.validation.verify_formatting_preservation().await?;
        
        // DoD Check 5: Performance within 2x baseline
        compliance.performance_acceptable = self.metrics.verify_performance_requirements().await?;
        
        // DoD Check 6: Rollback tested successfully
        compliance.rollback_tested = results.rollback_test_passed;
        
        // DoD Check 7: Large repository migration successful
        compliance.large_repo_tested = results.large_repo_test_passed;
        
        // DoD Check 8: No regression in functionality
        compliance.no_regressions = self.validation.verify_no_regressions().await?;
        
        Ok(compliance)
    }
}

#[derive(Debug, Clone)]
pub struct ReadinessReport {
    pub compilation_clean: bool,
    pub schema_aligned: bool,
    pub parsers_operational: bool,
    pub template_coverage: bool,
    pub test_suite_adequate: bool,
    pub backup_system_ready: bool,
    pub rollback_ready: bool,
    pub benchmarks_established: bool,
}

impl ReadinessReport {
    pub fn new() -> Self {
        Self {
            compilation_clean: false,
            schema_aligned: false,
            parsers_operational: false,
            template_coverage: false,
            test_suite_adequate: false,
            backup_system_ready: false,
            rollback_ready: false,
            benchmarks_established: false,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.compilation_clean &&
        self.schema_aligned &&
        self.parsers_operational &&
        self.template_coverage &&
        self.test_suite_adequate &&
        self.backup_system_ready &&
        self.rollback_ready &&
        self.benchmarks_established
    }

    pub fn blocking_issues(&self) -> Vec<String> {
        let mut issues = Vec::new();
        
        if !self.compilation_clean { issues.push("Compilation errors exist".to_string()); }
        if !self.schema_aligned { issues.push("Database schema not aligned".to_string()); }
        if !self.parsers_operational { issues.push("Language parsers not operational".to_string()); }
        if !self.template_coverage { issues.push("Template coverage insufficient".to_string()); }
        if !self.test_suite_adequate { issues.push("Test suite inadequate".to_string()); }
        if !self.backup_system_ready { issues.push("Backup system not ready".to_string()); }
        if !self.rollback_ready { issues.push("Rollback mechanism not ready".to_string()); }
        if !self.benchmarks_established { issues.push("Performance benchmarks not established".to_string()); }
        
        issues
    }
}

#[derive(Debug, Clone)]
pub struct DoDCompliance {
    pub source_code_eliminated: bool,
    pub round_trip_accuracy: f64,
    pub block_types_reconstruct: bool,
    pub formatting_preserved: bool,
    pub performance_acceptable: bool,
    pub rollback_tested: bool,
    pub large_repo_tested: bool,
    pub no_regressions: bool,
}

impl DoDCompliance {
    pub fn new() -> Self {
        Self {
            source_code_eliminated: false,
            round_trip_accuracy: 0.0,
            block_types_reconstruct: false,
            formatting_preserved: false,
            performance_acceptable: false,
            rollback_tested: false,
            large_repo_tested: false,
            no_regressions: false,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.source_code_eliminated &&
        self.round_trip_accuracy >= 99.5 &&
        self.block_types_reconstruct &&
        self.formatting_preserved &&
        self.performance_acceptable &&
        self.rollback_tested &&
        self.large_repo_tested &&
        self.no_regressions
    }

    pub fn failures(&self) -> Vec<String> {
        let mut failures = Vec::new();
        
        if !self.source_code_eliminated { failures.push("Source code not fully eliminated".to_string()); }
        if self.round_trip_accuracy < 99.5 { failures.push(format!("Round-trip accuracy {}% < 99.5%", self.round_trip_accuracy)); }
        if !self.block_types_reconstruct { failures.push("Block reconstruction failures".to_string()); }
        if !self.formatting_preserved { failures.push("Formatting preservation failures".to_string()); }
        if !self.performance_acceptable { failures.push("Performance requirements not met".to_string()); }
        if !self.rollback_tested { failures.push("Rollback not tested".to_string()); }
        if !self.large_repo_tested { failures.push("Large repository testing incomplete".to_string()); }
        if !self.no_regressions { failures.push("Regressions detected".to_string()); }
        
        failures
    }
}

#[derive(Debug, Clone)]
pub struct Phase2Results {
    pub backup_id: Uuid,
    pub migration_results: migration_strategy::MigrationResults,
    pub validation_results: validation::ValidationResults,
    pub dod_compliance: DoDCompliance,
}
