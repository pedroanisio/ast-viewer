use async_graphql::{Object, Context, Result, ID, InputObject, SimpleObject};
use uuid::Uuid;
use std::collections::HashMap;

use crate::database::{Database, schema::{BlockVersion, SemanticBranch}};
use crate::versioning::{
    SemanticVersionControl, LLMProviderManager, SemanticMergeHandler,
};
use crate::versioning::semantic_version_control::{LLMConfig, VersioningContext};
use crate::versioning::semantic_merge_handler::MergeResult;
use crate::versioning::semantic_version_control::SemanticDiff;

#[derive(SimpleObject)]
pub struct BlockVersionType {
    pub id: ID,
    pub block_id: ID,
    pub version_number: i32,
    pub semantic_hash: String,
    pub syntax_hash: String,
    pub created_at: String,
    pub created_by: Option<String>,
    pub breaking_change: bool,
    pub llm_provider: Option<String>,
    pub llm_model: Option<String>,
    pub llm_reasoning: Option<String>,
    pub change_type: Option<String>,
    pub change_description: Option<String>,
    pub branch_name: Option<String>,
}

impl From<BlockVersion> for BlockVersionType {
    fn from(version: BlockVersion) -> Self {
        Self {
            id: ID(version.id.to_string()),
            block_id: ID(version.block_id.to_string()),
            version_number: version.version_number,
            semantic_hash: version.semantic_hash,
            syntax_hash: version.syntax_hash,
            created_at: version.created_at.to_rfc3339(),
            created_by: version.created_by,
            breaking_change: version.breaking_change,
            llm_provider: version.llm_provider,
            llm_model: version.llm_model,
            llm_reasoning: version.llm_reasoning,
            change_type: version.change_type,
            change_description: version.change_description,
            branch_name: version.branch_name,
        }
    }
}

#[derive(SimpleObject)]
pub struct SemanticBranchType {
    pub id: ID,
    pub name: String,
    pub intent: Option<String>,
    pub merge_strategy: Option<String>,
    pub default_llm_provider: Option<String>,
    pub default_llm_model: Option<String>,
    pub created_at: String,
    pub created_by: Option<String>,
}

impl From<SemanticBranch> for SemanticBranchType {
    fn from(branch: SemanticBranch) -> Self {
        Self {
            id: ID(branch.id.to_string()),
            name: branch.name,
            intent: branch.intent,
            merge_strategy: branch.merge_strategy,
            default_llm_provider: branch.default_llm_provider,
            default_llm_model: branch.default_llm_model,
            created_at: branch.created_at.to_rfc3339(),
            created_by: branch.created_by,
        }
    }
}

#[derive(SimpleObject)]
pub struct SemanticDiffType {
    pub behavioral_changes: Vec<String>,
    pub interface_changes: Vec<String>,
    pub performance_changes: Vec<String>,
    pub breaking_changes: Vec<String>,
    pub llm_explanation: Option<String>,
}

impl From<SemanticDiff> for SemanticDiffType {
    fn from(diff: SemanticDiff) -> Self {
        Self {
            behavioral_changes: diff.behavioral_changes,
            interface_changes: diff.interface_changes,
            performance_changes: diff.performance_changes,
            breaking_changes: diff.breaking_changes,
            llm_explanation: diff.llm_explanation,
        }
    }
}

#[derive(SimpleObject)]
pub struct MergeResultType {
    pub success: bool,
    pub merged_branch: Option<SemanticBranchType>,
    pub auto_resolved: i32,
    pub requires_review: i32,
    pub merge_summary: String,
}

impl From<MergeResult> for MergeResultType {
    fn from(result: MergeResult) -> Self {
        Self {
            success: result.success,
            merged_branch: result.merged_branch.map(|b| b.into()),
            auto_resolved: result.auto_resolved as i32,
            requires_review: result.requires_review.len() as i32,
            merge_summary: result.merge_summary,
        }
    }
}

#[derive(InputObject)]
pub struct CreateVersionInput {
    pub block_id: ID,
    pub changes: String, // JSON string
    pub breaking_change: bool,
    pub change_type: String,
    pub change_description: String,
    pub branch_name: Option<String>,
    pub llm_config: Option<LLMConfigInput>,
}

#[derive(InputObject)]
pub struct LLMConfigInput {
    pub provider: String,
    pub model: String,
    pub prompt_template_id: Option<ID>,
    pub temperature: Option<f64>,
    pub reasoning: Option<String>,
}

#[derive(InputObject)]
pub struct CreateBranchInput {
    pub name: String,
    pub base_migration_id: Option<ID>,
    pub intent: Option<String>,
    pub merge_strategy: Option<String>,
    pub llm_provider: Option<String>,
    pub llm_model: Option<String>,
    pub temperature: Option<f64>,
}

#[derive(InputObject)]
pub struct MergeBranchesInput {
    pub source_branch: String,
    pub target_branch: String,
    pub strategy: String,
}

#[derive(InputObject)]
pub struct RollbackInput {
    pub block_id: ID,
    pub target_version: i32,
    pub reason: String,
}

pub struct VersioningQuery;

#[Object]
impl VersioningQuery {
    /// Get version history for a block
    async fn block_versions(
        &self,
        ctx: &Context<'_>,
        block_id: ID,
    ) -> Result<Vec<BlockVersionType>> {
        let db = ctx.data::<Database>()?;
        let block_uuid = Uuid::parse_str(&block_id)?;
        
        let versions = db.get_block_versions(block_uuid).await?;
        Ok(versions.into_iter().map(|v| v.into()).collect())
    }
    
    /// Get a specific block version
    async fn block_version(
        &self,
        ctx: &Context<'_>,
        block_id: ID,
        version_number: i32,
    ) -> Result<BlockVersionType> {
        let db = ctx.data::<Database>()?;
        let block_uuid = Uuid::parse_str(&block_id)?;
        
        let version = db.get_block_version(block_uuid, version_number).await?;
        Ok(version.into())
    }
    
    /// Compare two versions semantically
    async fn semantic_diff(
        &self,
        ctx: &Context<'_>,
        version1_id: ID,
        version2_id: ID,
        use_llm: Option<bool>,
    ) -> Result<SemanticDiffType> {
        let db = ctx.data::<Database>()?;
        let version_control = SemanticVersionControl::new(db.clone());
        
        let v1_uuid = Uuid::parse_str(&version1_id)?;
        let v2_uuid = Uuid::parse_str(&version2_id)?;
        
        let diff = version_control.semantic_diff(
            v1_uuid,
            v2_uuid,
            use_llm.unwrap_or(false),
        ).await?;
        
        Ok(diff.into())
    }
    
    /// Get all semantic branches
    async fn semantic_branches(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<SemanticBranchType>> {
        let db = ctx.data::<Database>()?;
        let branches = db.get_semantic_branches().await?;
        Ok(branches.into_iter().map(|b| b.into()).collect())
    }
    
    /// Find blocks with the same semantic hash
    async fn semantic_duplicates(
        &self,
        ctx: &Context<'_>,
        semantic_hash: String,
    ) -> Result<Vec<BlockVersionType>> {
        let db = ctx.data::<Database>()?;
        let versions = db.find_blocks_by_semantic_hash(&semantic_hash).await?;
        Ok(versions.into_iter().map(|v| v.into()).collect())
    }
    
    /// Preview merge conflicts without actually merging
    async fn preview_merge(
        &self,
        ctx: &Context<'_>,
        source_branch: String,
        target_branch: String,
    ) -> Result<i32> {
        let db = ctx.data::<Database>()?;
        let llm_manager = ctx.data::<LLMProviderManager>()?;
        let merge_handler = SemanticMergeHandler::new(db.clone(), (*llm_manager).clone());
        
        let conflicts = merge_handler.preview_merge(&source_branch, &target_branch).await?;
        Ok(conflicts.len() as i32)
    }
}

pub struct VersioningMutation;

#[Object]
impl VersioningMutation {
    /// Create a new version of a block
    async fn create_version(
        &self,
        ctx: &Context<'_>,
        input: CreateVersionInput,
    ) -> Result<BlockVersionType> {
        let db = ctx.data::<Database>()?;
        let version_control = SemanticVersionControl::new(db.clone());
        
        let block_uuid = Uuid::parse_str(&input.block_id)?;
        let block = db.get_block_by_id(block_uuid).await?;
        
        let changes: HashMap<String, serde_json::Value> = 
            serde_json::from_str(&input.changes)?;
        
        let context = VersioningContext {
            changes,
            breaking_change: input.breaking_change,
            change_type: input.change_type,
            change_description: input.change_description,
            branch_name: input.branch_name,
        };
        
        let llm_config = input.llm_config.map(|config| LLMConfig {
            provider: config.provider,
            model: config.model,
            prompt_template_id: config.prompt_template_id
                .and_then(|id| Uuid::parse_str(&id).ok()),
            temperature: config.temperature.map(|t| t as f32),
            reasoning: config.reasoning,
        });
        
        let version = version_control.create_version(&block, context, llm_config).await?;
        Ok(version.into())
    }
    
    /// Create a new semantic branch
    async fn create_semantic_branch(
        &self,
        ctx: &Context<'_>,
        input: CreateBranchInput,
    ) -> Result<SemanticBranchType> {
        let db = ctx.data::<Database>()?;
        
        let branch = SemanticBranch {
            id: Uuid::new_v4(),
            name: input.name,
            base_migration_id: input.base_migration_id
                .and_then(|id| Uuid::parse_str(&id).ok()),
            intent: input.intent,
            constraints: None,
            merge_strategy: input.merge_strategy,
            default_llm_provider: input.llm_provider,
            default_llm_model: input.llm_model,
            default_temperature: input.temperature.map(|t| t as f32),
            created_at: chrono::Utc::now(),
            created_by: Some("graphql_api".to_string()),
        };
        
        db.create_semantic_branch(&branch).await?;
        Ok(branch.into())
    }
    
    /// Merge two semantic branches
    async fn merge_branches(
        &self,
        ctx: &Context<'_>,
        input: MergeBranchesInput,
    ) -> Result<MergeResultType> {
        let db = ctx.data::<Database>()?;
        let llm_manager = ctx.data::<LLMProviderManager>()?;
        let merge_handler = SemanticMergeHandler::new(db.clone(), (*llm_manager).clone());
        
        let result = merge_handler.merge_branches(
            &input.source_branch,
            &input.target_branch,
            input.strategy,
        ).await?;
        
        Ok(result.into())
    }
    
    /// Rollback to a previous version
    async fn rollback_version(
        &self,
        ctx: &Context<'_>,
        input: RollbackInput,
    ) -> Result<BlockVersionType> {
        let db = ctx.data::<Database>()?;
        let version_control = SemanticVersionControl::new(db.clone());
        
        let block_uuid = Uuid::parse_str(&input.block_id)?;
        
        let version = version_control.rollback_to_version(
            block_uuid,
            input.target_version,
            input.reason,
        ).await?;
        
        Ok(version.into())
    }
}
