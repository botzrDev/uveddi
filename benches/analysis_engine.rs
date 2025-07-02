use criterion::{criterion_group, criterion_main, Criterion};
// Update the path below to the correct location of AnalysisEngine in your uveddi crate.
// For example, if AnalysisEngine is defined in uveddi::analysis::engine:
use uveddi::analysis::engine::AnalysisEngine;
use std::path::Path;
use tokio::runtime::Runtime;

fn bench_analysis_engine(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let sample_path = Path::new("target/benchmark-data/small");

    let mut group = c.benchmark_group("analysis_engine");

    group.bench_function("small_dataset", |b| {
        b.to_async(&rt).iter(|| async {
            let mut engine = AnalysisEngine::new().unwrap();
            let _ = engine.analyze(sample_path).await;
        });
    });

    group.finish();
}

criterion_group!(benches, bench_analysis_engine);
criterion_main!(benches);
