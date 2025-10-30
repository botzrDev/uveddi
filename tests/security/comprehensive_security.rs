//! Comprehensive security testing suite
//! 
//! Tests include:
//! - RBAC enforcement validation
//! - Authentication mechanism testing
//! - Data protection and encryption
//! - Input validation and sanitization
//! - Security headers and CSRF protection
//! - JSON deserialization security (UV-275)

mod deserialization_security_tests;

#[cfg(test)]
mod security_tests {
    use std::collections::HashMap;
    use std::time::Duration;
    use tokio::time::timeout;
    use serde_json::json;
    
    use uveddi::security::{
        rbac::{RBACManager, Role, Permission, Resource},
        auth::{AuthenticationManager, AuthenticationConfig, Credentials},
        encryption::{EncryptionManager, EncryptionConfig},
        validation::{InputValidator, ValidationRule},
    };
    use uveddi::database::DatabaseManager;
    use uveddi::config::SecurityConfig;

    /// Test RBAC enforcement with various permission scenarios
    #[tokio::test]
    async fn test_rbac_enforcement() {
        let rbac_manager = create_test_rbac_manager().await;
        
        // Create test roles and permissions
        let admin_role = Role {
            id: "admin".to_string(),
            name: "Administrator".to_string(),
            permissions: vec![
                Permission::new("analysis", "read"),
                Permission::new("analysis", "write"),
                Permission::new("analysis", "delete"),
                Permission::new("config", "read"),
                Permission::new("config", "write"),
                Permission::new("users", "manage"),
            ],
        };
        
        let analyst_role = Role {
            id: "analyst".to_string(),
            name: "Analyst".to_string(),
            permissions: vec![
                Permission::new("analysis", "read"),
                Permission::new("analysis", "write"),
                Permission::new("config", "read"),
            ],
        };
        
        let viewer_role = Role {
            id: "viewer".to_string(),
            name: "Viewer".to_string(),
            permissions: vec![
                Permission::new("analysis", "read"),
                Permission::new("config", "read"),
            ],
        };
        
        // Test role creation
        rbac_manager.create_role(admin_role.clone()).await.unwrap();
        rbac_manager.create_role(analyst_role.clone()).await.unwrap();
        rbac_manager.create_role(viewer_role.clone()).await.unwrap();
        
        // Test user role assignment
        rbac_manager.assign_role_to_user("admin_user", "admin").await.unwrap();
        rbac_manager.assign_role_to_user("analyst_user", "analyst").await.unwrap();
        rbac_manager.assign_role_to_user("viewer_user", "viewer").await.unwrap();
        
        // Test permission checks - Admin user
        assert!(rbac_manager.has_permission("admin_user", &Resource::Analysis, "read").await.unwrap());
        assert!(rbac_manager.has_permission("admin_user", &Resource::Analysis, "write").await.unwrap());
        assert!(rbac_manager.has_permission("admin_user", &Resource::Analysis, "delete").await.unwrap());
        assert!(rbac_manager.has_permission("admin_user", &Resource::Users, "manage").await.unwrap());
        
        // Test permission checks - Analyst user
        assert!(rbac_manager.has_permission("analyst_user", &Resource::Analysis, "read").await.unwrap());
        assert!(rbac_manager.has_permission("analyst_user", &Resource::Analysis, "write").await.unwrap());
        assert!(!rbac_manager.has_permission("analyst_user", &Resource::Analysis, "delete").await.unwrap());
        assert!(!rbac_manager.has_permission("analyst_user", &Resource::Users, "manage").await.unwrap());
        
        // Test permission checks - Viewer user
        assert!(rbac_manager.has_permission("viewer_user", &Resource::Analysis, "read").await.unwrap());
        assert!(!rbac_manager.has_permission("viewer_user", &Resource::Analysis, "write").await.unwrap());
        assert!(!rbac_manager.has_permission("viewer_user", &Resource::Analysis, "delete").await.unwrap());
        assert!(!rbac_manager.has_permission("viewer_user", &Resource::Users, "manage").await.unwrap());
        
        // Test permission denial for non-existent user
        assert!(!rbac_manager.has_permission("nonexistent_user", &Resource::Analysis, "read").await.unwrap());
        
        println!("✅ RBAC enforcement tests passed");
    }

    /// Test authentication mechanisms and security
    #[tokio::test]
    async fn test_authentication_mechanisms() {
        // Use secret store for JWT secret instead of hardcoded value
        let secret_store = crate::security::secrets::MockSecretStore::new();
        let jwt_secret = secret_store.get_secret("jwt_secret").await
            .unwrap_or_else(|_| "test_secret_key_for_testing_only_fallback".to_string());
        
        let auth_config = AuthenticationConfig {
            jwt_secret,
            jwt_expiry: Duration::from_secs(3600),
            password_min_length: 12,
            require_special_chars: true,
            require_numbers: true,
            require_uppercase: true,
            max_login_attempts: 3,
            lockout_duration: Duration::from_secs(300),
            enable_two_factor: true,
        };
        
        let auth_manager = AuthenticationManager::new(auth_config).await.unwrap();
        
        // Test user registration with strong password
        let strong_password = "StrongPassword123!";
        let registration_result = auth_manager.register_user(
            "test_user",
            "test@example.com",
            strong_password,
        ).await;
        assert!(registration_result.is_ok());
        
        // Test user registration with weak password (should fail)
        let weak_password = "weak";
        let weak_registration_result = auth_manager.register_user(
            "weak_user",
            "weak@example.com",
            weak_password,
        ).await;
        assert!(weak_registration_result.is_err());
        
        // Test successful login
        let credentials = Credentials {
            username: "test_user".to_string(),
            password: strong_password.to_string(),
            two_factor_code: None,
        };
        
        let login_result = auth_manager.authenticate(credentials).await;
        assert!(login_result.is_ok());
        
        let token = login_result.unwrap();
        assert!(!token.is_empty());
        
        // Test token validation
        let validation_result = auth_manager.validate_token(&token).await;
        assert!(validation_result.is_ok());
        
        let user_info = validation_result.unwrap();
        assert_eq!(user_info.username, "test_user");
        
        // Test failed login with wrong password
        let wrong_credentials = Credentials {
            username: "test_user".to_string(),
            password: "wrong_password".to_string(),
            two_factor_code: None,
        };
        
        let failed_login = auth_manager.authenticate(wrong_credentials).await;
        assert!(failed_login.is_err());
        
        // Test brute force protection
        for _ in 0..3 {
            let _ = auth_manager.authenticate(Credentials {
                username: "test_user".to_string(),
                password: "wrong_password".to_string(),
                two_factor_code: None,
            }).await;
        }
        
        // Account should be locked after max attempts
        let locked_attempt = auth_manager.authenticate(Credentials {
            username: "test_user".to_string(),
            password: strong_password.to_string(),
            two_factor_code: None,
        }).await;
        assert!(locked_attempt.is_err());
        
        // Test JWT token expiry (simplified test)
        let expired_token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ0ZXN0X3VzZXIiLCJleHAiOjE2MzAwMDAwMDB9.invalid";
        let expired_validation = auth_manager.validate_token(expired_token).await;
        assert!(expired_validation.is_err());
        
        println!("✅ Authentication mechanism tests passed");
    }

    /// Test data protection and encryption
    #[tokio::test]
    async fn test_data_protection_encryption() {
        let encryption_config = EncryptionConfig {
            algorithm: "AES-256-GCM".to_string(),
            key_derivation: "PBKDF2".to_string(),
            key_size: 256,
            salt_size: 32,
            iteration_count: 100000,
        };
        
        let encryption_manager = EncryptionManager::new(encryption_config).await.unwrap();
        
        // Test data encryption and decryption
        let sensitive_data = "This is sensitive user data that must be encrypted";
        let encrypted_data = encryption_manager.encrypt(sensitive_data.as_bytes()).await.unwrap();
        
        // Ensure encrypted data is different from original
        assert_ne!(encrypted_data, sensitive_data.as_bytes());
        
        // Test decryption
        let decrypted_data = encryption_manager.decrypt(&encrypted_data).await.unwrap();
        let decrypted_string = String::from_utf8(decrypted_data).unwrap();
        assert_eq!(decrypted_string, sensitive_data);
        
        // Test encryption of personal identifiable information (PII)
        let pii_data = json!({
            "email": "user@example.com",
            "phone": "+1-555-123-4567",
            "ssn": "123-45-6789",
            "address": "123 Main St, Anytown, USA"
        });
        
        let pii_string = pii_data.to_string();
        let encrypted_pii = encryption_manager.encrypt(pii_string.as_bytes()).await.unwrap();
        let decrypted_pii = encryption_manager.decrypt(&encrypted_pii).await.unwrap();
        let decrypted_pii_string = String::from_utf8(decrypted_pii).unwrap();
        
        assert_eq!(decrypted_pii_string, pii_string);
        
        // Test key rotation
        encryption_manager.rotate_keys().await.unwrap();
        
        // Old encrypted data should still be decryptable with key versioning
        let post_rotation_decrypted = encryption_manager.decrypt(&encrypted_data).await.unwrap();
        let post_rotation_string = String::from_utf8(post_rotation_decrypted).unwrap();
        assert_eq!(post_rotation_string, sensitive_data);
        
        // Test data at rest encryption for database
        let db_manager = DatabaseManager::new_with_encryption(&encryption_manager).await.unwrap();
        
        // Test storing and retrieving encrypted data
        let user_data = json!({
            "user_id": "user123",
            "profile": {
                "name": "Test User",
                "email": "test@example.com",
                "preferences": {
                    "theme": "dark",
                    "notifications": true
                }
            }
        });
        
        db_manager.store_encrypted_user_data("user123", &user_data).await.unwrap();
        let retrieved_data = db_manager.retrieve_encrypted_user_data("user123").await.unwrap();
        
        assert_eq!(retrieved_data, user_data);
        
        println!("✅ Data protection and encryption tests passed");
    }

    /// Test input validation and sanitization
    #[tokio::test]
    async fn test_input_validation_sanitization() {
        let validator = InputValidator::new();
        
        // Test SQL injection prevention
        let sql_injection_attempts = vec![
            "'; DROP TABLE users; --",
            "1' OR '1'='1",
            "admin'/*",
            "'; INSERT INTO users VALUES ('hacker', 'password'); --",
        ];
        
        for attempt in sql_injection_attempts {
            let validation_result = validator.validate_input(
                attempt,
                &ValidationRule::NoSqlInjection
            ).await;
            assert!(validation_result.is_err(), "SQL injection attempt should be blocked: {}", attempt);
        }
        
        // Test XSS prevention
        let xss_attempts = vec![
            "<script>alert('XSS')</script>",
            "javascript:alert('XSS')",
            "<img src=x onerror=alert('XSS')>",
            "<svg onload=alert('XSS')>",
            "';alert(String.fromCharCode(88,83,83))//';alert(String.fromCharCode(88,83,83))//\"",
        ];
        
        for attempt in xss_attempts {
            let validation_result = validator.validate_input(
                attempt,
                &ValidationRule::NoXSS
            ).await;
            assert!(validation_result.is_err(), "XSS attempt should be blocked: {}", attempt);
        }
        
        // Test command injection prevention
        let command_injection_attempts = vec![
            "; rm -rf /",
            "| cat /etc/passwd",
            "&& rm important_file.txt",
            "`cat /etc/shadow`",
            "$(rm -rf /)",
        ];
        
        for attempt in command_injection_attempts {
            let validation_result = validator.validate_input(
                attempt,
                &ValidationRule::NoCommandInjection
            ).await;
            assert!(validation_result.is_err(), "Command injection attempt should be blocked: {}", attempt);
        }
        
        // Test path traversal prevention
        let path_traversal_attempts = vec![
            "../../../etc/passwd",
            "..\\..\\..\\windows\\system32\\config\\sam",
            "/etc/passwd",
            "....//....//....//etc/passwd",
            "%2e%2e%2f%2e%2e%2f%2e%2e%2fetc%2fpasswd",
        ];
        
        for attempt in path_traversal_attempts {
            let validation_result = validator.validate_input(
                attempt,
                &ValidationRule::NoPathTraversal
            ).await;
            assert!(validation_result.is_err(), "Path traversal attempt should be blocked: {}", attempt);
        }
        
        // Test valid inputs
        let valid_inputs = vec![
            "normal_user_input",
            "user@example.com",
            "This is a normal text message",
            "ValidFileName.txt",
            "123456",
        ];
        
        for input in valid_inputs {
            let validation_result = validator.validate_input(
                input,
                &ValidationRule::General
            ).await;
            assert!(validation_result.is_ok(), "Valid input should pass validation: {}", input);
        }
        
        // Test input sanitization
        let unsanitized_input = "<script>alert('test')</script>Hello World!";
        let sanitized = validator.sanitize_input(unsanitized_input).await.unwrap();
        assert!(!sanitized.contains("<script>"));
        assert!(sanitized.contains("Hello World!"));
        
        // Test email validation
        let valid_emails = vec![
            "user@example.com",
            "test.email+tag@domain.co.uk",
            "user123@subdomain.example.org",
        ];
        
        let invalid_emails = vec![
            "invalid.email",
            "@example.com",
            "user@",
            "user..name@example.com",
            "user@.com",
        ];
        
        for email in valid_emails {
            assert!(validator.validate_email(email).await.unwrap());
        }
        
        for email in invalid_emails {
            assert!(!validator.validate_email(email).await.unwrap());
        }
        
        println!("✅ Input validation and sanitization tests passed");
    }

    /// Test security headers and CSRF protection
    #[tokio::test]
    async fn test_security_headers_csrf() {
        use axum::{
            body::Body,
            http::{Request, StatusCode},
            response::Response,
            Router,
        };
        use tower::ServiceExt;
        use uveddi::security::middleware::{SecurityHeadersMiddleware, CSRFMiddleware};
        
        // Create test application with security middleware
        let app = Router::new()
            .route("/api/test", axum::routing::get(|| async { "test" }))
            .route("/api/protected", axum::routing::post(|| async { "protected" }))
            .layer(SecurityHeadersMiddleware::new())
            .layer(CSRFMiddleware::new("test_secret"));
        
        // Test security headers on GET request
        let request = Request::builder()
            .method("GET")
            .uri("/api/test")
            .body(Body::empty())
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        
        // Check required security headers
        let headers = response.headers();
        
        assert!(headers.contains_key("X-Content-Type-Options"));
        assert_eq!(headers["X-Content-Type-Options"], "nosniff");
        
        assert!(headers.contains_key("X-Frame-Options"));
        assert_eq!(headers["X-Frame-Options"], "DENY");
        
        assert!(headers.contains_key("X-XSS-Protection"));
        assert_eq!(headers["X-XSS-Protection"], "1; mode=block");
        
        assert!(headers.contains_key("Strict-Transport-Security"));
        assert!(headers["Strict-Transport-Security"].to_str().unwrap().contains("max-age"));
        
        assert!(headers.contains_key("Content-Security-Policy"));
        assert!(headers["Content-Security-Policy"].to_str().unwrap().contains("default-src"));
        
        assert!(headers.contains_key("Referrer-Policy"));
        assert_eq!(headers["Referrer-Policy"], "strict-origin-when-cross-origin");
        
        // Test CSRF token generation
        let csrf_request = Request::builder()
            .method("GET")
            .uri("/api/csrf-token")
            .body(Body::empty())
            .unwrap();
        
        let csrf_response = app.clone().oneshot(csrf_request).await.unwrap();
        let csrf_token = csrf_response.headers()
            .get("X-CSRF-Token")
            .map(|v| v.to_str().unwrap())
            .unwrap_or("");
        
        assert!(!csrf_token.is_empty());
        
        // Test POST request without CSRF token (should fail)
        let unprotected_request = Request::builder()
            .method("POST")
            .uri("/api/protected")
            .body(Body::empty())
            .unwrap();
        
        let unprotected_response = app.clone().oneshot(unprotected_request).await.unwrap();
        assert_eq!(unprotected_response.status(), StatusCode::FORBIDDEN);
        
        // Test POST request with valid CSRF token (should succeed)
        let protected_request = Request::builder()
            .method("POST")
            .uri("/api/protected")
            .header("X-CSRF-Token", csrf_token)
            .body(Body::empty())
            .unwrap();
        
        let protected_response = app.clone().oneshot(protected_request).await.unwrap();
        assert_eq!(protected_response.status(), StatusCode::OK);
        
        println!("✅ Security headers and CSRF protection tests passed");
    }

    /// Test session security and management
    #[tokio::test]
    async fn test_session_security() {
        use uveddi::security::session::{SessionManager, SessionConfig};
        
        let session_config = SessionConfig {
            session_timeout: Duration::from_secs(3600),
            max_sessions_per_user: 3,
            secure_cookies: true,
            http_only_cookies: true,
            same_site_strict: true,
            session_rotation_interval: Duration::from_secs(300),
        };
        
        let session_manager = SessionManager::new(session_config).await.unwrap();
        
        // Test session creation
        let session_id = session_manager.create_session("test_user", "127.0.0.1").await.unwrap();
        assert!(!session_id.is_empty());
        
        // Test session validation
        let session_info = session_manager.validate_session(&session_id).await.unwrap();
        assert_eq!(session_info.user_id, "test_user");
        assert_eq!(session_info.ip_address, "127.0.0.1");
        
        // Test session updates and activity tracking
        session_manager.update_session_activity(&session_id).await.unwrap();
        
        // Test session expiration
        let expired_session_id = session_manager.create_expired_session_for_test().await.unwrap();
        let expired_validation = session_manager.validate_session(&expired_session_id).await;
        assert!(expired_validation.is_err());
        
        // Test session termination
        session_manager.terminate_session(&session_id).await.unwrap();
        let terminated_validation = session_manager.validate_session(&session_id).await;
        assert!(terminated_validation.is_err());
        
        // Test concurrent session limits
        let mut session_ids = Vec::new();
        for i in 0..5 {
            let result = session_manager.create_session("limited_user", &format!("127.0.0.{}", i)).await;
            if result.is_ok() {
                session_ids.push(result.unwrap());
            }
        }
        
        // Should only allow max_sessions_per_user (3) sessions
        assert!(session_ids.len() <= 3);
        
        // Test session hijacking protection (IP address binding)
        if let Some(valid_session) = session_ids.first() {
            let hijack_attempt = session_manager.validate_session_from_ip(valid_session, "192.168.1.100").await;
            assert!(hijack_attempt.is_err());
        }
        
        println!("✅ Session security tests passed");
    }

    /// Test API rate limiting and DOS protection
    #[tokio::test]
    async fn test_rate_limiting_dos_protection() {
        use uveddi::security::rate_limiting::{RateLimiter, RateLimitConfig};
        use std::net::IpAddr;
        
        let rate_limit_config = RateLimitConfig {
            requests_per_minute: 60,
            requests_per_hour: 1000,
            burst_size: 10,
            ban_threshold: 100,
            ban_duration: Duration::from_secs(3600),
        };
        
        let rate_limiter = RateLimiter::new(rate_limit_config).await.unwrap();
        
        let test_ip: IpAddr = "127.0.0.1".parse().unwrap();
        
        // Test normal rate limiting
        for i in 0..10 {
            let result = rate_limiter.check_rate_limit(test_ip, "/api/test").await;
            assert!(result.is_ok(), "Request {} should be allowed", i);
        }
        
        // Test burst limit exceeded
        for i in 10..15 {
            let result = rate_limiter.check_rate_limit(test_ip, "/api/test").await;
            if i >= 10 {
                assert!(result.is_err(), "Request {} should be rate limited", i);
            }
        }
        
        // Test different endpoints have separate limits
        let other_endpoint_result = rate_limiter.check_rate_limit(test_ip, "/api/other").await;
        assert!(other_endpoint_result.is_ok());
        
        // Test IP-based blocking for abuse
        let abusive_ip: IpAddr = "192.168.1.100".parse().unwrap();
        
        // Simulate abusive behavior
        for _ in 0..110 {
            let _ = rate_limiter.check_rate_limit(abusive_ip, "/api/test").await;
        }
        
        // IP should be banned
        let banned_result = rate_limiter.check_rate_limit(abusive_ip, "/api/test").await;
        assert!(banned_result.is_err());
        
        // Test whitelist functionality
        let whitelisted_ip: IpAddr = "10.0.0.1".parse().unwrap();
        rate_limiter.add_to_whitelist(whitelisted_ip).await.unwrap();
        
        // Whitelisted IP should not be rate limited
        for _ in 0..200 {
            let result = rate_limiter.check_rate_limit(whitelisted_ip, "/api/test").await;
            assert!(result.is_ok());
        }
        
        println!("✅ Rate limiting and DOS protection tests passed");
    }

    /// Test audit logging and security monitoring
    #[tokio::test]
    async fn test_audit_logging_monitoring() {
        use uveddi::security::audit::{AuditLogger, AuditEvent, AuditLevel};
        
        let audit_logger = AuditLogger::new().await.unwrap();
        
        // Test authentication audit events
        let login_event = AuditEvent {
            event_type: "authentication".to_string(),
            level: AuditLevel::Info,
            user_id: Some("test_user".to_string()),
            ip_address: Some("127.0.0.1".to_string()),
            resource: Some("login".to_string()),
            action: Some("login_success".to_string()),
            details: json!({
                "method": "password",
                "user_agent": "test_agent"
            }),
            timestamp: std::time::SystemTime::now(),
        };
        
        audit_logger.log_event(login_event).await.unwrap();
        
        // Test authorization audit events
        let authz_event = AuditEvent {
            event_type: "authorization".to_string(),
            level: AuditLevel::Warning,
            user_id: Some("test_user".to_string()),
            ip_address: Some("127.0.0.1".to_string()),
            resource: Some("admin_panel".to_string()),
            action: Some("access_denied".to_string()),
            details: json!({
                "reason": "insufficient_permissions",
                "required_role": "admin"
            }),
            timestamp: std::time::SystemTime::now(),
        };
        
        audit_logger.log_event(authz_event).await.unwrap();
        
        // Test data access audit events
        let data_access_event = AuditEvent {
            event_type: "data_access".to_string(),
            level: AuditLevel::Info,
            user_id: Some("analyst_user".to_string()),
            ip_address: Some("10.0.0.5".to_string()),
            resource: Some("sensitive_report".to_string()),
            action: Some("view".to_string()),
            details: json!({
                "report_id": "report_123",
                "classification": "confidential"
            }),
            timestamp: std::time::SystemTime::now(),
        };
        
        audit_logger.log_event(data_access_event).await.unwrap();
        
        // Test security incident events
        let security_incident = AuditEvent {
            event_type: "security_incident".to_string(),
            level: AuditLevel::Critical,
            user_id: None,
            ip_address: Some("192.168.1.100".to_string()),
            resource: Some("api".to_string()),
            action: Some("sql_injection_attempt".to_string()),
            details: json!({
                "payload": "'; DROP TABLE users; --",
                "endpoint": "/api/search",
                "blocked": true
            }),
            timestamp: std::time::SystemTime::now(),
        };
        
        audit_logger.log_event(security_incident).await.unwrap();
        
        // Test audit log query and analysis
        let recent_events = audit_logger.query_events(
            std::time::SystemTime::now() - Duration::from_secs(3600),
            std::time::SystemTime::now(),
            Some(AuditLevel::Warning),
        ).await.unwrap();
        
        assert!(!recent_events.is_empty());
        
        // Test security metrics aggregation
        let security_metrics = audit_logger.get_security_metrics(
            std::time::SystemTime::now() - Duration::from_secs(3600)
        ).await.unwrap();
        
        assert!(security_metrics.failed_login_attempts >= 0);
        assert!(security_metrics.blocked_requests >= 0);
        assert!(security_metrics.security_incidents >= 1); // We logged one incident
        
        println!("✅ Audit logging and security monitoring tests passed");
    }

    // Helper functions

    async fn create_test_rbac_manager() -> RBACManager {
        let config = SecurityConfig::default();
        RBACManager::new(config.rbac_config).await.unwrap()
    }

    #[tokio::test]
    async fn test_comprehensive_security_integration() {
        println!("🔒 Running comprehensive security integration test...");
        
        // This test simulates a complete security workflow
        let security_config = SecurityConfig::default();
        
        // Initialize all security components
        let rbac_manager = RBACManager::new(security_config.rbac_config).await.unwrap();
        let auth_manager = AuthenticationManager::new(security_config.auth_config).await.unwrap();
        let encryption_manager = EncryptionManager::new(security_config.encryption_config).await.unwrap();
        let audit_logger = AuditLogger::new().await.unwrap();
        
        // Simulate user registration with strong security
        let user_id = "integration_test_user";
        let password = "StrongTestPassword123!";
        let email = "test@secure-domain.com";
        
        // Register user
        auth_manager.register_user(user_id, email, password).await.unwrap();
        
        // Assign role
        rbac_manager.assign_role_to_user(user_id, "analyst").await.unwrap();
        
        // Authenticate user
        let credentials = Credentials {
            username: user_id.to_string(),
            password: password.to_string(),
            two_factor_code: None,
        };
        
        let token = auth_manager.authenticate(credentials).await.unwrap();
        
        // Validate token and check permissions
        let user_info = auth_manager.validate_token(&token).await.unwrap();
        assert_eq!(user_info.username, user_id);
        
        // Check specific permission
        let has_read_permission = rbac_manager.has_permission(
            user_id,
            &Resource::Analysis,
            "read"
        ).await.unwrap();
        assert!(has_read_permission);
        
        // Encrypt sensitive data
        let sensitive_data = "User's confidential analysis results";
        let encrypted_data = encryption_manager.encrypt(sensitive_data.as_bytes()).await.unwrap();
        
        // Log security event
        let audit_event = AuditEvent {
            event_type: "integration_test".to_string(),
            level: AuditLevel::Info,
            user_id: Some(user_id.to_string()),
            ip_address: Some("127.0.0.1".to_string()),
            resource: Some("test_resource".to_string()),
            action: Some("access_granted".to_string()),
            details: json!({
                "test_type": "comprehensive_security_integration",
                "data_encrypted": true
            }),
            timestamp: std::time::SystemTime::now(),
        };
        
        audit_logger.log_event(audit_event).await.unwrap();
        
        // Verify data can be decrypted
        let decrypted_data = encryption_manager.decrypt(&encrypted_data).await.unwrap();
        let decrypted_string = String::from_utf8(decrypted_data).unwrap();
        assert_eq!(decrypted_string, sensitive_data);
        
        println!("✅ Comprehensive security integration test passed");
        println!("   🔐 Authentication: PASSED");
        println!("   🛡️  Authorization: PASSED");
        println!("   🔒 Encryption: PASSED");
        println!("   📋 Audit Logging: PASSED");
    }
}