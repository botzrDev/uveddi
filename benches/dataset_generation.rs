use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Represents different types of data structures for benchmarking
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct DatasetEntry {
    id: u64,
    name: String,
    metadata: HashMap<String, String>,
    values: Vec<f64>,
    tags: Vec<String>,
}

impl DatasetEntry {
    fn new(id: u64, complexity: &str) -> Self {
        let mut metadata = HashMap::new();
        let mut values = Vec::new();
        let mut tags = Vec::new();

        match complexity {
            "simple" => {
                metadata.insert("type".to_string(), "simple".to_string());
                values = vec![1.0, 2.0, 3.0];
                tags = vec!["tag1".to_string()];
            }
            "medium" => {
                for i in 0..10 {
                    metadata.insert(format!("key_{}", i), format!("value_{}", i));
                    values.push(i as f64 * 1.5);
                    tags.push(format!("tag_{}", i));
                }
            }
            "complex" => {
                for i in 0..100 {
                    metadata.insert(
                        format!("complex_key_{}", i),
                        format!("complex_value_{}", i * 2),
                    );
                    values.push((i as f64 * 3.14159).sin());
                    tags.push(format!("complex_tag_{}_{}", i, i % 7));
                }
            }
            _ => {}
        }

        Self {
            id,
            name: format!("{}_entry_{}", complexity, id),
            metadata,
            values,
            tags,
        }
    }

    fn serialize_to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    fn process_values(&self) -> Vec<f64> {
        self.values
            .iter()
            .enumerate()
            .map(|(i, &val)| {
                if i % 2 == 0 {
                    val * val
                } else {
                    val.sqrt().abs()
                }
            })
            .collect()
    }

    fn filter_metadata(&self, pattern: &str) -> HashMap<String, String> {
        self.metadata
            .iter()
            .filter(|(k, v)| k.contains(pattern) || v.contains(pattern))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}

/// Dataset generator for benchmark testing
struct DatasetGenerator {
    entries: Vec<DatasetEntry>,
    complexity_distribution: HashMap<String, usize>,
}

impl DatasetGenerator {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
            complexity_distribution: HashMap::new(),
        }
    }

    fn generate_entries(&mut self, count: usize, complexity_mix: &[(&str, f32)]) {
        self.entries.clear();
        self.complexity_distribution.clear();

        let mut current_id = 0;

        for (complexity, ratio) in complexity_mix {
            let entry_count = (count as f32 * ratio) as usize;
            self.complexity_distribution
                .insert(complexity.to_string(), entry_count);

            for _ in 0..entry_count {
                let entry = DatasetEntry::new(current_id, complexity);
                self.entries.push(entry);
                current_id += 1;
            }
        }
    }

    fn process_all_entries(&self) -> Vec<Vec<f64>> {
        self.entries
            .iter()
            .map(|entry| entry.process_values())
            .collect()
    }

    fn filter_entries_by_tag(&self, tag_pattern: &str) -> Vec<&DatasetEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.tags.iter().any(|tag| tag.contains(tag_pattern)))
            .collect()
    }

    fn aggregate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        let total_entries = self.entries.len() as f64;
        stats.insert("total_entries".to_string(), total_entries);

        let total_values: usize = self.entries.iter().map(|e| e.values.len()).sum();
        stats.insert("total_values".to_string(), total_values as f64);

        let avg_values_per_entry = if total_entries > 0.0 {
            total_values as f64 / total_entries
        } else {
            0.0
        };
        stats.insert("avg_values_per_entry".to_string(), avg_values_per_entry);

        let total_metadata: usize = self.entries.iter().map(|e| e.metadata.len()).sum();
        stats.insert("total_metadata_entries".to_string(), total_metadata as f64);

        stats
    }

    fn export_to_json(&self) -> String {
        serde_json::to_string(&self.entries).unwrap_or_default()
    }

    fn save_to_file(&self, path: &Path) -> std::io::Result<()> {
        let json_data = self.export_to_json();
        fs::write(path, json_data)
    }

    fn load_from_file(&mut self, path: &Path) -> std::io::Result<()> {
        let json_data = fs::read_to_string(path)?;
        self.entries = serde_json::from_str(&json_data).unwrap_or_default();
        Ok(())
    }
}

/// Benchmark dataset generation with different sizes
fn bench_dataset_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("dataset_generation");

    let complexity_mix = &[("simple", 0.3), ("medium", 0.5), ("complex", 0.2)];

    for &entry_count in &[100, 500, 1000, 5000] {
        group.throughput(Throughput::Elements(entry_count as u64));
        group.bench_with_input(
            BenchmarkId::new("generate_entries", entry_count),
            &entry_count,
            |b, &entry_count| {
                b.iter(|| {
                    let mut generator = DatasetGenerator::new();
                    generator.generate_entries(entry_count, complexity_mix);
                    black_box(generator)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark data processing operations
fn bench_data_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_processing");

    let complexity_mix = &[("simple", 0.2), ("medium", 0.6), ("complex", 0.2)];

    for &entry_count in &[100, 500, 1000] {
        group.throughput(Throughput::Elements(entry_count as u64));
        group.bench_with_input(
            BenchmarkId::new("process_all_values", entry_count),
            &entry_count,
            |b, &entry_count| {
                let mut generator = DatasetGenerator::new();
                generator.generate_entries(entry_count, complexity_mix);

                b.iter(|| {
                    let processed = generator.process_all_entries();
                    black_box(processed)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark filtering and search operations
fn bench_filtering_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("filtering_operations");

    let complexity_mix = &[("simple", 0.3), ("medium", 0.4), ("complex", 0.3)];

    let mut generator = DatasetGenerator::new();
    generator.generate_entries(1000, complexity_mix);

    group.bench_function("filter_by_tag", |b| {
        b.iter(|| {
            let filtered = generator.filter_entries_by_tag("tag_5");
            black_box(filtered)
        });
    });

    group.bench_function("filter_metadata", |b| {
        b.iter(|| {
            let mut all_filtered = Vec::new();
            for entry in &generator.entries {
                let filtered = entry.filter_metadata("key_");
                all_filtered.push(filtered);
            }
            black_box(all_filtered)
        });
    });

    group.bench_function("aggregate_statistics", |b| {
        b.iter(|| {
            let stats = generator.aggregate_statistics();
            black_box(stats)
        });
    });

    group.finish();
}

/// Benchmark serialization and I/O operations
fn bench_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");

    let complexity_mix = &[("simple", 0.4), ("medium", 0.4), ("complex", 0.2)];

    for &entry_count in &[100, 500, 1000] {
        group.throughput(Throughput::Elements(entry_count as u64));
        group.bench_with_input(
            BenchmarkId::new("json_serialization", entry_count),
            &entry_count,
            |b, &entry_count| {
                let mut generator = DatasetGenerator::new();
                generator.generate_entries(entry_count, complexity_mix);

                b.iter(|| {
                    let json = generator.export_to_json();
                    black_box(json)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark file I/O operations
fn bench_file_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_operations");

    let complexity_mix = &[("simple", 0.3), ("medium", 0.5), ("complex", 0.2)];

    for &entry_count in &[100, 500, 1000] {
        group.bench_with_input(
            BenchmarkId::new("save_and_load", entry_count),
            &entry_count,
            |b, &entry_count| {
                let mut generator = DatasetGenerator::new();
                generator.generate_entries(entry_count, complexity_mix);

                b.iter_custom(|iters| {
                    let start = std::time::Instant::now();

                    for _ in 0..iters {
                        let temp_dir = TempDir::new().unwrap();
                        let file_path = temp_dir.path().join("dataset.json");

                        // Save
                        generator.save_to_file(&file_path).unwrap();

                        // Load
                        let mut new_generator = DatasetGenerator::new();
                        new_generator.load_from_file(&file_path).unwrap();

                        black_box(new_generator);
                    }

                    start.elapsed()
                });
            },
        );
    }

    group.finish();
}

/// Benchmark memory usage patterns
fn bench_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_patterns");

    group.bench_function("large_dataset_memory", |b| {
        b.iter_custom(|iters| {
            let start = std::time::Instant::now();

            for _ in 0..iters {
                let mut generator = DatasetGenerator::new();

                // Generate a large dataset
                let complexity_mix = &[("simple", 0.2), ("medium", 0.3), ("complex", 0.5)];

                generator.generate_entries(10000, complexity_mix);

                // Perform some operations
                let _ = generator.process_all_entries();
                let _ = generator.aggregate_statistics();
                let _ = generator.filter_entries_by_tag("complex");

                black_box(generator);
            }

            start.elapsed()
        });
    });

    group.finish();
}

/// Benchmark concurrent dataset operations
fn bench_concurrent_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_operations");

    for &thread_count in &[1, 2, 4, 8] {
        group.bench_with_input(
            BenchmarkId::new("concurrent_generation", thread_count),
            &thread_count,
            |b, &thread_count| {
                let complexity_mix = vec![("simple", 0.3), ("medium", 0.4), ("complex", 0.3)];

                b.iter(|| {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let mut handles = Vec::new();

                        for _ in 0..thread_count {
                            let mix = complexity_mix.clone();
                            let handle = tokio::spawn(async move {
                                let mut generator = DatasetGenerator::new();
                                generator.generate_entries(500, &mix);
                                let processed = generator.process_all_entries();
                                black_box(processed)
                            });
                            handles.push(handle);
                        }

                        for handle in handles {
                            let _ = handle.await;
                        }
                    })
                });
            },
        );
    }

    group.finish();
}

/// Benchmark data transformation pipelines
fn bench_transformation_pipelines(c: &mut Criterion) {
    let mut group = c.benchmark_group("transformation_pipelines");

    let mut generator = DatasetGenerator::new();
    generator.generate_entries(1000, &[("simple", 0.2), ("medium", 0.5), ("complex", 0.3)]);

    group.bench_function("full_pipeline", |b| {
        b.iter(|| {
            // Step 1: Filter entries
            let filtered = generator.filter_entries_by_tag("tag");

            // Step 2: Process values for filtered entries
            let processed: Vec<Vec<f64>> = filtered
                .iter()
                .map(|entry| entry.process_values())
                .collect();

            // Step 3: Aggregate results
            let total_values: usize = processed.iter().map(|v| v.len()).sum();
            let avg_per_entry = if !processed.is_empty() {
                total_values as f64 / processed.len() as f64
            } else {
                0.0
            };

            // Step 4: Serialize results
            let mut result = HashMap::new();
            result.insert("total_values".to_string(), total_values as f64);
            result.insert("avg_per_entry".to_string(), avg_per_entry);

            let json_result = serde_json::to_string(&result).unwrap_or_default();

            black_box(json_result)
        });
    });

    group.finish();
}

// Serde support is now built into DatasetEntry via derive macros

criterion_group!(
    dataset_generation_benches,
    bench_dataset_generation,
    bench_data_processing,
    bench_filtering_operations,
    bench_serialization,
    bench_file_operations,
    bench_memory_patterns,
    bench_concurrent_operations,
    bench_transformation_pipelines
);

criterion_main!(dataset_generation_benches);
