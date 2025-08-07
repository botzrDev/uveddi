//! Arena allocation for transient analysis objects
//! Based on UV-210 research: "Arena for Computation, Owned for Results" pattern with bumpalo-herd

#![cfg(feature = "memory-optimization")]

use crate::analysis::memory::metrics::BASIC_MEMORY_METRICS;
use crate::database::models::ArchitecturalIssue;
use bumpalo::Bump;
use bumpalo_herd::Herd;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// High-level arena manager for concurrent file analysis
/// Uses the bumpalo-herd pattern for optimal performance and thread safety
pub struct AnalysisArenaManager {
    herd: Arc<Herd>,
    default_arena_capacity: usize,
    total_arenas_created: std::sync::atomic::AtomicUsize,
}

impl AnalysisArenaManager {
    /// Create a new arena manager with default settings
    pub fn new() -> Self {
        Self {
            herd: Arc::new(Herd::new()),
            default_arena_capacity: 32 * 1024 * 1024, // 32MB default
            total_arenas_created: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Create arena manager with custom capacity
    pub fn with_capacity(default_capacity_bytes: usize) -> Self {
        Self {
            herd: Arc::new(Herd::new()),
            default_arena_capacity: default_capacity_bytes,
            total_arenas_created: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Get a borrowed arena for single-file analysis
    /// This is thread-safe and contention-free
    pub fn get_arena(&self) -> ArenaHandle<'_> {
        let arena = self.herd.get();
        self.total_arenas_created
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        ArenaHandle {
            arena,
            manager: Arc::clone(&self.herd),
            bytes_allocated: 0,
        }
    }

    /// Get statistics about arena usage
    pub fn get_stats(&self) -> ArenaManagerStats {
        ArenaManagerStats {
            total_arenas_created: self
                .total_arenas_created
                .load(std::sync::atomic::Ordering::Relaxed),
            active_arenas: 0, // bumpalo-herd doesn't expose len(), set to 0 for now
            default_capacity_mb: self.default_arena_capacity / (1024 * 1024),
        }
    }

    /// Export metrics for observability
    pub fn export_metrics(&self) -> serde_json::Value {
        let stats = self.get_stats();

        serde_json::json!({
            "arena_manager": {
                "pattern": "bumpalo-herd",
                "total_arenas_created": stats.total_arenas_created,
                "active_arenas": stats.active_arenas,
                "default_capacity_mb": stats.default_capacity_mb,
                "thread_safe": true,
                "contention_free": true
            }
        })
    }
}

impl Default for AnalysisArenaManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle to a borrowed arena from the herd
/// Automatically returned to pool when dropped
pub struct ArenaHandle<'a> {
    arena: bumpalo_herd::Member<'a>,
    manager: Arc<Herd>,
    bytes_allocated: usize,
}

impl<'a> ArenaHandle<'a> {
    /// Get the underlying bump allocator
    /// Use this for all temporary allocations during analysis  
    pub fn bump(&self) -> &Bump {
        self.arena.as_bump()
    }

    /// Allocate a value in the arena (temporary, computation phase)
    pub fn alloc<T>(&mut self, value: T) -> &mut T {
        let allocated = self.arena.alloc(value);
        self.bytes_allocated += std::mem::size_of::<T>();
        allocated
    }

    /// Allocate a string in the arena (temporary, computation phase)
    pub fn alloc_str(&mut self, s: &str) -> &mut str {
        let allocated = self.arena.alloc_str(s);
        self.bytes_allocated += s.len();
        allocated
    }

    /// Create an arena-backed vector (temporary, computation phase)
    pub fn create_vec<T>(&self) -> bumpalo::collections::Vec<T> {
        bumpalo::collections::Vec::new_in(self.arena.as_bump())
    }

    /// Create an arena-backed string (temporary, computation phase)
    pub fn create_string(&self) -> bumpalo::collections::String {
        bumpalo::collections::String::new_in(self.arena.as_bump())
    }

    /// Get bytes allocated in this arena session
    pub fn bytes_allocated(&self) -> usize {
        self.bytes_allocated
    }

    /// Reset the arena for reuse (typically done automatically by herd)
    pub fn reset(&mut self) {
        // Note: bumpalo-herd handles reset automatically, this is just for our tracking
        self.bytes_allocated = 0;
    }
}

/// Statistics for the arena manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArenaManagerStats {
    pub total_arenas_created: usize,
    pub active_arenas: usize,
    pub default_capacity_mb: usize,
}

/// Analysis result structure following "Arena for Computation, Owned for Results" pattern
/// This struct must be Send and contain no lifetimes tied to temporary arenas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArenaAnalysisResult {
    /// File that was analyzed
    pub file_path: String,

    /// Owned issues found during analysis (converted from arena-allocated temporary structures)
    pub issues: Vec<ArchitecturalIssue>,

    /// Owned metadata about the analysis
    pub analysis_metadata: AnalysisMetadata,

    /// Performance metrics for this analysis
    pub performance_metrics: AnalysisPerformanceMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisMetadata {
    pub lines_of_code: usize,
    pub functions_found: usize,
    pub complexity_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisPerformanceMetrics {
    pub arena_bytes_used: usize,
    pub analysis_duration_ms: u64,
    pub memory_efficiency: f64,
}

/// Helper functions for converting arena-allocated data to owned data
pub mod conversion {
    use super::*;
    use bumpalo::collections::{String as ArenaString, Vec as ArenaVec};

    /// Convert arena-allocated vector of issues to owned vector
    /// This is the critical "Communication Phase" that converts temporary data to Send-able results
    pub fn arena_issues_to_owned(
        arena_issues: ArenaVec<ArchitecturalIssue>,
    ) -> Vec<ArchitecturalIssue> {
        arena_issues.into_iter().collect()
    }

    /// Convert arena-allocated strings to owned strings
    pub fn arena_strings_to_owned(arena_strings: ArenaVec<ArenaString>) -> Vec<String> {
        arena_strings.into_iter().map(|s| s.to_string()).collect()
    }

    /// Convert arena string to owned string
    pub fn arena_string_to_owned(arena_string: ArenaString) -> String {
        arena_string.to_string()
    }

    /// Helper to build final analysis result from arena-computed data
    pub fn build_analysis_result(
        file_path: String,
        arena_issues: ArenaVec<ArchitecturalIssue>,
        metadata: AnalysisMetadata,
        performance: AnalysisPerformanceMetrics,
    ) -> ArenaAnalysisResult {
        ArenaAnalysisResult {
            file_path,
            issues: arena_issues_to_owned(arena_issues),
            analysis_metadata: metadata,
            performance_metrics: performance,
        }
    }
}

/// Integration helpers for rayon-based concurrent analysis
pub mod rayon_integration {
    use super::*;
    use rayon::prelude::*;

    /// Analyze multiple files concurrently using the arena pool pattern
    /// This demonstrates the recommended architecture from UV-210 research
    pub fn analyze_files_concurrent<F>(
        file_paths: Vec<String>,
        arena_manager: &AnalysisArenaManager,
        analyzer: F,
    ) -> Vec<ArenaAnalysisResult>
    where
        F: Fn(&mut ArenaHandle, &str) -> ArenaAnalysisResult + Sync + Send,
    {
        file_paths
            .par_iter()
            .map_init(
                || arena_manager.get_arena(),
                |arena_handle, file_path| {
                    // Each parallel task gets its own arena from the herd
                    // This is contention-free and provides optimal performance
                    analyzer(arena_handle, file_path)
                },
            )
            .collect()
    }

    /// Example analysis function that follows the recommended pattern
    pub fn example_file_analyzer(
        arena_handle: &mut ArenaHandle,
        file_path: &str,
    ) -> ArenaAnalysisResult {
        let start_time = std::time::Instant::now();

        // Phase 1: Computation using arena (temporary allocations)
        // Simulate some arena allocations to demonstrate the pattern
        for i in 0..10 {
            // Allocate temporary data in arena to simulate computation
            let _temp_data = arena_handle.alloc([0u8; 64]);
            let _temp_string = arena_handle.alloc_str("temporary analysis data");
        }

        // Create issues using standard allocations (the arena was used for temporary computation above)
        let mut temp_issues = arena_handle.create_vec();
        for i in 0..10 {
            temp_issues.push(ArchitecturalIssue {
                issue_id: Some(i as i64),
                analysis_run_id: 1,
                anti_pattern_type_id: 1,
                file_path: file_path.to_string(),
                start_line: Some(i as i32),
                end_line: Some(i as i32 + 5),
                line_number: Some(i as i32),
                column_number: Some(1),
                message: format!("Issue {} in {}", i, file_path),
                metadata: "{}".to_string(),
                detector_name: "arena_test".to_string(),
                created_at: chrono::Utc::now(),
                severity: "medium".to_string(),
                description: format!("Issue {} in {}", i, file_path),
                code_snippet: Some("sample code".to_string()),
                ai_explanation: None,
            });
        }

        // Phase 2: Convert to owned result (communication phase)
        let analysis_duration = start_time.elapsed();
        let bytes_used = arena_handle.bytes_allocated();

        conversion::build_analysis_result(
            file_path.to_string(),
            temp_issues,
            AnalysisMetadata {
                lines_of_code: 100,
                functions_found: 5,
                complexity_score: 2.5,
            },
            AnalysisPerformanceMetrics {
                arena_bytes_used: bytes_used,
                analysis_duration_ms: analysis_duration.as_millis() as u64,
                memory_efficiency: 0.85,
            },
        )
    }
}

// Global arena manager instance following the recommended pattern
lazy_static::lazy_static! {
    pub static ref GLOBAL_ARENA_MANAGER: AnalysisArenaManager = AnalysisArenaManager::new();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_manager_creation() {
        let manager = AnalysisArenaManager::new();
        let stats = manager.get_stats();

        assert_eq!(stats.total_arenas_created, 0);
        assert_eq!(stats.active_arenas, 0);
        assert_eq!(stats.default_capacity_mb, 32);
    }

    #[test]
    fn test_arena_handle_allocation() {
        let manager = AnalysisArenaManager::new();
        let mut handle = manager.get_arena();

        // Test basic allocation
        let value = handle.alloc(42i32);
        assert_eq!(*value, 42);

        // Test string allocation
        let text = handle.alloc_str("test string");
        assert_eq!(text, "test string");

        // Test vector creation
        let mut vec = handle.create_vec();
        vec.push("item1".to_string());
        vec.push("item2".to_string());
        assert_eq!(vec.len(), 2);

        // Check bytes allocated
        assert!(handle.bytes_allocated() > 0);
    }

    #[test]
    fn test_arena_conversion_helpers() {
        let manager = AnalysisArenaManager::new();
        let handle = manager.get_arena();

        // Create arena-allocated data
        let mut arena_issues = handle.create_vec();
        arena_issues.push(ArchitecturalIssue {
            issue_id: Some(1),
            analysis_run_id: 1,
            anti_pattern_type_id: 1,
            file_path: "test.rs".to_string(),
            start_line: Some(1),
            end_line: Some(10),
            severity: "high".to_string(),
            description: "test issue".to_string(),
            code_snippet: Some("test code".to_string()),
            ai_explanation: None,
        });

        // Convert to owned
        let owned_issues = conversion::arena_issues_to_owned(arena_issues);
        assert_eq!(owned_issues.len(), 1);
        assert_eq!(owned_issues[0].file_path, "test.rs");
    }

    #[test]
    fn test_analysis_result_structure() {
        let result = ArenaAnalysisResult {
            file_path: "example.rs".to_string(),
            issues: vec![],
            analysis_metadata: AnalysisMetadata {
                lines_of_code: 50,
                functions_found: 3,
                complexity_score: 1.5,
            },
            performance_metrics: AnalysisPerformanceMetrics {
                arena_bytes_used: 1024,
                analysis_duration_ms: 10,
                memory_efficiency: 0.9,
            },
        };

        // Verify it's Send (can be passed between threads)
        fn assert_send<T: Send>(_: &T) {}
        assert_send(&result);

        // Verify it can be serialized
        let _json = serde_json::to_string(&result).unwrap();
    }

    #[test]
    fn test_concurrent_arena_access() {
        use rayon::prelude::*;

        let manager = AnalysisArenaManager::new();
        let files = vec!["file1.rs", "file2.rs", "file3.rs", "file4.rs"];

        // This demonstrates the recommended concurrent pattern
        let results: Vec<ArenaAnalysisResult> = files
            .par_iter()
            .map_init(
                || manager.get_arena(),
                |arena_handle, file_path| {
                    rayon_integration::example_file_analyzer(arena_handle, file_path)
                },
            )
            .collect();

        assert_eq!(results.len(), 4);
        for (i, result) in results.iter().enumerate() {
            assert!(result.file_path.contains(&format!("file{}", i + 1)));
            assert_eq!(result.issues.len(), 10);
            assert!(result.performance_metrics.arena_bytes_used > 0);
        }
    }

    #[test]
    fn test_arena_manager_stats() {
        let manager = AnalysisArenaManager::new();

        // Get some arenas
        let _handle1 = manager.get_arena();
        let _handle2 = manager.get_arena();

        let stats = manager.get_stats();
        assert_eq!(stats.total_arenas_created, 2);

        // Export metrics
        let metrics = manager.export_metrics();
        assert_eq!(metrics["arena_manager"]["total_arenas_created"], 2);
        assert_eq!(metrics["arena_manager"]["pattern"], "bumpalo-herd");
        assert_eq!(metrics["arena_manager"]["thread_safe"], true);
        assert_eq!(metrics["arena_manager"]["contention_free"], true);
    }

    #[test]
    fn test_global_arena_manager() {
        let stats = GLOBAL_ARENA_MANAGER.get_stats();
        assert_eq!(stats.default_capacity_mb, 32);

        let _handle = GLOBAL_ARENA_MANAGER.get_arena();
        let updated_stats = GLOBAL_ARENA_MANAGER.get_stats();
        assert!(updated_stats.total_arenas_created > stats.total_arenas_created);
    }
}
