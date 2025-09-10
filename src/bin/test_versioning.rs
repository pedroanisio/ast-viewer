use anyhow::Result;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

use metaforge_engine::database::{Database, Block, Container};
use metaforge_engine::versioning::{
    SemanticVersionControl, LLMProviderManager, SemanticMergeHandler,
    semantic_version_control::{LLMConfig, VersioningContext}, 
    llm_context_manager::LLMContextManager
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔄 Testing Semantic Block Versioning System");
    
    // Initialize database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost/block_migrate".to_string());
    
    let db = Database::new(&database_url).await?;
    
    // Initialize versioning schema
    println!("📊 Creating versioning schema...");
    db.create_versioning_schema().await?;
    
    // Create test data
    let migration_id = db.create_migration("test://versioning", "versioning_test", "test_commit").await?;
    
    let container = Container {
        id: Uuid::new_v4(),
        name: "test_versioning_container".to_string(),
        container_type: "file".to_string(),
        language: Some("rust".to_string()),
        original_path: Some("src/lib.rs".to_string()),
        original_hash: Some("test_hash".to_string()),
        source_code: Some("fn calculate(x: i32) -> i32 { x * 2 }".to_string()),
        version: 1,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        semantic_summary: None,
        parsing_metadata: None,
        formatting_preferences: None,
        reconstruction_hints: None,
    };
    
    db.insert_container(&container, migration_id).await?;
    
    let mut test_block = Block {
        id: Uuid::new_v4(),
        container_id: container.id,
        block_type: "Function".to_string(),
        semantic_name: Some("calculate".to_string()),
        abstract_syntax: serde_json::json!({
            "type": "function",
            "name": "calculate",
            "parameters": [{"name": "x", "type": "i32"}],
            "return_type": "i32",
            "body": "x * 2"
        }),
        position: 0,
        indent_level: 0,
        metadata: Some(serde_json::json!({
            "file_path": "src/lib.rs",
            "line_range": [1, 3]
        })),
        created_at: Utc::now(),
        parent_block_id: None,
        position_in_parent: 0,
        parameters: Some(serde_json::json!([{"name": "x", "type": "i32"}])),
        return_type: Some("i32".to_string()),
        modifiers: Some(vec!["pub".to_string()]),
        decorators: None,
        body_ast: None,
        language_ast: None,
        language_features: None,
        complexity_metrics: Some(serde_json::json!({"cyclomatic_complexity": 1})),
        scope_info: None,
        syntax_preservation: None,
        structural_context: None,
        semantic_metadata: None,
        source_language: Some("rust".to_string()),
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
    };
    
    db.insert_block(&test_block).await?;
    
    // Test 1: Semantic Version Control
    println!("\n🧪 Test 1: Creating semantic versions");
    
    let version_control = SemanticVersionControl::new(db.clone());
    
    // Create initial version
    let context1 = VersioningContext {
        changes: HashMap::from([
            ("initial_implementation".to_string(), serde_json::json!(true)),
        ]),
        breaking_change: false,
        change_type: "initial".to_string(),
        change_description: "Initial implementation of calculate function".to_string(),
        branch_name: Some("main".to_string()),
    };
    
    let llm_config = LLMConfig {
        provider: "test".to_string(),
        model: "test-model".to_string(),
        prompt_template_id: None,
        temperature: Some(0.7),
        reasoning: Some("Initial version created for testing".to_string()),
    };
    
    let version1 = version_control.create_version(&test_block, context1, Some(llm_config)).await?;
    println!("✅ Created version 1: {} (semantic hash: {})", version1.version_number, version1.semantic_hash);
    
    // Modify block and create second version
    test_block.abstract_syntax = serde_json::json!({
        "type": "function",
        "name": "calculate",
        "parameters": [{"name": "x", "type": "i32"}],
        "return_type": "i32",
        "body": "x * 3" // Changed multiplier
    });
    
    let context2 = VersioningContext {
        changes: HashMap::from([
            ("multiplier_change".to_string(), serde_json::json!("2 -> 3")),
            ("behavior_change".to_string(), serde_json::json!(true)),
        ]),
        breaking_change: false,
        change_type: "optimization".to_string(),
        change_description: "Changed multiplier from 2 to 3 for better performance".to_string(),
        branch_name: Some("main".to_string()),
    };
    
    let version2 = version_control.create_version(&test_block, context2, None).await?;
    println!("✅ Created version 2: {} (semantic hash: {})", version2.version_number, version2.semantic_hash);
    
    // Test semantic diff
    println!("\n🧪 Test 2: Semantic diff analysis");
    let diff = version_control.semantic_diff(version1.id, version2.id, false).await?;
    println!("✅ Semantic diff completed:");
    println!("   - Behavioral changes: {:?}", diff.behavioral_changes);
    println!("   - Breaking changes: {:?}", diff.breaking_changes);
    
    // Test 3: Version history
    println!("\n🧪 Test 3: Version history");
    let history = version_control.get_version_history(test_block.id).await?;
    println!("✅ Found {} versions in history", history.len());
    for version in &history {
        println!("   - Version {}: {} ({})", 
            version.version_number, 
            version.change_description.as_deref().unwrap_or("No description"),
            version.created_at.format("%Y-%m-%d %H:%M:%S")
        );
    }
    
    // Test 4: Semantic duplicates
    println!("\n🧪 Test 4: Finding semantic duplicates");
    let duplicates = version_control.find_semantic_duplicates(&version1.semantic_hash).await?;
    println!("✅ Found {} blocks with same semantic hash", duplicates.len());
    
    // Test 5: LLM Context Manager
    println!("\n🧪 Test 5: LLM Context Manager");
    let context_manager = LLMContextManager::new(db.clone());
    
    let llm_context = context_manager.build_context_for_block(
        test_block.id,
        true,  // include_history
        true,  // include_dependencies
        false, // include_consumers
        false, // include_patterns
        true,  // include_domain
    ).await?;
    
    println!("✅ Built LLM context:");
    println!("   - Block: {} ({})", 
        llm_context.block_info.semantic_name.as_deref().unwrap_or("unnamed"),
        llm_context.block_info.block_type
    );
    println!("   - Complexity score: {:.2}", llm_context.block_info.complexity_score.unwrap_or(0.0));
    
    if let Some(history) = &llm_context.history {
        println!("   - Version history: {} versions", history.versions.len());
        println!("   - Evolution trend: {:?}", history.evolution_trend);
        println!("   - Stability score: {:.2}", history.stability_score);
    }
    
    if let Some(domain) = &llm_context.domain {
        println!("   - Domain: {}", domain.domain_name);
        println!("   - Business concepts: {:?}", domain.business_concepts);
    }
    
    // Test 6: LLM Provider Manager
    println!("\n🧪 Test 6: LLM Provider Manager");
    let llm_manager = LLMProviderManager::new(db.clone());
    let providers = llm_manager.list_providers();
    println!("✅ Available LLM providers: {:?}", providers);
    
    for provider in &providers {
        if let Some(capabilities) = llm_manager.get_provider_capabilities(provider) {
            println!("   - {}: supports code generation: {}, avg latency: {}ms", 
                provider, 
                capabilities.supports_code_generation,
                capabilities.avg_latency_ms
            );
        }
    }
    
    // Test 7: Rollback
    println!("\n🧪 Test 7: Version rollback");
    let rollback_version = version_control.rollback_to_version(
        test_block.id,
        version1.version_number,
        "Testing rollback functionality".to_string(),
    ).await?;
    
    println!("✅ Rollback completed:");
    println!("   - New version: {}", rollback_version.version_number);
    println!("   - Restored semantic hash: {}", rollback_version.semantic_hash);
    println!("   - Rollback reason: {}", rollback_version.llm_reasoning.as_deref().unwrap_or(""));
    
    println!("\n🎉 All versioning tests completed successfully!");
    println!("\n📊 Summary:");
    println!("   ✅ Semantic version control");
    println!("   ✅ Semantic hashing");
    println!("   ✅ Version history tracking");
    println!("   ✅ LLM integration tracking");
    println!("   ✅ Context management");
    println!("   ✅ Provider management");
    println!("   ✅ Rollback functionality");
    
    Ok(())
}
