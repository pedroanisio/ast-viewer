-- Initial schema for block-migrate
-- Based on the Rust structs in src/database/schema.rs

BEGIN;

-- Create extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Containers table
CREATE TABLE IF NOT EXISTS containers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    container_type TEXT NOT NULL,
    language TEXT,
    original_path TEXT,
    original_hash TEXT,
    source_code TEXT,
    version INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Blocks table
CREATE TABLE IF NOT EXISTS blocks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    container_id UUID NOT NULL REFERENCES containers(id) ON DELETE CASCADE,
    block_type TEXT NOT NULL,
    semantic_name TEXT,
    abstract_syntax JSONB NOT NULL DEFAULT '{}',
    position INTEGER NOT NULL,
    indent_level INTEGER NOT NULL DEFAULT 0,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Hierarchical fields
    parent_block_id UUID REFERENCES blocks(id) ON DELETE CASCADE,
    position_in_parent INTEGER NOT NULL DEFAULT 0,
    parameters JSONB,
    return_type TEXT,
    modifiers TEXT[],
    decorators JSONB,
    body_ast JSONB,
    language_ast JSONB,
    language_features JSONB,
    complexity_metrics JSONB,
    scope_info JSONB
);

-- Block relationships table
CREATE TABLE IF NOT EXISTS block_relationships (
    source_block_id UUID NOT NULL REFERENCES blocks(id) ON DELETE CASCADE,
    target_block_id UUID NOT NULL REFERENCES blocks(id) ON DELETE CASCADE,
    relationship_type TEXT NOT NULL,
    metadata JSONB,
    PRIMARY KEY (source_block_id, target_block_id, relationship_type)
);

-- Prompt templates table
CREATE TABLE IF NOT EXISTS prompt_templates (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    category TEXT,
    prompts JSONB NOT NULL DEFAULT '{}',
    variables JSONB,
    constraints JSONB,
    examples JSONB,
    version INTEGER NOT NULL DEFAULT 1,
    effectiveness_score REAL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Block versions table
CREATE TABLE IF NOT EXISTS block_versions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    block_id UUID NOT NULL REFERENCES blocks(id) ON DELETE CASCADE,
    version_number INTEGER NOT NULL,
    semantic_hash TEXT NOT NULL,
    syntax_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by TEXT,
    
    -- Semantic versioning
    semantic_changes JSONB,
    breaking_change BOOLEAN NOT NULL DEFAULT FALSE,
    
    -- LLM tracking
    llm_provider TEXT,
    llm_model TEXT,
    llm_prompt_id UUID REFERENCES prompt_templates(id),
    llm_temperature REAL,
    llm_reasoning TEXT,
    
    -- Change metadata
    change_type TEXT,
    change_description TEXT,
    parent_version UUID REFERENCES block_versions(id),
    branch_name TEXT
);

-- Semantic branches table
CREATE TABLE IF NOT EXISTS semantic_branches (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    repository_id UUID NOT NULL,
    base_commit_hash TEXT,
    head_commit_hash TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status TEXT NOT NULL DEFAULT 'active',
    metadata JSONB
);

-- Semantic commits table
CREATE TABLE IF NOT EXISTS semantic_commits (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    repository_id UUID NOT NULL,
    commit_hash TEXT NOT NULL,
    parent_commit_hash TEXT,
    branch_id UUID REFERENCES semantic_branches(id),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    author TEXT,
    message TEXT,
    change_type TEXT,
    impact_analysis JSONB,
    metadata JSONB
);

-- Semantic changes table
CREATE TABLE IF NOT EXISTS semantic_changes (
    change_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    commit_id UUID NOT NULL REFERENCES semantic_commits(id) ON DELETE CASCADE,
    block_id UUID NOT NULL REFERENCES blocks(id) ON DELETE CASCADE,
    change_type TEXT NOT NULL,
    before_state JSONB,
    after_state JSONB,
    impact_score REAL,
    metadata JSONB
);

-- Migrations table
CREATE TABLE IF NOT EXISTS migrations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    repository_name TEXT NOT NULL,
    source_language TEXT,
    target_language TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    status TEXT NOT NULL DEFAULT 'pending',
    metadata JSONB
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_blocks_container_id ON blocks(container_id);
CREATE INDEX IF NOT EXISTS idx_blocks_parent_id ON blocks(parent_block_id);
CREATE INDEX IF NOT EXISTS idx_blocks_semantic_name ON blocks(semantic_name);
CREATE INDEX IF NOT EXISTS idx_block_relationships_source ON block_relationships(source_block_id);
CREATE INDEX IF NOT EXISTS idx_block_relationships_target ON block_relationships(target_block_id);
CREATE INDEX IF NOT EXISTS idx_block_versions_block_id ON block_versions(block_id);
CREATE INDEX IF NOT EXISTS idx_semantic_commits_repo ON semantic_commits(repository_id);
CREATE INDEX IF NOT EXISTS idx_semantic_changes_commit ON semantic_changes(commit_id);
CREATE INDEX IF NOT EXISTS idx_semantic_changes_block ON semantic_changes(block_id);

COMMIT;