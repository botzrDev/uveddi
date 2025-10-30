-- Create technical_debt table for storing technical debt analysis
-- Used by TechnicalDebtRepository for managing code quality and maintainability issues

CREATE TABLE technical_debt (
    -- Unique identifier for the technical debt item
    id TEXT PRIMARY KEY,

    -- Analysis run that detected this debt
    analysis_id TEXT NOT NULL,

    -- Project context
    project_id INTEGER,

    -- Debt category (complexity, duplication, maintainability, etc.)
    category TEXT NOT NULL,

    -- Specific type of debt (code_smell, complex_method, duplicated_code, etc.)
    debt_type TEXT NOT NULL,

    -- Debt severity (info, low, medium, high, critical)
    severity TEXT NOT NULL,

    -- File path where the debt was detected
    file_path TEXT NOT NULL,

    -- Line number range start
    line_start INTEGER,

    -- Line number range end
    line_end INTEGER,

    -- Function or method name (if applicable)
    function_name TEXT,

    -- Class name (if applicable)
    class_name TEXT,

    -- Debt title/summary
    title TEXT NOT NULL,

    -- Detailed description of the technical debt
    description TEXT,

    -- Code snippet showing the problematic code
    code_snippet TEXT,

    -- Suggested refactoring approach
    refactoring_suggestion TEXT,

    -- Estimated effort to fix (in hours)
    effort_estimate REAL,

    -- Priority for addressing (1-10, higher is more important)
    priority INTEGER DEFAULT 5,

    -- JSON-encoded metrics (complexity scores, etc.)
    metrics TEXT,

    -- JSON-encoded metadata (AST nodes, additional context)
    metadata TEXT,

    -- Debt confidence score (0.0 to 1.0)
    confidence REAL DEFAULT 1.0,

    -- Impact on maintainability (0.0 to 10.0)
    maintainability_impact REAL DEFAULT 5.0,

    -- When the debt was first detected
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    -- When the debt was last seen
    last_seen DATETIME DEFAULT CURRENT_TIMESTAMP,

    -- Debt status (active, resolved, accepted, planned)
    status TEXT DEFAULT 'active',

    -- Hash for duplicate detection
    context_hash TEXT
);

-- Index on analysis_id for analysis-specific queries
CREATE INDEX idx_debt_analysis ON technical_debt(analysis_id);

-- Index on project_id for project-specific queries
CREATE INDEX idx_debt_project ON technical_debt(project_id);

-- Index on category for debt categorization
CREATE INDEX idx_debt_category ON technical_debt(category);

-- Index on debt_type for specific debt type queries
CREATE INDEX idx_debt_type ON technical_debt(debt_type);

-- Index on severity for prioritization
CREATE INDEX idx_debt_severity ON technical_debt(severity);

-- Index on file_path for file-specific queries
CREATE INDEX idx_debt_file ON technical_debt(file_path);

-- Index on priority for prioritization
CREATE INDEX idx_debt_priority ON technical_debt(priority);

-- Index on status for filtering active debt
CREATE INDEX idx_debt_status ON technical_debt(status);

-- Index on context_hash for duplicate detection
CREATE INDEX idx_debt_context_hash ON technical_debt(context_hash);

-- Composite index for efficient debt queries
CREATE INDEX idx_debt_project_status ON technical_debt(project_id, status);

-- Composite index for priority analysis
CREATE INDEX idx_debt_severity_priority ON technical_debt(severity, priority);