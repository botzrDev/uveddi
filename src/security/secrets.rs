//! Secure Secret Management for Uveddi
//!
//! This module provides secure storage and retrieval of sensitive configuration
//! including API keys, database credentials, and encryption keys.

use crate::security::errors::{SecurityError, SecurityResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Trait for secure secret storage implementations
#[async_trait]
pub trait SecretStore: Send + Sync {
    /// Get a secret by key
    async fn get_secret(&self, key: &str) -> SecurityResult<String>;
    
    /// Set a secret by key
    async fn set_secret(&self, key: &str, value: &str) -> SecurityResult<()>;
    
    /// Delete a secret by key
    async fn delete_secret(&self, key: &str) -> SecurityResult<()>;
    
    /// List all secret keys (not values)
    async fn list_secret_keys(&self) -> SecurityResult<Vec<String>>;
    
    /// Check if a secret exists
    async fn secret_exists(&self, key: &str) -> SecurityResult<bool>;
}

/// Environment variable secret store
pub struct EnvSecretStore {
    prefix: String,
}

impl EnvSecretStore {
    /// Create a new environment variable secret store
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
        }
    }
    
    /// Get the full environment variable name
    fn get_env_var_name(&self, key: &str) -> String {
        format!("{}_{}", self.prefix, key.to_uppercase())
    }
}

#[async_trait]
impl SecretStore for EnvSecretStore {
    async fn get_secret(&self, key: &str) -> SecurityResult<String> {
        let env_var = self.get_env_var_name(key);
        std::env::var(&env_var).map_err(|_| SecurityError::SecretNotFound {
            key: key.to_string(),
        })
    }
    
    async fn set_secret(&self, key: &str, value: &str) -> SecurityResult<()> {
        let env_var = self.get_env_var_name(key);
        std::env::set_var(&env_var, value);
        Ok(())
    }
    
    async fn delete_secret(&self, key: &str) -> SecurityResult<()> {
        let env_var = self.get_env_var_name(key);
        std::env::remove_var(&env_var);
        Ok(())
    }
    
    async fn list_secret_keys(&self) -> SecurityResult<Vec<String>> {
        let prefix = format!("{}_", self.prefix);
        let keys: Vec<String> = std::env::vars()
            .filter_map(|(key, _)| {
                if key.starts_with(&prefix) {
                    Some(key[prefix.len()..].to_lowercase())
                } else {
                    None
                }
            })
            .collect();
        Ok(keys)
    }
    
    async fn secret_exists(&self, key: &str) -> SecurityResult<bool> {
        let env_var = self.get_env_var_name(key);
        Ok(std::env::var(&env_var).is_ok())
    }
}

/// In-memory secret store (for testing and development)
pub struct InMemorySecretStore {
    secrets: Arc<RwLock<HashMap<String, String>>>,
}

impl InMemorySecretStore {
    /// Create a new in-memory secret store
    pub fn new() -> Self {
        Self {
            secrets: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Create a new in-memory secret store with initial secrets
    pub fn with_secrets(initial_secrets: HashMap<String, String>) -> Self {
        Self {
            secrets: Arc::new(RwLock::new(initial_secrets)),
        }
    }
}

impl Default for InMemorySecretStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SecretStore for InMemorySecretStore {
    async fn get_secret(&self, key: &str) -> SecurityResult<String> {
        let secrets = self.secrets.read().await;
        secrets.get(key).cloned().ok_or_else(|| SecurityError::SecretNotFound {
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

/// HashiCorp Vault secret store
pub struct VaultSecretStore {
    client: vault::Client,
    mount_path: String,
}

impl VaultSecretStore {
    /// Create a new Vault secret store
    pub async fn new(vault_url: &str, vault_token: &str, mount_path: &str) -> SecurityResult<Self> {
        let client = vault::Client::new(vault_url, vault_token)
            .map_err(|e| SecurityError::SecretStoreUnavailable {
                store_type: "HashiCorp Vault".to_string(),
            })?;
        
        Ok(Self {
            client,
            mount_path: mount_path.to_string(),
        })
    }
    
    /// Get the full secret path
    fn get_secret_path(&self, key: &str) -> String {
        format!("{}/{}", self.mount_path, key)
    }
}

#[async_trait]
impl SecretStore for VaultSecretStore {
    async fn get_secret(&self, key: &str) -> SecurityResult<String> {
        let path = self.get_secret_path(key);
        let secret = self.client.get_secret(&path).await
            .map_err(|e| SecurityError::SecretOperationError {
                operation: "get".to_string(),
                error: e.to_string(),
            })?;
        
        secret.get("value")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| SecurityError::SecretNotFound {
                key: key.to_string(),
            })
    }
    
    async fn set_secret(&self, key: &str, value: &str) -> SecurityResult<()> {
        let path = self.get_secret_path(key);
        let mut data = std::collections::HashMap::new();
        data.insert("value".to_string(), serde_json::Value::String(value.to_string()));
        
        self.client.set_secret(&path, &data).await
            .map_err(|e| SecurityError::SecretOperationError {
                operation: "set".to_string(),
                error: e.to_string(),
            })?;
        
        Ok(())
    }
    
    async fn delete_secret(&self, key: &str) -> SecurityResult<()> {
        let path = self.get_secret_path(key);
        self.client.delete_secret(&path).await
            .map_err(|e| SecurityError::SecretOperationError {
                operation: "delete".to_string(),
                error: e.to_string(),
            })?;
        
        Ok(())
    }
    
    async fn list_secret_keys(&self) -> SecurityResult<Vec<String>> {
        let keys = self.client.list_secrets(&self.mount_path).await
            .map_err(|e| SecurityError::SecretOperationError {
                operation: "list".to_string(),
                error: e.to_string(),
            })?;
        
        Ok(keys)
    }
    
    async fn secret_exists(&self, key: &str) -> SecurityResult<bool> {
        let path = self.get_secret_path(key);
        match self.client.get_secret(&path).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

/// AWS Secrets Manager secret store (placeholder - requires aws-sdk-secretsmanager)
pub struct AwsSecretsManagerStore {
    secret_prefix: String,
}

impl AwsSecretsManagerStore {
    /// Create a new AWS Secrets Manager secret store
    pub async fn new(_region: &str, secret_prefix: &str) -> SecurityResult<Self> {
        Ok(Self {
            secret_prefix: secret_prefix.to_string(),
        })
    }
    
    /// Get the full secret name
    fn get_secret_name(&self, key: &str) -> String {
        format!("{}/{}", self.secret_prefix, key)
    }
}

#[async_trait]
impl SecretStore for AwsSecretsManagerStore {
    async fn get_secret(&self, key: &str) -> SecurityResult<String> {
        // Placeholder implementation - would integrate with AWS SDK
        Err(SecurityError::SecretNotFound {
            key: key.to_string(),
        })
    }
    
    async fn set_secret(&self, _key: &str, _value: &str) -> SecurityResult<()> {
        // Placeholder implementation - would integrate with AWS SDK
        Ok(())
    }
    
    async fn delete_secret(&self, _key: &str) -> SecurityResult<()> {
        // Placeholder implementation - would integrate with AWS SDK
        Ok(())
    }
    
    async fn list_secret_keys(&self) -> SecurityResult<Vec<String>> {
        // Placeholder implementation - would integrate with AWS SDK
        Ok(Vec::new())
    }
    
    async fn secret_exists(&self, _key: &str) -> SecurityResult<bool> {
        // Placeholder implementation - would integrate with AWS SDK
        Ok(false)
    }
}

/// Composite secret store that tries multiple stores in order
pub struct CompositeSecretStore {
    stores: Vec<Arc<dyn SecretStore>>,
}

impl CompositeSecretStore {
    /// Create a new composite secret store
    pub fn new(stores: Vec<Arc<dyn SecretStore>>) -> Self {
        Self { stores }
    }
    
    /// Add a secret store to the composite
    pub fn add_store(&mut self, store: Arc<dyn SecretStore>) {
        self.stores.push(store);
    }
}

#[async_trait]
impl SecretStore for CompositeSecretStore {
    async fn get_secret(&self, key: &str) -> SecurityResult<String> {
        for store in &self.stores {
            if let Ok(secret) = store.get_secret(key).await {
                return Ok(secret);
            }
        }
        
        Err(SecurityError::SecretNotFound {
            key: key.to_string(),
        })
    }
    
    async fn set_secret(&self, key: &str, value: &str) -> SecurityResult<()> {
        // Set in the first store that supports it
        for store in &self.stores {
            if let Ok(()) = store.set_secret(key, value).await {
                return Ok(());
            }
        }
        
        Err(SecurityError::SecretOperationError {
            operation: "set".to_string(),
            error: "No store available for setting secrets".to_string(),
        })
    }
    
    async fn delete_secret(&self, key: &str) -> SecurityResult<()> {
        // Delete from all stores
        let mut last_error = None;
        for store in &self.stores {
            if let Err(e) = store.delete_secret(key).await {
                last_error = Some(e);
            }
        }
        
        if let Some(error) = last_error {
            Err(error)
        } else {
            Ok(())
        }
    }
    
    async fn list_secret_keys(&self) -> SecurityResult<Vec<String>> {
        let mut all_keys = std::collections::HashSet::new();
        
        for store in &self.stores {
            if let Ok(keys) = store.list_secret_keys().await {
                all_keys.extend(keys);
            }
        }
        
        Ok(all_keys.into_iter().collect())
    }
    
    async fn secret_exists(&self, key: &str) -> SecurityResult<bool> {
        for store in &self.stores {
            if let Ok(exists) = store.secret_exists(key).await {
                if exists {
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }
}

/// Secret store factory for creating appropriate stores based on configuration
pub struct SecretStoreFactory;

impl SecretStoreFactory {
    /// Create a secret store based on configuration
    pub async fn create_store(store_type: &str, config: &SecretStoreConfig) -> SecurityResult<Arc<dyn SecretStore>> {
        match store_type {
            "env" => {
                let prefix = config.env_prefix.as_deref().unwrap_or("UVEDDI_SECRET");
                Ok(Arc::new(EnvSecretStore::new(prefix)))
            }
            "memory" => {
                Ok(Arc::new(InMemorySecretStore::new()))
            }
            "vault" => {
                let vault_url = config.vault_url.as_ref().ok_or_else(|| SecurityError::MissingConfiguration {
                    key: "vault_url".to_string(),
                })?;
                let vault_token = config.vault_token.as_ref().ok_or_else(|| SecurityError::MissingConfiguration {
                    key: "vault_token".to_string(),
                })?;
                let mount_path = config.vault_mount_path.as_deref().unwrap_or("secret");
                
                Ok(Arc::new(VaultSecretStore::new(vault_url, vault_token, mount_path).await?))
            }
            "aws" => {
                let region = config.aws_region.as_deref().unwrap_or("us-east-1");
                let prefix = config.aws_secret_prefix.as_deref().unwrap_or("uveddi");
                
                Ok(Arc::new(AwsSecretsManagerStore::new(region, prefix).await?))
            }
            _ => Err(SecurityError::ConfigurationError {
                message: format!("Unknown secret store type: {}", store_type),
            })
        }
    }
    
    /// Create a composite store with multiple backends
    pub async fn create_composite_store(store_configs: Vec<(String, SecretStoreConfig)>) -> SecurityResult<Arc<dyn SecretStore>> {
        let mut stores = Vec::new();
        
        for (store_type, config) in store_configs {
            let store = Self::create_store(&store_type, &config).await?;
            stores.push(store);
        }
        
        Ok(Arc::new(CompositeSecretStore::new(stores)))
    }
}

/// Configuration for secret stores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretStoreConfig {
    pub env_prefix: Option<String>,
    pub vault_url: Option<String>,
    pub vault_token: Option<String>,
    pub vault_mount_path: Option<String>,
    pub aws_region: Option<String>,
    pub aws_secret_prefix: Option<String>,
}

impl Default for SecretStoreConfig {
    fn default() -> Self {
        Self {
            env_prefix: Some("UVEDDI_SECRET".to_string()),
            vault_url: None,
            vault_token: None,
            vault_mount_path: Some("secret".to_string()),
            aws_region: Some("us-east-1".to_string()),
            aws_secret_prefix: Some("uveddi".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_in_memory_secret_store() {
        let store = InMemorySecretStore::new();
        
        // Test setting and getting a secret
        store.set_secret("test_key", "test_value").await.unwrap();
        let value = store.get_secret("test_key").await.unwrap();
        assert_eq!(value, "test_value");
        
        // Test secret existence
        assert!(store.secret_exists("test_key").await.unwrap());
        assert!(!store.secret_exists("nonexistent").await.unwrap());
        
        // Test listing keys
        let keys = store.list_secret_keys().await.unwrap();
        assert!(keys.contains(&"test_key".to_string()));
        
        // Test deleting a secret
        store.delete_secret("test_key").await.unwrap();
        assert!(!store.secret_exists("test_key").await.unwrap());
    }
    
    #[tokio::test]
    async fn test_env_secret_store() {
        let store = EnvSecretStore::new("TEST");
        
        // Set environment variable
        std::env::set_var("TEST_MY_SECRET", "secret_value");
        
        // Test getting secret
        let value = store.get_secret("my_secret").await.unwrap();
        assert_eq!(value, "secret_value");
        
        // Test secret existence
        assert!(store.secret_exists("my_secret").await.unwrap());
        
        // Test listing keys
        let keys = store.list_secret_keys().await.unwrap();
        assert!(keys.contains(&"my_secret".to_string()));
        
        // Clean up
        std::env::remove_var("TEST_MY_SECRET");
    }
    
    #[tokio::test]
    async fn test_composite_secret_store() {
        let store1 = Arc::new(InMemorySecretStore::new()) as Arc<dyn SecretStore>;
        let store2 = Arc::new(InMemorySecretStore::new()) as Arc<dyn SecretStore>;
        
        // Set different secrets in each store
        store1.set_secret("secret1", "value1").await.unwrap();
        store2.set_secret("secret2", "value2").await.unwrap();
        
        let composite = CompositeSecretStore::new(vec![store1, store2]);
        
        // Should be able to get secrets from both stores
        assert_eq!(composite.get_secret("secret1").await.unwrap(), "value1");
        assert_eq!(composite.get_secret("secret2").await.unwrap(), "value2");
        
        // Should list all keys
        let keys = composite.list_secret_keys().await.unwrap();
        assert!(keys.contains(&"secret1".to_string()));
        assert!(keys.contains(&"secret2".to_string()));
    }
    
    #[tokio::test]
    async fn test_secret_store_factory() {
        let config = SecretStoreConfig::default();
        
        // Test creating memory store
        let store = SecretStoreFactory::create_store("memory", &config).await.unwrap();
        store.set_secret("test", "value").await.unwrap();
        assert_eq!(store.get_secret("test").await.unwrap(), "value");
        
        // Test creating env store
        let store = SecretStoreFactory::create_store("env", &config).await.unwrap();
        std::env::set_var("UVEDDI_SECRET_TEST", "env_value");
        assert_eq!(store.get_secret("test").await.unwrap(), "env_value");
        std::env::remove_var("UVEDDI_SECRET_TEST");
    }
}

// Mock implementations for testing
#[cfg(test)]
pub struct MockSecretStore {
    secrets: Arc<RwLock<HashMap<String, String>>>,
}

#[cfg(test)]
impl MockSecretStore {
    pub fn new() -> Self {
        let mut secrets = HashMap::new();
        secrets.insert("jwt_secret".to_string(), "test-jwt-secret".to_string());
        secrets.insert("database_url".to_string(), "sqlite://test.db".to_string());
        
        Self {
            secrets: Arc::new(RwLock::new(secrets)),
        }
    }
}

#[cfg(test)]
#[async_trait]
impl SecretStore for MockSecretStore {
    async fn get_secret(&self, key: &str) -> SecurityResult<String> {
        let secrets = self.secrets.read().await;
        secrets.get(key).cloned().ok_or_else(|| SecurityError::SecretNotFound {
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

// Placeholder implementations for external dependencies
mod vault {
    use super::*;
    
    pub struct Client;
    
    impl Client {
        pub fn new(_url: &str, _token: &str) -> Result<Self, Box<dyn std::error::Error>> {
            Ok(Self)
        }
        
        pub async fn get_secret(&self, _path: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
            let mut map = serde_json::Map::new();
            map.insert("value".to_string(), serde_json::Value::String("test_value".to_string()));
            Ok(serde_json::Value::Object(map))
        }
        
        pub async fn set_secret(&self, _path: &str, _data: &std::collections::HashMap<String, serde_json::Value>) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }
        
        pub async fn delete_secret(&self, _path: &str) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }
        
        pub async fn list_secrets(&self, _path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
            Ok(vec!["test_key".to_string()])
        }
    }
}

mod aws_config {
    pub fn from_env() -> AwsConfigBuilder {
        AwsConfigBuilder
    }
    
    pub struct AwsConfigBuilder;
    
    impl AwsConfigBuilder {
        pub fn region(self, _region: &str) -> Self {
            self
        }
        
        pub async fn load(self) -> AwsConfig {
            AwsConfig
        }
    }
    
    pub struct AwsConfig;
}

mod aws_sdk_secretsmanager {
    use super::*;
    
    pub struct Client;
    
    impl Client {
        pub fn new(_config: &aws_config::AwsConfig) -> Self {
            Self
        }
        
        pub fn get_secret_value(&self) -> GetSecretValueBuilder {
            GetSecretValueBuilder
        }
        
        pub fn update_secret(&self) -> UpdateSecretBuilder {
            UpdateSecretBuilder
        }
        
        pub fn create_secret(&self) -> CreateSecretBuilder {
            CreateSecretBuilder
        }
        
        pub fn delete_secret(&self) -> DeleteSecretBuilder {
            DeleteSecretBuilder
        }
        
        pub fn list_secrets(&self) -> ListSecretsBuilder {
            ListSecretsBuilder
        }
        
        pub fn describe_secret(&self) -> DescribeSecretBuilder {
            DescribeSecretBuilder
        }
    }
    
    pub struct GetSecretValueBuilder;
    impl GetSecretValueBuilder {
        pub fn secret_id(self, _id: &str) -> Self { self }
        pub async fn send(self) -> Result<GetSecretValueResponse, Box<dyn std::error::Error>> {
            Ok(GetSecretValueResponse)
        }
    }
    
    pub struct GetSecretValueResponse;
    impl GetSecretValueResponse {
        pub fn secret_string(&self) -> Option<&str> { Some("test_value") }
    }
    
    pub struct UpdateSecretBuilder;
    impl UpdateSecretBuilder {
        pub fn secret_id(self, _id: &str) -> Self { self }
        pub fn secret_string(self, _value: &str) -> Self { self }
        pub async fn send(self) -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
    }
    
    pub struct CreateSecretBuilder;
    impl CreateSecretBuilder {
        pub fn name(self, _name: &str) -> Self { self }
        pub fn secret_string(self, _value: &str) -> Self { self }
        pub async fn send(self) -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
    }
    
    pub struct DeleteSecretBuilder;
    impl DeleteSecretBuilder {
        pub fn secret_id(self, _id: &str) -> Self { self }
        pub fn force_delete_without_recovery(self, _force: bool) -> Self { self }
        pub async fn send(self) -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
    }
    
    pub struct ListSecretsBuilder;
    impl ListSecretsBuilder {
        pub async fn send(self) -> Result<ListSecretsResponse, Box<dyn std::error::Error>> {
            Ok(ListSecretsResponse)
        }
    }
    
    pub struct ListSecretsResponse;
    impl ListSecretsResponse {
        pub fn secret_list(&self) -> &[SecretListEntry] { &[] }
    }
    
    pub struct SecretListEntry;
    impl SecretListEntry {
        pub fn name(&self) -> Option<&str> { Some("test_secret") }
    }
    
    pub struct DescribeSecretBuilder;
    impl DescribeSecretBuilder {
        pub fn secret_id(self, _id: &str) -> Self { self }
        pub async fn send(self) -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
    }
}