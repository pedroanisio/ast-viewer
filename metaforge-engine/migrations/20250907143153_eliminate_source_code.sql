-- Migration: Eliminate source_code field and enhance semantic storage
-- Part of Phase 1A.3: Complete transition to source-code-free operation
-- 
-- This migration removes the dependency on raw source code text and enhances
-- semantic storage capabilities for pure semantic-first development.

BEGIN;

-- Step 1: Add enhanced semantic fields to containers table
ALTER TABLE containers 
ADD COLUMN IF NOT EXISTS semantic_summary JSONB DEFAULT '{}',
ADD COLUMN IF NOT EXISTS parsing_metadata JSONB DEFAULT '{}',
ADD COLUMN IF NOT EXISTS formatting_preferences JSONB DEFAULT '{}',
ADD COLUMN IF NOT EXISTS reconstruction_hints JSONB DEFAULT '{}';

-- Step 2: Add enhanced semantic fields to blocks table  
ALTER TABLE blocks
ADD COLUMN IF NOT EXISTS semantic_signature JSONB DEFAULT '{}',
ADD COLUMN IF NOT EXISTS behavioral_contract JSONB DEFAULT '{}',
ADD COLUMN IF NOT EXISTS formatting_metadata JSONB DEFAULT '{}',
ADD COLUMN IF NOT EXISTS attached_comments JSONB DEFAULT '{}',
ADD COLUMN IF NOT EXISTS dependency_info JSONB DEFAULT '{}';

-- Step 3: Create migration status tracking
CREATE TABLE IF NOT EXISTS source_code_migration_log (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    container_id UUID NOT NULL REFERENCES containers(id) ON DELETE CASCADE,
    migration_status TEXT NOT NULL DEFAULT 'pending', -- pending, in_progress, completed, failed
    semantic_extraction_quality REAL DEFAULT 0.0,
    original_size_bytes INTEGER,
    semantic_blocks_count INTEGER,
    migration_started_at TIMESTAMPTZ DEFAULT NOW(),
    migration_completed_at TIMESTAMPTZ,
    error_messages TEXT[],
    metadata JSONB DEFAULT '{}'
);

-- Step 4: Create backup table for source code (temporary safety measure)
CREATE TABLE IF NOT EXISTS source_code_backup (
    container_id UUID PRIMARY KEY REFERENCES containers(id) ON DELETE CASCADE,
    original_source_code TEXT,
    original_path TEXT,
    original_hash TEXT,
    backup_created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    restored BOOLEAN DEFAULT FALSE
);

-- Step 5: Create enhanced semantic reconstruction view
CREATE OR REPLACE VIEW semantic_container_view AS
SELECT 
    c.id,
    c.name,
    c.container_type,
    c.language,
    c.version,
    c.semantic_summary,
    c.parsing_metadata,
    c.formatting_preferences,
    c.reconstruction_hints,
    COUNT(b.id) as block_count,
    COALESCE(
        ROUND(AVG(
            CASE 
                WHEN b.language_features IS NOT NULL THEN 1.0
                WHEN b.body_ast IS NOT NULL THEN 0.8
                WHEN b.abstract_syntax != '{}' THEN 0.6
                ELSE 0.0
            END
        ), 2), 0.0
    ) as semantic_completeness_score,
    ARRAY_AGG(DISTINCT b.block_type) FILTER (WHERE b.block_type IS NOT NULL) as block_types,
    c.created_at,
    c.updated_at
FROM containers c
LEFT JOIN blocks b ON c.id = b.container_id
GROUP BY c.id, c.name, c.container_type, c.language, c.version, 
         c.semantic_summary, c.parsing_metadata, c.formatting_preferences, 
         c.reconstruction_hints, c.created_at, c.updated_at;

-- Step 6: Create indexes for new semantic fields
CREATE INDEX IF NOT EXISTS idx_containers_semantic_summary ON containers USING GIN (semantic_summary);
CREATE INDEX IF NOT EXISTS idx_blocks_semantic_signature ON blocks USING GIN (semantic_signature);
CREATE INDEX IF NOT EXISTS idx_blocks_behavioral_contract ON blocks USING GIN (behavioral_contract);
CREATE INDEX IF NOT EXISTS idx_migration_log_status ON source_code_migration_log(migration_status);
CREATE INDEX IF NOT EXISTS idx_migration_log_container ON source_code_migration_log(container_id);

-- Step 7: Add migration metadata
UPDATE containers SET 
    semantic_summary = jsonb_build_object(
        'migration_version', '002_eliminate_source_code',
        'enhanced_semantic_storage', true,
        'source_code_free_ready', false,
        'migration_applied_at', NOW()
    )
WHERE semantic_summary = '{}' OR semantic_summary IS NULL;

COMMIT;