pub mod universal;
pub mod extractors;
pub mod extraction_context;

// Re-export core types for external use
#[allow(unused_imports)]
pub use extraction_context::{ExtractionContext, ParseResult, BlockRelationship, RelationshipType, LanguageExtractor};

// pub use universal::UniversalParser;
