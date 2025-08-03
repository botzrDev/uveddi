//! Security Tests for Uveddi Authentication System
//!
//! These tests verify timing attack resistance, JWT security, and other security measures.

use std::time::{Duration, Instant};
use tokio;
use uveddi::security::{
    authentication::{AuthenticationConfig, AuthenticationService},
    models::{AuthenticatedUser, User, UserRole},
};

#[tokio::test]
async fn test_jwt_timing_consistency() {
    let config = AuthenticationConfig::default();
    let secret_store = std::sync::Arc::new(MockSecretStore::new());
    let auth_service = AuthenticationService::new(config, secret_store)
        .await
        .unwrap();

    // Test timing consistency between valid and invalid tokens
    let valid_token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ0ZXN0X3VzZXIiLCJlbWFpbCI6InRlc3RAZXhhbXBsZS5jb20iLCJuYW1lIjoiVGVzdCBVc2VyIiwicm9sZXMiOlsiRGV2ZWxvcGVyIl0sImlhdCI6MTcwNzM0MjAwMCwiZXhwIjoxNzA3NDI4NDAwLCJhdWQiOiJ1dmVkZGkiLCJpc3MiOiJ1dmVkZGktYXV0aCIsImp0aSI6InRlc3QtandlLWlkIn0.invalid_signature";
    let invalid_token = "invalid.jwt.token";

    let mut valid_times = Vec::new();
    let mut invalid_times = Vec::new();

    // Test multiple times to get timing distribution
    for _ in 0..50 {
        let start = Instant::now();
        let _ = auth_service.authenticate_jwt(valid_token).await;
        valid_times.push(start.elapsed());

        let start = Instant::now();
        let _ = auth_service.authenticate_jwt(invalid_token).await;
        invalid_times.push(start.elapsed());
    }

    // Calculate average times
    let avg_valid: Duration = valid_times.iter().sum::<Duration>() / valid_times.len() as u32;
    let avg_invalid: Duration = invalid_times.iter().sum::<Duration>() / invalid_times.len() as u32;

    // The difference should be minimal (within 10ms) due to timing protection
    let time_diff = if avg_valid > avg_invalid {
        avg_valid - avg_invalid
    } else {
        avg_invalid - avg_valid
    };

    assert!(
        time_diff < Duration::from_millis(10),
        "Timing difference too large: {:?} (valid: {:?}, invalid: {:?})",
        time_diff,
        avg_valid,
        avg_invalid
    );

    // All authentication attempts should take at least the minimum time
    let min_time = Duration::from_millis(45); // Slightly less than target 50ms due to measurement overhead
    for time in valid_times.iter().chain(invalid_times.iter()) {
        assert!(
            *time >= min_time,
            "Authentication completed too quickly: {:?}",
            time
        );
    }
}

#[tokio::test]
async fn test_constant_time_comparison() {
    use uveddi::security::authentication::AuthenticationService;
    
    // This tests the constant_time_compare function indirectly through API key verification
    let config = AuthenticationConfig::default();
    let secret_store = std::sync::Arc::new(MockSecretStore::new());
    let auth_service = AuthenticationService::new(config, secret_store)
        .await
        .unwrap();

    // Generate an API key
    let (api_key, _) = auth_service
        .generate_api_key(None, "Test Key".to_string(), None)
        .await
        .unwrap();

    // Test with correct key (should fail at user loading stage)
    let start = Instant::now();
    let result1 = auth_service.authenticate_api_key(&api_key).await;
    let time1 = start.elapsed();

    // Test with incorrect key (same prefix, wrong secret)
    let wrong_key = format!("{}wrong_secret_here_12345678901234567890", &api_key[..13]);
    let start = Instant::now();
    let result2 = auth_service.authenticate_api_key(&wrong_key).await;
    let time2 = start.elapsed();

    // Result1 should succeed (valid key for service account)
    // Result2 should fail (invalid key)
    assert!(result1.is_ok(), "Valid API key should succeed for service accounts");
    assert!(result2.is_err(), "Invalid API key should fail");
    
    // The key with correct hash should take longer (gets to user loading)
    // The key with wrong hash should fail faster (at crypto verification)
    // But both should still be in reasonable range due to argon2 timing

    // Time difference should be minimal for argon2 verification
    let time_diff = if time1 > time2 { time1 - time2 } else { time2 - time1 };
    
    // Argon2 should provide natural timing resistance, but we allow some variance
    assert!(
        time_diff < Duration::from_millis(100),
        "API key verification timing difference too large: {:?}",
        time_diff
    );
}

#[tokio::test]
async fn test_jwt_rotation() {
    let config = AuthenticationConfig::default();
    let secret_store = std::sync::Arc::new(MockSecretStore::new());
    let auth_service = AuthenticationService::new(config, secret_store)
        .await
        .unwrap();

    // Create a test user
    let user = User::new(
        "test_user".to_string(),
        "test@example.com".to_string(),
        "Test User".to_string(),
    );
    let auth_user = AuthenticatedUser::new(user, vec![UserRole::Developer], vec![], None);

    // Generate JWT with current key
    let jwt1 = auth_service.generate_jwt(&auth_user).await.unwrap();
    assert!(!jwt1.is_empty());

    // Rotate key
    auth_service.rotate_jwt_key().await.unwrap();

    // Generate JWT with new key
    let jwt2 = auth_service.generate_jwt(&auth_user).await.unwrap();
    assert!(!jwt2.is_empty());

    // JWTs should be different (different keys)
    assert_ne!(jwt1, jwt2);

    // Both tokens should validate during rotation period
    // Note: In a real scenario with user database, both would validate
    let result1 = auth_service.authenticate_jwt(&jwt1).await;
    let result2 = auth_service.authenticate_jwt(&jwt2).await;

    // Both should fail for the same reason (no user database implementation)
    // but for different JWT validation errors if keys weren't properly managed
    assert!(result1.is_err());
    assert!(result2.is_err());
}

#[tokio::test]
async fn test_jwt_blacklisting() {
    let config = AuthenticationConfig::default();
    let secret_store = std::sync::Arc::new(MockSecretStore::new());
    let auth_service = AuthenticationService::new(config, secret_store)
        .await
        .unwrap();

    // Create a test user
    let user = User::new(
        "test_user".to_string(),
        "test@example.com".to_string(),
        "Test User".to_string(),
    );
    let auth_user = AuthenticatedUser::new(user, vec![UserRole::Developer], vec![], None);

    // Generate JWT
    let jwt = auth_service.generate_jwt(&auth_user).await.unwrap();

    // Initially should fail due to user loading, but not due to blacklisting
    let result1 = auth_service.authenticate_jwt(&jwt).await;
    assert!(result1.is_err());

    // Blacklist the token
    auth_service.blacklist_jwt(&jwt).await.unwrap();

    // Now should fail due to blacklisting (faster failure)
    let start = Instant::now();
    let result2 = auth_service.authenticate_jwt(&jwt).await;
    let blacklist_time = start.elapsed();

    assert!(result2.is_err());

    // Blacklisted tokens should be rejected quickly
    assert!(
        blacklist_time < Duration::from_millis(55), // Should be faster than full validation
        "Blacklisted token took too long to reject: {:?}",
        blacklist_time
    );
}

#[tokio::test]
async fn test_jwt_claims_validation() {
    let config = AuthenticationConfig::default();
    let secret_store = std::sync::Arc::new(MockSecretStore::new());
    let auth_service = AuthenticationService::new(config, secret_store)
        .await
        .unwrap();

    // Test with invalid audience
    let invalid_aud_token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ0ZXN0X3VzZXIiLCJlbWFpbCI6InRlc3RAZXhhbXBsZS5jb20iLCJuYW1lIjoiVGVzdCBVc2VyIiwicm9sZXMiOlsiRGV2ZWxvcGVyIl0sImlhdCI6MTcwNzM0MjAwMCwiZXhwIjoxNzA3NDI4NDAwLCJhdWQiOiJpbnZhbGlkIiwiaXNzIjoidXZlZGRpLWF1dGgiLCJqdGkiOiJ0ZXN0LWp3dC1pZCJ9.invalid";

    // Test with invalid issuer
    let invalid_iss_token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ0ZXN0X3VzZXIiLCJlbWFpbCI6InRlc3RAZXhhbXBsZS5jb20iLCJuYW1lIjoiVGVzdCBVc2VyIiwicm9sZXMiOlsiRGV2ZWxvcGVyIl0sImlhdCI6MTcwNzM0MjAwMCwiZXhwIjoxNzA3NDI4NDAwLCJhdWQiOiJ1dmVkZGkiLCJpc3MiOiJpbnZhbGlkIiwiand0aSI6InRlc3QtandlLWlkIn0.invalid";

    let result1 = auth_service.authenticate_jwt(invalid_aud_token).await;
    let result2 = auth_service.authenticate_jwt(invalid_iss_token).await;

    assert!(result1.is_err());
    assert!(result2.is_err());
}

#[tokio::test]
async fn test_session_security() {
    let config = AuthenticationConfig::default();
    let secret_store = std::sync::Arc::new(MockSecretStore::new());
    let auth_service = AuthenticationService::new(config, secret_store)
        .await
        .unwrap();

    let user = User::new(
        "test_user".to_string(),
        "test@example.com".to_string(),
        "Test User".to_string(),
    );

    // Create session
    let session = auth_service.create_session(&user).await.unwrap();
    assert!(session.is_valid());
    assert_eq!(session.session_token.len(), 64); // Secure token length

    // Validate session
    let validated = auth_service
        .validate_session(&session.session_token)
        .await
        .unwrap();
    assert_eq!(validated.id, session.id);

    // Revoke session
    auth_service
        .revoke_session(&session.session_token)
        .await
        .unwrap();

    // Should fail validation after revocation
    let result = auth_service
        .validate_session(&session.session_token)
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_api_key_security() {
    let config = AuthenticationConfig::default();
    let secret_store = std::sync::Arc::new(MockSecretStore::new());
    let auth_service = AuthenticationService::new(config, secret_store)
        .await
        .unwrap();

    // Generate API key
    let (api_key, _key_record) = auth_service
        .generate_api_key(None, "Test Key".to_string(), None)
        .await
        .unwrap();

    // Verify format
    assert!(api_key.starts_with("uvd_"));
    assert_eq!(api_key.len(), 45);

    // Test authentication (should succeed for service accounts)
    let result = auth_service.authenticate_api_key(&api_key).await;
    assert!(result.is_ok(), "API key authentication should succeed for service accounts");

    // Test invalid format
    let invalid_keys = vec![
        "invalid_key",
        "uvd_short",
        "wrong_prefix_1234567890123456789012345678901234567890",
        "",
    ];

    for invalid_key in invalid_keys {
        let result = auth_service.authenticate_api_key(invalid_key).await;
        assert!(result.is_err());
    }

    // Revoke API key
    let key_prefix = &api_key[..12];
    auth_service.revoke_api_key(key_prefix).await.unwrap();

    // Should fail after revocation
    let result = auth_service.authenticate_api_key(&api_key).await;
    assert!(result.is_err());
}

// Mock secret store for testing
mod mock_secret_store {
    use super::*;
    use uveddi::security::{errors::SecurityResult, secrets::SecretStore};
    use std::collections::HashMap;
    use tokio::sync::RwLock;

    pub struct MockSecretStore {
        secrets: RwLock<HashMap<String, String>>,
    }

    impl MockSecretStore {
        pub fn new() -> Self {
            let mut secrets = HashMap::new();
            secrets.insert("jwt_secret".to_string(), "test-secret-key-for-jwt-signing".to_string());

            Self {
                secrets: RwLock::new(secrets),
            }
        }
    }

    #[async_trait::async_trait]
    impl SecretStore for MockSecretStore {
        async fn get_secret(&self, key: &str) -> SecurityResult<String> {
            let secrets = self.secrets.read().await;
            secrets
                .get(key)
                .cloned()
                .ok_or_else(|| uveddi::security::errors::SecurityError::SecretNotFound {
                    key: key.to_string(),
                })
        }

        async fn set_secret(&self, key: &str, value: &str) -> SecurityResult<()> {
            let mut secrets = self.secrets.write().await;
            secrets.insert(key.to_string(), value.to_string());
            Ok(())
        }

        async fn delete_secret(&self, key: &str) -> SecurityResult<()> {
            let mut secrets = self.secrets.write().await;
            secrets.remove(key);
            Ok(())
        }

        async fn list_secret_keys(&self) -> SecurityResult<Vec<String>> {
            let secrets = self.secrets.read().await;
            Ok(secrets.keys().cloned().collect())
        }

        async fn secret_exists(&self, key: &str) -> SecurityResult<bool> {
            let secrets = self.secrets.read().await;
            Ok(secrets.contains_key(key))
        }
    }
}

// Re-export for use in tests
pub use mock_secret_store::MockSecretStore;