-- Migration 003: Database Schema Alignment
-- Phase 1A.1: Fix all struct/database mismatches
-- 
-- This migration aligns the database schema with the Rust SemanticBlock structure
-- to enable proper semantic-first development without compilation errors.

BEGIN;

-- Add missing semantic model columns to blocks table
ALTER TABLE blocks ADD COLUMN IF NOT EXISTS syntax_preservation JSONB DEFAULT '{}';
ALTER TABLE blocks ADD COLUMN IF NOT EXISTS structural_context JSONB DEFAULT '{}';
ALTER TABLE blocks ADD COLUMN IF NOT EXISTS template_metadata JSONB DEFAULT '{}';
ALTER TABLE blocks ADD COLUMN IF NOT EXISTS generation_hints JSONB DEFAULT '{}';
ALTER TABLE blocks ADD COLUMN IF NOT EXISTS source_language VARCHAR(50);

-- Add position and hierarchical enhancement columns
ALTER TABLE blocks ADD COLUMN IF NOT EXISTS position_metadata JSONB DEFAULT '{}';
ALTER TABLE blocks ADD COLUMN IF NOT EXISTS hierarchical_index INTEGER DEFAULT 0;
ALTER TABLE blocks ADD COLUMN IF NOT EXISTS depth_level INTEGER DEFAULT 0;

-- Create missing tables for complete semantic model support

-- Create table for tracking semantic block relationships with enhanced metadata
CREATE TABLE IF NOT EXISTS enhanced_block_relationships (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    source_block_id UUID NOT NULL REFERENCES blocks(id) ON DELETE CASCADE,
    target_block_id UUID NOT NULL REFERENCES blocks(id) ON DELETE CASCADE,
    relationship_type TEXT NOT NULL,
    relationship_strength REAL DEFAULT 1.0,
    bidirectional BOOLEAN DEFAULT FALSE,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(source_block_id, target_block_id, relationship_type)
);

-- Create table for semantic block templates
CREATE TABLE IF NOT EXISTS block_templates (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    block_type TEXT NOT NULL,
    language TEXT NOT NULL,
    template_content JSONB NOT NULL,
    variables JSONB DEFAULT '{}',
    constraints JSONB DEFAULT '{}',
    examples JSONB DEFAULT '{}',
    effectiveness_score REAL DEFAULT 0.0,
    usage_count INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(name, block_type, language)
);

-- Create table for semantic reconstruction metadata
CREATE TABLE IF NOT EXISTS reconstruction_metadata (
    block_id UUID PRIMARY KEY REFERENCES blocks(id) ON DELETE CASCADE,
    reconstruction_quality REAL DEFAULT 0.0,
    template_id UUID REFERENCES block_templates(id),
    reconstruction_hints JSONB DEFAULT '{}',
    formatting_preferences JSONB DEFAULT '{}',
    last_reconstructed_at TIMESTAMPTZ,
    reconstruction_count INTEGER DEFAULT 0,
    validation_errors JSONB DEFAULT '[]',
    metadata JSONB DEFAULT '{}'
);

-- Create table for tracking block evolution and transformation history
CREATE TABLE IF NOT EXISTS block_evolution (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    block_id UUID NOT NULL REFERENCES blocks(id) ON DELETE CASCADE,
    transformation_type TEXT NOT NULL,
    before_snapshot JSONB NOT NULL,
    after_snapshot JSONB NOT NULL,
    transformation_metadata JSONB DEFAULT '{}',
    applied_by TEXT,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    parent_evolution_id UUID REFERENCES block_evolution(id),
    
    -- Transformation tracking
    semantic_diff JSONB DEFAULT '{}',
    impact_analysis JSONB DEFAULT '{}',
    rollback_information JSONB DEFAULT '{}'
);

-- Add indexes for performance optimization
CREATE INDEX IF NOT EXISTS idx_blocks_syntax_preservation ON blocks USING GIN(syntax_preservation);
CREATE INDEX IF NOT EXISTS idx_blocks_structural_context ON blocks USING GIN(structural_context);
CREATE INDEX IF NOT EXISTS idx_blocks_template_metadata ON blocks USING GIN(template_metadata);
CREATE INDEX IF NOT EXISTS idx_blocks_source_language ON blocks(source_language);
CREATE INDEX IF NOT EXISTS idx_blocks_hierarchical_index ON blocks(hierarchical_index);
CREATE INDEX IF NOT EXISTS idx_blocks_depth_level ON blocks(depth_level);

-- Enhanced relationship indexes
CREATE INDEX IF NOT EXISTS idx_enhanced_relationships_source ON enhanced_block_relationships(source_block_id);
CREATE INDEX IF NOT EXISTS idx_enhanced_relationships_target ON enhanced_block_relationships(target_block_id);
CREATE INDEX IF NOT EXISTS idx_enhanced_relationships_type ON enhanced_block_relationships(relationship_type);
CREATE INDEX IF NOT EXISTS idx_enhanced_relationships_strength ON enhanced_block_relationships(relationship_strength);

-- Template and reconstruction indexes
CREATE INDEX IF NOT EXISTS idx_block_templates_type_lang ON block_templates(block_type, language);
CREATE INDEX IF NOT EXISTS idx_block_templates_effectiveness ON block_templates(effectiveness_score);
CREATE INDEX IF NOT EXISTS idx_reconstruction_quality ON reconstruction_metadata(reconstruction_quality);
CREATE INDEX IF NOT EXISTS idx_block_evolution_block_id ON block_evolution(block_id);
CREATE INDEX IF NOT EXISTS idx_block_evolution_type ON block_evolution(transformation_type);
CREATE INDEX IF NOT EXISTS idx_block_evolution_applied_at ON block_evolution(applied_at);

-- Create view for complete semantic block representation
CREATE OR REPLACE VIEW complete_semantic_blocks AS
SELECT 
    b.*,
    rm.reconstruction_quality,
    rm.template_id,
    rm.reconstruction_hints as reconstruction_metadata_hints,
    rm.formatting_preferences as reconstruction_formatting,
    bt.name as template_name,
    bt.template_content,
    COUNT(er.source_block_id) as outgoing_relationships,
    COUNT(er2.target_block_id) as incoming_relationships
FROM blocks b
LEFT JOIN reconstruction_metadata rm ON b.id = rm.block_id
LEFT JOIN block_templates bt ON rm.template_id = bt.id
LEFT JOIN enhanced_block_relationships er ON b.id = er.source_block_id
LEFT JOIN enhanced_block_relationships er2 ON b.id = er2.target_block_id
GROUP BY b.id, rm.reconstruction_quality, rm.template_id, rm.reconstruction_hints, 
         rm.formatting_preferences, bt.name, bt.template_content;

-- Create view for hierarchical block tree
CREATE OR REPLACE VIEW block_hierarchy AS
WITH RECURSIVE block_tree AS (
    -- Base case: root blocks (no parent)
    SELECT 
        id,
        container_id,
        block_type,
        semantic_name,
        parent_block_id,
        position_in_parent,
        depth_level,
        ARRAY[id] as path,
        0 as level
    FROM blocks 
    WHERE parent_block_id IS NULL
    
    UNION ALL
    
    -- Recursive case: child blocks
    SELECT 
        b.id,
        b.container_id,
        b.block_type,
        b.semantic_name,
        b.parent_block_id,
        b.position_in_parent,
        b.depth_level,
        bt.path || b.id,
        bt.level + 1
    FROM blocks b
    INNER JOIN block_tree bt ON b.parent_block_id = bt.id
)
SELECT * FROM block_tree
ORDER BY container_id, path;

-- Add comments for documentation
COMMENT ON TABLE enhanced_block_relationships IS 'Enhanced semantic relationships between blocks with strength metrics';
COMMENT ON TABLE block_templates IS 'Templates for generating code from semantic blocks';
COMMENT ON TABLE reconstruction_metadata IS 'Metadata for reconstructing source code from semantic blocks';
COMMENT ON TABLE block_evolution IS 'History of block transformations and semantic changes';

COMMENT ON VIEW complete_semantic_blocks IS 'Complete semantic block view with reconstruction and relationship metadata';
COMMENT ON VIEW block_hierarchy IS 'Hierarchical tree view of semantic blocks with path tracking';

-- Update schema version tracking
INSERT INTO migrations (repository_name, source_language, target_language, status, metadata)
VALUES (
    'schema_alignment_003',
    'sql',
    'sql', 
    'completed',
    '{"migration_type": "schema_alignment", "version": "1A.1", "description": "Database schema alignment with Rust structs"}'
) ON CONFLICT DO NOTHING;

COMMIT;