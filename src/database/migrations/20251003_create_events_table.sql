-- Create events table for storing system events and audit trails
-- Used by EventRepository for tracking analysis events and system activities

CREATE TABLE events (
    -- Unique identifier for the event
    id TEXT PRIMARY KEY,

    -- Event type (e.g., "analysis.started", "security.issue_found")
    event_type TEXT NOT NULL,

    -- Optional source component that generated the event
    source TEXT,

    -- Event severity level (info, warn, error, critical)
    severity TEXT DEFAULT 'info',

    -- Human-readable event message
    message TEXT NOT NULL,

    -- JSON-encoded event metadata and context
    metadata TEXT,

    -- When the event occurred
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,

    -- Optional project context
    project_id INTEGER,

    -- Optional analysis context
    analysis_id TEXT,

    -- Optional user context
    user_id TEXT,

    -- Event correlation ID for grouping related events
    correlation_id TEXT
);

-- Index on event_type for filtering
CREATE INDEX idx_events_type ON events(event_type);

-- Index on timestamp for chronological queries
CREATE INDEX idx_events_timestamp ON events(timestamp);

-- Index on severity for filtering by importance
CREATE INDEX idx_events_severity ON events(severity);

-- Index on project_id for project-specific events
CREATE INDEX idx_events_project ON events(project_id);

-- Index on analysis_id for analysis-specific events
CREATE INDEX idx_events_analysis ON events(analysis_id);

-- Index on correlation_id for grouping related events
CREATE INDEX idx_events_correlation ON events(correlation_id);

-- Composite index for efficient event queries
CREATE INDEX idx_events_type_timestamp ON events(event_type, timestamp);