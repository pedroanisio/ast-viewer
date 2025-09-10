pub mod semantic_vcs;
pub mod semantic_diff;
pub mod semantic_merge;
pub mod semantic_version_control;
pub mod llm_provider_manager;
pub mod semantic_merge_handler;
pub mod llm_context_manager;

// Explicit re-exports to avoid ambiguity
pub use semantic_version_control::SemanticVersionControl; 
pub use llm_provider_manager::*;
pub use semantic_merge_handler::SemanticMergeHandler;
