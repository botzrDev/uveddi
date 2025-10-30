# Database Schema Documentation

## Overview

This document provides comprehensive documentation of the Uveddi database schema, including all tables, relationships, indexes, and migration details.

## Schema Version History

| Version | Date | Description | Migration File |
|---------|------|-------------|----------------|
| 001 | 2024 | Create cache table | 001_create_cache_table.sql |
| 002 | 2024 | Create metrics table | 002_create_metrics_table.sql |
| 003 | 2024 | Create events table | 003_create_events_table.sql |
| 004 | 2024 | Create issues table | 004_create_issues_table.sql |
| 005 | 2024 | Create dependencies table | 005_create_dependencies_table.sql |
| 006 | 2024 | Create security findings table | 006_create_security_findings_table.sql |
| 007 | 2024 | Create technical debt table | 007_create_technical_debt_table.sql |

## Core Tables

### projects

Stores information about analyzed projects.

```sql
CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    path TEXT NOT NULL UNIQUE,
    language TEXT,
    framework TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Indexes
CREATE INDEX idx_projects_path ON projects(path);
CREATE INDEX idx_projects_created_at ON projects(created_at);
CREATE INDEX idx_projects_language ON projects(language);
```

**Columns:**
- `id` (TEXT, PK): UUID of the project
- `name` (TEXT): Project name
- `path` (TEXT): File system path to project (unique)
- `language` (TEXT): Primary programming language
- `framework` (TEXT): Framework used (e.g., React, Django)
- `created_at` (DATETIME): Creation timestamp
- `updated_at` (DATETIME): Last update timestamp

### analysis_runs

Tracks individual analysis executions.

```sql
CREATE TABLE IF NOT EXISTS analysis_runs (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    started_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    completed_at DATETIME,
    status TEXT DEFAULT 'pending',
    error TEXT,
    metadata TEXT,  -- JSON
    FOREIGN KEY (project_id) REFERENCES projects(id)
);

-- Indexes
CREATE INDEX idx_analysis_runs_project_id ON analysis_runs(project_id);
CREATE INDEX idx_analysis_runs_status ON analysis_runs(status);
CREATE INDEX idx_analysis_runs_started_at ON analysis_runs(started_at);
```

**Columns:**
- `id` (TEXT, PK): UUID of the analysis run
- `project_id` (TEXT, FK): Associated project
- `started_at` (DATETIME): When analysis started
- `completed_at` (DATETIME): When analysis completed
- `status` (TEXT): pending, running, completed, failed
- `error` (TEXT): Error message if failed
- `metadata` (TEXT/JSON): Additional analysis metadata

## Cache and Performance Tables

### cache (Migration 001)

Implements LRU cache for expensive computations.

```sql
CREATE TABLE IF NOT EXISTS cache (
    id TEXT PRIMARY KEY,
    key TEXT NOT NULL UNIQUE,
    data BLOB NOT NULL,
    metadata TEXT,  -- JSON with size, type, etc.
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    accessed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    expires_at DATETIME,
    access_count INTEGER DEFAULT 0,
    size_bytes INTEGER
);

-- Indexes for efficient cache operations
CREATE UNIQUE INDEX idx_cache_key ON cache(key);
CREATE INDEX idx_cache_expires_at ON cache(expires_at);
CREATE INDEX idx_cache_accessed_at ON cache(accessed_at);
CREATE INDEX idx_cache_size ON cache(size_bytes);
```

**Columns:**
- `id` (TEXT, PK): UUID of cache entry
- `key` (TEXT): Unique cache key
- `data` (BLOB): Cached data (compressed)
- `metadata` (TEXT/JSON): Type, compression, etc.
- `created_at` (DATETIME): Creation time
- `accessed_at` (DATETIME): Last access time (for LRU)
- `expires_at` (DATETIME): TTL expiration
- `access_count` (INTEGER): Hit counter
- `size_bytes` (INTEGER): Size for memory management

### metrics (Migration 002)

Stores performance and usage metrics.

```sql
CREATE TABLE IF NOT EXISTS metrics (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    value REAL NOT NULL,
    unit TEXT,
    labels TEXT,  -- JSON key-value pairs
    metadata TEXT,  -- JSON additional context
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    aggregation_type TEXT DEFAULT 'gauge'  -- gauge, counter, histogram
);

-- Indexes for time-series queries
CREATE INDEX idx_metrics_name ON metrics(name);
CREATE INDEX idx_metrics_timestamp ON metrics(timestamp);
CREATE INDEX idx_metrics_name_timestamp ON metrics(name, timestamp);
CREATE INDEX idx_metrics_aggregation ON metrics(aggregation_type);
```

**Columns:**
- `id` (TEXT, PK): UUID of metric
- `name` (TEXT): Metric name (e.g., "analysis.duration")
- `value` (REAL): Numeric value
- `unit` (TEXT): Unit of measurement
- `labels` (TEXT/JSON): Dimensional labels
- `metadata` (TEXT/JSON): Additional context
- `timestamp` (DATETIME): When recorded
- `aggregation_type` (TEXT): How to aggregate

## Audit and Event Tables

### events (Migration 003)

System events and audit trail.

```sql
CREATE TABLE IF NOT EXISTS events (
    id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,
    severity TEXT DEFAULT 'info',  -- debug, info, warning, error, critical
    source TEXT NOT NULL,  -- Component that generated event
    target TEXT,  -- Entity affected (e.g., project_id, analysis_id)
    target_type TEXT,  -- Type of target entity
    message TEXT NOT NULL,
    data TEXT,  -- JSON event-specific data
    metadata TEXT,  -- JSON additional context
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    acknowledged_at DATETIME,
    acknowledged_by TEXT
);

-- Indexes for event queries
CREATE INDEX idx_events_type ON events(event_type);
CREATE INDEX idx_events_severity ON events(severity);
CREATE INDEX idx_events_source ON events(source);
CREATE INDEX idx_events_target ON events(target, target_type);
CREATE INDEX idx_events_created_at ON events(created_at);
CREATE INDEX idx_events_acknowledged ON events(acknowledged_at);
```

**Columns:**
- `id` (TEXT, PK): UUID of event
- `event_type` (TEXT): Type of event
- `severity` (TEXT): Event severity level
- `source` (TEXT): Component source
- `target` (TEXT): Affected entity ID
- `target_type` (TEXT): Type of target
- `message` (TEXT): Human-readable message
- `data` (TEXT/JSON): Event payload
- `metadata` (TEXT/JSON): Additional context
- `created_at` (DATETIME): Event time
- `acknowledged_at` (DATETIME): When acknowledged
- `acknowledged_by` (TEXT): Who acknowledged

## Analysis Results Tables

### issues (Migration 004)

Detected code issues and problems.

```sql
CREATE TABLE IF NOT EXISTS issues (
    id TEXT PRIMARY KEY,
    analysis_run_id TEXT NOT NULL,
    issue_type TEXT NOT NULL,  -- bug, vulnerability, code_smell, etc.
    severity TEXT NOT NULL,  -- critical, high, medium, low
    category TEXT NOT NULL,  -- security, performance, maintainability, etc.
    rule_id TEXT,  -- Reference to detection rule
    file_path TEXT NOT NULL,
    line_start INTEGER,
    line_end INTEGER,
    column_start INTEGER,
    column_end INTEGER,
    message TEXT NOT NULL,
    suggestion TEXT,  -- Suggested fix
    metadata TEXT,  -- JSON with additional details
    fingerprint TEXT,  -- For deduplication
    status TEXT DEFAULT 'open',  -- open, resolved, ignored, false_positive
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (analysis_run_id) REFERENCES analysis_runs(id)
);

-- Indexes for issue queries
CREATE INDEX idx_issues_analysis_run ON issues(analysis_run_id);
CREATE INDEX idx_issues_type ON issues(issue_type);
CREATE INDEX idx_issues_severity ON issues(severity);
CREATE INDEX idx_issues_category ON issues(category);
CREATE INDEX idx_issues_file ON issues(file_path);
CREATE INDEX idx_issues_status ON issues(status);
CREATE INDEX idx_issues_fingerprint ON issues(fingerprint);
```

**Columns:**
- `id` (TEXT, PK): UUID of issue
- `analysis_run_id` (TEXT, FK): Associated analysis
- `issue_type` (TEXT): Type of issue
- `severity` (TEXT): Severity level
- `category` (TEXT): Issue category
- `rule_id` (TEXT): Detection rule reference
- `file_path` (TEXT): File containing issue
- `line_start/end` (INTEGER): Line range
- `column_start/end` (INTEGER): Column range
- `message` (TEXT): Issue description
- `suggestion` (TEXT): Fix suggestion
- `metadata` (TEXT/JSON): Additional data
- `fingerprint` (TEXT): Dedup hash
- `status` (TEXT): Issue status

### dependencies (Migration 005)

Code dependency relationships.

```sql
CREATE TABLE IF NOT EXISTS dependencies (
    id TEXT PRIMARY KEY,
    analysis_run_id TEXT NOT NULL,
    source_type TEXT NOT NULL,  -- file, module, class, function
    source_path TEXT NOT NULL,
    source_name TEXT NOT NULL,
    target_type TEXT NOT NULL,
    target_path TEXT NOT NULL,
    target_name TEXT NOT NULL,
    dependency_type TEXT NOT NULL,  -- import, inheritance, composition, call
    strength TEXT DEFAULT 'normal',  -- weak, normal, strong
    metadata TEXT,  -- JSON with language-specific details
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (analysis_run_id) REFERENCES analysis_runs(id)
);

-- Indexes for dependency queries
CREATE INDEX idx_dependencies_analysis ON dependencies(analysis_run_id);
CREATE INDEX idx_dependencies_source ON dependencies(source_path, source_name);
CREATE INDEX idx_dependencies_target ON dependencies(target_path, target_name);
CREATE INDEX idx_dependencies_type ON dependencies(dependency_type);
CREATE INDEX idx_dependencies_strength ON dependencies(strength);
```

**Columns:**
- `id` (TEXT, PK): UUID of dependency
- `analysis_run_id` (TEXT, FK): Associated analysis
- `source_type` (TEXT): Type of source entity
- `source_path` (TEXT): Source file path
- `source_name` (TEXT): Source entity name
- `target_type` (TEXT): Type of target entity
- `target_path` (TEXT): Target file path
- `target_name` (TEXT): Target entity name
- `dependency_type` (TEXT): Type of dependency
- `strength` (TEXT): Dependency strength
- `metadata` (TEXT/JSON): Additional details

## Security and Quality Tables

### security_findings (Migration 006)

Security vulnerabilities and risks.

```sql
CREATE TABLE IF NOT EXISTS security_findings (
    id TEXT PRIMARY KEY,
    analysis_run_id TEXT NOT NULL,
    vulnerability_type TEXT NOT NULL,  -- SQL injection, XSS, etc.
    severity TEXT NOT NULL,  -- critical, high, medium, low
    confidence TEXT NOT NULL,  -- high, medium, low
    cwe_id TEXT,  -- Common Weakness Enumeration ID
    owasp_category TEXT,  -- OWASP Top 10 category
    file_path TEXT NOT NULL,
    line_number INTEGER,
    code_snippet TEXT,
    description TEXT NOT NULL,
    impact TEXT,
    remediation TEXT,
    references TEXT,  -- JSON array of reference URLs
    metadata TEXT,  -- JSON additional details
    false_positive BOOLEAN DEFAULT FALSE,
    verified BOOLEAN DEFAULT FALSE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (analysis_run_id) REFERENCES analysis_runs(id)
);

-- Indexes for security queries
CREATE INDEX idx_security_analysis ON security_findings(analysis_run_id);
CREATE INDEX idx_security_type ON security_findings(vulnerability_type);
CREATE INDEX idx_security_severity ON security_findings(severity);
CREATE INDEX idx_security_cwe ON security_findings(cwe_id);
CREATE INDEX idx_security_owasp ON security_findings(owasp_category);
CREATE INDEX idx_security_file ON security_findings(file_path);
CREATE INDEX idx_security_verified ON security_findings(verified);
```

**Columns:**
- `id` (TEXT, PK): UUID of finding
- `analysis_run_id` (TEXT, FK): Associated analysis
- `vulnerability_type` (TEXT): Type of vulnerability
- `severity` (TEXT): Severity level
- `confidence` (TEXT): Detection confidence
- `cwe_id` (TEXT): CWE identifier
- `owasp_category` (TEXT): OWASP category
- `file_path` (TEXT): Affected file
- `line_number` (INTEGER): Line number
- `code_snippet` (TEXT): Vulnerable code
- `description` (TEXT): Detailed description
- `impact` (TEXT): Potential impact
- `remediation` (TEXT): How to fix
- `references` (TEXT/JSON): Reference links
- `metadata` (TEXT/JSON): Additional data
- `false_positive` (BOOLEAN): Marked as FP
- `verified` (BOOLEAN): Manually verified

### technical_debt (Migration 007)

Technical debt and code quality issues.

```sql
CREATE TABLE IF NOT EXISTS technical_debt (
    id TEXT PRIMARY KEY,
    analysis_run_id TEXT NOT NULL,
    debt_type TEXT NOT NULL,  -- design, documentation, test, etc.
    category TEXT NOT NULL,  -- maintainability, reliability, etc.
    severity TEXT NOT NULL,  -- high, medium, low
    effort_minutes INTEGER,  -- Estimated fix time
    principal_minutes INTEGER,  -- Time saved by not doing it right
    interest_minutes INTEGER,  -- Extra time due to debt
    file_path TEXT,
    component TEXT,  -- Affected component/module
    description TEXT NOT NULL,
    impact TEXT,
    recommendation TEXT,
    tags TEXT,  -- JSON array of tags
    metadata TEXT,  -- JSON additional details
    status TEXT DEFAULT 'identified',  -- identified, planned, fixing, resolved
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (analysis_run_id) REFERENCES analysis_runs(id)
);

-- Indexes for debt queries
CREATE INDEX idx_debt_analysis ON technical_debt(analysis_run_id);
CREATE INDEX idx_debt_type ON technical_debt(debt_type);
CREATE INDEX idx_debt_category ON technical_debt(category);
CREATE INDEX idx_debt_severity ON technical_debt(severity);
CREATE INDEX idx_debt_effort ON technical_debt(effort_minutes);
CREATE INDEX idx_debt_component ON technical_debt(component);
CREATE INDEX idx_debt_status ON technical_debt(status);
```

**Columns:**
- `id` (TEXT, PK): UUID of debt item
- `analysis_run_id` (TEXT, FK): Associated analysis
- `debt_type` (TEXT): Type of debt
- `category` (TEXT): Debt category
- `severity` (TEXT): Severity level
- `effort_minutes` (INTEGER): Fix effort estimate
- `principal_minutes` (INTEGER): Original time saved
- `interest_minutes` (INTEGER): Ongoing cost
- `file_path` (TEXT): Affected file
- `component` (TEXT): Affected component
- `description` (TEXT): Debt description
- `impact` (TEXT): Business impact
- `recommendation` (TEXT): How to address
- `tags` (TEXT/JSON): Categorization tags
- `metadata` (TEXT/JSON): Additional data
- `status` (TEXT): Current status

## Migration Management Table

### schema_migrations

Tracks applied migrations (created automatically by migration runner).

```sql
CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    checksum TEXT NOT NULL,
    applied_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    execution_time_ms INTEGER
);
```

**Columns:**
- `version` (INTEGER, PK): Migration version number
- `name` (TEXT): Migration name
- `checksum` (TEXT): Migration file checksum
- `applied_at` (DATETIME): When applied
- `execution_time_ms` (INTEGER): Execution duration

## Relationships

### Entity Relationship Diagram

```
projects (1) ─────< (N) analysis_runs
                              │
                              ├──< issues
                              ├──< dependencies
                              ├──< security_findings
                              └──< technical_debt

cache (standalone)
metrics (standalone)
events (references various entities)
schema_migrations (standalone)
```

### Foreign Key Relationships

1. **analysis_runs.project_id** → projects.id
2. **issues.analysis_run_id** → analysis_runs.id
3. **dependencies.analysis_run_id** → analysis_runs.id
4. **security_findings.analysis_run_id** → analysis_runs.id
5. **technical_debt.analysis_run_id** → analysis_runs.id

## Index Strategy

### Primary Indexes

All tables have primary key indexes automatically created.

### Query Optimization Indexes

1. **Lookup Indexes**: For finding specific records
   - `idx_projects_path`: Quick project lookup by path
   - `idx_cache_key`: Fast cache retrieval
   - `idx_issues_fingerprint`: Deduplication checks

2. **Range Scan Indexes**: For time-based queries
   - `idx_analysis_runs_started_at`: Recent analyses
   - `idx_metrics_timestamp`: Time-series queries
   - `idx_events_created_at`: Event history

3. **Filter Indexes**: For common WHERE clauses
   - `idx_issues_severity`: Filter by severity
   - `idx_security_verified`: Find verified issues
   - `idx_debt_status`: Track debt resolution

4. **Composite Indexes**: For multi-column queries
   - `idx_metrics_name_timestamp`: Metric time-series
   - `idx_dependencies_source`: Source lookups

## Data Types and Constraints

### UUID Format
All ID fields use UUID v4 format stored as TEXT:
```
Format: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
Example: 550e8400-e29b-41d4-a716-446655440000
```

### JSON Fields
Fields marked as JSON store structured data:
```json
{
  "key": "value",
  "nested": {
    "field": 123
  },
  "array": [1, 2, 3]
}
```

### DateTime Format
All datetime fields use ISO 8601 format:
```
Format: YYYY-MM-DD HH:MM:SS
Example: 2024-01-15 14:30:00
```

### Status Enumerations

**Analysis Status:**
- pending
- running
- completed
- failed

**Issue Status:**
- open
- resolved
- ignored
- false_positive

**Severity Levels:**
- critical
- high
- medium
- low

**Event Severity:**
- debug
- info
- warning
- error
- critical

## Performance Considerations

### Index Coverage

Each table has indexes covering:
1. Primary key (automatic)
2. Foreign keys (for joins)
3. Common query patterns
4. Time-based queries
5. Status/type filters

### Query Optimization

1. **Use covering indexes** where possible
2. **Avoid full table scans** on large tables
3. **Use EXPLAIN QUERY PLAN** to verify index usage
4. **Batch inserts** for bulk operations
5. **Use transactions** for related updates

### Storage Optimization

1. **BLOB compression** for cache.data
2. **JSON compression** for large metadata fields
3. **Regular VACUUM** for SQLite databases
4. **Archive old data** periodically

## Migration Best Practices

### Writing Migrations

1. **Idempotent**: Use `IF NOT EXISTS` clauses
2. **Reversible**: Provide DOWN migrations
3. **Atomic**: Wrap in transactions
4. **Tested**: Verify on test database first
5. **Documented**: Clear descriptions

### Migration Template

```sql
-- Migration: XXX_description
-- Author: Name
-- Date: YYYY-MM-DD
-- Description: What this migration does

-- UP Migration
BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS table_name (
    -- columns
);

CREATE INDEX IF NOT EXISTS idx_name ON table_name(column);

COMMIT;

-- DOWN Migration (in separate file or section)
BEGIN TRANSACTION;

DROP INDEX IF EXISTS idx_name;
DROP TABLE IF EXISTS table_name;

COMMIT;
```

## Maintenance Tasks

### Regular Maintenance

1. **Daily**:
   - Clear expired cache entries
   - Archive old events

2. **Weekly**:
   - VACUUM database (SQLite)
   - Update statistics (PostgreSQL)
   - Review slow queries

3. **Monthly**:
   - Archive old analysis runs
   - Compress large metadata fields
   - Review index usage

### Health Checks

```sql
-- Check table sizes
SELECT
    name AS table_name,
    COUNT(*) AS row_count
FROM sqlite_master
WHERE type = 'table'
GROUP BY name;

-- Check index usage
EXPLAIN QUERY PLAN
SELECT * FROM issues WHERE severity = 'high';

-- Find missing indexes
SELECT DISTINCT file_path
FROM issues
WHERE analysis_run_id = ?;
```

## Conclusion

This database schema provides:

1. **Comprehensive Coverage**: All aspects of code analysis
2. **Performance Optimization**: Strategic indexing
3. **Flexibility**: JSON fields for extensibility
4. **Auditability**: Event tracking and timestamps
5. **Scalability**: Designed for large codebases

The migration system ensures smooth schema evolution while maintaining data integrity and backward compatibility.