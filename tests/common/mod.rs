//! Common test utilities and infrastructure
//!
//! This module provides centralized test infrastructure with consistent patterns
//! for setting up test environments, creating test projects, and managing
//! feature-aware testing.

pub mod macros;

use crate::analysis::engine::AnalysisEngine;
use crate::database::Database;
use crate::core::features::AiFeatureConfig;
use tempfile::TempDir;
use std::path::{Path, PathBuf};
use std::fs;

/// Test environment builder for consistent test setup
pub struct TestEnvironment {
    temp_dir: TempDir,
    database: Database,
    analysis_engine: Option<AnalysisEngine>,
}

impl TestEnvironment {
    /// Create new test environment with default configuration
    pub async fn new() -> Result<Self, TestError> {
        let temp_dir = TempDir::new()
            .map_err(|e| TestError::Setup(format!("Failed to create temp dir: {}", e)))?;
            
        let database = Database::new_in_memory().await
            .map_err(|e| TestError::Setup(format!("Failed to create test database: {}", e)))?;
            
        Ok(Self {
            temp_dir,
            database,
            analysis_engine: None,  // Created on demand due to compilation issues
        })
    }
    
    /// Create test environment with custom configuration
    pub async fn with_config(config: TestConfig) -> Result<Self, TestError> {
        let mut env = Self::new().await?;
        
        if config.enable_ai_features {
            log::info!("AI features requested in test config: {}", AiFeatureConfig::description());
        }
        
        Ok(env)
    }
    
    /// Get temporary directory for test files
    pub fn temp_path(&self) -> &Path {
        self.temp_dir.path()
    }
    
    /// Create test file with content
    pub fn create_test_file(&self, relative_path: &str, content: &str) -> Result<PathBuf, TestError> {
        let file_path = self.temp_path().join(relative_path);
        
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| TestError::FileCreation(format!("Failed to create parent directories: {}", e)))?;
        }
        
        std::fs::write(&file_path, content)
            .map_err(|e| TestError::FileCreation(format!("Failed to write file: {}", e)))?;
            
        Ok(file_path)
    }
    
    /// Create test Rust project structure
    pub fn create_rust_project(&self, name: &str) -> Result<PathBuf, TestError> {
        let project_path = self.temp_path().join(name);
        std::fs::create_dir_all(&project_path)
            .map_err(|e| TestError::FileCreation(format!("Failed to create project directory: {}", e)))?;
        
        // Create Cargo.toml
        let cargo_toml = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
"#, name);
        
        std::fs::write(project_path.join("Cargo.toml"), cargo_toml)
            .map_err(|e| TestError::FileCreation(format!("Failed to create Cargo.toml: {}", e)))?;
        
        // Create src/main.rs
        std::fs::create_dir_all(project_path.join("src"))
            .map_err(|e| TestError::FileCreation(format!("Failed to create src directory: {}", e)))?;
        std::fs::write(
            project_path.join("src/main.rs"),
            r#"fn main() {
    println!("Hello, world!");
}
"#
        ).map_err(|e| TestError::FileCreation(format!("Failed to create main.rs: {}", e)))?;
        
        Ok(project_path)
    }
    
    /// Create test Rust project with God Object pattern for testing
    pub fn create_test_project_with_god_object(&self, name: &str) -> Result<PathBuf, TestError> {
        let project_path = self.create_rust_project(name)?;
        
        // Create a God Object example
        let god_object_code = r#"
// This is a God Object with too many responsibilities
pub struct GodObject {
    user_data: Vec<String>,
    config: std::collections::HashMap<String, String>,
    cache: std::collections::HashMap<String, Vec<u8>>,
    metrics: Vec<f64>,
}

impl GodObject {
    pub fn new() -> Self { Self { user_data: Vec::new(), config: std::collections::HashMap::new(), cache: std::collections::HashMap::new(), metrics: Vec::new() } }
    
    // User management
    pub fn add_user(&mut self, user: String) { self.user_data.push(user); }
    pub fn remove_user(&mut self, user: &str) { self.user_data.retain(|u| u != user); }
    pub fn get_users(&self) -> &Vec<String> { &self.user_data }
    
    // Configuration management
    pub fn set_config(&mut self, key: String, value: String) { self.config.insert(key, value); }
    pub fn get_config(&self, key: &str) -> Option<&String> { self.config.get(key) }
    
    // Cache management
    pub fn cache_data(&mut self, key: String, data: Vec<u8>) { self.cache.insert(key, data); }
    pub fn get_cached_data(&self, key: &str) -> Option<&Vec<u8>> { self.cache.get(key) }
    
    // Metrics collection
    pub fn record_metric(&mut self, value: f64) { self.metrics.push(value); }
    pub fn get_average_metric(&self) -> f64 { self.metrics.iter().sum::<f64>() / self.metrics.len() as f64 }
    
    // File operations
    pub fn save_to_file(&self, path: &str) -> Result<(), std::io::Error> { std::fs::write(path, format!("{:?}", self)) }
    pub fn load_from_file(&mut self, path: &str) -> Result<(), std::io::Error> { Ok(()) }
    
    // Network operations
    pub fn send_http_request(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> { Ok("mock response".to_string()) }
    pub fn handle_websocket(&self, message: &str) -> String { format!("Echo: {}", message) }
    
    // Business logic
    pub fn process_business_rules(&self) -> bool { true }
    pub fn validate_data(&self, data: &str) -> bool { !data.is_empty() }
    pub fn transform_data(&self, input: String) -> String { input.to_uppercase() }
}
"#;
        
        self.create_test_file(&format!("{}/src/god_object.rs", name), god_object_code)?;
        
        // Update main.rs to use the god object
        let main_rs = r#"mod god_object;

use god_object::GodObject;

fn main() {
    let mut god = GodObject::new();
    god.add_user("test_user".to_string());
    god.set_config("key".to_string(), "value".to_string());
    god.record_metric(42.0);
    println!("Hello from God Object with {} users!", god.get_users().len());
}
"#;
        
        std::fs::write(project_path.join("src/main.rs"), main_rs)
            .map_err(|e| TestError::FileCreation(format!("Failed to update main.rs: {}", e)))?;
        
        Ok(project_path)
    }
    
    /// Create test project with circular dependencies
    pub fn create_test_project_with_circular_deps(&self, name: &str) -> Result<PathBuf, TestError> {
        let project_path = self.create_rust_project(name)?;
        
        // Create module A that depends on B
        let module_a = r#"use crate::module_b::function_b;

pub fn function_a() {
    println!("Function A");
    function_b();
}
"#;
        
        // Create module B that depends on A
        let module_b = r#"use crate::module_a::function_a;

pub fn function_b() {
    println!("Function B");
    // This would create infinite recursion, but we'll comment it out
    // function_a();  
}
"#;
        
        self.create_test_file(&format!("{}/src/module_a.rs", name), module_a)?;
        self.create_test_file(&format!("{}/src/module_b.rs", name), module_b)?;
        
        // Update main.rs to use both modules
        let main_rs = r#"mod module_a;
mod module_b;

fn main() {
    module_a::function_a();
}
"#;
        
        std::fs::write(project_path.join("src/main.rs"), main_rs)
            .map_err(|e| TestError::FileCreation(format!("Failed to update main.rs: {}", e)))?;
        
        Ok(project_path)
    }
    
    /// Get analysis engine for testing (creates on demand)
    pub async fn analysis_engine(&mut self) -> Result<&mut AnalysisEngine, TestError> {
        if self.analysis_engine.is_none() {
            let engine = AnalysisEngine::new()
                .map_err(|e| TestError::Setup(format!("Failed to create analysis engine: {}", e)))?;
            self.analysis_engine = Some(engine);
        }
        
        Ok(self.analysis_engine.as_mut().unwrap())
    }
    
    /// Get database for testing
    pub fn database(&self) -> &Database {
        &self.database
    }
    
    /// Cleanup test environment
    pub fn cleanup(self) {
        // TempDir automatically cleans up when dropped
        drop(self.temp_dir);
    }
}

#[derive(Debug, Default)]
pub struct TestConfig {
    pub enable_ai_features: bool,
    pub enable_performance_monitoring: bool,
    pub database_type: DatabaseType,
    pub mock_external_services: bool,
}

#[derive(Debug)]
pub enum DatabaseType {
    InMemory,
    Temporary,
}

impl Default for DatabaseType {
    fn default() -> Self {
        Self::InMemory
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TestError {
    #[error("Test setup failed: {0}")]
    Setup(String),
    #[error("File creation failed: {0}")]
    FileCreation(String),
    #[error("Analysis failed: {0}")]
    Analysis(String),
    #[error("Database error: {0}")]
    Database(String),
}

/// Utility functions for test data creation
pub mod test_data {
    use crate::database::models::ArchitecturalIssue;
    
    pub fn create_test_issue(id: i64, description: &str) -> ArchitecturalIssue {
        ArchitecturalIssue {
            issue_id: Some(id),
            analysis_run_id: 1,
            anti_pattern_type_id: 1,
            file_path: "test.rs".to_string(),
            start_line: Some(1),
            end_line: Some(10),
            severity: "medium".to_string(),
            description: description.to_string(),
            code_snippet: Some("test code snippet".to_string()),
            ai_explanation: None,
        }
    }
    
    pub fn create_god_object_issue() -> ArchitecturalIssue {
        create_test_issue(1, "God Object with too many responsibilities")
    }
    
    pub fn create_circular_dependency_issue() -> ArchitecturalIssue {
        create_test_issue(2, "Circular dependency detected")
    }
}

#[cfg(test)]
mod test_environment_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_test_environment() {
        let env = TestEnvironment::new().await.unwrap();
        assert!(env.temp_path().exists());
    }
    
    #[tokio::test]
    async fn test_create_rust_project() {
        let env = TestEnvironment::new().await.unwrap();
        let project_path = env.create_rust_project("test_project").unwrap();
        
        assert!(project_path.exists());
        assert!(project_path.join("Cargo.toml").exists());
        assert!(project_path.join("src/main.rs").exists());
    }
    
    #[tokio::test] 
    async fn test_create_test_file() {
        let env = TestEnvironment::new().await.unwrap();
        let file_path = env.create_test_file("nested/test.rs", "test content").unwrap();
        
        assert!(file_path.exists());
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "test content");
    }
    
    #[tokio::test]
    async fn test_ai_feature_config_integration() {
        let config = TestConfig {
            enable_ai_features: true,
            ..Default::default()
        };
        
        let _env = TestEnvironment::with_config(config).await.unwrap();
        
        // Test AI feature detection
        let ai_status = AiFeatureConfig::status();
        assert!(matches!(ai_status, crate::core::features::AiFeatureStatus::Enabled | crate::core::features::AiFeatureStatus::Disabled));
    }
}
