//! Security Configuration Management for Uveddi RBAC
//!
//! This module provides hierarchical configuration management for security settings
//! including authentication, authorization, and audit logging configuration.

use crate::security::{
    authentication::{AuthenticationConfig, OAuthProviderConfig, OidcProviderConfig},
    errors::{SecurityError, SecurityResult},
    secrets::{SecretStore, SecretStoreConfig},
};
use config::{Config, ConfigError, Environment, File, FileFormat, Source};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Complete security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Authentication configuration
    pub authentication: AuthenticationConfig,
    /// Authorization configuration
    pub authorization: AuthorizationConfig,
    /// Audit logging configuration
    pub audit: AuditConfig,
    /// API security configuration
    pub api_security: ApiSecurityConfig,
    /// Rate limiting configuration
    pub rate_limiting: RateLimitingConfig,
    /// Secret management configuration
    pub secrets: SecretManagementConfig,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            authentication: AuthenticationConfig::default(),
            authorization: AuthorizationConfig::default(),
            audit: AuditConfig::default(),
            api_security: ApiSecurityConfig::default(),
            rate_limiting: RateLimitingConfig::default(),
            secrets: SecretManagementConfig::default(),
        }
    }
}

/// Authorization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationConfig {
    /// Enable authorization engine
    pub enabled: bool,
    /// Cache user roles for this duration (seconds)
    pub role_cache_ttl: u64,
    /// Cache role permissions for this duration (seconds)
    pub permission_cache_ttl: u64,
    /// Default role for new users
    pub default_role: String,
    /// Enable ABAC (Attribute-Based Access Control)
    pub enable_abac: bool,
    /// Custom policy file path
    pub policy_file: Option<String>,
}

impl Default for AuthorizationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            role_cache_ttl: 300,       // 5 minutes
            permission_cache_ttl: 600, // 10 minutes
            default_role: "Developer".to_string(),
            enable_abac: true,
            policy_file: None,
        }
    }
}

/// Audit logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// Enable audit logging
    pub enabled: bool,
    /// Audit store type (memory, database, file)
    pub store_type: String,
    /// Database connection string for database store
    pub database_url: Option<String>,
    /// File path for file store
    pub file_path: Option<String>,
    /// Log all authentication attempts
    pub log_authentication: bool,
    /// Log all authorization decisions
    pub log_authorization: bool,
    /// Log all data access
    pub log_data_access: bool,
    /// Log configuration changes
    pub log_configuration_changes: bool,
    /// Log security violations
    pub log_security_violations: bool,
    /// Enable integrity verification
    pub enable_integrity_verification: bool,
    /// Batch size for bulk operations
    pub batch_size: usize,
    /// Retention period in days
    pub retention_days: u32,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            store_type: "memory".to_string(), // Use memory store by default
            database_url: None,
            file_path: None,
            log_authentication: true,
            log_authorization: true,
            log_data_access: true,
            log_configuration_changes: true,
            log_security_violations: true,
            enable_integrity_verification: true,
            batch_size: 100,
            retention_days: 2555, // 7 years for compliance
        }
    }
}

/// API security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSecurityConfig {
    /// Enable API key authentication
    pub enable_api_keys: bool,
    /// API key expiration days
    pub api_key_expiry_days: Option<u32>,
    /// Enable request signing
    pub enable_request_signing: bool,
    /// Signing algorithm
    pub signing_algorithm: String,
    /// Enable CORS
    pub enable_cors: bool,
    /// Allowed origins for CORS
    pub cors_origins: Vec<String>,
    /// Enable CSRF protection
    pub enable_csrf: bool,
    /// CSRF token expiry minutes
    pub csrf_token_expiry_minutes: u32,
    /// Enable request validation
    pub enable_request_validation: bool,
    /// Maximum request size in bytes
    pub max_request_size: u64,
    /// Request timeout in seconds
    pub request_timeout_seconds: u32,
}

impl Default for ApiSecurityConfig {
    fn default() -> Self {
        Self {
            enable_api_keys: true,
            api_key_expiry_days: Some(365),
            enable_request_signing: false,
            signing_algorithm: "HS256".to_string(),
            enable_cors: true,
            cors_origins: vec!["*".to_string()],
            enable_csrf: true,
            csrf_token_expiry_minutes: 30,
            enable_request_validation: true,
            max_request_size: 10 * 1024 * 1024, // 10MB
            request_timeout_seconds: 30,
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitingConfig {
    /// Enable rate limiting
    pub enabled: bool,
    /// Default rate limit per IP (requests per minute)
    pub default_rate_limit: u32,
    /// Rate limit per authenticated user (requests per minute)
    pub user_rate_limit: u32,
    /// Rate limit per API key (requests per minute)
    pub api_key_rate_limit: u32,
    /// Rate limit window size in seconds
    pub window_size: u32,
    /// Rate limit burst size
    pub burst_size: u32,
    /// Enable distributed rate limiting
    pub distributed: bool,
    /// Redis URL for distributed rate limiting
    pub redis_url: Option<String>,
    /// Custom rate limits per endpoint
    pub endpoint_limits: HashMap<String, u32>,
}

impl Default for RateLimitingConfig {
    fn default() -> Self {
        let mut endpoint_limits = HashMap::new();
        endpoint_limits.insert("/api/auth/login".to_string(), 5); // 5 login attempts per minute
        endpoint_limits.insert("/api/auth/register".to_string(), 2); // 2 registration attempts per minute

        Self {
            enabled: true,
            default_rate_limit: 60,   // 60 requests per minute
            user_rate_limit: 300,     // 300 requests per minute for authenticated users
            api_key_rate_limit: 1000, // 1000 requests per minute for API keys
            window_size: 60,          // 1 minute window
            burst_size: 10,           // Allow bursts of 10 requests
            distributed: false,
            redis_url: None,
            endpoint_limits,
        }
    }
}

/// Secret management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretManagementConfig {
    /// Primary secret store type
    pub primary_store: String,
    /// Fallback secret store types
    pub fallback_stores: Vec<String>,
    /// Secret store configurations
    pub store_configs: HashMap<String, SecretStoreConfig>,
    /// Enable secret rotation
    pub enable_rotation: bool,
    /// Secret rotation interval in days
    pub rotation_interval_days: u32,
    /// Enable secret encryption at rest
    pub enable_encryption_at_rest: bool,
    /// Master key for encryption
    pub master_key: Option<String>,
}

impl Default for SecretManagementConfig {
    fn default() -> Self {
        let mut store_configs = HashMap::new();
        store_configs.insert("env".to_string(), SecretStoreConfig::default());
        store_configs.insert("memory".to_string(), SecretStoreConfig::default());

        Self {
            primary_store: "env".to_string(),
            fallback_stores: vec!["memory".to_string()],
            store_configs,
            enable_rotation: false,
            rotation_interval_days: 90,
            enable_encryption_at_rest: true,
            master_key: None,
        }
    }
}

/// Security configuration loader
pub struct SecurityConfigLoader {
    config_builder: config::ConfigBuilder<config::builder::DefaultState>,
}

impl SecurityConfigLoader {
    /// Create a new security configuration loader
    pub fn new() -> Self {
        Self {
            config_builder: Config::builder(),
        }
    }

    /// Load configuration from default locations
    pub fn load_default() -> SecurityResult<SecurityConfig> {
        let loader = Self::new();

        // Load from default file locations
        let loader = loader.add_file("config/security/default", false)?;

        // Load environment-specific configuration
        let env = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
        let loader = loader.add_file(&format!("config/security/{}", env), true)?;

        // Load from environment variables
        let mut loader = loader;
        loader.add_environment("UVEDDI_SECURITY")?;
        loader.build()
    }

    /// Add a configuration file
    pub fn add_file(mut self, path: &str, optional: bool) -> SecurityResult<Self> {
        let file_source = File::with_name(path).required(!optional);
        self.config_builder = self.config_builder.add_source(file_source);
        Ok(self)
    }

    /// Add environment variable source
    pub fn add_environment(&mut self, prefix: &str) -> SecurityResult<()> {
        let env_source = Environment::with_prefix(prefix)
            .prefix_separator("_")
            .separator("__");
        self.config_builder = self.config_builder.clone().add_source(env_source);
        Ok(())
    }

    /// Add a custom configuration source
    pub fn add_source<T>(mut self, source: T) -> SecurityResult<Self>
    where
        T: Source + Send + Sync + 'static,
    {
        self.config_builder = self.config_builder.add_source(source);
        Ok(self)
    }

    /// Build the final configuration
    pub fn build(self) -> SecurityResult<SecurityConfig> {
        let config =
            self.config_builder
                .build()
                .map_err(|e| SecurityError::ConfigurationError {
                    message: format!("Failed to build configuration: {}", e),
                })?;

        let security_config = config.try_deserialize::<SecurityConfig>().map_err(|e| {
            SecurityError::ConfigurationError {
                message: format!("Failed to deserialize security configuration: {}", e),
            }
        })?;

        // Validate configuration
        security_config.validate()?;

        Ok(security_config)
    }
}

impl Default for SecurityConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl SecurityConfig {
    /// Load security configuration from files and environment
    pub fn load() -> SecurityResult<Self> {
        SecurityConfigLoader::load_default()
    }

    /// Load security configuration from a specific file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> SecurityResult<Self> {
        let loader = SecurityConfigLoader::new();
        let loader = loader.add_file(path.as_ref().to_str().unwrap(), false)?;
        loader.build()
    }

    /// Load security configuration from environment variables
    pub fn load_from_env(prefix: &str) -> SecurityResult<Self> {
        let loader = SecurityConfigLoader::new();
        let mut loader = loader;
        loader.add_environment(prefix)?;
        loader.build()
    }

    /// Validate the security configuration
    pub fn validate(&self) -> SecurityResult<()> {
        // Validate authentication configuration
        self.validate_authentication()?;

        // Validate authorization configuration
        self.validate_authorization()?;

        // Validate audit configuration
        self.validate_audit()?;

        // Validate API security configuration
        self.validate_api_security()?;

        // Validate rate limiting configuration
        self.validate_rate_limiting()?;

        // Validate secret management configuration
        self.validate_secret_management()?;

        Ok(())
    }

    /// Validate authentication configuration
    fn validate_authentication(&self) -> SecurityResult<()> {
        if self.authentication.jwt_secret.is_empty() {
            return Err(SecurityError::ConfigurationError {
                message: "JWT secret cannot be empty".to_string(),
            });
        }

        if self.authentication.jwt_secret.len() < 32 {
            return Err(SecurityError::ConfigurationError {
                message: "JWT secret must be at least 32 characters long".to_string(),
            });
        }

        if self.authentication.jwt_expiry_hours == 0 {
            return Err(SecurityError::ConfigurationError {
                message: "JWT expiry hours must be greater than 0".to_string(),
            });
        }

        if self.authentication.session_expiry_hours == 0 {
            return Err(SecurityError::ConfigurationError {
                message: "Session expiry hours must be greater than 0".to_string(),
            });
        }

        // Validate OAuth providers
        for provider in &self.authentication.oauth_providers {
            if provider.client_id.is_empty() {
                return Err(SecurityError::ConfigurationError {
                    message: format!(
                        "OAuth provider '{}' client ID cannot be empty",
                        provider.provider_name
                    ),
                });
            }

            if provider.client_secret.is_empty() {
                return Err(SecurityError::ConfigurationError {
                    message: format!(
                        "OAuth provider '{}' client secret cannot be empty",
                        provider.provider_name
                    ),
                });
            }

            if provider.auth_url.is_empty() {
                return Err(SecurityError::ConfigurationError {
                    message: format!(
                        "OAuth provider '{}' auth URL cannot be empty",
                        provider.provider_name
                    ),
                });
            }

            if provider.token_url.is_empty() {
                return Err(SecurityError::ConfigurationError {
                    message: format!(
                        "OAuth provider '{}' token URL cannot be empty",
                        provider.provider_name
                    ),
                });
            }
        }

        // Validate OIDC providers
        for provider in &self.authentication.oidc_providers {
            if provider.client_id.is_empty() {
                return Err(SecurityError::ConfigurationError {
                    message: format!(
                        "OIDC provider '{}' client ID cannot be empty",
                        provider.provider_name
                    ),
                });
            }

            if provider.client_secret.is_empty() {
                return Err(SecurityError::ConfigurationError {
                    message: format!(
                        "OIDC provider '{}' client secret cannot be empty",
                        provider.provider_name
                    ),
                });
            }

            if provider.issuer_url.is_empty() {
                return Err(SecurityError::ConfigurationError {
                    message: format!(
                        "OIDC provider '{}' issuer URL cannot be empty",
                        provider.provider_name
                    ),
                });
            }
        }

        Ok(())
    }

    /// Validate authorization configuration
    fn validate_authorization(&self) -> SecurityResult<()> {
        if self.authorization.role_cache_ttl == 0 {
            return Err(SecurityError::ConfigurationError {
                message: "Role cache TTL must be greater than 0".to_string(),
            });
        }

        if self.authorization.permission_cache_ttl == 0 {
            return Err(SecurityError::ConfigurationError {
                message: "Permission cache TTL must be greater than 0".to_string(),
            });
        }

        if self.authorization.default_role.is_empty() {
            return Err(SecurityError::ConfigurationError {
                message: "Default role cannot be empty".to_string(),
            });
        }

        // Validate default role is a valid role
        let valid_roles = ["Admin", "Developer", "QA", "Manager", "Service"];
        if !valid_roles.contains(&self.authorization.default_role.as_str()) {
            return Err(SecurityError::ConfigurationError {
                message: format!("Invalid default role: {}", self.authorization.default_role),
            });
        }

        Ok(())
    }

    /// Validate audit configuration
    fn validate_audit(&self) -> SecurityResult<()> {
        if self.audit.enabled {
            match self.audit.store_type.as_str() {
                "memory" => {
                    // Memory store is always valid
                }
                "database" => {
                    if self.audit.database_url.is_none() {
                        return Err(SecurityError::ConfigurationError {
                            message: "Database URL is required for database audit store"
                                .to_string(),
                        });
                    }
                }
                "file" => {
                    if self.audit.file_path.is_none() {
                        return Err(SecurityError::ConfigurationError {
                            message: "File path is required for file audit store".to_string(),
                        });
                    }
                }
                _ => {
                    return Err(SecurityError::ConfigurationError {
                        message: format!("Invalid audit store type: {}", self.audit.store_type),
                    });
                }
            }

            if self.audit.batch_size == 0 {
                return Err(SecurityError::ConfigurationError {
                    message: "Audit batch size must be greater than 0".to_string(),
                });
            }

            if self.audit.retention_days == 0 {
                return Err(SecurityError::ConfigurationError {
                    message: "Audit retention days must be greater than 0".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Validate API security configuration
    fn validate_api_security(&self) -> SecurityResult<()> {
        if self.api_security.max_request_size == 0 {
            return Err(SecurityError::ConfigurationError {
                message: "Maximum request size must be greater than 0".to_string(),
            });
        }

        if self.api_security.request_timeout_seconds == 0 {
            return Err(SecurityError::ConfigurationError {
                message: "Request timeout must be greater than 0".to_string(),
            });
        }

        if self.api_security.csrf_token_expiry_minutes == 0 {
            return Err(SecurityError::ConfigurationError {
                message: "CSRF token expiry must be greater than 0".to_string(),
            });
        }

        // Validate signing algorithm
        if self.api_security.enable_request_signing {
            let valid_algorithms = ["HS256", "HS384", "HS512", "RS256", "RS384", "RS512"];
            if !valid_algorithms.contains(&self.api_security.signing_algorithm.as_str()) {
                return Err(SecurityError::ConfigurationError {
                    message: format!(
                        "Invalid signing algorithm: {}",
                        self.api_security.signing_algorithm
                    ),
                });
            }
        }

        Ok(())
    }

    /// Validate rate limiting configuration
    fn validate_rate_limiting(&self) -> SecurityResult<()> {
        if self.rate_limiting.enabled {
            if self.rate_limiting.default_rate_limit == 0 {
                return Err(SecurityError::ConfigurationError {
                    message: "Default rate limit must be greater than 0".to_string(),
                });
            }

            if self.rate_limiting.user_rate_limit == 0 {
                return Err(SecurityError::ConfigurationError {
                    message: "User rate limit must be greater than 0".to_string(),
                });
            }

            if self.rate_limiting.api_key_rate_limit == 0 {
                return Err(SecurityError::ConfigurationError {
                    message: "API key rate limit must be greater than 0".to_string(),
                });
            }

            if self.rate_limiting.window_size == 0 {
                return Err(SecurityError::ConfigurationError {
                    message: "Rate limit window size must be greater than 0".to_string(),
                });
            }

            if self.rate_limiting.burst_size == 0 {
                return Err(SecurityError::ConfigurationError {
                    message: "Rate limit burst size must be greater than 0".to_string(),
                });
            }

            if self.rate_limiting.distributed && self.rate_limiting.redis_url.is_none() {
                return Err(SecurityError::ConfigurationError {
                    message: "Redis URL is required for distributed rate limiting".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Validate secret management configuration
    fn validate_secret_management(&self) -> SecurityResult<()> {
        if self.secrets.primary_store.is_empty() {
            return Err(SecurityError::ConfigurationError {
                message: "Primary secret store cannot be empty".to_string(),
            });
        }

        let valid_stores = ["env", "memory", "vault", "aws"];
        if !valid_stores.contains(&self.secrets.primary_store.as_str()) {
            return Err(SecurityError::ConfigurationError {
                message: format!(
                    "Invalid primary secret store: {}",
                    self.secrets.primary_store
                ),
            });
        }

        for store in &self.secrets.fallback_stores {
            if !valid_stores.contains(&store.as_str()) {
                return Err(SecurityError::ConfigurationError {
                    message: format!("Invalid fallback secret store: {}", store),
                });
            }
        }

        if self.secrets.enable_rotation && self.secrets.rotation_interval_days == 0 {
            return Err(SecurityError::ConfigurationError {
                message: "Secret rotation interval must be greater than 0".to_string(),
            });
        }

        Ok(())
    }

    /// Get configuration as JSON string
    pub fn to_json(&self) -> SecurityResult<String> {
        serde_json::to_string_pretty(self).map_err(|e| SecurityError::ConfigurationError {
            message: format!("Failed to serialize configuration to JSON: {}", e),
        })
    }

    /// Get configuration as TOML string
    pub fn to_toml(&self) -> SecurityResult<String> {
        toml::to_string_pretty(self).map_err(|e| SecurityError::ConfigurationError {
            message: format!("Failed to serialize configuration to TOML: {}", e),
        })
    }

    /// Save configuration to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> SecurityResult<()> {
        let content = if path.as_ref().extension().and_then(|ext| ext.to_str()) == Some("json") {
            self.to_json()?
        } else {
            self.to_toml()?
        };

        std::fs::write(path, content).map_err(|e| SecurityError::ConfigurationError {
            message: format!("Failed to write configuration file: {}", e),
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_default_security_config() {
        let config = SecurityConfig::default();
        assert!(config.validate().is_ok());
        assert!(config.authentication.jwt_secret.len() >= 32);
        assert!(config.authorization.enabled);
        assert!(config.audit.enabled);
        assert!(config.api_security.enable_api_keys);
        assert!(config.rate_limiting.enabled);
        // Verify audit uses memory store by default
        assert_eq!(config.audit.store_type, "memory");
    }

    #[test]
    fn test_config_validation() {
        let mut config = SecurityConfig::default();

        // Test invalid JWT secret
        config.authentication.jwt_secret = "short".to_string();
        assert!(config.validate().is_err());

        // Fix JWT secret
        config.authentication.jwt_secret = "this-is-a-very-long-jwt-secret-for-testing".to_string();
        assert!(config.validate().is_ok());

        // Test invalid default role
        config.authorization.default_role = "InvalidRole".to_string();
        assert!(config.validate().is_err());

        // Fix default role
        config.authorization.default_role = "Developer".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_serialization() {
        let config = SecurityConfig::default();

        // Test JSON serialization
        let json = config.to_json().unwrap();
        assert!(json.contains("authentication"));
        assert!(json.contains("authorization"));
        assert!(json.contains("audit"));

        // Test TOML serialization
        let toml = config.to_toml().unwrap();
        assert!(toml.contains("[authentication]"));
        assert!(toml.contains("[authorization]"));
        assert!(toml.contains("[audit]"));
    }

    #[test]
    fn test_config_file_operations() {
        let config = SecurityConfig::default();
        let temp_dir = tempdir().unwrap();

        // Test saving to JSON file
        let json_path = temp_dir.path().join("config.json");
        config.save_to_file(&json_path).unwrap();
        assert!(json_path.exists());

        // Test saving to TOML file
        let toml_path = temp_dir.path().join("config.toml");
        config.save_to_file(&toml_path).unwrap();
        assert!(toml_path.exists());

        // Test loading from file
        let loaded_config = SecurityConfig::load_from_file(&toml_path).unwrap();
        assert_eq!(
            loaded_config.authentication.jwt_expiry_hours,
            config.authentication.jwt_expiry_hours
        );
        // Verify the loaded config is also valid
        assert!(loaded_config.validate().is_ok());
    }

    #[test]
    fn test_config_builder() {
        // Test with default configuration as baseline
        let temp_dir = tempdir().unwrap();
        let default_config_path = temp_dir.path().join("default.toml");
        let default_config = SecurityConfig::default();
        default_config.save_to_file(&default_config_path).unwrap();

        let mut loader = SecurityConfigLoader::new();
        loader = loader
            .add_file(default_config_path.to_str().unwrap(), false)
            .unwrap();

        // Test adding environment variables as overrides
        std::env::set_var("UVEDDI_SECURITY_AUTHENTICATION__JWT_EXPIRY_HOURS", "48");
        std::env::set_var("UVEDDI_SECURITY_AUTHORIZATION__ENABLED", "false");

        loader.add_environment("UVEDDI_SECURITY").unwrap();
        let config = loader.build().unwrap();

        assert_eq!(config.authentication.jwt_expiry_hours, 48);
        assert!(!config.authorization.enabled);

        // Clean up
        std::env::remove_var("UVEDDI_SECURITY_AUTHENTICATION__JWT_EXPIRY_HOURS");
        std::env::remove_var("UVEDDI_SECURITY_AUTHORIZATION__ENABLED");
    }

    #[test]
    fn test_oauth_provider_validation() {
        let mut config = SecurityConfig::default();

        // Add invalid OAuth provider
        config
            .authentication
            .oauth_providers
            .push(OAuthProviderConfig {
                provider_name: "test".to_string(),
                client_id: "".to_string(), // Empty client ID
                client_secret: "secret".to_string(),
                auth_url: "https://example.com/auth".to_string(),
                token_url: "https://example.com/token".to_string(),
                redirect_url: "https://example.com/callback".to_string(),
                scopes: vec!["profile".to_string()],
            });

        assert!(config.validate().is_err());

        // Fix OAuth provider
        config.authentication.oauth_providers[0].client_id = "client123".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_oidc_provider_validation() {
        let mut config = SecurityConfig::default();

        // Add invalid OIDC provider
        config
            .authentication
            .oidc_providers
            .push(OidcProviderConfig {
                provider_name: "test".to_string(),
                issuer_url: "".to_string(), // Empty issuer URL
                client_id: "client123".to_string(),
                client_secret: "secret".to_string(),
                redirect_url: "https://example.com/callback".to_string(),
                scopes: vec!["openid".to_string()],
            });

        assert!(config.validate().is_err());

        // Fix OIDC provider
        config.authentication.oidc_providers[0].issuer_url =
            "https://example.com/.well-known/openid_configuration".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_rate_limiting_validation() {
        let mut config = SecurityConfig::default();

        // Test zero rate limit
        config.rate_limiting.default_rate_limit = 0;
        assert!(config.validate().is_err());

        // Fix rate limit
        config.rate_limiting.default_rate_limit = 60;
        assert!(config.validate().is_ok());

        // Test distributed rate limiting without Redis
        config.rate_limiting.distributed = true;
        config.rate_limiting.redis_url = None;
        assert!(config.validate().is_err());

        // Fix Redis URL
        config.rate_limiting.redis_url = Some("redis://localhost:6379".to_string());
        assert!(config.validate().is_ok());
    }
}
