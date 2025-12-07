use std::collections::HashMap;
use uuid::Uuid;
use crate::core::*;

#[derive(Debug, Clone)]
pub struct ExtractionContext {
    parent_stack: Vec<Uuid>,
    blocks: Vec<SemanticBlock>,
    relationships: Vec<BlockRelationship>,
    symbol_table: HashMap<String, Uuid>,
    current_scope: ScopeInfo,
    position_counter: usize,
}

#[derive(Debug, Clone)]
pub struct BlockRelationship {
    pub source_block_id: Uuid,
    pub target_block_id: Uuid,
    pub target_name: Option<String>, // For unresolved relationships
    pub relationship_type: RelationshipType,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum RelationshipType {
    Calls,
    Inherits,
    Implements,
    Uses,
    Imports,
    Exports,
    Contains,
    References,
}

#[derive(Debug, Clone)]
pub struct ParseResult {
    pub blocks: Vec<SemanticBlock>,
    pub relationships: Vec<BlockRelationship>,
    #[allow(dead_code)]
    pub symbol_table: HashMap<String, Uuid>,
}

impl ExtractionContext {
    pub fn new() -> Self {
        Self {
            parent_stack: Vec::new(),
            blocks: Vec::new(),
            relationships: Vec::new(),
            symbol_table: HashMap::new(),
            current_scope: ScopeInfo::Module("main".to_string()),
            position_counter: 0,
        }
    }
    
    pub fn enter_block(&mut self, mut block: SemanticBlock) -> Uuid {
        // Set parent from stack
        block.structural_context.parent_block = self.parent_stack.last().cloned();
        
        // Set position in parent
        let _position_in_parent = self.count_siblings(&block.structural_context.parent_block);
        
        let block_id = block.id;
        
        // Register in symbol table
        if !block.semantic_identity.canonical_name.is_empty() {
            self.symbol_table.insert(block.semantic_identity.canonical_name.clone(), block_id);
        }
        
        // Update position
        block.position.index = self.position_counter;
        self.position_counter += 1;
        
        // Push to stack if this creates a scope
        if self.creates_scope(&block) {
            self.parent_stack.push(block_id);
            self.update_scope(&block);
        }
        
        self.blocks.push(block);
        block_id
    }
    
    pub fn exit_block(&mut self, block_id: Uuid) {
        if self.parent_stack.last() == Some(&block_id) {
            self.parent_stack.pop();
            // Restore previous scope
            if let Some(&parent_id) = self.parent_stack.last() {
                if let Some(parent_block) = self.blocks.iter().find(|b| b.id == parent_id) {
                    self.current_scope = parent_block.structural_context.scope.clone();
                }
            } else {
                self.current_scope = ScopeInfo::Module("main".to_string());
            }
        }
    }
    
    pub fn add_relationship(&mut self, source: Uuid, target_name: &str, rel_type: RelationshipType) {
        // Queue for resolution in second pass
        self.relationships.push(BlockRelationship {
            source_block_id: source,
            target_block_id: Uuid::nil(), // Will resolve later
            target_name: Some(target_name.to_string()),
            relationship_type: rel_type,
            metadata: HashMap::new(),
        });
    }
    
    #[allow(dead_code)]
    pub fn add_direct_relationship(&mut self, source: Uuid, target: Uuid, rel_type: RelationshipType) {
        self.relationships.push(BlockRelationship {
            source_block_id: source,
            target_block_id: target,
            target_name: None,
            relationship_type: rel_type,
            metadata: HashMap::new(),
        });
    }
    
    pub fn resolve_relationships(&mut self) {
        for rel in &mut self.relationships {
            if let Some(name) = &rel.target_name {
                if let Some(&target_id) = self.symbol_table.get(name) {
                    rel.target_block_id = target_id;
                }
            }
        }
        // Remove unresolved relationships
        self.relationships.retain(|r| r.target_block_id != Uuid::nil());
    }
    
    #[allow(dead_code)]
    pub fn get_current_parent(&self) -> Option<Uuid> {
        self.parent_stack.last().cloned()
    }
    
    #[allow(dead_code)]
    pub fn get_current_scope(&self) -> &ScopeInfo {
        &self.current_scope
    }
    
    pub fn finish(mut self) -> ParseResult {
        self.resolve_relationships();
        ParseResult {
            blocks: self.blocks,
            relationships: self.relationships,
            symbol_table: self.symbol_table,
        }
    }
    
    fn count_siblings(&self, parent_id: &Option<Uuid>) -> usize {
        self.blocks.iter()
            .filter(|b| b.structural_context.parent_block == *parent_id)
            .count()
    }
    
    fn creates_scope(&self, block: &SemanticBlock) -> bool {
        matches!(
            block.block_type,
            BlockType::Function | BlockType::Class | BlockType::Interface | BlockType::Module
        )
    }
    
    fn update_scope(&mut self, block: &SemanticBlock) {
        self.current_scope = match block.block_type {
            BlockType::Function => ScopeInfo::Function(block.semantic_identity.canonical_name.clone()),
            BlockType::Class => ScopeInfo::Class(block.semantic_identity.canonical_name.clone()),
            BlockType::Module => ScopeInfo::Module(block.semantic_identity.canonical_name.clone()),
            _ => ScopeInfo::Block,
        };
    }
}

impl ParseResult {
    pub fn resolve_relationships(&mut self) {
        // This is already done in ExtractionContext::finish()
        // No additional work needed here
    }
}

impl Default for ExtractionContext {
    fn default() -> Self {
        Self::new()
    }
}

// Helper trait for language extractors
pub trait LanguageExtractor {
    fn extract_with_context(
        &self,
        root: tree_sitter::Node,
        source: &str,
        file_path: &str,
    ) -> anyhow::Result<ParseResult>;
}

impl std::fmt::Display for RelationshipType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelationshipType::Calls => write!(f, "calls"),
            RelationshipType::Inherits => write!(f, "inherits"),
            RelationshipType::Implements => write!(f, "implements"),
            RelationshipType::Uses => write!(f, "uses"),
            RelationshipType::Imports => write!(f, "imports"),
            RelationshipType::Exports => write!(f, "exports"),
            RelationshipType::Contains => write!(f, "contains"),
            RelationshipType::References => write!(f, "references"),
        }
    }
}
