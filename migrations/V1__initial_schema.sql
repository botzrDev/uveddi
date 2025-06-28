-- V1__initial_schema.sql
-- Initial SQLite schema for CodeAtlas local database

-- AntiPatternType table (catalog of predefined patterns)
CREATE TABLE anti_pattern_types (
    anti_pattern_type_id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    category TEXT,
    detection_heuristic_notes TEXT
);

-- Project table (local project tracking)
CREATE TABLE projects (
    project_id INTEGER PRIMARY KEY AUTOINCREMENT,
    organization_id INTEGER, -- Nullable for local-only projects
    name TEXT NOT NULL,
    repository_url TEXT NOT NULL,
    last_analyzed_commit TEXT,
    config_data TEXT, -- JSON string for .archlintignore rules, etc.
    created_at INTEGER NOT NULL -- Unix timestamp
);

-- AnalysisRun table (execution records)
CREATE TABLE analysis_runs (
    run_id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL,
    user_id INTEGER, -- Nullable if run by CI/CD
    start_time INTEGER NOT NULL, -- Unix timestamp
    end_time INTEGER, -- Unix timestamp
    status TEXT NOT NULL CHECK (status IN ('running', 'completed', 'failed', 'cancelled')),
    ai_model_used TEXT,
    total_files_scanned INTEGER,
    total_issues_found INTEGER,
    exit_code INTEGER,
    raw_analysis_output TEXT, -- JSON string for debugging
    FOREIGN KEY (project_id) REFERENCES projects(project_id)
);

-- ArchitecturalIssue table (detected issues)
CREATE TABLE architectural_issues (
    issue_id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id INTEGER NOT NULL,
    anti_pattern_type_id INTEGER NOT NULL,
    file_path TEXT NOT NULL,
    line_start INTEGER,
    line_end INTEGER,
    severity TEXT NOT NULL CHECK (severity IN ('critical', 'high', 'medium', 'low')),
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    ai_refactoring_suggestion TEXT,
    is_ignored INTEGER NOT NULL DEFAULT 0, -- SQLite boolean as INTEGER
    ignored_by_user_id INTEGER,
    ignored_at INTEGER, -- Unix timestamp
    FOREIGN KEY (run_id) REFERENCES analysis_runs(run_id),
    FOREIGN KEY (anti_pattern_type_id) REFERENCES anti_pattern_types(anti_pattern_type_id)
);

-- CodeSnippet table (code segments related to issues)
CREATE TABLE code_snippets (
    snippet_id INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id INTEGER NOT NULL,
    content TEXT NOT NULL,
    language TEXT,
    context_lines_before INTEGER,
    context_lines_after INTEGER,
    FOREIGN KEY (issue_id) REFERENCES architectural_issues(issue_id)
);

-- Report table (generated markdown reports)
CREATE TABLE reports (
    report_id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id INTEGER NOT NULL UNIQUE,
    file_path TEXT,
    content TEXT,
    generated_at INTEGER NOT NULL, -- Unix timestamp
    FOREIGN KEY (run_id) REFERENCES analysis_runs(run_id)
);

-- Diagram table (visual diagrams in reports)
CREATE TABLE diagrams (
    diagram_id INTEGER PRIMARY KEY AUTOINCREMENT,
    report_id INTEGER NOT NULL,
    type TEXT NOT NULL,
    syntax_content TEXT NOT NULL,
    description TEXT,
    associated_issue_id INTEGER,
    FOREIGN KEY (report_id) REFERENCES reports(report_id),
    FOREIGN KEY (associated_issue_id) REFERENCES architectural_issues(issue_id)
);

-- Plugin table (custom scanners/extensions)
CREATE TABLE plugins (
    plugin_id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    version TEXT NOT NULL,
    description TEXT,
    author_user_id INTEGER, -- Nullable for official plugins
    github_repo_url TEXT,
    is_official INTEGER NOT NULL DEFAULT 0, -- SQLite boolean as INTEGER
    is_approved INTEGER NOT NULL DEFAULT 0, -- SQLite boolean as INTEGER
    created_at INTEGER NOT NULL, -- Unix timestamp
    last_updated INTEGER NOT NULL -- Unix timestamp
);

-- Create indexes for better query performance
CREATE INDEX idx_analysis_runs_project_id ON analysis_runs(project_id);
CREATE INDEX idx_analysis_runs_start_time ON analysis_runs(start_time);
CREATE INDEX idx_architectural_issues_run_id ON architectural_issues(run_id);
CREATE INDEX idx_architectural_issues_file_path ON architectural_issues(file_path);
CREATE INDEX idx_architectural_issues_severity ON architectural_issues(severity);
CREATE INDEX idx_code_snippets_issue_id ON code_snippets(issue_id);
CREATE INDEX idx_reports_run_id ON reports(run_id);
CREATE INDEX idx_diagrams_report_id ON diagrams(report_id);

-- Insert some default anti-pattern types
INSERT INTO anti_pattern_types (name, description, category, detection_heuristic_notes) VALUES
('Cyclic Dependency', 'Circular dependencies between modules or components', 'Dependency-Based', 'Detect using dependency graph analysis'),
('God Object', 'Classes or modules with too many responsibilities', 'Abstraction-Based', 'Analyze class size, method count, and coupling metrics'),
('Leaky Abstraction', 'Abstractions that expose implementation details', 'Abstraction-Based', 'Look for inappropriate exposure of internal state'),
('Tight Coupling', 'Excessive dependencies between components', 'Dependency-Based', 'Measure coupling metrics and dependency counts'),
('Long Parameter List', 'Functions with too many parameters', 'Code Smell', 'Count function parameters beyond reasonable threshold'),
('Duplicate Code', 'Repeated code blocks across the codebase', 'Code Smell', 'Use AST similarity analysis and string matching');
