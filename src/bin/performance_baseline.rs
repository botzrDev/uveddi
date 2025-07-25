use uveddi::analysis::performance::PerformanceAnalyzer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting UV-49 Performance Baseline Analysis");

    let analyzer = PerformanceAnalyzer::new()?;

    match analyzer.analyze_rendering_performance().await {
        Ok(baseline) => {
            println!("✅ Baseline analysis completed successfully!");
            println!("📊 Summary:");
            println!("  - Total measurements: {}", baseline.rendering_times.len());
            println!(
                "  - P50 (median): {:.2}ms",
                baseline.percentile_analysis.p50_ms
            );
            println!("  - P95: {:.2}ms", baseline.percentile_analysis.p95_ms);
            println!("  - P99: {:.2}ms", baseline.percentile_analysis.p99_ms);
            println!(
                "  - Cache hit rate: {:.1}%",
                baseline.cache_performance.hit_rate * 100.0
            );
            println!(
                "  - Critical bottlenecks: {}",
                baseline
                    .bottlenecks
                    .iter()
                    .filter(|b| matches!(
                        b.impact_level,
                        uveddi::analysis::performance::BottleneckSeverity::Critical
                    ))
                    .count()
            );

            // Check if we meet the <50ms target
            let target_compliance = baseline
                .rendering_times
                .iter()
                .filter(|t| t.as_millis() < 50)
                .count() as f64
                / baseline.rendering_times.len() as f64
                * 100.0;

            println!("  - <50ms target compliance: {:.1}%", target_compliance);

            if target_compliance >= 95.0 {
                println!("✅ UV-12 target achievement: PASSED");
            } else {
                println!("❌ UV-12 target achievement: FAILED (needs optimization)");
            }
        }
        Err(e) => {
            println!("❌ Baseline analysis failed: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
