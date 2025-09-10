/// Test configuration and utilities for the Block Migrate test suite
/// 
/// This module provides common test utilities, fixtures, and configuration
/// that can be shared across all test modules.

use anyhow::Result;
use uuid::Uuid;
use std::sync::Once;
use std::collections::HashMap;

static INIT: Once = Once::new();

/// Initialize test environment (logging, etc.)
pub fn init_test_env() {
    INIT.call_once(|| {
        // Initialize test logging
        let _ = tracing_subscriber::fmt()
            .with_env_filter("debug")
            .with_test_writer()
            .try_init();
    });
}

/// Test database configuration
pub struct TestDatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub timeout_seconds: u64,
}

impl Default for TestDatabaseConfig {
    fn default() -> Self {
        Self {
            url: std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://metaforge_user:metaforge_pass@localhost/metaforge_test".to_string()),
            max_connections: 5,
            timeout_seconds: 30,
        }
    }
}

/// Test fixtures for creating consistent test data
pub struct TestFixtures;

impl TestFixtures {
    /// Create a test semantic block with default values
    pub fn create_test_semantic_block(name: &str) -> metaforge_engine::core::SemanticBlock {
        metaforge_engine::core::SemanticBlock {
            id: Uuid::new_v4(),
            block_type: metaforge_engine::core::BlockType::Function,
            semantic_identity: metaforge_engine::core::SemanticIdentity {
                canonical_name: name.to_string(),
                aliases: vec![],
                fully_qualified_name: Some(format!("test.{}", name)),
                signature_hash: blake3::hash(name.as_bytes()).to_string(),
            },
            syntax_preservation: metaforge_engine::core::SyntaxPreservation {
                original_text: format!("def {}():\n    pass", name),
                normalized_ast: serde_json::json!({
                    "type": "function",
                    "name": name
                }),
                reconstruction_hints: metaforge_engine::core::ReconstructionHints {
                    prefer_original: true,
                    template: None,
                    parameter_positions: vec![],
                    body_extraction: metaforge_engine::core::BodyExtraction {
                        method: "indent".to_string(),
                        start_marker: ":".to_string(),
                        preserve_indentation: true,
                    },
                },
                formatting_preserved: metaforge_engine::core::FormattingInfo {
                    indentation: "    ".to_string(),
                    line_endings: "\n".to_string(),
                    spacing_style: "standard".to_string(),
                },
            },
            structural_context: metaforge_engine::core::StructuralContext {
                parent_block: None,
                child_blocks: vec![],
                sibling_blocks: vec![],
                inheritance_chain: vec![],
                composition_relationships: vec![],
                decorator_chain: vec![],
            },
            semantic_metadata: metaforge_engine::core::SemanticMetadata {
                parameters: Some(serde_json::json!([])),
                return_type: Some("None".to_string()),
                modifiers: vec![],
                annotations: vec![],
                complexity_metrics: Some(serde_json::json!({
                    "cyclomatic_complexity": 1,
                    "cognitive_complexity": 1,
                    "lines_of_code": 2
                })),
                side_effects: vec![],
                preconditions: vec![],
                postconditions: vec![],
                invariants: vec![],
            },
            position: metaforge_engine::core::BlockPosition {
                start_line: 1,
                end_line: 2,
                start_column: 0,
                end_column: 8,
                byte_offset: 0,
                byte_length: format!("def {}():\n    pass", name).len(),
            },
            source_language: "python".to_string(),
        }
    }

    /// Create a test container with default values
    pub fn create_test_container(name: &str) -> metaforge_engine::database::Container {
        metaforge_engine::database::Container {
            id: Uuid::new_v4(),
            name: name.to_string(),
            container_type: "code".to_string(),
            language: Some("python".to_string()),
            original_path: Some(format!("test/{}", name)),
            original_hash: Some(blake3::hash(name.as_bytes()).to_string()),
            source_code: Some(format!("# Test file: {}\n\ndef test_function():\n    pass", name)),
            version: 1,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    /// Create a test intent for AI operations
    pub fn create_test_intent(description: &str) -> metaforge_engine::ai_operations::intent_processor::Intent {
        metaforge_engine::ai_operations::intent_processor::Intent {
            description: description.to_string(),
            context: metaforge_engine::ai_operations::intent_processor::IntentContext {
                target_blocks: vec![],
                current_language: "python".to_string(),
                project_type: "web_app".to_string(),
                existing_patterns: vec!["MVC".to_string()],
                performance_requirements: None,
                security_requirements: None,
            },
            priority: metaforge_engine::ai_operations::intent_processor::IntentPriority::Medium,
            constraints: vec![],
        }
    }

    /// Create a test specification
    pub fn create_test_specification(name: &str) -> metaforge_engine::synthesis::specification_parser::CodeSpecification {
        metaforge_engine::synthesis::specification_parser::CodeSpecification {
            id: Uuid::new_v4(),
            spec_type: metaforge_engine::synthesis::specification_parser::SpecificationType::FunctionSpec,
            content: serde_json::json!({
                "name": name,
                "parameters": [],
                "return_type": "void",
                "documentation": format!("Test function: {}", name)
            }),
            metadata: HashMap::new(),
        }
    }

    /// Create test dependency relationships
    pub fn create_test_dependency(
        source_id: Uuid,
        target_id: Uuid,
        dep_type: &str
    ) -> metaforge_engine::database::schema::BlockRelationship {
        metaforge_engine::database::schema::BlockRelationship {
            source_block_id: source_id,
            target_block_id: target_id,
            relationship_type: dep_type.to_string(),
            metadata: Some(serde_json::json!({
                "created_by": "test",
                "confidence": 1.0
            })),
        }
    }
}

/// Test assertions and utilities
pub struct TestAssertions;

impl TestAssertions {
    /// Assert that two semantic blocks are equivalent (ignoring IDs and timestamps)
    pub fn assert_blocks_equivalent(
        block1: &metaforge_engine::core::SemanticBlock,
        block2: &metaforge_engine::core::SemanticBlock
    ) {
        assert_eq!(block1.block_type, block2.block_type);
        assert_eq!(block1.semantic_identity.canonical_name, block2.semantic_identity.canonical_name);
        assert_eq!(block1.source_language, block2.source_language);
        assert_eq!(block1.syntax_preservation.original_text, block2.syntax_preservation.original_text);
    }

    /// Assert that a dependency graph is valid (no self-loops, etc.)
    pub fn assert_dependency_graph_valid(
        graph: &metaforge_engine::analysis::dependency_analyzer::DependencyGraph
    ) {
        // Check for self-dependencies (should not exist)
        for dep in &graph.dependencies {
            assert_ne!(dep.source_id, dep.target_id, "Dependency graph should not contain self-loops");
        }

        // Check that circular dependencies are properly detected
        for cycle in &graph.circular_dependencies {
            assert!(cycle.len() >= 2, "Circular dependency should involve at least 2 blocks");
        }

        // Check that dependency layers are properly ordered
        for (i, layer) in graph.dependency_layers.iter().enumerate() {
            assert!(!layer.is_empty(), "Dependency layer {} should not be empty", i);
        }
    }

    /// Assert that quality metrics are within reasonable bounds
    pub fn assert_quality_metrics_valid(
        metrics: &metaforge_engine::synthesis::implementation_generator::QualityMetrics
    ) {
        assert!(metrics.completeness_score >= 0.0 && metrics.completeness_score <= 1.0,
                "Completeness score should be between 0 and 1");
        assert!(metrics.correctness_score >= 0.0 && metrics.correctness_score <= 1.0,
                "Correctness score should be between 0 and 1");
        assert!(metrics.maintainability_score >= 0.0 && metrics.maintainability_score <= 1.0,
                "Maintainability score should be between 0 and 1");
        assert!(metrics.performance_score >= 0.0 && metrics.performance_score <= 1.0,
                "Performance score should be between 0 and 1");
    }

    /// Assert that generated code contains expected elements
    pub fn assert_generated_code_valid(code: &str, language: &str, expected_name: &str) {
        assert!(!code.is_empty(), "Generated code should not be empty");
        assert!(code.contains(expected_name), "Generated code should contain the expected name");
        
        match language {
            "python" => {
                assert!(code.contains("def ") || code.contains("class "), 
                        "Python code should contain function or class definitions");
            }
            "typescript" | "javascript" => {
                assert!(code.contains("function ") || code.contains("class "), 
                        "TypeScript/JavaScript code should contain function or class definitions");
            }
            "rust" => {
                assert!(code.contains("fn ") || code.contains("struct "), 
                        "Rust code should contain function or struct definitions");
            }
            _ => {
                // Generic assertion for unknown languages
                assert!(code.len() > 10, "Generated code should be substantial");
            }
        }
    }
}

/// Performance test utilities
pub struct PerformanceTestUtils;

impl PerformanceTestUtils {
    /// Measure execution time of a function
    pub fn measure_time<F, R>(f: F) -> (R, std::time::Duration)
    where
        F: FnOnce() -> R,
    {
        let start = std::time::Instant::now();
        let result = f();
        let duration = start.elapsed();
        (result, duration)
    }

    /// Measure async execution time
    pub async fn measure_time_async<F, Fut, R>(f: F) -> (R, std::time::Duration)
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let start = std::time::Instant::now();
        let result = f().await;
        let duration = start.elapsed();
        (result, duration)
    }

    /// Assert that an operation completes within a time limit
    pub fn assert_within_time_limit<F, R>(f: F, limit: std::time::Duration, operation_name: &str) -> R
    where
        F: FnOnce() -> R,
    {
        let (result, duration) = Self::measure_time(f);
        assert!(duration <= limit, 
                "{} took {:?}, which exceeds the limit of {:?}", 
                operation_name, duration, limit);
        result
    }

    /// Assert that an async operation completes within a time limit
    pub async fn assert_within_time_limit_async<F, Fut, R>(
        f: F, 
        limit: std::time::Duration, 
        operation_name: &str
    ) -> R
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let (result, duration) = Self::measure_time_async(f).await;
        assert!(duration <= limit, 
                "{} took {:?}, which exceeds the limit of {:?}", 
                operation_name, duration, limit);
        result
    }
}

/// Mock implementations for testing
pub struct MockImplementations;

impl MockImplementations {
    /// Create a mock database that doesn't require actual PostgreSQL
    pub fn create_mock_database() -> MockDatabase {
        MockDatabase::new()
    }
}

/// Mock database for testing without requiring actual PostgreSQL
pub struct MockDatabase {
    pub containers: HashMap<Uuid, metaforge_engine::database::Container>,
    pub blocks: HashMap<Uuid, metaforge_engine::core::SemanticBlock>,
    pub relationships: Vec<metaforge_engine::database::schema::BlockRelationship>,
    pub migrations: HashMap<Uuid, MockMigration>,
}

#[derive(Debug, Clone)]
pub struct MockMigration {
    pub id: Uuid,
    pub repository_url: String,
    pub repository_name: String,
    pub commit_hash: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl MockDatabase {
    pub fn new() -> Self {
        Self {
            containers: HashMap::new(),
            blocks: HashMap::new(),
            relationships: Vec::new(),
            migrations: HashMap::new(),
        }
    }

    pub fn insert_container(&mut self, container: metaforge_engine::database::Container) {
        self.containers.insert(container.id, container);
    }

    pub fn insert_block(&mut self, block: metaforge_engine::core::SemanticBlock) {
        self.blocks.insert(block.id, block);
    }

    pub fn insert_relationship(&mut self, relationship: metaforge_engine::database::schema::BlockRelationship) {
        self.relationships.push(relationship);
    }

    pub fn get_block(&self, id: Uuid) -> Option<&metaforge_engine::core::SemanticBlock> {
        self.blocks.get(&id)
    }

    pub fn get_container(&self, id: Uuid) -> Option<&metaforge_engine::database::Container> {
        self.containers.get(&id)
    }

    pub fn create_migration(&mut self, url: &str, name: &str, hash: &str) -> Uuid {
        let id = Uuid::new_v4();
        let migration = MockMigration {
            id,
            repository_url: url.to_string(),
            repository_name: name.to_string(),
            commit_hash: hash.to_string(),
            created_at: chrono::Utc::now(),
        };
        self.migrations.insert(id, migration);
        id
    }
}

/// Test data generators for property-based testing
pub struct TestDataGenerators;

impl TestDataGenerators {
    /// Generate random valid function names
    pub fn generate_function_name() -> String {
        let prefixes = ["test", "handle", "process", "create", "update", "delete", "get", "set"];
        let suffixes = ["data", "request", "response", "item", "user", "config", "result"];
        
        let prefix = prefixes[fastrand::usize(..prefixes.len())];
        let suffix = suffixes[fastrand::usize(..suffixes.len())];
        
        format!("{}_{}", prefix, suffix)
    }

    /// Generate random valid class names
    pub fn generate_class_name() -> String {
        let prefixes = ["Test", "Mock", "Base", "Abstract", "Default"];
        let suffixes = ["Handler", "Manager", "Service", "Controller", "Repository", "Factory"];
        
        let prefix = prefixes[fastrand::usize(..prefixes.len())];
        let suffix = suffixes[fastrand::usize(..suffixes.len())];
        
        format!("{}{}", prefix, suffix)
    }

    /// Generate random programming languages
    pub fn generate_language() -> String {
        let languages = ["python", "typescript", "javascript", "rust", "java", "go"];
        languages[fastrand::usize(..languages.len())].to_string()
    }

    /// Generate random complexity metrics
    pub fn generate_complexity_metrics() -> serde_json::Value {
        serde_json::json!({
            "cyclomatic_complexity": fastrand::u32(1..=20),
            "cognitive_complexity": fastrand::u32(1..=25),
            "lines_of_code": fastrand::u32(1..=100),
            "parameter_count": fastrand::u32(0..=10)
        })
    }
}

