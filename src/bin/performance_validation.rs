use uveddi::analysis::performance::PerformanceValidator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Starting UV-48 Performance Validation Suite");
    println!("===============================================");
    
    let validator = PerformanceValidator::new();
    
    match validator.run_comprehensive_validation().await {
        Ok(test_suite) => {
            println!("\n🎯 VALIDATION COMPLETE");
            println!("=====================");
            println!("📊 Summary:");
            println!("  - Total Tests: {}", test_suite.summary.total_tests);
            println!("  - Passed: {}", test_suite.summary.passed_tests);
            println!("  - Failed: {}", test_suite.summary.failed_tests);
            println!("  - Success Rate: {:.1}%", 
                test_suite.summary.passed_tests as f64 / test_suite.summary.total_tests as f64 * 100.0);
            println!("  - Target Compliance: {:.1}%", test_suite.summary.overall_target_compliance);
            
            println!("\n📋 Test Results:");
            for result in &test_suite.test_results {
                let status = if result.success { "✅" } else { "❌" };
                println!("  {} {} ({:.1}ms avg, {:.1}% success)", 
                    status, 
                    result.test_name,
                    result.details.average_time_ms,
                    result.details.success_count as f64 / result.details.iterations as f64 * 100.0
                );
            }
            
            println!("\n💡 Recommendations:");
            for (i, rec) in test_suite.summary.recommendations.iter().enumerate() {
                println!("  {}. {}", i + 1, rec);
            }
            
            // Overall assessment
            if test_suite.summary.failed_tests == 0 && test_suite.summary.overall_target_compliance >= 95.0 {
                println!("\n🎉 UV-48 VALIDATION: PASSED");
                println!("✅ System ready for UV-12 fine-tuning");
            } else if test_suite.summary.overall_target_compliance >= 80.0 {
                println!("\n⚠️  UV-48 VALIDATION: PARTIALLY PASSED");
                println!("🔧 Some optimizations needed before UV-12");
            } else {
                println!("\n❌ UV-48 VALIDATION: FAILED");
                println!("🚨 Significant improvements required");
            }
        }
        Err(e) => {
            println!("❌ Validation failed: {}", e);
            std::process::exit(1);
        }
    }
    
    Ok(())
}