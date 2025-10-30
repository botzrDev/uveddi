//! Service registry for dependency injection and lifecycle management
//!
//! This module provides a container pattern for managing service dependencies
//! and coordinating their lifecycles across the application.

use crate::error::UveddiError;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::traits::{HealthCheck, Service, ServiceHealth};

/// Wrapper for services to make them object-safe
pub struct ServiceWrapper {
    name: String,
    is_running: bool,
}

impl ServiceWrapper {
    pub fn new(name: String) -> Self {
        Self {
            name,
            is_running: false,
        }
    }

    pub async fn start(&mut self) -> Result<(), UveddiError> {
        self.is_running = true;
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), UveddiError> {
        self.is_running = false;
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Service registry providing dependency injection container functionality
pub struct ServiceRegistry {
    /// Registered services by name
    services: Arc<RwLock<HashMap<String, ServiceWrapper>>>,
    /// Service configurations
    configs: HashMap<String, serde_json::Value>,
    /// Service startup order
    startup_order: Vec<String>,
}

/// Builder for constructing and configuring the service registry
pub struct ServiceBuilder {
    registry: ServiceRegistry,
}

/// Service registration information
#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub is_running: bool,
    pub health: Option<ServiceHealth>,
    pub dependencies: Vec<String>,
}

impl ServiceRegistry {
    /// Create a new service registry
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            configs: HashMap::new(),
            startup_order: Vec::new(),
        }
    }

    /// Register a service with the registry
    pub async fn register_service(&mut self, name: String) -> Result<(), UveddiError> {
        let mut services = self.services.write().await;

        if services.contains_key(&name) {
            return Err(UveddiError::config_error(
                &format!("Service '{}' is already registered", name),
                "service registration",
            ));
        }

        services.insert(name.clone(), ServiceWrapper::new(name.clone()));
        self.startup_order.push(name);

        Ok(())
    }

    /// Get a reference to a registered service
    pub async fn get_service(&self, name: &str) -> Option<()> {
        let services = self.services.read().await;
        if services.contains_key(name) {
            Some(())
        } else {
            None
        }
    }

    /// Start all registered services in dependency order
    pub async fn start_all_services(&mut self) -> Result<(), UveddiError> {
        let service_names: Vec<String> = self.startup_order.clone();

        for name in service_names {
            self.start_service(&name).await.map_err(|e| {
                UveddiError::config_error(
                    &format!("Failed to start service '{}': {}", name, e),
                    "service startup",
                )
            })?;
        }

        Ok(())
    }

    /// Start a specific service
    pub async fn start_service(&mut self, name: &str) -> Result<(), UveddiError> {
        let mut services = self.services.write().await;

        if let Some(service) = services.get_mut(name) {
            service.start().await.map_err(|e| {
                UveddiError::config_error(
                    &format!("Failed to start service '{}': {}", name, e),
                    "service startup",
                )
            })
        } else {
            Err(UveddiError::config_error(
                &format!("Service '{}' not found", name),
                "service lookup",
            ))
        }
    }

    /// Stop all registered services in reverse dependency order
    pub async fn stop_all_services(&mut self) -> Result<(), UveddiError> {
        let service_names: Vec<String> = self.startup_order.iter().rev().cloned().collect();

        for name in service_names {
            if let Err(e) = self.stop_service(&name).await {
                tracing::warn!("Failed to stop service '{}': {}", name, e);
            }
        }

        Ok(())
    }

    /// Stop a specific service
    pub async fn stop_service(&mut self, name: &str) -> Result<(), UveddiError> {
        let mut services = self.services.write().await;

        if let Some(service) = services.get_mut(name) {
            service.stop().await.map_err(|e| {
                UveddiError::config_error(
                    &format!("Failed to stop service '{}': {}", name, e),
                    "service shutdown",
                )
            })
        } else {
            Err(UveddiError::config_error(
                &format!("Service '{}' not found", name),
                "service lookup",
            ))
        }
    }

    /// Get information about all registered services
    pub async fn get_service_info(&self) -> Vec<ServiceInfo> {
        let services = self.services.read().await;
        let mut info = Vec::new();

        for (name, service) in services.iter() {
            info.push(ServiceInfo {
                name: name.clone(),
                is_running: service.is_running(),
                health: Some(ServiceHealth::Healthy), // Simplified for now
                dependencies: Vec::new(),             // TODO: Implement dependency tracking
            });
        }

        info
    }

    /// Check if all services are healthy
    pub async fn are_all_services_healthy(&self) -> Result<bool, UveddiError> {
        let services = self.services.read().await;

        for (_name, service) in services.iter() {
            if !service.is_running() {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Set configuration for a service
    pub fn set_config(&mut self, service_name: String, config: serde_json::Value) {
        self.configs.insert(service_name, config);
    }

    /// Get configuration for a service
    pub fn get_config(&self, service_name: &str) -> Option<&serde_json::Value> {
        self.configs.get(service_name)
    }

    /// Clear all services and configurations
    pub async fn clear(&mut self) {
        self.stop_all_services().await.ok();
        let mut services = self.services.write().await;
        services.clear();
        self.configs.clear();
        self.startup_order.clear();
    }
}

impl ServiceBuilder {
    /// Create a new service builder
    pub fn new() -> Self {
        Self {
            registry: ServiceRegistry::new(),
        }
    }

    /// Add a service to the builder
    pub async fn with_service(mut self, name: String) -> Result<Self, UveddiError> {
        self.registry.register_service(name).await?;
        Ok(self)
    }

    /// Add configuration for a service
    pub fn with_config(mut self, service_name: String, config: serde_json::Value) -> Self {
        self.registry.set_config(service_name, config);
        self
    }

    /// Set the startup order for services
    pub fn with_startup_order(mut self, order: Vec<String>) -> Result<Self, UveddiError> {
        // Validate that all services in the order are registered
        for service_name in &order {
            if !self.registry.startup_order.contains(service_name) {
                return Err(UveddiError::config_error(
                    &format!(
                        "Service '{}' in startup order is not registered",
                        service_name
                    ),
                    "startup order validation",
                ));
            }
        }

        self.registry.startup_order = order;
        Ok(self)
    }

    /// Build the configured service registry
    pub async fn build(mut self) -> Result<ServiceRegistry, UveddiError> {
        // Start all services in the configured order
        self.registry.start_all_services().await?;
        Ok(self.registry)
    }

    /// Build the registry without starting services
    pub fn build_without_start(self) -> ServiceRegistry {
        self.registry
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ServiceBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ServiceRegistry {
    fn drop(&mut self) {
        // Note: We can't call async methods in Drop, so services will need to be
        // stopped manually before dropping the registry
        tracing::debug!("ServiceRegistry dropped");
    }
}

/// Convenience macro for registering services
#[macro_export]
macro_rules! register_service {
    ($registry:expr, $name:expr, $service:expr) => {
        $registry
            .register_service($name.to_string(), $service)
            .await?;
    };
}

/// Convenience macro for building a service registry
#[macro_export]
macro_rules! build_registry {
    ($($name:expr => $service:expr),*) => {
        {
            let mut builder = ServiceBuilder::new();
            $(
                builder = builder.with_service($name.to_string(), $service).await?;
            )*
            builder.build().await?
        }
    };
}
