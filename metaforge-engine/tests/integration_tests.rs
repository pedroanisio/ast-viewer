use anyhow::Result;
use uuid::Uuid;
use std::collections::HashMap;
use metaforge_engine::{
    database::Database,
    core::SemanticBlock,
    versioning::semantic_vcs::{SemanticVCS, SemanticChangeType},
    analysis::dependency_analyzer::{DependencyAnalyzer, DependencyType},
    ai_operations::intent_processor::{IntentProcessor, Intent, IntentContext, IntentPriority},
    synthesis::{
        specification_parser::{SpecificationParser, CodeSpecification, SpecificationType},
        implementation_generator::{ImplementationGenerator, ImplementationRequest},
    },
};

/// Integration tests for the Block Migrate system
/// 
/// These tests verify that all major components work together correctly
/// and that the complete workflow from repository migration to code generation functions properly.

#[tokio::test]
async fn test_complete_migration_workflow() -> Result<()> {
    // Setup test database
    let db = setup_test_database().await?;
    
    // Test 1: Database initialization
    db.initialize_schema().await?;
    
    // Test 2: Create a test migration
    let migration_id = db.create_migration(
        "https://github.com/test/repo.git",
        "test-repo",
        "abc123def456"
    ).await?;
    
    // Test 3: Create test containers and blocks
    let container_id = create_test_container(&db, migration_id).await?;
    let block_id = create_test_semantic_block(&db, container_id).await?;
    
    // Test 4: Verify block retrieval
    let retrieved_block = db.get_block_by_id(block_id).await?;
    assert_eq!(retrieved_block.id, block_id);
    assert_eq!(retrieved_block.semantic_identity.canonical_name, "test_function");
    
    // Test 5: Create relationships
    let relationship = metaforge_engine::database::schema::BlockRelationship {
        source_block_id: block_id,
        target_block_id: block_id, // Self-reference for testing
        relationship_type: "calls".to_string(),
        metadata: Some(serde_json::json!({"test": true})),
    };
    db.insert_relationship(&relationship).await?;
    
    println!("✅ Complete migration workflow test passed");
    Ok(())
}

#[tokio::test]
async fn test_semantic_version_control() -> Result<()> {
    let db = setup_test_database().await?;
    db.initialize_schema().await?;
    
    let repository_id = Uuid::new_v4();
    let vcs = SemanticVCS::new(db.clone(), repository_id);
    
    // Test semantic commit creation
    let commit = vcs.commit(
        "Add test function".to_string(),
        "Test Author".to_string(),
        vec![], // changes
        vec![], // parent commits
    ).await?;
    
    assert!(!commit.message.is_empty());
    assert_eq!(commit.author, "Test Author");
    
    // Test evolution analysis
    let evolution = vcs.analyze_evolution().await?;
    assert_eq!(evolution.total_commits, 1);
    
    println!("✅ Semantic version control test passed");
    Ok(())
}

#[tokio::test]
async fn test_dependency_analysis() -> Result<()> {
    let db = setup_test_database().await?;
    db.initialize_schema().await?;
    
    let analyzer = DependencyAnalyzer::new(db.clone());
    
    // Create test blocks with dependencies
    let migration_id = db.create_migration("test", "test", "test").await?;
    let container_id = create_test_container(&db, migration_id).await?;
    let block1_id = create_test_semantic_block(&db, container_id).await?;
    let block2_id = create_test_semantic_block(&db, container_id).await?;
    
    // Create dependency relationship
    let relationship = metaforge_engine::database::schema::BlockRelationship {
        source_block_id: block1_id,
        target_block_id: block2_id,
        relationship_type: "calls".to_string(),
        metadata: None,
    };
    db.insert_relationship(&relationship).await?;
    
    // Test dependency analysis
    let dependency_graph = analyzer.analyze_dependencies(&[block1_id, block2_id]).await?;
    assert!(!dependency_graph.dependencies.is_empty());
    
    // Test circular dependency detection
    let circular_deps = analyzer.detect_circular_dependencies(&dependency_graph.dependencies).await?;
    assert!(circular_deps.is_empty()); // No circular dependencies in this simple case
    
    // Test dependency metrics
    let metrics = analyzer.calculate_dependency_metrics(block1_id).await?;
    assert_eq!(metrics.efferent_coupling, 1); // block1 depends on block2
    
    println!("✅ Dependency analysis test passed");
    Ok(())
}

#[tokio::test]
async fn test_intent_processing() -> Result<()> {
    let db = setup_test_database().await?;
    db.initialize_schema().await?;
    
    let processor = IntentProcessor::new(db.clone());
    
    // Create test intent
    let intent = Intent {
        description: "Create a new authentication function".to_string(),
        context: IntentContext {
            target_blocks: vec![],
            current_language: "python".to_string(),
            project_type: "web_app".to_string(),
            existing_patterns: vec!["MVC".to_string()],
            performance_requirements: None,
            security_requirements: None,
        },
        priority: IntentPriority::High,
        constraints: vec!["must be secure".to_string()],
    };
    
    // Test intent processing
    let plan = processor.process_intent(&intent).await?;
    assert!(!plan.operations.is_empty());
    assert!(plan.estimated_complexity.time_estimate_hours > 0.0);
    
    println!("✅ Intent processing test passed");
    Ok(())
}

#[tokio::test]
async fn test_code_synthesis() -> Result<()> {
    let db = setup_test_database().await?;
    db.initialize_schema().await?;
    
    // Test specification parsing
    let parser = SpecificationParser::new(db.clone());
    let spec = CodeSpecification {
        id: Uuid::new_v4(),
        spec_type: SpecificationType::FunctionSpec,
        content: serde_json::json!({
            "name": "test_function",
            "parameters": ["param1", "param2"],
            "return_type": "string",
            "documentation": "A test function"
        }),
        metadata: HashMap::new(),
    };
    
    let parsed = parser.parse_specification(spec.clone()).await?;
    assert!(!parsed.parsed_elements.is_empty());
    assert_eq!(parsed.parsed_elements[0].name, "test_function");
    
    // Test implementation generation
    let generator = ImplementationGenerator::new(db.clone());
    let request = ImplementationRequest {
        id: Uuid::new_v4(),
        specification_id: spec.id,
        target_language: "python".to_string(),
        generation_options: HashMap::new(),
    };
    
    let implementation = generator.generate_implementation(request).await?;
    assert!(!implementation.code.is_empty());
    assert!(implementation.code.contains("test_function"));
    assert!(implementation.quality_metrics.completeness_score >= 0.0);
    
    println!("✅ Code synthesis test passed");
    Ok(())
}

#[tokio::test]
async fn test_llm_context_management() -> Result<()> {
    let db = setup_test_database().await?;
    db.initialize_schema().await?;
    
    use metaforge_engine::versioning::llm_context_manager::LLMContextManager;
    
    let context_manager = LLMContextManager::new(db.clone());
    let migration_id = db.create_migration("test", "test", "test").await?;
    let container_id = create_test_container(&db, migration_id).await?;
    let block_id = create_test_semantic_block(&db, container_id).await?;
    
    // Test context building
    let context = context_manager.build_context_for_block(block_id, "refactoring").await?;
    assert!(context.block_info.id == block_id);
    
    println!("✅ LLM context management test passed");
    Ok(())
}

#[tokio::test]
async fn test_database_schema_migration() -> Result<()> {
    let db = setup_test_database().await?;
    
    // Test initial schema creation
    db.initialize_schema().await?;
    
    // Test schema migration to hierarchical
    db.migrate_to_hierarchical_schema().await?;
    
    // Verify tables exist by attempting to query them
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM migrations")
        .fetch_one(&db.pool)
        .await?;
    assert!(count >= 0);
    
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM containers")
        .fetch_one(&db.pool)
        .await?;
    assert!(count >= 0);
    
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM blocks")
        .fetch_one(&db.pool)
        .await?;
    assert!(count >= 0);
    
    println!("✅ Database schema migration test passed");
    Ok(())
}

#[tokio::test]
async fn test_round_trip_consistency() -> Result<()> {
    let db = setup_test_database().await?;
    db.initialize_schema().await?;
    
    // Create original semantic block
    let migration_id = db.create_migration("test", "test", "test").await?;
    let container_id = create_test_container(&db, migration_id).await?;
    let original_block = create_test_semantic_block(&db, container_id).await?;
    
    // Retrieve the block
    let retrieved_block = db.get_block_by_id(original_block).await?;
    
    // Verify round-trip consistency
    assert_eq!(retrieved_block.id, original_block);
    assert_eq!(retrieved_block.semantic_identity.canonical_name, "test_function");
    assert_eq!(retrieved_block.source_language, "python");
    
    // Test block update
    let mut updated_block = retrieved_block.clone();
    updated_block.semantic_identity.canonical_name = "updated_test_function".to_string();
    db.update_block(&updated_block).await?;
    
    // Verify update
    let final_block = db.get_block_by_id(original_block).await?;
    assert_eq!(final_block.semantic_identity.canonical_name, "updated_test_function");
    
    println!("✅ Round-trip consistency test passed");
    Ok(())
}

// Helper functions for test setup

async fn setup_test_database() -> Result<Database> {
    // Use a test database URL or in-memory database
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://metaforge_user:metaforge_pass@localhost/metaforge_test".to_string());
    
    let db = Database::new(&database_url).await?;
    
    // Clean up any existing test data
    let _ = db.reset_database().await;
    
    Ok(db)
}

async fn create_test_container(db: &Database, migration_id: Uuid) -> Result<Uuid> {
    let container = metaforge_engine::database::Container {
        id: Uuid::new_v4(),
        name: "test_file.py".to_string(),
        container_type: "code".to_string(),
        language: Some("python".to_string()),
        original_path: Some("test/test_file.py".to_string()),
        original_hash: Some("test_hash".to_string()),
        source_code: Some("def test_function():\n    pass".to_string()),
        version: 1,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    db.insert_container(&container, migration_id).await?;
    Ok(container.id)
}

async fn create_test_semantic_block(db: &Database, container_id: Uuid) -> Result<Uuid> {
    let block = SemanticBlock {
        id: Uuid::new_v4(),
        block_type: metaforge_engine::core::BlockType::Function,
        semantic_identity: metaforge_engine::core::SemanticIdentity {
            canonical_name: "test_function".to_string(),
            aliases: vec![],
            fully_qualified_name: Some("test.test_function".to_string()),
            signature_hash: "test_hash".to_string(),
        },
        syntax_preservation: metaforge_engine::core::SyntaxPreservation {
            original_text: "def test_function():\n    pass".to_string(),
            normalized_ast: serde_json::json!({"type": "function", "name": "test_function"}),
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
            complexity_metrics: Some(serde_json::json!({"cyclomatic_complexity": 1})),
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
            byte_length: 25,
        },
        source_language: "python".to_string(),
    };
    
    db.insert_semantic_block(&block, container_id).await?;
    Ok(block.id)
}

/// Performance benchmarks for critical operations
#[tokio::test]
async fn benchmark_block_insertion() -> Result<()> {
    let db = setup_test_database().await?;
    db.initialize_schema().await?;
    
    let migration_id = db.create_migration("benchmark", "benchmark", "test").await?;
    let container_id = create_test_container(&db, migration_id).await?;
    
    let start_time = std::time::Instant::now();
    let mut block_ids = Vec::new();
    
    // Insert 100 blocks
    for i in 0..100 {
        let block_id = create_test_semantic_block(&db, container_id).await?;
        block_ids.push(block_id);
    }
    
    let insertion_time = start_time.elapsed();
    println!("⏱️  Inserted 100 blocks in {:?} ({:.2} blocks/sec)", 
             insertion_time, 
             100.0 / insertion_time.as_secs_f64());
    
    // Benchmark retrieval
    let start_time = std::time::Instant::now();
    for block_id in &block_ids {
        let _block = db.get_block_by_id(*block_id).await?;
    }
    let retrieval_time = start_time.elapsed();
    println!("⏱️  Retrieved 100 blocks in {:?} ({:.2} blocks/sec)", 
             retrieval_time, 
             100.0 / retrieval_time.as_secs_f64());
    
    // Performance assertions
    assert!(insertion_time.as_secs() < 10, "Block insertion should complete within 10 seconds");
    assert!(retrieval_time.as_secs() < 5, "Block retrieval should complete within 5 seconds");
    
    println!("✅ Performance benchmark test passed");
    Ok(())
}

/// Test error handling and edge cases
#[tokio::test]
async fn test_error_handling() -> Result<()> {
    let db = setup_test_database().await?;
    db.initialize_schema().await?;
    
    // Test retrieving non-existent block
    let non_existent_id = Uuid::new_v4();
    let result = db.get_block_by_id(non_existent_id).await;
    assert!(result.is_err(), "Should fail when retrieving non-existent block");
    
    // Test invalid migration creation
    let result = db.create_migration("", "", "").await;
    assert!(result.is_err(), "Should fail with empty migration parameters");
    
    println!("✅ Error handling test passed");
    Ok(())
}

