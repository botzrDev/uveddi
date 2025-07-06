//! Community Database Models
//!
//! Data structures for community member management, separate from the main analysis database.
//! These models are optimized for web access, admin management, and marketing analytics.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Member roles with hierarchical permissions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MemberRole {
    /// Regular community member - basic access
    Member,
    /// Developer - additional access to developer resources
    Developer,
    /// Administrator - full access to all features and admin panel
    Admin,
}

impl MemberRole {
    /// Check if this role has admin privileges
    pub fn is_admin(&self) -> bool {
        matches!(self, MemberRole::Admin)
    }

    /// Check if this role has developer privileges (Developer or Admin)
    pub fn is_developer_or_higher(&self) -> bool {
        matches!(self, MemberRole::Developer | MemberRole::Admin)
    }

    /// Get role as string for database storage
    pub fn as_str(&self) -> &'static str {
        match self {
            MemberRole::Member => "member",
            MemberRole::Developer => "developer", 
            MemberRole::Admin => "admin",
        }
    }

    /// Parse role from string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "admin" => MemberRole::Admin,
            "developer" => MemberRole::Developer,
            _ => MemberRole::Member,
        }
    }
}

/// Core community member entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityMember {
    /// Unique member identifier (UUID)
    pub id: String,
    /// Email address (unique, used for login)
    pub email: String,
    /// Display name
    pub name: String,
    /// Member role and permissions
    pub role: MemberRole,
    /// Account creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last activity timestamp
    pub last_active: Option<DateTime<Utc>>,
    /// Account status
    pub is_active: bool,
    /// Profile information for marketing
    pub profile: MemberProfile,
    /// Email verification status
    pub email_verified: bool,
    /// Optional avatar URL
    pub avatar_url: Option<String>,
}

/// Member profile information for marketing analytics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemberProfile {
    /// Company/Organization (optional)
    pub company: Option<String>,
    /// Job title (optional)
    pub job_title: Option<String>,
    /// Programming languages of interest
    pub languages: Vec<String>,
    /// Experience level (beginner, intermediate, advanced, expert)
    pub experience_level: Option<String>,
    /// Primary use case for Uveddi
    pub use_case: Option<String>,
    /// How they heard about Uveddi
    pub referral_source: Option<String>,
    /// Newsletter subscription
    pub newsletter_subscribed: bool,
    /// Marketing consent
    pub marketing_consent: bool,
    /// Additional custom fields as JSON
    pub custom_fields: HashMap<String, String>,
}

/// Activity types for tracking member engagement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivityType {
    /// Account related activities
    Registration,
    Login,
    Logout,
    ProfileUpdate,
    EmailVerified,
    PasswordReset,
    
    /// Feature usage
    DashboardView,
    AnalysisDownload,
    ReportGenerated,
    ApiCall,
    PluginInstall,
    
    /// Community engagement
    ForumPost,
    CommentPosted,
    DocumentationView,
    TutorialComplete,
    
    /// Admin activities
    AdminAction,
    RoleChanged,
    AccountDeactivated,
    
    /// Custom activity (with description)
    Custom(String),
}

impl ActivityType {
    /// Get activity type as string for database storage
    pub fn as_str(&self) -> String {
        match self {
            ActivityType::Registration => "registration".to_string(),
            ActivityType::Login => "login".to_string(),
            ActivityType::Logout => "logout".to_string(),
            ActivityType::ProfileUpdate => "profile_update".to_string(),
            ActivityType::EmailVerified => "email_verified".to_string(),
            ActivityType::PasswordReset => "password_reset".to_string(),
            ActivityType::DashboardView => "dashboard_view".to_string(),
            ActivityType::AnalysisDownload => "analysis_download".to_string(),
            ActivityType::ReportGenerated => "report_generated".to_string(),
            ActivityType::ApiCall => "api_call".to_string(),
            ActivityType::PluginInstall => "plugin_install".to_string(),
            ActivityType::ForumPost => "forum_post".to_string(),
            ActivityType::CommentPosted => "comment_posted".to_string(),
            ActivityType::DocumentationView => "documentation_view".to_string(),
            ActivityType::TutorialComplete => "tutorial_complete".to_string(),
            ActivityType::AdminAction => "admin_action".to_string(),
            ActivityType::RoleChanged => "role_changed".to_string(),
            ActivityType::AccountDeactivated => "account_deactivated".to_string(),
            ActivityType::Custom(desc) => format!("custom:{}", desc),
        }
    }

    /// Parse activity type from string
    pub fn from_str(s: &str) -> Self {
        if let Some(custom_desc) = s.strip_prefix("custom:") {
            return ActivityType::Custom(custom_desc.to_string());
        }
        
        match s {
            "registration" => ActivityType::Registration,
            "login" => ActivityType::Login,
            "logout" => ActivityType::Logout,
            "profile_update" => ActivityType::ProfileUpdate,
            "email_verified" => ActivityType::EmailVerified,
            "password_reset" => ActivityType::PasswordReset,
            "dashboard_view" => ActivityType::DashboardView,
            "analysis_download" => ActivityType::AnalysisDownload,
            "report_generated" => ActivityType::ReportGenerated,
            "api_call" => ActivityType::ApiCall,
            "plugin_install" => ActivityType::PluginInstall,
            "forum_post" => ActivityType::ForumPost,
            "comment_posted" => ActivityType::CommentPosted,
            "documentation_view" => ActivityType::DocumentationView,
            "tutorial_complete" => ActivityType::TutorialComplete,
            "admin_action" => ActivityType::AdminAction,
            "role_changed" => ActivityType::RoleChanged,
            "account_deactivated" => ActivityType::AccountDeactivated,
            _ => ActivityType::Custom(s.to_string()),
        }
    }
}

/// Member activity tracking for engagement analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberActivity {
    /// Unique activity identifier
    pub id: Option<i64>,
    /// Member who performed the activity
    pub member_id: String,
    /// Type of activity performed
    pub activity_type: ActivityType,
    /// Activity timestamp
    pub timestamp: DateTime<Utc>,
    /// Optional activity description or metadata
    pub description: Option<String>,
    /// IP address (for security tracking)
    pub ip_address: Option<String>,
    /// User agent (for analytics)
    pub user_agent: Option<String>,
    /// Additional metadata as JSON
    pub metadata: HashMap<String, String>,
}

/// Member session tracking for authentication and security
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberSession {
    /// Unique session identifier
    pub id: String,
    /// Member who owns this session
    pub member_id: String,
    /// Session creation timestamp
    pub created_at: DateTime<Utc>,
    /// Session expiration timestamp
    pub expires_at: DateTime<Utc>,
    /// Last activity in this session
    pub last_activity: DateTime<Utc>,
    /// IP address for this session
    pub ip_address: Option<String>,
    /// User agent for this session
    pub user_agent: Option<String>,
    /// Whether session is currently active
    pub is_active: bool,
}
