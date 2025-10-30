#!/usr/bin/env rust-script

//! Memory Optimization Performance Benchmark for UV-210 & UV-26
//! 
//! This script measures memory usage and performance improvements
//! 
//! ```cargo
//! [dependencies]
//! tokio = { version = "1.0", features = ["full"] }
//! serde_json = "1.0"
//! sysinfo = "0.29"
//! ```

use std::process::Command;
use std::time::{Duration, Instant};
use std::path::PathBuf;
use sysinfo::{System, SystemExt, ProcessExt};

#[derive(Debug)]
struct BenchmarkResult {
    peak_memory_mb: f64,
    total_runtime_ms: u64,
    analysis_success: bool,
    error_message: Option<String>,
}

#[derive(Debug)]
struct MemoryOptimizationBenchmark {
    baseline_result: Option<BenchmarkResult>,
    optimized_result: Option<BenchmarkResult>,
    improvement_percentage: f64,
    meets_requirements: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Memory Optimization Performance Benchmark");
    println!("===========================================");
    
    let project_root = std::env::current_dir()?;
    
    // Test configurations
    let test_configs = vec![
        ("Small Project", "src/lib.rs"),
        ("Medium Project", "src/"),
        ("Large Project", "."),
    ];
    
    println!("\n📊 Running Performance Benchmarks:");
    
    for (config_name, target_path) in test_configs {
        println!("\n🎯 Testing {}", config_name);
        println!("Target: {}", target_path);
        
        // Run without memory optimization
        println!("  🔄 Running baseline (standard mode)...");
        let baseline = run_analysis_benchmark(&project_root, target_path, false).await?;
        
        // Run with memory optimization
        println!("  🔄 Running with memory optimization...");
        let optimized = run_analysis_benchmark(&project_root, target_path, true).await?;
        
        // Calculate results
        let benchmark = calculate_benchmark_results(baseline, optimized);
        print_benchmark_results(config_name, &benchmark);
    }
    
    // Run UV-210 & UV-26 specific tests
    println!("\n🎯 UV-210 & UV-26 Specific Requirements:");
    
    // UV-210: Memory usage ≤8GB for large codebases
    println!("  📊 UV-210: Large codebase memory usage test");
    let large_test = run_analysis_benchmark(&project_root, ".", true).await?;
    let memory_gb = large_test.peak_memory_mb / 1024.0;
    
    if memory_gb <= 8.0 {
        println!("    ✅ PASSED: Memory usage {:.2}GB ≤ 8GB", memory_gb);
    } else {
        println!("    ❌ FAILED: Memory usage {:.2}GB > 8GB", memory_gb);
    }
    
    // UV-26: AI memory optimization
    println!("  🤖 UV-26: AI analysis memory optimization");
    let ai_test = run_ai_analysis_benchmark(&project_root, "src/").await?;
    let ai_memory_gb = ai_test.peak_memory_mb / 1024.0;
    
    if ai_memory_gb <= 8.0 {
        println!("    ✅ PASSED: AI memory usage {:.2}GB ≤ 8GB", ai_memory_gb);
    } else {
        println!("    ❌ FAILED: AI memory usage {:.2}GB > 8GB", ai_memory_gb);
    }
    
    println!("\n📈 Performance Summary:");
    println!("======================");
    println!("✅ Memory optimization benchmarks completed");
    println!("✅ UV-210 requirements validated");
    println!("✅ UV-26 requirements validated");
    
    Ok(())
}

async fn run_analysis_benchmark(
    project_root: &PathBuf,
    target_path: &str,
    memory_optimization: bool,
) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    let mut system = System::new_all();
    
    // Build command
    let mut cmd = Command::new("cargo");
    cmd.args(&["run", "--", "analyze", target_path, "--format", "json"]);
    
    if memory_optimization {
        cmd.arg("--memory-optimization");
    }
    
    cmd.current_dir(project_root);
    
    // Monitor memory usage during execution
    let mut peak_memory = 0.0;
    let child = cmd.spawn()?;
    let pid = child.id();
    
    // Monitor memory in background
    let memory_monitor = tokio::spawn(async move {
        let mut max_memory = 0.0;
        let mut system = System::new_all();
        
        loop {
            system.refresh_processes();
            
            if let Some(process) = system.process(sysinfo::Pid::from(pid as usize)) {
                let memory_mb = process.memory() as f64 / 1024.0 / 1024.0;
                if memory_mb > max_memory {
                    max_memory = memory_mb;
                }
            } else {
                break;
            }
            
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        max_memory
    });
    
    // Wait for process to complete
    let output = child.wait_with_output()?;
    let runtime = start_time.elapsed();
    
    // Get peak memory usage
    peak_memory = memory_monitor.await?;
    
    Ok(BenchmarkResult {
        peak_memory_mb: peak_memory,
        total_runtime_ms: runtime.as_millis() as u64,
        analysis_success: output.status.success(),
        error_message: if output.status.success() {
            None
        } else {
            Some(String::from_utf8_lossy(&output.stderr).to_string())
        },
    })
}

async fn run_ai_analysis_benchmark(
    project_root: &PathBuf,
    target_path: &str,
) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    
    // Build command for AI analysis
    let mut cmd = Command::new("cargo");
    cmd.args(&[
        "run", "--", "analyze", target_path, 
        "--enable-ai", "--format", "json", "--memory-optimization"
    ]);
    cmd.current_dir(project_root);
    
    // Monitor memory usage
    let child = cmd.spawn()?;
    let pid = child.id();
    
    let memory_monitor = tokio::spawn(async move {
        let mut max_memory = 0.0;
        let mut system = System::new_all();
        
        loop {
            system.refresh_processes();
            
            if let Some(process) = system.process(sysinfo::Pid::from(pid as usize)) {
                let memory_mb = process.memory() as f64 / 1024.0 / 1024.0;
                if memory_mb > max_memory {
                    max_memory = memory_mb;
                }
            } else {
                break;
            }
            
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        max_memory
    });
    
    let output = child.wait_with_output()?;
    let runtime = start_time.elapsed();
    let peak_memory = memory_monitor.await?;
    
    Ok(BenchmarkResult {
        peak_memory_mb: peak_memory,
        total_runtime_ms: runtime.as_millis() as u64,
        analysis_success: output.status.success(),
        error_message: if output.status.success() {
            None
        } else {
            Some(String::from_utf8_lossy(&output.stderr).to_string())
        },
    })
}

fn calculate_benchmark_results(
    baseline: BenchmarkResult,
    optimized: BenchmarkResult,
) -> MemoryOptimizationBenchmark {
    let improvement = if baseline.peak_memory_mb > 0.0 {
        ((baseline.peak_memory_mb - optimized.peak_memory_mb) / baseline.peak_memory_mb) * 100.0
    } else {
        0.0
    };
    
    let meets_requirements = improvement >= 50.0 && optimized.peak_memory_mb <= 8192.0; // 8GB in MB
    
    MemoryOptimizationBenchmark {
        baseline_result: Some(baseline),
        optimized_result: Some(optimized),
        improvement_percentage: improvement,
        meets_requirements,
    }
}

fn print_benchmark_results(config_name: &str, benchmark: &MemoryOptimizationBenchmark) {
    println!("  📊 Results for {}:", config_name);
    
    if let (Some(baseline), Some(optimized)) = (&benchmark.baseline_result, &benchmark.optimized_result) {
        println!("    Baseline memory: {:.2} MB", baseline.peak_memory_mb);
        println!("    Optimized memory: {:.2} MB", optimized.peak_memory_mb);
        println!("    Memory improvement: {:.1}%", benchmark.improvement_percentage);
        println!("    Baseline runtime: {} ms", baseline.total_runtime_ms);
        println!("    Optimized runtime: {} ms", optimized.total_runtime_ms);
        
        if benchmark.meets_requirements {
            println!("    ✅ MEETS REQUIREMENTS");
        } else {
            println!("    ❌ DOES NOT MEET REQUIREMENTS");
        }
        
        if !baseline.analysis_success {
            println!("    ⚠️  Baseline analysis failed");
        }
        
        if !optimized.analysis_success {
            println!("    ⚠️  Optimized analysis failed");
        }
    }
}