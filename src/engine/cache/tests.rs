//! # Cache Execution Integration Tests
//!
//! Comprehensive tests for cache execution, service lifecycle, and integration.

#[cfg(test)]
mod cache_execution_tests {
    use crate::engine::analysis::context::{CacheHandles, FileInfo, ProjectContext};
    use crate::engine::analysis::pipeline::{AnalysisPipeline, Detector, PipelineError};
    use crate::engine::cache::{AnalysisCache, AstCache, CacheServiceManager};
    use crate::engine::parsing::AstBuilder;
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    /// Mock detector for testing
    struct MockDetector {
        name: String,
        cache_usage_count: Arc<Mutex<usize>>,
    }

    impl MockDetector {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                cache_usage_count: Arc::new(Mutex::new(0)),
            }
        }

        fn get_cache_usage_count(&self) -> usize {
            *self.cache_usage_count.lock().unwrap()
        }
    }

    impl Detector for MockDetector {
        fn detect(
            &self,
            context: &crate::engine::analysis::AnalysisContext,
        ) -> Result<Vec<crate::database::models::ArchitecturalIssue>, PipelineError> {
            // Check if context has cache handles
            #[cfg(feature = "analysis-cache")]
            if context.caches.is_some() {
                *self.cache_usage_count.lock().unwrap() += 1;
            }

            Ok(vec![])
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn supports_language(&self, _language: &crate::ast::SourceLanguage) -> bool {
            true
        }
    }

    #[test]
    #[cfg(feature = "analysis-cache")]
    fn test_cache_handles_clone() {
        // Create cache handles
        let handles = CacheHandles {
            ast_cache: Arc::new(Mutex::new(AstCache::new(100))),
            analysis_cache: Arc::new(Mutex::new(AnalysisCache::new(50))),
        };

        // Clone should work
        let cloned = handles.clone();

        // Both should point to the same underlying caches
        assert!(Arc::ptr_eq(&handles.ast_cache, &cloned.ast_cache));
        assert!(Arc::ptr_eq(&handles.analysis_cache, &cloned.analysis_cache));
    }

    #[test]
    #[cfg(feature = "analysis-cache")]
    fn test_cache_service_lifecycle() {
        // Create cache handles
        let handles = CacheHandles {
            ast_cache: Arc::new(Mutex::new(AstCache::new(100))),
            analysis_cache: Arc::new(Mutex::new(AnalysisCache::new(50))),
        };

        // Create service manager
        let mut manager = CacheServiceManager::new(handles);

        // Start services
        let watch_paths = vec![PathBuf::from("/tmp/test")];
        manager.start_services(watch_paths).unwrap();

        // Verify services are running
        assert!(manager.is_running());

        // Stop services
        manager.stop_services();

        // Verify services have stopped
        assert!(!manager.is_running());
    }

    #[test]
    #[cfg(feature = "analysis-cache")]
    fn test_pipeline_with_cache_services() {
        // Create AST builder
        let ast_builder = Arc::new(AstBuilder::new().unwrap());

        // Create cache handles
        let handles = CacheHandles {
            ast_cache: Arc::new(Mutex::new(AstCache::new(100))),
            analysis_cache: Arc::new(Mutex::new(AnalysisCache::new(50))),
        };

        // Create pipeline with caches
        let pipeline = AnalysisPipeline::with_caches(ast_builder, handles);

        // Start cache services
        let watch_paths = vec![PathBuf::from("/tmp/test")];
        pipeline.start_cache_services(watch_paths).unwrap();

        // Verify services are running
        assert!(pipeline.cache_services_running());

        // Stop services
        pipeline.stop_cache_services();

        // Verify services have stopped
        assert!(!pipeline.cache_services_running());
    }

    #[test]
    #[cfg(feature = "analysis-cache")]
    fn test_detector_receives_cache_context() {
        use crate::engine::analysis::AnalysisContext;

        // Create cache handles
        let handles = CacheHandles {
            ast_cache: Arc::new(Mutex::new(AstCache::new(100))),
            analysis_cache: Arc::new(Mutex::new(AnalysisCache::new(50))),
        };

        // Create mock detector
        let detector = MockDetector::new("test_detector");
        let usage_counter = detector.cache_usage_count.clone();

        // Create context with caches
        let file_info = FileInfo {
            path: PathBuf::from("/test/file.rs"),
            language: crate::ast::SourceLanguage::Rust,
            lines_of_code: 100,
            size_bytes: 1024,
            modified_at: std::time::SystemTime::now(),
        };

        let project_context = ProjectContext {
            project_root: PathBuf::from("/test"),
            project_files: vec![],
            dependencies: vec![],
            global_symbols: vec![],
        };

        let context = AnalysisContext::with_caches(
            file_info,
            None,
            String::new(),
            vec![],
            vec![],
            project_context,
            handles,
        );

        // Run detector
        detector.detect(&context).unwrap();

        // Verify detector received cache-enabled context
        assert_eq!(*usage_counter.lock().unwrap(), 1);
    }

    #[test]
    #[cfg(feature = "analysis-cache")]
    fn test_cache_hit_miss_recording() {
        use crate::engine::cache::CacheMetricsCollector;

        let mut collector = CacheMetricsCollector::new();

        // Record some operations
        collector.record_cache_operation("ast", "hit");
        collector.record_cache_operation("ast", "miss");
        collector.record_cache_operation("analysis", "hit");
        collector.record_cache_operation("analysis", "hit");

        // Generate report
        let report = collector.generate_report();

        // Verify report contains expected metrics
        assert!(report.contains("ast"));
        assert!(report.contains("analysis"));
        assert!(report.contains("hit"));
        assert!(report.contains("miss"));
    }

    #[test]
    #[cfg(feature = "analysis-cache")]
    fn test_file_watcher_invalidation() {
        use crate::engine::cache::FileWatcher;
        use std::fs;
        use tempfile::tempdir;

        // Create temporary directory
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() {}").unwrap();

        // Create cache handles
        let handles = CacheHandles {
            ast_cache: Arc::new(Mutex::new(AstCache::new(100))),
            analysis_cache: Arc::new(Mutex::new(AnalysisCache::new(50))),
        };

        // Add entry to AST cache
        {
            let mut ast_cache = handles.ast_cache.lock().unwrap();
            // Note: We're using a mock Tree here, in production this would be a real AST
            #[cfg(feature = "tree-sitter")]
            {
                // Create a simple parser for testing
                let mut parser = tree_sitter::Parser::new();
                let language = tree_sitter_rust::LANGUAGE;
                parser.set_language(&language.into()).unwrap();

                if let Some(tree) = parser.parse("fn main() {}", None) {
                    ast_cache
                        .put(test_file.clone(), tree, "fn main() {}".to_string())
                        .unwrap();
                }
            }
        }

        // Create file watcher
        let (mut watcher, _sender) = FileWatcher::new(Duration::from_millis(100));
        watcher.watch_path(test_file.clone());

        // Record initial timestamp
        watcher.update_timestamp(&test_file);

        // Modify file
        std::thread::sleep(Duration::from_millis(200));
        fs::write(&test_file, "fn main() { println!(\"modified\"); }").unwrap();

        // Poll for changes
        watcher.poll_changes().unwrap();

        // Verify cache was invalidated
        {
            let mut ast_cache = handles.ast_cache.lock().unwrap();
            let detector_versions: std::collections::HashMap<String, String> =
                std::collections::HashMap::new();
            assert!(ast_cache.get(&test_file).is_none());
        }
    }

    #[test]
    #[cfg(feature = "analysis-cache")]
    fn test_context_builder_cache_usage() {
        use crate::engine::analysis::pipeline::ContextBuilder;
        use std::fs;
        use tempfile::tempdir;

        // Create temporary test file
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() {}").unwrap();

        // Create AST builder
        let ast_builder = Arc::new(AstBuilder::new().unwrap());

        // Create cache handles
        let handles = CacheHandles {
            ast_cache: Arc::new(Mutex::new(AstCache::new(100))),
            analysis_cache: Arc::new(Mutex::new(AnalysisCache::new(50))),
        };

        // Create context builder with caches
        let builder = ContextBuilder::with_caches(ast_builder, handles.clone());

        // Build context (should populate cache)
        let project_context = ProjectContext {
            project_root: temp_dir.path().to_path_buf(),
            project_files: vec![test_file.clone()],
            dependencies: vec![],
            global_symbols: vec![],
        };

        let context1 = builder
            .build_context(&test_file, project_context.clone())
            .unwrap();

        // Build context again (should use cache)
        let context2 = builder.build_context(&test_file, project_context).unwrap();

        // Both contexts should have cache handles
        assert!(context1.caches.is_some());
        assert!(context2.caches.is_some());
    }

    #[test]
    #[cfg(feature = "analysis-cache")]
    fn test_parallel_cache_access() {
        use std::thread;

        // Create cache handles
        let handles = CacheHandles {
            ast_cache: Arc::new(Mutex::new(AstCache::new(100))),
            analysis_cache: Arc::new(Mutex::new(AnalysisCache::new(50))),
        };

        let mut threads = vec![];

        // Spawn multiple threads accessing the cache
        for i in 0..10 {
            let cache_clone = handles.clone();
            let handle = thread::spawn(move || {
                // Access AST cache
                if let Ok(mut ast_cache) = cache_clone.ast_cache.lock() {
                    // Simulate cache operation
                    let path = PathBuf::from(format!("/test/file{}.rs", i));
                    ast_cache.get(&path);
                }

                // Access analysis cache
                if let Ok(mut analysis_cache) = cache_clone.analysis_cache.lock() {
                    // Simulate cache operation
                    let key = format!("detector_{}", i);
                    let detector_versions: std::collections::HashMap<String, String> =
                        std::collections::HashMap::new();
                    analysis_cache.get(&PathBuf::from("/test"), &detector_versions);
                }
            });
            threads.push(handle);
        }

        // Wait for all threads to complete
        for handle in threads {
            handle.join().unwrap();
        }

        // Test passed if no deadlocks or panics occurred
    }
}
