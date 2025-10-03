//! Cache Performance Benchmark Tool
//!
//! This binary provides cache performance benchmarking on real codebases,
//! measuring hit rates, performance improvements, and memory usage patterns.

use clap::{Arg, Command};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::fs;

use uveddi::analysis::components::cache_manager::{CacheManager, CacheManagerImpl};
use uveddi::ast::{tree_sitter_impl::AstParser, SourceLanguage};
use uveddi::core::logging::{info, warn};

/// Benchmark configuration
#[derive(Debug, Clone)]
struct BenchmarkConfig {
    pub target_path: PathBuf,
    pub file_patterns: Vec<String>,
    pub warmup_runs: u32,
    pub test_runs: u32,
    pub cache_enabled: bool,
    pub report_format: ReportFormat,
}

#[derive(Debug, Clone)]
enum ReportFormat {
    Human,
    Json,
    Csv,
}

/// Benchmark results for a single file
#[derive(Debug, Clone)]
struct FileBenchmarkResult {
    pub file_path: PathBuf,
    pub language: SourceLanguage,
    pub file_size_bytes: u64,
    pub lines_of_code: usize,

    // Performance metrics
    pub first_parse_duration: Duration,
    pub subsequent_parse_durations: Vec<Duration>,
    pub average_hit_duration: Duration,
    pub cache_hit_improvement: f64, // Speedup factor

    // Cache statistics
    pub cache_hit_rate: f64,
    pub memory_usage_bytes: usize,
}

/// Overall benchmark results
#[derive(Debug)]
struct BenchmarkResults {
    pub config: BenchmarkConfig,
    pub file_results: Vec<FileBenchmarkResult>,
    pub total_duration: Duration,
    pub overall_hit_rate: f64,
    pub total_memory_usage: usize,
    pub average_speedup: f64,
}

impl BenchmarkResults {
    /// Generate human-readable report
    pub fn print_human_report(&self) {
        println!("\n🚀 Cache Performance Benchmark Results");
        println!("====================================");
        println!("Target Directory: {}", self.config.target_path.display());
        println!("Files Analyzed: {}", self.file_results.len());
        println!("Total Duration: {:?}", self.total_duration);
        println!("Overall Cache Hit Rate: {:.2}%", self.overall_hit_rate * 100.0);
        println!("Average Speedup: {:.2}x", self.average_speedup);
        println!("Total Memory Usage: {} KB", self.total_memory_usage / 1024);

        println!("\n📊 Per-File Results:");
        println!("{:<50} {:<8} {:<12} {:<12} {:<10} {:<8}",
            "File", "Lang", "Size (KB)", "First Parse", "Avg Hit", "Speedup");
        println!("{}", "-".repeat(100));

        for result in &self.file_results {
            let file_name = result.file_path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown");

            println!("{:<50} {:<8} {:<12} {:<12} {:<10} {:.2}x",
                file_name,
                format!("{:?}", result.language),
                result.file_size_bytes / 1024,
                format!("{:.1}ms", result.first_parse_duration.as_secs_f64() * 1000.0),
                format!("{:.1}ms", result.average_hit_duration.as_secs_f64() * 1000.0),
                result.cache_hit_improvement
            );
        }

        println!("\n💡 Performance Analysis:");

        // Categorize files by language
        let mut lang_stats: HashMap<SourceLanguage, Vec<&FileBenchmarkResult>> = HashMap::new();
        for result in &self.file_results {
            lang_stats.entry(result.language).or_default().push(result);
        }

        for (lang, results) in lang_stats {
            let avg_speedup: f64 = results.iter()
                .map(|r| r.cache_hit_improvement)
                .sum::<f64>() / results.len() as f64;

            let avg_hit_rate: f64 = results.iter()
                .map(|r| r.cache_hit_rate)
                .sum::<f64>() / results.len() as f64;

            println!("  {:?}: {} files, {:.2}x average speedup, {:.1}% hit rate",
                lang, results.len(), avg_speedup, avg_hit_rate * 100.0);
        }

        // Memory usage breakdown
        let total_mb = self.total_memory_usage as f64 / (1024.0 * 1024.0);
        let avg_per_file = self.total_memory_usage / self.file_results.len().max(1);
        println!("\n🧠 Memory Usage:");
        println!("  Total: {:.2} MB", total_mb);
        println!("  Average per file: {} KB", avg_per_file / 1024);

        if self.average_speedup > 2.0 {
            println!("\n✅ Cache performance is excellent (>2x speedup)");
        } else if self.average_speedup > 1.5 {
            println!("\n⚡ Cache performance is good (>1.5x speedup)");
        } else {
            println!("\n⚠️  Cache performance could be improved (<1.5x speedup)");
        }
    }

    /// Generate JSON report
    pub fn print_json_report(&self) {
        let json_data = serde_json::json!({
            "config": {
                "target_path": self.config.target_path,
                "warmup_runs": self.config.warmup_runs,
                "test_runs": self.config.test_runs,
                "cache_enabled": self.config.cache_enabled
            },
            "summary": {
                "files_analyzed": self.file_results.len(),
                "total_duration_ms": self.total_duration.as_millis(),
                "overall_hit_rate": self.overall_hit_rate,
                "average_speedup": self.average_speedup,
                "total_memory_usage_bytes": self.total_memory_usage
            },
            "file_results": self.file_results.iter().map(|result| {
                serde_json::json!({
                    "file_path": result.file_path,
                    "language": format!("{:?}", result.language),
                    "file_size_bytes": result.file_size_bytes,
                    "lines_of_code": result.lines_of_code,
                    "first_parse_duration_ms": result.first_parse_duration.as_millis(),
                    "average_hit_duration_ms": result.average_hit_duration.as_millis(),
                    "cache_hit_improvement": result.cache_hit_improvement,
                    "cache_hit_rate": result.cache_hit_rate,
                    "memory_usage_bytes": result.memory_usage_bytes
                })
            }).collect::<Vec<_>>()
        });

        println!("{}", serde_json::to_string_pretty(&json_data).unwrap());
    }
}

/// Find files matching patterns in the target directory
fn find_source_files(path: &Path, patterns: &[String]) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<PathBuf>, Box<dyn std::error::Error>>> + Send + '_>> {
    Box::pin(async move {
        let mut files = Vec::new();

        if path.is_file() {
            files.push(path.to_path_buf());
            return Ok(files);
        }

        let mut entries = fs::read_dir(path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();

            if entry_path.is_dir() {
                // Recursively search subdirectories
                let mut sub_files = find_source_files(&entry_path, patterns).await?;
                files.append(&mut sub_files);
        } else if let Some(file_name) = entry_path.file_name().and_then(|n| n.to_str()) {
            // Check if file matches any pattern
            let matches_pattern = patterns.is_empty() || patterns.iter().any(|pattern| {
                if pattern == "*" {
                    true
                } else if pattern.starts_with("*.") {
                    file_name.ends_with(&pattern[1..])
                } else {
                    file_name.contains(pattern)
                }
            });

            if matches_pattern {
                // Check if it's a source file we can parse
                if let Some(ext) = entry_path.extension().and_then(|e| e.to_str()) {
                    match ext {
                        "rs" | "py" | "js" | "ts" | "jsx" | "tsx" => {
                            files.push(entry_path);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

        Ok(files)
    })
}

/// Detect language from file extension
fn detect_language(file_path: &Path) -> SourceLanguage {
    match file_path.extension().and_then(|e| e.to_str()) {
        Some("rs") => SourceLanguage::Rust,
        Some("py") => SourceLanguage::Python,
        Some("js") | Some("jsx") => SourceLanguage::JavaScript,
        Some("ts") | Some("tsx") => SourceLanguage::TypeScript,
        _ => SourceLanguage::Rust, // Default
    }
}

/// Benchmark a single file
async fn benchmark_file(
    file_path: &Path,
    cache_manager: &Arc<CacheManagerImpl>,
    config: &BenchmarkConfig
) -> Result<FileBenchmarkResult, Box<dyn std::error::Error>> {
    let language = detect_language(file_path);

    // Get file metadata
    let metadata = fs::metadata(file_path).await?;
    let file_size_bytes = metadata.len();

    let content = fs::read_to_string(file_path).await?;
    let lines_of_code = content.lines().count();

    info!("Benchmarking file: {} ({:?}, {} KB)",
        file_path.display(), language, file_size_bytes / 1024);

    // First parse (cache miss)
    let start = Instant::now();
    let _first_parse = cache_manager.get_or_parse_ast(file_path).await?;
    let first_parse_duration = start.elapsed();

    // Warmup runs
    for _ in 0..config.warmup_runs {
        let _ = cache_manager.get_or_parse_ast(file_path).await?;
    }

    // Test runs for cache hits
    let mut hit_durations = Vec::new();
    for _ in 0..config.test_runs {
        let start = Instant::now();
        let _ = cache_manager.get_or_parse_ast(file_path).await?;
        hit_durations.push(start.elapsed());
    }

    // Calculate statistics
    let total_hit_time: Duration = hit_durations.iter().sum();
    let average_hit_duration = total_hit_time / config.test_runs;

    let cache_hit_improvement = if average_hit_duration.as_nanos() > 0 {
        first_parse_duration.as_nanos() as f64 / average_hit_duration.as_nanos() as f64
    } else {
        1.0
    };

    // Get cache statistics
    let cache_stats = cache_manager.get_cache_stats().await;

    Ok(FileBenchmarkResult {
        file_path: file_path.to_path_buf(),
        language,
        file_size_bytes,
        lines_of_code,
        first_parse_duration,
        subsequent_parse_durations: hit_durations,
        average_hit_duration,
        cache_hit_improvement,
        cache_hit_rate: cache_stats.ast_hit_rate,
        memory_usage_bytes: cache_stats.total_memory_usage,
    })
}

/// Run cache performance benchmark
async fn run_benchmark(config: BenchmarkConfig) -> Result<BenchmarkResults, Box<dyn std::error::Error>> {
    let start_time = Instant::now();

    info!("Starting cache benchmark on: {}", config.target_path.display());

    // Find source files
    let files = find_source_files(&config.target_path, &config.file_patterns).await?;
    info!("Found {} source files to benchmark", files.len());

    if files.is_empty() {
        warn!("No source files found matching patterns: {:?}", config.file_patterns);
        return Ok(BenchmarkResults {
            config,
            file_results: vec![],
            total_duration: start_time.elapsed(),
            overall_hit_rate: 0.0,
            total_memory_usage: 0,
            average_speedup: 1.0,
        });
    }

    // Create cache manager
    let cache_manager = Arc::new(CacheManagerImpl::new().await?);

    // Benchmark each file
    let mut file_results = Vec::new();
    for file_path in files.iter() {
        match benchmark_file(file_path, &cache_manager, &config).await {
            Ok(result) => file_results.push(result),
            Err(e) => {
                warn!("Failed to benchmark {}: {}", file_path.display(), e);
            }
        }
    }

    // Calculate overall statistics
    let overall_hit_rate = if !file_results.is_empty() {
        file_results.iter().map(|r| r.cache_hit_rate).sum::<f64>() / file_results.len() as f64
    } else {
        0.0
    };

    let average_speedup = if !file_results.is_empty() {
        file_results.iter().map(|r| r.cache_hit_improvement).sum::<f64>() / file_results.len() as f64
    } else {
        1.0
    };

    let total_memory_usage = file_results.iter()
        .map(|r| r.memory_usage_bytes)
        .max()
        .unwrap_or(0); // Max because cache is shared

    Ok(BenchmarkResults {
        config,
        file_results,
        total_duration: start_time.elapsed(),
        overall_hit_rate,
        total_memory_usage,
        average_speedup,
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let matches = Command::new("cache-benchmark")
        .version("1.0")
        .about("Cache Performance Benchmark Tool for Uveddi")
        .arg(
            Arg::new("path")
                .help("Target file or directory to benchmark")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("patterns")
                .long("patterns")
                .help("File patterns to match (e.g., '*.rs,*.py')")
                .default_value("*.rs,*.py,*.js,*.ts")
                .value_delimiter(','),
        )
        .arg(
            Arg::new("warmup-runs")
                .long("warmup-runs")
                .help("Number of warmup runs per file")
                .default_value("3")
                .value_parser(clap::value_parser!(u32)),
        )
        .arg(
            Arg::new("test-runs")
                .long("test-runs")
                .help("Number of test runs per file for measuring cache hits")
                .default_value("5")
                .value_parser(clap::value_parser!(u32)),
        )
        .arg(
            Arg::new("format")
                .long("format")
                .help("Output format")
                .value_parser(["human", "json", "csv"])
                .default_value("human"),
        )
        .get_matches();

    let target_path = PathBuf::from(matches.get_one::<String>("path").unwrap());
    let patterns: Vec<String> = matches
        .get_many::<String>("patterns")
        .unwrap_or_default()
        .map(|s| s.to_string())
        .collect();
    let warmup_runs = *matches.get_one::<u32>("warmup-runs").unwrap();
    let test_runs = *matches.get_one::<u32>("test-runs").unwrap();

    let report_format = match matches.get_one::<String>("format").unwrap().as_str() {
        "json" => ReportFormat::Json,
        "csv" => ReportFormat::Csv,
        _ => ReportFormat::Human,
    };

    let config = BenchmarkConfig {
        target_path,
        file_patterns: patterns,
        warmup_runs,
        test_runs,
        cache_enabled: true,
        report_format,
    };

    // Run benchmark
    let results = run_benchmark(config).await?;

    // Print results
    match results.config.report_format {
        ReportFormat::Human => results.print_human_report(),
        ReportFormat::Json => results.print_json_report(),
        ReportFormat::Csv => {
            println!("CSV format not yet implemented");
            results.print_human_report();
        }
    }

    Ok(())
}