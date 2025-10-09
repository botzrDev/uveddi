-- Create security_findings table for storing security analysis results
-- Used by SecurityRepository for managing security vulnerabilities and issues

CREATE TABLE security_findings (
    -- Unique identifier for the security finding
    id TEXT PRIMARY KEY,

    -- Analysis run that detected this finding
    analysis_id TEXT NOT NULL,

    -- Project context
    project_id INTEGER,

    -- OWASP category (if applicable)
    owasp_category TEXT,

    -- CWE identifier (if applicable)
    cwe_id TEXT,

    -- CVE identifier (if applicable)
    cve_id TEXT,

    -- Security rule/detector that found this issue
    rule_id TEXT NOT NULL,

    -- Finding severity (info, low, medium, high, critical)
    severity TEXT NOT NULL,

    -- Risk level (low, medium, high, critical)
    risk_level TEXT NOT NULL,

    -- File path where the finding was detected
    file_path TEXT NOT NULL,

    -- Line number (if applicable)
    line_number INTEGER,

    -- Column number (if applicable)
    column_number INTEGER,

    -- Function or method name (if applicable)
    function_name TEXT,

    -- Finding title/summary
    title TEXT NOT NULL,

    -- Detailed description of the security issue
    description TEXT,

    -- Code snippet showing the vulnerable code
    code_snippet TEXT,

    -- Recommended remediation
    remediation TEXT,

    -- References and links for more information
    references TEXT,

    -- JSON-encoded taint flow analysis (if applicable)
    taint_flow TEXT,

    -- JSON-encoded metadata (AST nodes, additional context)
    metadata TEXT,

    -- Finding confidence score (0.0 to 1.0)
    confidence REAL DEFAULT 1.0,

    -- Impact score (0.0 to 10.0)
    impact_score REAL DEFAULT 5.0,

    -- Exploitability score (0.0 to 10.0)
    exploitability_score REAL DEFAULT 5.0,

    -- When the finding was first detected
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    -- When the finding was last seen
    last_seen DATETIME DEFAULT CURRENT_TIMESTAMP,

    -- Finding status (active, resolved, suppressed, false_positive)
    status TEXT DEFAULT 'active',

    -- Hash for duplicate detection
    context_hash TEXT
);

-- Index on analysis_id for analysis-specific queries
CREATE INDEX idx_sec_analysis ON security_findings(analysis_id);

-- Index on project_id for project-specific queries
CREATE INDEX idx_sec_project ON security_findings(project_id);

-- Index on owasp_category for OWASP Top 10 analysis
CREATE INDEX idx_sec_owasp ON security_findings(owasp_category);

-- Index on cwe_id for CWE classification
CREATE INDEX idx_sec_cwe ON security_findings(cwe_id);

-- Index on severity for prioritization
CREATE INDEX idx_sec_severity ON security_findings(severity);

-- Index on risk_level for risk assessment
CREATE INDEX idx_sec_risk ON security_findings(risk_level);

-- Index on rule_id for detector-specific queries
CREATE INDEX idx_sec_rule ON security_findings(rule_id);

-- Index on file_path for file-specific queries
CREATE INDEX idx_sec_file ON security_findings(file_path);

-- Index on status for filtering active findings
CREATE INDEX idx_sec_status ON security_findings(status);

-- Index on context_hash for duplicate detection
CREATE INDEX idx_sec_context_hash ON security_findings(context_hash);

-- Composite index for efficient security queries
CREATE INDEX idx_sec_project_status ON security_findings(project_id, status);

-- Composite index for severity analysis
CREATE INDEX idx_sec_severity_risk ON security_findings(severity, risk_level);