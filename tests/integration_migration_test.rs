use metaforge_engine::database::Database;
use anyhow::Result;
use uuid::Uuid;

/// Test helper to get test database URL
fn test_database_url() -> String {
    std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://metaforge_user:metaforge_pass@localhost/metaforge".to_string())
}

#[tokio::test]
async fn test_complete_repository_migration_flow() -> Result<()> {
    let db = Database::setup(&test_database_url()).await?;
    
    // Test creating migration with repo info (simulating the main.rs flow)
    let repo_url = "https://github.com/rust-lang/rustlings.git";
    let repo_name = "rustlings";
    let commit_hash = "2af9e89b";
    
    // Create migration using the Database method
    let migration_id = db.create_migration(repo_url, repo_name, commit_hash).await?;
    
    // Verify migration record was created correctly
    let migration = sqlx::query!(
        "SELECT id, repository_name, repo_name, repo_url, commit_hash, status, migrated_at 
         FROM migrations WHERE id = $1",
        migration_id
    )
    .fetch_one(db.pool())
    .await?;
    
    assert_eq!(migration.id, migration_id);
    assert_eq!(migration.repository_name, repo_name);
    assert_eq!(migration.repo_name, repo_name);
    assert_eq!(migration.repo_url, repo_url);
    assert_eq!(migration.commit_hash.as_deref(), Some(commit_hash));
    assert_eq!(migration.status.as_str(), "in_progress");
    assert!(migration.migrated_at.is_some());
    
    Ok(())
}

#[tokio::test]
async fn test_migration_status_updates() -> Result<()> {
    let db = Database::setup(&test_database_url()).await?;
    
    // Create a migration
    let repo_url = "https://github.com/test/repo.git";
    let repo_name = "test-repo";
    let commit_hash = "abc123def";
    
    let migration_id = db.create_migration(repo_url, repo_name, commit_hash).await?;
    
    // Test updating migration status to completed
    sqlx::query!(
        "UPDATE migrations SET status = $2, migrated_at = CURRENT_TIMESTAMP WHERE id = $1",
        migration_id,
        "completed"
    )
    .execute(db.pool())
    .await?;
    
    // Verify status was updated
    let updated_migration = sqlx::query!(
        "SELECT status FROM migrations WHERE id = $1",
        migration_id
    )
    .fetch_one(db.pool())
    .await?;
    
    assert_eq!(updated_migration.status.as_str(), "completed");
    
    Ok(())
}

#[tokio::test]
async fn test_migration_query_by_repo_name() -> Result<()> {
    let db = Database::setup(&test_database_url()).await?;
    
    // Use unique repo names to avoid conflicts with other tests
    let test_suffix = format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
    let repos = vec![
        (format!("https://github.com/rust-lang/rustlings-{}.git", test_suffix), format!("rustlings-{}", test_suffix), "commit1"),
        (format!("https://github.com/tokio-rs/tokio-{}.git", test_suffix), format!("tokio-{}", test_suffix), "commit2"),
        (format!("https://github.com/serde-rs/serde-{}.git", test_suffix), format!("serde-{}", test_suffix), "commit3"),
    ];
    
    let mut migration_ids = Vec::new();
    for (url, name, commit) in &repos {
        let id = db.create_migration(url, name, commit).await?;
        migration_ids.push(id);
    }
    
    // Test querying by repo_name
    let rustlings_name = format!("rustlings-{}", test_suffix);
    let rustlings_migrations = sqlx::query!(
        "SELECT id, repo_name, repo_url FROM migrations WHERE repo_name = $1",
        rustlings_name
    )
    .fetch_all(db.pool())
    .await?;
    
    assert_eq!(rustlings_migrations.len(), 1);
    assert_eq!(rustlings_migrations[0].repo_name, rustlings_name);
    assert_eq!(rustlings_migrations[0].repo_url, format!("https://github.com/rust-lang/rustlings-{}.git", test_suffix));
    
    // Test querying all migrations we just created
    let all_migrations = sqlx::query!(
        "SELECT repo_name FROM migrations WHERE id = ANY($1)",
        &migration_ids
    )
    .fetch_all(db.pool())
    .await?;
    
    assert_eq!(all_migrations.len(), 3);
    
    Ok(())
}

#[tokio::test]
async fn test_migration_indexes_performance() -> Result<()> {
    let db = Database::setup(&test_database_url()).await?;
    
    // Verify that our indexes exist and can be used
    let indexes = sqlx::query!(
        "SELECT indexname FROM pg_indexes 
         WHERE tablename = 'migrations' 
         AND indexname IN ('idx_migrations_repo_name', 'idx_migrations_repo_url', 'idx_migrations_status')"
    )
    .fetch_all(db.pool())
    .await?;
    
    assert!(indexes.len() >= 3, "Expected at least 3 indexes on migrations table");
    
    // Test that queries can use the indexes (this would show up in EXPLAIN ANALYZE)
    let test_suffix = format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
    let repo_name = format!("performance-test-{}", test_suffix);
    
    let migration_id = db.create_migration(
        &format!("https://github.com/test/performance-{}.git", test_suffix),
        &repo_name,
        "perf123"
    ).await?;
    
    // Query by repo_name (should use idx_migrations_repo_name)
    let result = sqlx::query!(
        "SELECT id FROM migrations WHERE repo_name = $1",
        repo_name
    )
    .fetch_optional(db.pool())
    .await?;
    
    assert!(result.is_some());
    assert_eq!(result.unwrap().id, migration_id);
    
    Ok(())
}

#[tokio::test]
async fn test_migration_constraint_validation() -> Result<()> {
    let db = Database::setup(&test_database_url()).await?;
    
    // Test that status constraint works
    let migration_id = Uuid::new_v4();
    
    // Valid status should work
    let result = sqlx::query!(
        "INSERT INTO migrations (id, repository_name, repo_name, repo_url, commit_hash, status) 
         VALUES ($1, $2, $3, $4, $5, $6)",
        migration_id,
        "test-repo",
        "test-repo",
        "https://github.com/test/repo.git",
        "abc123",
        "pending"
    )
    .execute(db.pool())
    .await;
    
    assert!(result.is_ok(), "Valid status should be accepted");
    
    // Invalid status should fail
    let invalid_migration_id = Uuid::new_v4();
    let result = sqlx::query!(
        "INSERT INTO migrations (id, repository_name, repo_name, repo_url, commit_hash, status) 
         VALUES ($1, $2, $3, $4, $5, $6)",
        invalid_migration_id,
        "test-repo-2",
        "test-repo-2",
        "https://github.com/test/repo2.git",
        "def456",
        "invalid_status"
    )
    .execute(db.pool())
    .await;
    
    assert!(result.is_err(), "Invalid status should be rejected");
    
    Ok(())
}
