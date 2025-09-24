//! # Detector Migration Benchmark
//!
//! Benchmarks comparing legacy detectors vs. new context detectors.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;

use uveddi::analysis::detector_factory::DetectorFactory;
use uveddi::analysis::detectors::anti_patterns::god_object::GodObjectDetector;
use uveddi::ast::{
    compatibility_shim::{DetectorAdapter, ParsedFileCompat},
    SourceLanguage,
};
use uveddi::engine::analysis::context::{FileInfo, ProjectContext};
use uveddi::engine::analysis::{AnalysisContext, AnalysisPipeline, ContextDetectorFactory};
use uveddi::engine::parsing::{AstBuilder, Relation, RelationKind, Symbol, SymbolKind};

fn create_sample_rust_code() -> String {
    r#"
struct LargeStruct {
    field1: i32,
    field2: String,
    field3: Vec<i32>,
    field4: HashMap<String, i32>,
    field5: Option<bool>,
    field6: Result<i32, String>,
    field7: Box<dyn Display>,
    field8: Arc<Mutex<i32>>,
    field9: RefCell<String>,
    field10: Rc<Vec<i32>>,
}

impl LargeStruct {
    pub fn new() -> Self { todo!() }
    pub fn method1(&self) -> i32 { todo!() }
    pub fn method2(&mut self, x: i32) { todo!() }
    pub fn method3(&self) -> String { todo!() }
    pub fn method4(&self, s: &str) -> bool { todo!() }
    pub fn method5(&mut self) { todo!() }
    pub fn method6(&self) -> Vec<i32> { todo!() }
    pub fn method7(&self, v: Vec<i32>) { todo!() }
    pub fn method8(&mut self, x: i32, y: String) { todo!() }
    pub fn method9(&self) -> Option<i32> { todo!() }
    pub fn method10(&self) -> Result<String, String> { todo!() }
}

fn unused_function() {
    println!("This function is never called");
}

fn another_unused_function(x: i32) -> i32 {
    x * 2
}

fn used_function() -> i32 {
    42
}

fn duplicate_logic_1() {
    let mut sum = 0;
    for i in 0..10 {
        sum += i * 2;
        println!("Processing {}", i);
    }
    println!("Result: {}", sum);
}

fn duplicate_logic_2() {
    let mut sum = 0;
    for i in 0..10 {
        sum += i * 2;
        println!("Processing {}", i);
    }
    println!("Result: {}", sum);
}

fn main() {
    let result = used_function();
    println!("Result: {}", result);
}
"#
    .to_string()
}

fn create_analysis_context() -> AnalysisContext {
    let source = create_sample_rust_code();

    let symbols = vec![
        Symbol {
            name: "LargeStruct".to_string(),
            kind: SymbolKind::Struct,
            line: 2,
            column: 0,
            end_line: 13,
            end_column: 1,
            parent: None,
        },
        // Add 10+ methods
        Symbol {
            name: "new".to_string(),
            kind: SymbolKind::Method,
            line: 16,
            column: 4,
            end_line: 16,
            end_column: 30,
            parent: Some("LargeStruct".to_string()),
        },
        Symbol {
            name: "method1".to_string(),
            kind: SymbolKind::Method,
            line: 17,
            column: 4,
            end_line: 17,
            end_column: 35,
            parent: Some("LargeStruct".to_string()),
        },
        Symbol {
            name: "method2".to_string(),
            kind: SymbolKind::Method,
            line: 18,
            column: 4,
            end_line: 18,
            end_column: 40,
            parent: Some("LargeStruct".to_string()),
        },
        Symbol {
            name: "method3".to_string(),
            kind: SymbolKind::Method,
            line: 19,
            column: 4,
            end_line: 19,
            end_column: 38,
            parent: Some("LargeStruct".to_string()),
        },
        Symbol {
            name: "method4".to_string(),
            kind: SymbolKind::Method,
            line: 20,
            column: 4,
            end_line: 20,
            end_column: 45,
            parent: Some("LargeStruct".to_string()),
        },
        Symbol {
            name: "method5".to_string(),
            kind: SymbolKind::Method,
            line: 21,
            column: 4,
            end_line: 21,
            end_column: 28,
            parent: Some("LargeStruct".to_string()),
        },
        Symbol {
            name: "method6".to_string(),
            kind: SymbolKind::Method,
            line: 22,
            column: 4,
            end_line: 22,
            end_column: 35,
            parent: Some("LargeStruct".to_string()),
        },
        Symbol {
            name: "unused_function".to_string(),
            kind: SymbolKind::Function,
            line: 27,
            column: 0,
            end_line: 29,
            end_column: 1,
            parent: None,
        },
        Symbol {
            name: "used_function".to_string(),
            kind: SymbolKind::Function,
            line: 35,
            column: 0,
            end_line: 37,
            end_column: 1,
            parent: None,
        },
        Symbol {
            name: "main".to_string(),
            kind: SymbolKind::Function,
            line: 55,
            column: 0,
            end_line: 58,
            end_column: 1,
            parent: None,
        },
    ];

    let relations = vec![Relation {
        from: "main".to_string(),
        to: "used_function".to_string(),
        kind: RelationKind::Calls,
    }];

    let file_info = FileInfo {
        path: PathBuf::from("benchmark_test.rs"),
        language: SourceLanguage::Rust,
        lines_of_code: source.lines().count(),
        size_bytes: source.len(),
        modified_at: SystemTime::now(),
    };

    let project_context = ProjectContext {
        project_root: PathBuf::from("/tmp/benchmark"),
        project_files: vec![],
        dependencies: vec![],
        global_symbols: vec![],
    };

    AnalysisContext::new(
        file_info,
        None, // No syntax tree for benchmark
        source,
        symbols,
        relations,
        project_context,
    )
}

fn create_parsed_file_compat() -> ParsedFileCompat {
    let source = create_sample_rust_code();
    ParsedFileCompat::new(
        PathBuf::from("benchmark_test.rs"),
        SourceLanguage::Rust,
        source,
    )
}

fn benchmark_context_detectors(c: &mut Criterion) {
    let mut group = c.benchmark_group("detector_migration");

    // Create test data
    let context = create_analysis_context();
    let parsed_file = create_parsed_file_compat();

    // Setup pipeline and factory
    let ast_builder = Arc::new(AstBuilder::new().expect("Failed to create AstBuilder"));
    let pipeline =
        Arc::new(AnalysisPipeline::new(ast_builder.clone()).with_performance_instrumentation(true));
    let factory = ContextDetectorFactory::new(pipeline.clone());

    // Get context detectors
    let context_detectors = factory.create_all_context_detectors();

    // Get legacy detectors
    let legacy_detectors = DetectorFactory::create_default_detectors();

    // Benchmark context detectors
    group.bench_with_input(
        BenchmarkId::new("context_detectors", "all"),
        &context,
        |b, ctx| {
            b.iter(|| {
                for detector in &context_detectors {
                    if detector.supports_language(&ctx.file_info.language) {
                        let _ = black_box(detector.detect(ctx));
                    }
                }
            })
        },
    );

    // Benchmark legacy detectors
    group.bench_with_input(
        BenchmarkId::new("legacy_detectors", "all"),
        &parsed_file,
        |b, pf| {
            b.iter(|| {
                for detector in &legacy_detectors {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    let _ = black_box(rt.block_on(async { detector.detect_issues(pf).await }));
                }
            })
        },
    );

    // Benchmark individual detector comparison - God Object
    let god_detector_context = factory
        .create_context_detector("context_god_object")
        .unwrap();
    let god_detector_legacy = GodObjectDetector::default();

    group.bench_with_input(
        BenchmarkId::new("god_object_context", "single"),
        &context,
        |b, ctx| {
            b.iter(|| {
                let _ = black_box(god_detector_context.detect(ctx));
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("god_object_legacy", "single"),
        &parsed_file,
        |b, pf| {
            b.iter(|| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let _ =
                    black_box(rt.block_on(async { god_detector_legacy.detect_issues(pf).await }));
            })
        },
    );

    // Benchmark pipeline with mixed detectors
    let hybrid_pipeline = pipeline
        .with_detector(
            factory
                .create_context_detector("context_god_object")
                .unwrap(),
        )
        .with_detector(
            factory
                .create_context_detector("context_code_duplication")
                .unwrap(),
        )
        .with_detector(
            factory
                .create_context_detector("context_dead_code")
                .unwrap(),
        );

    group.bench_with_input(
        BenchmarkId::new("hybrid_pipeline", "context"),
        &context,
        |b, ctx| {
            b.iter(|| {
                let _ = black_box(hybrid_pipeline.analyze(ctx.clone()));
            })
        },
    );

    group.finish();
}

fn benchmark_knowledge_graph_integration(c: &mut Criterion) {
    let mut group = c.benchmark_group("knowledge_graph");

    let context = create_analysis_context();

    // Benchmark context detector with knowledge graph queries
    let ast_builder = Arc::new(AstBuilder::new().expect("Failed to create AstBuilder"));
    let pipeline = Arc::new(AnalysisPipeline::new(ast_builder));
    let factory = ContextDetectorFactory::new(pipeline);

    let god_detector = factory
        .create_context_detector("context_god_object")
        .unwrap();

    group.bench_with_input(
        BenchmarkId::new("god_object_with_kg", "single"),
        &context,
        |b, ctx| {
            b.iter(|| {
                let _ = black_box(god_detector.detect(ctx));
            })
        },
    );

    group.finish();
}

criterion_group!(
    benches,
    benchmark_context_detectors,
    benchmark_knowledge_graph_integration
);
criterion_main!(benches);
