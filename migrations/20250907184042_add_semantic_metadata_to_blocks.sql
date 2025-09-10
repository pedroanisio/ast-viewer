-- Add missing semantic_metadata column to blocks table
ALTER TABLE blocks 
ADD COLUMN IF NOT EXISTS semantic_metadata JSONB;

-- Add index for performance on semantic_metadata queries
CREATE INDEX IF NOT EXISTS idx_blocks_semantic_metadata ON blocks USING GIN (semantic_metadata);

-- Add comment to document the column
COMMENT ON COLUMN blocks.semantic_metadata IS 'JSON object containing semantic metadata (parameters, return types, modifiers, etc.)';