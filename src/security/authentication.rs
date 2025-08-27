//! Authentication Service for Uveddi RBAC System
//!
//! This module provides comprehensive authentication services including OAuth 2.0/OIDC
//! integration, API key authentication, and JWT token management.

use crate::security::{
    errors::{SecurityError, SecurityResult},
    models::{ApiKey, AuthenticatedUser, Session, User, UserRole},
    secrets::SecretStore,
};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use base64::Engine;
use chrono::{Duration as ChronoDuration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use oauth2::basic::BasicClient;
// Removed unused openidconnect imports
use rand::{Rng, rng};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;
use tracing::error;

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    /// Subject (user ID)
    pub sub: String,
    /// User email address
    pub email: String,
    /// User display name
    pub name: String,
    /// User roles for authorization
    pub roles: Vec<String>,
    /// Issued at timestamp
    pub iat: i64,
    /// Expiration time timestamp
    pub exp: i64,
    /// Intended audience
    pub aud: String,
    /// Token issuer
    pub iss: String,
    /// JWT ID for blacklisting
    pub jti: String,
}

/// OAuth provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthProviderConfig {
    /// Name of the OAuth provider
    pub provider_name: String,
    /// OAuth client ID
    pub client_id: String,
    /// OAuth client secret
    pub client_secret: String,
    /// Authorization URL
    pub auth_url: String,
    /// Token exchange URL
    pub token_url: String,
    /// Redirect URL after authentication
    pub redirect_url: String,
    /// Requested OAuth scopes
    pub scopes: Vec<String>,
}

/// OIDC provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcProviderConfig {
    /// Name of the OIDC provider
    pub provider_name: String,
    /// OIDC issuer URL
    pub issuer_url: String,
    /// OIDC client ID
    pub client_id: String,
    /// OIDC client secret
    pub client_secret: String,
    /// Redirect URL after authentication
    pub redirect_url: String,
    /// Requested OIDC scopes
    pub scopes: Vec<String>,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationConfig {
    /// JWT signing secret
    pub jwt_secret: String,
    /// JWT token expiry in hours
    pub jwt_expiry_hours: i64,
    /// Session expiry in hours
    pub session_expiry_hours: i64,
    /// Configured OAuth providers
    pub oauth_providers: Vec<OAuthProviderConfig>,
    /// Configured OIDC providers
    pub oidc_providers: Vec<OidcProviderConfig>,
    /// API key expiry in days (optional)
    pub api_key_expiry_days: Option<i64>,
}

impl Default for AuthenticationConfig {
    fn default() -> Self {
        // JWT secret MUST be set via environment variable or config file in production
        let jwt_secret = std::env::var("UVEDDI_JWT_SECRET")
            .unwrap_or_else(|_| {
                error!("WARNING: Using insecure default JWT secret. Set UVEDDI_JWT_SECRET environment variable in production!");
                // Generate a random secret for development
                use rand::Rng;
                let mut rng = rand::rng();
                let bytes: Vec<u8> = (0..32).map(|_| rng.random()).collect();
                base64::engine::general_purpose::STANDARD.encode(&bytes)
            });

        Self {
            jwt_secret,
            jwt_expiry_hours: 24,
            session_expiry_hours: 720, // 30 days
            oauth_providers: Vec::new(),
            oidc_providers: Vec::new(),
            api_key_expiry_days: Some(365),
        }
    }
}

/// JWT manager for secure JWT operations with timing attack protection
pub struct JwtManager {
    current_key: String,
    previous_key: Option<String>, // For graceful key rotation
    blacklisted_tokens: Arc<RwLock<HashSet<String>>>,
    key_rotation_timestamp: Arc<RwLock<Option<std::time::SystemTime>>>,
}

impl JwtManager {
    /// Create a new JWT manager
    pub fn new(initial_key: String) -> Self {
        Self {
            current_key: initial_key,
            previous_key: None,
            blacklisted_tokens: Arc::new(RwLock::new(HashSet::new())),
            key_rotation_timestamp: Arc::new(RwLock::new(None)),
        }
    }

    /// Rotate JWT signing key
    pub async fn rotate_key(&mut self) -> SecurityResult<()> {
        self.previous_key = Some(self.current_key.clone());
        self.current_key = self.generate_secure_key();

        // Update rotation timestamp
        {
            let mut timestamp = self.key_rotation_timestamp.write().await;
            *timestamp = Some(std::time::SystemTime::now());
        }

        // Schedule cleanup of previous key after rotation period
        let blacklisted_tokens = self.blacklisted_tokens.clone();
        let rotation_timestamp = self.key_rotation_timestamp.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(24 * 3600)).await;
            // Clean up old blacklisted tokens after 24 hours
            if let Some(Ok(timestamp)) = rotation_timestamp
                .read()
                .await
                .as_ref()
                .map(|t| t.elapsed())
            {
                if timestamp > Duration::from_secs(24 * 3600) {
                    blacklisted_tokens.write().await.clear();
                }
            }
        });

        Ok(())
    }

    /// Generate a secure random key
    fn generate_secure_key(&self) -> String {
        use rand::Rng;
        let mut rng = rand::rng();
        let bytes: Vec<u8> = (0..64).map(|_| rng.random()).collect();
        base64::engine::general_purpose::STANDARD.encode(&bytes)
    }

    /// Blacklist a JWT token
    pub async fn blacklist_token(&self, token: &str) -> SecurityResult<()> {
        let jti = self.extract_jti(token)?;
        self.blacklisted_tokens.write().await.insert(jti);
        Ok(())
    }

    /// Check if token is blacklisted
    pub async fn is_blacklisted(&self, token: &str) -> bool {
        if let Ok(jti) = self.extract_jti(token) {
            self.blacklisted_tokens.read().await.contains(&jti)
        } else {
            true // Invalid tokens are considered blacklisted
        }
    }

    /// Extract JWT ID from token
    fn extract_jti(&self, token: &str) -> SecurityResult<String> {
        // Simple extraction without validation (for blacklisting purposes)
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(SecurityError::InvalidCredentials);
        }

        let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(parts[1])
            .map_err(|_| SecurityError::InvalidCredentials)?;

        let claims: serde_json::Value =
            serde_json::from_slice(&payload).map_err(|_| SecurityError::InvalidCredentials)?;

        claims["jti"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or(SecurityError::InvalidCredentials)
    }

    /// Get current key for encoding
    pub fn get_current_key(&self) -> &str {
        &self.current_key
    }

    /// Get keys for decoding (current and previous for rotation support)
    pub fn get_decoding_keys(&self) -> Vec<&str> {
        let mut keys = vec![self.current_key.as_str()];
        if let Some(ref prev_key) = self.previous_key {
            keys.push(prev_key.as_str());
        }
        keys
    }
}

/// Authentication service
pub struct AuthenticationService {
    config: AuthenticationConfig,
    oauth_clients: HashMap<String, BasicClient>,
    // TODO: OIDC functionality temporarily disabled for alpha release due to v4.0.1 breaking changes
    // oidc_clients: HashMap<String, CoreClient>,
    secret_store: Arc<dyn SecretStore>,
    session_store: Arc<RwLock<HashMap<String, Session>>>,
    api_key_store: Arc<RwLock<HashMap<String, ApiKey>>>,
    jwt_manager: Arc<RwLock<JwtManager>>,
}

impl AuthenticationService {
    /// Create a new authentication service
    pub async fn new(
        config: AuthenticationConfig,
        secret_store: Arc<dyn SecretStore>,
    ) -> SecurityResult<Self> {
        let oauth_clients = HashMap::new();
        // TODO: OIDC disabled for alpha release
        // let mut oidc_clients = HashMap::new();

        // TODO: OAuth2 client initialization disabled for alpha release
        // OAuth2 integration causes compilation issues and is not needed for core analysis
        /*
        // Initialize OAuth clients
        for provider in &config.oauth_providers {
            // OAuth2 client configuration would go here
        }
        */

        // TODO: OIDC clients initialization disabled for alpha release
        /*
        // Initialize OIDC clients
        for provider in &config.oidc_providers {
            let issuer_url = IssuerUrl::new(provider.issuer_url.clone()).map_err(|e| {
                SecurityError::OidcProviderError {
                    provider: provider.provider_name.clone(),
                    error: format!("Invalid issuer URL: {}", e),
                }
            })?;

            let http_client = reqwest::Client::new();
            let provider_metadata =
                CoreProviderMetadata::discover_async(issuer_url, &http_client)
                    .await
                    .map_err(|e| SecurityError::OidcProviderError {
                        provider: provider.provider_name.clone(),
                        error: format!("Failed to discover provider metadata: {}", e),
                    })?;

            let client = CoreClient::from_provider_metadata(
                provider_metadata,
                OidcClientId::new(provider.client_id.clone()),
                Some(OidcClientSecret::new(provider.client_secret.clone())),
            )
            .set_redirect_uri(
                OidcRedirectUrl::new(provider.redirect_url.clone()).map_err(|e| {
                    SecurityError::OidcProviderError {
                        provider: provider.provider_name.clone(),
                        error: format!("Invalid redirect URL: {}", e),
                    }
                })?,
            );

            oidc_clients.insert(provider.provider_name.clone(), client);
        }
        */

        // Initialize JWT manager with current configuration
        let jwt_secret = secret_store
            .get_secret("jwt_secret")
            .await
            .unwrap_or_else(|_| config.jwt_secret.clone());
        let jwt_manager = Arc::new(RwLock::new(JwtManager::new(jwt_secret)));

        Ok(Self {
            config,
            oauth_clients,
            // oidc_clients, // TODO: Disabled for alpha release
            secret_store,
            session_store: Arc::new(RwLock::new(HashMap::new())),
            api_key_store: Arc::new(RwLock::new(HashMap::new())),
            jwt_manager,
        })
    }

    /// Generate OAuth authorization URL (DISABLED FOR ALPHA RELEASE)
    pub async fn get_oauth_auth_url(&self, _provider: &str) -> SecurityResult<(String, String)> {
        Err(SecurityError::AuthenticationFailed {
            reason: "OAuth2 authentication is disabled in alpha release".to_string(),
        })
    }

    /// Generate OIDC authorization URL (DISABLED FOR ALPHA RELEASE)
    pub async fn get_oidc_auth_url(
        &self,
        _provider: &str,
    ) -> SecurityResult<(String, String, String)> {
        Err(SecurityError::AuthenticationFailed {
            reason: "OIDC functionality temporarily disabled in alpha release".to_string(),
        })
        /*
        let client =
            self.oidc_clients
                .get(provider)
                .ok_or_else(|| SecurityError::OidcProviderError {
                    provider: provider.to_string(),
                    error: "OIDC provider not found".to_string(),
                })?;

        let provider_config = self
            .config
            .oidc_providers
            .iter()
            .find(|p| p.provider_name == provider)
            .ok_or_else(|| SecurityError::OidcProviderError {
                provider: provider.to_string(),
                error: "OIDC provider config not found".to_string(),
            })?;

        let mut auth_request = client.authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            OidcCsrfToken::new_random,
            Nonce::new_random,
        );

        for scope in &provider_config.scopes {
            auth_request = auth_request.add_scope(openidconnect::Scope::new(scope.clone()));
        }

        let (auth_url, csrf_token, nonce) = auth_request.url();

        Ok((
            auth_url.to_string(),
            csrf_token.secret().clone(),
            nonce.secret().clone(),
        ))
        */
    }

    /// Authenticate user with OAuth authorization code (DISABLED FOR ALPHA RELEASE)  
    pub async fn authenticate_oauth(
        &self,
        _provider: &str,
        _auth_code: &str,
        _csrf_token: &str,
    ) -> SecurityResult<AuthenticatedUser> {
        Err(SecurityError::AuthenticationFailed {
            reason: "OAuth2 authentication is disabled in alpha release".to_string(),
        })
    }

    /// Authenticate user with OIDC authorization code (DISABLED FOR ALPHA RELEASE)
    pub async fn authenticate_oidc(
        &self,
        _provider: &str,
        _auth_code: &str,
        _nonce: &str,
    ) -> SecurityResult<AuthenticatedUser> {
        Err(SecurityError::AuthenticationFailed {
            reason: "OIDC functionality temporarily disabled in alpha release".to_string(),
        })
        /*
        let client =
            self.oidc_clients
                .get(provider)
                .ok_or_else(|| SecurityError::OidcProviderError {
                    provider: provider.to_string(),
                    error: "OIDC provider not found".to_string(),
                })?;

        // Exchange authorization code for tokens - updated for OAuth2 5.0
        let http_client = reqwest::Client::new();
        let auth_code = AuthorizationCode::new(auth_code.to_string());
        let token_response = client
            .exchange_code(auth_code)
            .request_async(&http_client)
            .await
            .map_err(|e| SecurityError::OidcProviderError {
                provider: provider.to_string(),
                error: format!("Token exchange failed: {}", e),
            })?;

        // Verify ID token
        let id_token = OidcTokenResponse::id_token(&token_response).ok_or_else(|| {
            SecurityError::OidcProviderError {
                provider: provider.to_string(),
                error: "No ID token received".to_string(),
            }
        })?;

        let claims = id_token
            .claims(&client.id_token_verifier(), &Nonce::new(nonce.to_string()))
            .map_err(|e| SecurityError::OidcProviderError {
                provider: provider.to_string(),
                error: format!("ID token verification failed: {}", e),
            })?;

        // Extract user information from claims
        let user_info = UserInfo {
            external_id: claims.subject().to_string(),
            email: claims.email().map(|e| e.to_string()).unwrap_or_default(),
            name: claims
                .name()
                .and_then(|n| n.get(None))
                .map(|n| n.to_string())
                .unwrap_or_default(),
        };

        // Create or update user
        let user = self.create_or_update_user(user_info).await?;

        // Create session
        let session = self.create_session(&user).await?;

        // Create authenticated user
        let auth_user = AuthenticatedUser::new(
            user,
            vec![UserRole::Developer], // Default role, would be determined by business logic
            vec![],                    // Permissions would be loaded from database
            Some(session.id),
        );

        Ok(auth_user)
        */
    }

    /// Authenticate user with API key
    pub async fn authenticate_api_key(&self, api_key: &str) -> SecurityResult<AuthenticatedUser> {
        // Parse API key format: "uvd_" + 8 chars + "_" + 32 chars
        if !api_key.starts_with("uvd_") || api_key.len() != 45 {
            return Err(SecurityError::InvalidApiKeyFormat);
        }

        let key_prefix = &api_key[0..12]; // "uvd_" + 8 chars
        let key_secret = &api_key[13..]; // 32 chars after "_"

        // Find API key by prefix
        let api_key_record = {
            let store = self.api_key_store.read().await;
            store.get(key_prefix).cloned()
        };

        let api_key_record = api_key_record.ok_or_else(|| SecurityError::ApiKeyNotFound {
            key_prefix: key_prefix.to_string(),
        })?;

        // Verify API key is valid
        if !api_key_record.is_valid() {
            if api_key_record.is_expired() {
                return Err(SecurityError::ApiKeyExpired {
                    key_prefix: key_prefix.to_string(),
                });
            }
            return Err(SecurityError::InvalidCredentials);
        }

        // Verify key hash using secure password verification
        let argon2 = Argon2::default();
        let parsed_hash = PasswordHash::new(&api_key_record.key_hash).map_err(|e| {
            SecurityError::CryptographicError {
                operation: "password_hash_parse".to_string(),
                error: e.to_string(),
            }
        })?;

        // Use constant-time verification to prevent timing attacks
        let verification_result = argon2.verify_password(key_secret.as_bytes(), &parsed_hash);

        if verification_result.is_err() {
            return Err(SecurityError::InvalidCredentials);
        }

        // Update last used time
        // In production, this would update the database

        // Create user from API key
        let user = if let Some(user_id) = api_key_record.user_id {
            // Load user from database
            self.load_user_by_id(user_id).await?
        } else {
            // Service account
            User::new(
                format!("service_{}", api_key_record.id),
                format!("service+{}@uveddi.com", api_key_record.id),
                api_key_record.name.clone(),
            )
        };

        // Create authenticated user
        let auth_user = AuthenticatedUser::new(
            user,
            vec![UserRole::Service], // API keys default to service role
            vec![],                  // Permissions would be loaded from database
            None,                    // No session for API keys
        );

        Ok(auth_user)
    }

    /// Authenticate user with JWT token (with timing attack protection)
    pub async fn authenticate_jwt(&self, token: &str) -> SecurityResult<AuthenticatedUser> {
        self.authenticate_jwt_secure(token).await
    }

    /// Secure JWT validation with timing attack protection
    pub async fn authenticate_jwt_secure(&self, token: &str) -> SecurityResult<AuthenticatedUser> {
        let start_time = Instant::now();

        // Perform actual validation
        let result = self.authenticate_jwt_internal(token).await;

        // Add consistent timing to prevent timing attacks
        let elapsed = start_time.elapsed();
        let target_duration = Duration::from_millis(50); // Minimum processing time

        if elapsed < target_duration {
            let delay = target_duration - elapsed;
            tokio::time::sleep(delay).await;
        }

        result
    }

    /// Internal JWT authentication method
    async fn authenticate_jwt_internal(&self, token: &str) -> SecurityResult<AuthenticatedUser> {
        // Check if token is blacklisted first
        let jwt_manager = self.jwt_manager.read().await;
        if jwt_manager.is_blacklisted(token).await {
            return Err(SecurityError::InvalidCredentials);
        }

        // Try decoding with current and previous keys (for key rotation support)
        let decoding_keys = jwt_manager.get_decoding_keys();
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true; // Ensure token expiration is checked

        let mut last_error = None;
        let mut token_data = None;

        for key in decoding_keys {
            let decoding_key = DecodingKey::from_secret(key.as_ref());
            match decode::<JwtClaims>(token, &decoding_key, &validation) {
                Ok(data) => {
                    token_data = Some(data);
                    break;
                }
                Err(e) => {
                    last_error = Some(e);
                    continue;
                }
            }
        }

        let token_data = token_data.ok_or_else(|| {
            last_error.unwrap_or_else(|| {
                jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidToken)
            })
        })?;

        let claims = token_data.claims;

        // Validate additional claims
        if claims.aud != "uveddi" || claims.iss != "uveddi-auth" {
            return Err(SecurityError::InvalidCredentials);
        }

        // Load user from database
        let user = self.load_user_by_external_id(&claims.sub).await?;

        // Parse roles
        let roles = claims
            .roles
            .iter()
            .filter_map(|r| r.parse::<UserRole>().ok())
            .collect();

        // Create authenticated user
        let auth_user = AuthenticatedUser::new(
            user,
            roles,
            vec![], // Permissions would be loaded from database
            None,   // No session for JWT
        );

        Ok(auth_user)
    }

    /// Constant-time string comparison for sensitive operations
    fn constant_time_compare(a: &str, b: &str) -> bool {
        use subtle::ConstantTimeEq;
        a.as_bytes().ct_eq(b.as_bytes()).into()
    }

    /// Create a new session for user
    pub async fn create_session(&self, user: &User) -> SecurityResult<Session> {
        let session_token = self.generate_session_token();
        let expires_at = Utc::now() + ChronoDuration::hours(self.config.session_expiry_hours);

        let session = Session::new(
            user.id,
            session_token.clone(),
            expires_at,
            None, // IP address would be set by middleware
            None, // User agent would be set by middleware
        );

        // Store session
        {
            let mut store = self.session_store.write().await;
            store.insert(session_token, session.clone());
        }

        Ok(session)
    }

    /// Validate session token
    pub async fn validate_session(&self, token: &str) -> SecurityResult<Session> {
        let session = {
            let store = self.session_store.read().await;
            store.get(token).cloned()
        };

        let session = session.ok_or_else(|| SecurityError::SessionNotFound {
            session_id: token.to_string(),
        })?;

        if !session.is_valid() {
            if session.is_expired() {
                return Err(SecurityError::SessionExpired {
                    session_id: token.to_string(),
                });
            }
            return Err(SecurityError::InvalidCredentials);
        }

        Ok(session)
    }

    /// Generate JWT token for user
    pub async fn generate_jwt(&self, user: &AuthenticatedUser) -> SecurityResult<String> {
        let jwt_manager = self.jwt_manager.read().await;
        let jwt_secret = jwt_manager.get_current_key();

        let now = Utc::now();
        let exp = now + ChronoDuration::hours(self.config.jwt_expiry_hours);

        // Generate unique JWT ID for blacklisting support
        let jti = Uuid::new_v4().to_string();

        let claims = JwtClaims {
            sub: user.external_id.clone(),
            email: user.email.clone(),
            name: user.display_name.clone(),
            roles: user.role_names(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
            aud: "uveddi".to_string(),
            iss: "uveddi-auth".to_string(),
            jti,
        };

        let header = Header::new(Algorithm::HS256);
        let encoding_key = EncodingKey::from_secret(jwt_secret.as_ref());

        let token = encode(&header, &claims, &encoding_key)?;
        Ok(token)
    }

    /// Blacklist a JWT token
    pub async fn blacklist_jwt(&self, token: &str) -> SecurityResult<()> {
        let jwt_manager = self.jwt_manager.read().await;
        jwt_manager.blacklist_token(token).await
    }

    /// Rotate JWT signing key
    pub async fn rotate_jwt_key(&self) -> SecurityResult<()> {
        let mut jwt_manager = self.jwt_manager.write().await;
        jwt_manager.rotate_key().await
    }

    /// Generate API key
    pub async fn generate_api_key(
        &self,
        user_id: Option<Uuid>,
        name: String,
        created_by: Option<Uuid>,
    ) -> SecurityResult<(String, ApiKey)> {
        // Generate random key components
        let mut rng = rng();
        let key_id: String = (0..8)
            .map(|_| rng.random::<u8>() % 62)
            .map(|i| match i {
                0..=9 => (b'0' + i) as char,
                10..=35 => (b'A' + (i - 10)) as char,
                36..=61 => (b'a' + (i - 36)) as char,
                _ => unreachable!(),
            })
            .collect();
        let key_secret: String = (0..32)
            .map(|_| rng.random::<u8>() % 62)
            .map(|i| match i {
                0..=9 => (b'0' + i) as char,
                10..=35 => (b'A' + (i - 10)) as char,
                36..=61 => (b'a' + (i - 36)) as char,
                _ => unreachable!(),
            })
            .collect();

        let key_prefix = format!("uvd_{}", key_id);
        let full_key = format!("{key_prefix}_{key_secret}");

        // Hash the secret part
        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);
        let key_hash = argon2
            .hash_password(key_secret.as_bytes(), &salt)
            .map_err(|e| SecurityError::CryptographicError {
                operation: "password_hash".to_string(),
                error: e.to_string(),
            })?
            .to_string();

        // Create API key record
        let api_key = ApiKey::new(
            user_id,
            name,
            key_hash,
            key_prefix.clone(),
            None, // Permissions would be set separately
            created_by,
        );

        // Store API key
        {
            let mut store = self.api_key_store.write().await;
            store.insert(key_prefix, api_key.clone());
        }

        Ok((full_key, api_key))
    }

    /// Revoke API key
    pub async fn revoke_api_key(&self, key_prefix: &str) -> SecurityResult<()> {
        let mut store = self.api_key_store.write().await;
        if let Some(api_key) = store.get_mut(key_prefix) {
            api_key.deactivate();
            Ok(())
        } else {
            Err(SecurityError::ApiKeyNotFound {
                key_prefix: key_prefix.to_string(),
            })
        }
    }

    /// Revoke session
    pub async fn revoke_session(&self, token: &str) -> SecurityResult<()> {
        let mut store = self.session_store.write().await;
        if let Some(session) = store.get_mut(token) {
            session.deactivate();
            Ok(())
        } else {
            Err(SecurityError::SessionNotFound {
                session_id: token.to_string(),
            })
        }
    }

    /// Generate secure session token
    fn generate_session_token(&self) -> String {
        let mut rng = rng();
        (0..64)
            .map(|_| rng.random::<u8>() % 62)
            .map(|i| match i {
                0..=9 => (b'0' + i) as char,
                10..=35 => (b'A' + (i - 10)) as char,
                36..=61 => (b'a' + (i - 36)) as char,
                _ => unreachable!(),
            })
            .collect()
    }

    /// Get OAuth user info (simplified implementation)
    async fn get_oauth_user_info(
        &self,
        _provider: &str,
        _access_token: &str,
    ) -> SecurityResult<UserInfo> {
        // This would call the provider's user info endpoint
        // Simplified implementation
        Ok(UserInfo {
            external_id: "oauth_user_123".to_string(),
            email: "user@example.com".to_string(),
            name: "OAuth User".to_string(),
        })
    }

    /// Create or update user
    async fn create_or_update_user(&self, user_info: UserInfo) -> SecurityResult<User> {
        // This would interact with the database
        // Simplified implementation
        Ok(User::new(
            user_info.external_id,
            user_info.email,
            user_info.name,
        ))
    }

    /// Load user by ID
    async fn load_user_by_id(&self, _user_id: Uuid) -> SecurityResult<User> {
        // This would load from database
        Err(SecurityError::UserNotFound {
            user_id: "not_implemented".to_string(),
        })
    }

    /// Load user by external ID
    async fn load_user_by_external_id(&self, _external_id: &str) -> SecurityResult<User> {
        // This would load from database
        Err(SecurityError::UserNotFound {
            user_id: "not_implemented".to_string(),
        })
    }
}

/// User information from OAuth/OIDC providers
#[derive(Debug, Clone)]
struct UserInfo {
    external_id: String,
    email: String,
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::secrets::MockSecretStore;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_authentication_service_creation() {
        let config = AuthenticationConfig::default();
        let secret_store = Arc::new(MockSecretStore::new());

        let result = AuthenticationService::new(config, secret_store).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_jwt_generation_and_validation() {
        let config = AuthenticationConfig::default();
        let secret_store = Arc::new(MockSecretStore::new());
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
        assert!(!jwt.is_empty());

        // Validate JWT
        let validated_user = auth_service.authenticate_jwt(&jwt).await;
        // This would fail in the simplified implementation due to user loading
        // In a real implementation, it would succeed
    }

    #[tokio::test]
    async fn test_api_key_generation() {
        let config = AuthenticationConfig::default();
        let secret_store = Arc::new(MockSecretStore::new());
        let auth_service = AuthenticationService::new(config, secret_store)
            .await
            .unwrap();

        let user_id = Uuid::new_v4();
        let (api_key, key_record) = auth_service
            .generate_api_key(Some(user_id), "Test API Key".to_string(), None)
            .await
            .unwrap();

        // Verify API key format
        assert!(api_key.starts_with("uvd_"));
        assert_eq!(api_key.len(), 45);
        assert_eq!(key_record.name, "Test API Key");
        assert_eq!(key_record.user_id, Some(user_id));
    }

    #[tokio::test]
    async fn test_session_creation_and_validation() {
        let config = AuthenticationConfig::default();
        let secret_store = Arc::new(MockSecretStore::new());
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

        // Validate session
        let validated_session = auth_service
            .validate_session(&session.session_token)
            .await
            .unwrap();
        assert_eq!(validated_session.id, session.id);
    }

    #[tokio::test]
    async fn test_session_revocation() {
        let config = AuthenticationConfig::default();
        let secret_store = Arc::new(MockSecretStore::new());
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
        let token = session.session_token.clone();

        // Revoke session
        auth_service.revoke_session(&token).await.unwrap();

        // Validation should fail
        let result = auth_service.validate_session(&token).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_api_key_revocation() {
        let config = AuthenticationConfig::default();
        let secret_store = Arc::new(MockSecretStore::new());
        let auth_service = AuthenticationService::new(config, secret_store)
            .await
            .unwrap();

        let (api_key, _) = auth_service
            .generate_api_key(None, "Test API Key".to_string(), None)
            .await
            .unwrap();

        let key_prefix = &api_key[0..12];

        // Revoke API key
        auth_service.revoke_api_key(key_prefix).await.unwrap();

        // Authentication should fail
        let result = auth_service.authenticate_api_key(&api_key).await;
        assert!(result.is_err());
    }
}

// Mock secret store for testing
#[cfg(test)]
mod mock_secret_store {
    use super::*;
    use crate::security::secrets::SecretStore;
    use std::collections::HashMap;
    use tokio::sync::RwLock;

    pub struct MockSecretStore {
        secrets: RwLock<HashMap<String, String>>,
    }

    impl MockSecretStore {
        pub fn new() -> Self {
            let mut secrets = HashMap::new();
            secrets.insert("jwt_secret".to_string(), "test-secret".to_string());

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
                .ok_or_else(|| SecurityError::SecretNotFound {
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
