//! # Cache Service Management
//!
//! Coordinates background cache services including file watching and metrics collection.

use super::{CacheMetricsCollector, FileWatcher};
use crate::engine::analysis::context::CacheHandles;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;
use tracing::{error, info, warn};

/// Cache service manager that coordinates background tasks
pub struct CacheServiceManager {
    /// File watcher thread handle
    watcher_handle: Option<JoinHandle<()>>,

    /// Metrics collector thread handle
    metrics_handle: Option<JoinHandle<()>>,

    /// Cache handles shared with services
    cache_handles: CacheHandles,

    /// Shared shutdown signal
    shutdown_signal: Arc<std::sync::atomic::AtomicBool>,
}

impl CacheServiceManager {
    /// Create new cache service manager
    pub fn new(cache_handles: CacheHandles) -> Self {
        Self {
            watcher_handle: None,
            metrics_handle: None,
            cache_handles,
            shutdown_signal: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Start all cache services
    pub fn start_services(&mut self, watch_paths: Vec<PathBuf>) -> Result<(), String> {
        // Start file watcher if not already running
        if self.watcher_handle.is_none() {
            self.start_file_watcher(watch_paths)?;
        }

        // Start metrics collector if not already running
        if self.metrics_handle.is_none() {
            self.start_metrics_collector()?;
        }

        Ok(())
    }

    /// Start file watcher service
    fn start_file_watcher(&mut self, paths: Vec<PathBuf>) -> Result<(), String> {
        let cache_handles = self.cache_handles.clone();
        let shutdown_signal = Arc::clone(&self.shutdown_signal);

        // Create file watcher
        let (mut watcher, _event_sender) = FileWatcher::new(
            Duration::from_secs(2), // Poll every 2 seconds
        );

        // Configure watcher with cache handles
        #[cfg(feature = "ast-cache")]
        {
            watcher = watcher.with_ast_cache(cache_handles.ast_cache.clone());
        }

        #[cfg(feature = "analysis-cache")]
        {
            watcher = watcher.with_analysis_cache(cache_handles.analysis_cache.clone());
        }

        // Add paths to watch
        for path in paths {
            watcher.watch_path(path);
        }

        // Spawn watcher thread
        let handle = std::thread::spawn(move || {
            info!("Cache file watcher service started");

            while !shutdown_signal.load(std::sync::atomic::Ordering::Relaxed) {
                // Poll for changes
                if let Err(e) = watcher.poll_changes() {
                    error!("File watcher error: {}", e);
                }

                // Sleep before next poll
                std::thread::sleep(Duration::from_secs(1));
            }

            info!("Cache file watcher service stopped");
        });

        self.watcher_handle = Some(handle);
        Ok(())
    }

    /// Start metrics collection service
    fn start_metrics_collector(&mut self) -> Result<(), String> {
        let cache_handles = self.cache_handles.clone();
        let shutdown_signal = Arc::clone(&self.shutdown_signal);

        // Create metrics collector
        let mut collector = CacheMetricsCollector::new();

        // Spawn metrics thread
        let handle = std::thread::spawn(move || {
            info!("Cache metrics collector service started");

            while !shutdown_signal.load(std::sync::atomic::Ordering::Relaxed) {
                // Collect metrics
                if let Ok(ast_cache) = cache_handles.ast_cache.lock() {
                    collector.record_cache_operation("ast", "access");
                    drop(ast_cache); // Release lock quickly
                }

                if let Ok(analysis_cache) = cache_handles.analysis_cache.lock() {
                    collector.record_cache_operation("analysis", "access");
                    drop(analysis_cache); // Release lock quickly
                }

                // Report metrics periodically (every 30 seconds)
                if collector.should_report() {
                    let report = collector.generate_report();
                    info!("Cache metrics: {}", report);
                }

                // Sleep before next collection
                std::thread::sleep(Duration::from_secs(5));
            }

            // Final metrics report
            let final_report = collector.generate_report();
            info!("Final cache metrics: {}", final_report);
            info!("Cache metrics collector service stopped");
        });

        self.metrics_handle = Some(handle);
        Ok(())
    }

    /// Add a path to watch
    pub fn add_watch_path(&self, path: PathBuf) {
        // This would require more complex communication with the watcher thread
        // For now, paths must be specified at startup
        warn!(
            "Dynamic path addition not yet supported. Path '{}' not added.",
            path.display()
        );
    }

    /// Check if services are running
    pub fn is_running(&self) -> bool {
        let watcher_running = self
            .watcher_handle
            .as_ref()
            .map(|h| !h.is_finished())
            .unwrap_or(false);

        let metrics_running = self
            .metrics_handle
            .as_ref()
            .map(|h| !h.is_finished())
            .unwrap_or(false);

        watcher_running && metrics_running
    }

    /// Stop all services gracefully
    pub fn stop_services(&mut self) {
        info!("Stopping cache services...");

        // Signal shutdown
        self.shutdown_signal
            .store(true, std::sync::atomic::Ordering::Relaxed);

        // Wait for watcher thread to finish
        if let Some(handle) = self.watcher_handle.take() {
            if let Err(e) = handle.join() {
                error!("Error joining watcher thread: {:?}", e);
            }
        }

        // Wait for metrics thread to finish
        if let Some(handle) = self.metrics_handle.take() {
            if let Err(e) = handle.join() {
                error!("Error joining metrics thread: {:?}", e);
            }
        }

        info!("Cache services stopped");
    }
}

impl Drop for CacheServiceManager {
    fn drop(&mut self) {
        // Ensure services are stopped on drop
        self.stop_services();
    }
}

// Extension to CacheMetricsCollector for periodic reporting
impl CacheMetricsCollector {
    /// Check if it's time to report metrics (every 30 seconds)
    fn should_report(&self) -> bool {
        // Simple implementation - in production, track last report time
        static REPORT_COUNTER: std::sync::atomic::AtomicUsize =
            std::sync::atomic::AtomicUsize::new(0);

        let count = REPORT_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        count % 6 == 0 // Report every 6 polls (30 seconds at 5-second intervals)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::cache::{AnalysisCache, AstCache};
    use std::sync::Mutex;

    #[test]
    fn test_service_lifecycle() {
        // Create test cache handles
        let cache_handles = CacheHandles {
            ast_cache: Arc::new(Mutex::new(AstCache::new(100))),
            analysis_cache: Arc::new(Mutex::new(AnalysisCache::new(50))),
        };

        // Create service manager
        let mut manager = CacheServiceManager::new(cache_handles);

        // Start services
        let paths = vec![PathBuf::from("/tmp/test")];
        manager.start_services(paths).unwrap();

        // Check services are running
        assert!(manager.is_running());

        // Stop services
        manager.stop_services();

        // Check services have stopped
        assert!(!manager.is_running());
    }
}
