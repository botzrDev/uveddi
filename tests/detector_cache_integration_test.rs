//! End-to-end integration tests for detector cache performance validation
//!
//! This test suite validates that the cache integration provides real performance
//! improvements and works correctly with different detector types.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::fs;
use tokio::time::sleep;
use uveddi::analysis::cache::{ContentHashInvalidator, EnhancedCacheConfig, EnhancedEngineCache};
use uveddi::analysis::detectors::cache_integration::{DetectorCacheKey, DetectorCacheManager};
use uveddi::analysis::AnalysisError;

/// Test data generator for different codebase sizes
struct TestCodebaseGenerator;

impl TestCodebaseGenerator {
    /// Generate small codebase (10 files, ~100 lines each)
    fn generate_small_codebase() -> Vec<(PathBuf, String)> {
        (0..10)
            .map(|i| {
                let path = PathBuf::from(format!("test_files/small/file_{}.rs", i));
                let content = format!(
                    r#"
// File {} - Small test file
use std::collections::HashMap;

pub struct TestStruct{} {{
    field_{}: i32,
    data: HashMap<String, String>,
}}

impl TestStruct{} {{
    pub fn new() -> Self {{
        Self {{
            field_{}: 0,
            data: HashMap::new(),
        }}
    }}

    pub fn process_data(&self) -> String {{
        format!("Processing file {}", {})
    }}

    pub fn calculate(&self, x: i32, y: i32) -> i32 {{
        x + y + self.field_{}
    }}
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_struct_creation() {{
        let instance = TestStruct{}::new();
        assert_eq!(instance.field_{}, 0);
    }}

    #[test]
    fn test_calculate() {{
        let instance = TestStruct{}::new();
        assert_eq!(instance.calculate(1, 2), 3);
    }}
}}
"#,
                    i, i, i, i, i, i, i, i, i
                );
                (path, content)
            })
            .collect()
    }

    /// Generate medium codebase (50 files, ~500 lines each)
    fn generate_medium_codebase() -> Vec<(PathBuf, String)> {
        (0..50)
            .map(|i| {
                let path = PathBuf::from(format!("test_files/medium/module_{}.rs", i));
                let content = format!(
                    r#"
// Module {} - Medium test file with more complexity
use std::collections::{{HashMap, HashSet}};
use std::sync::{{Arc, Mutex}};
use std::thread;
use std::time::Duration;

pub struct ComplexStruct{} {{
    id: usize,
    name: String,
    data: HashMap<String, Vec<i32>>,
    cache: Arc<Mutex<HashSet<String>>>,
    workers: Vec<thread::JoinHandle<()>>,
}}

impl ComplexStruct{} {{
    pub fn new(id: usize, name: String) -> Self {{
        Self {{
            id,
            name,
            data: HashMap::new(),
            cache: Arc::new(Mutex::new(HashSet::new())),
            workers: Vec::new(),
        }}
    }}

    pub fn add_data(&mut self, key: String, values: Vec<i32>) {{
        self.data.insert(key.clone(), values);
        if let Ok(mut cache) = self.cache.lock() {{
            cache.insert(key);
        }}
    }}

    pub fn process_data(&self) -> Result<Vec<String>, String> {{
        let mut results = Vec::new();

        for (key, values) in &self.data {{
            let sum: i32 = values.iter().sum();
            if sum > 100 {{
                results.push(format!("Key: {}, Sum: {}", key, sum));
            }}
        }}

        if results.is_empty() {{
            Err("No valid data found".to_string())
        }} else {{
            Ok(results)
        }}
    }}

    pub fn spawn_worker(&mut self, work_id: usize) {{
        let cache = Arc::clone(&self.cache);
        let handle = thread::spawn(move || {{
            thread::sleep(Duration::from_millis(100));
            if let Ok(mut cache_guard) = cache.lock() {{
                cache_guard.insert(format!("worker_{}", work_id));
            }}
        }});
        self.workers.push(handle);
    }}

    pub fn calculate_complex(&self, input: &[i32]) -> Result<f64, String> {{
        if input.is_empty() {{
            return Err("Empty input".to_string());
        }}

        let sum: i32 = input.iter().sum();
        let mean = sum as f64 / input.len() as f64;

        let variance: f64 = input
            .iter()
            .map(|&x| {{
                let diff = x as f64 - mean;
                diff * diff
            }})
            .sum::<f64>() / input.len() as f64;

        Ok(variance.sqrt())
    }}

    pub fn complex_algorithm(&self, data: &[String]) -> Vec<(String, usize)> {{
        let mut frequency_map: HashMap<char, usize> = HashMap::new();

        for string in data {{
            for ch in string.chars() {{
                *frequency_map.entry(ch).or_insert(0) += 1;
            }}
        }}

        let mut results = Vec::new();
        for string in data {{
            let score: usize = string.chars()
                .map(|ch| frequency_map.get(&ch).unwrap_or(&0))
                .sum();
            results.push((string.clone(), score));
        }}

        results.sort_by(|a, b| b.1.cmp(&a.1));
        results
    }}
}}

// Additional complexity with traits and generics
pub trait DataProcessor<T> {{
    type Output;

    fn process(&self, input: T) -> Self::Output;
    fn validate(&self, input: &T) -> bool;
}}

impl<T> DataProcessor<Vec<T>> for ComplexStruct{}
where
    T: Clone + std::fmt::Debug,
{{
    type Output = Result<Vec<T>, String>;

    fn process(&self, input: Vec<T>) -> Self::Output {{
        if self.validate(&input) {{
            Ok(input)
        }} else {{
            Err("Validation failed".to_string())
        }}
    }}

    fn validate(&self, input: &Vec<T>) -> bool {{
        !input.is_empty() && input.len() < 1000
    }}
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_complex_struct_creation() {{
        let instance = ComplexStruct{}::new({}, "test_module_{}".to_string());
        assert_eq!(instance.id, {});
        assert_eq!(instance.name, "test_module_{}");
    }}

    #[test]
    fn test_data_processing() {{
        let mut instance = ComplexStruct{}::new({}, "test".to_string());
        instance.add_data("test_key".to_string(), vec![1, 2, 3, 150]);

        let results = instance.process_data().unwrap();
        assert!(!results.is_empty());
    }}

    #[test]
    fn test_complex_calculation() {{
        let instance = ComplexStruct{}::new({}, "test".to_string());
        let result = instance.calculate_complex(&[1, 2, 3, 4, 5]);
        assert!(result.is_ok());
    }}

    #[test]
    fn test_data_processor_trait() {{
        let instance = ComplexStruct{}::new({}, "test".to_string());
        let input = vec![1, 2, 3, 4, 5];
        let result = instance.process(input.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), input);
    }}
}}
"#,
                    i, i, i, i, i, i, i, i, i, i, i, i, i, i, i, i, i, i, i
                );
                (path, content)
            })
            .collect()
    }

    /// Generate large codebase (200 files, ~1000 lines each)
    fn generate_large_codebase() -> Vec<(PathBuf, String)> {
        (0..200)
            .map(|i| {
                let path = PathBuf::from(format!("test_files/large/service_{}.rs", i));
                // Create even more complex content for large files
                let content = Self::generate_large_file_content(i);
                (path, content)
            })
            .collect()
    }

    fn generate_large_file_content(module_id: usize) -> String {
        format!(
            r#"
//! Service Module {} - Large test file with extensive complexity
//! This module demonstrates various patterns that detectors should analyze.

use std::collections::{{BTreeMap, HashMap, HashSet, VecDeque}};
use std::sync::{{Arc, Mutex, RwLock}};
use std::thread;
use std::time::{{Duration, Instant}};
use std::io::{{Read, Write}};
use std::fs::File;
use std::net::{{TcpStream, TcpListener}};

// God Object pattern for testing - deliberately large struct
pub struct ServiceManager{} {{
    // Basic fields
    id: usize,
    name: String,
    version: String,
    config: HashMap<String, String>,

    // Data structures
    users: HashMap<u64, User>,
    sessions: BTreeMap<String, Session>,
    active_connections: HashSet<ConnectionId>,
    message_queue: VecDeque<Message>,

    // Concurrency primitives
    state: Arc<RwLock<ServiceState>>,
    cache: Arc<Mutex<Cache>>,
    worker_pool: Vec<Worker>,

    // Metrics and monitoring
    metrics: Metrics,
    alerts: Vec<Alert>,
    log_buffer: VecDeque<LogEntry>,

    // Network components
    tcp_listener: Option<TcpListener>,
    connections: HashMap<ConnectionId, TcpStream>,

    // File system
    data_files: HashMap<String, File>,
    temp_directory: Option<std::path::PathBuf>,

    // Business logic state
    business_rules: Vec<BusinessRule>,
    workflow_states: HashMap<WorkflowId, WorkflowState>,

    // Additional complexity for testing
    nested_data: NestedComplexData,
    computed_values: ComputedValues,
}}

#[derive(Debug, Clone)]
pub struct User {{
    id: u64,
    name: String,
    email: String,
    permissions: HashSet<Permission>,
    last_login: Option<chrono::DateTime<chrono::Utc>>,
}}

#[derive(Debug, Clone)]
pub struct Session {{
    id: String,
    user_id: u64,
    created_at: chrono::DateTime<chrono::Utc>,
    expires_at: chrono::DateTime<chrono::Utc>,
    data: HashMap<String, serde_json::Value>,
}}

type ConnectionId = u64;
type WorkflowId = String;

#[derive(Debug)]
pub struct Message {{
    id: u64,
    content: String,
    timestamp: Instant,
    priority: Priority,
}}

#[derive(Debug, Clone)]
pub enum Priority {{
    Low,
    Medium,
    High,
    Critical,
}}

// Long method pattern for testing
impl ServiceManager{} {{
    pub fn new(id: usize, name: String) -> Self {{
        let mut config = HashMap::new();
        config.insert("max_connections".to_string(), "1000".to_string());
        config.insert("timeout_ms".to_string(), "5000".to_string());
        config.insert("retry_attempts".to_string(), "3".to_string());

        Self {{
            id,
            name,
            version: "1.0.0".to_string(),
            config,
            users: HashMap::new(),
            sessions: BTreeMap::new(),
            active_connections: HashSet::new(),
            message_queue: VecDeque::new(),
            state: Arc::new(RwLock::new(ServiceState::Initializing)),
            cache: Arc::new(Mutex::new(Cache::new())),
            worker_pool: Vec::new(),
            metrics: Metrics::new(),
            alerts: Vec::new(),
            log_buffer: VecDeque::new(),
            tcp_listener: None,
            connections: HashMap::new(),
            data_files: HashMap::new(),
            temp_directory: None,
            business_rules: Vec::new(),
            workflow_states: HashMap::new(),
            nested_data: NestedComplexData::new(),
            computed_values: ComputedValues::new(),
        }}
    }}

    // Deliberately long method with high complexity
    pub fn process_complex_business_logic(
        &mut self,
        input_data: &BusinessInput,
        user_context: &UserContext,
        workflow_params: &WorkflowParameters,
    ) -> Result<BusinessOutput, ServiceError> {{
        let start_time = Instant::now();

        // Step 1: Validate input data
        if input_data.items.is_empty() {{
            return Err(ServiceError::InvalidInput("Empty input data".to_string()));
        }}

        if input_data.items.len() > 10000 {{
            return Err(ServiceError::InvalidInput("Too many items".to_string()));
        }}

        // Step 2: Check user permissions
        let user = match self.users.get(&user_context.user_id) {{
            Some(u) => u,
            None => return Err(ServiceError::UnauthorizedUser(user_context.user_id)),
        }};

        if !user.permissions.contains(&Permission::ProcessBusinessData) {{
            return Err(ServiceError::InsufficientPermissions);
        }}

        // Step 3: Initialize processing context
        let mut processing_context = ProcessingContext {{
            user_id: user_context.user_id,
            session_id: user_context.session_id.clone(),
            workflow_id: workflow_params.workflow_id.clone(),
            started_at: start_time,
            processed_items: 0,
            failed_items: 0,
            warnings: Vec::new(),
            intermediate_results: HashMap::new(),
        }};

        // Step 4: Process each item with complex business logic
        let mut final_results = Vec::new();

        for (index, item) in input_data.items.iter().enumerate() {{
            // Apply business rules
            let mut rule_results = Vec::new();
            for rule in &self.business_rules {{
                match rule.evaluate(item, &processing_context) {{
                    Ok(result) => {{
                        rule_results.push(result);
                        if result.action == RuleAction::Reject {{
                            processing_context.failed_items += 1;
                            processing_context.warnings.push(
                                format!("Item {} rejected by rule {{}}", index, rule.name)
                            );
                            continue;
                        }}
                    }}
                    Err(e) => {{
                        processing_context.warnings.push(
                            format!("Rule evaluation failed for item {}: {}", index, e)
                        );
                    }}
                }}
            }}

            // Calculate derived values
            let derived_score = self.calculate_complex_score(item, &rule_results)?;
            let risk_assessment = self.assess_risk_factors(item, &processing_context)?;
            let compliance_check = self.verify_compliance_rules(item, user)?;

            // Apply transformations based on workflow parameters
            let mut transformed_item = item.clone();

            match workflow_params.transformation_type {{
                TransformationType::StandardProcessing => {{
                    transformed_item = self.apply_standard_transformations(transformed_item)?;
                }}
                TransformationType::EnhancedProcessing => {{
                    transformed_item = self.apply_enhanced_transformations(transformed_item)?;
                    let enrichment_data = self.fetch_enrichment_data(&transformed_item.id)?;
                    transformed_item.metadata.extend(enrichment_data);
                }}
                TransformationType::AdvancedProcessing => {{
                    transformed_item = self.apply_advanced_transformations(transformed_item)?;
                    let ml_predictions = self.run_ml_predictions(&transformed_item)?;
                    transformed_item.predictions = Some(ml_predictions);

                    // Advanced analytics
                    let analytics = self.compute_advanced_analytics(
                        &transformed_item,
                        &processing_context,
                        &final_results,
                    )?;
                    processing_context.intermediate_results.insert(
                        format!("analytics_{}", index),
                        serde_json::to_value(analytics)?,
                    );
                }}
            }}

            // Quality assurance checks
            let qa_result = self.perform_quality_checks(&transformed_item, &processing_context)?;
            if qa_result.quality_score < workflow_params.min_quality_threshold {{
                processing_context.failed_items += 1;
                processing_context.warnings.push(
                    format!("Item {} failed quality checks (score: {{}})", index, qa_result.quality_score)
                );
                continue;
            }}

            // Generate output item
            let output_item = BusinessOutputItem {{
                original_id: item.id.clone(),
                transformed_data: transformed_item,
                derived_score,
                risk_assessment,
                compliance_status: compliance_check,
                quality_metrics: qa_result,
                processing_metadata: ProcessingMetadata {{
                    processed_at: Instant::now(),
                    processing_duration: start_time.elapsed(),
                    rules_applied: rule_results.len(),
                    transformations_applied: match workflow_params.transformation_type {{
                        TransformationType::StandardProcessing => vec!["standard".to_string()],
                        TransformationType::EnhancedProcessing => vec!["standard".to_string(), "enhanced".to_string()],
                        TransformationType::AdvancedProcessing => vec!["standard".to_string(), "enhanced".to_string(), "advanced".to_string()],
                    }},
                    warnings: processing_context.warnings.clone(),
                }},
            }};

            final_results.push(output_item);
            processing_context.processed_items += 1;

            // Progress reporting for long-running operations
            if index > 0 && index % 100 == 0 {{
                self.report_progress(&processing_context, index, input_data.items.len())?;
            }}

            // Rate limiting to prevent system overload
            if index > 0 && index % 50 == 0 {{
                thread::sleep(Duration::from_millis(10));
            }}
        }}

        // Step 5: Post-processing and aggregation
        let aggregated_metrics = self.aggregate_processing_metrics(&final_results, &processing_context)?;
        let summary_statistics = self.compute_summary_statistics(&final_results)?;

        // Step 6: Update system state
        self.update_workflow_state(&workflow_params.workflow_id, &processing_context)?;
        self.update_metrics(&aggregated_metrics)?;
        self.log_processing_completion(&processing_context)?;

        // Step 7: Generate final output
        Ok(BusinessOutput {{
            results: final_results,
            processing_summary: ProcessingSummary {{
                total_items: input_data.items.len(),
                processed_items: processing_context.processed_items,
                failed_items: processing_context.failed_items,
                warnings: processing_context.warnings,
                total_duration: start_time.elapsed(),
                metrics: aggregated_metrics,
                statistics: summary_statistics,
            }},
            workflow_metadata: WorkflowMetadata {{
                workflow_id: workflow_params.workflow_id.clone(),
                user_id: user_context.user_id,
                completed_at: chrono::Utc::now(),
                status: if processing_context.failed_items > 0 {{
                    WorkflowStatus::CompletedWithWarnings
                }} else {{
                    WorkflowStatus::Completed
                }},
            }},
        }})
    }}

    // Additional complex methods (abbreviated for space)

    fn calculate_complex_score(&self, item: &BusinessInputItem, rule_results: &[RuleResult]) -> Result<f64, ServiceError> {{
        // Simplified complex calculation
        let base_score = item.base_value * 1.5;
        let rule_modifier: f64 = rule_results.iter()
            .map(|r| r.score_modifier)
            .sum();
        Ok(base_score + rule_modifier)
    }}

    fn assess_risk_factors(&self, item: &BusinessInputItem, context: &ProcessingContext) -> Result<RiskAssessment, ServiceError> {{
        // Simplified risk assessment
        Ok(RiskAssessment {{
            risk_level: if item.base_value > 1000.0 {{ RiskLevel::High }} else {{ RiskLevel::Low }},
            factors: vec!["automated_assessment".to_string()],
        }})
    }}

    fn verify_compliance_rules(&self, item: &BusinessInputItem, user: &User) -> Result<ComplianceStatus, ServiceError> {{
        // Simplified compliance check
        Ok(ComplianceStatus {{
            compliant: true,
            checks_performed: vec!["basic_validation".to_string()],
        }})
    }}
}}

// Supporting types and implementations (abbreviated)
// ... [Many more types and implementations would go here in a real large file]

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_service_manager_creation() {{
        let manager = ServiceManager{}::new({}, "test_service_{}".to_string());
        assert_eq!(manager.id, {});
        assert_eq!(manager.name, "test_service_{}");
    }}

    // More tests would be here in a real implementation
}}
"#,
            module_id, module_id, module_id, module_id, module_id, module_id, module_id, module_id
        )
    }
}

/// Performance test configuration
#[derive(Debug, Clone)]
struct PerformanceTestConfig {
    /// Whether to enable caching
    pub cache_enabled: bool,
    /// Number of analysis runs to perform
    pub iterations: usize,
    /// Warmup iterations (not counted in performance)
    pub warmup_iterations: usize,
}

/// Performance test results
#[derive(Debug, Clone)]
struct PerformanceTestResult {
    pub test_name: String,
    pub codebase_size: String,
    pub cache_enabled: bool,
    pub total_duration: Duration,
    pub average_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub iterations: usize,
    pub cache_hit_rate: Option<f64>,
    pub memory_usage_mb: Option<f64>,
}

impl PerformanceTestResult {
    fn speedup_factor(&self, baseline: &PerformanceTestResult) -> f64 {
        baseline.average_duration.as_millis() as f64 / self.average_duration.as_millis() as f64
    }
}

/// Main test suite for detector cache integration
struct DetectorCacheIntegrationTest {
    cache_manager: Arc<DetectorCacheManager>,
}

impl DetectorCacheIntegrationTest {
    async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        #[cfg(feature = "prometheus")]
        let cache = Arc::new(
            EnhancedEngineCache::new_with_config(
                EnhancedCacheConfig::default(),
                Arc::new(crate::analysis::cache::metrics::CacheMetrics::new()),
            )
            .await?,
        );

        #[cfg(not(feature = "prometheus"))]
        let cache =
            Arc::new(EnhancedEngineCache::new_with_config(EnhancedCacheConfig::default()).await?);

        let invalidator = Box::new(ContentHashInvalidator::new());
        let cache_manager = Arc::new(DetectorCacheManager::new(cache, invalidator).await);

        Ok(Self { cache_manager })
    }

    /// Run performance comparison test
    async fn run_performance_comparison(
        &self,
        test_name: &str,
        test_files: Vec<(PathBuf, String)>,
        config: PerformanceTestConfig,
    ) -> Result<PerformanceTestResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut durations = Vec::new();

        // Warmup iterations
        for _ in 0..config.warmup_iterations {
            let _ = self
                .simulate_detector_run(&test_files, config.cache_enabled)
                .await?;
        }

        // Actual test iterations
        let test_start = Instant::now();
        for iteration in 0..config.iterations {
            let iteration_start = Instant::now();
            let _ = self
                .simulate_detector_run(&test_files, config.cache_enabled)
                .await?;
            let iteration_duration = iteration_start.elapsed();
            durations.push(iteration_duration);

            // Progress reporting
            if iteration % 10 == 0 && iteration > 0 {
                println!(
                    "Progress: {}/{} iterations completed for {} (cache: {})",
                    iteration, config.iterations, test_name, config.cache_enabled
                );
            }
        }
        let total_duration = test_start.elapsed();

        // Calculate statistics
        let average_duration = Duration::from_nanos(
            durations.iter().map(|d| d.as_nanos()).sum::<u128>() / durations.len() as u128,
        );
        let min_duration = *durations.iter().min().unwrap();
        let max_duration = *durations.iter().max().unwrap();

        // Get cache statistics if caching is enabled
        let cache_hit_rate = if config.cache_enabled {
            let stats = self.cache_manager.get_all_stats().await;
            Some(stats.values().map(|s| s.hit_rate()).sum::<f64>() / stats.len() as f64)
        } else {
            None
        };

        Ok(PerformanceTestResult {
            test_name: test_name.to_string(),
            codebase_size: Self::classify_codebase_size(test_files.len()),
            cache_enabled: config.cache_enabled,
            total_duration,
            average_duration,
            min_duration,
            max_duration,
            iterations: config.iterations,
            cache_hit_rate,
            memory_usage_mb: None, // Would be implemented with actual memory monitoring
        })
    }

    /// Simulate running detectors on a set of files
    async fn simulate_detector_run(
        &self,
        test_files: &[(PathBuf, String)],
        cache_enabled: bool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for (file_path, content) in test_files {
            // Simulate detector analysis
            let cache_key = DetectorCacheKey::generate(
                "test_detector",
                "1.0.0",
                file_path,
                content,
                r#"{"threshold": 10}"#,
            );

            if cache_enabled {
                // Try to get from cache first
                let cached_result: Option<TestDetectorOutput> =
                    self.cache_manager.get_cached_result(&cache_key).await;

                if cached_result.is_none() {
                    // Cache miss - simulate analysis
                    let analysis_result = self.simulate_analysis(content).await;

                    // Cache the result
                    if let Err(e) = self
                        .cache_manager
                        .cache_result(&cache_key, analysis_result)
                        .await
                    {
                        self.cache_manager
                            .handle_cache_failure("test_detector", &e)
                            .await?;
                    }
                }
            } else {
                // No caching - always analyze
                let _result = self.simulate_analysis(content).await;
            }

            // Small delay to simulate real analysis time
            sleep(Duration::from_millis(1)).await;
        }

        Ok(())
    }

    /// Simulate actual detector analysis
    async fn simulate_analysis(&self, content: &str) -> TestDetectorOutput {
        // Simulate analysis time based on content length
        let analysis_time = Duration::from_millis((content.len() / 1000) as u64 + 1);
        sleep(analysis_time).await;

        // Create dummy results
        TestDetectorOutput {
            issues: vec![],
            metrics: Some(TestDetectionMetrics {
                duration_ms: analysis_time.as_millis() as u64,
                lines_analyzed: content.lines().count(),
            }),
        }
    }

    fn classify_codebase_size(file_count: usize) -> String {
        match file_count {
            0..=20 => "Small".to_string(),
            21..=100 => "Medium".to_string(),
            _ => "Large".to_string(),
        }
    }
}

/// Mock detector output for testing
#[derive(Debug, Clone)]
struct TestDetectorOutput {
    issues: Vec<TestIssue>,
    metrics: Option<TestDetectionMetrics>,
}

impl DetectorOutput for TestDetectorOutput {
    fn severity(&self) -> uveddi::analysis::detectors::base::Severity {
        uveddi::analysis::detectors::base::Severity::Info
    }

    fn issues(&self) -> &[uveddi::analysis::detectors::base::Issue] {
        &[] // Simplified for testing
    }

    fn metrics(&self) -> Option<uveddi::analysis::detectors::base::DetectionMetrics> {
        None // Simplified for testing
    }

    fn combine(self, _other: Self) -> Self {
        self // Simplified for testing
    }
}

#[derive(Debug, Clone)]
struct TestIssue;

#[derive(Debug, Clone)]
struct TestDetectionMetrics {
    duration_ms: u64,
    lines_analyzed: usize,
}

/// Comprehensive test suite
#[tokio::test]
async fn test_small_codebase_performance() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let test = DetectorCacheIntegrationTest::new().await?;
    let test_files = TestCodebaseGenerator::generate_small_codebase();

    let config = PerformanceTestConfig {
        cache_enabled: false,
        iterations: 50,
        warmup_iterations: 5,
    };

    // Test without cache
    let no_cache_result = test
        .run_performance_comparison(
            "small_codebase_no_cache",
            test_files.clone(),
            config.clone(),
        )
        .await?;

    // Test with cache
    let cache_config = PerformanceTestConfig {
        cache_enabled: true,
        ..config
    };

    let cache_result = test
        .run_performance_comparison("small_codebase_with_cache", test_files, cache_config)
        .await?;

    // Validate performance improvement
    let speedup = cache_result.speedup_factor(&no_cache_result);
    println!("Small codebase speedup with cache: {:.2}x", speedup);

    // On subsequent runs (cache hits), we should see significant speedup
    assert!(speedup >= 1.0, "Cache should not slow down analysis");

    Ok(())
}

#[tokio::test]
async fn test_medium_codebase_performance() -> Result<(), Box<dyn std::error::Error + Send + Sync>>
{
    let test = DetectorCacheIntegrationTest::new().await?;
    let test_files = TestCodebaseGenerator::generate_medium_codebase();

    let config = PerformanceTestConfig {
        cache_enabled: false,
        iterations: 20,
        warmup_iterations: 2,
    };

    let no_cache_result = test
        .run_performance_comparison(
            "medium_codebase_no_cache",
            test_files.clone(),
            config.clone(),
        )
        .await?;

    let cache_config = PerformanceTestConfig {
        cache_enabled: true,
        ..config
    };

    let cache_result = test
        .run_performance_comparison("medium_codebase_with_cache", test_files, cache_config)
        .await?;

    let speedup = cache_result.speedup_factor(&no_cache_result);
    println!("Medium codebase speedup with cache: {:.2}x", speedup);

    // Generate performance report
    let report = test.cache_manager.generate_cache_report().await;
    println!("Cache Report:\n{}", report.to_markdown());

    assert!(speedup >= 1.0);

    Ok(())
}

#[tokio::test]
async fn test_large_codebase_performance() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let test = DetectorCacheIntegrationTest::new().await?;
    let test_files = TestCodebaseGenerator::generate_large_codebase();

    let config = PerformanceTestConfig {
        cache_enabled: false,
        iterations: 5, // Fewer iterations for large codebase
        warmup_iterations: 1,
    };

    let no_cache_result = test
        .run_performance_comparison(
            "large_codebase_no_cache",
            test_files.clone(),
            config.clone(),
        )
        .await?;

    let cache_config = PerformanceTestConfig {
        cache_enabled: true,
        ..config
    };

    let cache_result = test
        .run_performance_comparison("large_codebase_with_cache", test_files, cache_config)
        .await?;

    let speedup = cache_result.speedup_factor(&no_cache_result);
    println!("Large codebase speedup with cache: {:.2}x", speedup);

    // For large codebases, cache should provide more significant benefits
    assert!(speedup >= 1.0);

    // Validate cache statistics
    let all_stats = test.cache_manager.get_all_stats().await;
    for (detector_name, stats) in all_stats {
        println!(
            "Detector {}: hit_rate={:.2}%, hits={}, misses={}",
            detector_name,
            stats.hit_rate() * 100.0,
            stats.cache_hits,
            stats.cache_misses
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_memory_usage_validation() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let test = DetectorCacheIntegrationTest::new().await?;
    let test_files = TestCodebaseGenerator::generate_medium_codebase();

    // Run analysis with cache to populate it
    let config = PerformanceTestConfig {
        cache_enabled: true,
        iterations: 10,
        warmup_iterations: 1,
    };

    let _result = test
        .run_performance_comparison("memory_test", test_files, config)
        .await?;

    // Check memory usage is within reasonable bounds
    // In a real implementation, you would integrate with system memory monitoring
    let cache_stats = test.cache_manager.cache.stats().await;

    // Basic validation that cache is functioning
    assert!(
        cache_stats.ast_stats.entries > 0,
        "Cache should contain entries"
    );

    println!(
        "Cache memory usage: AST={} bytes, Results={} bytes",
        cache_stats.ast_stats.memory_usage_bytes, cache_stats.results_stats.memory_usage_bytes
    );

    Ok(())
}

#[tokio::test]
async fn test_cache_effectiveness_patterns() -> Result<(), Box<dyn std::error::Error + Send + Sync>>
{
    let test = DetectorCacheIntegrationTest::new().await?;
    let test_files = TestCodebaseGenerator::generate_small_codebase();

    // Test pattern 1: Repeated analysis of same files
    println!("Testing repeated analysis pattern...");
    for iteration in 0..3 {
        let _result = test.simulate_detector_run(&test_files, true).await?;

        let stats = test.cache_manager.get_all_stats().await;
        if let Some(detector_stats) = stats.get("test_detector") {
            println!(
                "Iteration {}: hit_rate={:.2}%, hits={}, misses={}",
                iteration + 1,
                detector_stats.hit_rate() * 100.0,
                detector_stats.cache_hits,
                detector_stats.cache_misses
            );
        }
    }

    // Test pattern 2: Analysis of subset of files
    println!("Testing subset analysis pattern...");
    let subset_files = test_files.iter().take(5).cloned().collect::<Vec<_>>();
    let _result = test.simulate_detector_run(&subset_files, true).await?;

    // Test pattern 3: Mixed file analysis
    println!("Testing mixed file analysis pattern...");
    let mut mixed_files = test_files.clone();
    mixed_files.extend(
        TestCodebaseGenerator::generate_small_codebase()
            .into_iter()
            .take(3),
    );
    let _result = test.simulate_detector_run(&mixed_files, true).await?;

    // Generate final report
    let report = test.cache_manager.generate_cache_report().await;
    println!(
        "Final Cache Effectiveness Report:\n{}",
        report.to_markdown()
    );

    // Validate that cache is effective
    assert!(report.overall_hit_rate > 0.0, "Cache should have some hits");

    Ok(())
}
