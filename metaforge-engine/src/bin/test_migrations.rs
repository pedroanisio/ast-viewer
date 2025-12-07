use metaforge_engine::database::Database;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧪 Testing SQLx migrations...");
    
    // Use test database URL
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://metaforge_user:metaforge_pass@localhost/metaforge_test".to_string());
    
    // Test migration setup
    match Database::setup(&database_url).await {
        Ok(_db) => {
            println!("✅ SQLx migrations completed successfully!");
            println!("✅ Database setup with embedded migrations working!");
        }
        Err(e) => {
            println!("❌ Migration test failed: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}
