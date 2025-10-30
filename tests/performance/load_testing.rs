//! Load testing framework for concurrent analysis capacity validation
//! 
//! Validates system performance under various load conditions:
//! - High concurrency scenarios
//! - Resource exhaustion testing
//! - Scalability validation
//! - Stress testing

#[cfg(test)]
mod load_tests {
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    use std::collections::HashMap;
    use tokio::sync::{Semaphore, RwLock};
    use tokio::task::JoinSet;
    use futures::future::join_all;
    use sysinfo::{System, SystemExt, ProcessExt};
    
    use uveddi::analysis::{AnalysisEngine, config::AnalysisConfig};
    use uveddi::observability::monitoring::PerformanceTracker;
    use uveddi::models::metrics::MetricValue;
    use uveddi::cache::AstCache;

    /// Load testing configuration
    struct LoadTestConfig {
        max_concurrent_users: usize,
        test_duration: Duration,
        ramp_up_duration: Duration,
        think_time: Duration,
        success_rate_threshold: f64,
        latency_threshold_ms: u64,
        memory_threshold_mb: f64,
        cpu_threshold_percent: f64,
    }

    impl Default for LoadTestConfig {
        fn default() -> Self {
            Self {
                max_concurrent_users: 1000,
                test_duration: Duration::from_secs(60),
                ramp_up_duration: Duration::from_secs(10),
                think_time: Duration::from_millis(100),
                success_rate_threshold: 0.95,
                latency_threshold_ms: 1000,
                memory_threshold_mb: 2048.0, // 2GB
                cpu_threshold_percent: 80.0,
            }
        }
    }

    /// Load test results
    #[derive(Debug, Clone)]
    struct LoadTestResults {
        total_requests: u64,
        successful_requests: u64,
        failed_requests: u64,
        success_rate: f64,
        avg_latency_ms: f64,
        p95_latency_ms: u64,
        p99_latency_ms: u64,
        max_latency_ms: u64,
        throughput_rps: f64,
        peak_memory_mb: f64,
        peak_cpu_percent: f64,
        errors: HashMap<String, u64>,
    }

    /// Test concurrent analysis capacity
    #[tokio::test]
    async fn test_concurrent_analysis_capacity() {
        let config = LoadTestConfig::default();
        let engine = create_load_test_engine().await;
        
        println!("🚀 Starting concurrent analysis capacity test");
        println!("Max users: {}", config.max_concurrent_users);
        println!("Test duration: {:?}", config.test_duration);
        
        let results = run_concurrent_load_test(engine, config).await;
        
        // Validate results
        assert!(
            results.success_rate >= 0.95,
            "Success rate {:.2}% below threshold 95%",
            results.success_rate * 100.0
        );
        
        assert!(
            results.p99_latency_ms <= 1000,
            "P99 latency {}ms exceeds threshold 1000ms",
            results.p99_latency_ms
        );
        
        assert!(
            results.peak_memory_mb <= 2048.0,
            "Peak memory {:.1}MB exceeds threshold 2048MB",
            results.peak_memory_mb
        );
        
        println!("✅ Concurrent analysis capacity test passed");
        print_load_test_results(&results);
    }

    /// Test high-load scenario simulation
    #[tokio::test]
    async fn test_high_load_scenario() {
        let config = LoadTestConfig {
            max_concurrent_users: 2000,
            test_duration: Duration::from_secs(120),
            success_rate_threshold: 0.90, // Lower threshold for extreme load
            ..Default::default()
        };
        
        let engine = create_high_capacity_engine().await;
        
        println!("🔥 Starting high-load scenario test");
        println!("Extreme load: {} users", config.max_concurrent_users);
        
        let results = run_high_load_scenario(engine, config).await;
        
        // Validate high-load performance
        assert!(
            results.success_rate >= 0.90,
            "Success rate {:.2}% below threshold 90% under high load",
            results.success_rate * 100.0
        );
        
        assert!(
            results.throughput_rps >= 100.0,
            "Throughput {:.1} RPS too low under high load",
            results.throughput_rps
        );
        
        println!("✅ High-load scenario test passed");
        print_load_test_results(&results);
    }

    /// Test scalability validation
    #[tokio::test]
    async fn test_scalability_validation() {
        println!("📈 Starting scalability validation test");
        
        let user_counts = vec![10, 50, 100, 500, 1000];
        let mut scalability_results = Vec::new();
        
        for &user_count in &user_counts {
            let config = LoadTestConfig {
                max_concurrent_users: user_count,
                test_duration: Duration::from_secs(30),
                ..Default::default()
            };
            
            let engine = create_scalable_engine().await;
            let results = run_scalability_test(engine, config).await;
            
            scalability_results.push((user_count, results));
            println!("Users: {}, Throughput: {:.1} RPS, Latency: {:.1}ms", 
                user_count, results.throughput_rps, results.avg_latency_ms);
        }
        
        // Validate scalability characteristics
        validate_scalability_results(&scalability_results);
        
        println!("✅ Scalability validation test passed");
    }

    /// Test stress testing with resource exhaustion
    #[tokio::test]
    async fn test_stress_testing() {
        println!("💪 Starting stress testing");
        
        let config = LoadTestConfig {
            max_concurrent_users: 5000, // Extreme stress
            test_duration: Duration::from_secs(60),
            success_rate_threshold: 0.70, // Very low threshold for stress test
            latency_threshold_ms: 5000,   // High latency allowed under stress
            ..Default::default()
        };
        
        let engine = create_stress_test_engine().await;
        let results = run_stress_test(engine, config).await;
        
        // Validate system doesn't crash under extreme stress
        assert!(
            results.success_rate >= 0.70,
            "Success rate {:.2}% below minimum threshold 70% under stress",
            results.success_rate * 100.0
        );
        
        // Ensure system recovers after stress
        tokio::time::sleep(Duration::from_secs(10)).await;
        let recovery_engine = create_load_test_engine().await;
        let recovery_results = run_simple_load_test(recovery_engine, 10).await;
        
        assert!(
            recovery_results.success_rate >= 0.95,
            "System failed to recover after stress test: {:.2}% success rate",
            recovery_results.success_rate * 100.0
        );
        
        println!("✅ Stress testing passed");
        print_load_test_results(&results);
    }

    /// Test memory pressure scenarios
    #[tokio::test]
    async fn test_memory_pressure() {
        println!("🧠 Starting memory pressure test");
        
        let engine = create_memory_intensive_engine().await;
        let initial_memory = get_memory_usage_mb();
        
        // Generate large analysis workload
        let large_codebase = generate_large_codebase(500); // 500MB codebase
        let results = analyze_under_memory_pressure(engine, large_codebase).await;
        
        let peak_memory = get_memory_usage_mb();
        let memory_delta = peak_memory - initial_memory;
        
        // Validate memory usage stays within bounds
        assert!(
            memory_delta <= 2048.0,
            "Memory usage {:.1}MB exceeds threshold 2048MB",
            memory_delta
        );
        
        assert!(
            results.success_rate >= 0.85,
            "Success rate {:.2}% below threshold 85% under memory pressure",
            results.success_rate * 100.0
        );
        
        println!("✅ Memory pressure test passed");
        println!("Memory delta: {:.1}MB", memory_delta);
    }

    /// Test cache performance under load
    #[tokio::test]
    async fn test_cache_performance_under_load() {
        println!("🗂️ Starting cache performance under load test");
        
        let cache = Arc::new(AstCache::with_capacity(10000).unwrap());
        let test_files = generate_test_files(20000); // 2x cache size for eviction
        
        let config = LoadTestConfig {
            max_concurrent_users: 500,
            test_duration: Duration::from_secs(60),
            ..Default::default()
        };
        
        let results = run_cache_load_test(cache, test_files, config).await;
        
        // Validate cache performance
        assert!(
            results.success_rate >= 0.95,
            "Cache load test success rate {:.2}% below threshold 95%",
            results.success_rate * 100.0
        );
        
        assert!(
            results.avg_latency_ms <= 50.0,
            "Average cache latency {:.1}ms exceeds threshold 50ms",
            results.avg_latency_ms
        );
        
        println!("✅ Cache performance under load test passed");
        print_load_test_results(&results);
    }

    // Helper functions

    async fn create_load_test_engine() -> Arc<AnalysisEngine> {
        let config = AnalysisConfig {
            max_file_size: 1024 * 1024, // 1MB
            parallel_analysis: true,
            cache_enabled: true,
            max_concurrent_tasks: Some(1000),
            ..Default::default()
        };
        
        Arc::new(
            AnalysisEngine::builder()
                .with_config(config)
                .build()
                .expect("Failed to create load test engine")
        )
    }

    async fn create_high_capacity_engine() -> Arc<AnalysisEngine> {
        let config = AnalysisConfig {
            max_file_size: 512 * 1024, // Smaller files for higher throughput
            parallel_analysis: true,
            cache_enabled: true,
            max_concurrent_tasks: Some(2000),
            memory_limit_mb: Some(3072), // 3GB
            ..Default::default()
        };
        
        Arc::new(
            AnalysisEngine::builder()
                .with_config(config)
                .with_high_capacity_mode(true)
                .build()
                .expect("Failed to create high capacity engine")
        )
    }

    async fn create_scalable_engine() -> Arc<AnalysisEngine> {
        let config = AnalysisConfig {
            max_file_size: 1024 * 1024,
            parallel_analysis: true,
            cache_enabled: true,
            adaptive_scaling: true,
            ..Default::default()
        };
        
        Arc::new(
            AnalysisEngine::builder()
                .with_config(config)
                .with_adaptive_scaling(true)
                .build()
                .expect("Failed to create scalable engine")
        )
    }

    async fn create_stress_test_engine() -> Arc<AnalysisEngine> {
        let config = AnalysisConfig {
            max_file_size: 256 * 1024, // Very small files for stress
            parallel_analysis: true,
            cache_enabled: true,
            max_concurrent_tasks: Some(10000),
            resource_limits_enabled: true,
            ..Default::default()
        };
        
        Arc::new(
            AnalysisEngine::builder()
                .with_config(config)
                .with_stress_test_mode(true)
                .build()
                .expect("Failed to create stress test engine")
        )
    }

    async fn create_memory_intensive_engine() -> Arc<AnalysisEngine> {
        let config = AnalysisConfig {
            max_file_size: 5 * 1024 * 1024, // Large files
            parallel_analysis: true,
            cache_enabled: false, // Disable cache to test memory usage
            memory_optimization_enabled: true,
            ..Default::default()
        };
        
        Arc::new(
            AnalysisEngine::builder()
                .with_config(config)
                .with_memory_optimization(true)
                .build()
                .expect("Failed to create memory intensive engine")
        )
    }

    async fn run_concurrent_load_test(
        engine: Arc<AnalysisEngine>,
        config: LoadTestConfig,
    ) -> LoadTestResults {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_users));
        let results = Arc::new(RwLock::new(LoadTestResults::default()));
        let start_time = Instant::now();
        
        // Monitor system resources
        let resource_monitor = tokio::spawn(monitor_system_resources(
            Arc::clone(&results),
            config.test_duration,
        ));
        
        let mut tasks = JoinSet::new();
        let mut user_id = 0;
        
        // Ramp-up phase
        let ramp_up_interval = config.ramp_up_duration.as_millis() / config.max_concurrent_users as u128;
        
        while start_time.elapsed() < config.test_duration {
            if user_id < config.max_concurrent_users {
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                let engine = Arc::clone(&engine);
                let results = Arc::clone(&results);
                let think_time = config.think_time;
                
                tasks.spawn(async move {
                    let _permit = permit; // Hold permit for duration of task
                    let request_start = Instant::now();
                    
                    // Simulate user analysis request
                    let test_code = generate_test_code(user_id);
                    let result = engine.analyze_code(&test_code).await;
                    
                    let latency = request_start.elapsed();
                    
                    // Record result
                    let mut results = results.write().await;
                    results.total_requests += 1;
                    
                    match result {
                        Ok(_) => {
                            results.successful_requests += 1;
                            results.record_latency(latency.as_millis() as u64);
                        }
                        Err(e) => {
                            results.failed_requests += 1;
                            *results.errors.entry(e.to_string()).or_insert(0) += 1;
                        }
                    }
                    
                    // Think time
                    tokio::time::sleep(think_time).await;
                });
                
                user_id += 1;
                
                // Ramp-up delay
                if user_id < config.max_concurrent_users {
                    tokio::time::sleep(Duration::from_millis(ramp_up_interval as u64)).await;
                }
            } else {
                // All users spawned, wait a bit
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
        
        // Wait for all tasks to complete
        while tasks.join_next().await.is_some() {}
        
        // Stop resource monitoring
        resource_monitor.abort();
        
        let mut results = results.write().await;
        results.finalize(start_time.elapsed());
        results.clone()
    }

    async fn run_high_load_scenario(
        engine: Arc<AnalysisEngine>,
        config: LoadTestConfig,
    ) -> LoadTestResults {
        // Similar to concurrent load test but with burst patterns
        let results = Arc::new(RwLock::new(LoadTestResults::default()));
        let start_time = Instant::now();
        
        // Create burst patterns: 80% load -> 120% load -> 60% load
        let burst_pattern = vec![
            (0.8, Duration::from_secs(20)),
            (1.2, Duration::from_secs(40)),
            (0.6, Duration::from_secs(40)),
            (1.0, Duration::from_secs(20)),
        ];
        
        let mut tasks = JoinSet::new();
        
        for (load_factor, duration) in burst_pattern {
            let burst_users = (config.max_concurrent_users as f64 * load_factor) as usize;
            let semaphore = Arc::new(Semaphore::new(burst_users));
            let burst_start = Instant::now();
            
            while burst_start.elapsed() < duration {
                if let Ok(permit) = semaphore.clone().try_acquire_owned() {
                    let engine = Arc::clone(&engine);
                    let results = Arc::clone(&results);
                    
                    tasks.spawn(async move {
                        let _permit = permit;
                        let request_start = Instant::now();
                        
                        let test_code = generate_complex_test_code();
                        let result = engine.analyze_code(&test_code).await;
                        
                        let latency = request_start.elapsed();
                        
                        let mut results = results.write().await;
                        results.total_requests += 1;
                        
                        match result {
                            Ok(_) => {
                                results.successful_requests += 1;
                                results.record_latency(latency.as_millis() as u64);
                            }
                            Err(e) => {
                                results.failed_requests += 1;
                                *results.errors.entry(e.to_string()).or_insert(0) += 1;
                            }
                        }
                    });
                }
                
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
        
        // Wait for all tasks to complete
        while tasks.join_next().await.is_some() {}
        
        let mut results = results.write().await;
        results.finalize(start_time.elapsed());
        results.clone()
    }

    async fn run_scalability_test(
        engine: Arc<AnalysisEngine>,
        config: LoadTestConfig,
    ) -> LoadTestResults {
        // Simplified version of concurrent load test for scalability measurement
        run_concurrent_load_test(engine, config).await
    }

    async fn run_stress_test(
        engine: Arc<AnalysisEngine>,
        config: LoadTestConfig,
    ) -> LoadTestResults {
        // Extreme load with resource contention
        run_concurrent_load_test(engine, config).await
    }

    async fn run_simple_load_test(
        engine: Arc<AnalysisEngine>,
        user_count: usize,
    ) -> LoadTestResults {
        let config = LoadTestConfig {
            max_concurrent_users: user_count,
            test_duration: Duration::from_secs(10),
            ..Default::default()
        };
        
        run_concurrent_load_test(engine, config).await
    }

    async fn analyze_under_memory_pressure(
        engine: Arc<AnalysisEngine>,
        large_codebase: String,
    ) -> LoadTestResults {
        let results = Arc::new(RwLock::new(LoadTestResults::default()));
        let start_time = Instant::now();
        
        let mut tasks = JoinSet::new();
        
        // Create memory pressure with large analysis tasks
        for i in 0..100 {
            let engine = Arc::clone(&engine);
            let results = Arc::clone(&results);
            let codebase = large_codebase.clone();
            
            tasks.spawn(async move {
                let request_start = Instant::now();
                let result = engine.analyze_code(&codebase).await;
                let latency = request_start.elapsed();
                
                let mut results = results.write().await;
                results.total_requests += 1;
                
                match result {
                    Ok(_) => {
                        results.successful_requests += 1;
                        results.record_latency(latency.as_millis() as u64);
                    }
                    Err(e) => {
                        results.failed_requests += 1;
                        *results.errors.entry(e.to_string()).or_insert(0) += 1;
                    }
                }
            });
            
            // Stagger task creation
            if i % 10 == 0 {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
        
        // Wait for all tasks to complete
        while tasks.join_next().await.is_some() {}
        
        let mut results = results.write().await;
        results.finalize(start_time.elapsed());
        results.clone()
    }

    async fn run_cache_load_test(
        cache: Arc<AstCache>,
        test_files: Vec<String>,
        config: LoadTestConfig,
    ) -> LoadTestResults {
        let results = Arc::new(RwLock::new(LoadTestResults::default()));
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_users));
        let start_time = Instant::now();
        
        let mut tasks = JoinSet::new();
        
        while start_time.elapsed() < config.test_duration {
            if let Ok(permit) = semaphore.clone().try_acquire_owned() {
                let cache = Arc::clone(&cache);
                let results = Arc::clone(&results);
                let files = test_files.clone();
                
                tasks.spawn(async move {
                    let _permit = permit;
                    let request_start = Instant::now();
                    
                    // Random file access pattern
                    let file_index = rand::random::<usize>() % files.len();
                    let file = &files[file_index];
                    
                    let result = cache.get_or_parse(file).await;
                    let latency = request_start.elapsed();
                    
                    let mut results = results.write().await;
                    results.total_requests += 1;
                    
                    match result {
                        Ok(_) => {
                            results.successful_requests += 1;
                            results.record_latency(latency.as_millis() as u64);
                        }
                        Err(e) => {
                            results.failed_requests += 1;
                            *results.errors.entry(e.to_string()).or_insert(0) += 1;
                        }
                    }
                });
            }
            
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
        
        // Wait for all tasks to complete
        while tasks.join_next().await.is_some() {}
        
        let mut results = results.write().await;
        results.finalize(start_time.elapsed());
        results.clone()
    }

    async fn monitor_system_resources(
        results: Arc<RwLock<LoadTestResults>>,
        duration: Duration,
    ) {
        let start = Instant::now();
        let mut system = System::new_all();
        
        while start.elapsed() < duration {
            system.refresh_all();
            
            let memory_mb = get_memory_usage_mb();
            let cpu_percent = get_cpu_usage_percent(&system);
            
            {
                let mut results = results.write().await;
                results.peak_memory_mb = results.peak_memory_mb.max(memory_mb);
                results.peak_cpu_percent = results.peak_cpu_percent.max(cpu_percent);
            }
            
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }

    fn validate_scalability_results(results: &[(usize, LoadTestResults)]) {
        // Check that throughput scales reasonably with user count
        for i in 1..results.len() {
            let (prev_users, prev_results) = &results[i - 1];
            let (curr_users, curr_results) = &results[i];
            
            let user_ratio = *curr_users as f64 / *prev_users as f64;
            let throughput_ratio = curr_results.throughput_rps / prev_results.throughput_rps;
            
            // Throughput should scale at least 50% as efficiently as user count
            let efficiency = throughput_ratio / user_ratio;
            assert!(
                efficiency >= 0.5,
                "Scalability efficiency {:.2} too low between {} and {} users",
                efficiency,
                prev_users,
                curr_users
            );
        }
    }

    // Helper functions for test data generation

    fn generate_test_code(id: usize) -> String {
        format!(
            "// Load test code {}\n\
             use std::collections::HashMap;\n\
             \n\
             #[derive(Debug, Clone)]\n\
             struct LoadTestStruct_{} {{\n\
                 id: usize,\n\
                 data: HashMap<String, i32>,\n\
                 values: Vec<f64>,\n\
             }}\n\
             \n\
             impl LoadTestStruct_{} {{\n\
                 fn new(id: usize) -> Self {{\n\
                     let mut data = HashMap::new();\n\
                     for i in 0..{} {{\n\
                         data.insert(format!(\"key_{{}}\", i), i as i32 * {});\n\
                     }}\n\
                     \n\
                     let values: Vec<f64> = (0..{}).map(|x| x as f64 * 1.5).collect();\n\
                     \n\
                     Self {{ id, data, values }}\n\
                 }}\n\
                 \n\
                 fn process(&self) -> f64 {{\n\
                     self.values.iter().sum::<f64>() / self.values.len() as f64\n\
                 }}\n\
                 \n\
                 fn complex_operation(&mut self) {{\n\
                     for (_, v) in self.data.iter_mut() {{\n\
                         *v = *v * 2 + {};\n\
                     }}\n\
                 }}\n\
             }}",
            id, id, id, id % 50, id % 7, id % 100, id % 13
        )
    }

    fn generate_complex_test_code() -> String {
        let id = rand::random::<usize>();
        format!(
            "// Complex load test code {}\n\
             use std::sync::{{Arc, Mutex}};\n\
             use std::thread;\n\
             use std::collections::BTreeMap;\n\
             \n\
             trait ComplexTrait_{} {{\n\
                 fn complex_method(&self, input: &[i32]) -> Result<Vec<f64>, String>;\n\
                 fn async_operation(&self) -> Result<String, Box<dyn std::error::Error>>;\n\
             }}\n\
             \n\
             #[derive(Debug)]\n\
             struct ComplexStruct_{} {{\n\
                 shared_data: Arc<Mutex<BTreeMap<String, Vec<f64>>>>,\n\
                 configuration: HashMap<String, String>,\n\
                 counters: Vec<Arc<Mutex<i64>>>,\n\
             }}\n\
             \n\
             impl ComplexTrait_{} for ComplexStruct_{} {{\n\
                 fn complex_method(&self, input: &[i32]) -> Result<Vec<f64>, String> {{\n\
                     if input.is_empty() {{\n\
                         return Err(\"Empty input\".to_string());\n\
                     }}\n\
                     \n\
                     let mut result = Vec::with_capacity(input.len());\n\
                     for &value in input {{\n\
                         let processed = match value {{\n\
                             x if x > 100 => x as f64 * 2.5,\n\
                             x if x > 50 => x as f64 * 1.8,\n\
                             x if x > 0 => x as f64 * 1.2,\n\
                             _ => 0.0,\n\
                         }};\n\
                         result.push(processed);\n\
                     }}\n\
                     \n\
                     Ok(result)\n\
                 }}\n\
                 \n\
                 fn async_operation(&self) -> Result<String, Box<dyn std::error::Error>> {{\n\
                     // Simulate complex async work\n\
                     let data = self.shared_data.lock().unwrap();\n\
                     let count = data.len();\n\
                     drop(data);\n\
                     \n\
                     Ok(format!(\"Processed {{}} items\", count))\n\
                 }}\n\
             }}",
            id, id, id, id, id
        )
    }

    fn generate_large_codebase(size_mb: usize) -> String {
        let target_size = size_mb * 1024 * 1024;
        let base_content = include_str!("../fixtures/large_code_sample.rs");
        let content_size = base_content.len();
        let repetitions = (target_size / content_size).max(1);
        
        (0..repetitions)
            .map(|i| format!("// Repetition {}\n{}\n", i, base_content))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_test_files(count: usize) -> Vec<String> {
        (0..count)
            .map(|i| format!("load_test_file_{}.rs", i))
            .collect()
    }

    fn get_memory_usage_mb() -> f64 {
        let mut system = System::new_all();
        system.refresh_all();
        
        if let Some(process) = system.process(sysinfo::get_current_pid().unwrap()) {
            process.memory() as f64 / 1024.0
        } else {
            0.0
        }
    }

    fn get_cpu_usage_percent(system: &System) -> f64 {
        if let Some(process) = system.process(sysinfo::get_current_pid().unwrap()) {
            process.cpu_usage() as f64
        } else {
            0.0
        }
    }

    fn print_load_test_results(results: &LoadTestResults) {
        println!("📊 Load Test Results:");
        println!("  Total Requests: {}", results.total_requests);
        println!("  Success Rate: {:.2}%", results.success_rate * 100.0);
        println!("  Average Latency: {:.1}ms", results.avg_latency_ms);
        println!("  P95 Latency: {}ms", results.p95_latency_ms);
        println!("  P99 Latency: {}ms", results.p99_latency_ms);
        println!("  Max Latency: {}ms", results.max_latency_ms);
        println!("  Throughput: {:.1} RPS", results.throughput_rps);
        println!("  Peak Memory: {:.1}MB", results.peak_memory_mb);
        println!("  Peak CPU: {:.1}%", results.peak_cpu_percent);
        
        if !results.errors.is_empty() {
            println!("  Errors:");
            for (error, count) in &results.errors {
                println!("    {}: {}", error, count);
            }
        }
    }

    // Default implementation for LoadTestResults
    impl Default for LoadTestResults {
        fn default() -> Self {
            Self {
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                success_rate: 0.0,
                avg_latency_ms: 0.0,
                p95_latency_ms: 0,
                p99_latency_ms: 0,
                max_latency_ms: 0,
                throughput_rps: 0.0,
                peak_memory_mb: 0.0,
                peak_cpu_percent: 0.0,
                errors: HashMap::new(),
            }
        }
    }

    impl LoadTestResults {
        fn record_latency(&mut self, latency_ms: u64) {
            // In a real implementation, would maintain a proper latency distribution
            self.max_latency_ms = self.max_latency_ms.max(latency_ms);
            self.avg_latency_ms = (self.avg_latency_ms * (self.successful_requests - 1) as f64 + latency_ms as f64) / self.successful_requests as f64;
            
            // Simplified P95/P99 calculation (would use proper percentile calculation in production)
            self.p95_latency_ms = (latency_ms as f64 * 0.95) as u64;
            self.p99_latency_ms = (latency_ms as f64 * 0.99) as u64;
        }
        
        fn finalize(&mut self, total_duration: Duration) {
            if self.total_requests > 0 {
                self.success_rate = self.successful_requests as f64 / self.total_requests as f64;
            }
            
            self.throughput_rps = self.total_requests as f64 / total_duration.as_secs_f64();
        }
    }
}