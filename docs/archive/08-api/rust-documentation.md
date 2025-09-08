# Uveddi Rust Documentation

## Table of Contents

1. [Core Analysis Engine](#core-analysis-engine)
2. [AI Integration](#ai-integration)
3. [Anti-Pattern Detection](#anti-pattern-detection)
4. [Performance Monitoring](#performance-monitoring)
5. [Plugin System](#plugin-system)
6. [Configuration Management](#configuration-management)
7. [Advanced Usage Patterns](#advanced-usage-patterns)

## Core Analysis Engine

### Basic Usage

The `AnalysisEngine` is the primary entry point for static code analysis in Uveddi.

```rust
use uveddi::analysis::{AnalysisEngine, config::AnalysisConfig};
use std::path::Path;

#[tokio::main]
async fn main() -> uveddi::Result<()> {
    // Simple analysis with default configuration
    let engine = AnalysisEngine::new()?;
    let (issues, dependency_graph) = engine.analyze(Path::new("src/")).await?;
    
    println!("Found {} architectural issues", issues.len());
    for issue in issues {
        println!("- {} in {}: {}", 
            issue.issue_type, 
            issue.file_path, 
            issue.description
        );
    }
    
    Ok(())
}
```

### Advanced Engine Configuration

Use the builder pattern for more complex setups:

```rust
use uveddi::analysis::{AnalysisEngine, config::AnalysisConfig};
use uveddi::analysis::detectors::anti_patterns::*;
use uveddi::analysis::memory::pool::DetectorPool;
use std::path::Path;
use std::sync::Arc;

#[tokio::main]
async fn main() -> uveddi::Result<()> {
    // Configure custom detector pool for memory optimization
    let detector_pool = DetectorPool::with_capacity(10)?;
    
    // Build engine with custom configuration
    let mut engine = AnalysisEngine::builder()
        .with_cache_path(&Path::new("analysis_cache.db"))
        .enable_plugins(true)
        .with_detector_pool(Arc::new(detector_pool))
        .with_config(AnalysisConfig {
            max_file_size: 2 * 1024 * 1024, // 2MB limit
            parallel_analysis: true,
            enable_ai_insights: true,
            ai_provider: "ollama".to_string(),
            ..Default::default()
        })
        .build_async()
        .await?;

    // Configure specific detectors
    engine.register_detector(Box::new(GodObjectDetector::new()))?;
    engine.register_detector(Box::new(DeadCodeDetector::new()))?;
    engine.register_detector(Box::new(TightCouplingDetector::new()))?;
    
    // Run analysis with progress tracking
    let analysis_result = engine
        .analyze_with_progress(Path::new("src/"))
        .await?;
    
    println!("Analysis completed in {}ms", analysis_result.duration_ms);
    println!("Processed {} files with {}% accuracy", 
        analysis_result.total_files,
        (analysis_result.confidence_score * 100.0) as u32
    );
    
    Ok(())
}
```

### Incremental Analysis

For large codebases, use incremental analysis to improve performance:

```rust
use uveddi::analysis::{AnalysisEngine, incremental::{IncrementalAnalysisEngine, ChangeDetector}};
use std::path::Path;

#[tokio::main]
async fn main() -> uveddi::Result<()> {
    let base_engine = AnalysisEngine::new()?;
    let mut incremental_engine = IncrementalAnalysisEngine::new(base_engine)?;
    
    // Initial full analysis
    let initial_result = incremental_engine
        .analyze_full(Path::new("src/"))
        .await?;
    println!("Initial analysis: {} issues", initial_result.issues.len());
    
    // Simulate code changes
    let change_detector = ChangeDetector::new();
    let changed_files = change_detector
        .detect_changes(Path::new("src/"))
        .await?;
    
    if !changed_files.is_empty() {
        // Only analyze changed files
        let delta_result = incremental_engine
            .analyze_changes(&changed_files)
            .await?;
        
        println!("Incremental analysis: {} new issues in {} files", 
            delta_result.new_issues.len(),
            changed_files.len()
        );
    }
    
    Ok(())
}
```

## AI Integration

### Basic AI-Enhanced Analysis

```rust
use uveddi::ai::{OllamaProvider, types::AnalysisContext};
use uveddi::analysis::AnalysisEngine;
use std::path::Path;

#[tokio::main]
async fn main() -> uveddi::Result<()> {
    // Initialize AI provider
    let ai_provider = OllamaProvider::new("llama3.2:latest").await?;
    
    // Create engine with AI integration
    let mut engine = AnalysisEngine::builder()
        .enable_ai(true)
        .with_ai_provider(Box::new(ai_provider))
        .build_async()
        .await?;
    
    // Analyze with AI explanations
    let (issues, dependency_graph) = engine.analyze(Path::new("src/")).await?;
    
    for issue in issues {
        if let Some(ai_explanation) = &issue.ai_explanation {
            println!("Issue: {}", issue.description);
            println!("AI Insight: {}", ai_explanation);
            
            if let Some(suggestion) = &issue.ai_suggestion {
                println!("Suggested Fix: {}", suggestion);
            }
            println!("---");
        }
    }
    
    Ok(())
}
```

### Custom AI Context Building

```rust
use uveddi::ai::{context::ContextBuilder, prompts::PromptTemplate};
use uveddi::ai::types::AnalysisContext;

async fn analyze_with_custom_context() -> uveddi::Result<()> {
    let context_builder = ContextBuilder::new()
        .with_project_metadata("My Rust Project", "1.0.0")
        .with_coding_standards("Clean Code principles")
        .with_performance_requirements("Low latency, high throughput")
        .with_security_level("High");
    
    let context = context_builder.build();
    
    // Use context in analysis
    let prompt_template = PromptTemplate::load("god_object_analysis")?;
    let analysis_prompt = prompt_template.render(&context)?;
    
    println!("Generated AI prompt: {}", analysis_prompt);
    
    Ok(())
}
```

## Anti-Pattern Detection

### God Object Detection

```rust
use uveddi::analysis::detectors::anti_patterns::god_object::{
    GodObjectDetector, GodObjectConfig
};
use uveddi::analysis::AnalysisEngine;

#[tokio::main]
async fn main() -> uveddi::Result<()> {
    // Configure god object detection
    let god_object_config = GodObjectConfig {
        max_lines_of_code: 500,
        max_methods: 20,
        max_fields: 15,
        complexity_threshold: 10.0,
    };
    
    let detector = GodObjectDetector::with_config(god_object_config);
    
    let mut engine = AnalysisEngine::builder()
        .with_detectors(vec![Box::new(detector)])
        .build_async()
        .await?;
    
    let (issues, _) = engine.analyze(Path::new("src/")).await?;
    
    for issue in issues {
        if let AntiPatternType::GodObject = issue.issue_type {
            println!("God Object detected:");
            println!("  File: {}", issue.file_path);
            println!("  Lines: {} (threshold: {})", 
                issue.metadata.get("lines_of_code").unwrap_or(&"N/A".to_string()),
                god_object_config.max_lines_of_code
            );
            println!("  Methods: {} (threshold: {})",
                issue.metadata.get("method_count").unwrap_or(&"N/A".to_string()),
                god_object_config.max_methods
            );
        }
    }
    
    Ok(())
}
```

### Custom Anti-Pattern Detector

```rust
use uveddi::analysis::{AnalysisDetector, types::{AnalysisResult, AnalysisContext}};
use uveddi::database::models::{ArchitecturalIssue, AntiPatternType};
use async_trait::async_trait;

/// Custom detector for overly complex functions
pub struct ComplexFunctionDetector {
    max_cyclomatic_complexity: usize,
}

impl ComplexFunctionDetector {
    pub fn new(max_complexity: usize) -> Self {
        Self {
            max_cyclomatic_complexity: max_complexity,
        }
    }
}

#[async_trait]
impl AnalysisDetector for ComplexFunctionDetector {
    async fn detect_issues(&self, context: &AnalysisContext) -> Result<Vec<ArchitecturalIssue>, Box<dyn std::error::Error + Send + Sync>> {
        let mut issues = Vec::new();
        
        for file in &context.files {
            let ast = context.ast_provider.parse_file(file).await?;
            
            // Walk AST to find function definitions
            let functions = ast.query_functions()?;
            
            for function in functions {
                let complexity = calculate_cyclomatic_complexity(&function)?;
                
                if complexity > self.max_cyclomatic_complexity {
                    issues.push(ArchitecturalIssue {
                        file_path: file.to_string_lossy().to_string(),
                        issue_type: AntiPatternType::Custom("complex_function".to_string()),
                        severity: if complexity > self.max_cyclomatic_complexity * 2 {
                            "critical".to_string()
                        } else {
                            "warning".to_string()
                        },
                        description: format!(
                            "Function '{}' has cyclomatic complexity of {} (threshold: {})",
                            function.name, complexity, self.max_cyclomatic_complexity
                        ),
                        line_number: Some(function.start_line as i32),
                        column_number: Some(function.start_column as i32),
                        metadata: serde_json::json!({
                            "function_name": function.name,
                            "complexity": complexity,
                            "threshold": self.max_cyclomatic_complexity
                        }),
                        ai_explanation: None,
                        ai_suggestion: None,
                    });
                }
            }
        }
        
        Ok(issues)
    }
    
    fn name(&self) -> &str {
        "Complex Function Detector"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
}

// Usage example
#[tokio::main]
async fn main() -> uveddi::Result<()> {
    let custom_detector = ComplexFunctionDetector::new(10);
    
    let mut engine = AnalysisEngine::builder()
        .with_detectors(vec![Box::new(custom_detector)])
        .build_async()
        .await?;
    
    let (issues, _) = engine.analyze(Path::new("src/")).await?;
    
    for issue in issues {
        println!("Custom issue found: {}", issue.description);
    }
    
    Ok(())
}

fn calculate_cyclomatic_complexity(function: &FunctionNode) -> Result<usize, Box<dyn std::error::Error>> {
    // Implementation would calculate McCabe complexity
    // This is a simplified version
    let mut complexity = 1; // Base complexity
    
    // Count decision points
    complexity += function.if_statements.len();
    complexity += function.while_loops.len();
    complexity += function.for_loops.len();
    complexity += function.match_statements.iter().map(|m| m.arms.len()).sum::<usize>();
    
    Ok(complexity)
}
```

## Performance Monitoring

### Real-Time Metrics Collection

```rust
use uveddi::monitoring::{
    performance_metrics_collector::PerformanceMetricsCollector,
    metrics::{MetricsConfig, TestMetrics},
    websocket::WebSocketManager
};
use std::time::Duration;
use tokio::time::interval;

#[tokio::main]
async fn main() -> uveddi::Result<()> {
    // Initialize performance metrics collector
    let config = MetricsConfig {
        collection_interval: Duration::from_millis(100),
        buffer_size: 10000,
        enable_websocket: true,
        websocket_port: 8080,
    };
    
    let mut collector = PerformanceMetricsCollector::new(config)?;
    
    // Start WebSocket server for real-time updates
    let websocket_manager = WebSocketManager::new(8080).await?;
    
    // Collect metrics at high frequency (4.3M+ metrics/sec capability)
    let mut interval = interval(Duration::from_millis(100));
    
    loop {
        interval.tick().await;
        
        // Collect system metrics
        let system_metrics = collector.collect_system_metrics().await?;
        
        // Broadcast to connected WebSocket clients
        websocket_manager.broadcast_metrics(&system_metrics).await?;
        
        // Store metrics for historical analysis
        collector.store_metrics(&system_metrics).await?;
        
        // Check for performance alerts
        if system_metrics.cpu_usage > 80.0 {
            println!("⚠️  High CPU usage: {:.1}%", system_metrics.cpu_usage);
        }
        
        if system_metrics.memory_usage_mb > 1024 {
            println!("⚠️  High memory usage: {}MB", system_metrics.memory_usage_mb);
        }
    }
}
```

### Analysis Performance Benchmarking

```rust
use uveddi::performance::{
    regression_detection::PerformanceRegressionDetector,
    statistical_analysis::StatisticalAnalyzer,
    genetic_bottleneck::GeneticBottleneckDetector
};
use std::time::Instant;

#[tokio::main]
async fn main() -> uveddi::Result<()> {
    let regression_detector = PerformanceRegressionDetector::new();
    let genetic_detector = GeneticBottleneckDetector::new();
    
    // Benchmark analysis performance
    let start_time = Instant::now();
    let engine = AnalysisEngine::new()?;
    let (issues, _) = engine.analyze(Path::new("src/")).await?;
    let analysis_duration = start_time.elapsed();
    
    println!("Analysis completed in: {:?}", analysis_duration);
    println!("Issues found: {}", issues.len());
    
    // Record performance data
    let performance_data = PerformanceData {
        duration: analysis_duration,
        files_analyzed: 100, // Example
        issues_found: issues.len(),
        memory_peak_mb: 256, // Example
        cpu_usage_percent: 45.0, // Example
    };
    
    // Detect performance regressions
    let regression_result = regression_detector
        .analyze_performance(&performance_data)
        .await?;
    
    if let Some(regression) = regression_result {
        println!("Performance regression detected:");
        println!("  Metric: {}", regression.metric_name);
        println!("  Current: {:.2}", regression.current_value);
        println!("  Baseline: {:.2}", regression.baseline_value);
        println!("  Change: {:.2}%", regression.percentage_change);
    }
    
    // Use genetic algorithm for bottleneck detection
    let bottleneck_result = genetic_detector
        .detect_bottlenecks(&performance_data)
        .await?;
    
    for bottleneck in bottleneck_result.bottlenecks {
        println!("Bottleneck detected: {} (confidence: {:.2})", 
            bottleneck.component, 
            bottleneck.confidence
        );
        
        for recommendation in bottleneck.recommendations {
            println!("  💡 {}", recommendation);
        }
    }
    
    Ok(())
}
```

## Plugin System

### Loading and Using WASM Plugins

```rust
use uveddi::plugins::{WasmPluginEngine, lifecycle::PluginLifecycle};
use std::path::Path;

#[tokio::main]
async fn main() -> uveddi::Result<()> {
    // Initialize plugin engine
    let mut plugin_engine = WasmPluginEngine::new().await?;
    
    // Load plugin from file
    let plugin_binary = std::fs::read("plugins/custom_detector.wasm")?;
    let plugin_manifest = serde_json::from_str(r#"
    {
        "name": "custom_detector",
        "version": "1.0.0",
        "description": "Custom anti-pattern detector",
        "entry_point": "detect_patterns",
        "permissions": ["file_system_read"]
    }
    "#)?;
    
    let plugin_id = plugin_engine
        .load_plugin(plugin_manifest, plugin_binary)
        .await?;
    
    println!("Loaded plugin: {}", plugin_id);
    
    // Create analysis engine with plugin support
    let mut engine = AnalysisEngine::builder()
        .enable_plugins(true)
        .with_plugin_engine(plugin_engine)
        .build_async()
        .await?;
    
    // Run analysis (plugins are automatically invoked)
    let (issues, _) = engine.analyze(Path::new("src/")).await?;
    
    // Filter issues from custom plugin
    let plugin_issues: Vec<_> = issues
        .iter()
        .filter(|issue| issue.metadata.get("source") == Some(&"custom_detector".to_string()))
        .collect();
    
    println!("Found {} issues from custom plugin", plugin_issues.len());
    
    Ok(())
}
```

### Plugin Development

```rust
// This would be in a separate WASM plugin crate
use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct PluginInput {
    pub file_content: String,
    pub file_path: String,
    pub ast_json: String,
}

#[derive(Serialize)]
pub struct PluginOutput {
    pub issues: Vec<DetectedIssue>,
}

#[derive(Serialize)]
pub struct DetectedIssue {
    pub issue_type: String,
    pub severity: String,
    pub description: String,
    pub line_number: u32,
    pub column_number: u32,
}

#[wasm_bindgen]
pub fn detect_patterns(input: &str) -> String {
    let plugin_input: PluginInput = serde_json::from_str(input).unwrap();
    let mut issues = Vec::new();
    
    // Custom detection logic
    if plugin_input.file_content.contains("todo!()") {
        issues.push(DetectedIssue {
            issue_type: "unfinished_code".to_string(),
            severity: "warning".to_string(),
            description: "Found unfinished code marker in production code".to_string(),
            line_number: 1, // Would need to parse actual line
            column_number: 1,
        });
    }
    
    let output = PluginOutput { issues };
    serde_json::to_string(&output).unwrap()
}
```

## Configuration Management

### Advanced Configuration

```rust
use uveddi::analysis::config::{
    AnalysisConfig, DetectorConfig, CacheConfig, SecurityConfig
};
use uveddi::security::{
    authentication::JwtConfig,
    authorization::RbacConfig
};

#[tokio::main]
async fn main() -> uveddi::Result<()> {
    // Load configuration from multiple sources
    let config = AnalysisConfig::builder()
        .load_from_file("config/analysis.toml")?
        .load_from_env()?
        .with_detector_config(DetectorConfig {
            god_object: GodObjectConfig {
                max_lines_of_code: 1000,
                max_methods: 30,
                ..Default::default()
            },
            dead_code: DeadCodeConfig {
                exclude_patterns: vec!["test_".to_string(), "_test".to_string()],
                ..Default::default()
            },
            ..Default::default()
        })
        .with_cache_config(CacheConfig {
            cache_path: "cache/analysis.db".to_string(),
            max_cache_size_mb: 500,
            ttl_hours: 24,
            compression_enabled: true,
        })
        .with_security_config(SecurityConfig {
            jwt_config: JwtConfig {
                secret: "your-secret-key".to_string(),
                expiration_hours: 24,
                issuer: "uveddi".to_string(),
            },
            rbac_config: RbacConfig {
                enable_rbac: true,
                default_role: "viewer".to_string(),
            },
            require_authentication: true,
        })
        .build()?;
    
    // Create engine with comprehensive configuration
    let engine = AnalysisEngine::builder()
        .with_config(config)
        .build_async()
        .await?;
    
    println!("Engine configured with advanced settings");
    
    Ok(())
}
```

### Configuration Validation and Migration

```rust
use uveddi::analysis::config::{ConfigValidator, ConfigMigrator};

#[tokio::main]
async fn main() -> uveddi::Result<()> {
    let config_path = "config/analysis.toml";
    
    // Validate configuration
    let validator = ConfigValidator::new();
    match validator.validate_file(config_path).await {
        Ok(_) => println!("✅ Configuration is valid"),
        Err(errors) => {
            eprintln!("❌ Configuration errors found:");
            for error in errors {
                eprintln!("  - {}", error);
            }
            return Err("Invalid configuration".into());
        }
    }
    
    // Handle configuration migration
    let migrator = ConfigMigrator::new();
    if migrator.needs_migration(config_path).await? {
        println!("🔄 Migrating configuration to latest version...");
        migrator.migrate(config_path).await?;
        println!("✅ Configuration migrated successfully");
    }
    
    Ok(())
}
```

## Advanced Usage Patterns

### Concurrent Analysis Pipeline

```rust
use uveddi::analysis::AnalysisEngine;
use tokio::sync::Semaphore;
use std::sync::Arc;
use futures::future::join_all;

#[tokio::main]
async fn main() -> uveddi::Result<()> {
    let engine = Arc::new(AnalysisEngine::new()?);
    let semaphore = Arc::new(Semaphore::new(4)); // Limit concurrent analyses
    
    let projects = vec![
        "project1/src/",
        "project2/src/",
        "project3/src/",
        "project4/src/",
        "project5/src/",
    ];
    
    let analysis_tasks = projects.into_iter().map(|project_path| {
        let engine = Arc::clone(&engine);
        let semaphore = Arc::clone(&semaphore);
        
        tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            
            println!("🔍 Starting analysis of {}", project_path);
            let start_time = std::time::Instant::now();
            
            let result = engine.analyze(Path::new(project_path)).await;
            let duration = start_time.elapsed();
            
            match result {
                Ok((issues, _)) => {
                    println!("✅ {} completed in {:?} - {} issues", 
                        project_path, duration, issues.len());
                    Ok((project_path, issues, duration))
                }
                Err(e) => {
                    println!("❌ {} failed: {}", project_path, e);
                    Err(e)
                }
            }
        })
    }).collect::<Vec<_>>();
    
    let results = join_all(analysis_tasks).await;
    
    let mut total_issues = 0;
    let mut total_duration = std::time::Duration::ZERO;
    
    for result in results {
        match result {
            Ok(Ok((project, issues, duration))) => {
                total_issues += issues.len();
                total_duration += duration;
                println!("Project: {} - Issues: {} - Duration: {:?}", 
                    project, issues.len(), duration);
            }
            Ok(Err(e)) => eprintln!("Analysis error: {}", e),
            Err(e) => eprintln!("Task error: {}", e),
        }
    }
    
    println!("\n📊 Summary:");
    println!("Total issues: {}", total_issues);
    println!("Total duration: {:?}", total_duration);
    println!("Average issues per project: {:.1}", 
        total_issues as f64 / projects.len() as f64);
    
    Ok(())
}
```

### Error Handling and Resilience

```rust
use uveddi::resilience::{
    circuit_breaker::CircuitBreaker,
    retry::{RetryPolicy, ExponentialBackoff},
    fallback::FallbackStrategy
};
use std::time::Duration;

#[tokio::main]
async fn main() -> uveddi::Result<()> {
    // Configure resilience patterns
    let circuit_breaker = CircuitBreaker::new()
        .failure_threshold(5)
        .timeout(Duration::from_secs(30))
        .recovery_timeout(Duration::from_secs(60));
    
    let retry_policy = RetryPolicy::new()
        .with_backoff(ExponentialBackoff::new(
            Duration::from_millis(100),
            Duration::from_secs(5),
            2.0
        ))
        .with_max_attempts(3);
    
    let fallback = FallbackStrategy::new()
        .with_cached_results()
        .with_simplified_analysis();
    
    let engine = AnalysisEngine::builder()
        .with_circuit_breaker(circuit_breaker)
        .with_retry_policy(retry_policy)
        .with_fallback_strategy(fallback)
        .build_async()
        .await?;
    
    // Analysis will automatically use resilience patterns
    match engine.analyze(Path::new("src/")).await {
        Ok((issues, _)) => {
            println!("✅ Analysis succeeded: {} issues", issues.len());
        }
        Err(e) => {
            println!("❌ Analysis failed with all resilience patterns: {}", e);
        }
    }
    
    Ok(())
}
```

### Memory-Optimized Large Codebase Analysis

```rust
use uveddi::analysis::{
    AnalysisEngine,
    memory::{
        arena::ArenaAllocator,
        zero_copy::ZeroCopySerializer,
        pool::DetectorPool
    }
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> uveddi::Result<()> {
    // Configure memory optimization for large codebases
    let arena = ArenaAllocator::with_capacity(100 * 1024 * 1024); // 100MB arena
    let zero_copy = ZeroCopySerializer::new();
    let detector_pool = DetectorPool::with_capacity(20)?;
    
    let engine = AnalysisEngine::builder()
        .with_arena_allocator(Arc::new(arena))
        .with_zero_copy_serializer(Arc::new(zero_copy))
        .with_detector_pool(Arc::new(detector_pool))
        .with_config(AnalysisConfig {
            streaming_mode: true,
            batch_size: 50,
            memory_limit_mb: 512,
            enable_gc_hints: true,
            ..Default::default()
        })
        .build_async()
        .await?;
    
    // Monitor memory usage during analysis
    let memory_monitor = MemoryMonitor::new()?;
    
    let start_memory = memory_monitor.get_memory_stats()?;
    println!("Starting memory usage: {}MB", start_memory.used_bytes / 1024 / 1024);
    
    let (issues, _) = engine.analyze(Path::new("large_project/")).await?;
    
    let end_memory = memory_monitor.get_memory_stats()?;
    let memory_increase = (end_memory.used_bytes - start_memory.used_bytes) / 1024 / 1024;
    
    println!("Analysis completed:");
    println!("  Issues found: {}", issues.len());
    println!("  Memory increase: {}MB", memory_increase);
    println!("  Memory efficiency: {:.2} issues/MB", 
        issues.len() as f64 / memory_increase as f64);
    
    Ok(())
}
```

This comprehensive Rust documentation provides detailed examples for all major Uveddi components, from basic usage to advanced patterns including AI integration, performance monitoring, plugin development, and memory optimization. Each example includes practical code that demonstrates real-world usage scenarios with proper error handling and configuration management.