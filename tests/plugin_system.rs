//! Comprehensive tests for the WASM plugin system

use std::path::PathBuf;
use tempfile::TempDir;
use uveddi::{
    analysis::AnalysisEngine,
    plugins::{
        WasmPluginEngine, PluginManifest, SecurityPolicy, Permission,
        PluginVerifier, AstDataPlane, PluginRegistry, PluginLifecycleManager,
    },
};

#[cfg(feature = "wasm-plugins")]
mod wasm_plugin_tests {
    use super::*;

    #[tokio::test]
    async fn test_plugin_engine_creation() {
        let temp_dir = TempDir::new().unwrap();
        let engine = WasmPluginEngine::with_config(
            temp_dir.path(),
            SecurityPolicy::permissive(),
        ).await;
        
        assert!(engine.is_ok());
        let engine = engine.unwrap();
        assert!(engine.is_enabled());
    }

    #[tokio::test]
    async fn test_plugin_registry() {
        let temp_dir = TempDir::new().unwrap();
        let mut registry = PluginRegistry::new(temp_dir.path()).await.unwrap();
        
        let manifest = create_test_manifest();
        let binary = create_minimal_wasm_binary();
        
        let plugin_id = registry.register_plugin(manifest.clone(), binary.clone()).await.unwrap();
        
        assert_eq!(registry.list_plugins().len(), 1);
        assert!(registry.get_plugin(&plugin_id).is_some());
        
        let loaded_binary = registry.load_plugin_binary(&plugin_id).await.unwrap();
        assert_eq!(loaded_binary, binary);
        
        registry.unregister_plugin(&plugin_id).await.unwrap();
        assert_eq!(registry.list_plugins().len(), 0);
    }

    #[tokio::test]
    async fn test_security_policy() {
        let policy = SecurityPolicy::restrictive();
        
        // Test permission checking
        assert!(!policy.has_permission(&Permission::FileRead(PathBuf::from("/tmp"))));
        
        let mut policy = SecurityPolicy::permissive();
        policy.add_permission(Permission::FileRead(PathBuf::from("/tmp")));
        assert!(policy.has_permission(&Permission::FileRead(PathBuf::from("/tmp/file.txt"))));
        
        // Test binary validation
        let valid_wasm = create_minimal_wasm_binary();
        assert!(policy.validate_binary(&valid_wasm).is_ok());
        
        let invalid_binary = b"invalid binary";
        assert!(policy.validate_binary(invalid_binary).is_err());
    }

    #[tokio::test]
    async fn test_plugin_verification() {
        let verifier = PluginVerifier::new();
        let manifest = create_test_manifest();
        let binary = create_minimal_wasm_binary();
        let policy = SecurityPolicy::default();
        
        let report = verifier.verify_plugin(&binary, &manifest, &policy).await.unwrap();
        
        assert_eq!(report.plugin_name, manifest.name);
        assert_eq!(report.plugin_version, manifest.version);
        assert!(!report.plugin_hash.is_empty());
    }

    #[tokio::test]
    async fn test_ast_data_plane() {
        use uveddi::ast::tree_sitter::{ParsedFile, AstNode, Position};
        
        let data_plane = AstDataPlane::new().unwrap();
        
        // Create a test parsed file
        let parsed_file = create_test_parsed_file();
        
        // Test serialization
        let serialized = data_plane.serialize_ast(&parsed_file).unwrap();
        assert!(!serialized.is_empty());
        
        // Test deserialization info
        let info = data_plane.deserialize_ast(&serialized).unwrap();
        assert!(info.total_nodes > 0);
        assert!(!info.languages.is_empty());
        assert!(!info.files.is_empty());
    }

    #[tokio::test]
    async fn test_lifecycle_manager() {
        let mut manager = PluginLifecycleManager::new();
        
        assert_eq!(manager.list_active_plugins().await.len(), 0);
        
        // Note: Full lifecycle testing would require valid WASM components
        // For now, we test the manager creation and basic operations
    }

    #[tokio::test]
    async fn test_analysis_engine_plugin_integration() {
        let mut engine = AnalysisEngine::new_with_plugins().await.unwrap();
        
        // Test plugin support detection
        assert!(engine.has_plugin_support());
        
        // Test loading plugins (will be 0 since no plugins are installed)
        let loaded_count = engine.load_plugins().await.unwrap();
        assert_eq!(loaded_count, 0);
        
        // Test registry stats
        let stats = engine.get_plugin_registry_stats();
        assert!(stats.is_some());
        let stats = stats.unwrap();
        assert_eq!(stats.total_plugins, 0);
    }

    #[test]
    fn test_permission_system() {
        let read_src = Permission::FileRead(PathBuf::from("/src"));
        let read_src_subdir = Permission::FileRead(PathBuf::from("/src/main.rs"));
        let read_other = Permission::FileRead(PathBuf::from("/other"));
        
        assert!(read_src.allows(&read_src_subdir));
        assert!(!read_src.allows(&read_other));
        
        let env_all = Permission::EnvRead("*".to_string());
        let env_specific = Permission::EnvRead("PATH".to_string());
        
        assert!(env_all.allows(&env_specific));
        assert!(!env_specific.allows(&env_all));
    }

    #[test]
    fn test_plugin_manifest() {
        let mut manifest = PluginManifest::new(
            "test-plugin".to_string(),
            "1.0.0".to_string(),
            "Test Author".to_string(),
        );
        
        manifest.add_permission(Permission::Logging);
        manifest.add_language("rust".to_string());
        manifest.add_anti_pattern_type("god-object".to_string());
        
        assert_eq!(manifest.permissions.len(), 1);
        assert_eq!(manifest.supported_languages.len(), 1);
        assert_eq!(manifest.anti_pattern_types.len(), 1);
    }

    // Helper functions for tests
    fn create_test_manifest() -> PluginManifest {
        PluginManifest {
            name: "test-plugin".to_string(),
            version: "1.0.0".to_string(),
            author: "Test Author".to_string(),
            description: "Test plugin for unit tests".to_string(),
            permissions: vec![Permission::Logging],
            supported_languages: vec!["rust".to_string()],
            anti_pattern_types: vec!["test-pattern".to_string()],
            signature: None,
        }
    }

    fn create_minimal_wasm_binary() -> Vec<u8> {
        // Minimal valid WASM binary
        vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]
    }

    fn create_test_parsed_file() -> uveddi::ast::tree_sitter::ParsedFile {
        use uveddi::ast::tree_sitter::{ParsedFile, AstNode, Position};
        
        ParsedFile {
            path: PathBuf::from("test.rs"),
            content: "fn main() {}".to_string(),
            language: "rust".to_string(),
            ast: Some(AstNode {
                kind: "source_file".to_string(),
                text: Some("fn main() {}".to_string()),
                start_position: Position { row: 0, column: 0 },
                end_position: Position { row: 0, column: 12 },
                is_named: true,
                children: vec![
                    AstNode {
                        kind: "function_item".to_string(),
                        text: Some("fn main() {}".to_string()),
                        start_position: Position { row: 0, column: 0 },
                        end_position: Position { row: 0, column: 12 },
                        is_named: true,
                        children: vec![],
                    }
                ],
            }),
        }
    }
}

#[cfg(not(feature = "wasm-plugins"))]
mod no_wasm_plugin_tests {
    use super::*;

    #[tokio::test]
    async fn test_plugin_engine_disabled() {
        let result = WasmPluginEngine::new().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_analysis_engine_no_plugin_support() {
        let engine = AnalysisEngine::new().unwrap();
        assert!(!engine.has_plugin_support());
    }
}

// Integration tests that work regardless of feature flags
#[tokio::test]
async fn test_analysis_engine_basic_functionality() {
    let engine = AnalysisEngine::new().unwrap();
    
    // Basic engine should work without plugins
    assert!(engine.get_files_analyzed() >= 0);
    
    // Anti-pattern types should be available
    let anti_pattern_types = engine.get_anti_pattern_types();
    assert!(!anti_pattern_types.is_empty());
}

#[test]
fn test_plugin_id_generation() {
    use uveddi::plugins::PluginId;
    
    let id1 = PluginId::new();
    let id2 = PluginId::new();
    
    // IDs should be unique
    assert_ne!(id1, id2);
    
    // IDs from same name should be identical
    let id3 = PluginId::from_name("test-plugin");
    let id4 = PluginId::from_name("test-plugin");
    assert_eq!(id3, id4);
}

#[test]
fn test_plugin_config() {
    use uveddi::plugins::PluginConfig;
    
    let config = PluginConfig::default();
    assert_eq!(config.severity_threshold, 0.5);
    assert_eq!(config.max_issues_per_file, 100);
    assert!(config.custom_settings.is_empty());
}

#[test]
fn test_resource_limits() {
    use uveddi::plugins::ResourceLimits;
    
    let limits = ResourceLimits::default();
    assert_eq!(limits.max_memory, 256 * 1024 * 1024); // 256MB
    assert_eq!(limits.max_fuel, 10_000_000);
    assert_eq!(limits.max_execution_time_ms, 30_000);
    assert_eq!(limits.max_file_handles, 10);
}

#[test]
fn test_plugin_stats() {
    use uveddi::plugins::PluginStats;
    
    let mut stats = PluginStats::default();
    
    stats.record_execution(100, 1000, 1024);
    assert_eq!(stats.invocations, 1);
    assert_eq!(stats.total_execution_time_ms, 100);
    assert_eq!(stats.avg_execution_time_ms, 100.0);
    assert_eq!(stats.total_fuel_consumed, 1000);
    assert_eq!(stats.peak_memory_usage, 1024);
    
    stats.record_error("Test error".to_string());
    assert_eq!(stats.error_count, 1);
    assert_eq!(stats.last_error, Some("Test error".to_string()));
}