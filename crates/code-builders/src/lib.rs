//! # Code Builders
//! 
//! Language-specific code builders that generate source code from semantic AST data.
//! This crate implements the core principle: **NO SOURCE CODE FALLBACK**.
//! All generation must come from semantic understanding.

pub mod builders;
pub mod formatters;
pub mod traits;

pub use builders::{PythonBuilder, RustBuilder, JavaScriptBuilder};
pub use formatters::{PythonFormatter, RustFormatter, JavaScriptFormatter};
pub use traits::{CodeBuilder, LanguageFormatter};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for code building
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub language: String,
    pub indent_style: IndentStyle,
    pub line_ending: LineEnding,
    pub max_line_length: usize,
    pub format_on_build: bool,
    pub strict_mode: bool, // If true, fail on incomplete AST data
    pub generation_hints: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndentStyle {
    Spaces(usize),
    Tabs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LineEnding {
    Unix,    // \n
    Windows, // \r\n
    Mac,     // \r
}

/// Result of code building
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    pub generated_code: String,
    pub metadata: BuildMetadata,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

/// Metadata about the build process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildMetadata {
    pub blocks_processed: usize,
    pub lines_generated: usize,
    pub build_time_ms: u64,
    pub ast_utilization: f64, // Percentage of AST data actually used
    pub generation_quality: f64, // Quality score 0.0-1.0
    pub language_specific: HashMap<String, serde_json::Value>,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            language: "python".to_string(),
            indent_style: IndentStyle::Spaces(4),
            line_ending: LineEnding::Unix,
            max_line_length: 88, // Black's default
            format_on_build: true,
            strict_mode: true, // Default to strict mode - fail on incomplete data
            generation_hints: HashMap::new(),
        }
    }
}

impl BuildResult {
    pub fn new(generated_code: String) -> Self {
        let lines_generated = generated_code.lines().count();
        
        Self {
            generated_code,
            metadata: BuildMetadata {
                blocks_processed: 0,
                lines_generated,
                build_time_ms: 0,
                ast_utilization: 0.0,
                generation_quality: 0.0,
                language_specific: HashMap::new(),
            },
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn is_success(&self) -> bool {
        !self.has_errors() && self.metadata.generation_quality >= 0.7
    }
}

impl IndentStyle {
    pub fn to_string(&self, level: usize) -> String {
        match self {
            IndentStyle::Spaces(size) => " ".repeat(size * level),
            IndentStyle::Tabs => "\t".repeat(level),
        }
    }
}

impl LineEnding {
    pub fn as_str(&self) -> &'static str {
        match self {
            LineEnding::Unix => "\n",
            LineEnding::Windows => "\r\n",
            LineEnding::Mac => "\r",
        }
    }
}
