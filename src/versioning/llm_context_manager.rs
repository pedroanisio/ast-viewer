use anyhow::Result;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use crate::database::{Database, schema::{Block, BlockVersion}};
use crate::analysis::DependencyAnalyzer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMContext {
    pub block_info: BlockContext,
    pub history: Option<VersionHistory>,
    pub dependencies: Option<DependencyContext>,
    pub consumers: Option<Vec<BlockContext>>,
    pub patterns: Option<Vec<ArchitecturalPattern>>,
    pub violations: Option<Vec<ArchitecturalViolation>>,
    pub domain: Option<DomainContext>,
    pub requirements: Option<Vec<Requirement>>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockContext {
    pub id: Uuid,
    pub semantic_name: Option<String>,
    pub block_type: String,
    pub complexity_score: Option<f64>,
    pub language: Option<String>,
    pub file_path: Option<String>,
    pub line_range: Option<(u32, u32)>,
    pub abstract_syntax: serde_json::Value,
    pub parameters: Option<serde_json::Value>,
    pub return_type: Option<String>,
    pub modifiers: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionHistory {
    pub versions: Vec<VersionSummary>,
    pub evolution_trend: EvolutionTrend,
    pub change_frequency: f64,
    pub stability_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionSummary {
    pub version_number: i32,
    pub created_at: String,
    pub change_type: Option<String>,
    pub description: Option<String>,
    pub breaking_change: bool,
    pub llm_provider: Option<String>,
    pub confidence_score: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvolutionTrend {
    Stable,           // Few changes, consistent behavior
    Growing,          // Adding features, expanding functionality
    Refactoring,      // Structural improvements, no new features
    Declining,        // Bug fixes, maintenance mode
    Volatile,         // Frequent changes, unstable
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyContext {
    pub direct_dependencies: Vec<BlockContext>,
    pub transitive_dependencies: Vec<BlockContext>,
    pub dependency_depth: u32,
    pub circular_dependencies: Vec<Vec<Uuid>>,
    pub critical_path: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalPattern {
    pub pattern_type: String,
    pub confidence: f64,
    pub description: String,
    pub components: Vec<Uuid>,
    pub benefits: Vec<String>,
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalViolation {
    pub violation_type: String,
    pub severity: ViolationSeverity,
    pub description: String,
    pub affected_blocks: Vec<Uuid>,
    pub suggested_fixes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainContext {
    pub domain_name: String,
    pub business_concepts: Vec<String>,
    pub domain_rules: Vec<String>,
    pub bounded_context: Option<String>,
    pub ubiquitous_language: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    pub id: String,
    pub description: String,
    pub priority: RequirementPriority,
    pub status: RequirementStatus,
    pub related_blocks: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequirementPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequirementStatus {
    Pending,
    InProgress,
    Completed,
    Blocked,
}

pub struct LLMContextManager {
    db: Database,
    dependency_analyzer: DependencyAnalyzer,
}

impl LLMContextManager {
    pub fn new(db: Database) -> Self {
        Self {
            dependency_analyzer: DependencyAnalyzer::new(db.clone()),
            db,
        }
    }
    
    /// Build comprehensive context for LLM operations
    pub async fn build_context_for_block(
        &self,
        block_id: Uuid,
        include_history: bool,
        include_dependencies: bool,
        include_consumers: bool,
        include_patterns: bool,
        include_domain: bool,
    ) -> Result<LLMContext> {
        let block = self.db.get_block_by_id(block_id).await?;
        let block_context = self.create_block_context(&block).await?;
        
        let mut context = LLMContext {
            block_info: block_context,
            history: None,
            dependencies: None,
            consumers: None,
            patterns: None,
            violations: None,
            domain: None,
            requirements: None,
            metadata: HashMap::new(),
        };
        
        // Include version history
        if include_history {
            context.history = Some(self.build_version_history(block_id).await?);
        }
        
        // Include dependencies
        if include_dependencies {
            context.dependencies = Some(self.build_dependency_context(block_id).await?);
        }
        
        // Include consumers
        if include_consumers {
            context.consumers = Some(self.build_consumer_context(block_id).await?);
        }
        
        // Include architectural patterns
        if include_patterns {
            let (patterns, violations) = self.analyze_architectural_patterns(block_id).await?;
            context.patterns = Some(patterns);
            context.violations = Some(violations);
        }
        
        // Include domain context
        if include_domain {
            context.domain = Some(self.build_domain_context(&block).await?);
        }
        
        // Add metadata
        context.metadata.insert("context_built_at".to_string(), 
            serde_json::json!(chrono::Utc::now().to_rfc3339()));
        context.metadata.insert("block_complexity".to_string(), 
            serde_json::json!(self.calculate_complexity_score(&block)));
        
        Ok(context)
    }
    
    async fn create_block_context(&self, block: &Block) -> Result<BlockContext> {
        // Extract file path and line information from metadata
        let (file_path, line_range) = self.extract_location_info(block);
        
        Ok(BlockContext {
            id: block.id,
            semantic_name: block.semantic_name.clone(),
            block_type: block.block_type.clone(),
            complexity_score: Some(self.calculate_complexity_score(block)),
            language: self.extract_language_from_container(block.container_id).await?,
            file_path,
            line_range,
            abstract_syntax: block.abstract_syntax.clone(),
            parameters: block.parameters.clone(),
            return_type: block.return_type.clone(),
            modifiers: block.modifiers.clone(),
        })
    }
    
    async fn build_version_history(&self, block_id: Uuid) -> Result<VersionHistory> {
        let versions = self.db.get_block_versions(block_id).await?;
        
        let version_summaries: Vec<VersionSummary> = versions.iter()
            .take(10) // Limit to last 10 versions for context
            .map(|v| VersionSummary {
                version_number: v.version_number,
                created_at: v.created_at.to_rfc3339(),
                change_type: v.change_type.clone(),
                description: v.change_description.clone(),
                breaking_change: v.breaking_change,
                llm_provider: v.llm_provider.clone(),
                confidence_score: v.llm_temperature, // Using temperature as proxy for confidence
            })
            .collect();
        
        let evolution_trend = self.analyze_evolution_trend(&versions);
        let change_frequency = self.calculate_change_frequency(&versions);
        let stability_score = self.calculate_stability_score(&versions);
        
        Ok(VersionHistory {
            versions: version_summaries,
            evolution_trend,
            change_frequency,
            stability_score,
        })
    }
    
    async fn build_dependency_context(&self, block_id: Uuid) -> Result<DependencyContext> {
        // Get direct dependencies (blocks this block depends on)
        let direct_deps = self.get_direct_dependencies(block_id).await?;
        let direct_contexts = self.create_block_contexts_for_ids(&direct_deps).await?;
        
        // Get transitive dependencies
        let transitive_deps = self.get_transitive_dependencies(block_id).await?;
        let transitive_contexts = self.create_block_contexts_for_ids(&transitive_deps).await?;
        
        // Analyze dependency graph
        let dependency_graph = self.dependency_analyzer
            .analyze_dependencies(&[block_id]).await?;
        
        Ok(DependencyContext {
            direct_dependencies: direct_contexts,
            transitive_dependencies: transitive_contexts,
            dependency_depth: self.calculate_dependency_depth(block_id).await?,
            circular_dependencies: dependency_graph.circular_dependencies,
            critical_path: self.find_critical_path(block_id).await?,
        })
    }
    
    async fn build_consumer_context(&self, block_id: Uuid) -> Result<Vec<BlockContext>> {
        let consumer_ids = self.get_block_consumers(block_id).await?;
        self.create_block_contexts_for_ids(&consumer_ids).await
    }
    
    async fn analyze_architectural_patterns(&self, block_id: Uuid) -> Result<(Vec<ArchitecturalPattern>, Vec<ArchitecturalViolation>)> {
        let mut patterns = Vec::new();
        let mut violations = Vec::new();
        
        // Detect common patterns
        if let Some(pattern) = self.detect_singleton_pattern(block_id).await? {
            patterns.push(pattern);
        }
        
        if let Some(pattern) = self.detect_factory_pattern(block_id).await? {
            patterns.push(pattern);
        }
        
        if let Some(pattern) = self.detect_observer_pattern(block_id).await? {
            patterns.push(pattern);
        }
        
        // Check for violations
        if let Some(violation) = self.check_solid_principles(block_id).await? {
            violations.push(violation);
        }
        
        if let Some(violation) = self.check_coupling_violations(block_id).await? {
            violations.push(violation);
        }
        
        Ok((patterns, violations))
    }
    
    async fn build_domain_context(&self, block: &Block) -> Result<DomainContext> {
        // Extract domain information from block metadata and naming
        let domain_name = self.extract_domain_from_block(block);
        let business_concepts = self.extract_business_concepts(block);
        let domain_rules = self.extract_domain_rules(block);
        
        Ok(DomainContext {
            domain_name,
            business_concepts,
            domain_rules,
            bounded_context: self.determine_bounded_context(block),
            ubiquitous_language: self.build_ubiquitous_language(block),
        })
    }
    
    // Helper methods
    
    fn extract_location_info(&self, block: &Block) -> (Option<String>, Option<(u32, u32)>) {
        // Extract from metadata if available
        if let Some(metadata) = &block.metadata {
            let file_path = metadata.get("file_path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            
            let line_range = metadata.get("line_range")
                .and_then(|v| v.as_array())
                .and_then(|arr| {
                    if arr.len() == 2 {
                        let start = arr[0].as_u64()? as u32;
                        let end = arr[1].as_u64()? as u32;
                        Some((start, end))
                    } else {
                        None
                    }
                });
            
            (file_path, line_range)
        } else {
            (None, None)
        }
    }
    
    async fn extract_language_from_container(&self, container_id: Uuid) -> Result<Option<String>> {
        let container = self.db.get_container_by_id(container_id).await?;
        Ok(container.language)
    }
    
    fn calculate_complexity_score(&self, block: &Block) -> f64 {
        // Simple complexity calculation based on available metrics
        let mut score = 1.0;
        
        // Add complexity based on parameters
        if let Some(params) = &block.parameters {
            if let Some(param_array) = params.as_array() {
                score += param_array.len() as f64 * 0.1;
            }
        }
        
        // Add complexity based on modifiers
        if let Some(modifiers) = &block.modifiers {
            score += modifiers.len() as f64 * 0.05;
        }
        
        // Add complexity based on nesting level
        score += block.indent_level as f64 * 0.1;
        
        // Use stored complexity metrics if available
        if let Some(metrics) = &block.complexity_metrics {
            if let Some(cyclomatic) = metrics.get("cyclomatic_complexity") {
                if let Some(cc) = cyclomatic.as_f64() {
                    score += cc * 0.2;
                }
            }
        }
        
        score
    }
    
    fn analyze_evolution_trend(&self, versions: &[BlockVersion]) -> EvolutionTrend {
        if versions.len() < 2 {
            return EvolutionTrend::Stable;
        }
        
        let recent_changes = versions.iter().take(5).count();
        let breaking_changes = versions.iter()
            .take(10)
            .filter(|v| v.breaking_change)
            .count();
        
        match (recent_changes, breaking_changes) {
            (0..=1, 0) => EvolutionTrend::Stable,
            (2..=3, 0..=1) => EvolutionTrend::Growing,
            (_, 2..) => EvolutionTrend::Volatile,
            (4.., 0..=1) => EvolutionTrend::Refactoring,
            _ => EvolutionTrend::Declining,
        }
    }
    
    fn calculate_change_frequency(&self, versions: &[BlockVersion]) -> f64 {
        if versions.len() < 2 {
            return 0.0;
        }
        
        let first = &versions[versions.len() - 1];
        let last = &versions[0];
        
        let duration = last.created_at.signed_duration_since(first.created_at);
        let days = duration.num_days() as f64;
        
        if days > 0.0 {
            versions.len() as f64 / days
        } else {
            0.0
        }
    }
    
    fn calculate_stability_score(&self, versions: &[BlockVersion]) -> f64 {
        if versions.is_empty() {
            return 1.0;
        }
        
        let breaking_changes = versions.iter()
            .filter(|v| v.breaking_change)
            .count() as f64;
        
        let total_changes = versions.len() as f64;
        
        // Higher score means more stable (fewer breaking changes)
        1.0 - (breaking_changes / total_changes)
    }
    
    // Placeholder implementations for complex analysis methods
    
    async fn get_direct_dependencies(&self, block_id: Uuid) -> Result<Vec<Uuid>> {
        // Get direct dependencies from block relationships
        let dependencies = sqlx::query_scalar!(
            r#"
            SELECT DISTINCT target_block_id 
            FROM block_relationships 
            WHERE source_block_id = $1 
                AND relationship_type IN ('calls', 'depends_on', 'imports', 'uses')
            "#,
            block_id
        )
        .fetch_all(self.db.pool())
        .await?;

        Ok(dependencies.into_iter().filter_map(|id| Some(id)).collect())
    }
    
    async fn get_transitive_dependencies(&self, block_id: Uuid) -> Result<Vec<Uuid>> {
        // Implement transitive dependency analysis using recursive CTE
        let dependencies = sqlx::query_scalar!(
            r#"
            WITH RECURSIVE transitive_deps AS (
                -- Base case: direct dependencies
                SELECT target_block_id as dep_id, 1 as depth
                FROM block_relationships 
                WHERE source_block_id = $1 
                    AND relationship_type IN ('calls', 'depends_on', 'imports', 'uses')
                
                UNION
                
                -- Recursive case: dependencies of dependencies
                SELECT br.target_block_id, td.depth + 1
                FROM block_relationships br
                JOIN transitive_deps td ON br.source_block_id = td.dep_id
                WHERE td.depth < 5  -- Limit depth to prevent infinite recursion
                    AND br.relationship_type IN ('calls', 'depends_on', 'imports', 'uses')
            )
            SELECT DISTINCT dep_id FROM transitive_deps
            "#,
            block_id
        )
        .fetch_all(self.db.pool())
        .await?;

        Ok(dependencies.into_iter().filter_map(|id| id).collect())
    }
    
    async fn create_block_contexts_for_ids(&self, block_ids: &[Uuid]) -> Result<Vec<BlockContext>> {
        let mut contexts = Vec::new();
        for &block_id in block_ids {
            if let Ok(block) = self.db.get_block_by_id(block_id).await {
                contexts.push(self.create_block_context(&block).await?);
            }
        }
        Ok(contexts)
    }
    
    async fn calculate_dependency_depth(&self, block_id: Uuid) -> Result<u32> {
        // Calculate maximum dependency depth using recursive query
        let max_depth: Option<i32> = sqlx::query_scalar!(
            r#"
            WITH RECURSIVE dep_depth AS (
                -- Base case: the block itself
                SELECT $1::uuid as block_id, 0 as depth
                
                UNION
                
                -- Recursive case: dependencies at increasing depth
                SELECT br.target_block_id, dd.depth + 1
                FROM block_relationships br
                JOIN dep_depth dd ON br.source_block_id = dd.block_id
                WHERE dd.depth < 10  -- Prevent infinite recursion
                    AND br.relationship_type IN ('calls', 'depends_on', 'imports', 'uses')
            )
            SELECT MAX(depth) FROM dep_depth
            "#,
            block_id
        )
        .fetch_one(self.db.pool())
        .await?;

        Ok(max_depth.unwrap_or(0) as u32)
    }
    
    async fn find_critical_path(&self, block_id: Uuid) -> Result<Option<Vec<Uuid>>> {
        // Find the longest dependency path from this block
        let path_query = sqlx::query!(
            r#"
            WITH RECURSIVE critical_path AS (
                -- Base case: start from the given block
                SELECT $1::uuid as block_id, ARRAY[$1::uuid] as path, 0 as depth
                
                UNION
                
                -- Recursive case: extend path through dependencies
                SELECT br.target_block_id, 
                       cp.path || br.target_block_id,
                       cp.depth + 1
                FROM block_relationships br
                JOIN critical_path cp ON br.source_block_id = cp.block_id
                WHERE cp.depth < 10  -- Prevent infinite recursion
                    AND NOT (br.target_block_id = ANY(cp.path))  -- Prevent cycles
                    AND br.relationship_type IN ('calls', 'depends_on', 'imports', 'uses')
            )
            SELECT path FROM critical_path 
            ORDER BY array_length(path, 1) DESC 
            LIMIT 1
            "#,
            block_id
        )
        .fetch_optional(self.db.pool())
        .await?;

        if let Some(row) = path_query {
            if let Some(path) = row.path {
                return Ok(Some(path));
            }
        }

        Ok(None)
    }
    
    async fn get_block_consumers(&self, block_id: Uuid) -> Result<Vec<Uuid>> {
        // Find blocks that depend on this block (reverse dependencies)
        let consumers = sqlx::query_scalar!(
            r#"
            SELECT DISTINCT source_block_id 
            FROM block_relationships 
            WHERE target_block_id = $1 
                AND relationship_type IN ('calls', 'depends_on', 'imports', 'uses')
            "#,
            block_id
        )
        .fetch_all(self.db.pool())
        .await?;

        Ok(consumers)
    }
    
    async fn detect_singleton_pattern(&self, block_id: Uuid) -> Result<Option<ArchitecturalPattern>> {
        // Detect singleton pattern by analyzing block structure
        let block = self.db.get_block_by_id(block_id).await?;
        
        // Check for singleton characteristics in the abstract syntax
        let syntax_text = block.abstract_syntax.get("raw_text")
            .and_then(|v| v.as_str())
            .unwrap_or("");
            
        let has_private_constructor = syntax_text.contains("private") && 
            (syntax_text.contains("constructor") || syntax_text.contains("__init__"));
        let has_static_instance = syntax_text.contains("static") && 
            (syntax_text.contains("instance") || syntax_text.contains("_instance"));
        let has_get_instance = syntax_text.contains("getInstance") || 
            syntax_text.contains("get_instance");
            
        if has_private_constructor && has_static_instance && has_get_instance {
            Ok(Some(ArchitecturalPattern {
                pattern_type: "Singleton".to_string(),
                confidence: 0.8,
                description: "Classic singleton pattern with private constructor and static instance".to_string(),
                components: vec![block_id],
                benefits: vec![
                    "Controlled access to single instance".to_string(),
                    "Reduced memory footprint".to_string(),
                ],
                constraints: vec![
                    "Global state access".to_string(),
                    "Testing complexity".to_string(),
                ],
            }))
        } else {
            Ok(None)
        }
    }
    
    async fn detect_factory_pattern(&self, block_id: Uuid) -> Result<Option<ArchitecturalPattern>> {
        // Detect factory pattern by analyzing method signatures and return types
        let block = self.db.get_block_by_id(block_id).await?;
        
        let syntax_text = block.abstract_syntax.get("raw_text")
            .and_then(|v| v.as_str())
            .unwrap_or("");
            
        let has_create_method = syntax_text.contains("create") || 
            syntax_text.contains("make") || 
            syntax_text.contains("build");
        let has_factory_name = block.semantic_name
            .as_ref()
            .map(|name| name.to_lowercase().contains("factory"))
            .unwrap_or(false);
        let returns_different_types = syntax_text.contains("return new") || 
            syntax_text.contains("return ") && syntax_text.matches("return").count() > 1;
            
        if (has_create_method || has_factory_name) && returns_different_types {
            Ok(Some(ArchitecturalPattern {
                pattern_type: "Factory".to_string(),
                confidence: 0.7,
                description: "Factory pattern for object creation".to_string(),
                components: vec![block_id],
                benefits: vec![
                    "Encapsulated object creation".to_string(),
                    "Flexible product instantiation".to_string(),
                ],
                constraints: vec![
                    "Additional abstraction layer".to_string(),
                    "Potential over-engineering".to_string(),
                ],
            }))
        } else {
            Ok(None)
        }
    }
    
    async fn detect_observer_pattern(&self, block_id: Uuid) -> Result<Option<ArchitecturalPattern>> {
        // Detect observer pattern by looking for notification mechanisms
        let block = self.db.get_block_by_id(block_id).await?;
        
        let syntax_text = block.abstract_syntax.get("raw_text")
            .and_then(|v| v.as_str())
            .unwrap_or("");
            
        let has_observer_methods = syntax_text.contains("addObserver") || 
            syntax_text.contains("removeObserver") || 
            syntax_text.contains("notifyObservers") ||
            syntax_text.contains("subscribe") ||
            syntax_text.contains("unsubscribe") ||
            syntax_text.contains("notify");
            
        let has_observer_collection = syntax_text.contains("observers") || 
            syntax_text.contains("listeners") ||
            syntax_text.contains("subscribers");
            
        let has_update_method = syntax_text.contains("update") || 
            syntax_text.contains("onNotify") ||
            syntax_text.contains("handleEvent");
            
        if has_observer_methods && (has_observer_collection || has_update_method) {
            Ok(Some(ArchitecturalPattern {
                pattern_type: "Observer".to_string(),
                confidence: 0.75,
                description: "Observer pattern for event notification".to_string(),
                components: vec![block_id],
                benefits: vec![
                    "Observer management methods".to_string(),
                    "Dynamic subscription management".to_string(),
                ],
                constraints: vec![
                    "Potential memory leaks from unreleased observers".to_string(),
                    "Complex debugging of notification chains".to_string(),
                ],
            }))
        } else {
            Ok(None)
        }
    }
    
    async fn check_solid_principles(&self, block_id: Uuid) -> Result<Option<ArchitecturalViolation>> {
        // Check for SOLID principle violations
        let block = self.db.get_block_by_id(block_id).await?;
        
        let syntax_text = block.abstract_syntax.get("raw_text")
            .and_then(|v| v.as_str())
            .unwrap_or("");
            
        // Check Single Responsibility Principle (SRP)
        let method_count = syntax_text.matches("def ").count() + 
                          syntax_text.matches("function ").count() +
                          syntax_text.matches("fn ").count();
        let class_responsibilities = syntax_text.matches("class ").count();
        
        // Check for SRP violation (too many methods in a class)
        if method_count > 15 && class_responsibilities > 0 {
            return Ok(Some(ArchitecturalViolation {
                violation_type: "Single Responsibility Principle".to_string(),
                severity: ViolationSeverity::Medium,
                description: format!("Class has {} methods, suggesting multiple responsibilities", method_count),
                suggested_fixes: vec![ "Consider splitting this class into smaller, more focused classes".to_string(),
                ],
                affected_blocks: vec![block_id],
            }));
        }
        
        // Check Open/Closed Principle (OCP) - look for excessive conditionals
        let conditional_count = syntax_text.matches("if ").count() + 
                               syntax_text.matches("switch ").count() +
                               syntax_text.matches("match ").count();
        
        if conditional_count > 10 {
            return Ok(Some(ArchitecturalViolation {
                violation_type: "Open/Closed Principle".to_string(),
                severity: ViolationSeverity::Medium,
                description: format!("Excessive conditionals ({}) may indicate OCP violation", conditional_count),
                suggested_fixes: vec![ "Consider using polymorphism or strategy pattern instead of conditionals".to_string(),
                ],
                affected_blocks: vec![block_id],
            }));
        }
        
        Ok(None)
    }
    
    async fn check_coupling_violations(&self, block_id: Uuid) -> Result<Option<ArchitecturalViolation>> {
        // Check for high coupling violations
        let dependency_count = self.get_direct_dependencies(block_id).await?.len();
        let consumer_count = self.get_block_consumers(block_id).await?.len();
        
        // High efferent coupling (too many outgoing dependencies)
        if dependency_count > 10 {
            return Ok(Some(ArchitecturalViolation {
                violation_type: "High Efferent Coupling".to_string(),
                severity: ViolationSeverity::High,
                description: format!("Block depends on {} other blocks, indicating high coupling", dependency_count),
                suggested_fixes: vec![ "Consider using dependency injection or facade pattern to reduce coupling".to_string(),
                ],
                affected_blocks: vec![block_id],
            }));
        }
        
        // High afferent coupling (too many incoming dependencies)
        if consumer_count > 20 {
            return Ok(Some(ArchitecturalViolation {
                violation_type: "High Afferent Coupling".to_string(),
                severity: ViolationSeverity::Medium,
                description: format!("Block is used by {} other blocks, indicating potential instability", consumer_count),
                suggested_fixes: vec![ "Consider stabilizing this interface or splitting functionality".to_string(),
                ],
                affected_blocks: vec![block_id],
            }));
        }
        
        Ok(None)
    }
    
    fn extract_domain_from_block(&self, block: &Block) -> String {
        // Extract domain from semantic name or block type
        if let Some(name) = &block.semantic_name {
            if name.contains("User") || name.contains("Account") {
                return "User Management".to_string();
            } else if name.contains("Order") || name.contains("Payment") {
                return "E-commerce".to_string();
            } else if name.contains("Auth") || name.contains("Security") {
                return "Security".to_string();
            }
        }
        
        "General".to_string()
    }
    
    fn extract_business_concepts(&self, block: &Block) -> Vec<String> {
        let mut concepts = Vec::new();
        
        if let Some(name) = &block.semantic_name {
            let name_lower = name.to_lowercase();
            if name_lower.contains("user") { concepts.push("User".to_string()); }
            if name_lower.contains("order") { concepts.push("Order".to_string()); }
            if name_lower.contains("payment") { concepts.push("Payment".to_string()); }
            if name_lower.contains("product") { concepts.push("Product".to_string()); }
        }
        
        concepts
    }
    
    fn extract_domain_rules(&self, _block: &Block) -> Vec<String> {
        // TODO: Extract domain rules from block logic and comments
        Vec::new()
    }
    
    fn determine_bounded_context(&self, block: &Block) -> Option<String> {
        // Determine bounded context from file path or namespace
        if let Some(metadata) = &block.metadata {
            if let Some(file_path) = metadata.get("file_path").and_then(|v| v.as_str()) {
                if file_path.contains("/user/") {
                    return Some("User Context".to_string());
                } else if file_path.contains("/order/") {
                    return Some("Order Context".to_string());
                }
            }
        }
        
        None
    }
    
    fn build_ubiquitous_language(&self, _block: &Block) -> HashMap<String, String> {
        // TODO: Build ubiquitous language dictionary
        HashMap::new()
    }
}
