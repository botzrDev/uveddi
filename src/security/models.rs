//! Security data models for the Uveddi RBAC system
//!
//! This module defines the core data structures used throughout the security system,
//! including users, roles, permissions, sessions, and audit events.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// User information from identity providers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct User {
    pub id: Uuid,
    pub external_id: String,
    pub email: String,
    pub display_name: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    /// Create a new user
    pub fn new(external_id: String, email: String, display_name: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            external_id,
            email,
            display_name,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if user is active
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Deactivate user
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }

    /// Activate user
    pub fn activate(&mut self) {
        self.is_active = true;
        self.updated_at = Utc::now();
    }
}

/// System role definitions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_system_role: bool,
    pub created_at: DateTime<Utc>,
}

impl Role {
    /// Create a new role
    pub fn new(name: String, description: Option<String>, is_system_role: bool) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            is_system_role,
            created_at: Utc::now(),
        }
    }

    /// Check if this is a system role
    pub fn is_system_role(&self) -> bool {
        self.is_system_role
    }
}

/// User roles defining access levels and permissions within the system
///
/// Roles are hierarchical with each level including capabilities of lower levels
/// where applicable. Used throughout the system for authorization decisions.
/// 
/// # Security Model
/// 
/// The role system implements a capability-based security model where each role
/// grants specific permissions for different system operations. Roles are enforced
/// at multiple levels including API endpoints, data access, and UI features.
/// 
/// # Examples
/// 
/// ```rust
/// use uveddi::security::models::UserRole;
/// 
/// let user_role = UserRole::Developer;
/// assert!(user_role.can_access_analysis_data());
/// assert!(!user_role.is_admin());
/// 
/// let admin_role = UserRole::Admin;
/// assert!(admin_role.is_admin());
/// assert!(admin_role.can_manage_users());
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum UserRole {
    /// Full system administrator with unrestricted access
    ///
    /// **Capabilities:**
    /// - Full system configuration access and settings management
    /// - User management and role assignment for all users
    /// - All analysis operations and unrestricted data access
    /// - System monitoring, maintenance, and troubleshooting
    /// - Security configuration and audit log access
    /// - Plugin and integration management
    Admin,
    
    /// Development team member with project-focused access
    ///
    /// **Capabilities:**
    /// - Analysis execution for owned/assigned projects
    /// - Code quality metrics access and trend analysis
    /// - Issue tracking, resolution, and code recommendations
    /// - Limited to development-related operations and tools
    /// - Can create and manage personal API keys
    /// - Access to development and staging environments
    Developer,
    
    /// Quality assurance team member with testing focus
    ///
    /// **Capabilities:**
    /// - Test execution and comprehensive failure analysis
    /// - Quality metrics, reporting access, and trend monitoring
    /// - Issue verification, validation, and regression testing
    /// - Read-only access to most analysis results and reports
    /// - Can trigger analysis runs for testing purposes
    /// - Limited access to test environment configurations
    QA,
    
    /// Management role with reporting and oversight access
    ///
    /// **Capabilities:**
    /// - Read-only access to reports, dashboards, and analytics
    /// - Team performance metrics and productivity insights
    /// - Project status, progress tracking, and milestone reporting
    /// - High-level quality and security trend analysis
    /// - No direct system configuration or development access
    /// - Can export reports and configure dashboard views
    Manager,
    
    /// Service account for automated integrations and CI/CD
    ///
    /// **Capabilities:**
    /// - API access for automated systems and integrations
    /// - Limited to specific integration endpoints and operations
    /// - No interactive UI access or user-facing features
    /// - Restricted to programmatic operations only
    /// - Can trigger analysis runs via API
    /// - Limited data export capabilities for integration purposes
    Service,
}

impl UserRole {
    /// Get the role name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::Admin => "Admin",
            UserRole::Developer => "Developer",
            UserRole::QA => "QA",
            UserRole::Manager => "Manager",
            UserRole::Service => "Service",
        }
    }

    /// Get role description
    pub fn description(&self) -> &'static str {
        match self {
            UserRole::Admin => "Full system access and configuration",
            UserRole::Developer => "Test data access for owned projects only",
            UserRole::QA => "Test execution and failure analysis access",
            UserRole::Manager => "Read-only access to reports and dashboards",
            UserRole::Service => "API access for automated integrations",
        }
    }

    /// Check if this role has administrative privileges
    pub fn is_admin(&self) -> bool {
        matches!(self, UserRole::Admin)
    }

    /// Check if this role can manage users
    pub fn can_manage_users(&self) -> bool {
        matches!(self, UserRole::Admin)
    }

    /// Check if this role can configure system settings
    pub fn can_configure_system(&self) -> bool {
        matches!(self, UserRole::Admin)
    }
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for UserRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Admin" => Ok(UserRole::Admin),
            "Developer" => Ok(UserRole::Developer),
            "QA" => Ok(UserRole::QA),
            "Manager" => Ok(UserRole::Manager),
            "Service" => Ok(UserRole::Service),
            _ => Err(format!("Invalid role: {}", s)),
        }
    }
}

/// Permission definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Permission {
    pub id: Uuid,
    pub resource: String,
    pub action: String,
    pub scope: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Permission {
    /// Create a new permission
    pub fn new(resource: String, action: String, scope: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            resource,
            action,
            scope,
            created_at: Utc::now(),
        }
    }

    /// Create a permission key for matching
    pub fn key(&self) -> String {
        match &self.scope {
            Some(scope) => format!("{}:{}:{}", self.resource, self.action, scope),
            None => format!("{}:{}", self.resource, self.action),
        }
    }

    /// Check if this permission matches the given resource and action
    pub fn matches(&self, resource: &str, action: &str, scope: Option<&str>) -> bool {
        self.resource == resource && self.action == action && self.scope.as_deref() == scope
    }
}

/// User role assignment
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserRoleAssignment {
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub granted_by: Option<Uuid>,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl UserRoleAssignment {
    /// Create a new user role assignment
    pub fn new(user_id: Uuid, role_id: Uuid, granted_by: Option<Uuid>) -> Self {
        Self {
            user_id,
            role_id,
            granted_by,
            granted_at: Utc::now(),
            expires_at: None,
        }
    }

    /// Create a new user role assignment with expiration
    pub fn new_with_expiration(
        user_id: Uuid,
        role_id: Uuid,
        granted_by: Option<Uuid>,
        expires_at: DateTime<Utc>,
    ) -> Self {
        Self {
            user_id,
            role_id,
            granted_by,
            granted_at: Utc::now(),
            expires_at: Some(expires_at),
        }
    }

    /// Check if this assignment has expired
    pub fn is_expired(&self) -> bool {
        self.expires_at
            .map_or(false, |expires| expires < Utc::now())
    }

    /// Check if this assignment is valid (not expired)
    pub fn is_valid(&self) -> bool {
        !self.is_expired()
    }
}

/// User session information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub session_token: String,
    pub expires_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
}

impl Session {
    /// Create a new session
    pub fn new(
        user_id: Uuid,
        session_token: String,
        expires_at: DateTime<Utc>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            session_token,
            expires_at,
            ip_address,
            user_agent,
            is_active: true,
            created_at: now,
            last_accessed: now,
        }
    }

    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if session is valid (active and not expired)
    pub fn is_valid(&self) -> bool {
        self.is_active && !self.is_expired()
    }

    /// Update last accessed time
    pub fn update_last_accessed(&mut self) {
        self.last_accessed = Utc::now();
    }

    /// Deactivate session
    pub fn deactivate(&mut self) {
        self.is_active = false;
    }
}

/// API key for service-to-service authentication
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiKey {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub name: String,
    pub key_hash: String,
    pub key_prefix: String,
    pub permissions: Option<String>, // JSON string
    pub is_active: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
}

impl ApiKey {
    /// Create a new API key
    pub fn new(
        user_id: Option<Uuid>,
        name: String,
        key_hash: String,
        key_prefix: String,
        permissions: Option<String>,
        created_by: Option<Uuid>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            name,
            key_hash,
            key_prefix,
            permissions,
            is_active: true,
            expires_at: None,
            last_used: None,
            created_at: Utc::now(),
            created_by,
        }
    }

    /// Check if API key is expired
    pub fn is_expired(&self) -> bool {
        self.expires_at
            .map_or(false, |expires| expires < Utc::now())
    }

    /// Check if API key is valid (active and not expired)
    pub fn is_valid(&self) -> bool {
        self.is_active && !self.is_expired()
    }

    /// Update last used time
    pub fn update_last_used(&mut self) {
        self.last_used = Some(Utc::now());
    }

    /// Deactivate API key
    pub fn deactivate(&mut self) {
        self.is_active = false;
    }
}

/// Audit event types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditEventType {
    Authentication,
    Authorization,
    DataAccess,
    ConfigurationChange,
    SecurityViolation,
    SystemEvent,
}

impl std::fmt::Display for AuditEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditEventType::Authentication => write!(f, "Authentication"),
            AuditEventType::Authorization => write!(f, "Authorization"),
            AuditEventType::DataAccess => write!(f, "DataAccess"),
            AuditEventType::ConfigurationChange => write!(f, "ConfigurationChange"),
            AuditEventType::SecurityViolation => write!(f, "SecurityViolation"),
            AuditEventType::SystemEvent => write!(f, "SystemEvent"),
        }
    }
}

impl std::str::FromStr for AuditEventType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Authentication" => Ok(AuditEventType::Authentication),
            "Authorization" => Ok(AuditEventType::Authorization),
            "DataAccess" => Ok(AuditEventType::DataAccess),
            "ConfigurationChange" => Ok(AuditEventType::ConfigurationChange),
            "SecurityViolation" => Ok(AuditEventType::SecurityViolation),
            "SystemEvent" => Ok(AuditEventType::SystemEvent),
            _ => Err(format!("Invalid audit event type: {}", s)),
        }
    }
}

/// Audit outcome
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditOutcome {
    Success,
    Failure,
    Denied,
}

impl std::fmt::Display for AuditOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditOutcome::Success => write!(f, "Success"),
            AuditOutcome::Failure => write!(f, "Failure"),
            AuditOutcome::Denied => write!(f, "Denied"),
        }
    }
}

impl std::str::FromStr for AuditOutcome {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Success" => Ok(AuditOutcome::Success),
            "Failure" => Ok(AuditOutcome::Failure),
            "Denied" => Ok(AuditOutcome::Denied),
            _ => Err(format!("Invalid audit outcome: {}", s)),
        }
    }
}

/// Audit event for security logging
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: AuditEventType,
    pub user_id: Option<Uuid>,
    pub session_id: Option<Uuid>,
    pub resource: String,
    pub action: String,
    pub outcome: AuditOutcome,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub additional_data: serde_json::Value,
    pub integrity_hash: String,
}

impl AuditEvent {
    /// Create a new audit event
    pub fn new(
        event_type: AuditEventType,
        user_id: Option<Uuid>,
        session_id: Option<Uuid>,
        resource: String,
        action: String,
        outcome: AuditOutcome,
        ip_address: Option<String>,
        user_agent: Option<String>,
        additional_data: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type,
            user_id,
            session_id,
            resource,
            action,
            outcome,
            ip_address,
            user_agent,
            additional_data,
            integrity_hash: String::new(), // Will be calculated later
        }
    }

    /// Calculate integrity hash for tamper detection
    pub fn calculate_integrity_hash(&self) -> String {
        let data = format!(
            "{}:{}:{}:{}:{}:{}:{}",
            self.id,
            self.timestamp.timestamp(),
            self.event_type,
            self.resource,
            self.action,
            self.outcome,
            self.additional_data
        );

        let hash = blake3::hash(data.as_bytes());
        hash.to_hex().to_string()
    }

    /// Set the integrity hash
    pub fn set_integrity_hash(&mut self) {
        self.integrity_hash = self.calculate_integrity_hash();
    }

    /// Verify integrity hash
    pub fn verify_integrity(&self) -> bool {
        let calculated_hash = self.calculate_integrity_hash();
        calculated_hash == self.integrity_hash
    }
}

/// Authentication context
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub external_id: String,
    pub email: String,
    pub display_name: String,
    pub roles: Vec<UserRole>,
    pub permissions: Vec<Permission>,
    pub session_id: Option<Uuid>,
    pub authenticated_at: DateTime<Utc>,
}

impl AuthenticatedUser {
    /// Create a new authenticated user
    pub fn new(
        user: User,
        roles: Vec<UserRole>,
        permissions: Vec<Permission>,
        session_id: Option<Uuid>,
    ) -> Self {
        Self {
            id: user.id,
            external_id: user.external_id,
            email: user.email,
            display_name: user.display_name,
            roles,
            permissions,
            session_id,
            authenticated_at: Utc::now(),
        }
    }

    /// Check if user has a specific role
    pub fn has_role(&self, role: &UserRole) -> bool {
        self.roles.contains(role)
    }

    /// Check if user has admin privileges
    pub fn is_admin(&self) -> bool {
        self.has_role(&UserRole::Admin)
    }

    /// Check if user has a specific permission
    pub fn has_permission(&self, resource: &str, action: &str, scope: Option<&str>) -> bool {
        self.permissions
            .iter()
            .any(|p| p.matches(resource, action, scope))
    }

    /// Get user's role names as strings
    pub fn role_names(&self) -> Vec<String> {
        self.roles.iter().map(|r| r.as_str().to_string()).collect()
    }
}

/// Authorization context for permission checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    pub user_id: Uuid,
    pub resource: String,
    pub action: String,
    pub scope: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub session_id: Option<Uuid>,
    pub additional_context: HashMap<String, serde_json::Value>,
}

impl AuthContext {
    /// Create a new authorization context
    pub fn new(user_id: Uuid, resource: String, action: String, scope: Option<String>) -> Self {
        Self {
            user_id,
            resource,
            action,
            scope,
            ip_address: None,
            user_agent: None,
            session_id: None,
            additional_context: HashMap::new(),
        }
    }

    /// Set IP address
    pub fn with_ip_address(mut self, ip_address: String) -> Self {
        self.ip_address = Some(ip_address);
        self
    }

    /// Set user agent
    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    /// Set session ID
    pub fn with_session_id(mut self, session_id: Uuid) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Add additional context
    pub fn with_context(mut self, key: String, value: serde_json::Value) -> Self {
        self.additional_context.insert(key, value);
        self
    }
}

/// Rate limiting information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RateLimitInfo {
    pub identifier: String,
    pub identifier_type: RateLimitIdentifierType,
    pub endpoint: String,
    pub request_count: u32,
    pub window_start: DateTime<Utc>,
    pub window_size: u32, // in seconds
    pub limit: u32,
}

impl RateLimitInfo {
    /// Check if rate limit is exceeded
    pub fn is_exceeded(&self) -> bool {
        self.request_count > self.limit
    }

    /// Check if the current window is still valid
    pub fn is_window_valid(&self) -> bool {
        let now = Utc::now();
        let window_end = self.window_start + chrono::Duration::seconds(self.window_size as i64);
        now < window_end
    }

    /// Get remaining requests in current window
    pub fn remaining_requests(&self) -> u32 {
        if self.request_count >= self.limit {
            0
        } else {
            self.limit - self.request_count
        }
    }
}

/// Rate limit identifier types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RateLimitIdentifierType {
    IpAddress,
    UserId,
    ApiKey,
}

impl std::fmt::Display for RateLimitIdentifierType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RateLimitIdentifierType::IpAddress => write!(f, "ip"),
            RateLimitIdentifierType::UserId => write!(f, "user"),
            RateLimitIdentifierType::ApiKey => write!(f, "api_key"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_user_creation() {
        let user = User::new(
            "external123".to_string(),
            "user@example.com".to_string(),
            "John Doe".to_string(),
        );

        assert!(user.is_active());
        assert_eq!(user.external_id, "external123");
        assert_eq!(user.email, "user@example.com");
        assert_eq!(user.display_name, "John Doe");
    }

    #[test]
    fn test_user_role_string_conversion() {
        let admin = UserRole::Admin;
        assert_eq!(admin.as_str(), "Admin");
        assert_eq!(admin.to_string(), "Admin");

        let parsed: UserRole = "Admin".parse().unwrap();
        assert_eq!(parsed, UserRole::Admin);
    }

    #[test]
    fn test_permission_matching() {
        let permission = Permission::new(
            "projects".to_string(),
            "read".to_string(),
            Some("own".to_string()),
        );

        assert!(permission.matches("projects", "read", Some("own")));
        assert!(!permission.matches("projects", "write", Some("own")));
        assert!(!permission.matches("projects", "read", Some("all")));
    }

    #[test]
    fn test_session_validity() {
        let future_time = Utc::now() + chrono::Duration::hours(1);
        let mut session = Session::new(
            Uuid::new_v4(),
            "token123".to_string(),
            future_time,
            Some("127.0.0.1".to_string()),
            Some("Mozilla/5.0".to_string()),
        );

        assert!(session.is_valid());
        assert!(!session.is_expired());

        session.deactivate();
        assert!(!session.is_valid());
    }

    #[test]
    fn test_api_key_validity() {
        let mut api_key = ApiKey::new(
            None,
            "Test Key".to_string(),
            "hash123".to_string(),
            "prefix".to_string(),
            None,
            None,
        );

        assert!(api_key.is_valid());
        assert!(!api_key.is_expired());

        api_key.deactivate();
        assert!(!api_key.is_valid());
    }

    #[test]
    fn test_audit_event_integrity() {
        let mut event = AuditEvent::new(
            AuditEventType::Authentication,
            Some(Uuid::new_v4()),
            None,
            "users".to_string(),
            "login".to_string(),
            AuditOutcome::Success,
            Some("127.0.0.1".to_string()),
            Some("Mozilla/5.0".to_string()),
            json!({"method": "password"}),
        );

        event.set_integrity_hash();
        assert!(event.verify_integrity());

        // Modify event to break integrity
        event.outcome = AuditOutcome::Failure;
        assert!(!event.verify_integrity());
    }

    #[test]
    fn test_authenticated_user_permissions() {
        let user = User::new(
            "external123".to_string(),
            "user@example.com".to_string(),
            "John Doe".to_string(),
        );

        let permissions = vec![
            Permission::new(
                "projects".to_string(),
                "read".to_string(),
                Some("own".to_string()),
            ),
            Permission::new(
                "projects".to_string(),
                "write".to_string(),
                Some("own".to_string()),
            ),
        ];

        let auth_user = AuthenticatedUser::new(user, vec![UserRole::Developer], permissions, None);

        assert!(auth_user.has_role(&UserRole::Developer));
        assert!(!auth_user.has_role(&UserRole::Admin));
        assert!(!auth_user.is_admin());

        assert!(auth_user.has_permission("projects", "read", Some("own")));
        assert!(auth_user.has_permission("projects", "write", Some("own")));
        assert!(!auth_user.has_permission("projects", "delete", Some("own")));
    }

    #[test]
    fn test_rate_limit_info() {
        let rate_limit = RateLimitInfo {
            identifier: "127.0.0.1".to_string(),
            identifier_type: RateLimitIdentifierType::IpAddress,
            endpoint: "/api/analysis".to_string(),
            request_count: 5,
            window_start: Utc::now() - chrono::Duration::minutes(30),
            window_size: 3600,
            limit: 100,
        };

        assert!(!rate_limit.is_exceeded());
        assert!(rate_limit.is_window_valid());
        assert_eq!(rate_limit.remaining_requests(), 95);
    }
}
