-- Semantic Query Templates for Block Migrate
-- These queries provide intelligent analysis of semantic code blocks

-- ============================================================================
-- QUALITY ANALYSIS QUERIES
-- ============================================================================

-- Find all untested functions
CREATE OR REPLACE VIEW untested_functions AS
SELECT 
    b.id,
    b.semantic_name,
    c.name as file_name,
    c.language,
    b.position,
    (b.complexity_metrics->>'cyclomatic_complexity')::int as complexity,
    b.created_at
FROM blocks b
JOIN containers c ON b.container_id = c.id
LEFT JOIN block_relationships br 
    ON b.id = br.target_block_id AND br.relationship_type = 'tests'
WHERE b.block_type = 'Function' 
    AND br.source_block_id IS NULL
    AND b.semantic_name IS NOT NULL
ORDER BY (b.complexity_metrics->>'cyclomatic_complexity')::int DESC NULLS LAST;

-- Find functions with high cyclomatic complexity
CREATE OR REPLACE VIEW complex_functions AS
SELECT 
    b.id,
    b.semantic_name,
    c.name as file_name,
    c.language,
    (b.complexity_metrics->>'cyclomatic_complexity')::int as complexity,
    (b.complexity_metrics->>'cognitive_complexity')::int as cognitive_complexity,
    array_length(string_to_array(b.parameters::text, ','), 1) as param_count,
    b.position
FROM blocks b
JOIN containers c ON b.container_id = c.id
WHERE b.block_type = 'Function'
    AND (b.complexity_metrics->>'cyclomatic_complexity')::int > 10
ORDER BY (b.complexity_metrics->>'cyclomatic_complexity')::int DESC;

-- Find functions with too many parameters
CREATE OR REPLACE VIEW functions_many_parameters AS
SELECT 
    b.id,
    b.semantic_name,
    c.name as file_name,
    c.language,
    array_length(string_to_array(b.parameters::text, ','), 1) as param_count,
    b.parameters,
    b.position
FROM blocks b
JOIN containers c ON b.container_id = c.id
WHERE b.block_type = 'Function'
    AND array_length(string_to_array(b.parameters::text, ','), 1) > 5
ORDER BY array_length(string_to_array(b.parameters::text, ','), 1) DESC;

-- Find duplicate or similar function names
CREATE OR REPLACE VIEW duplicate_function_names AS
SELECT 
    semantic_name,
    COUNT(*) as occurrence_count,
    array_agg(DISTINCT c.language) as languages,
    array_agg(c.name) as files,
    array_agg(b.id) as block_ids
FROM blocks b
JOIN containers c ON b.container_id = c.id
WHERE b.block_type = 'Function'
    AND b.semantic_name IS NOT NULL
GROUP BY semantic_name
HAVING COUNT(*) > 1
ORDER BY COUNT(*) DESC;

-- ============================================================================
-- SECURITY ANALYSIS QUERIES
-- ============================================================================

-- Find potential SQL injection vulnerabilities
CREATE OR REPLACE VIEW sql_injection_risks AS
SELECT 
    b.id,
    b.semantic_name,
    c.name as file_name,
    c.language,
    b.abstract_syntax->>'raw_text' as code_snippet,
    b.position
FROM blocks b
JOIN containers c ON b.container_id = c.id
WHERE b.block_type = 'Function'
    AND (
        b.abstract_syntax->>'raw_text' ILIKE '%execute(%'
        OR b.abstract_syntax->>'raw_text' ILIKE '%query(%'
        OR b.abstract_syntax->>'raw_text' ILIKE '%SELECT%'
        OR b.abstract_syntax->>'raw_text' ILIKE '%INSERT%'
        OR b.abstract_syntax->>'raw_text' ILIKE '%UPDATE%'
        OR b.abstract_syntax->>'raw_text' ILIKE '%DELETE%'
    )
    AND (
        b.abstract_syntax->>'raw_text' ILIKE '%+%'
        OR b.abstract_syntax->>'raw_text' ILIKE '%format%'
        OR b.abstract_syntax->>'raw_text' ILIKE '%f"%'
        OR b.abstract_syntax->>'raw_text' ILIKE '%${%'
    )
ORDER BY c.language, b.semantic_name;

-- Find hardcoded secrets and credentials
CREATE OR REPLACE VIEW hardcoded_secrets AS
SELECT 
    b.id,
    b.semantic_name,
    c.name as file_name,
    c.language,
    b.abstract_syntax->>'raw_text' as code_snippet,
    b.position
FROM blocks b
JOIN containers c ON b.container_id = c.id
WHERE (
    b.semantic_name ILIKE '%password%'
    OR b.semantic_name ILIKE '%secret%'
    OR b.semantic_name ILIKE '%key%'
    OR b.semantic_name ILIKE '%token%'
    OR b.semantic_name ILIKE '%api_key%'
    OR b.abstract_syntax->>'raw_text' ILIKE '%password%=%'
    OR b.abstract_syntax->>'raw_text' ILIKE '%secret%=%'
    OR b.abstract_syntax->>'raw_text' ILIKE '%token%=%'
)
AND b.block_type IN ('Variable', 'Function')
ORDER BY c.language, b.semantic_name;

-- Find unsafe eval/exec usage
CREATE OR REPLACE VIEW unsafe_execution AS
SELECT 
    b.id,
    b.semantic_name,
    c.name as file_name,
    c.language,
    b.abstract_syntax->>'raw_text' as code_snippet,
    b.position
FROM blocks b
JOIN containers c ON b.container_id = c.id
WHERE b.block_type = 'Function'
    AND (
        b.abstract_syntax->>'raw_text' ILIKE '%eval(%'
        OR b.abstract_syntax->>'raw_text' ILIKE '%exec(%'
        OR b.abstract_syntax->>'raw_text' ILIKE '%subprocess%'
        OR b.abstract_syntax->>'raw_text' ILIKE '%shell=True%'
        OR b.abstract_syntax->>'raw_text' ILIKE '%os.system%'
    )
ORDER BY c.language, b.semantic_name;

-- ============================================================================
-- PERFORMANCE ANALYSIS QUERIES
-- ============================================================================

-- Find nested loops (potential O(n²) or worse complexity)
CREATE OR REPLACE VIEW nested_loops AS
SELECT 
    b.id,
    b.semantic_name,
    c.name as file_name,
    c.language,
    b.position,
    (b.complexity_metrics->>'cyclomatic_complexity')::int as complexity
FROM blocks b
JOIN containers c ON b.container_id = c.id
WHERE b.block_type = 'Function'
    AND (b.complexity_metrics->>'cyclomatic_complexity')::int > 15
    AND (
        b.abstract_syntax->>'raw_text' ILIKE '%for%for%' OR
        b.abstract_syntax->>'raw_text' ILIKE '%while%while%' OR
        b.abstract_syntax->>'raw_text' ILIKE '%for%while%'
    )
ORDER BY (b.complexity_metrics->>'cyclomatic_complexity')::int DESC;

-- Complete the language statistics view
CREATE OR REPLACE VIEW language_statistics AS
SELECT 
    c.language,
    COUNT(DISTINCT c.id) as file_count,
    COUNT(b.id) as block_count,
    AVG((b.complexity_metrics->>'cyclomatic_complexity')::int) as avg_complexity,
    MAX((b.complexity_metrics->>'cyclomatic_complexity')::int) as max_complexity,
    COUNT(DISTINCT CASE WHEN b.block_type = 'Function' THEN b.id END) as function_count,
    COUNT(DISTINCT CASE WHEN b.block_type = 'Class' THEN b.id END) as class_count,
    COUNT(DISTINCT CASE WHEN b.block_type = 'Module' THEN b.id END) as module_count,
    COUNT(DISTINCT CASE WHEN b.block_type = 'Interface' THEN b.id END) as interface_count
FROM containers c
LEFT JOIN blocks b ON c.id = b.container_id
GROUP BY c.language
ORDER BY file_count DESC;

-- Add more semantic analysis views
CREATE OR REPLACE VIEW code_smells AS
SELECT 
    b.id,
    b.semantic_name,
    c.name as file_name,
    c.language,
    CASE 
        WHEN (b.complexity_metrics->>'cyclomatic_complexity')::int > 10 THEN 'High Complexity'
        WHEN (b.complexity_metrics->>'lines_of_code')::int > 50 THEN 'Long Method'
        WHEN array_length(string_to_array(b.parameters::text, ','), 1) > 5 THEN 'Too Many Parameters'
        WHEN b.semantic_name ILIKE '%manager%' AND b.semantic_name ILIKE '%handler%' THEN 'God Object'
        WHEN b.abstract_syntax->>'raw_text' ILIKE '%if%if%if%if%' THEN 'Complex Conditionals'
    END as smell_type,
    b.position
FROM blocks b
JOIN containers c ON b.container_id = c.id
WHERE 
    (b.complexity_metrics->>'cyclomatic_complexity')::int > 10 OR
    (b.complexity_metrics->>'lines_of_code')::int > 50 OR
    array_length(string_to_array(b.parameters::text, ','), 1) > 5 OR
    (b.semantic_name ILIKE '%manager%' AND b.semantic_name ILIKE '%handler%') OR
    b.abstract_syntax->>'raw_text' ILIKE '%if%if%if%if%'
ORDER BY 
    CASE smell_type
        WHEN 'High Complexity' THEN 1
        WHEN 'God Object' THEN 2
        WHEN 'Long Method' THEN 3
        WHEN 'Too Many Parameters' THEN 4
        WHEN 'Complex Conditionals' THEN 5
        ELSE 6
    END;

-- Dependency analysis view
CREATE OR REPLACE VIEW dependency_analysis AS
SELECT 
    b.id,
    b.semantic_name,
    c.name as file_name,
    c.language,
    COUNT(DISTINCT br_out.target_block_id) as efferent_coupling,
    COUNT(DISTINCT br_in.source_block_id) as afferent_coupling,
    CASE 
        WHEN COUNT(DISTINCT br_out.target_block_id) + COUNT(DISTINCT br_in.source_block_id) = 0 THEN 0
        ELSE COUNT(DISTINCT br_out.target_block_id)::float / 
             (COUNT(DISTINCT br_out.target_block_id) + COUNT(DISTINCT br_in.source_block_id))::float
    END as instability,
    b.position
FROM blocks b
JOIN containers c ON b.container_id = c.id
LEFT JOIN block_relationships br_out ON b.id = br_out.source_block_id
LEFT JOIN block_relationships br_in ON b.id = br_in.target_block_id
WHERE b.block_type IN ('Function', 'Class', 'Module')
GROUP BY b.id, b.semantic_name, c.name, c.language, b.position
HAVING COUNT(DISTINCT br_out.target_block_id) > 5 OR COUNT(DISTINCT br_in.source_block_id) > 10
ORDER BY instability DESC;

-- Test coverage analysis
CREATE OR REPLACE VIEW test_coverage_analysis AS
SELECT 
    c.language,
    COUNT(DISTINCT CASE WHEN b.block_type = 'Function' THEN b.id END) as total_functions,
    COUNT(DISTINCT CASE WHEN b.block_type = 'Function' AND test_rel.source_block_id IS NOT NULL THEN b.id END) as tested_functions,
    CASE 
        WHEN COUNT(DISTINCT CASE WHEN b.block_type = 'Function' THEN b.id END) = 0 THEN 0
        ELSE (COUNT(DISTINCT CASE WHEN b.block_type = 'Function' AND test_rel.source_block_id IS NOT NULL THEN b.id END)::float / 
              COUNT(DISTINCT CASE WHEN b.block_type = 'Function' THEN b.id END)::float) * 100
    END as test_coverage_percentage
FROM containers c
LEFT JOIN blocks b ON c.id = b.container_id
LEFT JOIN block_relationships test_rel ON b.id = test_rel.target_block_id AND test_rel.relationship_type = 'tests'
GROUP BY c.language
ORDER BY test_coverage_percentage DESC;

-- Architecture patterns detection
CREATE OR REPLACE VIEW architecture_patterns AS
SELECT 
    b.id,
    b.semantic_name,
    c.name as file_name,
    c.language,
    CASE 
        WHEN b.semantic_name ILIKE '%factory%' THEN 'Factory Pattern'
        WHEN b.semantic_name ILIKE '%singleton%' THEN 'Singleton Pattern'
        WHEN b.semantic_name ILIKE '%observer%' OR b.semantic_name ILIKE '%listener%' THEN 'Observer Pattern'
        WHEN b.semantic_name ILIKE '%adapter%' THEN 'Adapter Pattern'
        WHEN b.semantic_name ILIKE '%decorator%' THEN 'Decorator Pattern'
        WHEN b.semantic_name ILIKE '%facade%' THEN 'Facade Pattern'
        WHEN b.semantic_name ILIKE '%strategy%' THEN 'Strategy Pattern'
        WHEN b.semantic_name ILIKE '%command%' THEN 'Command Pattern'
        WHEN b.abstract_syntax->>'raw_text' ILIKE '%getInstance%' THEN 'Singleton Pattern'
        WHEN b.abstract_syntax->>'raw_text' ILIKE '%addObserver%' THEN 'Observer Pattern'
    END as pattern_type,
    b.position
FROM blocks b
JOIN containers c ON b.container_id = c.id
WHERE 
    b.semantic_name ILIKE '%factory%' OR
    b.semantic_name ILIKE '%singleton%' OR
    b.semantic_name ILIKE '%observer%' OR
    b.semantic_name ILIKE '%listener%' OR
    b.semantic_name ILIKE '%adapter%' OR
    b.semantic_name ILIKE '%decorator%' OR
    b.semantic_name ILIKE '%facade%' OR
    b.semantic_name ILIKE '%strategy%' OR
    b.semantic_name ILIKE '%command%' OR
    b.abstract_syntax->>'raw_text' ILIKE '%getInstance%' OR
    b.abstract_syntax->>'raw_text' ILIKE '%addObserver%'
ORDER BY pattern_type, c.language;

-- Migration quality metrics
CREATE OR REPLACE VIEW migration_quality_metrics AS
SELECT 
    m.id as migration_id,
    m.repository_name,
    m.created_at,
    COUNT(DISTINCT c.id) as total_containers,
    COUNT(DISTINCT b.id) as total_blocks,
    COUNT(DISTINCT CASE WHEN b.semantic_name IS NOT NULL THEN b.id END) as named_blocks,
    CASE 
        WHEN COUNT(DISTINCT b.id) = 0 THEN 0
        ELSE (COUNT(DISTINCT CASE WHEN b.semantic_name IS NOT NULL THEN b.id END)::float / COUNT(DISTINCT b.id)::float) * 100
    END as semantic_coverage_percentage,
    AVG((b.complexity_metrics->>'cyclomatic_complexity')::int) as avg_complexity,
    COUNT(DISTINCT br.id) as total_relationships
FROM migrations m
LEFT JOIN containers c ON m.id = c.migration_id
LEFT JOIN blocks b ON c.id = b.container_id
LEFT JOIN block_relationships br ON b.id = br.source_block_id OR b.id = br.target_block_id
GROUP BY m.id, m.repository_name, m.created_at
ORDER BY m.created_at DESC;

-- ============================================================================
-- UTILITY FUNCTIONS
-- ============================================================================

-- Function to calculate technical debt score
CREATE OR REPLACE FUNCTION calculate_technical_debt_score(block_id UUID)
RETURNS FLOAT AS $$
DECLARE
    complexity_score FLOAT := 0;
    coupling_score FLOAT := 0;
    test_score FLOAT := 0;
    total_score FLOAT := 0;
BEGIN
    -- Complexity contribution (0-40 points)
    SELECT COALESCE((complexity_metrics->>'cyclomatic_complexity')::int, 0) * 4
    INTO complexity_score
    FROM blocks WHERE id = block_id;
    
    -- Coupling contribution (0-30 points)
    SELECT COUNT(*) * 3
    INTO coupling_score
    FROM block_relationships 
    WHERE source_block_id = block_id;
    
    -- Test coverage contribution (0-30 points, inverted - less tests = more debt)
    SELECT CASE 
        WHEN COUNT(*) = 0 THEN 30
        ELSE GREATEST(0, 30 - COUNT(*) * 10)
    END
    INTO test_score
    FROM block_relationships 
    WHERE target_block_id = block_id AND relationship_type = 'tests';
    
    total_score := LEAST(100, complexity_score + coupling_score + test_score);
    
    RETURN total_score;
END;
$$ LANGUAGE plpgsql;

-- Function to find circular dependencies
CREATE OR REPLACE FUNCTION find_circular_dependencies()
RETURNS TABLE(cycle_blocks UUID[], cycle_length INT) AS $$
BEGIN
    RETURN QUERY
    WITH RECURSIVE dependency_paths AS (
        -- Base case: start from each block
        SELECT 
            source_block_id as start_block,
            target_block_id as current_block,
            ARRAY[source_block_id, target_block_id] as path,
            1 as depth
        FROM block_relationships
        WHERE relationship_type IN ('calls', 'depends_on', 'imports')
        
        UNION
        
        -- Recursive case: extend paths
        SELECT 
            dp.start_block,
            br.target_block_id,
            dp.path || br.target_block_id,
            dp.depth + 1
        FROM dependency_paths dp
        JOIN block_relationships br ON dp.current_block = br.source_block_id
        WHERE dp.depth < 10  -- Prevent infinite recursion
            AND NOT (br.target_block_id = ANY(dp.path))  -- Prevent immediate cycles
            AND br.relationship_type IN ('calls', 'depends_on', 'imports')
    )
    SELECT DISTINCT 
        dp.path,
        array_length(dp.path, 1)
    FROM dependency_paths dp
    WHERE dp.current_block = dp.start_block  -- Found a cycle
        AND array_length(dp.path, 1) > 2     -- Meaningful cycle
    ORDER BY array_length(dp.path, 1);
END;
$$ LANGUAGE plpgsql;

-- Find database queries in loops
CREATE OR REPLACE VIEW db_queries_in_loops AS
SELECT 
    b.id,
    b.semantic_name,
    c.name as file_name,
    c.language,
    b.position
FROM blocks b
JOIN containers c ON b.container_id = c.id
WHERE b.block_type = 'Function'
    AND (
        (b.abstract_syntax->>'raw_text' ILIKE '%for%' OR b.abstract_syntax->>'raw_text' ILIKE '%while%')
        AND (
            b.abstract_syntax->>'raw_text' ILIKE '%SELECT%'
            OR b.abstract_syntax->>'raw_text' ILIKE '%INSERT%'
            OR b.abstract_syntax->>'raw_text' ILIKE '%UPDATE%'
            OR b.abstract_syntax->>'raw_text' ILIKE '%DELETE%'
            OR b.abstract_syntax->>'raw_text' ILIKE '%query%'
            OR b.abstract_syntax->>'raw_text' ILIKE '%execute%'
        )
    )
ORDER BY c.language, b.semantic_name;

-- Find synchronous operations that could be async
CREATE OR REPLACE VIEW sync_operations AS
SELECT 
    b.id,
    b.semantic_name,
    c.name as file_name,
    c.language,
    b.position
FROM blocks b
JOIN containers c ON b.container_id = c.id
WHERE b.block_type = 'Function'
    AND NOT (b.modifiers @> ARRAY['async'])
    AND (
        b.abstract_syntax->>'raw_text' ILIKE '%requests.get%'
        OR b.abstract_syntax->>'raw_text' ILIKE '%requests.post%'
        OR b.abstract_syntax->>'raw_text' ILIKE '%urllib.request%'
        OR b.abstract_syntax->>'raw_text' ILIKE '%file.read%'
        OR b.abstract_syntax->>'raw_text' ILIKE '%open(%'
        OR b.abstract_syntax->>'raw_text' ILIKE '%database%'
        OR b.abstract_syntax->>'raw_text' ILIKE '%query%'
    )
ORDER BY c.language, b.semantic_name;

-- ============================================================================
-- ARCHITECTURAL ANALYSIS QUERIES
-- ============================================================================

-- Find circular dependencies
CREATE OR REPLACE VIEW circular_dependencies AS
WITH RECURSIVE dependency_path AS (
    -- Base case: direct dependencies
    SELECT 
        source_block_id, 
        target_block_id, 
        ARRAY[source_block_id, target_block_id] as path,
        1 as depth
    FROM block_relationships
    WHERE relationship_type = 'depends_on'
    
    UNION ALL
    
    -- Recursive case: extend the path
    SELECT 
        dp.source_block_id,
        br.target_block_id,
        dp.path || br.target_block_id,
        dp.depth + 1
    FROM dependency_path dp
    JOIN block_relationships br ON dp.target_block_id = br.source_block_id
    WHERE br.relationship_type = 'depends_on'
        AND dp.depth < 10  -- Prevent infinite recursion
        AND NOT br.target_block_id = ANY(dp.path)  -- Prevent cycles in path building
)
SELECT DISTINCT
    dp.source_block_id,
    dp.target_block_id,
    dp.path,
    dp.depth,
    bs.semantic_name as source_name,
    bt.semantic_name as target_name,
    cs.name as source_file,
    ct.name as target_file
FROM dependency_path dp
JOIN blocks bs ON dp.source_block_id = bs.id
JOIN blocks bt ON dp.target_block_id = bt.id
JOIN containers cs ON bs.container_id = cs.id
JOIN containers ct ON bt.container_id = ct.id
WHERE dp.source_block_id = dp.target_block_id  -- Circular dependency detected
ORDER BY dp.depth, bs.semantic_name;

-- Find orphaned blocks (no relationships)
CREATE OR REPLACE VIEW orphaned_blocks AS
SELECT 
    b.id,
    b.semantic_name,
    b.block_type,
    c.name as file_name,
    c.language,
    b.position
FROM blocks b
JOIN containers c ON b.container_id = c.id
LEFT JOIN block_relationships br1 ON b.id = br1.source_block_id
LEFT JOIN block_relationships br2 ON b.id = br2.target_block_id
WHERE br1.source_block_id IS NULL 
    AND br2.target_block_id IS NULL
    AND b.block_type IN ('Function', 'Class')
ORDER BY c.language, b.semantic_name;

-- Find API endpoints and their patterns
CREATE OR REPLACE VIEW api_endpoints AS
SELECT 
    b.id,
    b.semantic_name,
    c.name as file_name,
    c.language,
    b.position,
    CASE 
        WHEN b.abstract_syntax->>'raw_text' ILIKE '%@app.route%' THEN 'Flask'
        WHEN b.abstract_syntax->>'raw_text' ILIKE '%@router%' THEN 'FastAPI'
        WHEN b.abstract_syntax->>'raw_text' ILIKE '%express%' THEN 'Express.js'
        WHEN b.abstract_syntax->>'raw_text' ILIKE '%Controller%' THEN 'MVC Controller'
        ELSE 'Unknown'
    END as framework,
    CASE 
        WHEN b.abstract_syntax->>'raw_text' ILIKE '%GET%' THEN 'GET'
        WHEN b.abstract_syntax->>'raw_text' ILIKE '%POST%' THEN 'POST'
        WHEN b.abstract_syntax->>'raw_text' ILIKE '%PUT%' THEN 'PUT'
        WHEN b.abstract_syntax->>'raw_text' ILIKE '%DELETE%' THEN 'DELETE'
        ELSE 'Unknown'
    END as http_method
FROM blocks b
JOIN containers c ON b.container_id = c.id
WHERE b.block_type = 'Function'
    AND (
        b.semantic_name ILIKE '%Controller%'
        OR b.semantic_name ILIKE '%Route%'
        OR b.semantic_name ILIKE '%Handler%'
        OR b.semantic_name ILIKE '%endpoint%'
        OR b.abstract_syntax->>'raw_text' ILIKE '%@app.route%'
        OR b.abstract_syntax->>'raw_text' ILIKE '%@router%'
        OR b.abstract_syntax->>'raw_text' ILIKE '%express%'
    )
ORDER BY framework, http_method, b.semantic_name;

-- ============================================================================
-- TESTING ANALYSIS QUERIES
-- ============================================================================

-- Find test coverage by file
CREATE OR REPLACE VIEW test_coverage_by_file AS
SELECT 
    c.name as file_name,
    c.language,
    COUNT(b.id) as total_functions,
    COUNT(br.target_block_id) as tested_functions,
    ROUND(
        (COUNT(br.target_block_id)::float / NULLIF(COUNT(b.id), 0)) * 100, 
        2
    ) as coverage_percentage
FROM containers c
JOIN blocks b ON c.id = b.container_id
LEFT JOIN block_relationships br ON b.id = br.target_block_id AND br.relationship_type = 'tests'
WHERE b.block_type = 'Function'
GROUP BY c.id, c.name, c.language
ORDER BY coverage_percentage ASC, total_functions DESC;

-- Find test files and their patterns
CREATE OR REPLACE VIEW test_files AS
SELECT 
    c.id,
    c.name as file_name,
    c.language,
    COUNT(b.id) as test_count,
    array_agg(b.semantic_name) as test_names
FROM containers c
JOIN blocks b ON c.id = b.container_id
WHERE (
    c.name ILIKE '%test%'
    OR c.name ILIKE '%spec%'
    OR b.semantic_name ILIKE 'test_%'
    OR b.semantic_name ILIKE '%_test'
    OR b.semantic_name ILIKE 'it_%'
    OR b.semantic_name ILIKE 'describe_%'
)
AND b.block_type = 'Function'
GROUP BY c.id, c.name, c.language
ORDER BY test_count DESC;

-- ============================================================================
-- CODE METRICS AND STATISTICS
-- ============================================================================

-- Overall codebase statistics
CREATE OR REPLACE VIEW codebase_statistics AS
SELECT 
    COUNT(DISTINCT c.id) as total_files,
    COUNT(DISTINCT c.language) as languages_count,
    COUNT(b.id) as total_blocks,
    COUNT(CASE WHEN b.block_type = 'Function' THEN 1 END) as total_functions,
    COUNT(CASE WHEN b.block_type = 'Class' THEN 1 END) as total_classes,
    COUNT(CASE WHEN b.block_type = 'Interface' THEN 1 END) as total_interfaces,
    ROUND(AVG((b.complexity_metrics->>'cyclomatic_complexity')::int), 2) as avg_complexity,
    MAX((b.complexity_metrics->>'cyclomatic_complexity')::int) as max_complexity,
    COUNT(br.id) as total_relationships
FROM containers c
JOIN blocks b ON c.id = b.container_id
LEFT JOIN block_relationships br ON b.id = br.source_block_id;

-- Language-specific statistics
CREATE OR REPLACE VIEW language_statistics AS
SELECT 
    c.language,
    COUNT(DISTINCT c.id) as files_count,
    COUNT(b.id) as blocks_count,
    COUNT(CASE WHEN b.block_type = 'Function' THEN 1 END) as functions_count,
    COUNT(CASE WHEN b.block_type = 'Class' THEN 1 END) as classes_count,
    ROUND(AVG((b.complexity_metrics->>'cyclomatic_complexity')::int), 2) as avg_complexity,
    COUNT(CASE WHEN br.target_block_id IS NULL AND b.block_type = 'Function' THEN 1 END) as untested_functions
FROM containers c
JOIN blocks b ON c.id = b.container_id
LEFT JOIN block_relationships br ON b.id = br.target_block_id AND br.relationship_type = 'tests'
WHERE c.language IS NOT NULL
GROUP BY c.language
ORDER BY blocks_count DESC;

-- ============================================================================
-- HELPER FUNCTIONS
-- ============================================================================

-- Function to search blocks by semantic content
CREATE OR REPLACE FUNCTION search_blocks_semantic(
    search_term TEXT,
    language_filter TEXT DEFAULT NULL,
    block_type_filter TEXT DEFAULT NULL,
    limit_count INTEGER DEFAULT 100
)
RETURNS TABLE (
    id UUID,
    semantic_name TEXT,
    block_type TEXT,
    file_name TEXT,
    language TEXT,
    position INTEGER,
    relevance_score REAL
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        b.id,
        b.semantic_name,
        b.block_type,
        c.name as file_name,
        c.language,
        b.position,
        ts_rank(
            to_tsvector('english', 
                COALESCE(b.semantic_name, '') || ' ' || 
                COALESCE(b.abstract_syntax->>'raw_text', '')
            ),
            plainto_tsquery('english', search_term)
        ) as relevance_score
    FROM blocks b
    JOIN containers c ON b.container_id = c.id
    WHERE to_tsvector('english', 
        COALESCE(b.semantic_name, '') || ' ' || 
        COALESCE(b.abstract_syntax->>'raw_text', '')
    ) @@ plainto_tsquery('english', search_term)
    AND (language_filter IS NULL OR c.language = language_filter)
    AND (block_type_filter IS NULL OR b.block_type = block_type_filter)
    ORDER BY relevance_score DESC
    LIMIT limit_count;
END;
$$ LANGUAGE plpgsql;

-- Function to analyze code quality for a specific file
CREATE OR REPLACE FUNCTION analyze_file_quality(file_id UUID)
RETURNS TABLE (
    metric_name TEXT,
    metric_value NUMERIC,
    status TEXT,
    recommendation TEXT
) AS $$
BEGIN
    RETURN QUERY
    WITH file_metrics AS (
        SELECT 
            COUNT(CASE WHEN b.block_type = 'Function' THEN 1 END) as function_count,
            AVG((b.complexity_metrics->>'cyclomatic_complexity')::int) as avg_complexity,
            COUNT(CASE WHEN br.target_block_id IS NULL AND b.block_type = 'Function' THEN 1 END) as untested_functions,
            COUNT(CASE WHEN array_length(string_to_array(b.parameters::text, ','), 1) > 5 THEN 1 END) as functions_many_params
        FROM blocks b
        LEFT JOIN block_relationships br ON b.id = br.target_block_id AND br.relationship_type = 'tests'
        WHERE b.container_id = file_id
    )
    SELECT 
        'Average Complexity'::TEXT,
        COALESCE(fm.avg_complexity, 0),
        CASE 
            WHEN fm.avg_complexity <= 5 THEN 'Good'
            WHEN fm.avg_complexity <= 10 THEN 'Warning'
            ELSE 'Critical'
        END::TEXT,
        CASE 
            WHEN fm.avg_complexity <= 5 THEN 'Complexity is within acceptable range'
            WHEN fm.avg_complexity <= 10 THEN 'Consider refactoring complex functions'
            ELSE 'High complexity detected - refactoring recommended'
        END::TEXT
    FROM file_metrics fm
    
    UNION ALL
    
    SELECT 
        'Test Coverage'::TEXT,
        CASE 
            WHEN fm.function_count > 0 
            THEN ROUND(((fm.function_count - fm.untested_functions)::float / fm.function_count) * 100, 2)
            ELSE 0 
        END,
        CASE 
            WHEN fm.function_count = 0 THEN 'N/A'
            WHEN fm.untested_functions = 0 THEN 'Excellent'
            WHEN (fm.untested_functions::float / fm.function_count) <= 0.2 THEN 'Good'
            WHEN (fm.untested_functions::float / fm.function_count) <= 0.5 THEN 'Warning'
            ELSE 'Critical'
        END::TEXT,
        CASE 
            WHEN fm.function_count = 0 THEN 'No functions to test'
            WHEN fm.untested_functions = 0 THEN 'All functions have tests'
            ELSE 'Add tests for ' || fm.untested_functions || ' functions'
        END::TEXT
    FROM file_metrics fm;
END;
$$ LANGUAGE plpgsql;
