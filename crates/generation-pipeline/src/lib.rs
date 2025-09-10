//! # Generation Pipeline
//! 
//! Orchestrates the complete AST-based code generation pipeline.
//! Implements the clean architecture from plan-part-0.md with strict
//! adherence to ADR-001: NO SOURCE CODE FALLBACK.

pub mod pipeline;
pub mod tracer;
pub mod orchestrator;

pub use pipeline::{GenerationPipeline, PipelineConfig, PipelineResult};
pub use tracer::{GenerationTracer, TraceEvent, TraceLevel};
pub use orchestrator::{PipelineOrchestrator, ExecutionPlan};

use anyhow::Result;
use ast_extractor::{ASTExtractor, ExtractionContext, ExtractionResult};
use semantic_mapper::{SemanticMapper, EnhancedSemanticBlock};
use code_builders::{CodeBuilder, BuildConfig, BuildResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;
use uuid::Uuid;

/// Main pipeline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub language: String,
    pub strict_mode: bool, // ADR-001 compliance
    pub enable_tracing: bool,
    pub max_parallel_blocks: usize,
    pub quality_threshold: f64,
    pub build_config: BuildConfig,
    pub extraction_config: ExtractionSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionSettings {
    pub extract_expressions: bool,
    pub max_depth: Option<usize>,
    pub include_comments: bool,
    pub analyze_dependencies: bool,
}

/// Complete pipeline execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResult {
    pub success: bool,
    pub generated_files: HashMap<String, String>, // file_path -> generated_code
    pub metadata: PipelineMetadata,
    pub trace_events: Vec<TraceEvent>,
    pub errors: Vec<PipelineError>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineMetadata {
    pub pipeline_id: Uuid,
    pub execution_time_ms: u64,
    pub files_processed: usize,
    pub blocks_extracted: usize,
    pub blocks_generated: usize,
    pub ast_utilization: f64,
    pub generation_quality: f64,
    pub stage_timings: HashMap<String, u64>, // stage_name -> time_ms
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineError {
    pub stage: String,
    pub error_type: String,
    pub message: String,
    pub file_path: Option<String>,
    pub block_id: Option<Uuid>,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            language: "python".to_string(),
            strict_mode: true, // ADR-001: Always strict by default
            enable_tracing: true,
            max_parallel_blocks: 10,
            quality_threshold: 0.85,
            build_config: BuildConfig::default(),
            extraction_config: ExtractionSettings {
                extract_expressions: true,
                max_depth: None,
                include_comments: false,
                analyze_dependencies: true,
            },
        }
    }
}

impl PipelineResult {
    pub fn new(pipeline_id: Uuid) -> Self {
        Self {
            success: false,
            generated_files: HashMap::new(),
            metadata: PipelineMetadata {
                pipeline_id,
                execution_time_ms: 0,
                files_processed: 0,
                blocks_extracted: 0,
                blocks_generated: 0,
                ast_utilization: 0.0,
                generation_quality: 0.0,
                stage_timings: HashMap::new(),
            },
            trace_events: Vec::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, stage: String, error_type: String, message: String) {
        self.errors.push(PipelineError {
            stage,
            error_type,
            message,
            file_path: None,
            block_id: None,
        });
        self.success = false;
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn add_generated_file(&mut self, file_path: String, code: String) {
        self.generated_files.insert(file_path, code);
    }

    pub fn finalize(&mut self, start_time: Instant) {
        self.metadata.execution_time_ms = start_time.elapsed().as_millis() as u64;
        self.success = self.errors.is_empty() && 
                      self.metadata.generation_quality >= 0.7;
    }

    /// Get summary statistics
    pub fn get_summary(&self) -> String {
        format!(
            "Pipeline {} - Success: {}, Files: {}, Blocks: {}/{}, Quality: {:.1}%, Time: {}ms",
            self.metadata.pipeline_id,
            self.success,
            self.metadata.files_processed,
            self.metadata.blocks_generated,
            self.metadata.blocks_extracted,
            self.metadata.generation_quality * 100.0,
            self.metadata.execution_time_ms
        )
    }
}
