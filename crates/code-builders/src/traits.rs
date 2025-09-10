use anyhow::Result;
use semantic_mapper::CodeComponent;
use crate::{BuildConfig, BuildResult};

/// Core trait for language-specific code builders
pub trait CodeBuilder: Send + Sync {
    /// Build code from semantic components
    fn build_from_components(
        &self,
        components: Vec<CodeComponent>,
        config: &BuildConfig,
    ) -> Result<BuildResult>;
    
    /// Get the language this builder supports
    fn language(&self) -> &'static str;
    
    /// Check if this builder can handle the given component type
    fn supports_component(&self, component: &CodeComponent) -> bool;
    
    /// Validate that all required data is present for generation
    fn validate_components(&self, components: &[CodeComponent]) -> Result<()>;
}

/// Trait for language-specific code formatting
pub trait LanguageFormatter: Send + Sync {
    /// Format generated code according to language conventions
    fn format(&self, code: &str, config: &BuildConfig) -> Result<String>;
    
    /// Get the language this formatter supports
    fn language(&self) -> &'static str;
    
    /// Check if the code is already formatted
    fn is_formatted(&self, code: &str) -> bool;
}
