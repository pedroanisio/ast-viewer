-- Add missing repo_name column that the application code expects
ALTER TABLE migrations 
ADD COLUMN IF NOT EXISTS repo_name VARCHAR(255);

-- Extract repo_name from existing repository_name or repo_url for existing rows
UPDATE migrations 
SET repo_name = CASE 
    WHEN repository_name IS NOT NULL THEN repository_name
    WHEN repo_url IS NOT NULL THEN 
        CASE 
            WHEN repo_url LIKE '%.git' THEN 
                split_part(split_part(repo_url, '/', -1), '.', 1)
            ELSE 
                split_part(repo_url, '/', -1)
        END
    ELSE 'unknown'
END
WHERE repo_name IS NULL;

-- Make repo_name NOT NULL after populating
ALTER TABLE migrations ALTER COLUMN repo_name SET NOT NULL;

-- Add index for performance
CREATE INDEX IF NOT EXISTS idx_migrations_repo_name ON migrations(repo_name);

-- Add comment to document the column
COMMENT ON COLUMN migrations.repo_name IS 'Repository name extracted from URL or provided directly';