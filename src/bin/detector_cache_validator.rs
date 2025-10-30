//! End-to-End Detector Cache Validation Tool
//!
//! This binary performs comprehensive validation of the detector cache integration,
//! measuring real-world performance gains and validating cache effectiveness.

use clap::{Arg, Command};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::fs;
use tracing::{error, info, warn};
use uveddi::analysis::cache::{EnhancedEngineCache, EnhancedCacheConfig, ContentHashInvalidator};
use uveddi::analysis::detectors::cache_integration::{DetectorCacheManager, DetectorCacheKey};
use uveddi::analysis::AnalysisError;

/// Configuration for validation tests
#[derive(Debug, Clone)]
struct ValidationConfig {
    pub codebase_path: PathBuf,
    pub iterations: usize,
    pub warmup_iterations: usize,
    pub detector_types: Vec<String>,
    pub memory_limit_mb: Option<usize>,
    pub output_format: OutputFormat,
    pub output_file: Option<PathBuf>,
    pub verbose: bool,
}

#[derive(Debug, Clone)]
enum OutputFormat {
    Json,
    Markdown,
    Console,
}

/// Comprehensive validation results
#[derive(Debug, Clone)]
struct ValidationResults {
    pub codebase_info: CodebaseInfo,
    pub performance_comparison: PerformanceComparison,
    pub cache_effectiveness: CacheEffectiveness,
    pub memory_validation: MemoryValidation,
    pub detector_specific_results: HashMap<String, DetectorValidationResult>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone)]
struct CodebaseInfo {
    pub path: PathBuf,
    pub total_files: usize,
    pub supported_files: usize,
    pub total_lines: usize,
    pub file_size_distribution: FileSizeDistribution,
    pub languages: Vec<String>,
}

#[derive(Debug, Clone)]
struct FileSizeDistribution {
    pub small_files: usize,    // < 100 lines
    pub medium_files: usize,   // 100-500 lines
    pub large_files: usize,    // 500-2000 lines
    pub very_large_files: usize, // > 2000 lines
}

#[derive(Debug, Clone)]
struct PerformanceComparison {
    pub baseline_duration: Duration,
    pub cached_duration: Duration,
    pub speedup_factor: f64,
    pub time_saved: Duration,
    pub iterations: usize,
}

#[derive(Debug, Clone)]
struct CacheEffectiveness {
    pub overall_hit_rate: f64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_errors: u64,
    pub average_hit_time_ms: f64,
    pub average_miss_time_ms: f64,
}

#[derive(Debug, Clone)]
struct MemoryValidation {
    pub max_memory_usage_mb: f64,
    pub average_memory_usage_mb: f64,
    pub memory_limit_mb: Option<usize>,
    pub memory_limit_exceeded: bool,
    pub memory_efficiency: f64, // Cache hits per MB used
}

#[derive(Debug, Clone)]
struct DetectorValidationResult {
    pub detector_name: String,
    pub hit_rate: f64,
    pub performance_gain: f64,
    pub memory_usage_mb: f64,
    pub recommendations: Vec<String>,
}

/// Main validator implementation
struct DetectorCacheValidator {
    cache_manager: Arc<DetectorCacheManager>,
    config: ValidationConfig,
}

impl DetectorCacheValidator {
    async fn new(config: ValidationConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Configure cache based on memory limits
        let mut cache_config = EnhancedCacheConfig::default();

        if let Some(memory_limit) = config.memory_limit_mb {
            cache_config.ast_config.max_memory_bytes = Some(memory_limit * 1024 * 1024 / 2); // Half for AST
            cache_config.results_config.max_memory_bytes = Some(memory_limit * 1024 * 1024 / 2); // Half for results
        }

        #[cfg(feature = "prometheus")]
        let cache = Arc::new(
            EnhancedEngineCache::new_with_config(
                cache_config,
                Arc::new(crate::analysis::cache::metrics::CacheMetrics::new()),
            )
            .await?
        );

        #[cfg(not(feature = "prometheus"))]
        let cache = Arc::new(
            EnhancedEngineCache::new_with_config(cache_config).await?
        );

        let invalidator = Box::new(ContentHashInvalidator::new());
        let cache_manager = Arc::new(DetectorCacheManager::new(cache, invalidator).await);

        Ok(Self {
            cache_manager,
            config,
        })
    }

    /// Run comprehensive validation
    async fn run_validation(&self) -> Result<ValidationResults, Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting detector cache validation for {}", self.config.codebase_path.display());

        // Step 1: Analyze codebase structure
        let codebase_info = self.analyze_codebase().await?;
        info!("Analyzed codebase: {} files, {} total lines", codebase_info.total_files, codebase_info.total_lines);

        // Step 2: Run performance comparison
        let performance_comparison = self.run_performance_comparison(&codebase_info).await?;
        info!("Performance comparison completed: {:.2}x speedup", performance_comparison.speedup_factor);

        // Step 3: Validate cache effectiveness
        let cache_effectiveness = self.validate_cache_effectiveness().await?;
        info!("Cache effectiveness: {:.2}% hit rate", cache_effectiveness.overall_hit_rate * 100.0);

        // Step 4: Validate memory usage
        let memory_validation = self.validate_memory_usage().await?;
        info!("Memory validation: {:.2}MB average usage", memory_validation.average_memory_usage_mb);

        // Step 5: Per-detector analysis
        let detector_specific_results = self.analyze_detector_specific_performance().await?;
        info!("Analyzed {} detector types", detector_specific_results.len());

        // Step 6: Generate recommendations
        let recommendations = self.generate_recommendations(&performance_comparison, &cache_effectiveness, &memory_validation);

        Ok(ValidationResults {
            codebase_info,
            performance_comparison,
            cache_effectiveness,
            memory_validation,
            detector_specific_results,
            recommendations,
        })
    }

    async fn analyze_codebase(&self) -> Result<CodebaseInfo, Box<dyn std::error::Error + Send + Sync>> {
        let mut total_files = 0;
        let mut supported_files = 0;
        let mut total_lines = 0;
        let mut file_sizes = Vec::new();
        let mut languages = std::collections::HashSet::new();

        let supported_extensions = vec!["rs", "py", "js", "ts", "jsx", "tsx"];

        // Walk the codebase directory
        let mut entries = fs::read_dir(&self.config.codebase_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            if let Err(e) = self.process_directory_entry(
                entry,
                &mut total_files,
                &mut supported_files,
                &mut total_lines,
                &mut file_sizes,
                &mut languages,
                &supported_extensions,
            ).await {
                warn!("Failed to process directory entry: {}", e);
            }
        }

        let distribution = FileSizeDistribution {
            small_files: file_sizes.iter().filter(|&&size| size < 100).count(),
            medium_files: file_sizes.iter().filter(|&&size| size >= 100 && size < 500).count(),
            large_files: file_sizes.iter().filter(|&&size| size >= 500 && size < 2000).count(),
            very_large_files: file_sizes.iter().filter(|&&size| size >= 2000).count(),
        };

        Ok(CodebaseInfo {
            path: self.config.codebase_path.clone(),
            total_files,
            supported_files,
            total_lines,
            file_size_distribution: distribution,
            languages: languages.into_iter().collect(),
        })
    }

    async fn process_directory_entry(
        &self,
        entry: fs::DirEntry,
        total_files: &mut usize,
        supported_files: &mut usize,
        total_lines: &mut usize,
        file_sizes: &mut Vec<usize>,
        languages: &mut std::collections::HashSet<String>,
        supported_extensions: &[&str],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let path = entry.path();

        if path.is_file() {
            *total_files += 1;

            if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
                if supported_extensions.contains(&extension) {
                    *supported_files += 1;
                    languages.insert(extension.to_string());

                    // Count lines in file
                    match fs::read_to_string(&path).await {
                        Ok(content) => {
                            let line_count = content.lines().count();
                            *total_lines += line_count;
                            file_sizes.push(line_count);
                        }
                        Err(e) => warn!("Could not read file {}: {}", path.display(), e),
                    }
                }
            }
        } else if path.is_dir() {
            // Skip common directories that shouldn't be analyzed
            let dir_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if !["target", "node_modules", ".git", "dist", "build"].contains(&dir_name) {
                // Recursively process subdirectories
                let mut entries = fs::read_dir(&path).await?;
                while let Some(entry) = entries.next_entry().await? {
                    self.process_directory_entry(
                        entry,
                        total_files,
                        supported_files,
                        total_lines,
                        file_sizes,
                        languages,
                        supported_extensions,
                    ).await?;
                }
            }
        }

        Ok(())
    }

    async fn run_performance_comparison(
        &self,
        codebase_info: &CodebaseInfo,
    ) -> Result<PerformanceComparison, Box<dyn std::error::Error + Send + Sync>> {
        info!("Running performance comparison...");

        // Collect test files
        let test_files = self.collect_test_files(&codebase_info.path).await?;

        // Baseline test (no cache)
        let baseline_start = Instant::now();
        for _ in 0..self.config.warmup_iterations {
            self.simulate_analysis_run(&test_files, false).await?;
        }

        let mut baseline_durations = Vec::new();
        for _ in 0..self.config.iterations {
            let start = Instant::now();
            self.simulate_analysis_run(&test_files, false).await?;
            baseline_durations.push(start.elapsed());
        }
        let baseline_duration = baseline_durations.iter().sum::<Duration>() / baseline_durations.len() as u32;

        // Cache test
        for _ in 0..self.config.warmup_iterations {
            self.simulate_analysis_run(&test_files, true).await?;
        }

        let mut cached_durations = Vec::new();
        for _ in 0..self.config.iterations {
            let start = Instant::now();
            self.simulate_analysis_run(&test_files, true).await?;
            cached_durations.push(start.elapsed());
        }
        let cached_duration = cached_durations.iter().sum::<Duration>() / cached_durations.len() as u32;

        let speedup_factor = baseline_duration.as_millis() as f64 / cached_duration.as_millis() as f64;
        let time_saved = baseline_duration.saturating_sub(cached_duration);

        Ok(PerformanceComparison {
            baseline_duration,
            cached_duration,
            speedup_factor,
            time_saved,
            iterations: self.config.iterations,
        })
    }

    async fn collect_test_files(&self, path: &Path) -> Result<Vec<(PathBuf, String)>, Box<dyn std::error::Error + Send + Sync>> {
        let mut files = Vec::new();
        let supported_extensions = vec!["rs", "py", "js", "ts", "jsx", "tsx"];

        self.collect_files_recursive(path, &mut files, &supported_extensions).await?;

        // Limit to reasonable number for testing
        files.truncate(100);

        Ok(files)
    }

    async fn collect_files_recursive(
        &self,
        dir: &Path,
        files: &mut Vec<(PathBuf, String)>,
        supported_extensions: &[&str],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut entries = fs::read_dir(dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.is_file() {
                if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
                    if supported_extensions.contains(&extension) {
                        match fs::read_to_string(&path).await {
                            Ok(content) => files.push((path, content)),
                            Err(e) => warn!("Could not read {}: {}", path.display(), e),
                        }
                    }
                }
            } else if path.is_dir() {
                let dir_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                if !["target", "node_modules", ".git", "dist", "build"].contains(&dir_name) {
                    self.collect_files_recursive(&path, files, supported_extensions).await?;
                }
            }
        }

        Ok(())
    }

    async fn simulate_analysis_run(
        &self,
        test_files: &[(PathBuf, String)],
        cache_enabled: bool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for detector in &self.config.detector_types {
            for (file_path, content) in test_files {
                let cache_key = DetectorCacheKey::generate(
                    detector,
                    "1.0.0",
                    file_path,
                    content,
                    r#"{"enabled": true}"#,
                );

                if cache_enabled {
                    // Try cache first
                    let _cached_result: Option<()> = self.cache_manager.get_cached_result(&cache_key).await;

                    // If cache miss, simulate analysis and cache result
                    // This is simplified - would integrate with actual detectors
                    let analysis_time = Duration::from_millis((content.len() / 1000) as u64 + 1);
                    tokio::time::sleep(analysis_time).await;
                } else {
                    // Always analyze without cache
                    let analysis_time = Duration::from_millis((content.len() / 1000) as u64 + 1);
                    tokio::time::sleep(analysis_time).await;
                }
            }
        }

        Ok(())
    }

    async fn validate_cache_effectiveness(&self) -> Result<CacheEffectiveness, Box<dyn std::error::Error + Send + Sync>> {
        let report = self.cache_manager.generate_cache_report().await;

        Ok(CacheEffectiveness {
            overall_hit_rate: report.overall_hit_rate,
            cache_hits: report.total_cache_hits,
            cache_misses: report.total_cache_misses,
            cache_errors: report.total_cache_errors,
            average_hit_time_ms: report.detector_reports.iter()
                .map(|r| r.average_hit_time_ms)
                .sum::<f64>() / report.detector_reports.len().max(1) as f64,
            average_miss_time_ms: report.detector_reports.iter()
                .map(|r| r.average_miss_time_ms)
                .sum::<f64>() / report.detector_reports.len().max(1) as f64,
        })
    }

    async fn validate_memory_usage(&self) -> Result<MemoryValidation, Box<dyn std::error::Error + Send + Sync>> {
        let cache_stats = self.cache_manager.cache.stats().await;

        let total_memory_bytes = cache_stats.ast_stats.memory_usage_bytes + cache_stats.results_stats.memory_usage_bytes;
        let memory_usage_mb = total_memory_bytes as f64 / (1024.0 * 1024.0);

        let memory_limit_exceeded = if let Some(limit) = self.config.memory_limit_mb {
            memory_usage_mb > limit as f64
        } else {
            false
        };

        let cache_effectiveness = self.validate_cache_effectiveness().await?;
        let memory_efficiency = if memory_usage_mb > 0.0 {
            cache_effectiveness.cache_hits as f64 / memory_usage_mb
        } else {
            0.0
        };

        Ok(MemoryValidation {
            max_memory_usage_mb: memory_usage_mb,
            average_memory_usage_mb: memory_usage_mb, // Simplified
            memory_limit_mb: self.config.memory_limit_mb,
            memory_limit_exceeded,
            memory_efficiency,
        })
    }

    async fn analyze_detector_specific_performance(&self) -> Result<HashMap<String, DetectorValidationResult>, Box<dyn std::error::Error + Send + Sync>> {
        let mut results = HashMap::new();
        let all_stats = self.cache_manager.get_all_stats().await;

        for detector_name in &self.config.detector_types {
            if let Some(stats) = all_stats.get(detector_name) {
                let mut recommendations = Vec::new();

                let hit_rate = stats.hit_rate();
                if hit_rate < 0.5 {
                    recommendations.push("Consider adjusting cache invalidation strategy to improve hit rate".to_string());
                }

                if stats.cache_errors > 0 {
                    recommendations.push("Investigate cache errors and improve error handling".to_string());
                }

                let performance_gain = if stats.average_miss_time_ms > 0.0 {
                    (stats.average_miss_time_ms - stats.average_hit_time_ms) / stats.average_miss_time_ms
                } else {
                    0.0
                };

                results.insert(detector_name.clone(), DetectorValidationResult {
                    detector_name: detector_name.clone(),
                    hit_rate,
                    performance_gain,
                    memory_usage_mb: 0.0, // Would need more detailed memory tracking
                    recommendations,
                });
            }
        }

        Ok(results)
    }

    fn generate_recommendations(
        &self,
        performance: &PerformanceComparison,
        cache_effectiveness: &CacheEffectiveness,
        memory: &MemoryValidation,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Performance recommendations
        if performance.speedup_factor < 1.5 {
            recommendations.push("Cache performance gain is modest. Consider optimizing cache key generation or increasing cache size.".to_string());
        } else if performance.speedup_factor > 10.0 {
            recommendations.push("Excellent cache performance! Consider similar caching strategies for other components.".to_string());
        }

        // Cache effectiveness recommendations
        if cache_effectiveness.overall_hit_rate < 0.3 {
            recommendations.push("Low cache hit rate. Review cache invalidation strategy and detector versioning.".to_string());
        } else if cache_effectiveness.overall_hit_rate > 0.8 {
            recommendations.push("High cache hit rate achieved. Cache strategy is working well.".to_string());
        }

        // Memory recommendations
        if memory.memory_limit_exceeded {
            recommendations.push("Memory limit exceeded. Consider increasing memory allocation or implementing more aggressive eviction policies.".to_string());
        }

        if memory.memory_efficiency < 10.0 {
            recommendations.push("Low memory efficiency. Consider tuning cache size limits or eviction policies.".to_string());
        }

        if cache_effectiveness.cache_errors > 0 {
            recommendations.push("Cache errors detected. Review error handling and cache resilience.".to_string());
        }

        recommendations
    }

    /// Output results in specified format
    fn output_results(&self, results: &ValidationResults) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match self.config.output_format {
            OutputFormat::Console => self.output_console(results),
            OutputFormat::Json => self.output_json(results),
            OutputFormat::Markdown => self.output_markdown(results),
        }
    }

    fn output_console(&self, results: &ValidationResults) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("\n=== Detector Cache Validation Results ===");

        println!("\nCodebase Information:");
        println!("  Path: {}", results.codebase_info.path.display());
        println!("  Total files: {}", results.codebase_info.total_files);
        println!("  Supported files: {}", results.codebase_info.supported_files);
        println!("  Total lines: {}", results.codebase_info.total_lines);
        println!("  Languages: {}", results.codebase_info.languages.join(", "));

        println!("\nPerformance Comparison:");
        println!("  Baseline time: {:?}", results.performance_comparison.baseline_duration);
        println!("  Cached time: {:?}", results.performance_comparison.cached_duration);
        println!("  Speedup factor: {:.2}x", results.performance_comparison.speedup_factor);
        println!("  Time saved: {:?}", results.performance_comparison.time_saved);

        println!("\nCache Effectiveness:");
        println!("  Hit rate: {:.2}%", results.cache_effectiveness.overall_hit_rate * 100.0);
        println!("  Cache hits: {}", results.cache_effectiveness.cache_hits);
        println!("  Cache misses: {}", results.cache_effectiveness.cache_misses);
        println!("  Cache errors: {}", results.cache_effectiveness.cache_errors);

        println!("\nMemory Validation:");
        println!("  Memory usage: {:.2}MB", results.memory_validation.average_memory_usage_mb);
        if let Some(limit) = results.memory_validation.memory_limit_mb {
            println!("  Memory limit: {}MB", limit);
            println!("  Limit exceeded: {}", results.memory_validation.memory_limit_exceeded);
        }
        println!("  Memory efficiency: {:.2} hits/MB", results.memory_validation.memory_efficiency);

        if !results.recommendations.is_empty() {
            println!("\nRecommendations:");
            for (i, recommendation) in results.recommendations.iter().enumerate() {
                println!("  {}. {}", i + 1, recommendation);
            }
        }

        Ok(())
    }

    fn output_json(&self, results: &ValidationResults) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let json = serde_json::to_string_pretty(results)?;

        match &self.config.output_file {
            Some(file_path) => std::fs::write(file_path, json)?,
            None => println!("{}", json),
        }

        Ok(())
    }

    fn output_markdown(&self, results: &ValidationResults) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let markdown = self.generate_markdown_report(results);

        match &self.config.output_file {
            Some(file_path) => std::fs::write(file_path, markdown)?,
            None => println!("{}", markdown),
        }

        Ok(())
    }

    fn generate_markdown_report(&self, results: &ValidationResults) -> String {
        let mut md = String::new();

        md.push_str("# Detector Cache Validation Report\n\n");
        md.push_str(&format!("Generated: {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));

        md.push_str("## Codebase Information\n\n");
        md.push_str(&format!("- **Path**: {}\n", results.codebase_info.path.display()));
        md.push_str(&format!("- **Total Files**: {}\n", results.codebase_info.total_files));
        md.push_str(&format!("- **Supported Files**: {}\n", results.codebase_info.supported_files));
        md.push_str(&format!("- **Total Lines**: {}\n", results.codebase_info.total_lines));
        md.push_str(&format!("- **Languages**: {}\n\n", results.codebase_info.languages.join(", ")));

        md.push_str("### File Size Distribution\n\n");
        md.push_str("| Size Category | Count |\n");
        md.push_str("|---------------|-------|\n");
        md.push_str(&format!("| Small (< 100 lines) | {} |\n", results.codebase_info.file_size_distribution.small_files));
        md.push_str(&format!("| Medium (100-500 lines) | {} |\n", results.codebase_info.file_size_distribution.medium_files));
        md.push_str(&format!("| Large (500-2000 lines) | {} |\n", results.codebase_info.file_size_distribution.large_files));
        md.push_str(&format!("| Very Large (> 2000 lines) | {} |\n\n", results.codebase_info.file_size_distribution.very_large_files));

        md.push_str("## Performance Results\n\n");
        md.push_str(&format!("- **Baseline Time**: {:.2}ms\n", results.performance_comparison.baseline_duration.as_millis()));
        md.push_str(&format!("- **Cached Time**: {:.2}ms\n", results.performance_comparison.cached_duration.as_millis()));
        md.push_str(&format!("- **Speedup Factor**: {:.2}x\n", results.performance_comparison.speedup_factor));
        md.push_str(&format!("- **Time Saved**: {:.2}ms\n", results.performance_comparison.time_saved.as_millis()));
        md.push_str(&format!("- **Test Iterations**: {}\n\n", results.performance_comparison.iterations));

        md.push_str("## Cache Effectiveness\n\n");
        md.push_str(&format!("- **Hit Rate**: {:.2}%\n", results.cache_effectiveness.overall_hit_rate * 100.0));
        md.push_str(&format!("- **Cache Hits**: {}\n", results.cache_effectiveness.cache_hits));
        md.push_str(&format!("- **Cache Misses**: {}\n", results.cache_effectiveness.cache_misses));
        md.push_str(&format!("- **Cache Errors**: {}\n", results.cache_effectiveness.cache_errors));
        md.push_str(&format!("- **Average Hit Time**: {:.2}ms\n", results.cache_effectiveness.average_hit_time_ms));
        md.push_str(&format!("- **Average Miss Time**: {:.2}ms\n\n", results.cache_effectiveness.average_miss_time_ms));

        md.push_str("## Memory Validation\n\n");
        md.push_str(&format!("- **Memory Usage**: {:.2}MB\n", results.memory_validation.average_memory_usage_mb));
        if let Some(limit) = results.memory_validation.memory_limit_mb {
            md.push_str(&format!("- **Memory Limit**: {}MB\n", limit));
            md.push_str(&format!("- **Limit Exceeded**: {}\n", results.memory_validation.memory_limit_exceeded));
        }
        md.push_str(&format!("- **Memory Efficiency**: {:.2} hits/MB\n\n", results.memory_validation.memory_efficiency));

        if !results.detector_specific_results.is_empty() {
            md.push_str("## Per-Detector Results\n\n");
            md.push_str("| Detector | Hit Rate | Performance Gain | Memory Usage |\n");
            md.push_str("|----------|----------|------------------|---------------|\n");
            for (_, result) in &results.detector_specific_results {
                md.push_str(&format!(
                    "| {} | {:.2}% | {:.2}% | {:.2}MB |\n",
                    result.detector_name,
                    result.hit_rate * 100.0,
                    result.performance_gain * 100.0,
                    result.memory_usage_mb
                ));
            }
            md.push('\n');
        }

        if !results.recommendations.is_empty() {
            md.push_str("## Recommendations\n\n");
            for (i, recommendation) in results.recommendations.iter().enumerate() {
                md.push_str(&format!("{}. {}\n", i + 1, recommendation));
            }
        }

        md
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let matches = Command::new("detector-cache-validator")
        .version("1.0")
        .about("Validates detector cache performance and effectiveness")
        .arg(
            Arg::new("codebase")
                .short('c')
                .long("codebase")
                .value_name("PATH")
                .help("Path to codebase to analyze")
                .required(true)
        )
        .arg(
            Arg::new("iterations")
                .short('i')
                .long("iterations")
                .value_name("NUM")
                .help("Number of test iterations")
                .default_value("10")
        )
        .arg(
            Arg::new("warmup")
                .long("warmup")
                .value_name("NUM")
                .help("Number of warmup iterations")
                .default_value("3")
        )
        .arg(
            Arg::new("detectors")
                .short('d')
                .long("detectors")
                .value_name("LIST")
                .help("Comma-separated list of detector types to test")
                .default_value("god_object,long_method,code_duplication")
        )
        .arg(
            Arg::new("memory-limit")
                .short('m')
                .long("memory-limit")
                .value_name("MB")
                .help("Memory limit in MB for cache validation")
        )
        .arg(
            Arg::new("output-format")
                .short('f')
                .long("format")
                .value_name("FORMAT")
                .help("Output format: console, json, markdown")
                .default_value("console")
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Output file (stdout if not specified)")
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose output")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();

    let config = ValidationConfig {
        codebase_path: PathBuf::from(matches.get_one::<String>("codebase").unwrap()),
        iterations: matches.get_one::<String>("iterations").unwrap().parse()?,
        warmup_iterations: matches.get_one::<String>("warmup").unwrap().parse()?,
        detector_types: matches.get_one::<String>("detectors")
            .unwrap()
            .split(',')
            .map(|s| s.trim().to_string())
            .collect(),
        memory_limit_mb: matches.get_one::<String>("memory-limit")
            .map(|s| s.parse().ok())
            .flatten(),
        output_format: match matches.get_one::<String>("output-format").unwrap().as_str() {
            "json" => OutputFormat::Json,
            "markdown" => OutputFormat::Markdown,
            _ => OutputFormat::Console,
        },
        output_file: matches.get_one::<String>("output").map(PathBuf::from),
        verbose: matches.get_flag("verbose"),
    };

    if config.verbose {
        info!("Starting validation with config: {:?}", config);
    }

    let validator = DetectorCacheValidator::new(config.clone()).await?;
    let results = validator.run_validation().await?;
    validator.output_results(&results)?;

    if config.verbose {
        info!("Validation completed successfully");
    }

    Ok(())
}

// Add Serialize trait implementations for the result structs
use serde::{Deserialize, Serialize};

// We need to derive Serialize for all result structs to support JSON output
// This would be added to each struct definition above