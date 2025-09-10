use anyhow::Result;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use blake3::Hasher;
use chrono::Utc;

use crate::database::{Database, schema::{Block, BlockVersion, LLMInteraction}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub provider: String,
    pub model: String,
    pub prompt_template_id: Option<Uuid>,
    pub temperature: Option<f32>,
    pub reasoning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticDiff {
    pub behavioral_changes: Vec<String>,
    pub interface_changes: Vec<String>,
    pub performance_changes: Vec<String>,
    pub breaking_changes: Vec<String>,
    pub llm_explanation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersioningContext {
    pub changes: HashMap<String, serde_json::Value>,
    pub breaking_change: bool,
    pub change_type: String,
    pub change_description: String,
    pub branch_name: Option<String>,
}

pub struct SemanticVersionControl {
    db: Database,
}

impl SemanticVersionControl {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
    
    /// Create a new version with semantic tracking
    pub async fn create_version(
        &self,
        block: &Block,
        context: VersioningContext,
        llm_config: Option<LLMConfig>,
    ) -> Result<BlockVersion> {
        // Calculate semantic hash (structure + behavior)
        let semantic_hash = self.calculate_semantic_hash(block)?;
        
        // Calculate syntax hash (actual code)
        let syntax_hash = self.calculate_syntax_hash(block)?;
        
        // Determine version number based on semantic changes
        let version_number = self.determine_version_number(
            block.id,
            &context,
        ).await?;
        
        let version = BlockVersion {
            id: Uuid::new_v4(),
            block_id: block.id,
            version_number,
            semantic_hash,
            syntax_hash,
            created_at: Utc::now(),
            created_by: llm_config.as_ref().map(|c| format!("{}:{}", c.provider, c.model)),
            semantic_changes: Some(serde_json::to_value(&context.changes)?),
            breaking_change: context.breaking_change,
            llm_provider: llm_config.as_ref().map(|c| c.provider.clone()),
            llm_model: llm_config.as_ref().map(|c| c.model.clone()),
            llm_prompt_id: llm_config.as_ref().and_then(|c| c.prompt_template_id),
            llm_temperature: llm_config.as_ref().and_then(|c| c.temperature),
            llm_reasoning: llm_config.as_ref().and_then(|c| c.reasoning.clone()),
            change_type: Some(context.change_type),
            change_description: Some(context.change_description),
            parent_version: self.get_latest_version_id(block.id).await.ok(),
            branch_name: context.branch_name,
        };
        
        self.db.create_block_version(&version).await?;
        
        // Track LLM interaction if applicable
        if let Some(llm_config) = llm_config {
            let interaction = LLMInteraction {
                id: Uuid::new_v4(),
                block_version_id: Some(version.id),
                prompt_template_id: llm_config.prompt_template_id,
                provider: llm_config.provider,
                model: llm_config.model,
                request_payload: serde_json::json!({
                    "block_id": block.id,
                    "context": context.changes
                }),
                response_payload: serde_json::json!({
                    "version_id": version.id,
                    "semantic_hash": version.semantic_hash
                }),
                tokens_used: None, // Would be filled by actual LLM provider
                latency_ms: None,
                cost_cents: None,
                confidence_score: None,
                human_rating: None,
                automated_score: None,
                created_at: Utc::now(),
            };
            
            self.db.create_llm_interaction(&interaction).await?;
        }
        
        Ok(version)
    }
    
    /// Compare versions semantically, not textually
    pub async fn semantic_diff(
        &self,
        version1_id: Uuid,
        version2_id: Uuid,
        use_llm: bool,
    ) -> Result<SemanticDiff> {
        let version1 = self.db.get_block_version(version1_id, 1).await?; // TODO: Fix this
        let version2 = self.db.get_block_version(version2_id, 1).await?; // TODO: Fix this
        
        let mut diff = SemanticDiff {
            behavioral_changes: vec![],
            interface_changes: vec![],
            performance_changes: vec![],
            breaking_changes: vec![],
            llm_explanation: None,
        };
        
        // Analyze semantic changes from version metadata
        if let (Some(changes1), Some(changes2)) = (&version1.semantic_changes, &version2.semantic_changes) {
            // Compare semantic changes
            self.analyze_semantic_changes(changes1, changes2, &mut diff)?;
        }
        
        // Check for breaking changes
        if version2.breaking_change && !version1.breaking_change {
            diff.breaking_changes.push("Version introduces breaking changes".to_string());
        }
        
        // If LLM analysis requested, add AI-generated explanation
        if use_llm {
            // TODO: Implement LLM-based diff analysis
            diff.llm_explanation = Some("LLM-based semantic analysis would go here".to_string());
        }
        
        Ok(diff)
    }
    
    /// Calculate semantic hash based on structure and behavior
    fn calculate_semantic_hash(&self, block: &Block) -> Result<String> {
        let mut hasher = Hasher::new();
        
        // Hash semantic components
        hasher.update(block.block_type.as_bytes());
        
        if let Some(name) = &block.semantic_name {
            hasher.update(name.as_bytes());
        }
        
        // Hash abstract syntax (semantic structure)
        let ast_str = serde_json::to_string(&block.abstract_syntax)?;
        hasher.update(ast_str.as_bytes());
        
        // Hash parameters and return type (interface)
        if let Some(params) = &block.parameters {
            let params_str = serde_json::to_string(params)?;
            hasher.update(params_str.as_bytes());
        }
        
        if let Some(return_type) = &block.return_type {
            hasher.update(return_type.as_bytes());
        }
        
        // Hash modifiers (affects behavior)
        if let Some(modifiers) = &block.modifiers {
            for modifier in modifiers {
                hasher.update(modifier.as_bytes());
            }
        }
        
        Ok(hasher.finalize().to_hex().to_string())
    }
    
    /// Calculate syntax hash based on actual code
    fn calculate_syntax_hash(&self, block: &Block) -> Result<String> {
        let mut hasher = Hasher::new();
        
        // Hash the actual code representation
        let ast_str = serde_json::to_string(&block.abstract_syntax)?;
        hasher.update(ast_str.as_bytes());
        
        // Include language-specific AST if available
        if let Some(lang_ast) = &block.language_ast {
            let lang_ast_str = serde_json::to_string(lang_ast)?;
            hasher.update(lang_ast_str.as_bytes());
        }
        
        Ok(hasher.finalize().to_hex().to_string())
    }
    
    /// Determine version number based on semantic changes
    async fn determine_version_number(
        &self,
        block_id: Uuid,
        context: &VersioningContext,
    ) -> Result<i32> {
        let latest_version = self.db.get_latest_version_number(block_id).await?;
        
        // Increment based on change type
        let increment = match context.change_type.as_str() {
            "major" | "breaking" => {
                if context.breaking_change { 10 } else { 5 }
            },
            "minor" | "feature" => 3,
            "patch" | "fix" => 1,
            "optimization" => 1,
            _ => 1,
        };
        
        Ok(latest_version + increment)
    }
    
    async fn get_latest_version_id(&self, block_id: Uuid) -> Result<Uuid> {
        let versions = self.db.get_block_versions(block_id).await?;
        versions.first()
            .map(|v| v.id)
            .ok_or_else(|| anyhow::anyhow!("No versions found for block"))
    }
    
    fn analyze_semantic_changes(
        &self,
        changes1: &serde_json::Value,
        changes2: &serde_json::Value,
        diff: &mut SemanticDiff,
    ) -> Result<()> {
        // Compare changes and categorize them
        if changes1 != changes2 {
            diff.behavioral_changes.push("Semantic changes detected".to_string());
        }
        
        // TODO: Implement more sophisticated semantic change analysis
        
        Ok(())
    }
    
    /// Get version history for a block
    pub async fn get_version_history(&self, block_id: Uuid) -> Result<Vec<BlockVersion>> {
        self.db.get_block_versions(block_id).await
    }
    
    /// Find blocks with the same semantic hash (semantic duplicates)
    pub async fn find_semantic_duplicates(&self, semantic_hash: &str) -> Result<Vec<BlockVersion>> {
        self.db.find_blocks_by_semantic_hash(semantic_hash).await
    }
    
    /// Rollback to a specific version
    pub async fn rollback_to_version(
        &self,
        block_id: Uuid,
        target_version: i32,
        reason: String,
    ) -> Result<BlockVersion> {
        let target = self.db.get_block_version(block_id, target_version).await?;
        
        // Create a new version that restores the target state
        let rollback_context = VersioningContext {
            changes: HashMap::from([
                ("rollback_target".to_string(), serde_json::json!(target_version)),
                ("rollback_reason".to_string(), serde_json::json!(reason)),
            ]),
            breaking_change: false,
            change_type: "rollback".to_string(),
            change_description: format!("Rollback to version {}", target_version),
            branch_name: None,
        };
        
        // TODO: Actually restore the block state from the target version
        // For now, just create a version record
        let latest_version = self.db.get_latest_version_number(block_id).await?;
        
        let rollback_version = BlockVersion {
            id: Uuid::new_v4(),
            block_id,
            version_number: latest_version + 1,
            semantic_hash: target.semantic_hash.clone(),
            syntax_hash: target.syntax_hash.clone(),
            created_at: Utc::now(),
            created_by: Some("system:rollback".to_string()),
            semantic_changes: Some(serde_json::to_value(&rollback_context.changes)?),
            breaking_change: false,
            llm_provider: None,
            llm_model: None,
            llm_prompt_id: None,
            llm_temperature: None,
            llm_reasoning: Some(format!("Rollback to version {} due to: {}", target_version, reason)),
            change_type: Some("rollback".to_string()),
            change_description: Some(rollback_context.change_description),
            parent_version: Some(target.id),
            branch_name: None,
        };
        
        self.db.create_block_version(&rollback_version).await?;
        
        Ok(rollback_version)
    }
}
