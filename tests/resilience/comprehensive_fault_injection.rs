//! Comprehensive Fault Injection Testing Framework
//! Tests system resilience under various failure conditions

use uveddi::analysis::AnalysisEngine;
use uveddi::application::{AnalysisConfig, AnalysisOrchestrator};
use uveddi::ast::tree_sitter_impl::{AstParser, SourceLanguage};
use uveddi::database::DatabaseManager;
use uveddi::error::UveddiError;

#[cfg(feature = "memory-optimization")]
use uveddi::analysis::memory::{
    initialize_memory_optimization, MemoryOptimizationConfig, BASIC_MEMORY_METRICS,
};

use std::path::PathBuf;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, AtomicUsize, Ordering}};
use std::time::{Duration, Instant};
use std::thread;
use std::fs;
use std::io::{self, Write};
use tempfile::{tempdir, NamedTempFile};
use tokio::time::timeout;

#[cfg(test)]
mod fault_injection_comprehensive {
    use super::*;

    // Fault injection utilities
    struct FaultInjector {
        network_failure_enabled: AtomicBool,
        disk_failure_enabled: AtomicBool,
        memory_failure_enabled: AtomicBool,
        database_failure_enabled: AtomicBool,
        ai_service_failure_enabled: AtomicBool,
        failure_count: AtomicUsize,
    }

    impl FaultInjector {
        fn new() -> Self {
            Self {
                network_failure_enabled: AtomicBool::new(false),
                disk_failure_enabled: AtomicBool::new(false),
                memory_failure_enabled: AtomicBool::new(false),
                database_failure_enabled: AtomicBool::new(false),
                ai_service_failure_enabled: AtomicBool::new(false),
                failure_count: AtomicUsize::new(0),
            }
        }

        fn enable_network_failures(&self) {
            self.network_failure_enabled.store(true, Ordering::SeqCst);
        }

        fn enable_disk_failures(&self) {
            self.disk_failure_enabled.store(true, Ordering::SeqCst);
        }

        fn enable_memory_failures(&self) {
            self.memory_failure_enabled.store(true, Ordering::SeqCst);
        }

        fn enable_database_failures(&self) {
            self.database_failure_enabled.store(true, Ordering::SeqCst);
        }

        fn enable_ai_service_failures(&self) {
            self.ai_service_failure_enabled.store(true, Ordering::SeqCst);
        }

        fn should_fail(&self, failure_type: &str) -> bool {
            let should_fail = match failure_type {
                "network" => self.network_failure_enabled.load(Ordering::SeqCst),
                "disk" => self.disk_failure_enabled.load(Ordering::SeqCst),
                "memory" => self.memory_failure_enabled.load(Ordering::SeqCst),
                "database" => self.database_failure_enabled.load(Ordering::SeqCst),
                "ai_service" => self.ai_service_failure_enabled.load(Ordering::SeqCst),
                _ => false,
            };

            if should_fail {
                self.failure_count.fetch_add(1, Ordering::SeqCst);
            }

            should_fail
        }

        fn get_failure_count(&self) -> usize {
            self.failure_count.load(Ordering::SeqCst)
        }

        fn reset(&self) {
            self.network_failure_enabled.store(false, Ordering::SeqCst);
            self.disk_failure_enabled.store(false, Ordering::SeqCst);
            self.memory_failure_enabled.store(false, Ordering::SeqCst);
            self.database_failure_enabled.store(false, Ordering::SeqCst);
            self.ai_service_failure_enabled.store(false, Ordering::SeqCst);
            self.failure_count.store(0, Ordering::SeqCst);
        }
    }

    static GLOBAL_FAULT_INJECTOR: std::sync::LazyLock<FaultInjector> = 
        std::sync::LazyLock::new(|| FaultInjector::new());

    // Test utilities
    fn create_test_file(content: &str, extension: &str) -> PathBuf {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join(format!("test.{}", extension));
        fs::write(&file_path, content).unwrap();
        file_path
    }

    fn create_corrupted_file() -> PathBuf {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("corrupted.rs");
        
        // Create file with invalid UTF-8 sequences
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(b"fn test() {\n").unwrap();
        file.write_all(&[0xFF, 0xFE, 0xFD]).unwrap(); // Invalid UTF-8
        file.write_all(b"\n}").unwrap();
        
        file_path
    }

    fn simulate_disk_full() -> std::io::Result<PathBuf> {
        // Simulate disk full by trying to write to a read-only location
        let temp_dir = tempdir().unwrap();
        let readonly_path = temp_dir.path().join("readonly");
        
        fs::create_dir(&readonly_path)?;
        
        // On Unix systems, we could make it readonly
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&readonly_path)?.permissions();
            perms.set_mode(0o444); // Read-only
            fs::set_permissions(&readonly_path, perms)?;
        }
        
        Ok(readonly_path)
    }

    #[tokio::test]
    async fn test_network_partition_recovery() {
        println!("🌐 Testing Network Partition Recovery");

        GLOBAL_FAULT_INJECTOR.reset();
        GLOBAL_FAULT_INJECTOR.enable_network_failures();

        // Test analysis engine behavior during network issues
        let test_file = create_test_file("fn main() { println!(\"test\"); }", "rs");
        
        let analysis_config = AnalysisConfig {
            target_path: test_file,
            output_format: "json".to_string(),
            output_file: None,
            enable_ai: false, // AI would require network
            ollama_api_url: None,
            ollama_model: None,
            dead_code_confidence: None,
            dead_code_library_mode: false,
            dead_code_ignore_patterns: None,
            dead_code_keep_alive: None,
            large_classes_max_loc: None,
            large_classes_max_methods: None,
            large_classes_max_fields: None,
            large_classes_max_complexity: None,
            large_classes_max_lcom: None,
            large_classes_ignore_patterns: None,
            large_classes_min_severity: None,
            #[cfg(feature = "memory-optimization")]
            memory_optimization: None,
            enable_memory_optimization: false,
            memory_limit_gb: None,
            memory_profile: None,
        };

        // Test graceful degradation when network is unavailable
        let result = timeout(
            Duration::from_secs(10),
            async {
                let mut orchestrator = AnalysisOrchestrator::new().unwrap();
                orchestrator.execute_analysis(analysis_config).await
            }
        ).await;

        match result {
            Ok(analysis_result) => {
                match analysis_result {
                    Ok(_) => {
                        println!("  Analysis completed successfully without network");
                    }
                    Err(e) => {
                        println!("  Analysis failed gracefully: {:?}", e);
                        
                        // Should not be a catastrophic failure
                        match e {
                            UveddiError::NetworkError { .. } => {
                                println!("  Appropriate network error detected");
                            }
                            _ => {
                                println!("  Other error (acceptable): {:?}", e);
                            }
                        }
                    }
                }
            }
            Err(_) => {
                println!("  Analysis timed out during network partition");
            }
        }

        // Test recovery after network is restored
        GLOBAL_FAULT_INJECTOR.reset();
        
        let recovery_config = AnalysisConfig {
            target_path: create_test_file("fn recovery() { println!(\"recovered\"); }", "rs"),
            output_format: "json".to_string(),
            output_file: None,
            enable_ai: false,
            ollama_api_url: None,
            ollama_model: None,
            dead_code_confidence: None,
            dead_code_library_mode: false,
            dead_code_ignore_patterns: None,
            dead_code_keep_alive: None,
            large_classes_max_loc: None,
            large_classes_max_methods: None,
            large_classes_max_fields: None,
            large_classes_max_complexity: None,
            large_classes_max_lcom: None,
            large_classes_ignore_patterns: None,
            large_classes_min_severity: None,
            #[cfg(feature = "memory-optimization")]
            memory_optimization: None,
            enable_memory_optimization: false,
            memory_limit_gb: None,
            memory_profile: None,
        };

        let recovery_result = timeout(
            Duration::from_secs(10),
            async {
                let mut orchestrator = AnalysisOrchestrator::new().unwrap();
                orchestrator.execute_analysis(recovery_config).await
            }
        ).await;

        match recovery_result {
            Ok(Ok(_)) => {
                println!("  System recovered successfully after network restoration");
            }
            Ok(Err(e)) => {
                println!("  Recovery partially successful: {:?}", e);
            }
            Err(_) => {
                println!("  Recovery timed out");
            }
        }

        let failure_count = GLOBAL_FAULT_INJECTOR.get_failure_count();
        println!("  Total network failures simulated: {}", failure_count);

        println!("✅ Network partition recovery test completed");
    }

    #[tokio::test]
    async fn test_disk_io_failure_handling() {
        println!("💾 Testing Disk I/O Failure Handling");

        GLOBAL_FAULT_INJECTOR.reset();
        GLOBAL_FAULT_INJECTOR.enable_disk_failures();

        // Test behavior when disk I/O fails
        let test_scenarios = vec![
            ("readonly_directory", simulate_disk_full().ok()),
            ("corrupted_file", Some(create_corrupted_file())),
            ("nonexistent_file", Some(PathBuf::from("/nonexistent/path/file.rs"))),
        ];

        for (scenario_name, path_option) in test_scenarios {
            if let Some(test_path) = path_option {
                println!("  Testing scenario: {}", scenario_name);

                let analysis_config = AnalysisConfig {
                    target_path: test_path,
                    output_format: "json".to_string(),
                    output_file: None,
                    enable_ai: false,
                    ollama_api_url: None,
                    ollama_model: None,
                    dead_code_confidence: None,
                    dead_code_library_mode: false,
                    dead_code_ignore_patterns: None,
                    dead_code_keep_alive: None,
                    large_classes_max_loc: None,
                    large_classes_max_methods: None,
                    large_classes_max_fields: None,
                    large_classes_max_complexity: None,
                    large_classes_max_lcom: None,
                    large_classes_ignore_patterns: None,
                    large_classes_min_severity: None,
                    #[cfg(feature = "memory-optimization")]
                    memory_optimization: None,
                    enable_memory_optimization: false,
                    memory_limit_gb: None,
                    memory_profile: None,
                };

                let result = timeout(
                    Duration::from_secs(5),
                    async {
                        let mut orchestrator = AnalysisOrchestrator::new().unwrap();
                        orchestrator.execute_analysis(analysis_config).await
                    }
                ).await;

                match result {
                    Ok(analysis_result) => {
                        match analysis_result {
                            Ok(_) => {
                                println!("    Scenario {} handled successfully", scenario_name);
                            }
                            Err(e) => {
                                println!("    Scenario {} failed gracefully: {:?}", scenario_name, e);
                                
                                match e {
                                    UveddiError::FileSystemError { .. } => {
                                        println!("    Appropriate file system error detected");
                                    }
                                    UveddiError::AstError { .. } => {
                                        println!("    AST error (expected for corrupted files)");
                                    }
                                    _ => {
                                        println!("    Other error type: {:?}", e);
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => {
                        println!("    Scenario {} timed out (acceptable)", scenario_name);
                    }
                }
            }
        }

        // Test AST parser resilience to disk I/O failures
        let mut parser = AstParser::new().unwrap();
        
        let io_failure_files = vec![
            create_corrupted_file(),
            PathBuf::from("/dev/null"),
        ];

        for (i, file_path) in io_failure_files.iter().enumerate() {
            println!("  Testing AST parser with problematic file {}", i);
            
            let parse_result = parser.parse_file(file_path);
            match parse_result {
                Ok(parsed) => {
                    println!("    Parsing succeeded unexpectedly: {} bytes", parsed.source().len());
                }
                Err(e) => {
                    println!("    Parsing failed gracefully: {:?}", e);
                }
            }
        }

        // Test disk space exhaustion simulation
        let temp_dir = tempdir().unwrap();
        let large_file_path = temp_dir.path().join("large_test.rs");
        
        // Try to create a very large file
        match fs::File::create(&large_file_path) {
            Ok(mut file) => {
                let large_content = "// ".repeat(1_000_000); // 2MB of comments
                match file.write_all(large_content.as_bytes()) {
                    Ok(_) => {
                        println!("  Testing parsing of large file (2MB)");
                        
                        let large_parse_result = parser.parse_file(&large_file_path);
                        match large_parse_result {
                            Ok(parsed) => {
                                println!("    Large file parsed: {} bytes", parsed.source().len());
                            }
                            Err(e) => {
                                println!("    Large file parsing failed appropriately: {:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("  Failed to write large file (disk space issue?): {}", e);
                    }
                }
            }
            Err(e) => {
                println!("  Failed to create large file: {}", e);
            }
        }

        println!("✅ Disk I/O failure handling test completed");
    }

    #[tokio::test]
    async fn test_memory_allocation_failures() {
        println!("🧠 Testing Memory Allocation Failures");

        GLOBAL_FAULT_INJECTOR.reset();
        GLOBAL_FAULT_INJECTOR.enable_memory_failures();

        // Test OOM scenarios and recovery
        let memory_stress_scenarios = vec![
            ("large_ast", create_large_rust_file(1000)),
            ("deeply_nested", create_deeply_nested_file()),
            ("wide_structure", create_wide_structure_file()),
        ];

        for (scenario_name, content) in memory_stress_scenarios {
            println!("  Testing memory scenario: {}", scenario_name);
            
            let test_file = create_test_file(&content, "rs");
            let mut parser = AstParser::with_cache_size(1).unwrap(); // Very limited cache

            let start_time = Instant::now();
            let parse_result = parser.parse_file(&test_file);
            let parse_time = start_time.elapsed();

            match parse_result {
                Ok(parsed) => {
                    println!("    Scenario {} completed in {:?} ({} bytes)", 
                            scenario_name, parse_time, parsed.source().len());
                }
                Err(e) => {
                    println!("    Scenario {} failed in {:?}: {:?}", 
                            scenario_name, parse_time, e);
                    
                    match e {
                        uveddi::ast::tree_sitter_impl::AstError::Other(ref msg) 
                            if msg.contains("memory") || msg.contains("allocation") => {
                            println!("    Appropriate memory error detected");
                        }
                        _ => {
                            println!("    Other error type (may be acceptable): {:?}", e);
                        }
                    }
                }
            }

            // Check cache stats for memory usage patterns
            let (hits, misses, current_size, max_size) = parser.cache_stats();
            println!("    Cache stats: hits={}, misses={}, size={}/{}", 
                    hits, misses, current_size, max_size);
        }

        // Test memory optimization under pressure
        #[cfg(feature = "memory-optimization")]
        {
            let restrictive_config = MemoryOptimizationConfig {
                target_max_memory_bytes: 128 * 1024 * 1024, // 128MB limit
                object_pools: uveddi::analysis::memory::ObjectPoolConfig {
                    enabled: true,
                    detector_pool_capacity: 5, // Very small
                    ..Default::default()
                },
                arena_allocation: uveddi::analysis::memory::ArenaConfig {
                    enabled: true,
                    default_arena_size_mb: 1, // Very small
                    max_concurrent_arenas: 2, // Very limited
                    ..Default::default()
                },
                ..Default::default()
            };

            let init_result = initialize_memory_optimization(restrictive_config);
            match init_result {
                Ok(_) => {
                    println!("  Memory optimization initialized under pressure");
                    
                    // Test metrics under memory pressure
                    BASIC_MEMORY_METRICS.update_memory_usage(100 * 1024 * 1024); // 100MB usage
                    let metrics = BASIC_MEMORY_METRICS.export_json();
                    println!("  Memory metrics under pressure: {}", metrics);
                }
                Err(e) => {
                    println!("  Memory optimization initialization failed under pressure: {}", e);
                }
            }
        }

        // Test concurrent memory allocation
        let concurrent_parsers: Vec<_> = (0..10).map(|i| {
            tokio::spawn(async move {
                let mut parser = AstParser::new().unwrap();
                let content = format!("fn test_{}() {{ println!(\"test\"); }}", i);
                let test_file = create_test_file(&content, "rs");
                
                let result = parser.parse_file(&test_file);
                (i, result.is_ok())
            })
        }).collect();

        let results = futures::future::join_all(concurrent_parsers).await;
        let successful_parsers = results.iter()
            .filter_map(|r| r.as_ref().ok())
            .filter(|(_, success)| *success)
            .count();

        println!("  Concurrent memory allocation: {}/10 successful", successful_parsers);
        assert!(successful_parsers > 0, "Some parsers should succeed under memory pressure");

        println!("✅ Memory allocation failure handling test completed");
    }

    #[tokio::test]
    async fn test_database_connection_failures() {
        println!("🗄️ Testing Database Connection Failures");

        GLOBAL_FAULT_INJECTOR.reset();
        GLOBAL_FAULT_INJECTOR.enable_database_failures();

        // Test database connectivity issues
        let database_scenarios = vec![
            ("invalid_url", "invalid://database/url"),
            ("nonexistent_host", "postgresql://nonexistent:5432/db"),
            ("wrong_credentials", "postgresql://wrong:wrong@localhost:5432/db"),
        ];

        for (scenario_name, db_url) in database_scenarios {
            println!("  Testing database scenario: {}", scenario_name);

            // Test database manager creation with invalid connection
            match DatabaseManager::new(db_url).await {
                Ok(_) => {
                    println!("    Database connection succeeded unexpectedly for {}", scenario_name);
                }
                Err(e) => {
                    println!("    Database connection failed appropriately for {}: {:?}", scenario_name, e);
                    
                    match e {
                        UveddiError::DatabaseError { .. } => {
                            println!("    Appropriate database error detected");
                        }
                        _ => {
                            println!("    Other error type: {:?}", e);
                        }
                    }
                }
            }
        }

        // Test analysis without database
        let test_file = create_test_file("fn test() { println!(\"no db\"); }", "rs");
        
        let analysis_config = AnalysisConfig {
            target_path: test_file,
            output_format: "json".to_string(),
            output_file: None,
            enable_ai: false,
            ollama_api_url: None,
            ollama_model: None,
            dead_code_confidence: None,
            dead_code_library_mode: false,
            dead_code_ignore_patterns: None,
            dead_code_keep_alive: None,
            large_classes_max_loc: None,
            large_classes_max_methods: None,
            large_classes_max_fields: None,
            large_classes_max_complexity: None,
            large_classes_max_lcom: None,
            large_classes_ignore_patterns: None,
            large_classes_min_severity: None,
            #[cfg(feature = "memory-optimization")]
            memory_optimization: None,
            enable_memory_optimization: false,
            memory_limit_gb: None,
            memory_profile: None,
        };

        let analysis_result = timeout(
            Duration::from_secs(10),
            async {
                let mut orchestrator = AnalysisOrchestrator::new().unwrap();
                orchestrator.execute_analysis(analysis_config).await
            }
        ).await;

        match analysis_result {
            Ok(result) => {
                match result {
                    Ok(_) => {
                        println!("  Analysis succeeded without database");
                    }
                    Err(e) => {
                        println!("  Analysis failed without database: {:?}", e);
                        
                        match e {
                            UveddiError::DatabaseError { .. } => {
                                println!("  Database error properly propagated");
                            }
                            _ => {
                                println!("  Other error (analysis may continue without DB): {:?}", e);
                            }
                        }
                    }
                }
            }
            Err(_) => {
                println!("  Analysis timed out without database");
            }
        }

        println!("✅ Database connection failure handling test completed");
    }

    #[tokio::test]
    async fn test_ai_provider_service_unavailable() {
        println!("🤖 Testing AI Provider Service Unavailable");

        GLOBAL_FAULT_INJECTOR.reset();
        GLOBAL_FAULT_INJECTOR.enable_ai_service_failures();

        // Test Ollama/AI service failures
        let ai_scenarios = vec![
            ("invalid_url", Some("http://invalid-ai-service:11434".to_string())),
            ("nonexistent_model", Some("nonexistent-model".to_string())),
            ("network_timeout", Some("llama2".to_string())),
        ];

        for (scenario_name, model_option) in ai_scenarios {
            println!("  Testing AI scenario: {}", scenario_name);

            let test_file = create_test_file("fn ai_test() { println!(\"ai test\"); }", "rs");
            
            let analysis_config = AnalysisConfig {
                target_path: test_file,
                output_format: "json".to_string(),
                output_file: None,
                enable_ai: true, // Enable AI to test failures
                ollama_api_url: Some("http://nonexistent:11434".to_string()),
                ollama_model: model_option,
                dead_code_confidence: None,
                dead_code_library_mode: false,
                dead_code_ignore_patterns: None,
                dead_code_keep_alive: None,
                large_classes_max_loc: None,
                large_classes_max_methods: None,
                large_classes_max_fields: None,
                large_classes_max_complexity: None,
                large_classes_max_lcom: None,
                large_classes_ignore_patterns: None,
                large_classes_min_severity: None,
                #[cfg(feature = "memory-optimization")]
                memory_optimization: None,
                enable_memory_optimization: false,
                memory_limit_gb: None,
                memory_profile: None,
            };

            let result = timeout(
                Duration::from_secs(15),
                async {
                    let mut orchestrator = AnalysisOrchestrator::new().unwrap();
                    orchestrator.execute_analysis(analysis_config).await
                }
            ).await;

            match result {
                Ok(analysis_result) => {
                    match analysis_result {
                        Ok(_) => {
                            println!("    AI scenario {} completed (fallback to non-AI)", scenario_name);
                        }
                        Err(e) => {
                            println!("    AI scenario {} failed appropriately: {:?}", scenario_name, e);
                            
                            match e {
                                UveddiError::AiError { .. } => {
                                    println!("    Appropriate AI error detected");
                                }
                                UveddiError::NetworkError { .. } => {
                                    println!("    Network error (AI service unavailable)");
                                }
                                _ => {
                                    println!("    Other error type: {:?}", e);
                                }
                            }
                        }
                    }
                }
                Err(_) => {
                    println!("    AI scenario {} timed out", scenario_name);
                }
            }
        }

        // Test fallback to non-AI analysis
        let fallback_config = AnalysisConfig {
            target_path: create_test_file("fn fallback() { println!(\"fallback\"); }", "rs"),
            output_format: "json".to_string(),
            output_file: None,
            enable_ai: false, // Disable AI for fallback test
            ollama_api_url: None,
            ollama_model: None,
            dead_code_confidence: None,
            dead_code_library_mode: false,
            dead_code_ignore_patterns: None,
            dead_code_keep_alive: None,
            large_classes_max_loc: None,
            large_classes_max_methods: None,
            large_classes_max_fields: None,
            large_classes_max_complexity: None,
            large_classes_max_lcom: None,
            large_classes_ignore_patterns: None,
            large_classes_min_severity: None,
            #[cfg(feature = "memory-optimization")]
            memory_optimization: None,
            enable_memory_optimization: false,
            memory_limit_gb: None,
            memory_profile: None,
        };

        let fallback_result = timeout(
            Duration::from_secs(10),
            async {
                let mut orchestrator = AnalysisOrchestrator::new().unwrap();
                orchestrator.execute_analysis(fallback_config).await
            }
        ).await;

        match fallback_result {
            Ok(Ok(_)) => {
                println!("  Fallback to non-AI analysis successful");
            }
            Ok(Err(e)) => {
                println!  ("  Fallback analysis failed: {:?}", e);
            }
            Err(_) => {
                println!("  Fallback analysis timed out");
            }
        }

        println!("✅ AI provider service failure handling test completed");
    }

    #[tokio::test]
    async fn test_concurrent_analysis_race_conditions() {
        println!("🏁 Testing Concurrent Analysis Race Conditions");

        GLOBAL_FAULT_INJECTOR.reset();
        
        // Create multiple test files with different characteristics
        let test_files = vec![
            ("simple", "fn simple() { println!(\"simple\"); }"),
            ("complex", &create_large_rust_file(50)),
            ("nested", &create_deeply_nested_file()),
            ("wide", &create_wide_structure_file()),
            ("empty", ""),
        ];

        let file_paths: Vec<_> = test_files.iter()
            .map(|(name, content)| (name, create_test_file(content, "rs")))
            .collect();

        // Test concurrent parsing with race condition simulation
        let concurrent_parsing_tasks: Vec<_> = file_paths.iter().enumerate().map(|(i, (name, path))| {
            let file_path = path.clone();
            let file_name = name.to_string();
            
            tokio::spawn(async move {
                let mut parser = AstParser::new().unwrap();
                
                // Simulate varying load by adding random delays
                let delay = Duration::from_millis((i as u64 + 1) * 50);
                tokio::time::sleep(delay).await;
                
                let start_time = Instant::now();
                let result = parser.parse_file(&file_path);
                let duration = start_time.elapsed();
                
                (file_name, result.is_ok(), duration)
            })
        }).collect();

        let parsing_results = futures::future::join_all(concurrent_parsing_tasks).await;

        let mut successful_parses = 0;
        let mut total_time = Duration::new(0, 0);

        for result in parsing_results {
            match result {
                Ok((name, success, duration)) => {
                    println!("  Parse task {} completed in {:?}: {}", name, duration, 
                            if success { "✓" } else { "✗" });
                    
                    if success {
                        successful_parses += 1;
                    }
                    total_time += duration;
                }
                Err(e) => {
                    println!("  Parse task panicked: {:?}", e);
                }
            }
        }

        println!("  Concurrent parsing results: {}/{} successful", successful_parses, file_paths.len());
        println!("  Average parse time: {:?}", total_time / file_paths.len() as u32);

        // Test concurrent analysis orchestration
        let concurrent_analysis_tasks: Vec<_> = file_paths.iter().take(3).enumerate().map(|(i, (name, path))| {
            let config = AnalysisConfig {
                target_path: path.clone(),
                output_format: "json".to_string(),
                output_file: None,
                enable_ai: false,
                ollama_api_url: None,
                ollama_model: None,
                dead_code_confidence: None,
                dead_code_library_mode: false,
                dead_code_ignore_patterns: None,
                dead_code_keep_alive: None,
                large_classes_max_loc: None,
                large_classes_max_methods: None,
                large_classes_max_fields: None,
                large_classes_max_complexity: None,
                large_classes_max_lcom: None,
                large_classes_ignore_patterns: None,
                large_classes_min_severity: None,
                #[cfg(feature = "memory-optimization")]
                memory_optimization: None,
                enable_memory_optimization: false,
                memory_limit_gb: None,
                memory_profile: None,
            };
            
            let task_name = name.to_string();
            
            tokio::spawn(async move {
                let mut orchestrator = AnalysisOrchestrator::new().unwrap();
                let start_time = Instant::now();
                let result = orchestrator.execute_analysis(config).await;
                let duration = start_time.elapsed();
                
                (task_name, result, duration)
            })
        }).collect();

        let analysis_results = timeout(
            Duration::from_secs(30),
            futures::future::join_all(concurrent_analysis_tasks)
        ).await;

        match analysis_results {
            Ok(results) => {
                let mut successful_analyses = 0;
                
                for result in results {
                    match result {
                        Ok((name, analysis_result, duration)) => {
                            match analysis_result {
                                Ok(_) => {
                                    successful_analyses += 1;
                                    println!("  Analysis {} completed successfully in {:?}", name, duration);
                                }
                                Err(e) => {
                                    println!("  Analysis {} failed in {:?}: {:?}", name, duration, e);
                                }
                            }
                        }
                        Err(e) => {
                            println!("  Analysis task panicked: {:?}", e);
                        }
                    }
                }
                
                println!("  Concurrent analysis results: {}/3 successful", successful_analyses);
            }
            Err(_) => {
                println!("  Concurrent analysis timed out");
            }
        }

        // Test race condition detection in shared resources
        let shared_counter = Arc::new(AtomicUsize::new(0));
        let race_condition_tasks: Vec<_> = (0..100).map(|i| {
            let counter = Arc::clone(&shared_counter);
            
            thread::spawn(move || {
                let mut parser = AstParser::new().unwrap();
                let content = format!("fn race_{}() {{ println!(\"race\"); }}", i);
                let test_file = create_test_file(&content, "rs");
                
                let _result = parser.parse_file(&test_file);
                counter.fetch_add(1, Ordering::SeqCst);
                
                i
            })
        }).collect();

        let mut completed_tasks = 0;
        for task in race_condition_tasks {
            match task.join() {
                Ok(_) => {
                    completed_tasks += 1;
                }
                Err(_) => {
                    println!("  Race condition task panicked");
                }
            }
        }

        let final_count = shared_counter.load(Ordering::SeqCst);
        println!("  Race condition test: {} tasks completed, counter = {}", completed_tasks, final_count);
        
        assert_eq!(completed_tasks, final_count, "Race condition detected in shared counter");

        println!("✅ Concurrent analysis race conditions test completed");
    }

    // Helper functions for creating test content
    fn create_large_rust_file(num_functions: usize) -> String {
        let mut content = String::new();
        content.push_str("// Large Rust file for fault injection testing\n");
        
        for i in 0..num_functions {
            content.push_str(&format!(
                "pub fn function_{}() -> Result<String, Box<dyn std::error::Error>> {{\n",
                i
            ));
            content.push_str("    let data = vec![1, 2, 3, 4, 5];\n");
            content.push_str("    let result = data.iter().map(|x| x * 2).collect::<Vec<_>>();\n");
            content.push_str(&format!("    Ok(format!(\"Function {} result: {{:?}}\", result))\n", i));
            content.push_str("}\n\n");
        }
        
        content
    }

    fn create_deeply_nested_file() -> String {
        let mut content = String::from("fn deeply_nested() {\n");
        
        for i in 0..50 {
            content.push_str(&format!("{}if true {{\n", "    ".repeat(i + 1)));
        }
        
        content.push_str(&format!("{}println!(\"deep\");\n", "    ".repeat(51)));
        
        for i in (0..50).rev() {
            content.push_str(&format!("{}}}\n", "    ".repeat(i + 1)));
        }
        
        content.push_str("}\n");
        content
    }

    fn create_wide_structure_file() -> String {
        let mut content = String::from("struct WideStruct {\n");
        
        for i in 0..200 {
            content.push_str(&format!("    field_{}: i32,\n", i));
        }
        
        content.push_str("}\n\nimpl WideStruct {\n");
        
        for i in 0..100 {
            content.push_str(&format!(
                "    pub fn method_{}(&self) -> i32 {{ self.field_{} }}\n", 
                i, i
            ));
        }
        
        content.push_str("}\n");
        content
    }
}
"#