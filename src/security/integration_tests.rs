//! Integration tests for the security module
//!
//! These tests verify the complete security system integration including
//! authentication, authorization, audit logging, and rate limiting.

#[cfg(test)]
mod tests {
    use crate::security::{
        authentication::{AuthenticationService, AuthenticationConfig},
        authorization::AuthorizationEngine,
        audit::{AuditLogger, InMemoryAuditStore},
        config::SecurityConfig,
        errors::SecurityError,
        models::{User, UserRole, AuthContext, AuditEventType, AuditOutcome},
        rate_limiting::RateLimiter,
        config::RateLimitingConfig,
        secrets::{InMemorySecretStore, SecretStore},
    };
    use std::sync::Arc;
    use std::collections::HashMap;
    use uuid::Uuid;
    use serde_json::json;

    /// Test complete authentication flow
    #[tokio::test]
    async fn test_complete_authentication_flow() {
        let config = AuthenticationConfig::default();
        let secret_store = Arc::new(InMemorySecretStore::new());
        let auth_service = AuthenticationService::new(config, secret_store).await.unwrap();

        // Test JWT generation and validation
        let user = User::new(
            "test_user".to_string(),
            "test@example.com".to_string(),
            "Test User".to_string(),
        );

        let auth_user = crate::security::models::AuthenticatedUser::new(
            user,
            vec![UserRole::Developer],
            vec![],
            None,
        );

        // Generate JWT
        let jwt = auth_service.generate_jwt(&auth_user).await.unwrap();
        assert!(!jwt.is_empty());

        // Test API key generation and validation
        let (api_key, _) = auth_service.generate_api_key(
            Some(auth_user.id),
            "Test API Key".to_string(),
            None,
        ).await.unwrap();
        
        assert!(api_key.starts_with("uvd_"));
        assert_eq!(api_key.len(), 45);
    }

    /// Test complete authorization flow
    #[tokio::test]
    async fn test_complete_authorization_flow() {
        let mut engine = AuthorizationEngine::new().await.unwrap();
        let user_id = Uuid::new_v4();

        // Cache user roles
        let roles = vec![UserRole::Developer];
        engine.cache_user_roles(user_id, roles.clone()).await.unwrap();

        // Test permission check
        let context = AuthContext::new(
            user_id,
            "projects".to_string(),
            "read".to_string(),
            Some("own".to_string()),
        );

        let allowed = engine.check_permission(&user_id, "projects", "read", &context).await.unwrap();
        // This should be true since:
        // 1. User has Developer role cached
        // 2. Developer role has permission for "projects:own" with "read" action  
        // 3. Context has scope "own" so it becomes "projects:own"
        assert!(allowed); // Expected to be true with proper RBAC implementation

        // Test role-based permission check
        let has_permission = engine.check_role_permission(
            &UserRole::Developer,
            "projects",
            "read",
            &context,
        ).await.unwrap();
        assert!(has_permission); // Should be true based on default policies
    }

    /// Test audit logging integration
    #[tokio::test]
    async fn test_audit_logging_integration() {
        let store = Arc::new(InMemoryAuditStore::new());
        let logger = AuditLogger::new(store.clone());

        // Log authentication event
        logger.log(
            AuditEventType::Authentication,
            Some(Uuid::new_v4()),
            None,
            "users".to_string(),
            "login".to_string(),
            AuditOutcome::Success,
            Some("127.0.0.1".to_string()),
            Some("Mozilla/5.0".to_string()),
            json!({"method": "password"}),
        ).await.unwrap();

        // Log authorization event
        logger.log(
            AuditEventType::Authorization,
            Some(Uuid::new_v4()),
            None,
            "projects".to_string(),
            "read".to_string(),
            AuditOutcome::Success,
            Some("127.0.0.1".to_string()),
            None,
            json!({"resource_id": "123"}),
        ).await.unwrap();

        // Verify events were logged
        let criteria = crate::security::audit::AuditQueryCriteria::default();
        let events = logger.get_events(criteria).await.unwrap();
        assert_eq!(events.len(), 2);

        // Test statistics
        let start_time = chrono::Utc::now() - chrono::Duration::hours(1);
        let end_time = chrono::Utc::now();
        let stats = logger.get_statistics(start_time, end_time).await.unwrap();
        assert_eq!(stats.total_events, 2);
    }

    /// Test rate limiting integration
    #[tokio::test]
    async fn test_rate_limiting_integration() {
        let config = RateLimitingConfig::default();
        let limiter = RateLimiter::new_in_memory(config);

        // Test different identifier types
        let ip_identifier = "ip:192.168.1.1";
        let user_identifier = "user:123";
        let api_key_identifier = "api_key:abc123";

        // Test IP-based rate limiting
        for _ in 0..60 {
            let allowed = limiter.check_rate_limit(ip_identifier, "/api/test").await.unwrap();
            assert!(allowed);
        }

        // Should be rate limited now
        let allowed = limiter.check_rate_limit(ip_identifier, "/api/test").await.unwrap();
        assert!(!allowed);

        // User identifier should still work
        let allowed = limiter.check_rate_limit(user_identifier, "/api/test").await.unwrap();
        assert!(allowed);

        // API key identifier should still work
        let allowed = limiter.check_rate_limit(api_key_identifier, "/api/test").await.unwrap();
        assert!(allowed);

        // Test endpoint-specific limits
        let allowed = limiter.check_rate_limit(ip_identifier, "/api/auth/login").await.unwrap();
        assert!(allowed); // Should work because it's a different endpoint
    }

    /// Test complete security configuration
    #[tokio::test]
    async fn test_security_configuration() {
        let config = SecurityConfig::default();
        
        // Test configuration validation
        assert!(config.validate().is_ok());
        
        // Test invalid configuration
        let mut invalid_config = config.clone();
        invalid_config.authentication.jwt_secret = "short".to_string();
        assert!(invalid_config.validate().is_err());
        
        // Test serialization
        let json_config = config.to_json().unwrap();
        assert!(json_config.contains("authentication"));
        
        let toml_config = config.to_toml().unwrap();
        assert!(toml_config.contains("[authentication]"));
    }

    /// Test error handling and propagation
    #[tokio::test]
    async fn test_security_error_handling() {
        // Test authentication errors
        let auth_error = SecurityError::authentication_failed("Invalid credentials");
        assert!(auth_error.is_authentication_error());
        assert!(auth_error.should_audit_log());
        
        // Test authorization errors
        let authz_error = SecurityError::authorization_denied();
        assert!(authz_error.is_authorization_error());
        assert!(authz_error.should_audit_log());
        
        // Test rate limiting errors
        let rate_limit_error = SecurityError::rate_limit_exceeded("user123", 100, 50);
        assert!(rate_limit_error.is_rate_limit_error());
        assert!(rate_limit_error.should_audit_log());
        
        // Test error severity
        assert_eq!(auth_error.severity(), crate::security::errors::SecurityErrorSeverity::High);
        assert_eq!(rate_limit_error.severity(), crate::security::errors::SecurityErrorSeverity::Medium);
    }

    /// Test session management
    #[tokio::test]
    async fn test_session_management() {
        let config = AuthenticationConfig::default();
        let secret_store = Arc::new(InMemorySecretStore::new());
        let auth_service = AuthenticationService::new(config, secret_store).await.unwrap();

        let user = User::new(
            "test_user".to_string(),
            "test@example.com".to_string(),
            "Test User".to_string(),
        );

        // Create session
        let session = auth_service.create_session(&user).await.unwrap();
        assert!(session.is_valid());

        // Validate session
        let validated_session = auth_service.validate_session(&session.session_token).await.unwrap();
        assert_eq!(validated_session.id, session.id);

        // Revoke session
        auth_service.revoke_session(&session.session_token).await.unwrap();

        // Validation should fail
        let result = auth_service.validate_session(&session.session_token).await;
        assert!(result.is_err());
    }

    /// Test permission inheritance and role hierarchy
    #[tokio::test]
    async fn test_permission_inheritance() {
        let mut engine = AuthorizationEngine::new().await.unwrap();

        // Test admin role has all permissions
        let admin_context = AuthContext::new(
            Uuid::new_v4(),
            "projects".to_string(),
            "delete".to_string(),
            Some("all".to_string()),
        );

        let admin_allowed = engine.check_role_permission(
            &UserRole::Admin,
            "projects",
            "delete",
            &admin_context,
        ).await.unwrap();
        assert!(admin_allowed);

        // Test developer role has limited permissions
        let dev_context = AuthContext::new(
            Uuid::new_v4(),
            "projects".to_string(),
            "read".to_string(),
            Some("own".to_string()),
        );

        let dev_allowed = engine.check_role_permission(
            &UserRole::Developer,
            "projects",
            "read",
            &dev_context,
        ).await.unwrap();
        assert!(dev_allowed);

        // Test developer cannot delete
        let dev_delete_allowed = engine.check_role_permission(
            &UserRole::Developer,
            "projects",
            "delete",
            &dev_context,
        ).await.unwrap();
        assert!(!dev_delete_allowed);
    }

    /// Test secret management integration
    #[tokio::test]
    async fn test_secret_management() {
        let secret_store = InMemorySecretStore::new();

        // Test secret operations
        secret_store.set_secret("test_key", "test_value").await.unwrap();
        
        let value = secret_store.get_secret("test_key").await.unwrap();
        assert_eq!(value, "test_value");
        
        let exists = secret_store.secret_exists("test_key").await.unwrap();
        assert!(exists);
        
        let keys = secret_store.list_secret_keys().await.unwrap();
        assert!(keys.contains(&"test_key".to_string()));
        
        secret_store.delete_secret("test_key").await.unwrap();
        let exists = secret_store.secret_exists("test_key").await.unwrap();
        assert!(!exists);
    }

    /// Test concurrent access and thread safety
    #[tokio::test]
    async fn test_concurrent_access() {
        let config = RateLimitingConfig::default();
        let limiter = Arc::new(RateLimiter::new_in_memory(config));

        // Spawn multiple concurrent tasks
        let mut handles = Vec::new();
        for i in 0..10 {
            let limiter = limiter.clone();
            let handle = tokio::spawn(async move {
                let identifier = format!("user:{}", i);
                let allowed = limiter.check_rate_limit(&identifier, "/api/test").await.unwrap();
                allowed
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        let results = futures::future::join_all(handles).await;
        
        // All should be allowed since they're different identifiers
        for result in results {
            assert!(result.unwrap());
        }
    }

    /// Test compliance and audit requirements
    #[tokio::test]
    async fn test_compliance_requirements() {
        let store = Arc::new(InMemoryAuditStore::new());
        let logger = AuditLogger::new(store.clone());

        // Test that all security events are logged
        let user_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();

        // Authentication event
        logger.log(
            AuditEventType::Authentication,
            Some(user_id),
            Some(session_id),
            "users".to_string(),
            "login".to_string(),
            AuditOutcome::Success,
            Some("127.0.0.1".to_string()),
            Some("Mozilla/5.0".to_string()),
            json!({"method": "oauth", "provider": "google"}),
        ).await.unwrap();

        // Authorization event
        logger.log(
            AuditEventType::Authorization,
            Some(user_id),
            Some(session_id),
            "sensitive_data".to_string(),
            "access".to_string(),
            AuditOutcome::Success,
            Some("127.0.0.1".to_string()),
            None,
            json!({"resource_id": "confidential_report_123"}),
        ).await.unwrap();

        // Configuration change event
        logger.log(
            AuditEventType::ConfigurationChange,
            Some(user_id),
            Some(session_id),
            "system".to_string(),
            "update_security_policy".to_string(),
            AuditOutcome::Success,
            Some("127.0.0.1".to_string()),
            None,
            json!({"policy": "password_policy", "changes": ["min_length: 8 -> 12"]}),
        ).await.unwrap();

        // Security violation event
        logger.log(
            AuditEventType::SecurityViolation,
            None,
            None,
            "api".to_string(),
            "brute_force_attempt".to_string(),
            AuditOutcome::Denied,
            Some("192.168.1.100".to_string()),
            None,
            json!({"attempts": 50, "blocked": true}),
        ).await.unwrap();

        // Verify all events are logged
        let criteria = crate::security::audit::AuditQueryCriteria::default();
        let events = logger.get_events(criteria).await.unwrap();
        assert_eq!(events.len(), 4);

        // Test integrity verification
        let event_ids: Vec<Uuid> = events.iter().map(|e| e.id).collect();
        let integrity_result = logger.verify_integrity(event_ids).await.unwrap();
        assert!(integrity_result.integrity_ok);
        assert_eq!(integrity_result.verified_events, 4);
    }
}