-- Add missing migration_id column to containers table
ALTER TABLE containers 
ADD COLUMN IF NOT EXISTS migration_id UUID REFERENCES migrations(id) ON DELETE CASCADE;

-- Create index for performance
CREATE INDEX IF NOT EXISTS idx_containers_migration_id ON containers(migration_id);

-- Add comment to document the column
COMMENT ON COLUMN containers.migration_id IS 'References the migration that created this container';