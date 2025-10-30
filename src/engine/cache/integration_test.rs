//! # Cache Integration Test
//!
//! Minimal test to verify cache execution layer is operational

#[cfg(test)]
mod tests {
    use super::super::*;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_cache_handles_clone() {
        let handles = crate::engine::analysis::context::CacheHandles {
            ast_cache: Arc::new(Mutex::new(AstCache::new(100))),
            analysis_cache: Arc::new(Mutex::new(AnalysisCache::new(50))),
        };

        // Clone should work
        let cloned = handles.clone();

        // Verify they point to same underlying cache
        assert!(Arc::ptr_eq(&handles.ast_cache, &cloned.ast_cache));
        assert!(Arc::ptr_eq(&handles.analysis_cache, &cloned.analysis_cache));
    }

    #[test]
    fn test_cache_service_manager_lifecycle() {
        let handles = crate::engine::analysis::context::CacheHandles {
            ast_cache: Arc::new(Mutex::new(AstCache::new(100))),
            analysis_cache: Arc::new(Mutex::new(AnalysisCache::new(50))),
        };

        let mut manager = CacheServiceManager::new(handles);

        // Start services
        manager
            .start_services(vec![PathBuf::from("/tmp/test")])
            .unwrap();

        // Verify running
        assert!(manager.is_running());

        // Stop services
        manager.stop_services();

        // Verify stopped
        assert!(!manager.is_running());
    }

    #[test]
    fn test_file_watcher_basic() {
        use std::time::Duration;

        let handles = crate::engine::analysis::context::CacheHandles {
            ast_cache: Arc::new(Mutex::new(AstCache::new(100))),
            analysis_cache: Arc::new(Mutex::new(AnalysisCache::new(50))),
        };

        let (mut watcher, _sender) = FileWatcher::new(Duration::from_secs(1));

        // Add path to watch
        watcher.watch_path(PathBuf::from("/tmp/test.rs"));

        // Poll should not error
        assert!(watcher.poll_changes().is_ok());
    }

    #[test]
    fn test_metrics_collector() {
        let mut collector = CacheMetricsCollector::new();

        // Record operations
        collector.record_hit("ast");
        collector.record_miss("ast");
        collector.record_hit("analysis");

        // Get metrics
        let metrics = collector.get_simplified_metrics();
        assert_eq!(metrics.total_hits, 2);
        assert_eq!(metrics.total_misses, 1);
    }
}
