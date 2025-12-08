use sqlx::{PgPool, postgres::PgPoolOptions, migrate::Migrator};
use anyhow::Result;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

// Embed migrations at compile time
static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Container {
    pub id: Uuid,
    pub name: String,
    pub container_type: String,
    pub language: Option<String>,
    pub original_path: Option<String>,
    pub original_hash: Option<String>,
    pub source_code: Option<String>,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    
    // Enhanced semantic fields from migration 002
    pub semantic_summary: Option<serde_json::Value>,
    pub parsing_metadata: Option<serde_json::Value>,
    pub formatting_preferences: Option<serde_json::Value>,
    pub reconstruction_hints: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Block {
    pub id: Uuid,
    pub container_id: Uuid,
    pub block_type: String,
    pub semantic_name: Option<String>,
    pub abstract_syntax: serde_json::Value,
    pub position: i32,
    pub indent_level: i32,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    
    // New hierarchical fields
    pub parent_block_id: Option<Uuid>,
    pub position_in_parent: i32,
    pub parameters: Option<serde_json::Value>,
    pub return_type: Option<String>,
    pub modifiers: Option<Vec<String>>,
    pub decorators: Option<serde_json::Value>,
    pub body_ast: Option<serde_json::Value>,
    pub language_ast: Option<serde_json::Value>,
    pub language_features: Option<serde_json::Value>,
    pub complexity_metrics: Option<serde_json::Value>,
    pub scope_info: Option<serde_json::Value>,
    
    // Semantic model fields (Phase 1A.1 alignment)
    pub syntax_preservation: Option<serde_json::Value>,
    pub structural_context: Option<serde_json::Value>,
    pub semantic_metadata: Option<serde_json::Value>,
    pub source_language: Option<String>,
    pub template_metadata: Option<serde_json::Value>,
    pub generation_hints: Option<serde_json::Value>,
    
    // Enhanced semantic fields from migration 002
    pub semantic_signature: Option<serde_json::Value>,
    pub behavioral_contract: Option<serde_json::Value>,
    pub formatting_metadata: Option<serde_json::Value>,
    pub attached_comments: Option<serde_json::Value>,
    pub dependency_info: Option<serde_json::Value>,
    
    // Position and hierarchy enhancements
    pub position_metadata: Option<serde_json::Value>,
    pub hierarchical_index: Option<i32>,
    pub depth_level: Option<i32>,
}

impl Block {
    /// Helper method to set metadata fields safely
    pub fn set_metadata(&mut self, key: &str, value: serde_json::Value) {
        match &mut self.metadata {
            Some(serde_json::Value::Object(map)) => {
                map.insert(key.to_string(), value);
            },
            _ => {
                let mut map = serde_json::Map::new();
                map.insert(key.to_string(), value);
                self.metadata = Some(serde_json::Value::Object(map));
            }
        }
    }
    
    /// Helper method to get metadata fields safely
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.as_ref()?.as_object()?.get(key)
    }
    
    /// Helper method to get mutable metadata reference
    pub fn get_metadata_mut(&mut self, key: &str) -> Option<&mut serde_json::Value> {
        if let Some(serde_json::Value::Object(ref mut map)) = self.metadata {
            map.get_mut(key)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BlockRelationship {
    pub source_block_id: Uuid,
    pub target_block_id: Uuid,
    pub relationship_type: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BlockVersion {
    pub id: Uuid,
    pub block_id: Uuid,
    pub version_number: i32,
    pub semantic_hash: String,
    pub syntax_hash: String,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
    
    // Semantic versioning components
    pub semantic_changes: Option<serde_json::Value>,
    pub breaking_change: bool,
    
    // LLM tracking
    pub llm_provider: Option<String>,
    pub llm_model: Option<String>,
    pub llm_prompt_id: Option<Uuid>,
    pub llm_temperature: Option<f32>,
    pub llm_reasoning: Option<String>,
    
    // Change metadata
    pub change_type: Option<String>,
    pub change_description: Option<String>,
    pub parent_version: Option<Uuid>,
    pub branch_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PromptTemplate {
    pub id: Uuid,
    pub name: String,
    pub category: Option<String>,
    
    // Multi-provider prompts
    pub prompts: serde_json::Value,
    pub variables: Option<serde_json::Value>,
    pub constraints: Option<serde_json::Value>,
    pub examples: Option<serde_json::Value>,
    
    pub version: i32,
    pub effectiveness_score: Option<f32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LLMInteraction {
    pub id: Uuid,
    pub block_version_id: Option<Uuid>,
    pub prompt_template_id: Option<Uuid>,
    
    // Request details
    pub provider: String,
    pub model: String,
    pub request_payload: serde_json::Value,
    
    // Response details
    pub response_payload: serde_json::Value,
    pub tokens_used: Option<i32>,
    pub latency_ms: Option<i32>,
    pub cost_cents: Option<f32>,
    
    // Quality metrics
    pub confidence_score: Option<f32>,
    pub human_rating: Option<i32>,
    pub automated_score: Option<f32>,
    
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SemanticBranch {
    pub id: Uuid,
    pub name: String,
    pub base_migration_id: Option<Uuid>,
    
    // Semantic branching metadata
    pub intent: Option<String>,
    pub constraints: Option<serde_json::Value>,
    pub merge_strategy: Option<String>,
    
    // LLM configuration for this branch
    pub default_llm_provider: Option<String>,
    pub default_llm_model: Option<String>,
    pub default_temperature: Option<f32>,
    
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EnhancedBlockRelationship {
    pub id: Uuid,
    pub source_block_id: Uuid,
    pub target_block_id: Uuid,
    pub relationship_type: String,
    pub relationship_strength: Option<f32>,
    pub bidirectional: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BlockTemplate {
    pub id: Uuid,
    pub name: String,
    pub block_type: String,
    pub language: String,
    pub template_content: serde_json::Value,
    pub variables: Option<serde_json::Value>,
    pub constraints: Option<serde_json::Value>,
    pub examples: Option<serde_json::Value>,
    pub effectiveness_score: Option<f32>,
    pub usage_count: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReconstructionMetadata {
    pub block_id: Uuid,
    pub reconstruction_quality: Option<f32>,
    pub template_id: Option<Uuid>,
    pub reconstruction_hints: Option<serde_json::Value>,
    pub formatting_preferences: Option<serde_json::Value>,
    pub last_reconstructed_at: Option<DateTime<Utc>>,
    pub reconstruction_count: Option<i32>,
    pub validation_errors: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BlockEvolution {
    pub id: Uuid,
    pub block_id: Uuid,
    pub transformation_type: String,
    pub before_snapshot: serde_json::Value,
    pub after_snapshot: serde_json::Value,
    pub transformation_metadata: Option<serde_json::Value>,
    pub applied_by: Option<String>,
    pub applied_at: DateTime<Utc>,
    pub parent_evolution_id: Option<Uuid>,
    pub semantic_diff: Option<serde_json::Value>,
    pub impact_analysis: Option<serde_json::Value>,
    pub rollback_information: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SourceCodeMigrationLog {
    pub id: Uuid,
    pub container_id: Uuid,
    pub migration_status: String,
    pub semantic_extraction_quality: Option<f32>,
    pub original_size_bytes: Option<i32>,
    pub semantic_blocks_count: Option<i32>,
    pub migration_started_at: Option<DateTime<Utc>>,
    pub migration_completed_at: Option<DateTime<Utc>>,
    pub error_messages: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SourceCodeBackup {
    pub container_id: Uuid,
    pub original_source_code: Option<String>,
    pub original_path: Option<String>,
    pub original_hash: Option<String>,
    pub backup_created_at: DateTime<Utc>,
    pub restored: Option<bool>,
}

#[derive(Clone)]
#[derive(Debug)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;
        
        Ok(Self { pool })
    }

    /// Run embedded SQLx migrations
    pub async fn run_migrations(&self) -> Result<()> {
        MIGRATOR.run(&self.pool).await?;
        println!("✅ Database migrations completed successfully");
        Ok(())
    }
    
    /// Setup database with automatic migration
    pub async fn setup(database_url: &str) -> Result<Self> {
        let database = Self::new(database_url).await?;
        database.run_migrations().await?;
        Ok(database)
    }
    
    
    pub async fn create_versioning_schema(&self) -> Result<()> {
        // Create block_versions table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS block_versions (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                block_id UUID NOT NULL REFERENCES blocks(id) ON DELETE CASCADE,
                version_number INTEGER NOT NULL,
                semantic_hash VARCHAR(64) NOT NULL,
                syntax_hash VARCHAR(64) NOT NULL,
                created_at TIMESTAMPTZ DEFAULT NOW(),
                created_by VARCHAR(255),
                
                -- Semantic versioning components
                semantic_changes JSONB,
                breaking_change BOOLEAN DEFAULT FALSE,
                
                -- LLM tracking
                llm_provider VARCHAR(50),
                llm_model VARCHAR(100),
                llm_prompt_id UUID,
                llm_temperature FLOAT,
                llm_reasoning TEXT,
                
                -- Change metadata
                change_type VARCHAR(50),
                change_description TEXT,
                parent_version UUID REFERENCES block_versions(id),
                branch_name VARCHAR(100),
                
                UNIQUE(block_id, version_number)
            )
        "#).execute(&self.pool).await?;
        
        // Create prompt_templates table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS prompt_templates (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                name VARCHAR(255) NOT NULL,
                category VARCHAR(100),
                
                -- Multi-provider prompts
                prompts JSONB NOT NULL,
                variables JSONB,
                constraints JSONB,
                examples JSONB,
                
                version INTEGER DEFAULT 1,
                effectiveness_score FLOAT,
                created_at TIMESTAMPTZ DEFAULT NOW(),
                updated_at TIMESTAMPTZ DEFAULT NOW()
            )
        "#).execute(&self.pool).await?;
        
        // Create llm_interactions table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS llm_interactions (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                block_version_id UUID REFERENCES block_versions(id),
                prompt_template_id UUID REFERENCES prompt_templates(id),
                
                -- Request details
                provider VARCHAR(50) NOT NULL,
                model VARCHAR(100) NOT NULL,
                request_payload JSONB NOT NULL,
                
                -- Response details
                response_payload JSONB NOT NULL,
                tokens_used INTEGER,
                latency_ms INTEGER,
                cost_cents FLOAT,
                
                -- Quality metrics
                confidence_score FLOAT,
                human_rating INTEGER CHECK (human_rating >= 1 AND human_rating <= 5),
                automated_score FLOAT,
                
                created_at TIMESTAMPTZ DEFAULT NOW()
            )
        "#).execute(&self.pool).await?;
        
        // Create semantic_branches table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS semantic_branches (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                name VARCHAR(255) NOT NULL,
                base_migration_id UUID REFERENCES migrations(id),
                
                -- Semantic branching metadata
                intent TEXT,
                constraints JSONB,
                merge_strategy VARCHAR(50),
                
                -- LLM configuration for this branch
                default_llm_provider VARCHAR(50),
                default_llm_model VARCHAR(100),
                default_temperature FLOAT,
                
                created_at TIMESTAMPTZ DEFAULT NOW(),
                created_by VARCHAR(255)
            )
        "#).execute(&self.pool).await?;
        
        // Create indexes for versioning
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_block_versions_block ON block_versions(block_id, version_number)")
            .execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_block_versions_semantic_hash ON block_versions(semantic_hash)")
            .execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_llm_interactions_version ON llm_interactions(block_version_id)")
            .execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_prompt_templates_category ON prompt_templates(category)")
            .execute(&self.pool).await?;
        
        Ok(())
    }

    
    pub async fn create_migration(&self, repo_url: &str, repo_name: &str, commit_hash: &str) -> Result<Uuid> {
        let id = Uuid::new_v4();
        
        sqlx::query(
            "INSERT INTO migrations (id, repository_name, repo_name, repo_url, commit_hash, status) 
             VALUES ($1, $2, $3, $4, $5, 'in_progress')"
        )
        .bind(id)
        .bind(repo_name)  // Use repo_name for repository_name as well
        .bind(repo_name)
        .bind(repo_url)
        .bind(commit_hash)
        .execute(&self.pool)
        .await?;
        
        Ok(id)
    }
    
    pub async fn insert_container(&self, container: &Container, migration_id: Uuid) -> Result<()> {
        sqlx::query(
            "INSERT INTO containers (id, migration_id, name, container_type, language, 
                                    original_path, original_hash, source_code, version)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
        )
        .bind(container.id)
        .bind(migration_id)
        .bind(&container.name)
        .bind(&container.container_type)
        .bind(&container.language)
        .bind(&container.original_path)
        .bind(&container.original_hash)
        .bind(&container.source_code)
        .bind(container.version)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    #[allow(dead_code)]
    pub async fn insert_blocks(&self, blocks: &[Block]) -> Result<()> {
        for chunk in blocks.chunks(1000) {  // Batch insert for performance
            let mut query_builder = sqlx::QueryBuilder::new(
                "INSERT INTO blocks (id, container_id, block_type, semantic_name, 
                                    abstract_syntax, position, indent_level, metadata,
                                    parent_block_id, position_in_parent, parameters, return_type,
                                    modifiers, decorators, body_ast, language_ast, 
                                    language_features, complexity_metrics, scope_info) "
            );
            
            query_builder.push_values(chunk, |mut b, block| {
                b.push_bind(block.id)
                 .push_bind(block.container_id)
                 .push_bind(&block.block_type)
                 .push_bind(&block.semantic_name)
                 .push_bind(&block.abstract_syntax)
                 .push_bind(block.position)
                 .push_bind(block.indent_level)
                 .push_bind(&block.metadata)
                 .push_bind(block.parent_block_id)
                 .push_bind(block.position_in_parent)
                 .push_bind(&block.parameters)
                 .push_bind(&block.return_type)
                 .push_bind(&block.modifiers)
                 .push_bind(&block.decorators)
                 .push_bind(&block.body_ast)
                 .push_bind(&block.language_ast)
                 .push_bind(&block.language_features)
                 .push_bind(&block.complexity_metrics)
                 .push_bind(&block.scope_info);
            });
            
            query_builder.build().execute(&self.pool).await?;
        }
        
        Ok(())
    }
    
    pub async fn insert_semantic_block(&self, block: &crate::core::SemanticBlock, container_id: Uuid) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO blocks (
                id, container_id, block_type, semantic_name, abstract_syntax, 
                position, indent_level, parent_block_id, position_in_parent,
                parameters, return_type, modifiers, decorators, body_ast,
                language_ast, language_features, complexity_metrics, scope_info
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)"#
        )
        .bind(block.id)
        .bind(container_id)
        .bind(format!("{:?}", block.block_type))
        .bind(&block.semantic_identity.canonical_name)
        .bind(&block.syntax_preservation.normalized_ast)
        .bind(block.position.index as i32)
        .bind(0) // indent_level - will be calculated
        .bind(block.structural_context.parent_block)
        .bind(0) // position_in_parent - will be set during extraction
        .bind(serde_json::to_value(&block.semantic_metadata.parameters)?)
        .bind(block.semantic_metadata.return_type.as_ref().map(|rt| rt.representation.clone()))
        .bind(block.semantic_metadata.modifiers.iter().map(|m| format!("{:?}", m)).collect::<Vec<_>>())
        .bind(serde_json::to_value(&block.structural_context.decorators)?)
        .bind(serde_json::Value::Null) // body_ast - to be filled
        .bind(serde_json::Value::Null) // language_ast - to be filled
        .bind(serde_json::Value::Null) // language_features - to be filled
        .bind(serde_json::to_value(&block.semantic_metadata.complexity_metrics)?)
        .bind(serde_json::to_value(&block.structural_context.scope)?)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn insert_relationship(&self, relationship: &BlockRelationship) -> Result<()> {
        sqlx::query(
            "INSERT INTO block_relationships (source_block_id, target_block_id, relationship_type, metadata)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (source_block_id, target_block_id, relationship_type) DO NOTHING"
        )
        .bind(relationship.source_block_id)
        .bind(relationship.target_block_id)
        .bind(&relationship.relationship_type)
        .bind(&relationship.metadata)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn update_migration_status(&self, id: Uuid, status: &str, stats: &HashMap<String, i32>) -> Result<()> {
        sqlx::query(
            "UPDATE migrations SET status = $1, statistics = $2 WHERE id = $3"
        )
        .bind(status)
        .bind(serde_json::to_value(stats)?)
        .bind(id)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn get_containers_by_migration(&self, migration_id: Uuid) -> Result<Vec<Container>> {
        let containers = sqlx::query_as::<_, Container>(
            "SELECT * FROM containers WHERE migration_id = $1 ORDER BY created_at"
        )
        .bind(migration_id)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(containers)
    }
    
    pub async fn get_container_by_id(&self, container_id: Uuid) -> Result<Container> {
        let container = sqlx::query_as::<_, Container>(
            "SELECT * FROM containers WHERE id = $1"
        )
        .bind(container_id)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(container)
    }
    
    pub async fn get_blocks_by_container(&self, container_id: Uuid) -> Result<Vec<Block>> {
        let blocks = sqlx::query_as::<_, Block>(
            "SELECT * FROM blocks WHERE container_id = $1 ORDER BY position"
        )
        .bind(container_id)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(blocks)
    }

    #[allow(dead_code)]
    pub async fn get_relationships_by_container(&self, container_id: Uuid) -> Result<Vec<BlockRelationship>> {
        let relationships = sqlx::query_as::<_, BlockRelationship>(
            r#"
            SELECT br.* FROM block_relationships br
            JOIN blocks b ON br.source_block_id = b.id
            WHERE b.container_id = $1
            "#
        )
        .bind(container_id)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(relationships)
    }

    #[allow(dead_code)]
    pub async fn get_container_id_by_block(&self, block_id: Uuid) -> Result<Uuid> {
        let container_id = sqlx::query_scalar::<_, Uuid>(
            "SELECT container_id FROM blocks WHERE id = $1"
        )
        .bind(block_id)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(container_id)
    }

    #[allow(dead_code)]
    pub async fn get_block_by_id(&self, block_id: Uuid) -> Result<Block> {
        let block = sqlx::query_as::<_, Block>(
            "SELECT * FROM blocks WHERE id = $1"
        )
        .bind(block_id)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(block)
    }
    
    pub async fn insert_block(&self, block: &Block) -> Result<()> {
        sqlx::query(r#"
            INSERT INTO blocks (
                id, container_id, block_type, semantic_name, abstract_syntax,
                position, indent_level, metadata, created_at,
                parent_block_id, position_in_parent, parameters, return_type,
                modifiers, decorators, body_ast, language_ast, language_features,
                complexity_metrics, scope_info
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
        "#)
        .bind(&block.id)
        .bind(&block.container_id)
        .bind(&block.block_type)
        .bind(&block.semantic_name)
        .bind(&block.abstract_syntax)
        .bind(&block.position)
        .bind(&block.indent_level)
        .bind(&block.metadata)
        .bind(&block.created_at)
        .bind(&block.parent_block_id)
        .bind(&block.position_in_parent)
        .bind(&block.parameters)
        .bind(&block.return_type)
        .bind(&block.modifiers)
        .bind(&block.decorators)
        .bind(&block.body_ast)
        .bind(&block.language_ast)
        .bind(&block.language_features)
        .bind(&block.complexity_metrics)
        .bind(&block.scope_info)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    pub async fn reset_database(&self) -> Result<()> {
        // Delete all data in reverse dependency order to avoid foreign key constraints
        sqlx::query("DELETE FROM block_relationships")
            .execute(&self.pool)
            .await?;
        
        sqlx::query("DELETE FROM blocks")
            .execute(&self.pool)
            .await?;
        
        sqlx::query("DELETE FROM containers")
            .execute(&self.pool)
            .await?;
        
        sqlx::query("DELETE FROM migrations")
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
    
    pub async fn get_latest_migration(&self) -> Result<Uuid> {
        let migration = sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM migrations ORDER BY created_at DESC LIMIT 1"
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(migration)
    }
    
    pub async fn get_blocks_by_migration(&self, migration_id: Uuid) -> Result<Vec<Block>> {
        let blocks = sqlx::query_as::<_, Block>(
            r#"
            SELECT b.* FROM blocks b
            JOIN containers c ON b.container_id = c.id
            WHERE c.migration_id = $1
            "#
        )
        .bind(migration_id)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(blocks)
    }
    
    pub async fn get_relationships_by_migration(&self, migration_id: Uuid) -> Result<Vec<BlockRelationship>> {
        let relationships = sqlx::query_as::<_, BlockRelationship>(
            r#"
            SELECT br.* FROM block_relationships br
            JOIN blocks b ON br.source_block_id = b.id
            JOIN containers c ON b.container_id = c.id
            WHERE c.migration_id = $1
            "#
        )
        .bind(migration_id)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(relationships)
    }
    
    // Versioning methods
    pub async fn create_block_version(&self, version: &BlockVersion) -> Result<()> {
        sqlx::query(r#"
            INSERT INTO block_versions (
                id, block_id, version_number, semantic_hash, syntax_hash,
                created_by, semantic_changes, breaking_change,
                llm_provider, llm_model, llm_prompt_id, llm_temperature, llm_reasoning,
                change_type, change_description, parent_version, branch_name
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
        "#)
        .bind(&version.id)
        .bind(&version.block_id)
        .bind(&version.version_number)
        .bind(&version.semantic_hash)
        .bind(&version.syntax_hash)
        .bind(&version.created_by)
        .bind(&version.semantic_changes)
        .bind(&version.breaking_change)
        .bind(&version.llm_provider)
        .bind(&version.llm_model)
        .bind(&version.llm_prompt_id)
        .bind(&version.llm_temperature)
        .bind(&version.llm_reasoning)
        .bind(&version.change_type)
        .bind(&version.change_description)
        .bind(&version.parent_version)
        .bind(&version.branch_name)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn get_block_versions(&self, block_id: Uuid) -> Result<Vec<BlockVersion>> {
        let versions = sqlx::query_as::<_, BlockVersion>(
            "SELECT * FROM block_versions WHERE block_id = $1 ORDER BY version_number DESC"
        )
        .bind(block_id)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(versions)
    }
    
    pub async fn get_block_version(&self, block_id: Uuid, version_number: i32) -> Result<BlockVersion> {
        let version = sqlx::query_as::<_, BlockVersion>(
            "SELECT * FROM block_versions WHERE block_id = $1 AND version_number = $2"
        )
        .bind(block_id)
        .bind(version_number)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(version)
    }
    
    pub async fn get_latest_version_number(&self, block_id: Uuid) -> Result<i32> {
        let version_number = sqlx::query_scalar::<_, i32>(
            "SELECT COALESCE(MAX(version_number), 0) FROM block_versions WHERE block_id = $1"
        )
        .bind(block_id)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(version_number)
    }
    
    pub async fn create_prompt_template(&self, template: &PromptTemplate) -> Result<()> {
        sqlx::query(r#"
            INSERT INTO prompt_templates (
                id, name, category, prompts, variables, constraints, examples,
                version, effectiveness_score
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#)
        .bind(&template.id)
        .bind(&template.name)
        .bind(&template.category)
        .bind(&template.prompts)
        .bind(&template.variables)
        .bind(&template.constraints)
        .bind(&template.examples)
        .bind(&template.version)
        .bind(&template.effectiveness_score)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn get_prompt_template(&self, id: Uuid) -> Result<PromptTemplate> {
        let template = sqlx::query_as::<_, PromptTemplate>(
            "SELECT * FROM prompt_templates WHERE id = $1"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(template)
    }
    
    pub async fn get_prompt_templates_by_category(&self, category: &str) -> Result<Vec<PromptTemplate>> {
        let templates = sqlx::query_as::<_, PromptTemplate>(
            "SELECT * FROM prompt_templates WHERE category = $1 ORDER BY effectiveness_score DESC"
        )
        .bind(category)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(templates)
    }
    
    pub async fn create_llm_interaction(&self, interaction: &LLMInteraction) -> Result<()> {
        sqlx::query(r#"
            INSERT INTO llm_interactions (
                id, block_version_id, prompt_template_id, provider, model,
                request_payload, response_payload, tokens_used, latency_ms, cost_cents,
                confidence_score, human_rating, automated_score
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        "#)
        .bind(&interaction.id)
        .bind(&interaction.block_version_id)
        .bind(&interaction.prompt_template_id)
        .bind(&interaction.provider)
        .bind(&interaction.model)
        .bind(&interaction.request_payload)
        .bind(&interaction.response_payload)
        .bind(&interaction.tokens_used)
        .bind(&interaction.latency_ms)
        .bind(&interaction.cost_cents)
        .bind(&interaction.confidence_score)
        .bind(&interaction.human_rating)
        .bind(&interaction.automated_score)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn create_semantic_branch(&self, branch: &SemanticBranch) -> Result<()> {
        sqlx::query(r#"
            INSERT INTO semantic_branches (
                id, name, base_migration_id, intent, constraints, merge_strategy,
                default_llm_provider, default_llm_model, default_temperature, created_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#)
        .bind(&branch.id)
        .bind(&branch.name)
        .bind(&branch.base_migration_id)
        .bind(&branch.intent)
        .bind(&branch.constraints)
        .bind(&branch.merge_strategy)
        .bind(&branch.default_llm_provider)
        .bind(&branch.default_llm_model)
        .bind(&branch.default_temperature)
        .bind(&branch.created_by)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn get_semantic_branches(&self) -> Result<Vec<SemanticBranch>> {
        let branches = sqlx::query_as::<_, SemanticBranch>(
            "SELECT * FROM semantic_branches ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(branches)
    }
    
    pub async fn find_blocks_by_semantic_hash(&self, semantic_hash: &str) -> Result<Vec<BlockVersion>> {
        let versions = sqlx::query_as::<_, BlockVersion>(
            "SELECT * FROM block_versions WHERE semantic_hash = $1"
        )
        .bind(semantic_hash)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(versions)
    }
    
    /// Access to the database pool for query execution
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
    
    /// Update an existing block in the database
    pub async fn update_block(&self, block: &Block) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE blocks SET
                block_type = $2,
                semantic_name = $3,
                abstract_syntax = $4,
                position = $5,
                indent_level = $6,
                metadata = $7,
                parent_block_id = $8,
                position_in_parent = $9,
                parameters = $10,
                return_type = $11,
                modifiers = $12,
                decorators = $13,
                body_ast = $14,
                language_ast = $15,
                language_features = $16,
                complexity_metrics = $17,
                scope_info = $18
            WHERE id = $1
            "#
        )
        .bind(block.id)
        .bind(&block.block_type)
        .bind(&block.semantic_name)
        .bind(&block.abstract_syntax)
        .bind(block.position)
        .bind(block.indent_level)
        .bind(&block.metadata)
        .bind(block.parent_block_id)
        .bind(block.position_in_parent)
        .bind(&block.parameters)
        .bind(&block.return_type)
        .bind(block.modifiers.clone() as Option<Vec<String>>)
        .bind(&block.decorators)
        .bind(&block.body_ast)
        .bind(&block.language_ast)
        .bind(&block.language_features)
        .bind(&block.complexity_metrics)
        .bind(&block.scope_info)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    /// Add missing transaction method
    pub async fn begin_transaction(&self) -> Result<sqlx::Transaction<'_, sqlx::Postgres>> {
        self.pool.begin().await.map_err(Into::into)
    }
    
}
