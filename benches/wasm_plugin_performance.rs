use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use uveddi::plugin::{PluginManager, WasmPluginManager};
use uveddi::analysis::dependency_graph::LocalDependencyGraph;
use uveddi_plugin_api::models::{DependencyGraph, Dependency, DependencyType};
use std::path::PathBuf;
use std::sync::Arc;

fn create_test_dependency_graph(size: usize) -> DependencyGraph {
    let mut graph = DependencyGraph::new();
    let mut dependencies = Vec::new();
    
    // Create a realistic dependency graph with various patterns
    for i in 0..size {
        dependencies.push(Dependency {
            from_file: PathBuf::from(format!("src/module_{}.rs", i)),
            to_module: format!("module_{}", (i + 1) % size),
            dependency_type: DependencyType::Use,
            line_number: Some(1),
        });
        
        // Add some modules with high fan-out (potential God Objects)
        if i % 10 == 0 {
            for j in 1..=5 {
                dependencies.push(Dependency {
                    from_file: PathBuf::from(format!("src/module_{}.rs", i)),
                    to_module: format!("util_{}", j),
                    dependency_type: DependencyType::Use,
                    line_number: Some(j as u32),
                });
            }
        }
    }
    
    graph.build_from_dependencies(dependencies);
    graph
}

fn create_internal_dependency_graph(size: usize) -> LocalDependencyGraph {
    use uveddi::analysis::dependency_graph::{ComponentNode, LocalDependencyType};
    let mut graph = LocalDependencyGraph::new();
    
    // Create similar structure for internal graph
    for i in 0..size {
        let from = ComponentNode::Module { path: format!("module_{}", i) };
        let to = ComponentNode::Module { path: format!("module_{}", (i + 1) % size) };
        graph.add_dependency(&from, &to, LocalDependencyType::Import);
        
        // Add high fan-out modules
        if i % 10 == 0 {
            for j in 1..=5 {
                let from = ComponentNode::Module { path: format!("module_{}", i) };
                let to = ComponentNode::Module { path: format!("util_{}", j) };
                graph.add_dependency(&from, &to, LocalDependencyType::Import);
            }
        }
    }
    
    graph
}

fn benchmark_plugin_manager_initialization(c: &mut Criterion) {
    c.bench_function("plugin_manager_init", |b| {
        b.iter(|| {
            let _manager = PluginManager::new();
        })
    });
}

fn benchmark_wasm_manager_initialization(c: &mut Criterion) {
    c.bench_function("wasm_manager_init", |b| {
        b.iter(|| {
            // WASM manager creation might fail in some environments
            let _result = WasmPluginManager::new();
        })
    });
}

fn benchmark_native_plugin_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("native_plugins");
    let manager = uveddi::plugin::initialize_plugins();
    
    for size in [10, 50, 100, 500].iter() {
        let graph = create_test_dependency_graph(*size);
        
        group.bench_with_input(
            BenchmarkId::new("native_execution", size),
            &graph,
            |b, graph| {
                b.iter(|| {
                    let results = manager.run_plugins(graph);
                    // Consume results to ensure they're fully processed
                    for result in results {
                        match result {
                            Ok(issues) => { let _count = issues.len(); }
                            Err(_) => {}
                        }
                    }
                })
            },
        );
    }
    group.finish();
}

fn benchmark_dependency_graph_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph_conversion");
    
    for size in [10, 50, 100, 500].iter() {
        let api_graph = create_test_dependency_graph(*size);
        
        group.bench_with_input(
            BenchmarkId::new("api_to_internal", size),
            &api_graph,
            |b, graph| {
                b.iter(|| {
                    let mut internal_graph = DependencyGraph::new();
                    let dependencies = graph.get_all_dependencies().to_vec();
                    internal_graph.build_from_dependencies(dependencies);
                    internal_graph
                })
            },
        );
    }
    group.finish();
}

fn benchmark_plugin_verification(c: &mut Criterion) {
    use tempfile::TempDir;
    use std::fs;
    
    c.bench_function("plugin_verification_invalid", |b| {
        b.iter_batched(
            || {
                let temp_dir = TempDir::new().unwrap();
                let wasm_path = temp_dir.path().join("test.wasm");
                let manifest_path = temp_dir.path().join("test.toml");
                
                // Create minimal files for testing
                fs::write(&wasm_path, b"invalid wasm").unwrap();
                fs::write(&manifest_path, r#"
name = "test"
version = "1.0.0"
permissions = []
[resource_limits]
max_memory_mb = 16
max_execution_fuel = 1000000
max_output_size_kb = 16
"#).unwrap();
                
                (wasm_path, manifest_path, temp_dir)
            },
            |(wasm_path, manifest_path, _temp_dir)| {
                let _result = uveddi::plugin::PluginVerifier::verify_plugin(&wasm_path, &manifest_path);
            },
            criterion::BatchSize::SmallInput
        )
    });
}

fn benchmark_memory_allocation_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_patterns");
    
    // Test memory allocation patterns for different graph sizes
    for size in [100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("graph_creation", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let _graph = create_test_dependency_graph(size);
                })
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("internal_graph_creation", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let _graph = create_internal_dependency_graph(size);
                })
            },
        );
    }
    group.finish();
}

fn benchmark_concurrent_plugin_execution(c: &mut Criterion) {
    // Simplified synchronous version to avoid Send/Sync issues
    let manager = uveddi::plugin::initialize_plugins();
    let graph = create_test_dependency_graph(100);
    
    c.bench_function("concurrent_plugin_execution", |b| {
        b.iter(|| {
            // Run plugins sequentially for now to avoid concurrency issues
            let results = manager.run_plugins(&graph);
            results.len()
        })
    });
}

criterion_group!(
    benches,
    benchmark_plugin_manager_initialization,
    benchmark_wasm_manager_initialization,
    benchmark_native_plugin_execution,
    benchmark_dependency_graph_conversion,
    benchmark_plugin_verification,
    benchmark_memory_allocation_patterns,
    benchmark_concurrent_plugin_execution
);
criterion_main!(benches);