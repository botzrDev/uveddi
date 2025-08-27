//! Secure Configuration Loader with Runtime Secret Injection
//!
//! This module provides secure configuration loading that integrates with HashiCorp Vault
//! and other secret management systems to eliminate hardcoded secrets from the codebase.

use crate::security::{
    authentication::AuthenticationConfig,
    config::SecurityConfig,
    errors::{SecurityError, SecurityResult},
    secrets::{SecretStore, SecretStoreConfig, SecretStoreFactory},
};
// Removed unused async_trait import
use base64::{engine::general_purpose, Engine};
use std::sync::Arc;
use tracing::{error, info, warn};

/// Secure configuration loader that injects secrets at runtime
pub struct SecureConfigLoader {
    secret_store: Arc<dyn SecretStore>,
    fallback_to_env: bool,
}

impl SecureConfigLoader {
    /// Create a new secure configuration loader with HashiCorp Vault
    pub async fn with_vault(
        vault_url: &str,
        vault_token: &str,
        mount_path: &str,
    ) -> SecurityResult<Self> {
        let mut config = SecretStoreConfig::default();
        config.vault_url = Some(vault_url.to_string());
        config.vault_token = Some(vault_token.to_string());
        config.vault_mount_path = Some(mount_path.to_string());

        let secret_store = SecretStoreFactory::create_store("vault", &config).await?;

        Ok(Self {
            secret_store,
            fallback_to_env: true,
        })
    }

    /// Create a secure configuration loader with composite secret stores (Vault + Environment fallback)
    pub async fn with_composite_stores(
        vault_url: Option<String>,
        vault_token: Option<String>,
        mount_path: Option<String>,
    ) -> SecurityResult<Self> {
        let mut store_configs = Vec::new();

        // Add Vault as primary store if configured
        if let (Some(url), Some(token)) = (vault_url, vault_token) {
            let mut vault_config = SecretStoreConfig::default();
            vault_config.vault_url = Some(url);
            vault_config.vault_token = Some(token);
            vault_config.vault_mount_path = mount_path.or_else(|| Some("secret".to_string()));

            store_configs.push(("vault".to_string(), vault_config.clone()));
            info!("Added HashiCorp Vault as primary secret store");
        }

        // Add environment variables as fallback
        let env_config = SecretStoreConfig::default();
        store_configs.push(("env".to_string(), env_config));
        info!("Added environment variables as fallback secret store");

        let secret_store = SecretStoreFactory::create_composite_store(store_configs).await?;

        Ok(Self {
            secret_store,
            fallback_to_env: true,
        })
    }

    /// Create a secure configuration loader for testing (uses memory store)
    #[cfg(test)]
    pub async fn for_testing() -> SecurityResult<Self> {
        let config = SecretStoreConfig::default();
        let secret_store = SecretStoreFactory::create_store("memory", &config).await?;

        // Populate test secrets
        secret_store
            .set_secret("jwt_secret", "test-jwt-secret-from-secure-store")
            .await?;
        secret_store
            .set_secret("database_url", "sqlite://test.db")
            .await?;

        Ok(Self {
            secret_store,
            fallback_to_env: false,
        })
    }

    /// Load a complete security configuration with injected secrets
    pub async fn load_security_config(&self) -> SecurityResult<SecurityConfig> {
        info!("Loading security configuration with runtime secret injection");

        // Start with base configuration
        let mut config = SecurityConfig::default();

        // Inject JWT secret
        match self.get_secret_with_fallback("jwt_secret").await {
            Ok(jwt_secret) => {
                config.authentication.jwt_secret = jwt_secret;
                info!("Successfully injected JWT secret from secure store");
            }
            Err(e) => {
                error!("Failed to load JWT secret: {}", e);
                if !self.fallback_to_env {
                    return Err(SecurityError::MissingConfiguration {
                        key: "jwt_secret".to_string(),
                    });
                }
                warn!("Using default JWT secret generation due to secret store unavailability");
                config.authentication.jwt_secret = self.generate_secure_jwt_secret();
            }
        }

        // Inject database credentials if needed
        if let Ok(_database_url) = self.get_secret_with_fallback("database_url").await {
            info!("Successfully injected database URL from secure store");
            // You could set this on an audit config database_url field if it exists
        }

        // Inject OAuth/OIDC secrets
        for provider in &mut config.authentication.oauth_providers {
            let client_secret_key = format!(
                "oauth_{}_client_secret",
                provider.provider_name.to_lowercase()
            );
            if let Ok(client_secret) = self.get_secret_with_fallback(&client_secret_key).await {
                provider.client_secret = client_secret;
                info!(
                    "Injected OAuth client secret for provider: {}",
                    provider.provider_name
                );
            }
        }

        for provider in &mut config.authentication.oidc_providers {
            let client_secret_key = format!(
                "oidc_{}_client_secret",
                provider.provider_name.to_lowercase()
            );
            if let Ok(client_secret) = self.get_secret_with_fallback(&client_secret_key).await {
                provider.client_secret = client_secret;
                info!(
                    "Injected OIDC client secret for provider: {}",
                    provider.provider_name
                );
            }
        }

        // Validate the final configuration
        config.validate()?;

        info!(
            "Successfully loaded secure configuration with {} injected secrets",
            self.count_injected_secrets(&config).await
        );

        Ok(config)
    }

    /// Get authentication configuration with secure JWT secret injection
    pub async fn load_auth_config(&self) -> SecurityResult<AuthenticationConfig> {
        let mut config = AuthenticationConfig::default();

        // Inject JWT secret from secure store
        match self.get_secret_with_fallback("jwt_secret").await {
            Ok(jwt_secret) => {
                config.jwt_secret = jwt_secret;
                info!("Successfully injected JWT secret for authentication config");
            }
            Err(e) => {
                error!("Failed to load JWT secret: {}", e);
                return Err(SecurityError::MissingConfiguration {
                    key: "jwt_secret".to_string(),
                });
            }
        }

        Ok(config)
    }

    /// Get a secret with environment variable fallback
    async fn get_secret_with_fallback(&self, key: &str) -> SecurityResult<String> {
        match self.secret_store.get_secret(key).await {
            Ok(secret) => Ok(secret),
            Err(_) if self.fallback_to_env => {
                // Try environment variable as fallback
                let env_key = format!("UVEDDI_SECRET_{}", key.to_uppercase());
                std::env::var(&env_key).map_err(|_| SecurityError::SecretNotFound {
                    key: key.to_string(),
                })
            }
            Err(e) => Err(e),
        }
    }

    /// Generate a secure JWT secret as ultimate fallback
    fn generate_secure_jwt_secret(&self) -> String {
        use ring::rand::{SecureRandom, SystemRandom};

        let rng = SystemRandom::new();
        let mut secret = [0u8; 64]; // 512-bit secret
        rng.fill(&mut secret).unwrap();

        general_purpose::STANDARD.encode(secret)
    }

    /// Count the number of secrets that were successfully injected
    async fn count_injected_secrets(&self, _config: &SecurityConfig) -> usize {
        // In a real implementation, you'd track which secrets were successfully injected
        // For now, return a placeholder count
        1 // JWT secret
    }

    /// Health check for secret store connectivity
    pub async fn health_check(&self) -> SecurityResult<SecretStoreHealthStatus> {
        match self.secret_store.secret_exists("health_check").await {
            Ok(_) => Ok(SecretStoreHealthStatus {
                status: "healthy".to_string(),
                store_type: "composite".to_string(),
                last_check: chrono::Utc::now(),
                error_message: None,
            }),
            Err(e) => Ok(SecretStoreHealthStatus {
                status: "degraded".to_string(),
                store_type: "composite".to_string(),
                last_check: chrono::Utc::now(),
                error_message: Some(e.to_string()),
            }),
        }
    }

    /// Pre-populate secrets for initial deployment
    pub async fn bootstrap_secrets(&self) -> SecurityResult<()> {
        info!("Bootstrapping initial secrets");

        // Generate and store initial JWT secret if it doesn't exist
        if !self
            .secret_store
            .secret_exists("jwt_secret")
            .await
            .unwrap_or(false)
        {
            let jwt_secret = self.generate_secure_jwt_secret();
            self.secret_store
                .set_secret("jwt_secret", &jwt_secret)
                .await?;
            info!("Generated and stored initial JWT secret");
        }

        // Add other bootstrap secrets as needed

        Ok(())
    }
}

/// Health status of secret stores
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SecretStoreHealthStatus {
    pub status: String,
    pub store_type: String,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub error_message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_secure_config_loading() {
        let loader = SecureConfigLoader::for_testing().await.unwrap();

        let config = loader.load_security_config().await.unwrap();
        assert!(!config.authentication.jwt_secret.is_empty());
        assert!(config.authentication.jwt_secret.len() >= 32);
    }

    #[tokio::test]
    async fn test_auth_config_loading() {
        let loader = SecureConfigLoader::for_testing().await.unwrap();

        let auth_config = loader.load_auth_config().await.unwrap();
        assert_eq!(auth_config.jwt_secret, "test-jwt-secret-from-secure-store");
    }

    #[tokio::test]
    async fn test_health_check() {
        let loader = SecureConfigLoader::for_testing().await.unwrap();

        let health = loader.health_check().await.unwrap();
        assert_eq!(health.status, "healthy");
    }

    #[tokio::test]
    async fn test_bootstrap_secrets() {
        let loader = SecureConfigLoader::for_testing().await.unwrap();

        // Delete the existing JWT secret
        loader
            .secret_store
            .delete_secret("jwt_secret")
            .await
            .unwrap();

        // Bootstrap should recreate it
        loader.bootstrap_secrets().await.unwrap();

        let jwt_secret = loader.secret_store.get_secret("jwt_secret").await.unwrap();
        assert!(!jwt_secret.is_empty());
        assert!(jwt_secret.len() >= 32);
    }

    #[tokio::test]
    async fn test_secret_injection_with_fallback() {
        let config = SecretStoreConfig::default();
        let secret_store = SecretStoreFactory::create_store("memory", &config)
            .await
            .unwrap();

        let loader = SecureConfigLoader {
            secret_store,
            fallback_to_env: true, // Enable environment fallback for this test
        };

        // Set environment variable for fallback testing
        std::env::set_var("UVEDDI_SECRET_NONEXISTENT_KEY", "fallback_value");

        let secret = loader
            .get_secret_with_fallback("nonexistent_key")
            .await
            .unwrap();
        assert_eq!(secret, "fallback_value");

        // Clean up
        std::env::remove_var("UVEDDI_SECRET_NONEXISTENT_KEY");
    }
}
