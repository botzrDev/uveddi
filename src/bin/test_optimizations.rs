use uveddi::analysis::performance::{RenderingOptimizer, OptimizationRequest, RenderQuality};

#[cfg(feature = "image-rendering")]
use uveddi::report::image_renderer::ImageFormat;

#[cfg(not(feature = "image-rendering"))]
use uveddi::analysis::performance::image_stubs::ImageFormat;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Testing UV-47 Performance Optimizations");
    
    let optimizer = RenderingOptimizer::new();
    
    // Test different quality levels
    let test_diagram = r#"
graph TD
    A[Start] --> B[Process]
    B --> C{Decision}
    C -->|Yes| D[Action 1]
    C -->|No| E[Action 2]
    D --> F[End]
    E --> F
"#;

    let qualities = vec![
        RenderQuality::Fast,
        RenderQuality::Balanced,
        RenderQuality::High,
    ];

    for quality in qualities {
        println!("\n📊 Testing {:?} quality rendering...", quality);
        
        let request = OptimizationRequest {
            mermaid_code: test_diagram.to_string(),
            format: ImageFormat::Svg,
            width: Some(800),
            height: Some(600),
            quality: quality.clone(),
            cache_key: None,
        };

        // Test first render (cache miss)
        let start_time = std::time::Instant::now();
        match optimizer.optimize_rendering(request.clone()).await {
            Ok(result) => {
                let total_time = start_time.elapsed().as_millis();
                println!("  ✅ First render: {}ms (cache hit: {}, render time: {}ms)", 
                    total_time, result.cache_hit, result.render_time_ms);
            }
            Err(e) => {
                println!("  ❌ First render failed: {}", e);
                continue;
            }
        }

        // Test second render (should be cache hit)
        let start_time = std::time::Instant::now();
        match optimizer.optimize_rendering(request).await {
            Ok(result) => {
                let total_time = start_time.elapsed().as_millis();
                println!("  ✅ Second render: {}ms (cache hit: {}, render time: {}ms)", 
                    total_time, result.cache_hit, result.render_time_ms);
            }
            Err(e) => {
                println!("  ❌ Second render failed: {}", e);
            }
        }
    }

    // Test concurrent rendering
    println!("\n⚡ Testing concurrent rendering...");
    let concurrent_requests = 10;
    let mut handles = Vec::new();

    for i in 0..concurrent_requests {
        let optimizer = optimizer.clone();
        let diagram = format!(r#"
graph TD
    A{} --> B{}
    B{} --> C{}
"#, i, i, i, i);

        let handle = tokio::spawn(async move {
            let request = OptimizationRequest {
                mermaid_code: diagram,
                format: ImageFormat::Svg,
                width: Some(800),
                height: Some(600),
                quality: RenderQuality::Balanced,
                cache_key: None,
            };

            let start_time = std::time::Instant::now();
            match optimizer.optimize_rendering(request).await {
                Ok(result) => {
                    let total_time = start_time.elapsed().as_millis();
                    (i, total_time, result.render_time_ms, true)
                }
                Err(_) => (i, 0, 0, false)
            }
        });

        handles.push(handle);
    }

    let results = futures::future::join_all(handles).await;
    let mut successful = 0;
    let mut total_time = 0u128;
    let mut render_times = Vec::new();

    for result in results {
        if let Ok((i, total, render_time, success)) = result {
            if success {
                successful += 1;
                total_time += total;
                render_times.push(render_time);
                println!("  Request {}: {}ms total, {}ms render", i, total, render_time);
            }
        }
    }

    if successful > 0 {
        let avg_total = total_time / successful as u128;
        let avg_render = render_times.iter().sum::<u64>() / render_times.len() as u64;
        render_times.sort();
        let p95_render = render_times.get((render_times.len() as f64 * 0.95) as usize).unwrap_or(&0);
        
        println!("\n📈 Concurrent rendering results:");
        println!("  ✅ Successful requests: {}/{}", successful, concurrent_requests);
        println!("  📊 Average total time: {}ms", avg_total);
        println!("  📊 Average render time: {}ms", avg_render);
        println!("  📊 P95 render time: {}ms", p95_render);
    }

    // Get performance stats
    let stats = optimizer.get_performance_stats().await;
    println!("\n📊 Final Performance Stats:");
    println!("  Cache hit rate: {:.1}%", stats.cache_hit_rate * 100.0);
    println!("  Total requests: {}", stats.total_requests);
    println!("  Circuit breaker failures: {}", stats.circuit_breaker_failures);
    println!("  Circuit breaker state: {:?}", stats.circuit_breaker_state);

    // Check if we're meeting targets
    let target_met = successful == concurrent_requests && 
                     render_times.iter().all(|&t| t < 50);
    
    if target_met {
        println!("\n✅ UV-47 Performance Optimizations: SUCCESS");
        println!("   All renders completed under 50ms target");
    } else {
        println!("\n⚠️  UV-47 Performance Optimizations: NEEDS IMPROVEMENT");
        println!("   Some renders exceeded 50ms target");
    }

    Ok(())
}