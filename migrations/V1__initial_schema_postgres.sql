-- V1__initial_schema_postgres.sql
-- Initial PostgreSQL schema for CodeAtlas backend
-- Generated on 2025-06-30

-- Organization table
CREATE TABLE organizations (
    organization_id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    subscription_tier TEXT NOT NULL DEFAULT 'free',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    api_key_usage_limit INTEGER
);

-- User table
CREATE TABLE users (
    user_id SERIAL PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login TIMESTAMPTZ,
    role TEXT NOT NULL DEFAULT 'individual',
    organization_id INTEGER REFERENCES organizations(organization_id)
);

-- Project table
CREATE TABLE projects (
    project_id SERIAL PRIMARY KEY,
    organization_id INTEGER NOT NULL REFERENCES organizations(organization_id),
    name TEXT NOT NULL,
    repository_url TEXT NOT NULL,
    last_analyzed_commit TEXT,
    config_data JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- AntiPatternType table
CREATE TABLE anti_pattern_types (
    anti_pattern_type_id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    category TEXT,
    detection_heuristic_notes TEXT
);

-- AnalysisRun table
CREATE TABLE analysis_runs (
    run_id SERIAL PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES projects(project_id),
    user_id INTEGER REFERENCES users(user_id),
    start_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    end_time TIMESTAMPTZ,
    status TEXT NOT NULL CHECK (status IN ('running', 'completed', 'failed', 'cancelled')),
    ai_model_used TEXT,
    total_files_scanned INTEGER,
    total_issues_found INTEGER,
    exit_code INTEGER,
    raw_analysis_output JSONB
);

-- ArchitecturalIssue table
CREATE TABLE architectural_issues (
    issue_id SERIAL PRIMARY KEY,
    run_id INTEGER NOT NULL REFERENCES analysis_runs(run_id),
    anti_pattern_type_id INTEGER NOT NULL REFERENCES anti_pattern_types(anti_pattern_type_id),
    file_path TEXT NOT NULL,
    line_start INTEGER,
    line_end INTEGER,
    severity TEXT NOT NULL CHECK (severity IN ('critical', 'high', 'medium', 'low')),
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    ai_refactoring_suggestion TEXT,
    is_ignored BOOLEAN NOT NULL DEFAULT FALSE,
    ignored_by_user_id INTEGER REFERENCES users(user_id),
    ignored_at TIMESTAMPTZ
);

-- CodeSnippet table
CREATE TABLE code_snippets (
    snippet_id SERIAL PRIMARY KEY,
    issue_id INTEGER NOT NULL REFERENCES architectural_issues(issue_id),
    content TEXT NOT NULL,
    language TEXT,
    context_lines_before INTEGER,
    context_lines_after INTEGER
);

-- Report table
CREATE TABLE reports (
    report_id SERIAL PRIMARY KEY,
    run_id INTEGER NOT NULL UNIQUE REFERENCES analysis_runs(run_id),
    file_path TEXT,
    content TEXT,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Diagram table
CREATE TABLE diagrams (
    diagram_id SERIAL PRIMARY KEY,
    report_id INTEGER NOT NULL REFERENCES reports(report_id),
    type TEXT NOT NULL,
    syntax_content TEXT NOT NULL,
    description TEXT,
    associated_issue_id INTEGER REFERENCES architectural_issues(issue_id)
);

-- Plugin table
CREATE TABLE plugins (
    plugin_id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    version TEXT NOT NULL,
    description TEXT,
    author_user_id INTEGER REFERENCES users(user_id),
    github_repo_url TEXT,
    is_official BOOLEAN NOT NULL DEFAULT FALSE,
    is_approved BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_users_organization_id ON users(organization_id);
CREATE INDEX idx_projects_organization_id ON projects(organization_id);
CREATE INDEX idx_analysis_runs_project_id ON analysis_runs(project_id);
CREATE INDEX idx_analysis_runs_user_id ON analysis_runs(user_id);
CREATE INDEX idx_architectural_issues_run_id ON architectural_issues(run_id);
CREATE INDEX idx_architectural_issues_anti_pattern_type_id ON architectural_issues(anti_pattern_type_id);
CREATE INDEX idx_code_snippets_issue_id ON code_snippets(issue_id);
CREATE INDEX idx_reports_run_id ON reports(run_id);
CREATE INDEX idx_diagrams_report_id ON diagrams(report_id);

-- Default anti-pattern types
INSERT INTO anti_pattern_types (name, description, category, detection_heuristic_notes) VALUES
('Cyclic Dependency', 'Circular dependencies between modules or components', 'Dependency-Based', 'Detect using dependency graph analysis'),
('God Object', 'Classes or modules with too many responsibilities', 'Abstraction-Based', 'Analyze class size, method count, and coupling metrics'),
('Leaky Abstraction', 'Abstractions that expose implementation details', 'Abstraction-Based', 'Look for inappropriate exposure of internal state'),
('Tight Coupling', 'Excessive dependencies between components', 'Dependency-Based', 'Measure coupling metrics and dependency counts'),
('Long Parameter List', 'Functions with too many parameters', 'Code Smell', 'Count function parameters beyond reasonable threshold'),
('Duplicate Code', 'Repeated code blocks across the codebase', 'Code Smell', 'Use AST similarity analysis and string matching');
