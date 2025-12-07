use anyhow::Result;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chrono::Utc;

use crate::database::{Database, schema::{BlockVersion, SemanticBranch}};
use super::llm_provider_manager::{LLMProviderManager, LLMRequest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticConflict {
    pub conflict_type: ConflictType,
    pub location: String,
    pub base_version: Option<BlockVersion>,
    pub source_version: BlockVersion,
    pub target_version: BlockVersion,
    pub resolution: Option<ConflictResolution>,
    pub reasoning: Option<String>,
    pub confidence: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictType {
    BehavioralConflict,    // Same function, different behavior
    InterfaceConflict,     // Different signatures for same semantic function
    StructuralConflict,    // Different code structure, same intent
    SemanticConflict,      // Conflicting semantic meanings
    DependencyConflict,    // Conflicting dependencies
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolution {
    pub strategy: ResolutionStrategy,
    pub merged_version: Option<BlockVersion>,
    pub explanation: String,
    pub requires_human_review: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionStrategy {
    TakeSource,           // Use source version
    TakeTarget,           // Use target version
    Merge,                // Combine both versions
    CreateNew,            // Generate new version that satisfies both
    RequireHumanInput,    // Cannot auto-resolve
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResult {
    pub success: bool,
    pub merged_branch: Option<SemanticBranch>,
    pub conflicts: Vec<SemanticConflict>,
    pub auto_resolved: usize,
    pub requires_review: Vec<SemanticConflict>,
    pub merge_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeContext {
    pub source_intent: String,
    pub target_intent: String,
    pub business_rules: Vec<String>,
    pub constraints: HashMap<String, serde_json::Value>,
    pub merge_strategy: String,
}

pub struct SemanticMergeHandler {
    db: Database,
    llm_manager: LLMProviderManager,
}

impl SemanticMergeHandler {
    pub fn new(db: Database, llm_manager: LLMProviderManager) -> Self {
        Self { db, llm_manager }
    }
    
    /// Merge branches with LLM assistance for conflict resolution
    pub async fn merge_branches(
        &self,
        source_branch: &str,
        target_branch: &str,
        strategy: String,
    ) -> Result<MergeResult> {
        // Get branch information
        let branches = self.db.get_semantic_branches().await?;
        let source = branches.iter().find(|b| b.name == source_branch)
            .ok_or_else(|| anyhow::anyhow!("Source branch not found: {}", source_branch))?;
        let target = branches.iter().find(|b| b.name == target_branch)
            .ok_or_else(|| anyhow::anyhow!("Target branch not found: {}", target_branch))?;
        
        // Detect semantic conflicts
        let conflicts = self.detect_semantic_conflicts(source, target).await?;
        
        let mut auto_resolved = 0;
        let mut requires_review = Vec::new();
        let mut resolved_conflicts = Vec::new();
        
        // Attempt to resolve conflicts
        for mut conflict in conflicts {
            if strategy == "ai_assisted" {
                match self.resolve_conflict_with_llm(&mut conflict, source, target).await {
                    Ok(true) => {
                        auto_resolved += 1;
                        resolved_conflicts.push(conflict);
                    }
                    Ok(false) => {
                        requires_review.push(conflict);
                    }
                    Err(e) => {
                        println!("Failed to resolve conflict with LLM: {}", e);
                        requires_review.push(conflict);
                    }
                }
            } else {
                requires_review.push(conflict);
            }
        }
        
        let success = requires_review.is_empty();
        let merged_branch = if success {
            Some(self.create_merged_branch(source, target, &resolved_conflicts).await?)
        } else {
            None
        };
        
        let merge_summary = format!(
            "Merge {} -> {}: {} conflicts, {} auto-resolved, {} require review",
            source_branch, target_branch,
            resolved_conflicts.len() + requires_review.len(),
            auto_resolved,
            requires_review.len()
        );
        
        Ok(MergeResult {
            success,
            merged_branch,
            conflicts: resolved_conflicts,
            auto_resolved,
            requires_review,
            merge_summary,
        })
    }
    
    async fn detect_semantic_conflicts(
        &self,
        source: &SemanticBranch,
        target: &SemanticBranch,
    ) -> Result<Vec<SemanticConflict>> {
        let mut conflicts = Vec::new();
        
        // Get blocks from both branches
        let source_blocks = if let Some(migration_id) = source.base_migration_id {
            self.db.get_blocks_by_migration(migration_id).await?
        } else {
            Vec::new()
        };
        
        let target_blocks = if let Some(migration_id) = target.base_migration_id {
            self.db.get_blocks_by_migration(migration_id).await?
        } else {
            Vec::new()
        };
        
        // Find blocks that exist in both branches with different semantic hashes
        for source_block in &source_blocks {
            for target_block in &target_blocks {
                if source_block.semantic_name == target_block.semantic_name 
                   && source_block.semantic_name.is_some() {
                    
                    // Get latest versions for comparison
                    let source_versions = self.db.get_block_versions(source_block.id).await?;
                    let target_versions = self.db.get_block_versions(target_block.id).await?;
                    
                    if let (Some(source_version), Some(target_version)) = 
                       (source_versions.first(), target_versions.first()) {
                        
                        if source_version.semantic_hash != target_version.semantic_hash {
                            let conflict_type = self.classify_conflict(source_version, target_version);
                            
                            conflicts.push(SemanticConflict {
                                conflict_type,
                                location: source_block.semantic_name.clone().unwrap_or_default(),
                                base_version: None, // Would need common ancestor
                                source_version: source_version.clone(),
                                target_version: target_version.clone(),
                                resolution: None,
                                reasoning: None,
                                confidence: None,
                            });
                        }
                    }
                }
            }
        }
        
        Ok(conflicts)
    }
    
    fn classify_conflict(&self, source: &BlockVersion, target: &BlockVersion) -> ConflictType {
        // Analyze the types of changes to classify the conflict
        if source.breaking_change || target.breaking_change {
            ConflictType::InterfaceConflict
        } else if source.change_type.as_deref() == Some("behavioral") || 
                  target.change_type.as_deref() == Some("behavioral") {
            ConflictType::BehavioralConflict
        } else if source.syntax_hash != target.syntax_hash {
            ConflictType::StructuralConflict
        } else {
            ConflictType::SemanticConflict
        }
    }
    
    async fn resolve_conflict_with_llm(
        &self,
        conflict: &mut SemanticConflict,
        source_branch: &SemanticBranch,
        target_branch: &SemanticBranch,
    ) -> Result<bool> {
        // Create context for LLM
        let context = MergeContext {
            source_intent: source_branch.intent.clone().unwrap_or_default(),
            target_intent: target_branch.intent.clone().unwrap_or_default(),
            business_rules: vec![], // Would be loaded from configuration
            constraints: HashMap::new(),
            merge_strategy: "semantic_aware".to_string(),
        };
        
        // Create prompt for conflict resolution
        let prompt = self.create_conflict_resolution_prompt(conflict, &context)?;
        
        // Get LLM resolution
        let llm_request = LLMRequest {
            prompt,
            model: Some("gpt-4".to_string()),
            temperature: Some(0.3), // Lower temperature for more consistent results
            max_tokens: Some(2000),
            context: HashMap::from([
                ("conflict_type".to_string(), serde_json::json!(conflict.conflict_type)),
                ("location".to_string(), serde_json::json!(conflict.location)),
            ]),
        };
        
        // Use the first available provider for now
        let providers = self.llm_manager.list_providers();
        if let Some(provider_name) = providers.first() {
            if let Some(provider) = self.llm_manager.get_provider(provider_name) {
                let result = provider.execute(llm_request).await?;
                
                // Parse LLM response to determine resolution strategy
                let resolution = self.parse_llm_resolution(&result.content)?;
                
                conflict.resolution = Some(resolution);
                conflict.reasoning = Some(result.content);
                conflict.confidence = result.confidence_score;
                
                // Consider it resolved if confidence is high enough
                Ok(result.confidence_score.unwrap_or(0.0) > 0.8)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }
    
    fn create_conflict_resolution_prompt(
        &self,
        conflict: &SemanticConflict,
        context: &MergeContext,
    ) -> Result<String> {
        let prompt = format!(r#"
You are an expert code merge assistant. Analyze this semantic conflict and provide a resolution strategy.

## Conflict Details
- Type: {:?}
- Location: {}
- Source Intent: {}
- Target Intent: {}

## Source Version
- Change Type: {:?}
- Description: {}
- Breaking Change: {}
- LLM Reasoning: {}

## Target Version  
- Change Type: {:?}
- Description: {}
- Breaking Change: {}
- LLM Reasoning: {}

## Business Context
- Constraints: {:?}
- Merge Strategy: {}

## Instructions
Analyze the conflict and provide:
1. Resolution strategy (TakeSource, TakeTarget, Merge, CreateNew, or RequireHumanInput)
2. Detailed explanation of your reasoning
3. If merging, describe how to combine both changes
4. Confidence level (0.0-1.0)

Format your response as:
STRATEGY: [strategy]
CONFIDENCE: [0.0-1.0]
EXPLANATION: [detailed reasoning]
MERGE_APPROACH: [if applicable]
"#,
            conflict.conflict_type,
            conflict.location,
            context.source_intent,
            context.target_intent,
            conflict.source_version.change_type,
            conflict.source_version.change_description.as_deref().unwrap_or(""),
            conflict.source_version.breaking_change,
            conflict.source_version.llm_reasoning.as_deref().unwrap_or(""),
            conflict.target_version.change_type,
            conflict.target_version.change_description.as_deref().unwrap_or(""),
            conflict.target_version.breaking_change,
            conflict.target_version.llm_reasoning.as_deref().unwrap_or(""),
            context.constraints,
            context.merge_strategy
        );
        
        Ok(prompt)
    }
    
    fn parse_llm_resolution(&self, response: &str) -> Result<ConflictResolution> {
        // Parse structured LLM response
        let mut strategy = ResolutionStrategy::RequireHumanInput;
        let explanation = response.to_string();
        let mut confidence = 0.5;
        
        // Simple parsing - in production would use more robust parsing
        if response.contains("STRATEGY: TakeSource") {
            strategy = ResolutionStrategy::TakeSource;
        } else if response.contains("STRATEGY: TakeTarget") {
            strategy = ResolutionStrategy::TakeTarget;
        } else if response.contains("STRATEGY: Merge") {
            strategy = ResolutionStrategy::Merge;
        } else if response.contains("STRATEGY: CreateNew") {
            strategy = ResolutionStrategy::CreateNew;
        }
        
        // Extract confidence if present
        if let Some(conf_line) = response.lines().find(|line| line.starts_with("CONFIDENCE:")) {
            if let Some(conf_str) = conf_line.split(':').nth(1) {
                if let Ok(conf_val) = conf_str.trim().parse::<f32>() {
                    confidence = conf_val;
                }
            }
        }
        
        Ok(ConflictResolution {
            strategy,
            merged_version: None, // Would be created based on strategy
            explanation,
            requires_human_review: confidence < 0.8,
        })
    }
    
    async fn create_merged_branch(
        &self,
        source: &SemanticBranch,
        target: &SemanticBranch,
        resolved_conflicts: &[SemanticConflict],
    ) -> Result<SemanticBranch> {
        let merged_branch = SemanticBranch {
            id: Uuid::new_v4(),
            name: format!("merged-{}-{}", source.name, target.name),
            base_migration_id: target.base_migration_id, // Use target as base
            intent: Some(format!(
                "Merged branch combining: {} and {}",
                source.intent.as_deref().unwrap_or(&source.name),
                target.intent.as_deref().unwrap_or(&target.name)
            )),
            constraints: Some(serde_json::json!({
                "source_branch": source.name,
                "target_branch": target.name,
                "resolved_conflicts": resolved_conflicts.len(),
                "merge_timestamp": Utc::now()
            })),
            merge_strategy: Some("ai_assisted".to_string()),
            default_llm_provider: target.default_llm_provider.clone(),
            default_llm_model: target.default_llm_model.clone(),
            default_temperature: target.default_temperature,
            created_at: Utc::now(),
            created_by: Some("system:semantic_merge".to_string()),
        };
        
        self.db.create_semantic_branch(&merged_branch).await?;
        
        Ok(merged_branch)
    }
    
    /// Get merge preview without actually performing the merge
    pub async fn preview_merge(
        &self,
        source_branch: &str,
        target_branch: &str,
    ) -> Result<Vec<SemanticConflict>> {
        let branches = self.db.get_semantic_branches().await?;
        let source = branches.iter().find(|b| b.name == source_branch)
            .ok_or_else(|| anyhow::anyhow!("Source branch not found: {}", source_branch))?;
        let target = branches.iter().find(|b| b.name == target_branch)
            .ok_or_else(|| anyhow::anyhow!("Target branch not found: {}", target_branch))?;
        
        self.detect_semantic_conflicts(source, target).await
    }
    
    /// Rollback a merge by creating a new branch that reverts changes
    pub async fn rollback_merge(
        &self,
        merged_branch_id: Uuid,
        reason: String,
    ) -> Result<SemanticBranch> {
        // Get the merged branch
        let branches = self.db.get_semantic_branches().await?;
        let merged_branch = branches.iter().find(|b| b.id == merged_branch_id)
            .ok_or_else(|| anyhow::anyhow!("Merged branch not found"))?;
        
        // Create rollback branch
        let rollback_branch = SemanticBranch {
            id: Uuid::new_v4(),
            name: format!("rollback-{}", merged_branch.name),
            base_migration_id: merged_branch.base_migration_id,
            intent: Some(format!("Rollback of merge: {}", reason)),
            constraints: Some(serde_json::json!({
                "rollback_of": merged_branch.id,
                "rollback_reason": reason,
                "rollback_timestamp": Utc::now()
            })),
            merge_strategy: Some("rollback".to_string()),
            default_llm_provider: merged_branch.default_llm_provider.clone(),
            default_llm_model: merged_branch.default_llm_model.clone(),
            default_temperature: merged_branch.default_temperature,
            created_at: Utc::now(),
            created_by: Some("system:rollback".to_string()),
        };
        
        self.db.create_semantic_branch(&rollback_branch).await?;
        
        Ok(rollback_branch)
    }
}
