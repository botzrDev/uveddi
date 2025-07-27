//! Performance benchmarks for the AI Knowledge Library (UV-335)
//! 
//! This benchmark suite validates the performance requirements:
//! - <100ms knowledge retrieval
//! - O(1) lookup performance
//! - Memory efficiency targets

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::collections::HashMap;
use uveddi::ai::knowledge::*;

fn create_benchmark_library(pattern_count: usize) -> KnowledgeLibrary {
    let mut library = KnowledgeLibrary::new();

    for i in 0..pattern_count {
        let pattern = PatternKnowledge {
            id: format!("benchmark_pattern_{}", i),
            name: format!("Benchmark Pattern {}", i),
            definition: CompressedString::new(&format!(
                "This is a benchmark anti-pattern number {} used for performance testing. \
                It contains detailed information about the pattern including symptoms, \
                solutions, and examples. The content is designed to simulate real-world \
                knowledge base entries with appropriate compression characteristics.",
                i
            )),
            symptoms: vec![
                CompressedString::new(&format!("Primary symptom {} indicating the presence of this pattern", i)),
                CompressedString::new(&format!("Secondary symptom {} with additional context", i)),
                CompressedString::new(&format!("Tertiary symptom {} for comprehensive detection", i)),
            ],
            impact: match i % 4 {
                0 => ImpactLevel::Critical,
                1 => ImpactLevel::High,
                2 => ImpactLevel::Medium,
                _ => ImpactLevel::Low,
            },
            category: match i % 10 {
                0..=2 => AntiPatternCategory::ObjectOriented,
                3..=4 => AntiPatternCategory::Architectural,
                5..=6 => AntiPatternCategory::Performance,
                7 => AntiPatternCategory::Security,
                8 => AntiPatternCategory::Memory,
                _ => AntiPatternCategory::Maintainability,
            },
            detection_methods: vec![
                DetectionMethod::MetricThreshold {
                    metric_name: format!("metric_{}", i),
                    threshold: (i as f64) * 10.0,
                    operator: ComparisonOperator::GreaterThan,
                },
                DetectionMethod::RegexPattern {
                    pattern: format!(r"\bpattern_{}\b", i),
                    context: "code_analysis".to_string(),
                },
            ],
            solutions: vec![
                SolutionPattern {
                    id: format!("solution_{}", i),
                    title: format!("Solution for Pattern {}", i),
                    implementation: CompressedString::new(&format!(
                        "Detailed implementation guide for resolving pattern {}. \
                        This includes step-by-step instructions, code examples, \
                        and best practices for effective remediation.",
                        i
                    )),
                    examples: vec![],
                    effort_level: match i % 5 {
                        0 => EffortLevel::Trivial,
                        1 => EffortLevel::Low,
                        2 => EffortLevel::Medium,
                        3 => EffortLevel::High,
                        _ => EffortLevel::Significant,
                    },
                    prerequisites: vec![
                        format!("Understanding of pattern {}", i),
                        format!("Knowledge of domain {}", i % 10),
                    ],
                    expected_impact: ImpactLevel::High,
                },
            ],
            examples: CodeExamples {
                primary: vec![
                    CodeExample {
                        language: SourceLanguage::Universal,
                        problem_code: CompressedString::new(&format!(
                            "// Problem code example for pattern {}\n\
                            function problematicFunction{}() {{\n    \
                                // This demonstrates the anti-pattern\n    \
                                let result = doSomething{}();\n    \
                                return result;\n\
                            }}",
                            i, i, i
                        )),
                        solution_code: CompressedString::new(&format!(
                            "// Solution code example for pattern {}\n\
                            function improvedFunction{}() {{\n    \
                                // This demonstrates the correct approach\n    \
                                let result = doSomethingBetter{}();\n    \
                                return result;\n\
                            }}",
                            i, i, i
                        )),
                        explanation: CompressedString::new(&format!(
                            "The solution improves upon the original by applying \
                            best practices specific to pattern {}. This reduces \
                            technical debt and improves maintainability.",
                            i
                        )),
                        file_context: Some(format!("example_{}.js", i)),
                    },
                ],
                variations: HashMap::new(),
            },
            language_variations: HashMap::new(),
            related_patterns: vec![
                format!("related_pattern_{}", (i + 1) % pattern_count),
                format!("related_pattern_{}", (i + 2) % pattern_count),
            ],
            tags: vec![
                format!("category_{}", i % 10),
                format!("severity_{}", i % 4),
                format!("domain_{}", i % 5),
                "benchmark".to_string(),
                "performance_test".to_string(),
            ],
            frequency_score: (i as f32) / (pattern_count as f32),
            detection_confidence: 0.8 + ((i % 20) as f32) / 100.0,
        };

        library.add_universal_pattern(pattern);
    }

    library
}

fn create_benchmark_lookup(pattern_count: usize) -> KnowledgeLibraryLookup {
    let library = create_benchmark_library(pattern_count);
    let mut builder = IndexBuilder::default();
    let index = builder.build_from_library(&library).unwrap();
    KnowledgeLibraryLookup::new(library, index)
}

fn benchmark_pattern_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_lookup");
    
    for pattern_count in [100, 500, 1000, 5000].iter() {
        let lookup = create_benchmark_lookup(*pattern_count);
        
        group.bench_with_input(
            BenchmarkId::new("direct_lookup", pattern_count),
            pattern_count,
            |b, &pattern_count| {
                b.iter(|| {
                    for i in 0..10 {
                        let pattern_id = format!("benchmark_pattern_{}", i % pattern_count);
                        black_box(lookup.get_pattern(&pattern_id));
                    }
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_symptom_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("symptom_search");
    
    for pattern_count in [100, 500, 1000].iter() {
        let lookup = create_benchmark_lookup(*pattern_count);
        
        group.bench_with_input(
            BenchmarkId::new("symptom_search", pattern_count),
            pattern_count,
            |b, _| {
                b.iter(|| {
                    let symptoms = vec![
                        "primary symptom".to_string(),
                        "secondary symptom".to_string(),
                    ];
                    black_box(lookup.search_by_symptoms(&symptoms));
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_category_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("category_lookup");
    
    for pattern_count in [100, 500, 1000, 5000].iter() {
        let lookup = create_benchmark_lookup(*pattern_count);
        
        group.bench_with_input(
            BenchmarkId::new("category_lookup", pattern_count),
            pattern_count,
            |b, _| {
                b.iter(|| {
                    black_box(lookup.get_patterns_by_category(AntiPatternCategory::ObjectOriented));
                    black_box(lookup.get_patterns_by_category(AntiPatternCategory::Architectural));
                    black_box(lookup.get_patterns_by_category(AntiPatternCategory::Performance));
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_tag_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("tag_search");
    
    for pattern_count in [100, 500, 1000].iter() {
        let lookup = create_benchmark_lookup(*pattern_count);
        
        group.bench_with_input(
            BenchmarkId::new("tag_search", pattern_count),
            pattern_count,
            |b, _| {
                b.iter(|| {
                    let tags = vec!["benchmark".to_string(), "performance_test".to_string()];
                    black_box(lookup.search_by_tags(&tags));
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_language_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("language_patterns");
    
    for pattern_count in [100, 500, 1000, 5000].iter() {
        let lookup = create_benchmark_lookup(*pattern_count);
        
        group.bench_with_input(
            BenchmarkId::new("language_patterns", pattern_count),
            pattern_count,
            |b, _| {
                b.iter(|| {
                    black_box(lookup.get_language_patterns(SourceLanguage::Universal));
                    black_box(lookup.get_language_patterns(SourceLanguage::Rust));
                    black_box(lookup.get_language_patterns(SourceLanguage::Python));
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_index_building(c: &mut Criterion) {
    let mut group = c.benchmark_group("index_building");
    
    for pattern_count in [100, 500, 1000].iter() {
        let library = create_benchmark_library(*pattern_count);
        
        group.bench_with_input(
            BenchmarkId::new("build_all_indices", pattern_count),
            pattern_count,
            |b, _| {
                b.iter(|| {
                    let mut builder = IndexBuilder::default();
                    black_box(builder.build_from_library(&library).unwrap());
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_compression_training(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression_training");
    
    for sample_count in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("dictionary_training", sample_count),
            sample_count,
            |b, &sample_count| {
                b.iter(|| {
                    let mut trainer = DictionaryTrainer::new();
                    
                    // Add programming samples
                    for i in 0..sample_count {
                        trainer.add_sample(format!(
                            "fn process_data_{}() {{ \
                                let result = calculate_metrics_{}(); \
                                if result.is_valid() {{ \
                                    return Ok(result.value); \
                                }} else {{ \
                                    return Err(ProcessingError::InvalidData); \
                                }} \
                            }}",
                            i, i
                        ));
                    }
                    
                    black_box(trainer.analyze_training_data());
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");
    
    for pattern_count in [100, 500, 1000].iter() {
        let library = create_benchmark_library(*pattern_count);
        
        group.bench_with_input(
            BenchmarkId::new("json_serialization", pattern_count),
            pattern_count,
            |b, _| {
                b.iter(|| {
                    black_box(serde_json::to_string(&library).unwrap());
                });
            },
        );
        
        let serialized = serde_json::to_string(&library).unwrap();
        group.bench_with_input(
            BenchmarkId::new("json_deserialization", pattern_count),
            pattern_count,
            |b, _| {
                b.iter(|| {
                    black_box(serde_json::from_str::<KnowledgeLibrary>(&serialized).unwrap());
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    for pattern_count in [100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("library_creation", pattern_count),
            pattern_count,
            |b, &pattern_count| {
                b.iter(|| {
                    black_box(create_benchmark_library(pattern_count));
                });
            },
        );
    }
    
    group.finish();
}

// Performance target validation benchmarks
fn benchmark_retrieval_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("retrieval_latency");
    group.measurement_time(std::time::Duration::from_secs(30));
    
    // Target: <100ms for knowledge retrieval (UV-330 requirement)
    let lookup = create_benchmark_lookup(10000); // Large library for stress testing
    
    group.bench_function("100ms_target_validation", |b| {
        b.iter(|| {
            // Simulate comprehensive knowledge retrieval
            let start = std::time::Instant::now();
            
            // Pattern lookup
            black_box(lookup.get_pattern("benchmark_pattern_42"));
            
            // Category search
            black_box(lookup.get_patterns_by_category(AntiPatternCategory::Performance));
            
            // Symptom search
            let symptoms = vec!["performance".to_string(), "slow".to_string()];
            black_box(lookup.search_by_symptoms(&symptoms));
            
            // Tag search
            let tags = vec!["benchmark".to_string()];
            black_box(lookup.search_by_tags(&tags));
            
            // Language patterns
            black_box(lookup.get_language_patterns(SourceLanguage::Rust));
            
            let duration = start.elapsed();
            
            // Assert that total retrieval is under 100ms
            assert!(
                duration.as_millis() < 100,
                "Knowledge retrieval took {}ms, exceeding 100ms target",
                duration.as_millis()
            );
            
            duration
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_pattern_lookup,
    benchmark_symptom_search,
    benchmark_category_lookup,
    benchmark_tag_search,
    benchmark_language_patterns,
    benchmark_index_building,
    benchmark_compression_training,
    benchmark_serialization,
    benchmark_memory_usage,
    benchmark_retrieval_latency
);

criterion_main!(benches);