use metaforge_engine::database::Database;
use metaforge_engine::core::*;
use metaforge_engine::parser::extractors::PythonExtractor;
use metaforge_engine::parser::extraction_context::LanguageExtractor;
use anyhow::Result;
use uuid::Uuid;
use serde_json::json;

/// Test helper to get test database URL
fn test_database_url() -> String {
    std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://metaforge_user:metaforge_pass@localhost/metaforge".to_string())
}

#[tokio::test]
async fn test_migrations_table_has_required_columns() -> Result<()> {
    // Setup test database with migrations auto-applied
    let db = Database::setup(&test_database_url()).await?;
    
    // Test for repo_url column existence
    let query = sqlx::query(
        "SELECT column_name FROM information_schema.columns 
         WHERE table_name = 'migrations' AND column_name = 'repo_url'"
    )
    .fetch_optional(db.pool())
    .await?;
    
    assert!(query.is_some(), "repo_url column should exist");
    
    // Test for commit_hash column
    let query = sqlx::query(
        "SELECT column_name FROM information_schema.columns 
         WHERE table_name = 'migrations' AND column_name = 'commit_hash'"
    )
    .fetch_optional(db.pool())
    .await?;
    
    assert!(query.is_some(), "commit_hash column should exist");
    
    // Test for repo_name column
    let query = sqlx::query(
        "SELECT column_name FROM information_schema.columns 
         WHERE table_name = 'migrations' AND column_name = 'repo_name'"
    )
    .fetch_optional(db.pool())
    .await?;
    
    assert!(query.is_some(), "repo_name column should exist");
    
    Ok(())
}

#[tokio::test]
async fn test_can_insert_migration_with_repo_url() -> Result<()> {
    let db = Database::setup(&test_database_url()).await?;
    
    // This should work after migration is applied
    let migration_id = sqlx::query!(
        "INSERT INTO migrations (id, repository_name, repo_name, repo_url, commit_hash, status) 
         VALUES ($1, $2, $3, $4, $5, $6) 
         RETURNING id",
        Uuid::new_v4(),
        "test_migration",
        "test-repo",
        "https://github.com/test/repo.git",
        "abc123def",
        "completed"
    )
    .fetch_one(db.pool())
    .await?;
    
    assert!(!migration_id.id.is_nil());
    Ok(())
}

#[tokio::test]
async fn test_repo_url_index_exists() -> Result<()> {
    let db = Database::setup(&test_database_url()).await?;
    
    let index = sqlx::query!(
        "SELECT indexname FROM pg_indexes 
         WHERE tablename = 'migrations' 
         AND indexname = 'idx_migrations_repo_url'"
    )
    .fetch_optional(db.pool())
    .await?;
    
    assert!(index.is_some(), "Index on repo_url should exist");
    Ok(())
}

#[tokio::test]
async fn test_migration_status_constraint() -> Result<()> {
    let db = Database::setup(&test_database_url()).await?;
    
    // Test valid status values
    let valid_statuses = vec!["pending", "in_progress", "completed", "failed", "rolled_back"];
    
    for status in valid_statuses {
        let result = sqlx::query!(
            "INSERT INTO migrations (id, repository_name, repo_name, repo_url, commit_hash, status) 
             VALUES ($1, $2, $3, $4, $5, $6)",
            Uuid::new_v4(),
            format!("test_migration_{}", status),
            "test-repo",
            "https://github.com/test/repo.git",
            "abc123def",
            status
        )
        .execute(db.pool())
        .await;
        
        assert!(result.is_ok(), "Should accept valid status: {}", status);
    }
    
    // Test invalid status value
    let result = sqlx::query!(
        "INSERT INTO migrations (id, repository_name, repo_name, repo_url, commit_hash, status) 
         VALUES ($1, $2, $3, $4, $5, $6)",
        Uuid::new_v4(),
        "test_migration_invalid",
        "test-repo",
        "https://github.com/test/repo.git",
        "abc123def",
        "invalid_status"
    )
    .execute(db.pool())
    .await;
    
    assert!(result.is_err(), "Should reject invalid status value");
    Ok(())
}

#[tokio::test]
async fn test_migrated_at_default_value() -> Result<()> {
    let db = Database::setup(&test_database_url()).await?;
    
    let migration_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO migrations (id, repository_name, repo_name, repo_url, commit_hash, status) 
         VALUES ($1, $2, $3, $4, $5, $6)",
        migration_id,
        "test_migration",
        "test-repo",
        "https://github.com/test/repo.git",
        "abc123def",
        "completed"
    )
    .execute(db.pool())
    .await?;
    
    let migration = sqlx::query!(
        "SELECT migrated_at FROM migrations WHERE id = $1",
        migration_id
    )
    .fetch_one(db.pool())
    .await?;
    
    assert!(migration.migrated_at.is_some(), "migrated_at should have default value");
    Ok(())
}

#[tokio::test]
async fn test_python_extractor_preserves_implementation() -> Result<()> {
    let extractor = PythonExtractor;
    
    // Test Python function with implementation details
    let python_code = r#"
def calculate_fibonacci(n):
    if n <= 1:
        return n
    else:
        return calculate_fibonacci(n-1) + calculate_fibonacci(n-2)
"#;
    
    // Parse with tree-sitter
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(tree_sitter_python::language()).unwrap();
    let tree = parser.parse(python_code, None).unwrap();
    
    // Extract blocks
    let result = extractor.extract_with_context(tree.root_node(), python_code, "test.py")?;
    
    // Should have extracted function block
    assert!(!result.blocks.is_empty(), "Should extract at least one block");
    
    let function_block = &result.blocks[0];
    assert_eq!(function_block.block_type, BlockType::Function);
    assert_eq!(function_block.semantic_identity.canonical_name, "calculate_fibonacci");
    
    // ❌ FAILING TEST: Should preserve implementation details in normalized_ast
    let implementation = function_block.syntax_preservation.normalized_ast
        .get("implementation")
        .expect("Should have implementation details preserved");
    
    // Should preserve original function body
    let original_body = implementation.get("original_body")
        .expect("Should preserve original body")
        .as_str()
        .expect("Original body should be string");
    
    assert!(original_body.contains("if n <= 1:"), "Should preserve conditional logic");
    assert!(original_body.contains("return n"), "Should preserve return statements");
    assert!(original_body.contains("calculate_fibonacci(n-1)"), "Should preserve recursive calls");
    
    Ok(())
}
