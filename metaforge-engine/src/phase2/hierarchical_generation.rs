// Hierarchical Generation: Generate code from hierarchical block structure
// Following ARCHITECT principle: Build from known-working foundations

use anyhow::Result;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::database::{Database, Container, Block};
use crate::generator::templates::TemplateEngine;
use std::collections::{HashMap, HashSet};
use petgraph::{Graph, Directed, graph::NodeIndex};
use petgraph::algo::toposort;

pub struct HierarchicalGenerator {
    db: Database,
    template_engine: TemplateEngine,
}

impl HierarchicalGenerator {
    pub fn new(db: Database) -> Self {
        Self {
            template_engine: TemplateEngine::new(),
            db,
        }
    }

    /// Generate code from hierarchical block structure following the specified algorithm
    pub async fn generate_hierarchical(&self, container_id: Uuid) -> Result<GenerationResult> {
        let mut result = GenerationResult::new(container_id);
        
        // Step 1: Build block hierarchy tree from parent_block_id relationships
        println!("Building block hierarchy tree...");
        let hierarchy = self.build_block_hierarchy(container_id).await?;
        result.total_blocks = hierarchy.blocks.len();
        
        // Step 2: Topologically sort blocks by dependency
        println!("Performing topological sort of dependencies...");
        let sorted_blocks = self.topologically_sort_blocks(&hierarchy).await?;
        result.dependency_order = sorted_blocks.iter().map(|b| b.id).collect();
        
        // Step 3: Generate imports first, then top-level blocks
        println!("Generating imports and top-level structures...");
        let imports = self.generate_imports(&hierarchy, &sorted_blocks).await?;
        result.imports_generated = imports.len();
        
        // Step 4: Recursively generate child blocks with proper indentation
        println!("Recursively generating hierarchical code...");
        let code_sections = self.generate_recursive_hierarchy(&hierarchy, &sorted_blocks).await?;
        result.code_sections = code_sections.len();
        
        // Step 5: Apply language-specific formatting rules
        println!("Applying language-specific formatting...");
        let formatted_code = self.apply_formatting(&hierarchy, &imports, &code_sections).await?;
        result.generated_code = formatted_code;
        result.formatting_applied = true;
        
        // Step 6: Validate generation quality
        let quality = self.validate_generation_quality(&result).await?;
        result.quality_score = quality.score;
        result.validation_passed = quality.passed;
        
        if result.validation_passed {
            result.status = GenerationStatus::Completed;
        } else {
            result.status = GenerationStatus::Failed;
            result.error_message = Some(format!("Quality validation failed: {}", quality.reason));
        }
        
        Ok(result)
    }

    /// Build block hierarchy tree from parent_block_id relationships
    async fn build_block_hierarchy(&self, container_id: Uuid) -> Result<BlockHierarchy> {
        // Get container information
        let container_query = "SELECT * FROM containers WHERE id = $1";
        let container: Container = sqlx::query_as(container_query)
            .bind(container_id)
            .fetch_one(self.db.pool())
            .await?;

        // Get all blocks for this container
        let blocks_query = r#"
            SELECT * FROM blocks 
            WHERE container_id = $1 
            ORDER BY position, hierarchical_index, parent_block_id NULLS FIRST
        "#;
        let blocks: Vec<Block> = sqlx::query_as(blocks_query)
            .bind(container_id)
            .fetch_all(self.db.pool())
            .await?;

        // Build parent-child relationships
        let mut hierarchy = BlockHierarchy::new(container);
        let mut parent_map: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        let mut child_map: HashMap<Uuid, Uuid> = HashMap::new();

        for block in &blocks {
            hierarchy.blocks.insert(block.id, block.clone());
            
            if let Some(parent_id) = block.parent_block_id {
                parent_map.entry(parent_id).or_insert_with(Vec::new).push(block.id);
                child_map.insert(block.id, parent_id);
            } else {
                hierarchy.root_blocks.push(block.id);
            }
        }

        hierarchy.parent_children = parent_map;
        hierarchy.child_parent = child_map;

        // Detect circular dependencies
        self.detect_circular_dependencies(&hierarchy)?;

        Ok(hierarchy)
    }

    /// Topologically sort blocks by dependency
    async fn topologically_sort_blocks(&self, hierarchy: &BlockHierarchy) -> Result<Vec<Block>> {
        // Create dependency graph
        let mut graph = Graph::<Uuid, (), Directed>::new();
        let mut node_map: HashMap<Uuid, NodeIndex> = HashMap::new();

        // Add all blocks as nodes
        for block_id in hierarchy.blocks.keys() {
            let node_idx = graph.add_node(*block_id);
            node_map.insert(*block_id, node_idx);
        }

        // Add edges for dependencies
        for (child_id, parent_id) in &hierarchy.child_parent {
            if let (Some(&child_node), Some(&parent_node)) = (node_map.get(child_id), node_map.get(parent_id)) {
                // Parent must come before child
                graph.add_edge(parent_node, child_node, ());
            }
        }

        // Add import dependencies
        self.add_import_dependencies(&mut graph, &node_map, hierarchy).await?;

        // Perform topological sort
        match toposort(&graph, None) {
            Ok(sorted_nodes) => {
                let mut sorted_blocks = Vec::new();
                for node_idx in sorted_nodes {
                    let block_id = graph[node_idx];
                    if let Some(block) = hierarchy.blocks.get(&block_id) {
                        sorted_blocks.push(block.clone());
                    }
                }
                Ok(sorted_blocks)
            }
            Err(_) => {
                Err(anyhow::anyhow!("Circular dependency detected in block hierarchy"))
            }
        }
    }

    /// Generate imports first
    async fn generate_imports(&self, hierarchy: &BlockHierarchy, sorted_blocks: &[Block]) -> Result<Vec<String>> {
        let mut imports = Vec::new();
        
        // Extract import blocks and organize by type
        let import_blocks: Vec<&Block> = sorted_blocks.iter()
            .filter(|block| block.block_type == "Import")
            .collect();

        if let Some(language) = &hierarchy.container.language {
            // Categorize imports (stdlib, third-party, local)
            let categorized = self.categorize_imports(&import_blocks, language)?;
            
            // Generate imports in correct order based on language conventions
            for category in &["stdlib", "third_party", "local"] {
                if let Some(category_imports) = categorized.get(*category) {
                    for import_block in category_imports {
                        let import_code = self.template_engine.render_block(import_block, language)?;
                        imports.push(import_code);
                    }
                    
                    // Add blank line between categories
                    if !category_imports.is_empty() && *category != "local" {
                        imports.push(String::new());
                    }
                }
            }
        }

        Ok(imports)
    }

    /// Recursively generate child blocks with proper indentation
    async fn generate_recursive_hierarchy(&self, hierarchy: &BlockHierarchy, sorted_blocks: &[Block]) -> Result<Vec<CodeSection>> {
        let mut sections = Vec::new();
        
        if let Some(language) = &hierarchy.container.language {
            // Process root blocks first
            for &root_id in &hierarchy.root_blocks {
                if let Some(root_block) = hierarchy.blocks.get(&root_id) {
                    if root_block.block_type != "Import" { // Imports already handled
                        let section = self.generate_block_with_children(
                            root_block, 
                            hierarchy, 
                            language, 
                            0 // root level indentation
                        ).await?;
                        sections.push(section);
                    }
                }
            }
        }

        Ok(sections)
    }

    /// Generate a block and all its children recursively
    async fn generate_block_with_children(
        &self, 
        block: &Block, 
        hierarchy: &BlockHierarchy, 
        language: &str, 
        indent_level: usize
    ) -> Result<CodeSection> {
        let mut section = CodeSection::new(block.id, indent_level);
        
        // Generate the block itself
        let block_code = self.template_engine.render_block(block, language)?;
        section.content = self.apply_indentation(&block_code, indent_level);
        
        // Get children in correct order
        if let Some(children) = hierarchy.parent_children.get(&block.id) {
            let mut sorted_children: Vec<&Block> = children.iter()
                .filter_map(|child_id| hierarchy.blocks.get(child_id))
                .collect();
            
            // Sort children by position_in_parent
            sorted_children.sort_by_key(|child| child.position_in_parent);
            
            // Recursively generate children
            for child in sorted_children {
                let child_section = Box::pin(self.generate_block_with_children(
                    child, 
                    hierarchy, 
                    language, 
                    indent_level + 1
                )).await?;
                section.children.push(child_section);
            }
        }
        
        Ok(section)
    }

    /// Apply language-specific formatting rules
    async fn apply_formatting(
        &self, 
        hierarchy: &BlockHierarchy, 
        imports: &[String], 
        code_sections: &[CodeSection]
    ) -> Result<String> {
        let mut final_code = String::new();
        
        if let Some(language) = &hierarchy.container.language {
            // Add file header if specified
            let header = self.get_file_header(language);
            if !header.is_empty() {
                final_code.push_str(&header);
                final_code.push('\n');
            }
            
            // Add imports
            for import in imports {
                final_code.push_str(import);
                final_code.push('\n');
            }
            
            // Add blank line after imports if they exist
            if !imports.is_empty() {
                final_code.push('\n');
            }
            
            // Add code sections
            for (i, section) in code_sections.iter().enumerate() {
                final_code.push_str(&self.render_code_section(section)?);
                
                // Add blank line between top-level sections
                if i < code_sections.len() - 1 {
                    final_code.push('\n');
                }
            }
            
            // Apply external formatter if available
            let formatted = self.template_engine.format_code(&final_code, language)?;
            Ok(formatted)
        } else {
            Err(anyhow::anyhow!("No language specified for container"))
        }
    }

    /// Validate generation quality
    async fn validate_generation_quality(&self, result: &GenerationResult) -> Result<QualityValidation> {
        let mut validation = QualityValidation::new();
        
        // Check 1: Generated code is not empty
        if result.generated_code.is_empty() {
            validation.passed = false;
            validation.reason = "Generated code is empty".to_string();
            return Ok(validation);
        }
        
        // Check 2: All blocks were processed
        if result.code_sections == 0 {
            validation.passed = false;
            validation.reason = "No code sections generated".to_string();
            return Ok(validation);
        }
        
        // Check 3: Syntax validation (basic)
        let syntax_valid = self.validate_basic_syntax(&result.generated_code).await?;
        if !syntax_valid {
            validation.passed = false;
            validation.reason = "Generated code has syntax errors".to_string();
            return Ok(validation);
        }
        
        // Calculate quality score
        validation.score = self.calculate_quality_score(result);
        validation.passed = validation.score >= 0.8; // 80% threshold
        
        if !validation.passed {
            validation.reason = format!("Quality score {:.1}% below 80% threshold", validation.score * 100.0);
        }
        
        Ok(validation)
    }

    // Helper methods

    fn detect_circular_dependencies(&self, hierarchy: &BlockHierarchy) -> Result<()> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        
        for &root_id in &hierarchy.root_blocks {
            if self.has_cycle(root_id, hierarchy, &mut visited, &mut rec_stack) {
                return Err(anyhow::anyhow!("Circular dependency detected in block hierarchy"));
            }
        }
        
        Ok(())
    }

    fn has_cycle(
        &self, 
        block_id: Uuid, 
        hierarchy: &BlockHierarchy, 
        visited: &mut HashSet<Uuid>, 
        rec_stack: &mut HashSet<Uuid>
    ) -> bool {
        if rec_stack.contains(&block_id) {
            return true; // Back edge found, cycle detected
        }
        
        if visited.contains(&block_id) {
            return false; // Already processed
        }
        
        visited.insert(block_id);
        rec_stack.insert(block_id);
        
        if let Some(children) = hierarchy.parent_children.get(&block_id) {
            for &child_id in children {
                if self.has_cycle(child_id, hierarchy, visited, rec_stack) {
                    return true;
                }
            }
        }
        
        rec_stack.remove(&block_id);
        false
    }

    async fn add_import_dependencies(
        &self, 
        graph: &mut Graph<Uuid, (), Directed>, 
        node_map: &HashMap<Uuid, NodeIndex>,
        hierarchy: &BlockHierarchy
    ) -> Result<()> {
        // Add dependencies based on import relationships
        // This is a simplified implementation - full version would analyze actual dependencies
        
        for (block_id, block) in &hierarchy.blocks {
            if block.block_type == "Import" {
                // Imports should come before all other blocks
                for (other_id, other_block) in &hierarchy.blocks {
                    if other_block.block_type != "Import" && block_id != other_id {
                        if let (Some(&import_node), Some(&other_node)) = (node_map.get(block_id), node_map.get(other_id)) {
                            graph.add_edge(import_node, other_node, ());
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    fn categorize_imports<'a>(&self, import_blocks: &[&'a Block], language: &str) -> Result<HashMap<String, Vec<&'a Block>>> {
        let mut categorized: HashMap<String, Vec<&'a Block>> = HashMap::new();
        
        for &import_block in import_blocks {
            let category = self.classify_import(import_block, language)?;
            categorized.entry(category).or_insert_with(Vec::new).push(import_block);
        }
        
        // Sort within categories
        for (_, imports) in categorized.iter_mut() {
            imports.sort_by(|a, b| {
                let a_name = a.semantic_name.as_deref().unwrap_or("");
                let b_name = b.semantic_name.as_deref().unwrap_or("");
                a_name.cmp(b_name)
            });
        }
        
        Ok(categorized)
    }

    fn classify_import(&self, import_block: &Block, language: &str) -> Result<String> {
        // Simplified import classification
        if let Some(import_path) = import_block.semantic_name.as_ref() {
            match language {
                "python" => {
                    if import_path.starts_with("std") || 
                       ["os", "sys", "json", "re", "datetime"].iter().any(|std| import_path.contains(std)) {
                        Ok("stdlib".to_string())
                    } else if import_path.starts_with('.') {
                        Ok("local".to_string())
                    } else {
                        Ok("third_party".to_string())
                    }
                }
                "rust" => {
                    if import_path.starts_with("std::") {
                        Ok("stdlib".to_string())
                    } else if import_path.starts_with("crate::") || import_path.starts_with("super::") {
                        Ok("local".to_string())
                    } else {
                        Ok("third_party".to_string())
                    }
                }
                "javascript" | "typescript" => {
                    if import_path.starts_with('.') {
                        Ok("local".to_string())
                    } else if import_path.starts_with('@') || !import_path.contains('/') {
                        Ok("third_party".to_string())
                    } else {
                        Ok("stdlib".to_string())
                    }
                }
                _ => Ok("unknown".to_string()),
            }
        } else {
            Ok("unknown".to_string())
        }
    }

    fn apply_indentation(&self, code: &str, indent_level: usize) -> String {
        let indent = "    ".repeat(indent_level); // 4 spaces per level
        code.lines()
            .map(|line| {
                if line.trim().is_empty() {
                    line.to_string()
                } else {
                    format!("{}{}", indent, line)
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn get_file_header(&self, language: &str) -> String {
        match language {
            "python" => "#!/usr/bin/env python3\n# -*- coding: utf-8 -*-".to_string(),
            "rust" => "#![allow(unused)]".to_string(),
            "javascript" => "'use strict';".to_string(),
            "typescript" => "// TypeScript".to_string(),
            _ => String::new(),
        }
    }

    fn render_code_section(&self, section: &CodeSection) -> Result<String> {
        let mut output = String::new();
        
        // Add the section's content
        output.push_str(&section.content);
        
        // Add children
        for child in &section.children {
            output.push('\n');
            output.push_str(&self.render_code_section(child)?);
        }
        
        Ok(output)
    }

    async fn validate_basic_syntax(&self, _code: &str) -> Result<bool> {
        // Simplified syntax validation
        // Full implementation would use language-specific parsers
        Ok(true)
    }

    fn calculate_quality_score(&self, result: &GenerationResult) -> f64 {
        let mut score = 0.0;
        let mut weights = 0.0;
        
        // Code generation success (weight: 40%)
        if !result.generated_code.is_empty() {
            score += 0.4;
        }
        weights += 0.4;
        
        // Import handling (weight: 20%)
        if result.imports_generated > 0 {
            score += 0.2;
        }
        weights += 0.2;
        
        // Hierarchical structure (weight: 20%)
        if result.code_sections > 0 {
            score += 0.2;
        }
        weights += 0.2;
        
        // Formatting applied (weight: 20%)
        if result.formatting_applied {
            score += 0.2;
        }
        weights += 0.2;
        
        if weights > 0.0 {
            score / weights
        } else {
            0.0
        }
    }
}

// Data structures for hierarchical generation

#[derive(Debug, Clone)]
pub struct BlockHierarchy {
    pub container: Container,
    pub blocks: HashMap<Uuid, Block>,
    pub root_blocks: Vec<Uuid>,
    pub parent_children: HashMap<Uuid, Vec<Uuid>>,
    pub child_parent: HashMap<Uuid, Uuid>,
}

impl BlockHierarchy {
    fn new(container: Container) -> Self {
        Self {
            container,
            blocks: HashMap::new(),
            root_blocks: Vec::new(),
            parent_children: HashMap::new(),
            child_parent: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResult {
    pub container_id: Uuid,
    pub status: GenerationStatus,
    pub total_blocks: usize,
    pub dependency_order: Vec<Uuid>,
    pub imports_generated: usize,
    pub code_sections: usize,
    pub generated_code: String,
    pub formatting_applied: bool,
    pub quality_score: f64,
    pub validation_passed: bool,
    pub error_message: Option<String>,
}

impl GenerationResult {
    fn new(container_id: Uuid) -> Self {
        Self {
            container_id,
            status: GenerationStatus::InProgress,
            total_blocks: 0,
            dependency_order: Vec::new(),
            imports_generated: 0,
            code_sections: 0,
            generated_code: String::new(),
            formatting_applied: false,
            quality_score: 0.0,
            validation_passed: false,
            error_message: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GenerationStatus {
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Clone)]
struct CodeSection {
    block_id: Uuid,
    indent_level: usize,
    content: String,
    children: Vec<CodeSection>,
}

impl CodeSection {
    fn new(block_id: Uuid, indent_level: usize) -> Self {
        Self {
            block_id,
            indent_level,
            content: String::new(),
            children: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
struct QualityValidation {
    passed: bool,
    score: f64,
    reason: String,
}

impl QualityValidation {
    fn new() -> Self {
        Self {
            passed: true,
            score: 0.0,
            reason: String::new(),
        }
    }
}
