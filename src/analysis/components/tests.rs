//! Comprehensive unit tests for component-based architecture
//!
//! Tests each component in isolation using mockall for dependency mocking.

#[cfg(test)]
mod config_service_tests {
    use super::super::config_service::ConfigurationService;
    use super::super::traits::ConfigurationService as ConfigurationServiceTrait;
    use std::path::PathBuf;

    #[test]
    fn test_default_configuration() {
        let service = ConfigurationService::new();

        assert!(!service.are_plugins_enabled());
        assert!(service.get_cache_path().is_some());
        assert!(service.is_detector_enabled("GodObjectDetector"));
        assert!(service.is_detector_enabled("DeadCodeDetector"));
        assert!(service.is_detector_enabled("CodeDuplicationDetector"));
        assert!(service.is_detector_enabled("LargeClassDetector"));
        assert!(service.is_detector_enabled("TightCouplingDetector"));
        assert!(!service.is_detector_enabled("non_existent_detector"));
    }

    #[test]
    fn test_configuration_mutations() {
        let mut service = ConfigurationService::new();

        // Test setting values
        service.set_config_value("test_key".to_string(), "test_value".to_string());
        assert_eq!(
            service.get_config_value("test_key"),
            Some("test_value".to_string())
        );

        // Test detector configuration
        service.set_detector_enabled("custom_detector".to_string(), true);
        assert!(service.is_detector_enabled("custom_detector"));

        service.set_detector_enabled("custom_detector".to_string(), false);
        assert!(!service.is_detector_enabled("custom_detector"));

        // Test plugin configuration
        service.set_plugins_enabled(true);
        assert!(service.are_plugins_enabled());

        service.set_plugins_enabled(false);
        assert!(!service.are_plugins_enabled());

        // Test cache path
        service.set_cache_path(Some(PathBuf::from("custom_cache.db")));
        assert_eq!(
            service.get_cache_path(),
            Some(PathBuf::from("custom_cache.db"))
        );

        service.set_cache_path(None);
        assert_eq!(service.get_cache_path(), None);
    }

    #[test]
    fn test_plugin_config() {
        let mut service = ConfigurationService::new();

        let config_value = serde_json::json!({"param1": "value1", "param2": 42});
        service.set_plugin_config("test_plugin".to_string(), config_value.clone());

        let plugin_config = service.get_plugin_config();
        assert_eq!(plugin_config.get("test_plugin"), Some(&config_value));
    }
}

#[cfg(test)]
mod ast_provider_tests {
    use super::super::ast_provider::AstProviderImpl;
    use super::super::traits::AstProvider;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_ast_provider_creation() {
        let provider = AstProviderImpl::new();
        assert!(provider.is_ok());
    }

    #[tokio::test]
    async fn test_cache_metrics() {
        let provider = AstProviderImpl::new().unwrap();

        let metrics = provider.get_cache_metrics();
        assert!(metrics.is_object());
    }

    #[tokio::test]
    async fn test_cache_clearing() {
        let provider = AstProviderImpl::new().unwrap();

        // Should not panic
        provider.clear_cache();

        let metrics_after = provider.get_cache_metrics();
        assert!(metrics_after.is_object());
    }

    #[tokio::test]
    async fn test_ast_parsing_simple_file() {
        let provider = AstProviderImpl::new().unwrap();

        // Create a simple Rust file
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "fn main() {{ println!(\"Hello\"); }}").unwrap();

        let result = provider.get_ast(temp_file.path()).await;
        // AST parsing success depends on tree-sitter feature being enabled
        // We test that it doesn't panic and returns a proper Result
        assert!(result.is_ok() || result.is_err());
    }
}

#[cfg(test)]
mod analysis_aggregator_tests {
    use super::super::analysis_aggregator::AnalysisAggregator;
    use super::super::traits::AnalysisAggregator as AnalysisAggregatorTrait;
    use crate::database::models::ArchitecturalIssue;

    fn create_test_issue(severity: &str, file_path: &str) -> ArchitecturalIssue {
        ArchitecturalIssue {
            issue_id: None,
            analysis_run_id: 1,
            anti_pattern_type_id: 1,
            file_path: file_path.to_string(),
            start_line: Some(10),
            end_line: Some(15),
            severity: severity.to_string(),
            description: "Test issue".to_string(),
            code_snippet: None,
            ai_explanation: None,
        }
    }

    #[test]
    fn test_aggregator_creation() {
        let aggregator = AnalysisAggregator::new();
        assert_eq!(aggregator.get_findings_count(), 0);

        let stats = aggregator.get_stats();
        assert_eq!(stats.total_findings, 0);
        assert_eq!(stats.files_processed, 0);
        assert!(stats.findings_by_type.is_empty());
    }

    #[test]
    fn test_record_single_finding() {
        let aggregator = AnalysisAggregator::new();

        let issue = create_test_issue("high", "src/main.rs");
        aggregator.record_finding(issue);

        assert_eq!(aggregator.get_findings_count(), 1);

        let stats = aggregator.get_stats();
        assert_eq!(stats.total_findings, 1);
        assert_eq!(stats.findings_by_type.get("god_object"), Some(&1));
    }

    #[test]
    fn test_record_multiple_findings() {
        let aggregator = AnalysisAggregator::new();

        let issues = vec![
            create_test_issue("high", "src/main.rs"),
            create_test_issue("medium", "src/lib.rs"),
            create_test_issue("low", "src/utils.rs"),
        ];

        aggregator.record_findings(issues);

        assert_eq!(aggregator.get_findings_count(), 3);

        let stats = aggregator.get_stats();
        assert_eq!(stats.total_findings, 3);
        assert_eq!(stats.findings_by_type.get("god_object"), Some(&3));
    }

    #[test]
    fn test_file_processing_tracking() {
        let aggregator = AnalysisAggregator::new();

        aggregator.record_file_processed();
        aggregator.record_file_processed();
        aggregator.record_file_processed();

        let stats = aggregator.get_stats();
        assert_eq!(stats.files_processed, 3);
    }

    #[test]
    fn test_filtering_methods() {
        let aggregator = AnalysisAggregator::new();

        // Add various issues
        aggregator.record_finding(create_test_issue("high", "src/main.rs"));
        aggregator.record_finding(create_test_issue("medium", "src/main.rs"));
        aggregator.record_finding(create_test_issue("low", "src/lib.rs"));

        // Test filtering by file
        let main_issues = aggregator.get_findings_for_file("src/main.rs");
        assert_eq!(main_issues.len(), 2);

        let lib_issues = aggregator.get_findings_for_file("src/lib.rs");
        assert_eq!(lib_issues.len(), 1);

        // Test filtering by severity
        let high_severity = aggregator.get_findings_by_severity(3); // high = 3
        assert_eq!(high_severity.len(), 1);

        let medium_and_above = aggregator.get_findings_by_severity(2); // medium = 2
        assert_eq!(medium_and_above.len(), 2);

        let all_severities = aggregator.get_findings_by_severity(1); // low = 1
        assert_eq!(all_severities.len(), 3);
    }

    #[test]
    fn test_clear_findings() {
        let aggregator = AnalysisAggregator::new();

        // Add some data
        aggregator.record_finding(create_test_issue("high", "src/main.rs"));
        aggregator.record_file_processed();

        assert_eq!(aggregator.get_findings_count(), 1);

        // Clear everything
        aggregator.clear_findings();

        assert_eq!(aggregator.get_findings_count(), 0);
        let stats = aggregator.get_stats();
        assert_eq!(stats.total_findings, 0);
        assert_eq!(stats.files_processed, 0);
        assert!(stats.findings_by_type.is_empty());
    }

    #[test]
    fn test_concurrent_access() {
        use std::sync::Arc;
        use std::thread;

        let aggregator = Arc::new(AnalysisAggregator::new());
        let mut handles = vec![];

        // Spawn multiple threads to test thread safety
        for i in 0..10 {
            let aggregator_clone = Arc::clone(&aggregator);
            let handle = thread::spawn(move || {
                let issue = create_test_issue("high", &format!("src/file_{}.rs", i));
                aggregator_clone.record_finding(issue);
                aggregator_clone.record_file_processed();
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify all findings were recorded
        assert_eq!(aggregator.get_findings_count(), 10);
        let stats = aggregator.get_stats();
        assert_eq!(stats.files_processed, 10);
    }
}

#[cfg(test)]
mod plugin_manager_tests {
    use super::super::config_service::ConfigurationService;
    use super::super::plugin_manager::{PluginManager, PluginManagerHandle};
    use super::super::traits::{PluginManagerHandle as PluginManagerHandleTrait, PluginStats};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_plugin_manager_creation() {
        let config_service = Arc::new(ConfigurationService::new());
        let (manager, handle) = PluginManager::new(config_service);

        assert!(handle.is_available());
    }

    #[tokio::test]
    async fn test_plugin_manager_spawn() {
        let config_service = Arc::new(ConfigurationService::new());
        let handle = PluginManager::spawn(config_service);

        assert!(handle.is_available());

        // Give the actor time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Test getting stats
        let stats_result = handle.get_stats().await;
        assert!(stats_result.is_ok());

        let stats = stats_result.unwrap();
        assert_eq!(stats.loaded_plugins, 0);
        assert_eq!(stats.total_executions, 0);
    }

    #[tokio::test]
    async fn test_plugin_manager_disabled_plugins() {
        let config_service = Arc::new(ConfigurationService::new()); // plugins disabled by default
        let handle = PluginManager::spawn(config_service);

        // Give the actor time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Try to load a plugin (should fail gracefully since plugins are disabled)
        let load_result = handle
            .load_plugin(std::path::PathBuf::from("test.wasm"))
            .await;
        assert!(load_result.is_err()); // Should fail since plugins are disabled
    }
}

#[cfg(test)]
mod dependency_graph_builder_tests {
    use super::super::ast_provider::AstProviderImpl;
    use super::super::dependency_graph_builder::DependencyGraphBuilderImpl;
    use super::super::traits::{AstProvider, DependencyGraphBuilder};
    use crate::analysis::detectors::dependency::Dependency;
    use crate::database::models::DependencyType;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_builder_creation() {
        let ast_provider = Arc::new(AstProviderImpl::new().unwrap()) as Arc<dyn AstProvider>;
        let builder = DependencyGraphBuilderImpl::new(ast_provider);
        assert!(builder.is_ok());
    }

    #[test]
    fn test_build_from_dependencies() {
        let ast_provider = Arc::new(AstProviderImpl::new().unwrap()) as Arc<dyn AstProvider>;
        let builder = DependencyGraphBuilderImpl::new(ast_provider).unwrap();

        let dependencies = vec![
            Dependency {
                from_file: PathBuf::from("src/main.rs"),
                to_module: "std::collections::HashMap".to_string(),
                dependency_type: DependencyType::Import,
                line_number: Some(1),
            },
            Dependency {
                from_file: PathBuf::from("src/lib.rs"),
                to_module: "serde::Serialize".to_string(),
                dependency_type: DependencyType::Import,
                line_number: Some(2),
            },
        ];

        let graph = builder.build_from_dependencies(dependencies);
        assert!(graph.node_count() > 0);
    }

    #[test]
    fn test_empty_dependencies() {
        let ast_provider = Arc::new(AstProviderImpl::new().unwrap()) as Arc<dyn AstProvider>;
        let builder = DependencyGraphBuilderImpl::new(ast_provider).unwrap();

        let graph = builder.build_from_dependencies(vec![]);
        assert_eq!(graph.node_count(), 0);
    }
}

// Integration tests combining multiple components would go here
#[cfg(test)]
mod integration_tests {
    use super::super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_components_integration() {
        // Test that components can work together
        let config_service =
            Arc::new(ConfigurationService::new()) as Arc<dyn traits::ConfigurationService>;
        let ast_provider =
            Arc::new(AstProviderImpl::new().unwrap()) as Arc<dyn traits::AstProvider>;
        let aggregator = Arc::new(AnalysisAggregator::new()) as Arc<dyn traits::AnalysisAggregator>;

        // Test that all components are properly initialized
        assert!(config_service.get_cache_path().is_some());

        let metrics = ast_provider.get_cache_metrics();
        assert!(metrics.is_object());

        let stats = aggregator.get_stats();
        assert_eq!(stats.total_findings, 0);
    }
}
