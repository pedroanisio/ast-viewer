-- Add missing statistics column to migrations table
ALTER TABLE migrations 
ADD COLUMN IF NOT EXISTS statistics JSONB;

-- Add comment to document the column
COMMENT ON COLUMN migrations.statistics IS 'JSON object containing migration statistics (file counts, processing times, etc.)';