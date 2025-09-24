-- Create metrics table for storing performance and usage metrics
-- Used by MetricsRepository for monitoring and observability

CREATE TABLE metrics (
    -- Unique identifier for the metric entry
    id TEXT PRIMARY KEY,

    -- Metric name (e.g., "analysis.duration", "cache.hit_rate")
    name TEXT NOT NULL,

    -- Numeric value of the metric
    value REAL NOT NULL,

    -- JSON-encoded labels for dimensional data
    labels TEXT,

    -- Timestamp when the metric was recorded
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,

    -- Optional project context
    project_id INTEGER,

    -- Optional analysis context
    analysis_id TEXT,

    -- Metric type (gauge, counter, histogram)
    metric_type TEXT DEFAULT 'gauge'
);

-- Index on name for metric queries
CREATE INDEX idx_metrics_name ON metrics(name);

-- Index on timestamp for time-series queries
CREATE INDEX idx_metrics_timestamp ON metrics(timestamp);

-- Index on project_id for project-specific metrics
CREATE INDEX idx_metrics_project ON metrics(project_id);

-- Composite index for efficient metric queries
CREATE INDEX idx_metrics_name_timestamp ON metrics(name, timestamp);

-- Index on analysis_id for analysis-specific metrics
CREATE INDEX idx_metrics_analysis ON metrics(analysis_id);