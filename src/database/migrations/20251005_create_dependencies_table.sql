-- Create dependencies table for storing code dependency relationships
-- Used by DependencyRepository for managing code dependency graphs

CREATE TABLE dependencies (
    -- Unique identifier for the dependency relationship
    id TEXT PRIMARY KEY,

    -- Analysis run that detected this dependency
    analysis_id TEXT NOT NULL,

    -- Project context
    project_id INTEGER,

    -- Source entity (module, function, class, etc.)
    source_entity TEXT NOT NULL,

    -- Target entity that source depends on
    target_entity TEXT NOT NULL,

    -- Type of dependency (import, call, inheritance, etc.)
    dependency_type TEXT NOT NULL,

    -- Source file path
    source_file TEXT NOT NULL,

    -- Target file path
    target_file TEXT NOT NULL,

    -- Dependency strength/weight (how critical)
    weight INTEGER DEFAULT 1,

    -- Line number in source file
    source_line INTEGER,

    -- Column number in source file
    source_column INTEGER,

    -- Is this dependency external (outside project)
    is_external BOOLEAN DEFAULT FALSE,

    -- Is this a circular dependency
    is_circular BOOLEAN DEFAULT FALSE,

    -- JSON-encoded metadata (AST context, additional info)
    metadata TEXT,

    -- When the dependency was first detected
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    -- When the dependency was last confirmed
    last_seen DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Index on analysis_id for analysis-specific queries
CREATE INDEX idx_deps_analysis ON dependencies(analysis_id);

-- Index on project_id for project-specific queries
CREATE INDEX idx_deps_project ON dependencies(project_id);

-- Index on source_entity for dependency graph traversal
CREATE INDEX idx_deps_source ON dependencies(source_entity);

-- Index on target_entity for reverse dependency lookup
CREATE INDEX idx_deps_target ON dependencies(target_entity);

-- Index on dependency_type for filtering by type
CREATE INDEX idx_deps_type ON dependencies(dependency_type);

-- Index on source_file for file-specific dependencies
CREATE INDEX idx_deps_source_file ON dependencies(source_file);

-- Index on target_file for file-specific dependencies
CREATE INDEX idx_deps_target_file ON dependencies(target_file);

-- Index on is_external for filtering external dependencies
CREATE INDEX idx_deps_external ON dependencies(is_external);

-- Index on is_circular for finding circular dependencies
CREATE INDEX idx_deps_circular ON dependencies(is_circular);

-- Composite index for efficient dependency queries
CREATE INDEX idx_deps_source_target ON dependencies(source_entity, target_entity);

-- Composite index for project dependency analysis
CREATE INDEX idx_deps_project_type ON dependencies(project_id, dependency_type);