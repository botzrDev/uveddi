-- V5__security_schema.sql
-- RBAC (Role-Based Access Control) schema for Uveddi security system
-- UV-247: Security and RBAC Implementation

-- Enable UUID extension for PostgreSQL (ignored by SQLite)
-- CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users table - stores user information from identity providers
CREATE TABLE users (
    id TEXT PRIMARY KEY DEFAULT (hex(randomblob(16))), -- UUID equivalent for SQLite
    external_id TEXT UNIQUE NOT NULL, -- From IdP (OAuth/OIDC)
    email TEXT UNIQUE NOT NULL,
    display_name TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1, -- SQLite boolean as INTEGER
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')), -- Unix timestamp
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')) -- Unix timestamp
);

-- Roles table - defines system roles with descriptions
CREATE TABLE roles (
    id TEXT PRIMARY KEY DEFAULT (hex(randomblob(16))), -- UUID equivalent for SQLite
    name TEXT UNIQUE NOT NULL,
    description TEXT,
    is_system_role INTEGER NOT NULL DEFAULT 0, -- SQLite boolean as INTEGER
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')) -- Unix timestamp
);

-- Permissions table - defines granular permissions
CREATE TABLE permissions (
    id TEXT PRIMARY KEY DEFAULT (hex(randomblob(16))), -- UUID equivalent for SQLite
    resource TEXT NOT NULL, -- e.g., 'projects', 'reports', 'analysis_runs'
    action TEXT NOT NULL,   -- e.g., 'read', 'write', 'delete', 'execute'
    scope TEXT,             -- e.g., 'own', 'team', 'all', 'organization'
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')), -- Unix timestamp
    UNIQUE(resource, action, scope)
);

-- User-Role assignments - many-to-many relationship
CREATE TABLE user_roles (
    user_id TEXT NOT NULL,
    role_id TEXT NOT NULL,
    granted_by TEXT, -- User ID who granted this role
    granted_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')), -- Unix timestamp
    expires_at INTEGER, -- Optional expiration timestamp
    PRIMARY KEY (user_id, role_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE,
    FOREIGN KEY (granted_by) REFERENCES users(id)
);

-- Role-Permission assignments - many-to-many relationship
CREATE TABLE role_permissions (
    role_id TEXT NOT NULL,
    permission_id TEXT NOT NULL,
    PRIMARY KEY (role_id, permission_id),
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE,
    FOREIGN KEY (permission_id) REFERENCES permissions(id) ON DELETE CASCADE
);

-- Sessions table - track user sessions for authentication
CREATE TABLE sessions (
    id TEXT PRIMARY KEY DEFAULT (hex(randomblob(16))), -- UUID equivalent for SQLite
    user_id TEXT NOT NULL,
    session_token TEXT UNIQUE NOT NULL,
    expires_at INTEGER NOT NULL, -- Unix timestamp
    ip_address TEXT,
    user_agent TEXT,
    is_active INTEGER NOT NULL DEFAULT 1, -- SQLite boolean as INTEGER
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')), -- Unix timestamp
    last_accessed INTEGER NOT NULL DEFAULT (strftime('%s', 'now')), -- Unix timestamp
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- API Keys table - for service-to-service authentication
CREATE TABLE api_keys (
    id TEXT PRIMARY KEY DEFAULT (hex(randomblob(16))), -- UUID equivalent for SQLite
    user_id TEXT, -- Optional - for user-specific API keys
    name TEXT NOT NULL, -- Human-readable name for the key
    key_hash TEXT UNIQUE NOT NULL, -- SHA-256 hash of the actual key
    key_prefix TEXT NOT NULL, -- First 8 characters for identification
    permissions TEXT, -- JSON array of permissions or role reference
    is_active INTEGER NOT NULL DEFAULT 1, -- SQLite boolean as INTEGER
    expires_at INTEGER, -- Optional expiration timestamp
    last_used INTEGER, -- Unix timestamp of last usage
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')), -- Unix timestamp
    created_by TEXT, -- User who created this key
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (created_by) REFERENCES users(id)
);

-- Audit Events table - comprehensive audit logging
CREATE TABLE audit_events (
    id TEXT PRIMARY KEY DEFAULT (hex(randomblob(16))), -- UUID equivalent for SQLite
    timestamp INTEGER NOT NULL DEFAULT (strftime('%s', 'now')), -- Unix timestamp
    event_type TEXT NOT NULL CHECK (event_type IN ('Authentication', 'Authorization', 'DataAccess', 'ConfigurationChange', 'SecurityViolation', 'SystemEvent')),
    user_id TEXT, -- Optional - for system events
    session_id TEXT, -- Optional - links to sessions table
    resource TEXT NOT NULL, -- What was accessed/modified
    action TEXT NOT NULL, -- What action was performed
    outcome TEXT NOT NULL CHECK (outcome IN ('Success', 'Failure', 'Denied')),
    ip_address TEXT,
    user_agent TEXT,
    additional_data TEXT, -- JSON for extra context
    integrity_hash TEXT NOT NULL, -- Blake3 hash for tamper detection
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (session_id) REFERENCES sessions(id)
);

-- Rate Limiting table - track API usage for rate limiting
CREATE TABLE rate_limits (
    id TEXT PRIMARY KEY DEFAULT (hex(randomblob(16))), -- UUID equivalent for SQLite
    identifier TEXT NOT NULL, -- IP, user_id, or api_key_id
    identifier_type TEXT NOT NULL CHECK (identifier_type IN ('ip', 'user', 'api_key')),
    endpoint TEXT NOT NULL, -- API endpoint pattern
    request_count INTEGER NOT NULL DEFAULT 1,
    window_start INTEGER NOT NULL, -- Unix timestamp of rate limit window start
    window_size INTEGER NOT NULL DEFAULT 3600, -- Window size in seconds (default 1 hour)
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')), -- Unix timestamp
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')) -- Unix timestamp
);

-- Create indexes for better query performance
CREATE INDEX idx_users_external_id ON users(external_id);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_is_active ON users(is_active);

CREATE INDEX idx_roles_name ON roles(name);
CREATE INDEX idx_roles_is_system_role ON roles(is_system_role);

CREATE INDEX idx_permissions_resource ON permissions(resource);
CREATE INDEX idx_permissions_action ON permissions(action);
CREATE INDEX idx_permissions_scope ON permissions(scope);

CREATE INDEX idx_user_roles_user_id ON user_roles(user_id);
CREATE INDEX idx_user_roles_role_id ON user_roles(role_id);
CREATE INDEX idx_user_roles_expires_at ON user_roles(expires_at);

CREATE INDEX idx_role_permissions_role_id ON role_permissions(role_id);
CREATE INDEX idx_role_permissions_permission_id ON role_permissions(permission_id);

CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_session_token ON sessions(session_token);
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);
CREATE INDEX idx_sessions_is_active ON sessions(is_active);

CREATE INDEX idx_api_keys_user_id ON api_keys(user_id);
CREATE INDEX idx_api_keys_key_hash ON api_keys(key_hash);
CREATE INDEX idx_api_keys_key_prefix ON api_keys(key_prefix);
CREATE INDEX idx_api_keys_is_active ON api_keys(is_active);
CREATE INDEX idx_api_keys_expires_at ON api_keys(expires_at);

CREATE INDEX idx_audit_events_timestamp ON audit_events(timestamp);
CREATE INDEX idx_audit_events_event_type ON audit_events(event_type);
CREATE INDEX idx_audit_events_user_id ON audit_events(user_id);
CREATE INDEX idx_audit_events_resource ON audit_events(resource);
CREATE INDEX idx_audit_events_outcome ON audit_events(outcome);

CREATE INDEX idx_rate_limits_identifier ON rate_limits(identifier);
CREATE INDEX idx_rate_limits_identifier_type ON rate_limits(identifier_type);
CREATE INDEX idx_rate_limits_endpoint ON rate_limits(endpoint);
CREATE INDEX idx_rate_limits_window_start ON rate_limits(window_start);

-- Insert default system roles
INSERT INTO roles (name, description, is_system_role) VALUES
('Admin', 'Full system access and configuration', 1),
('Developer', 'Test data access for owned projects only', 1),
('QA', 'Test execution and failure analysis access', 1),
('Manager', 'Read-only access to reports and dashboards', 1),
('Service', 'API access for automated integrations', 1);

-- Insert default permissions
INSERT INTO permissions (resource, action, scope) VALUES
-- Project permissions
('projects', 'read', 'own'),
('projects', 'read', 'team'),
('projects', 'read', 'all'),
('projects', 'write', 'own'),
('projects', 'write', 'team'),
('projects', 'write', 'all'),
('projects', 'delete', 'own'),
('projects', 'delete', 'team'),
('projects', 'delete', 'all'),

-- Analysis permissions
('analysis_runs', 'read', 'own'),
('analysis_runs', 'read', 'team'),
('analysis_runs', 'read', 'all'),
('analysis_runs', 'execute', 'own'),
('analysis_runs', 'execute', 'team'),
('analysis_runs', 'execute', 'all'),
('analysis_runs', 'delete', 'own'),
('analysis_runs', 'delete', 'team'),
('analysis_runs', 'delete', 'all'),

-- Report permissions
('reports', 'read', 'own'),
('reports', 'read', 'team'),
('reports', 'read', 'all'),
('reports', 'write', 'own'),
('reports', 'write', 'team'),
('reports', 'write', 'all'),
('reports', 'delete', 'own'),
('reports', 'delete', 'team'),
('reports', 'delete', 'all'),

-- System permissions
('system', 'read', 'all'),
('system', 'configure', 'all'),
('users', 'read', 'all'),
('users', 'write', 'all'),
('roles', 'read', 'all'),
('roles', 'write', 'all'),
('audit', 'read', 'all');

-- Set up default role permissions
-- Admin: Full access to everything
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name = 'Admin';

-- Developer: Own projects and analysis runs
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name = 'Developer' AND (
    (p.resource = 'projects' AND p.scope = 'own') OR
    (p.resource = 'analysis_runs' AND p.scope = 'own') OR
    (p.resource = 'reports' AND p.scope = 'own')
);

-- QA: Team-level access to projects and analysis runs
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name = 'QA' AND (
    (p.resource = 'projects' AND p.scope IN ('own', 'team')) OR
    (p.resource = 'analysis_runs' AND p.scope IN ('own', 'team')) OR
    (p.resource = 'reports' AND p.scope IN ('own', 'team'))
);

-- Manager: Read-only access to all reports and projects
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name = 'Manager' AND (
    (p.resource = 'projects' AND p.action = 'read') OR
    (p.resource = 'analysis_runs' AND p.action = 'read') OR
    (p.resource = 'reports' AND p.action = 'read')
);

-- Service: API access for automated integrations
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name = 'Service' AND (
    (p.resource = 'projects' AND p.action IN ('read', 'write')) OR
    (p.resource = 'analysis_runs' AND p.action IN ('read', 'execute')) OR
    (p.resource = 'reports' AND p.action = 'read')
);