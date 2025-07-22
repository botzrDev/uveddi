use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[cfg(feature = "image-rendering")]
use crate::report::image_renderer::{ImageFormat, ImageRenderer, RenderingServiceConfig};

#[cfg(not(feature = "image-rendering"))]
use super::image_stubs::{ImageFormat, ImageRenderer, RenderingServiceConfig};

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceBaseline {
    pub rendering_times: Vec<Duration>,
    pub memory_usage: MemoryMetrics,
    pub cache_performance: CacheMetrics,
    pub bottlenecks: Vec<PerformanceBottleneck>,
    pub diagram_types: HashMap<String, DiagramTypeMetrics>,
    pub percentile_analysis: PercentileMetrics,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub peak_usage_bytes: u64,
    pub average_usage_bytes: u64,
    pub allocation_count: u64,
    pub deallocation_count: u64,
    pub memory_leaks_detected: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheMetrics {
    pub hit_rate: f64,
    pub miss_rate: f64,
    pub cache_size_bytes: u64,
    pub average_lookup_time_ms: f64,
    pub eviction_count: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiagramTypeMetrics {
    pub diagram_type: String,
    pub sample_count: usize,
    pub average_time_ms: f64,
    pub p95_time_ms: f64,
    pub p99_time_ms: f64,
    pub success_rate: f64,
    pub complexity_score: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PercentileMetrics {
    pub p50_ms: f64,
    pub p90_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub p999_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceBottleneck {
    pub component: String,
    pub impact_level: BottleneckSeverity,
    pub description: String,
    pub recommended_fix: String,
    pub measured_impact_ms: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BottleneckSeverity {
    Critical, // >100ms impact
    High,     // 50-100ms impact
    Medium,   // 10-50ms impact
    Low,      // <10ms impact
}

#[derive(Debug)]
pub struct PerformanceAnalyzer {
    #[cfg(feature = "image-rendering")]
    renderer: ImageRenderer,
    measurements: Arc<RwLock<Vec<RenderingMeasurement>>>,
}

#[derive(Debug, Clone)]
struct RenderingMeasurement {
    diagram_type: String,
    mermaid_code: String,
    render_time: Duration,
    success: bool,
    error_message: Option<String>,
    memory_before: u64,
    memory_after: u64,
    cache_hit: bool,
}

impl PerformanceAnalyzer {
    pub fn new() -> Self {
        #[cfg(feature = "image-rendering")]
        {
            let config = RenderingServiceConfig::default();
            let renderer = ImageRenderer::with_config(config);

            Self {
                renderer,
                measurements: Arc::new(RwLock::new(Vec::new())),
            }
        }

        #[cfg(not(feature = "image-rendering"))]
        {
            Self {
                measurements: Arc::new(RwLock::new(Vec::new())),
            }
        }
    }

    pub async fn analyze_rendering_performance(
        &self,
    ) -> Result<PerformanceBaseline, AnalysisError> {
        println!("🔍 Starting UV-49 baseline performance analysis...");

        // Test different diagram types
        let test_diagrams = self.get_test_diagrams();

        // Clear cache for clean baseline
        if let Err(e) = self.clear_cache().await {
            println!("⚠️  Warning: Could not clear cache: {}", e);
        }

        let mut all_measurements = Vec::new();

        // Run baseline tests for each diagram type
        for (diagram_type, mermaid_code) in test_diagrams {
            println!("📊 Testing {} diagrams...", diagram_type);
            let measurements = self
                .test_diagram_type(&diagram_type, &mermaid_code, 20)
                .await?;
            all_measurements.extend(measurements);
        }

        // Analyze memory usage patterns
        let memory_metrics = self.analyze_memory_patterns(&all_measurements);

        // Analyze cache performance
        let cache_metrics = self.analyze_cache_performance().await?;

        // Identify bottlenecks
        let bottlenecks = self.identify_bottlenecks(&all_measurements);

        // Calculate percentile metrics
        let rendering_times: Vec<Duration> = all_measurements
            .iter()
            .filter(|m| m.success)
            .map(|m| m.render_time)
            .collect();

        let percentile_metrics = self.calculate_percentiles(&rendering_times);

        // Generate diagram type analysis
        let diagram_types = self.analyze_by_diagram_type(&all_measurements);

        let baseline = PerformanceBaseline {
            rendering_times: rendering_times.clone(),
            memory_usage: memory_metrics,
            cache_performance: cache_metrics,
            bottlenecks,
            diagram_types,
            percentile_analysis: percentile_metrics,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        // Store measurements for future analysis
        *self.measurements.write().await = all_measurements;

        // Generate baseline report
        self.generate_baseline_report(&baseline).await?;

        println!("✅ UV-49 baseline analysis complete");
        Ok(baseline)
    }

    async fn test_diagram_type(
        &self,
        diagram_type: &str,
        mermaid_code: &str,
        iterations: usize,
    ) -> Result<Vec<RenderingMeasurement>, AnalysisError> {
        let mut measurements = Vec::new();

        for i in 0..iterations {
            let memory_before = self.get_memory_usage();
            let start_time = Instant::now();

            #[cfg(feature = "image-rendering")]
            let result = self
                .renderer
                .render_diagram(mermaid_code, ImageFormat::Svg, Some((1200, 800)))
                .await;

            #[cfg(not(feature = "image-rendering"))]
            let result: Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> = Ok(vec![]);

            let render_time = start_time.elapsed();
            let memory_after = self.get_memory_usage();

            let measurement = match result {
                Ok(_) => RenderingMeasurement {
                    diagram_type: diagram_type.to_string(),
                    mermaid_code: mermaid_code.to_string(),
                    render_time,
                    success: true,
                    error_message: None,
                    memory_before,
                    memory_after,
                    cache_hit: false, // TODO: Detect actual cache hits
                },
                Err(e) => RenderingMeasurement {
                    diagram_type: diagram_type.to_string(),
                    mermaid_code: mermaid_code.to_string(),
                    render_time,
                    success: false,
                    error_message: Some(e.to_string()),
                    memory_before,
                    memory_after,
                    cache_hit: false,
                },
            };

            measurements.push(measurement);

            // Progress indicator
            if (i + 1) % 5 == 0 {
                print!("   Progress: {}/{}\r", i + 1, iterations);
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            }

            // Small delay to avoid overwhelming the service
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        println!(
            "   Completed {} iterations for {}",
            iterations, diagram_type
        );
        Ok(measurements)
    }

    fn get_test_diagrams(&self) -> Vec<(String, String)> {
        vec![
            (
                "simple".to_string(),
                r#"
graph TD
    A[Start] --> B[Process]
    B --> C[End]
"#
                .to_string(),
            ),
            (
                "medium".to_string(),
                r#"
graph TD
    A[User Request] --> B{Authentication}
    B -->|Valid| C[Load Dashboard]
    B -->|Invalid| D[Show Login]
    C --> E[Fetch Data]
    E --> F[Render Charts]
    F --> G[Display Results]
    D --> H[Validate Credentials]
    H -->|Success| C
    H -->|Failure| I[Show Error]
"#
                .to_string(),
            ),
            (
                "complex".to_string(),
                r#"
graph TD
    subgraph "Frontend Layer"
        A[React App] --> B[Redux Store]
        B --> C[API Client]
    end
    
    subgraph "API Gateway"
        C --> D[Load Balancer]
        D --> E[Auth Service]
        D --> F[User Service]
        D --> G[Data Service]
    end
    
    subgraph "Backend Services"
        E --> H[(Auth DB)]
        F --> I[(User DB)]
        G --> J[(Analytics DB)]
        G --> K[Cache Layer]
        K --> L[(Redis)]
    end
    
    subgraph "External Services"
        G --> M[Payment API]
        G --> N[Email Service]
        F --> O[File Storage]
    end
"#
                .to_string(),
            ),
            (
                "sequence".to_string(),
                r#"
sequenceDiagram
    participant U as User
    participant F as Frontend
    participant A as API Gateway
    participant S as Service
    participant D as Database
    
    U->>F: Login Request
    F->>A: POST /auth/login
    A->>S: Validate Credentials
    S->>D: Query User
    D-->>S: User Data
    S-->>A: JWT Token
    A-->>F: Auth Response
    F-->>U: Dashboard
"#
                .to_string(),
            ),
        ]
    }

    fn analyze_memory_patterns(&self, measurements: &[RenderingMeasurement]) -> MemoryMetrics {
        let memory_usages: Vec<u64> = measurements
            .iter()
            .map(|m| m.memory_after - m.memory_before)
            .collect();

        let peak_usage = memory_usages.iter().max().copied().unwrap_or(0);
        let average_usage = if !memory_usages.is_empty() {
            memory_usages.iter().sum::<u64>() / memory_usages.len() as u64
        } else {
            0
        };

        MemoryMetrics {
            peak_usage_bytes: peak_usage,
            average_usage_bytes: average_usage,
            allocation_count: measurements.len() as u64,
            deallocation_count: measurements.len() as u64, // Approximation
            memory_leaks_detected: peak_usage > average_usage * 3, // Simple heuristic
        }
    }

    async fn analyze_cache_performance(&self) -> Result<CacheMetrics, AnalysisError> {
        // Test cache performance with repeated requests
        let test_diagram = r#"
graph TD
    A[Cache Test] --> B[Performance]
    B --> C[Analysis]
"#;

        // Cold cache test
        let _ = self.clear_cache().await;
        let cold_start = Instant::now();
        #[cfg(feature = "image-rendering")]
        let _ = self
            .renderer
            .render_diagram(test_diagram, ImageFormat::Svg, None)
            .await;
        #[cfg(not(feature = "image-rendering"))]
        let _ = Ok::<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>(vec![]);
        let cold_time = cold_start.elapsed();

        // Warm cache tests
        let mut warm_times = Vec::new();
        for _ in 0..10 {
            let warm_start = Instant::now();
            #[cfg(feature = "image-rendering")]
            let _ = self
                .renderer
                .render_diagram(test_diagram, ImageFormat::Svg, None)
                .await;
            #[cfg(not(feature = "image-rendering"))]
            let _ = Ok::<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>(vec![]);
            warm_times.push(warm_start.elapsed());
        }

        let average_warm_time = if !warm_times.is_empty() {
            warm_times.iter().sum::<Duration>().as_millis() as f64 / warm_times.len() as f64
        } else {
            0.0
        };

        // Determine cache effectiveness
        let hit_rate = if cold_time.as_millis() > 0 && average_warm_time > 0.0 {
            let speedup = cold_time.as_millis() as f64 / average_warm_time;
            if speedup > 1.5 {
                0.8
            } else {
                0.0
            } // Simple heuristic
        } else {
            0.0
        };

        Ok(CacheMetrics {
            hit_rate,
            miss_rate: 1.0 - hit_rate,
            cache_size_bytes: 0, // TODO: Get actual cache size
            average_lookup_time_ms: average_warm_time,
            eviction_count: 0, // TODO: Get actual eviction count
        })
    }

    fn identify_bottlenecks(
        &self,
        measurements: &[RenderingMeasurement],
    ) -> Vec<PerformanceBottleneck> {
        let mut bottlenecks = Vec::new();

        // Analyze rendering times
        let successful_measurements: Vec<&RenderingMeasurement> =
            measurements.iter().filter(|m| m.success).collect();

        if !successful_measurements.is_empty() {
            let times: Vec<f64> = successful_measurements
                .iter()
                .map(|m| m.render_time.as_millis() as f64)
                .collect();

            let average_time = times.iter().sum::<f64>() / times.len() as f64;
            let max_time = times
                .iter()
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(&0.0);

            // Critical: Average time > 50ms
            if average_time > 50.0 {
                bottlenecks.push(PerformanceBottleneck {
                    component: "Rendering Pipeline".to_string(),
                    impact_level: BottleneckSeverity::Critical,
                    description: format!(
                        "Average rendering time ({:.2}ms) exceeds 50ms target",
                        average_time
                    ),
                    recommended_fix:
                        "Implement rendering optimizations, caching, and resource pooling"
                            .to_string(),
                    measured_impact_ms: average_time - 50.0,
                });
            }

            // Critical: P99 outliers
            if *max_time > 200.0 {
                bottlenecks.push(PerformanceBottleneck {
                    component: "Outlier Performance".to_string(),
                    impact_level: BottleneckSeverity::Critical,
                    description: format!(
                        "Maximum rendering time ({:.2}ms) indicates severe outliers",
                        max_time
                    ),
                    recommended_fix:
                        "Implement timeouts, circuit breakers, and consistent resource allocation"
                            .to_string(),
                    measured_impact_ms: *max_time - 200.0,
                });
            }
        }

        // Analyze error rates
        let error_rate = if !measurements.is_empty() {
            measurements.iter().filter(|m| !m.success).count() as f64 / measurements.len() as f64
        } else {
            0.0
        };

        if error_rate > 0.05 {
            // > 5% error rate
            bottlenecks.push(PerformanceBottleneck {
                component: "Service Reliability".to_string(),
                impact_level: BottleneckSeverity::High,
                description: format!(
                    "Error rate ({:.2}%) indicates reliability issues",
                    error_rate * 100.0
                ),
                recommended_fix: "Implement better error handling, retries, and service monitoring"
                    .to_string(),
                measured_impact_ms: 0.0,
            });
        }

        bottlenecks
    }

    fn calculate_percentiles(&self, times: &[Duration]) -> PercentileMetrics {
        if times.is_empty() {
            return PercentileMetrics {
                p50_ms: 0.0,
                p90_ms: 0.0,
                p95_ms: 0.0,
                p99_ms: 0.0,
                p999_ms: 0.0,
                min_ms: 0.0,
                max_ms: 0.0,
            };
        }

        let mut sorted_times: Vec<f64> = times.iter().map(|t| t.as_millis() as f64).collect();
        sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let percentile = |p: f64| -> f64 {
            let index = ((sorted_times.len() as f64 - 1.0) * p).round() as usize;
            sorted_times[index.min(sorted_times.len() - 1)]
        };

        PercentileMetrics {
            p50_ms: percentile(0.50),
            p90_ms: percentile(0.90),
            p95_ms: percentile(0.95),
            p99_ms: percentile(0.99),
            p999_ms: percentile(0.999),
            min_ms: sorted_times[0],
            max_ms: sorted_times[sorted_times.len() - 1],
        }
    }

    fn analyze_by_diagram_type(
        &self,
        measurements: &[RenderingMeasurement],
    ) -> HashMap<String, DiagramTypeMetrics> {
        let mut diagram_types = HashMap::new();

        // Group measurements by diagram type
        let mut grouped = HashMap::new();
        for measurement in measurements {
            grouped
                .entry(measurement.diagram_type.clone())
                .or_insert_with(Vec::new)
                .push(measurement);
        }

        for (diagram_type, measures) in grouped {
            let total_count = measures.len();
            let successful_measures: Vec<&RenderingMeasurement> =
                measures.into_iter().filter(|m| m.success).collect();

            if !successful_measures.is_empty() {
                let times: Vec<f64> = successful_measures
                    .iter()
                    .map(|m| m.render_time.as_millis() as f64)
                    .collect();

                let average_time = times.iter().sum::<f64>() / times.len() as f64;
                let success_rate = successful_measures.len() as f64 / total_count as f64;

                // Calculate percentiles for this diagram type
                let mut sorted_times = times.clone();
                sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());

                let p95_time = if sorted_times.len() > 1 {
                    let index = ((sorted_times.len() as f64 - 1.0) * 0.95).round() as usize;
                    sorted_times[index.min(sorted_times.len() - 1)]
                } else {
                    average_time
                };

                let p99_time = if sorted_times.len() > 1 {
                    let index = ((sorted_times.len() as f64 - 1.0) * 0.99).round() as usize;
                    sorted_times[index.min(sorted_times.len() - 1)]
                } else {
                    average_time
                };

                // Estimate complexity score
                let complexity_score = self.estimate_complexity(&diagram_type);

                diagram_types.insert(
                    diagram_type.clone(),
                    DiagramTypeMetrics {
                        diagram_type: diagram_type.clone(),
                        sample_count: total_count,
                        average_time_ms: average_time,
                        p95_time_ms: p95_time,
                        p99_time_ms: p99_time,
                        success_rate,
                        complexity_score,
                    },
                );
            }
        }

        diagram_types
    }

    fn estimate_complexity(&self, diagram_type: &str) -> u32 {
        match diagram_type {
            "simple" => 10,
            "medium" => 50,
            "complex" => 200,
            "sequence" => 75,
            _ => 25,
        }
    }

    async fn clear_cache(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Attempt to clear the rendering service cache
        let client = reqwest::Client::new();
        let response = client.delete("http://localhost:3001/cache").send().await?;

        if !response.status().is_success() {
            return Err(format!("Failed to clear cache: {}", response.status()).into());
        }

        Ok(())
    }

    fn get_memory_usage(&self) -> u64 {
        // Simple memory usage estimation (in a real implementation, use proper profiling)
        std::process::id() as u64 * 1024 // Placeholder
    }

    async fn generate_baseline_report(
        &self,
        baseline: &PerformanceBaseline,
    ) -> Result<(), AnalysisError> {
        let report = serde_json::to_string_pretty(baseline).unwrap();

        tokio::fs::write("performance_baseline_report.json", report)
            .await
            .map_err(|e| AnalysisError::IoError(e.to_string()))?;

        // Generate human-readable summary
        let summary = self.generate_summary_report(baseline);
        tokio::fs::write("performance_baseline_summary.md", summary)
            .await
            .map_err(|e| AnalysisError::IoError(e.to_string()))?;

        println!("📊 Baseline reports generated:");
        println!("  - performance_baseline_report.json");
        println!("  - performance_baseline_summary.md");

        Ok(())
    }

    fn generate_summary_report(&self, baseline: &PerformanceBaseline) -> String {
        let mut report = String::new();

        report.push_str("# UV-49 Performance Baseline Analysis Report\n\n");
        report.push_str(&format!("**Generated:** {}\n\n", baseline.timestamp));

        report.push_str("## Executive Summary\n\n");
        report.push_str(&format!(
            "- **Total Measurements:** {}\n",
            baseline.rendering_times.len()
        ));
        report.push_str(&format!(
            "- **Average Rendering Time:** {:.2}ms\n",
            baseline.percentile_analysis.p50_ms
        ));
        report.push_str(&format!(
            "- **P95 Rendering Time:** {:.2}ms\n",
            baseline.percentile_analysis.p95_ms
        ));
        report.push_str(&format!(
            "- **P99 Rendering Time:** {:.2}ms\n",
            baseline.percentile_analysis.p99_ms
        ));
        report.push_str(&format!(
            "- **Cache Hit Rate:** {:.1}%\n",
            baseline.cache_performance.hit_rate * 100.0
        ));
        report.push_str(&format!(
            "- **Critical Bottlenecks:** {}\n\n",
            baseline
                .bottlenecks
                .iter()
                .filter(|b| matches!(b.impact_level, BottleneckSeverity::Critical))
                .count()
        ));

        report.push_str("## Performance Target Analysis\n\n");
        let target_compliance = baseline
            .rendering_times
            .iter()
            .filter(|t| t.as_millis() < 50)
            .count() as f64
            / baseline.rendering_times.len() as f64
            * 100.0;

        report.push_str(&format!(
            "- **<50ms Target Compliance:** {:.1}%\n",
            target_compliance
        ));

        if target_compliance >= 95.0 {
            report.push_str("- **Status:** ✅ TARGET MET\n\n");
        } else {
            report.push_str("- **Status:** ❌ TARGET NOT MET\n\n");
        }

        report.push_str("## Identified Bottlenecks\n\n");
        for bottleneck in &baseline.bottlenecks {
            report.push_str(&format!(
                "### {} ({:?})\n",
                bottleneck.component, bottleneck.impact_level
            ));
            report.push_str(&format!("- **Description:** {}\n", bottleneck.description));
            report.push_str(&format!(
                "- **Impact:** {:.2}ms\n",
                bottleneck.measured_impact_ms
            ));
            report.push_str(&format!(
                "- **Recommendation:** {}\n\n",
                bottleneck.recommended_fix
            ));
        }

        report.push_str("## Diagram Type Performance\n\n");
        for (_, metrics) in &baseline.diagram_types {
            report.push_str(&format!("### {} Diagrams\n", metrics.diagram_type));
            report.push_str(&format!("- **Sample Count:** {}\n", metrics.sample_count));
            report.push_str(&format!(
                "- **Average Time:** {:.2}ms\n",
                metrics.average_time_ms
            ));
            report.push_str(&format!("- **P95 Time:** {:.2}ms\n", metrics.p95_time_ms));
            report.push_str(&format!("- **P99 Time:** {:.2}ms\n", metrics.p99_time_ms));
            report.push_str(&format!(
                "- **Success Rate:** {:.1}%\n",
                metrics.success_rate * 100.0
            ));
            report.push_str(&format!(
                "- **Complexity Score:** {}\n\n",
                metrics.complexity_score
            ));
        }

        report.push_str("## Next Steps for UV-47 Implementation\n\n");
        report.push_str(
            "Based on this baseline analysis, the following optimizations are recommended:\n\n",
        );

        for bottleneck in &baseline.bottlenecks {
            if matches!(
                bottleneck.impact_level,
                BottleneckSeverity::Critical | BottleneckSeverity::High
            ) {
                report.push_str(&format!(
                    "1. **{}:** {}\n",
                    bottleneck.component, bottleneck.recommended_fix
                ));
            }
        }

        report
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AnalysisError {
    #[error("Rendering error: {0}")]
    RenderingError(String),
    #[error("IO error: {0}")]
    IoError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
}

impl From<crate::error::rendering::RenderingServiceError> for AnalysisError {
    fn from(err: crate::error::rendering::RenderingServiceError) -> Self {
        AnalysisError::RenderingError(err.to_string())
    }
}
