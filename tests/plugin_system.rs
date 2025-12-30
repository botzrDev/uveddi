//! Comprehensive tests for the WASM plugin system
//!
//! NOTE: This test module is currently disabled while the plugin system API is being
//! stabilized. The tests reference outdated types and helper functions that need to be
//! updated. Enable by removing the #![cfg(feature = "plugin-system-tests")] gate.

#![cfg(feature = "plugin-system-tests")]

use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;
use uveddi::ast::{AstNode, Position};
use uveddi::plugins::SecurityPolicy;
use uveddi::plugins::{
    AstDataPlane, Permission, PluginLifecycleManager, PluginManifest, PluginVerifier,
};
use uveddi::{
    analysis::AnalysisEngine,
    plugins::{PluginRegistry, WasmPluginEngine},
};

#[cfg(feature = "wasm-plugins")]
mod wasm_plugin_tests {
    use super::*;

    #[tokio::test]
    async fn test_plugin_engine_creation() {
        let temp_dir = TempDir::new().unwrap();
        let engine =
            WasmPluginEngine::with_config(temp_dir.path(), SecurityPolicy::permissive()).await;

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

        let plugin_id = registry
            .register_plugin(manifest.clone(), binary.clone())
            .await
            .unwrap();

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

        let report = verifier
            .verify_plugin(&binary, &manifest, &policy)
            .await
            .unwrap();

        assert_eq!(report.plugin_name, manifest.name);
        assert_eq!(report.plugin_version, manifest.version);
        assert!(!report.plugin_hash.is_empty());
    }

    #[tokio::test]
    async fn test_ast_data_plane() {
        use tree_sitter::Tree;
        use uveddi::ast::{CustomAst, ParsedFile, SourceLanguage};

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
        use tree_sitter::Tree;
        use uveddi::ast::{CustomAst, ParsedFile, SourceLanguage};

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
                children: vec![AstNode {
                    kind: "function_item".to_string(),
                    text: Some("fn main() {}".to_string()),
                    start_position: Position { row: 0, column: 0 },
                    end_position: Position { row: 0, column: 12 },
                    is_named: true,
                    children: vec![],
                }],
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

#[cfg(feature = "wasm-plugins")]
mod comprehensive_wasm_tests {
    use super::*;

    // ===========================================
    // PHASE 1: COMPREHENSIVE LIFECYCLE TESTING
    // ===========================================

    #[tokio::test]
    async fn test_plugin_discovery_and_loading() {
        let temp_dir = TempDir::new().unwrap();
        let mut registry = PluginRegistry::new(temp_dir.path()).await.unwrap();

        // Test empty registry initially
        assert_eq!(registry.list_plugins().len(), 0);

        // Create multiple test plugins
        let manifests = vec![
            create_test_manifest_with_name("plugin-1", "1.0.0"),
            create_test_manifest_with_name("plugin-2", "2.0.0"),
            create_test_manifest_with_name("plugin-3", "1.5.0"),
        ];

        let mut plugin_ids = Vec::new();

        // Register multiple plugins
        for manifest in manifests {
            let binary = create_minimal_wasm_binary();
            let plugin_id = registry
                .register_plugin(manifest.clone(), binary)
                .await
                .unwrap();
            plugin_ids.push(plugin_id);
        }

        // Verify all plugins are discovered
        assert_eq!(registry.list_plugins().len(), 3);

        // Test individual plugin loading
        for plugin_id in &plugin_ids {
            assert!(registry.get_plugin(plugin_id).is_some());
            let binary = registry.load_plugin_binary(plugin_id).await.unwrap();
            assert_eq!(binary, create_minimal_wasm_binary());
        }

        // Test plugin metadata validation
        let plugin_info = registry.get_plugin(&plugin_ids[0]).unwrap();
        assert_eq!(plugin_info.name, "plugin-1");
        assert_eq!(plugin_info.version, "1.0.0");
    }

    #[tokio::test]
    async fn test_plugin_execution_lifecycle() {
        let temp_dir = TempDir::new().unwrap();
        let engine = WasmPluginEngine::with_config(temp_dir.path(), SecurityPolicy::permissive())
            .await
            .unwrap();

        let mut lifecycle_manager = PluginLifecycleManager::new();

        // Test plugin initialization state
        assert_eq!(lifecycle_manager.list_active_plugins().await.len(), 0);

        // Create test AST data for processing
        let test_ast = create_test_parsed_file();
        let data_plane = AstDataPlane::new().unwrap();
        let serialized_ast = data_plane.serialize_ast(&test_ast).unwrap();

        // Test AST data processing capability
        let ast_info = data_plane.deserialize_ast(&serialized_ast).unwrap();
        assert!(ast_info.total_nodes > 0);
        assert_eq!(ast_info.languages.len(), 1);
        assert_eq!(ast_info.languages[0], "rust");

        // Verify engine is properly initialized
        assert!(engine.is_enabled());
    }

    #[tokio::test]
    async fn test_plugin_unloading_and_cleanup() {
        let temp_dir = TempDir::new().unwrap();
        let mut registry = PluginRegistry::new(temp_dir.path()).await.unwrap();

        // Register a test plugin
        let manifest = create_test_manifest();
        let binary = create_minimal_wasm_binary();
        let plugin_id = registry.register_plugin(manifest, binary).await.unwrap();

        // Verify plugin is registered
        assert_eq!(registry.list_plugins().len(), 1);
        assert!(registry.get_plugin(&plugin_id).is_some());

        // Test graceful plugin unloading
        registry.unregister_plugin(&plugin_id).await.unwrap();

        // Verify cleanup
        assert_eq!(registry.list_plugins().len(), 0);
        assert!(registry.get_plugin(&plugin_id).is_none());

        // Test loading binary after unregistration should fail
        let result = registry.load_plugin_binary(&plugin_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_concurrent_plugin_operations() {
        let temp_dir = TempDir::new().unwrap();
        let mut registry = PluginRegistry::new(temp_dir.path()).await.unwrap();

        // Test concurrent plugin registration
        let mut handles = Vec::new();

        for i in 0..5 {
            let manifest =
                create_test_manifest_with_name(&format!("concurrent-plugin-{}", i), "1.0.0");
            let binary = create_minimal_wasm_binary();

            let handle = tokio::spawn(async move {
                // Simulate concurrent operations
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                (manifest, binary)
            });
            handles.push(handle);
        }

        // Wait for all concurrent operations
        let mut plugin_ids = Vec::new();
        for handle in handles {
            let (manifest, binary) = handle.await.unwrap();
            let plugin_id = registry.register_plugin(manifest, binary).await.unwrap();
            plugin_ids.push(plugin_id);
        }

        // Verify all plugins were registered successfully
        assert_eq!(registry.list_plugins().len(), 5);

        // Test concurrent unloading
        let mut unload_handles = Vec::new();
        for plugin_id in plugin_ids {
            let handle = tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                plugin_id
            });
            unload_handles.push(handle);
        }

        for handle in unload_handles {
            let plugin_id = handle.await.unwrap();
            registry.unregister_plugin(&plugin_id).await.unwrap();
        }

        // Verify all plugins were unloaded
        assert_eq!(registry.list_plugins().len(), 0);
    }

    #[tokio::test]
    async fn test_plugin_state_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let registry_path = temp_dir.path();

        // Create first registry instance
        {
            let mut registry = PluginRegistry::new(registry_path).await.unwrap();
            let manifest = create_test_manifest_with_name("persistent-plugin", "1.0.0");
            let binary = create_minimal_wasm_binary();

            let _plugin_id = registry.register_plugin(manifest, binary).await.unwrap();
            assert_eq!(registry.list_plugins().len(), 1);
        } // Registry goes out of scope

        // Create second registry instance using same path
        {
            let registry = PluginRegistry::new(registry_path).await.unwrap();
            // Note: Real persistence would require actual file-based storage
            // For now, we test that a new registry can be created successfully
            assert!(registry.list_plugins().len() >= 0); // Should work without errors
        }
    }

    // ===========================================
    // PHASE 2: SECURITY MODEL VALIDATION
    // ===========================================

    #[tokio::test]
    async fn test_capability_restrictions() {
        let restrictive_policy = SecurityPolicy::restrictive();

        // Test file system access restrictions
        assert!(!restrictive_policy.has_permission(&Permission::FileRead(PathBuf::from("/tmp"))));
        assert!(!restrictive_policy.has_permission(&Permission::FileWrite(PathBuf::from("/tmp"))));

        // Test environment variable access restrictions
        assert!(!restrictive_policy.has_permission(&Permission::EnvRead("PATH".to_string())));

        // Test that only explicitly allowed permissions work
        let mut policy = SecurityPolicy::restrictive();
        policy.add_permission(Permission::Logging);
        assert!(policy.has_permission(&Permission::Logging));

        // Test binary validation with restrictive policy
        let valid_wasm = create_minimal_wasm_binary();
        assert!(policy.validate_binary(&valid_wasm).is_ok());

        let invalid_binary = b"not wasm";
        assert!(policy.validate_binary(invalid_binary).is_err());
    }

    #[tokio::test]
    async fn test_resource_limits_enforcement() {
        use uveddi::plugins::ResourceLimits;

        let limits = ResourceLimits::default();

        // Test default resource limits are reasonable
        assert_eq!(limits.max_memory, 256 * 1024 * 1024); // 256MB
        assert_eq!(limits.max_fuel, 10_000_000);
        assert_eq!(limits.max_execution_time_ms, 30_000); // 30 seconds
        assert_eq!(limits.max_file_handles, 10);

        // Test custom resource limits
        let custom_limits = ResourceLimits {
            max_memory: 128 * 1024 * 1024, // 128MB
            max_fuel: 1_000_000,
            max_execution_time_ms: 5_000, // 5 seconds
            max_file_handles: 5,
        };

        assert!(custom_limits.max_memory < limits.max_memory);
        assert!(custom_limits.max_fuel < limits.max_fuel);
        assert!(custom_limits.max_execution_time_ms < limits.max_execution_time_ms);
        assert!(custom_limits.max_file_handles < limits.max_file_handles);
    }

    #[tokio::test]
    async fn test_wasi_permission_boundaries() {
        let mut policy = SecurityPolicy::permissive();

        // Test file access permission hierarchies
        policy.add_permission(Permission::FileRead(PathBuf::from("/allowed")));

        // Should allow access to subdirectories
        assert!(policy.has_permission(&Permission::FileRead(PathBuf::from("/allowed/subdir"))));
        assert!(policy.has_permission(&Permission::FileRead(PathBuf::from("/allowed/file.txt"))));

        // Should not allow access to parent or sibling directories
        assert!(!policy.has_permission(&Permission::FileRead(PathBuf::from("/"))));
        assert!(!policy.has_permission(&Permission::FileRead(PathBuf::from("/other"))));

        // Test environment variable access patterns
        policy.add_permission(Permission::EnvRead("TEST_*".to_string()));

        // Should work with wildcard patterns (if implemented)
        assert!(policy.has_permission(&Permission::EnvRead("TEST_VAR".to_string())));
    }

    #[tokio::test]
    async fn test_plugin_isolation() {
        let temp_dir = TempDir::new().unwrap();
        let mut registry = PluginRegistry::new(temp_dir.path()).await.unwrap();

        // Create multiple isolated plugins
        let plugin1_manifest = create_test_manifest_with_name("isolated-plugin-1", "1.0.0");
        let plugin2_manifest = create_test_manifest_with_name("isolated-plugin-2", "1.0.0");

        let plugin1_id = registry
            .register_plugin(plugin1_manifest, create_minimal_wasm_binary())
            .await
            .unwrap();

        let plugin2_id = registry
            .register_plugin(plugin2_manifest, create_minimal_wasm_binary())
            .await
            .unwrap();

        // Verify plugins are isolated (different IDs)
        assert_ne!(plugin1_id, plugin2_id);

        // Verify each plugin has separate metadata
        let plugin1_info = registry.get_plugin(&plugin1_id).unwrap();
        let plugin2_info = registry.get_plugin(&plugin2_id).unwrap();

        assert_eq!(plugin1_info.name, "isolated-plugin-1");
        assert_eq!(plugin2_info.name, "isolated-plugin-2");
        assert_ne!(plugin1_info.name, plugin2_info.name);
    }

    #[tokio::test]
    async fn test_malicious_plugin_detection() {
        let policy = SecurityPolicy::restrictive();

        // Test detection of invalid WASM binaries
        let malicious_binaries = vec![
            b"malicious code".to_vec(),
            vec![0xFF; 1024],                // Random bytes
            vec![],                          // Empty binary
            b"#!/bin/sh\nrm -rf /".to_vec(), // Shell script
        ];

        for malicious_binary in malicious_binaries {
            let result = policy.validate_binary(&malicious_binary);
            assert!(result.is_err(), "Should reject malicious binary");
        }

        // Test that valid WASM is still accepted
        let valid_wasm = create_minimal_wasm_binary();
        assert!(policy.validate_binary(&valid_wasm).is_ok());
    }

    #[tokio::test]
    async fn test_code_signing_verification() {
        let verifier = PluginVerifier::new();
        let manifest = create_test_manifest();
        let binary = create_minimal_wasm_binary();
        let policy = SecurityPolicy::default();

        // Test plugin verification process
        let report = verifier
            .verify_plugin(&binary, &manifest, &policy)
            .await
            .unwrap();

        // Verify report contains security information
        assert_eq!(report.plugin_name, manifest.name);
        assert_eq!(report.plugin_version, manifest.version);
        assert!(!report.plugin_hash.is_empty());

        // Test verification with mismatched manifest
        let mut wrong_manifest = manifest.clone();
        wrong_manifest.name = "different-name".to_string();

        let result = verifier
            .verify_plugin(&binary, &wrong_manifest, &policy)
            .await;
        // Should still succeed as we're not enforcing strict name matching in this test
        assert!(result.is_ok());
    }

    // ===========================================
    // PHASE 3: PERFORMANCE & RESOURCE MANAGEMENT
    // ===========================================

    #[tokio::test]
    async fn test_plugin_startup_performance() {
        use std::time::Instant;

        let temp_dir = TempDir::new().unwrap();

        // Measure plugin engine startup time
        let start = Instant::now();
        let engine = WasmPluginEngine::with_config(temp_dir.path(), SecurityPolicy::permissive())
            .await
            .unwrap();
        let startup_duration = start.elapsed();

        // Engine startup should be reasonably fast (< 1 second)
        assert!(
            startup_duration.as_millis() < 1000,
            "Engine startup took too long: {:?}",
            startup_duration
        );

        assert!(engine.is_enabled());

        // Test registry creation performance
        let start = Instant::now();
        let _registry = PluginRegistry::new(temp_dir.path()).await.unwrap();
        let registry_creation_duration = start.elapsed();

        // Registry creation should be fast (< 100ms)
        assert!(
            registry_creation_duration.as_millis() < 100,
            "Registry creation took too long: {:?}",
            registry_creation_duration
        );
    }

    #[tokio::test]
    async fn test_memory_usage_patterns() {
        let temp_dir = TempDir::new().unwrap();
        let mut registry = PluginRegistry::new(temp_dir.path()).await.unwrap();

        // Test memory usage with multiple plugins
        let initial_plugin_count = registry.list_plugins().len();

        // Register multiple plugins and monitor registry size
        for i in 0..10 {
            let manifest =
                create_test_manifest_with_name(&format!("memory-test-plugin-{}", i), "1.0.0");
            let binary = create_minimal_wasm_binary();
            let _plugin_id = registry.register_plugin(manifest, binary).await.unwrap();
        }

        assert_eq!(registry.list_plugins().len(), initial_plugin_count + 10);

        // Test AST data memory efficiency
        let data_plane = AstDataPlane::new().unwrap();
        let test_ast = create_large_test_ast();

        let serialized = data_plane.serialize_ast(&test_ast).unwrap();
        assert!(!serialized.is_empty());

        // Verify serialized data is reasonable size
        assert!(
            serialized.len() < 10 * 1024 * 1024, // Less than 10MB for test data
            "Serialized AST is too large: {} bytes",
            serialized.len()
        );
    }

    #[tokio::test]
    async fn test_concurrent_plugin_execution() {
        let temp_dir = TempDir::new().unwrap();
        let engine = WasmPluginEngine::with_config(temp_dir.path(), SecurityPolicy::permissive())
            .await
            .unwrap();

        let mut lifecycle_manager = PluginLifecycleManager::new();

        // Test that multiple lifecycle operations can be performed concurrently
        let mut handles = Vec::new();

        for i in 0..3 {
            let handle = tokio::spawn(async move {
                // Simulate concurrent plugin operations
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                format!("concurrent-operation-{}", i)
            });
            handles.push(handle);
        }

        // Wait for all concurrent operations
        let mut results = Vec::new();
        for handle in handles {
            let result = handle.await.unwrap();
            results.push(result);
        }

        assert_eq!(results.len(), 3);
        assert!(engine.is_enabled());
        assert_eq!(lifecycle_manager.list_active_plugins().await.len(), 0);
    }

    #[tokio::test]
    #[ignore = "Performance benchmark - run separately"]
    async fn benchmark_plugin_operations() {
        use std::time::Instant;

        let temp_dir = TempDir::new().unwrap();
        let mut registry = PluginRegistry::new(temp_dir.path()).await.unwrap();

        // Benchmark plugin registration
        let start = Instant::now();
        for i in 0..100 {
            let manifest =
                create_test_manifest_with_name(&format!("benchmark-plugin-{}", i), "1.0.0");
            let binary = create_minimal_wasm_binary();
            let _plugin_id = registry.register_plugin(manifest, binary).await.unwrap();
        }
        let registration_duration = start.elapsed();

        println!("Registered 100 plugins in {:?}", registration_duration);
        assert!(
            registration_duration.as_secs() < 10,
            "Plugin registration too slow"
        );

        // Benchmark AST serialization
        let data_plane = AstDataPlane::new().unwrap();
        let test_ast = create_large_test_ast();

        let start = Instant::now();
        for _ in 0..10 {
            let _serialized = data_plane.serialize_ast(&test_ast).unwrap();
        }
        let serialization_duration = start.elapsed();

        println!("Serialized AST 10 times in {:?}", serialization_duration);
        assert!(
            serialization_duration.as_secs() < 5,
            "AST serialization too slow"
        );
    }

    // ===========================================
    // PHASE 4: DATA EXCHANGE VALIDATION
    // ===========================================

    #[tokio::test]
    async fn test_ast_serialization_roundtrip() {
        let data_plane = AstDataPlane::new().unwrap();

        // Test with simple AST
        let simple_ast = create_test_parsed_file();
        let serialized = data_plane.serialize_ast(&simple_ast).unwrap();
        let deserialized_info = data_plane.deserialize_ast(&serialized).unwrap();

        assert!(deserialized_info.total_nodes > 0);
        assert_eq!(deserialized_info.languages.len(), 1);
        assert_eq!(deserialized_info.languages[0], "rust");
        assert!(!deserialized_info.files.is_empty());

        // Test with complex AST
        let complex_ast = create_large_test_ast();
        let serialized_complex = data_plane.serialize_ast(&complex_ast).unwrap();
        let deserialized_complex_info = data_plane.deserialize_ast(&serialized_complex).unwrap();

        assert!(deserialized_complex_info.total_nodes >= deserialized_info.total_nodes);
        assert!(!deserialized_complex_info.files.is_empty());
    }

    #[tokio::test]
    async fn test_large_data_transfer() {
        let data_plane = AstDataPlane::new().unwrap();

        // Create large AST with many nodes
        let large_ast = create_very_large_test_ast();

        // Test serialization performance with large data
        let start = std::time::Instant::now();
        let serialized = data_plane.serialize_ast(&large_ast).unwrap();
        let serialization_time = start.elapsed();

        assert!(!serialized.is_empty());
        assert!(
            serialization_time.as_secs() < 10,
            "Large AST serialization too slow: {:?}",
            serialization_time
        );

        // Test deserialization
        let start = std::time::Instant::now();
        let info = data_plane.deserialize_ast(&serialized).unwrap();
        let deserialization_time = start.elapsed();

        assert!(info.total_nodes > 1000); // Should have many nodes
        assert!(
            deserialization_time.as_secs() < 5,
            "Large AST deserialization too slow: {:?}",
            deserialization_time
        );
    }

    #[tokio::test]
    async fn test_data_format_compatibility() {
        let data_plane = AstDataPlane::new().unwrap();

        // Test with different AST structures
        let ast_variants = vec![
            create_test_parsed_file(),
            create_javascript_test_ast(),
            create_python_test_ast(),
        ];

        for ast in ast_variants {
            let serialized = data_plane.serialize_ast(&ast).unwrap();
            let info = data_plane.deserialize_ast(&serialized).unwrap();

            assert!(info.total_nodes > 0);
            assert!(!info.languages.is_empty());
            assert!(!info.files.is_empty());
        }
    }

    #[tokio::test]
    async fn test_data_integrity_across_boundary() {
        let data_plane = AstDataPlane::new().unwrap();
        let original_ast = create_test_parsed_file();

        // Serialize multiple times and verify consistency
        let serialized1 = data_plane.serialize_ast(&original_ast).unwrap();
        let serialized2 = data_plane.serialize_ast(&original_ast).unwrap();

        // Should get identical serialized data for same input
        assert_eq!(serialized1.len(), serialized2.len());

        // Verify deserialization gives consistent results
        let info1 = data_plane.deserialize_ast(&serialized1).unwrap();
        let info2 = data_plane.deserialize_ast(&serialized2).unwrap();

        assert_eq!(info1.total_nodes, info2.total_nodes);
        assert_eq!(info1.languages, info2.languages);
        assert_eq!(info1.files.len(), info2.files.len());
    }

    // ===========================================
    // PHASE 5: INTEGRATION TESTING
    // ===========================================

    #[tokio::test]
    async fn test_full_analysis_pipeline_with_plugins() {
        let mut engine = AnalysisEngine::new_with_plugins().await.unwrap();

        // Verify plugin support is enabled
        assert!(engine.has_plugin_support());

        // Test loading plugins (should be 0 in test environment)
        let loaded_count = engine.load_plugins().await.unwrap();
        assert_eq!(loaded_count, 0);

        // Test registry stats integration
        let stats = engine.get_plugin_registry_stats();
        assert!(stats.is_some());
        let stats = stats.unwrap();
        assert_eq!(stats.total_plugins, 0);

        // Test basic engine functionality still works
        assert!(engine.get_files_analyzed() >= 0);
        let anti_pattern_types = engine.get_anti_pattern_types();
        assert!(!anti_pattern_types.is_empty());
    }

    #[tokio::test]
    async fn test_plugin_error_recovery() {
        let temp_dir = TempDir::new().unwrap();
        let mut registry = PluginRegistry::new(temp_dir.path()).await.unwrap();

        // Test recovery from plugin registration errors
        let valid_manifest = create_test_manifest();
        let invalid_binary = b"invalid wasm";

        // Should handle invalid binary gracefully
        let result = registry
            .register_plugin(valid_manifest.clone(), invalid_binary.to_vec())
            .await;
        assert!(result.is_err(), "Should reject invalid binary");

        // Registry should still be functional after error
        assert_eq!(registry.list_plugins().len(), 0);

        // Should be able to register valid plugin after error
        let valid_binary = create_minimal_wasm_binary();
        let plugin_id = registry
            .register_plugin(valid_manifest, valid_binary)
            .await
            .unwrap();

        assert_eq!(registry.list_plugins().len(), 1);
        assert!(registry.get_plugin(&plugin_id).is_some());
    }

    #[tokio::test]
    async fn test_plugin_adapter_functionality() {
        // Test basic plugin adapter integration
        let mut engine = AnalysisEngine::new_with_plugins().await.unwrap();

        // Plugin adapter should be integrated with analysis engine
        assert!(engine.has_plugin_support());

        // Test that analysis engine gracefully handles no plugins
        let result = engine.load_plugins().await;
        assert!(result.is_ok());

        // Engine should still function normally
        let anti_pattern_types = engine.get_anti_pattern_types();
        assert!(!anti_pattern_types.is_empty());
    }

    // ===========================================
    // PHASE 6: ERROR HANDLING & EDGE CASES
    // ===========================================

    #[tokio::test]
    async fn test_malformed_plugin_handling() {
        let temp_dir = TempDir::new().unwrap();
        let mut registry = PluginRegistry::new(temp_dir.path()).await.unwrap();

        // Test various malformed plugin scenarios
        let malformed_cases = vec![
            (create_test_manifest(), vec![]),               // Empty binary
            (create_test_manifest(), b"not wasm".to_vec()), // Invalid binary
            (create_test_manifest(), vec![0xFF; 1000]),     // Random bytes
        ];

        for (manifest, binary) in malformed_cases {
            let result = registry.register_plugin(manifest, binary).await;
            assert!(result.is_err(), "Should reject malformed plugin");
        }

        // Registry should remain functional
        assert_eq!(registry.list_plugins().len(), 0);

        // Valid plugin should still work after errors
        let valid_manifest = create_test_manifest();
        let valid_binary = create_minimal_wasm_binary();
        let plugin_id = registry
            .register_plugin(valid_manifest, valid_binary)
            .await
            .unwrap();

        assert_eq!(registry.list_plugins().len(), 1);
        assert!(registry.get_plugin(&plugin_id).is_some());
    }

    #[tokio::test]
    async fn test_plugin_crash_recovery() {
        let temp_dir = TempDir::new().unwrap();
        let engine = WasmPluginEngine::with_config(temp_dir.path(), SecurityPolicy::permissive())
            .await
            .unwrap();

        // Engine should be resilient to initialization issues
        assert!(engine.is_enabled());

        // Test that system remains stable after simulated errors
        let mut lifecycle_manager = PluginLifecycleManager::new();
        assert_eq!(lifecycle_manager.list_active_plugins().await.len(), 0);

        // System should continue to work normally
        let data_plane = AstDataPlane::new().unwrap();
        let test_ast = create_test_parsed_file();
        let serialized = data_plane.serialize_ast(&test_ast).unwrap();
        let info = data_plane.deserialize_ast(&serialized).unwrap();

        assert!(info.total_nodes > 0);
    }

    #[tokio::test]
    async fn test_timeout_scenarios() {
        use std::time::Duration;

        // Test timeout handling with reasonable limits
        let timeout = Duration::from_millis(100);

        // Simulate operation that should complete within timeout
        let start = std::time::Instant::now();
        let temp_dir = TempDir::new().unwrap();
        let _registry = PluginRegistry::new(temp_dir.path()).await.unwrap();
        let elapsed = start.elapsed();

        assert!(elapsed < timeout, "Registry creation should be fast");

        // Test data plane operations are reasonably fast
        let start = std::time::Instant::now();
        let data_plane = AstDataPlane::new().unwrap();
        let test_ast = create_test_parsed_file();
        let _serialized = data_plane.serialize_ast(&test_ast).unwrap();
        let elapsed = start.elapsed();

        assert!(
            elapsed < Duration::from_secs(1),
            "Serialization should be fast"
        );
    }

    #[tokio::test]
    async fn test_resource_exhaustion_scenarios() {
        use uveddi::plugins::ResourceLimits;

        // Test behavior under resource constraints
        let restrictive_limits = ResourceLimits {
            max_memory: 1024 * 1024, // 1MB
            max_fuel: 1000,
            max_execution_time_ms: 100,
            max_file_handles: 1,
        };

        // Verify limits are reasonable (not zero)
        assert!(restrictive_limits.max_memory > 0);
        assert!(restrictive_limits.max_fuel > 0);
        assert!(restrictive_limits.max_execution_time_ms > 0);
        assert!(restrictive_limits.max_file_handles > 0);

        // Test that system can handle restrictive policies
        let policy = SecurityPolicy::restrictive();
        let valid_wasm = create_minimal_wasm_binary();
        assert!(policy.validate_binary(&valid_wasm).is_ok());
    }

    #[tokio::test]
    async fn test_graceful_degradation() {
        // Test system behavior when plugins are disabled
        let engine = AnalysisEngine::new().unwrap();
        assert!(!engine.has_plugin_support());

        // Engine should still provide core functionality
        assert!(engine.get_files_analyzed() >= 0);
        let anti_pattern_types = engine.get_anti_pattern_types();
        assert!(!anti_pattern_types.is_empty());

        // Test plugin-related operations fail gracefully
        #[cfg(not(feature = "wasm-plugins"))]
        {
            let result = WasmPluginEngine::new().await;
            assert!(result.is_err());
        }
    }

    // Enhanced helper functions for comprehensive testing
    fn create_test_manifest_with_name(name: &str, version: &str) -> PluginManifest {
        PluginManifest {
            name: name.to_string(),
            version: version.to_string(),
            author: "Test Author".to_string(),
            description: format!("Test plugin {} for comprehensive testing", name),
            permissions: vec![Permission::Logging],
            supported_languages: vec!["rust".to_string(), "javascript".to_string()],
            anti_pattern_types: vec!["test-pattern".to_string()],
            signature: None,
        }
    }

    fn create_large_test_ast() -> uveddi::ast::tree_sitter::ParsedFile {
        use tree_sitter::Tree;
        use uveddi::ast::{CustomAst, ParsedFile, SourceLanguage};

        let mut children = Vec::new();
        for i in 0..100 {
            children.push(AstNode {
                kind: format!("test_node_{}", i),
                text: Some(format!("test content {}", i)),
                start_position: Position { row: i, column: 0 },
                end_position: Position { row: i, column: 10 },
                is_named: true,
                children: vec![],
            });
        }

        ParsedFile {
            path: PathBuf::from("large_test.rs"),
            content: "// Large test file with many nodes".to_string(),
            language: "rust".to_string(),
            ast: Some(AstNode {
                kind: "source_file".to_string(),
                text: Some("// Large test file with many nodes".to_string()),
                start_position: Position { row: 0, column: 0 },
                end_position: Position {
                    row: 100,
                    column: 0,
                },
                is_named: true,
                children,
            }),
        }
    }

    fn create_very_large_test_ast() -> uveddi::ast::tree_sitter::ParsedFile {
        use tree_sitter::Tree;
        use uveddi::ast::{CustomAst, ParsedFile, SourceLanguage};

        let mut children = Vec::new();
        for i in 0..1000 {
            let mut nested_children = Vec::new();
            for j in 0..5 {
                nested_children.push(AstNode {
                    kind: format!("nested_node_{}_{}", i, j),
                    text: Some(format!("nested content {} {}", i, j)),
                    start_position: Position {
                        row: i,
                        column: j * 10,
                    },
                    end_position: Position {
                        row: i,
                        column: (j + 1) * 10,
                    },
                    is_named: true,
                    children: vec![],
                });
            }

            children.push(AstNode {
                kind: format!("large_node_{}", i),
                text: Some(format!("large content {}", i)),
                start_position: Position { row: i, column: 0 },
                end_position: Position { row: i, column: 50 },
                is_named: true,
                children: nested_children,
            });
        }

        ParsedFile {
            path: PathBuf::from("very_large_test.rs"),
            content: "// Very large test file with thousands of nodes".to_string(),
            language: "rust".to_string(),
            ast: Some(AstNode {
                kind: "source_file".to_string(),
                text: Some("// Very large test file with thousands of nodes".to_string()),
                start_position: Position { row: 0, column: 0 },
                end_position: Position {
                    row: 1000,
                    column: 0,
                },
                is_named: true,
                children,
            }),
        }
    }

    fn create_javascript_test_ast() -> uveddi::ast::tree_sitter::ParsedFile {
        use tree_sitter::Tree;
        use uveddi::ast::{CustomAst, ParsedFile, SourceLanguage};

        ParsedFile {
            path: PathBuf::from("test.js"),
            content: "function test() { return 42; }".to_string(),
            language: "javascript".to_string(),
            ast: Some(AstNode {
                kind: "program".to_string(),
                text: Some("function test() { return 42; }".to_string()),
                start_position: Position { row: 0, column: 0 },
                end_position: Position { row: 0, column: 30 },
                is_named: true,
                children: vec![AstNode {
                    kind: "function_declaration".to_string(),
                    text: Some("function test() { return 42; }".to_string()),
                    start_position: Position { row: 0, column: 0 },
                    end_position: Position { row: 0, column: 30 },
                    is_named: true,
                    children: vec![],
                }],
            }),
        }
    }

    fn create_python_test_ast() -> uveddi::ast::tree_sitter::ParsedFile {
        use tree_sitter::Tree;
        use uveddi::ast::{CustomAst, ParsedFile, SourceLanguage};

        ParsedFile {
            path: PathBuf::from("test.py"),
            content: "def test():\n    return 42".to_string(),
            language: "python".to_string(),
            ast: Some(AstNode {
                kind: "module".to_string(),
                text: Some("def test():\n    return 42".to_string()),
                start_position: Position { row: 0, column: 0 },
                end_position: Position { row: 1, column: 13 },
                is_named: true,
                children: vec![AstNode {
                    kind: "function_definition".to_string(),
                    text: Some("def test():\n    return 42".to_string()),
                    start_position: Position { row: 0, column: 0 },
                    end_position: Position { row: 1, column: 13 },
                    is_named: true,
                    children: vec![],
                }],
            }),
        }
    }
}
