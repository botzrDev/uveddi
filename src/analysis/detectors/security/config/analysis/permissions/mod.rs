//! Permission and access control analysis for configuration files
//!
//! This module detects overly permissive access controls, weak authentication
//! requirements, and inappropriate privilege escalations in configuration files.

pub mod access_control;
pub mod analyzer;
pub mod file_permissions;
pub mod privilege_escalation;
pub mod user_group;

// Re-export main analyzer
pub use access_control::AccessControlChecker;
pub use analyzer::PermissionAnalyzer;
pub use file_permissions::FilePermissionChecker;
pub use privilege_escalation::PrivilegeEscalationChecker;
pub use user_group::UserGroupChecker;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::detectors::security::config::ConfigSecurityConfig;

    #[test]
    fn test_permission_analyzer_creation() {
        let config = ConfigSecurityConfig::default();
        let analyzer = PermissionAnalyzer::new(&config).unwrap();
        // Basic creation test
    }
}
