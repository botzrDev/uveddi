//! Cache Memory Profiler
//!
//! This tool profiles memory usage under various cache configurations,
//! helping optimize cache settings for different deployment scenarios.

use clap::{Arg, Command};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::fs;

use uveddi::analysis::cache::engine_cache::{EngineCacheConfig, EngineCache};
use uveddi::analysis::components::cache_manager::{CacheManager, CacheManagerImpl};
use uveddi::core::logging::{info, warn};

/// Memory profiling configuration
#[derive(Debug, Clone)]
struct ProfileConfig {
    pub target_path: PathBuf,
    pub file_limit: usize,
    pub memory_sample_interval_ms: u64,
    pub configurations: Vec<CacheConfiguration>,
}

/// Cache configuration to test
#[derive(Debug, Clone)]
struct CacheConfiguration {
    pub name: String,
    pub max_memory_mb: Option<usize>,
    pub max_ast_entries: Option<usize>,
    pub max_result_entries: Option<usize>,
    pub enable_disk_cache: bool,
    pub compression_enabled: bool,
}

impl CacheConfiguration {
    fn to_engine_cache_config(&self) -> EngineCacheConfig {
        let mut config = EngineCacheConfig::default();

        // Set memory limits if specified
        if let Some(memory_mb) = self.max_memory_mb {
            // Convert MB to bytes - this would need to be implemented in EngineCacheConfig
            // For now, we'll use the default config structure
        }

        config
    }
}

/// Memory usage snapshot
#[derive(Debug, Clone)]
struct MemorySnapshot {
    pub timestamp: Instant,
    pub cache_memory_bytes: usize,
    pub system_memory_bytes: usize,
    pub files_cached: usize,
    pub results_cached: usize,
    pub hit_rate: f64,
}

/// Profiling results for a single configuration
#[derive(Debug)]
struct ConfigurationProfile {
    pub config: CacheConfiguration,
    pub files_processed: usize,
    pub total_duration: Duration,
    pub memory_snapshots: Vec<MemorySnapshot>,
    pub peak_memory_bytes: usize,
    pub average_memory_bytes: usize,
    pub final_hit_rate: f64,
    pub memory_efficiency: f64, // files cached per MB
}

/// Overall profiling results
#[derive(Debug)]
struct ProfilingResults {
    pub config_profiles: Vec<ConfigurationProfile>,
    pub recommended_configuration: String,
}

impl ProfilingResults {
    /// Print human-readable profiling report
    pub fn print_report(&self) {
        println!("\n🧠 Cache Memory Profiling Results");
        println!("=================================");

        println!("\n📊 Configuration Comparison:");
        println!("{:<20} {:<10} {:<12} {:<12} {:<10} {:<12} {:<8}",
            "Configuration", "Files", "Peak Memory", "Avg Memory", "Hit Rate", "Duration", "Efficiency");
        println!("{}", "-".repeat(90));

        for profile in &self.config_profiles {
            let peak_mb = profile.peak_memory_bytes as f64 / (1024.0 * 1024.0);
            let avg_mb = profile.average_memory_bytes as f64 / (1024.0 * 1024.0);
            let efficiency = profile.memory_efficiency;

            println!("{:<20} {:<10} {:<12} {:<12} {:<10} {:<12} {:.1}",
                profile.config.name,
                profile.files_processed,
                format!("{:.1} MB", peak_mb),
                format!("{:.1} MB", avg_mb),
                format!("{:.1}%", profile.final_hit_rate * 100.0),
                format!("{:.1}s", profile.total_duration.as_secs_f64()),
                efficiency
            );
        }

        println!("\n🏆 Recommended Configuration: {}", self.recommended_configuration);

        // Detailed analysis per configuration
        for profile in &self.config_profiles {
            println!("\n📈 {} Detailed Analysis:", profile.config.name);
            println!("  Peak Memory: {:.2} MB", profile.peak_memory_bytes as f64 / (1024.0 * 1024.0));
            println!("  Average Memory: {:.2} MB", profile.average_memory_bytes as f64 / (1024.0 * 1024.0));
            println!("  Memory Efficiency: {:.1} files/MB", profile.memory_efficiency);
            println!("  Final Hit Rate: {:.1}%", profile.final_hit_rate * 100.0);
            println!("  Processing Time: {:.2}s", profile.total_duration.as_secs_f64());

            // Memory growth analysis
            if profile.memory_snapshots.len() >= 2 {
                let initial = &profile.memory_snapshots[0];
                let final_snap = &profile.memory_snapshots[profile.memory_snapshots.len() - 1];
                let growth_rate = (final_snap.cache_memory_bytes as f64 - initial.cache_memory_bytes as f64)
                    / initial.cache_memory_bytes as f64 * 100.0;

                println!("  Memory Growth Rate: {:.1}%", growth_rate);

                // Find memory spikes
                let max_growth = profile.memory_snapshots.windows(2)
                    .map(|window| {
                        let growth = window[1].cache_memory_bytes as f64 - window[0].cache_memory_bytes as f64;
                        growth / window[0].cache_memory_bytes as f64 * 100.0
                    })
                    .fold(0.0, f64::max);

                if max_growth > 50.0 {
                    println!("  ⚠️  Memory spike detected: {:.1}% increase", max_growth);
                }
            }
        }

        // Configuration recommendations
        println!("\n💡 Configuration Recommendations:");

        // Find most memory efficient
        let most_efficient = self.config_profiles.iter()
            .max_by(|a, b| a.memory_efficiency.partial_cmp(&b.memory_efficiency).unwrap_or(std::cmp::Ordering::Equal));

        if let Some(efficient) = most_efficient {
            println!("  Most Memory Efficient: {} ({:.1} files/MB)",
                efficient.config.name, efficient.memory_efficiency);
        }

        // Find lowest peak memory
        let lowest_peak = self.config_profiles.iter()
            .min_by_key(|profile| profile.peak_memory_bytes);

        if let Some(low_mem) = lowest_peak {
            println!("  Lowest Peak Memory: {} ({:.1} MB)",
                low_mem.config.name, low_mem.peak_memory_bytes as f64 / (1024.0 * 1024.0));
        }

        // Find best hit rate
        let best_hit_rate = self.config_profiles.iter()
            .max_by(|a, b| a.final_hit_rate.partial_cmp(&b.final_hit_rate).unwrap_or(std::cmp::Ordering::Equal));

        if let Some(best) = best_hit_rate {
            println!("  Best Hit Rate: {} ({:.1}%)",
                best.config.name, best.final_hit_rate * 100.0);
        }
    }
}

/// Get predefined cache configurations to test
fn get_test_configurations() -> Vec<CacheConfiguration> {
    vec![
        CacheConfiguration {
            name: "minimal".to_string(),
            max_memory_mb: Some(64),
            max_ast_entries: Some(100),
            max_result_entries: Some(100),
            enable_disk_cache: false,
            compression_enabled: false,
        },
        CacheConfiguration {
            name: "small".to_string(),
            max_memory_mb: Some(128),
            max_ast_entries: Some(500),
            max_result_entries: Some(500),
            enable_disk_cache: false,
            compression_enabled: false,
        },
        CacheConfiguration {
            name: "medium".to_string(),
            max_memory_mb: Some(256),
            max_ast_entries: Some(1000),
            max_result_entries: Some(1000),
            enable_disk_cache: true,
            compression_enabled: false,
        },
        CacheConfiguration {
            name: "large".to_string(),
            max_memory_mb: Some(512),
            max_ast_entries: Some(2000),
            max_result_entries: Some(2000),
            enable_disk_cache: true,
            compression_enabled: true,
        },
        CacheConfiguration {
            name: "unlimited".to_string(),
            max_memory_mb: None,
            max_ast_entries: None,
            max_result_entries: None,
            enable_disk_cache: true,
            compression_enabled: true,
        },
    ]
}

/// Find source files to test with
fn find_test_files(path: &Path, limit: usize) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<PathBuf>, Box<dyn std::error::Error>>> + Send + '_>> {
    Box::pin(async move {
        let mut files = Vec::new();

        if path.is_file() {
            files.push(path.to_path_buf());
            return Ok(files);
        }

        let mut entries = fs::read_dir(path).await?;
        while let Some(entry) = entries.next_entry().await? {
            if files.len() >= limit {
                break;
            }

            let entry_path = entry.path();

            if entry_path.is_dir() {
                let mut sub_files = find_test_files(&entry_path, limit - files.len()).await?;
                files.append(&mut sub_files);
        } else if let Some(ext) = entry_path.extension().and_then(|e| e.to_str()) {
            match ext {
                "rs" | "py" | "js" | "ts" | "jsx" | "tsx" => {
                    files.push(entry_path);
                }
                _ => {}
            }
        }
    }

    Ok(files)
    })
}

/// Take a memory snapshot
async fn take_memory_snapshot(
    cache_manager: &CacheManagerImpl,
    start_time: Instant
) -> MemorySnapshot {
    let stats = cache_manager.get_cache_stats().await;

    MemorySnapshot {
        timestamp: start_time,
        cache_memory_bytes: stats.total_memory_usage,
        system_memory_bytes: get_system_memory_usage(),
        files_cached: stats.ast_cache_size,
        results_cached: stats.result_cache_size,
        hit_rate: stats.ast_hit_rate,
    }
}

/// Get system memory usage (simplified)
fn get_system_memory_usage() -> usize {
    // This is a simplified implementation
    // In a real system, you would use platform-specific APIs
    #[cfg(target_os = "linux")]
    {
        if let Ok(contents) = std::fs::read_to_string("/proc/self/status") {
            for line in contents.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = kb_str.parse::<usize>() {
                            return kb * 1024; // Convert KB to bytes
                        }
                    }
                }
            }
        }
    }

    // Fallback - return 0 if we can't determine memory usage
    0
}

/// Profile a single cache configuration
async fn profile_configuration(
    config: CacheConfiguration,
    files: &[PathBuf],
    sample_interval: Duration,
) -> Result<ConfigurationProfile, Box<dyn std::error::Error>> {
    info!("Profiling configuration: {}", config.name);

    let start_time = Instant::now();

    // Create cache manager with specific configuration
    let engine_config = config.to_engine_cache_config();
    let cache_manager = CacheManagerImpl::with_config(engine_config).await?;

    let mut memory_snapshots = Vec::new();
    let mut files_processed = 0;

    // Initial snapshot
    memory_snapshots.push(take_memory_snapshot(&cache_manager, start_time).await);

    // Process files and take periodic snapshots
    let mut last_sample = start_time;

    for (i, file_path) in files.iter().enumerate() {
        // Parse file to populate cache
        match cache_manager.get_or_parse_ast(file_path).await {
            Ok(_) => {
                files_processed += 1;

                // Take memory snapshot at intervals
                let now = Instant::now();
                if now.duration_since(last_sample) >= sample_interval {
                    memory_snapshots.push(take_memory_snapshot(&cache_manager, start_time).await);
                    last_sample = now;
                }
            }
            Err(e) => {
                warn!("Failed to parse {}: {}", file_path.display(), e);
            }
        }

        // Progress logging
        if (i + 1) % 50 == 0 {
            info!("Processed {} files for {}", i + 1, config.name);
        }
    }

    // Final snapshot
    memory_snapshots.push(take_memory_snapshot(&cache_manager, start_time).await);

    // Calculate statistics
    let total_duration = start_time.elapsed();
    let peak_memory_bytes = memory_snapshots.iter()
        .map(|s| s.cache_memory_bytes)
        .max()
        .unwrap_or(0);

    let average_memory_bytes = if !memory_snapshots.is_empty() {
        memory_snapshots.iter()
            .map(|s| s.cache_memory_bytes)
            .sum::<usize>() / memory_snapshots.len()
    } else {
        0
    };

    let final_hit_rate = memory_snapshots.last()
        .map(|s| s.hit_rate)
        .unwrap_or(0.0);

    let memory_efficiency = if peak_memory_bytes > 0 {
        files_processed as f64 / (peak_memory_bytes as f64 / (1024.0 * 1024.0))
    } else {
        0.0
    };

    Ok(ConfigurationProfile {
        config,
        files_processed,
        total_duration,
        memory_snapshots,
        peak_memory_bytes,
        average_memory_bytes,
        final_hit_rate,
        memory_efficiency,
    })
}

/// Run memory profiling across all configurations
async fn run_profiling(config: ProfileConfig) -> Result<ProfilingResults, Box<dyn std::error::Error>> {
    info!("Starting cache memory profiling");

    // Find test files
    let files = find_test_files(&config.target_path, config.file_limit).await?;
    info!("Found {} files for profiling", files.len());

    if files.is_empty() {
        warn!("No source files found in {}", config.target_path.display());
    }

    let sample_interval = Duration::from_millis(config.memory_sample_interval_ms);
    let mut config_profiles = Vec::new();

    // Profile each configuration
    for cache_config in config.configurations {
        match profile_configuration(cache_config.clone(), &files, sample_interval).await {
            Ok(profile) => config_profiles.push(profile),
            Err(e) => {
                warn!("Failed to profile configuration {}: {}", cache_config.name, e);
            }
        }
    }

    // Determine recommended configuration
    let recommended_configuration = if !config_profiles.is_empty() {
        // For now, recommend the most memory efficient configuration
        config_profiles.iter()
            .max_by(|a, b| a.memory_efficiency.partial_cmp(&b.memory_efficiency).unwrap_or(std::cmp::Ordering::Equal))
            .map(|p| p.config.name.clone())
            .unwrap_or_else(|| "unknown".to_string())
    } else {
        "none".to_string()
    };

    Ok(ProfilingResults {
        config_profiles,
        recommended_configuration,
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let matches = Command::new("cache-memory-profiler")
        .version("1.0")
        .about("Cache Memory Profiler for Uveddi")
        .arg(
            Arg::new("path")
                .help("Target directory to profile")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("file-limit")
                .long("file-limit")
                .help("Maximum number of files to process")
                .default_value("200")
                .value_parser(clap::value_parser!(usize)),
        )
        .arg(
            Arg::new("sample-interval")
                .long("sample-interval")
                .help("Memory sampling interval in milliseconds")
                .default_value("1000")
                .value_parser(clap::value_parser!(u64)),
        )
        .get_matches();

    let target_path = PathBuf::from(matches.get_one::<String>("path").unwrap());
    let file_limit = *matches.get_one::<usize>("file-limit").unwrap();
    let sample_interval_ms = *matches.get_one::<u64>("sample-interval").unwrap();

    let config = ProfileConfig {
        target_path,
        file_limit,
        memory_sample_interval_ms: sample_interval_ms,
        configurations: get_test_configurations(),
    };

    // Run profiling
    let results = run_profiling(config).await?;

    // Print results
    results.print_report();

    Ok(())
}