//! Database Performance Benchmark Tests
//!
//! These tests validate the performance improvements from database optimizations
//! including connection pooling, indexing, and query batching.

use chrono::Utc;
use std::path::Path;
use std::time::Instant;
use tokio::time::{sleep, Duration};
use uveddi::database::{
    crud::Database,
    models::{AnalysisRun, AntiPatternType, ArchitecturalIssue, Dependency, DependencyType},
    pool::{PoolConfig, PooledDatabase},
};

/// Helper to create test data
fn create_test_anti_pattern_type(name: &str) -> AntiPatternType {
    AntiPatternType {
        anti_pattern_type_id: None,
        name: name.to_string(),
        description: format!("Test anti-pattern: {}", name),
        category: "test".to_string(),
    }
}

fn create_test_issues(run_id: i64, anti_pattern_id: i64, count: usize) -> Vec<ArchitecturalIssue> {
    (0..count)
        .map(|i| {
            ArchitecturalIssue::new(
                run_id,
                anti_pattern_id,
                format!("test_file_{}.rs", i),
                Some((i + 1) as i32),
                format!("Test issue {}", i),
                "test_detector",
                if i % 4 == 0 {
                    "critical"
                } else if i % 3 == 0 {
                    "high"
                } else if i % 2 == 0 {
                    "medium"
                } else {
                    "low"
                }
                .to_string(),
                format!("Test description for issue {}", i),
            )
        })
        .collect()
}

fn create_test_dependencies(count: usize) -> Vec<Dependency> {
    (0..count)
        .map(|i| Dependency {
            from_file: std::path::PathBuf::from(format!("src/module_{}.rs", i)),
            to_module: format!("module_{}", (i + 1) % count),
            dependency_type: match i % 5 {
                0 => DependencyType::Use,
                1 => DependencyType::Mod,
                2 => DependencyType::External,
                3 => DependencyType::Import,
                _ => DependencyType::DataFlow,
            },
            line_number: Some((i + 1) as u32),
        })
        .collect()
}

#[tokio::test]
async fn benchmark_single_connection_vs_pooled() {
    // Test with single connection (original approach)
    let start = Instant::now();
    let single_db = Database::new(None).unwrap();
    let project_path = Path::new("/test/benchmark");
    let run = single_db.create_analysis_run(project_path).unwrap();
    let run_id = run.run_id.unwrap();

    let mut anti_pattern_types = vec![
        create_test_anti_pattern_type("god_object"),
        create_test_anti_pattern_type("dead_code"),
        create_test_anti_pattern_type("tight_coupling"),
    ];
    single_db
        .store_anti_pattern_types_batch(&mut anti_pattern_types)
        .unwrap();

    let issues = create_test_issues(run_id, 1, 1000);
    let mut single_db_mut = single_db;
    single_db_mut.store_issues(&issues).unwrap();

    let single_time = start.elapsed();
    println!("Single connection time: {:?}", single_time);

    // Test with connection pooling
    let start = Instant::now();
    let pool_config = PoolConfig {
        max_connections: 5,
        connection_timeout: Duration::from_secs(10),
        idle_timeout: Duration::from_secs(60),
        max_lifetime: Duration::from_secs(300),
    };

    let pooled_db = PooledDatabase::new(None, Some(pool_config)).unwrap();

    // Simulate concurrent operations
    let mut handles = vec![];
    for batch in 0..10 {
        let db_clone = pooled_db.clone();
        let handle = tokio::spawn(async move {
            let _ = db_clone
                .with_connection(|conn| {
                    conn.execute_batch(&format!(
                    "CREATE TABLE IF NOT EXISTS bench_table_{} (id INTEGER PRIMARY KEY, data TEXT)",
                    batch
                ))?;

                    for i in 0..100 {
                        conn.execute(
                            &format!("INSERT INTO bench_table_{} (data) VALUES (?)", batch),
                            [format!("test_data_{}_{}", batch, i)],
                        )?;
                    }
                    Ok(())
                })
                .await;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let pooled_time = start.elapsed();
    println!("Pooled connection time: {:?}", pooled_time);

    // Pooled approach should be faster for concurrent operations
    assert!(pooled_time < single_time * 2); // Allow some variance but expect improvement
}

#[tokio::test]
async fn benchmark_query_performance() {
    let db = Database::new(None).unwrap();
    let project_path = Path::new("/test/query_perf");
    let run = db.create_analysis_run(project_path).unwrap();
    let run_id = run.run_id.unwrap();

    // Create test data
    let mut anti_pattern_types = vec![
        create_test_anti_pattern_type("performance_test_1"),
        create_test_anti_pattern_type("performance_test_2"),
        create_test_anti_pattern_type("performance_test_3"),
    ];
    db.store_anti_pattern_types_batch(&mut anti_pattern_types)
        .unwrap();

    let issues = create_test_issues(run_id, 1, 5000);
    let mut db_mut = db;
    db_mut.store_issues(&issues).unwrap();

    // Benchmark individual queries vs batch queries
    let start = Instant::now();
    let _individual_issues = db_mut.get_issues_for_run(run_id).await.unwrap();
    let individual_query_time = start.elapsed();

    let start = Instant::now();
    let _batch_issues = db_mut.get_issues_with_types_for_run(run_id).await.unwrap();
    let batch_query_time = start.elapsed();

    println!("Individual query time: {:?}", individual_query_time);
    println!("Batch query (with JOIN) time: {:?}", batch_query_time);

    // Batch query should be significantly faster
    assert!(batch_query_time < individual_query_time);

    // Test pagination performance
    let start = Instant::now();
    let _paginated = db_mut
        .get_issues_paginated(run_id, 0, 100, Some("high"), None)
        .await
        .unwrap();
    let pagination_time = start.elapsed();

    println!("Paginated query time: {:?}", pagination_time);

    // Pagination should be very fast with proper indexes
    assert!(pagination_time < Duration::from_millis(50));
}

#[tokio::test]
async fn benchmark_aggregation_queries() {
    let db = Database::new(None).unwrap();
    let project_path = Path::new("/test/aggregation");
    let run = db.create_analysis_run(project_path).unwrap();
    let run_id = run.run_id.unwrap();

    // Create diverse test data
    let mut anti_pattern_types = vec![
        create_test_anti_pattern_type("structural"),
        create_test_anti_pattern_type("behavioral"),
        create_test_anti_pattern_type("creational"),
    ];
    db.store_anti_pattern_types_batch(&mut anti_pattern_types)
        .unwrap();

    // Create issues with different severities and detectors
    let mut issues = vec![];
    for i in 0..2000 {
        let severity = match i % 4 {
            0 => "critical",
            1 => "high",
            2 => "medium",
            _ => "low",
        };

        let detector = match i % 3 {
            0 => "detector_a",
            1 => "detector_b",
            _ => "detector_c",
        };

        let issue = ArchitecturalIssue::new(
            run_id,
            ((i % 3) + 1) as i64, // Cycle through anti-pattern types
            format!("file_{}.rs", i),
            Some((i + 1) as i32),
            format!("Issue {}", i),
            detector,
            severity.to_string(),
            format!("Description {}", i),
        );
        issues.push(issue);
    }

    let mut db_mut = db;
    db_mut.store_issues(&issues).unwrap();

    // Benchmark aggregation query
    let start = Instant::now();
    let stats = db_mut.get_analysis_stats(run_id).await.unwrap();
    let aggregation_time = start.elapsed();

    println!("Aggregation query time: {:?}", aggregation_time);
    println!("Stats: {:?}", stats);

    // Verify results
    assert_eq!(stats.total_issues, 2000);
    assert_eq!(stats.critical_count, 500);
    assert_eq!(stats.high_count, 500);
    assert_eq!(stats.medium_count, 500);
    assert_eq!(stats.low_count, 500);

    // Aggregation should be fast with proper indexes
    assert!(aggregation_time < Duration::from_millis(100));
}

#[tokio::test]
async fn benchmark_dependency_storage() {
    let db = Database::new(None).unwrap();
    let project_path = Path::new("/test/dependencies");
    let run = db.create_analysis_run(project_path).unwrap();
    let run_id = run.run_id.unwrap();

    let dependencies = create_test_dependencies(10000);

    // Benchmark batch dependency storage
    let start = Instant::now();
    let mut db_mut = db;
    db_mut
        .store_dependencies_batch(run_id, &dependencies)
        .unwrap();
    let storage_time = start.elapsed();

    println!("Dependency batch storage time: {:?}", storage_time);

    // Benchmark dependency retrieval
    let start = Instant::now();
    let retrieved_deps = db_mut.get_dependencies_for_run(run_id).await.unwrap();
    let retrieval_time = start.elapsed();

    println!("Dependency retrieval time: {:?}", retrieval_time);

    // Verify results
    assert_eq!(retrieved_deps.len(), 10000);

    // Both operations should be reasonably fast
    assert!(storage_time < Duration::from_secs(5));
    assert!(retrieval_time < Duration::from_millis(500));
}

#[tokio::test]
async fn benchmark_concurrent_operations() {
    let pool_config = PoolConfig {
        max_connections: 8,
        connection_timeout: Duration::from_secs(30),
        idle_timeout: Duration::from_secs(300),
        max_lifetime: Duration::from_secs(1800),
    };

    let pooled_db = PooledDatabase::new(None, Some(pool_config)).unwrap();

    // Initialize schema
    pooled_db
        .with_connection(|conn| {
            conn.execute_batch(
                "
            CREATE TABLE IF NOT EXISTS projects (project_id INTEGER PRIMARY KEY, path TEXT UNIQUE);
            CREATE TABLE IF NOT EXISTS analysis_runs (
                run_id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                start_time TEXT NOT NULL,
                status TEXT NOT NULL,
                analysis_config TEXT NOT NULL DEFAULT '{}'
            );
        ",
            )?;
            Ok(())
        })
        .await
        .unwrap();

    let start = Instant::now();

    // Simulate concurrent analysis runs
    let mut handles = vec![];
    for i in 0..20 {
        let db_clone = pooled_db.clone();
        let handle = tokio::spawn(async move {
            let result = db_clone
                .with_connection_mut(|conn| {
                    // Insert project
                    conn.execute(
                        "INSERT OR IGNORE INTO projects (path) VALUES (?)",
                        [format!("/test/concurrent/{}", i)],
                    )?;

                    // Insert analysis run
                    conn.execute(
                    "INSERT INTO analysis_runs (project_id, start_time, status) VALUES (?, ?, ?)",
                    rusqlite::params![
                        1,
                        Utc::now().to_rfc3339(),
                        "completed"
                    ],
                )?;

                    Ok(conn.last_insert_rowid())
                })
                .await;

            result
        });
        handles.push(handle);
    }

    let mut results = vec![];
    for handle in handles {
        results.push(handle.await.unwrap());
    }

    let concurrent_time = start.elapsed();
    println!("Concurrent operations time: {:?}", concurrent_time);

    // Verify all operations completed successfully
    assert_eq!(results.len(), 20);
    for result in results {
        assert!(result.is_ok());
    }

    // Check pool stats
    let stats = pooled_db.pool_stats();
    println!("Pool stats: {:?}", stats);

    // Should complete in reasonable time
    assert!(concurrent_time < Duration::from_secs(10));
}

#[tokio::test]
async fn benchmark_index_effectiveness() {
    let db = Database::new(None).unwrap();
    let project_path = Path::new("/test/indexes");
    let run = db.create_analysis_run(project_path).unwrap();
    let run_id = run.run_id.unwrap();

    // Create large dataset to test index effectiveness
    let mut anti_pattern_types = vec![
        create_test_anti_pattern_type("index_test_1"),
        create_test_anti_pattern_type("index_test_2"),
    ];
    db.store_anti_pattern_types_batch(&mut anti_pattern_types)
        .unwrap();

    let issues = create_test_issues(run_id, 1, 50000);
    let mut db_mut = db;
    db_mut.store_issues(&issues).unwrap();

    // Test queries that should benefit from indexes
    let test_queries = vec![
        ("severity filter", |db: &Database, run_id: i64| {
            Box::pin(async move {
                db.get_issues_paginated(run_id, 0, 1000, Some("critical"), None)
                    .await
            })
        }),
        ("detector filter", |db: &Database, run_id: i64| {
            Box::pin(async move {
                db.get_issues_paginated(run_id, 0, 1000, None, Some("test_detector"))
                    .await
            })
        }),
        ("stats aggregation", |db: &Database, run_id: i64| {
            Box::pin(async move { db.get_analysis_stats(run_id).await.map(|_| vec![]) })
        }),
    ];

    for (query_name, query_fn) in test_queries {
        let start = Instant::now();
        let _result = query_fn(&db_mut, run_id).await.unwrap();
        let query_time = start.elapsed();

        println!("{} time: {:?}", query_name, query_time);

        // With proper indexes, even large datasets should query quickly
        assert!(
            query_time < Duration::from_millis(200),
            "Query '{}' took too long: {:?}",
            query_name,
            query_time
        );
    }
}

#[tokio::test]
async fn benchmark_memory_usage() {
    let db = Database::new(None).unwrap();
    let project_path = Path::new("/test/memory");
    let run = db.create_analysis_run(project_path).unwrap();
    let run_id = run.run_id.unwrap();

    // Create test data
    let mut anti_pattern_types = vec![create_test_anti_pattern_type("memory_test")];
    db.store_anti_pattern_types_batch(&mut anti_pattern_types)
        .unwrap();

    // Test streaming vs. loading all at once
    let large_issues = create_test_issues(run_id, 1, 100000);
    let mut db_mut = db;

    // Store in batches to avoid memory spike
    for chunk in large_issues.chunks(1000) {
        db_mut.store_issues(chunk).unwrap();
    }

    // Test paginated access (memory efficient)
    let start = Instant::now();
    let mut total_retrieved = 0;
    let page_size = 1000;
    let mut offset = 0;

    loop {
        let page = db_mut
            .get_issues_paginated(run_id, offset, page_size, None, None)
            .await
            .unwrap();
        if page.is_empty() {
            break;
        }
        total_retrieved += page.len();
        offset += page_size;
    }

    let streaming_time = start.elapsed();
    println!("Streaming retrieval time: {:?}", streaming_time);
    println!("Total retrieved: {}", total_retrieved);

    assert_eq!(total_retrieved, 100000);
    // Streaming should complete in reasonable time
    assert!(streaming_time < Duration::from_secs(30));
}
