-- Add missing columns to migrations table for repository tracking
ALTER TABLE migrations 
ADD COLUMN IF NOT EXISTS repo_url TEXT,
ADD COLUMN IF NOT EXISTS commit_hash VARCHAR(40),
ADD COLUMN IF NOT EXISTS migrated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
ADD COLUMN IF NOT EXISTS status VARCHAR(50) DEFAULT 'pending';

-- Make repo_url NOT NULL after adding (allows existing rows)
UPDATE migrations SET repo_url = 'legacy_migration' WHERE repo_url IS NULL;
ALTER TABLE migrations ALTER COLUMN repo_url SET NOT NULL;

-- Add performance indexes
CREATE INDEX IF NOT EXISTS idx_migrations_repo_url ON migrations(repo_url);
CREATE INDEX IF NOT EXISTS idx_migrations_status ON migrations(status);
CREATE INDEX IF NOT EXISTS idx_migrations_migrated_at ON migrations(migrated_at DESC);

-- Add constraint for valid status values
ALTER TABLE migrations ADD CONSTRAINT check_migration_status 
CHECK (status IN ('pending', 'in_progress', 'completed', 'failed', 'rolled_back'));