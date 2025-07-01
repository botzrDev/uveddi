use criterion::{criterion_group, criterion_main, Criterion};
use std::{fs, io::Write};

/// Generates a dataset of the given name, number of files, and file size (in bytes).
fn generate_dataset(name: &str, num_files: usize, file_size: usize) -> std::io::Result<()> {
    let dir = format!("target/benchmark-data/{}", name);
    fs::create_dir_all(&dir)?;
    for i in 0..num_files {
        let file_path = format!("{}/file_{}.txt", &dir, i);
        let mut file = fs::File::create(&file_path)?;
        let content = vec![b'a'; file_size];
        file.write_all(&content)?;
    }
    Ok(())
}

fn bench_small(c: &mut Criterion) {
    c.bench_function("generate_small_dataset", |b| {
        b.iter(|| {
            generate_dataset("small", 100, 1024 * 100).unwrap(); // ~10MB total
        });
    });
}

fn bench_medium(c: &mut Criterion) {
    c.bench_function("generate_medium_dataset", |b| {
        b.iter(|| {
            generate_dataset("medium", 1000, 1024 * 100).unwrap(); // ~100MB total
        });
    });
}

fn bench_large(c: &mut Criterion) {
    c.bench_function("generate_large_dataset", |b| {
        b.iter(|| {
            generate_dataset("large", 10000, 1024 * 100).unwrap(); // ~1GB total
        });
    });
}

criterion_group!(benches, bench_small, bench_medium, bench_large);
criterion_main!(benches);
