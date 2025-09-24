-- Create issues table for storing detected code issues
-- Used by IssueRepository for managing architectural and code quality issues

CREATE TABLE issues (
    -- Unique identifier for the issue
    id TEXT PRIMARY KEY,

    -- Analysis run that detected this issue
    analysis_id TEXT NOT NULL,

    -- Project context
    project_id INTEGER,

    -- Issue type/category (e.g., "anti_pattern", "security", "performance")
    issue_type TEXT NOT NULL,

    -- Specific detector that found this issue
    detector TEXT NOT NULL,

    -- Issue severity (info, low, medium, high, critical)
    severity TEXT NOT NULL,

    -- File path where the issue was found
    file_path TEXT NOT NULL,

    -- Line number (if applicable)
    line_number INTEGER,

    -- Column number (if applicable)
    column_number INTEGER,

    -- Issue title/summary
    title TEXT NOT NULL,

    -- Detailed issue description
    description TEXT,

    -- Code snippet showing the problematic code
    code_snippet TEXT,

    -- Suggested fix or remediation
    suggestion TEXT,

    -- JSON-encoded metadata (AST nodes, additional context)
    metadata TEXT,

    -- Issue confidence score (0.0 to 1.0)
    confidence REAL DEFAULT 1.0,

    -- When the issue was first detected
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    -- When the issue was last seen
    last_seen DATETIME DEFAULT CURRENT_TIMESTAMP,

    -- Issue status (active, resolved, suppressed)
    status TEXT DEFAULT 'active',

    -- Hash of the issue context for duplicate detection
    context_hash TEXT
);

-- Index on analysis_id for analysis-specific queries
CREATE INDEX idx_issues_analysis ON issues(analysis_id);

-- Index on project_id for project-specific queries
CREATE INDEX idx_issues_project ON issues(project_id);

-- Index on issue_type for filtering by category
CREATE INDEX idx_issues_type ON issues(issue_type);

-- Index on severity for prioritization
CREATE INDEX idx_issues_severity ON issues(severity);

-- Index on file_path for file-specific queries
CREATE INDEX idx_issues_file ON issues(file_path);

-- Index on detector for detector-specific queries
CREATE INDEX idx_issues_detector ON issues(detector);

-- Index on status for filtering active issues
CREATE INDEX idx_issues_status ON issues(status);

-- Index on context_hash for duplicate detection
CREATE INDEX idx_issues_context_hash ON issues(context_hash);

-- Composite index for efficient issue queries
CREATE INDEX idx_issues_project_status ON issues(project_id, status);

-- Composite index for file-based queries
CREATE INDEX idx_issues_file_line ON issues(file_path, line_number);