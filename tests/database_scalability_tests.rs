//! Comprehensive Database Scalability Tests
//!
//! This module contains tests for database scalability features including
//! connection pooling, read/write separation, load balancing, and performance
//! under concurrent load.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use uveddi::database::{
    config_manager::{DatabaseConfigManager, Environment},
    migration_manager::MigrationManager,
    models::{AnalysisRun, AntiPatternType, ArchitecturalIssue},
    monitoring::{DatabaseMonitor, MonitoringConfig},
    providers::create_database_provider,
    scalable_manager::ScalableDatabase,
    DatabaseConfig,
    DatabaseType,
    PoolConfig,
};
use uveddi::error::Result;

fn build_sqlite_config(
    connection_string: &str,
    read_connections: Vec<String>,
    max_connections: usize,
    min_connections: usize,
    connection_timeout: Duration,
    idle_timeout: Duration,
    max_lifetime: Duration,
    pool_timeout: Duration,
) -> DatabaseConfig {
    let pool = PoolConfig::builder()
        .max_connections(max_connections)
        .min_connections(min_connections)
        .connection_timeout(connection_timeout)
        .idle_timeout(idle_timeout)
        .max_lifetime(max_lifetime)
        .pool_timeout(pool_timeout)
        .build();

    DatabaseConfig::sqlite(connection_string)
        .with_pool(pool)
        .with_read_connections(read_connections)
        .with_logging(false)
        .with_prepared_statements(true)
}

/// Test basic connection pooling functionality
#[tokio::test]
async fn test_connection_pooling() -> Result<()> {
    let config = build_sqlite_config(
        ":memory:",
        vec![],
        10,
        2,
        Duration::from_secs(5),
        Duration::from_secs(30),
        Duration::from_secs(300),
        Duration::from_secs(5),
    );

    let database = Arc::new(ScalableDatabase::new(config).await?);

    // Test basic connectivity
    let health = database.get_health_status().await?;
    assert!(health.is_healthy);

    // Test concurrent operations
    let mut handles = vec![];

    for i in 0..20 {
        let db = database.clone();
        let handle = tokio::spawn(async move {
            let project_path = std::path::Path::new(&format!("/test/project/{}", i));
            let run = db.create_analysis_run(project_path).await?;
            assert!(run.run_id.is_some());
            Ok::<_, uveddi::error::UveddiError>(())
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    for handle in handles {
        handle.await.unwrap()?;
    }

    Ok(())
}

/// Test read/write separation and load balancing
#[tokio::test]
async fn test_read_write_separation() -> Result<()> {
    let config = build_sqlite_config(
        ":memory:",
        vec![":memory:".to_string()],
        10,
        2,
        Duration::from_secs(5),
        Duration::from_secs(30),
        Duration::from_secs(300),
        Duration::from_secs(5),
    );

    let database = Arc::new(ScalableDatabase::new(config).await?);

    // Create test data
    let project_path = std::path::Path::new("/test/project");
    let run = database.create_analysis_run(project_path).await?;
    let run_id = run.run_id.unwrap();

    // Test concurrent reads and writes
    let mut read_handles = vec![];
    let mut write_handles = vec![];

    // Spawn read operations
    for _ in 0..10 {
        let db = database.clone();
        let handle = tokio::spawn(async move {
            let result = db.get_analysis_run(run_id).await?;
            assert!(result.is_some());
            Ok::<_, uveddi::error::UveddiError>(())
        });
        read_handles.push(handle);
    }

    // Spawn write operations
    for i in 0..5 {
        let db = database.clone();
        let handle = tokio::spawn(async move {
            let project_path = std::path::Path::new(&format!("/test/write/{}", i));
            let run = db.create_analysis_run(project_path).await?;
            assert!(run.run_id.is_some());
            Ok::<_, uveddi::error::UveddiError>(())
        });
        write_handles.push(handle);
    }

    // Wait for all operations
    for handle in read_handles {
        handle.await.unwrap()?;
    }

    for handle in write_handles {
        handle.await.unwrap()?;
    }

    Ok(())
}

/// Test database under high concurrent load
#[tokio::test]
async fn test_concurrent_load() -> Result<()> {
    let config = build_sqlite_config(
        ":memory:",
        vec![],
        50,
        10,
        Duration::from_secs(10),
        Duration::from_secs(60),
        Duration::from_secs(600),
        Duration::from_secs(10),
    );

    let database = Arc::new(ScalableDatabase::new(config).await?);

    // Create initial data
    let project_path = std::path::Path::new("/test/load");
    let run = database.create_analysis_run(project_path).await?;
    let run_id = run.run_id.unwrap();

    // Add some anti-pattern types
    let mut anti_patterns = vec![
        AntiPatternType {
            anti_pattern_type_id: None,
            name: "GodObject".to_string(),
            description: "Class with too many responsibilities".to_string(),
            category: "Design".to_string(),
        },
        AntiPatternType {
            anti_pattern_type_id: None,
            name: "DeadCode".to_string(),
            description: "Unused code that can be removed".to_string(),
            category: "Maintenance".to_string(),
        },
    ];

    database
        .store_anti_pattern_types_batch(&mut anti_patterns)
        .await?;

    let start_time = Instant::now();
    let concurrent_operations = 100;
    let mut handles = vec![];

    // Mix of different operations
    for i in 0..concurrent_operations {
        let db = database.clone();
        let anti_pattern_id = anti_patterns[i % anti_patterns.len()]
            .anti_pattern_type_id
            .unwrap();

        let handle = tokio::spawn(async move {
            match i % 4 {
                0 => {
                    // Create analysis run
                    let path = std::path::Path::new(&format!("/test/concurrent/{}", i));
                    let run = db.create_analysis_run(path).await?;
                    assert!(run.run_id.is_some());
                }
                1 => {
                    // Store issues
                    let issues = vec![ArchitecturalIssue {
                        issue_id: None,
                        analysis_run_id: run_id,
                        anti_pattern_type_id: anti_pattern_id,
                        file_path: format!("/test/file_{}.rs", i),
                        start_line: Some(10),
                        end_line: Some(20),
                        line_number: Some(15),
                        column_number: Some(5),
                        message: format!("Issue {}", i),
                        metadata: "{}".to_string(),
                        detector_name: "TestDetector".to_string(),
                        created_at: chrono::Utc::now(),
                        severity: "medium".to_string(),
                        description: format!("Test issue {}", i),
                        code_snippet: Some("fn test() {}".to_string()),
                        ai_explanation: None,
                    }];
                    db.store_issues_batch(&issues).await?;
                }
                2 => {
                    // Read analysis run
                    let result = db.get_analysis_run(run_id).await?;
                    assert!(result.is_some());
                }
                3 => {
                    // Get analysis stats
                    let stats = db.get_analysis_stats(run_id).await?;
                    assert!(stats.total_issues >= 0);
                }
                _ => unreachable!(),
            }
            Ok::<_, uveddi::error::UveddiError>(())
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    let mut successes = 0;
    let mut failures = 0;

    for handle in handles {
        match handle.await.unwrap() {
            Ok(_) => successes += 1,
            Err(_) => failures += 1,
        }
    }

    let duration = start_time.elapsed();
    let ops_per_second = concurrent_operations as f64 / duration.as_secs_f64();

    println!("Concurrent load test results:");
    println!("  Operations: {}", concurrent_operations);
    println!("  Successes: {}", successes);
    println!("  Failures: {}", failures);
    println!("  Duration: {:?}", duration);
    println!("  Ops/sec: {:.2}", ops_per_second);

    // Verify performance requirements
    assert!(
        ops_per_second > 10.0,
        "Performance below threshold: {} ops/sec",
        ops_per_second
    );
    assert!(
        failures < concurrent_operations / 10,
        "Too many failures: {}",
        failures
    );

    Ok(())
}

/// Test connection pool limits and timeouts
#[tokio::test]
async fn test_connection_pool_limits() -> Result<()> {
    let config = build_sqlite_config(
        ":memory:",
        vec![],
        5,
        1,
        Duration::from_millis(100),
        Duration::from_secs(10),
        Duration::from_secs(60),
        Duration::from_millis(500),
    );

    let database = Arc::new(ScalableDatabase::new(config).await?);

    // Create more concurrent operations than available connections
    let mut handles = vec![];
    let operations_count = 10; // More than max_connections

    for i in 0..operations_count {
        let db = database.clone();
        let handle = tokio::spawn(async move {
            let path = std::path::Path::new(&format!("/test/pool_limit/{}", i));
            // Add some delay to hold connections longer
            sleep(Duration::from_millis(50)).await;
            let result = db.create_analysis_run(path).await;
            (i, result)
        });
        handles.push(handle);
    }

    let mut successes = 0;
    let mut timeouts = 0;

    for handle in handles {
        let (i, result) = handle.await.unwrap();
        match result {
            Ok(_) => {
                successes += 1;
                println!("Operation {} succeeded", i);
            }
            Err(e) => {
                timeouts += 1;
                println!("Operation {} failed: {}", i, e);
            }
        }
    }

    println!("Pool limits test results:");
    println!("  Total operations: {}", operations_count);
    println!("  Successes: {}", successes);
    println!("  Timeouts: {}", timeouts);

    // Should handle all operations eventually, even with limited connections
    assert!(successes > 0, "No operations succeeded");

    Ok(())
}

/// Test monitoring system functionality
#[tokio::test]
async fn test_monitoring_system() -> Result<()> {
    let config = build_sqlite_config(
        ":memory:",
        vec![],
        10,
        2,
        Duration::from_secs(5),
        Duration::from_secs(30),
        Duration::from_secs(300),
        Duration::from_secs(5),
    );

    let database = Arc::new(ScalableDatabase::new(config).await?);

    let monitoring_config = MonitoringConfig {
        health_check_interval: Duration::from_millis(100),
        metrics_collection_interval: Duration::from_millis(200),
        cleanup_interval: Duration::from_secs(1),
        metrics_retention_period: Duration::from_secs(60),
        error_rate_threshold: 0.1,
        max_connections_threshold: 50,
        response_time_threshold: Duration::from_secs(1),
    };

    let monitor = DatabaseMonitor::new(database.clone(), monitoring_config);
    let _monitoring_handle = monitor.start_monitoring();

    // Let monitoring run for a bit
    sleep(Duration::from_millis(500)).await;

    // Perform some operations to generate metrics
    for i in 0..5 {
        let path = std::path::Path::new(&format!("/test/monitoring/{}", i));
        database.create_analysis_run(path).await?;
    }

    sleep(Duration::from_millis(500)).await;

    // Check current metrics
    let metrics = monitor.get_current_metrics().await?;
    assert!(metrics.health_status.is_healthy);
    assert!(metrics.health_status.active_connections >= 0);

    // Check metrics history
    let history = monitor.get_metrics_history(Duration::from_secs(10)).await;
    assert!(!history.is_empty(), "Should have collected some metrics");

    // Generate monitoring report
    let report = monitor.generate_report(Duration::from_secs(10)).await;
    assert!(report.uptime_percentage >= 0.0);
    assert!(report.health_checks_performed > 0);

    println!("Monitoring report:");
    println!("  Uptime: {:.2}%", report.uptime_percentage);
    println!("  Avg response time: {:?}", report.average_response_time);
    println!("  Health checks: {}", report.health_checks_performed);

    Ok(())
}

/// Test configuration management
#[tokio::test]
async fn test_configuration_management() -> Result<()> {
    // Test config manager creation
    let config_manager = DatabaseConfigManager::new()?;

    // Test environment detection
    let env = config_manager.get_environment();
    println!("Detected environment: {:?}", env);

    // Test config validation
    config_manager.validate_config()?;

    // Test getting configs for different environments
    let dev_config = config_manager.get_config_for_env(Environment::Development);
    let prod_config = config_manager.get_config_for_env(Environment::Production);

    assert_eq!(dev_config.database_type, DatabaseType::SQLite);
    assert_eq!(prod_config.database_type, DatabaseType::PostgreSQL);
    assert!(
        prod_config.pool.max_connections > dev_config.pool.max_connections
    );

    // Test masked connection string
    let masked = config_manager.get_masked_connection_string();
    assert!(
        !masked.contains("password"),
        "Connection string should be masked"
    );

    Ok(())
}

/// Test database migration system
#[tokio::test]
async fn test_migration_system() -> Result<()> {
    let config = build_sqlite_config(
        ":memory:",
        vec![],
        5,
        1,
        Duration::from_secs(5),
        Duration::from_secs(30),
        Duration::from_secs(300),
        Duration::from_secs(5),
    );

    let temp_dir = tempfile::tempdir().unwrap();
    let migrations_dir = temp_dir.path().to_path_buf();

    // Create a test migration
    let migration_manager = MigrationManager::new(config, Some(migrations_dir.clone())).await?;
    let migration_path = migration_manager.create_migration("add_test_table").await?;

    // Write migration content
    let migration_sql = "CREATE TABLE test_table (id INTEGER PRIMARY KEY, name TEXT);";
    tokio::fs::write(&migration_path, migration_sql)
        .await
        .map_err(|e| uveddi::error::UveddiError::io_error(&e.to_string()))?;

    // Test current version
    let version = migration_manager.get_current_version().await?;
    assert_eq!(version, 0);

    // Apply migrations
    let result = migration_manager.migrate_to_latest().await?;
    assert!(result.success, "Migration should succeed");
    assert!(
        !result.applied_migrations.is_empty(),
        "Should have applied migrations"
    );

    // Check new version
    let new_version = migration_manager.get_current_version().await?;
    assert!(new_version > version);

    // Get migration history
    let history = migration_manager.get_migration_history().await?;
    assert!(!history.is_empty(), "Should have migration history");

    println!("Migration test results:");
    println!("  Initial version: {}", version);
    println!("  Final version: {}", new_version);
    println!("  Applied migrations: {}", result.applied_migrations.len());
    println!("  Execution time: {:?}", result.total_execution_time);

    Ok(())
}

/// Benchmark database performance under various loads
#[tokio::test]
async fn benchmark_database_performance() -> Result<()> {
    let config = build_sqlite_config(
        ":memory:",
        vec![],
        25,
        5,
        Duration::from_secs(10),
        Duration::from_secs(60),
        Duration::from_secs(600),
        Duration::from_secs(10),
    );

    let database = Arc::new(ScalableDatabase::new(config).await?);

    // Benchmark different operation types
    let benchmark_results = vec![
        benchmark_operation("Create Analysis Runs", 100, || {
            let db = database.clone();
            async move {
                let path = std::path::Path::new(&format!("/benchmark/{}", rand::random::<u32>()));
                db.create_analysis_run(path).await.map(|_| ())
            }
        })
        .await,
        benchmark_operation("Read Operations", 200, || {
            let db = database.clone();
            async move { db.get_recent_analysis_runs(10).await.map(|_| ()) }
        })
        .await,
    ];

    println!("\nBenchmark Results:");
    println!("==================");
    for (name, ops_per_sec, avg_duration) in benchmark_results {
        println!(
            "{}: {:.2} ops/sec, avg: {:?}",
            name, ops_per_sec, avg_duration
        );

        // Performance assertions
        match name.as_str() {
            "Create Analysis Runs" => assert!(
                ops_per_sec > 50.0,
                "Create operations too slow: {:.2} ops/sec",
                ops_per_sec
            ),
            "Read Operations" => assert!(
                ops_per_sec > 100.0,
                "Read operations too slow: {:.2} ops/sec",
                ops_per_sec
            ),
            _ => {}
        }
    }

    Ok(())
}

/// Generic benchmark function
async fn benchmark_operation<F, Fut>(
    name: &str,
    iterations: usize,
    operation: impl Fn() -> F + Send + Sync + Clone + 'static,
) -> (String, f64, Duration)
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<()>> + Send,
{
    let start_time = Instant::now();
    let mut handles = vec![];

    for _ in 0..iterations {
        let op = operation.clone();
        let handle = tokio::spawn(async move { op().await });
        handles.push(handle);
    }

    let mut successes = 0;
    for handle in handles {
        if handle.await.unwrap().is_ok() {
            successes += 1;
        }
    }

    let total_duration = start_time.elapsed();
    let ops_per_second = successes as f64 / total_duration.as_secs_f64();
    let avg_duration = total_duration / successes as u32;

    (name.to_string(), ops_per_second, avg_duration)
}

/// Test cleanup operations
#[tokio::test]
async fn test_cleanup_operations() -> Result<()> {
    let config = build_sqlite_config(
        ":memory:",
        vec![],
        10,
        2,
        Duration::from_secs(1),
        Duration::from_millis(100),
        Duration::from_millis(500),
        Duration::from_secs(5),
    );

    let database = Arc::new(ScalableDatabase::new(config).await?);

    // Create some operations to generate connections
    for i in 0..5 {
        let path = std::path::Path::new(&format!("/test/cleanup/{}", i));
        database.create_analysis_run(path).await?;
    }

    // Wait for connections to become eligible for cleanup
    sleep(Duration::from_secs(1)).await;

    // Perform cleanup
    let cleaned_connections = database.cleanup().await?;
    println!("Cleaned up {} expired connections", cleaned_connections);

    // Verify database is still functional after cleanup
    let health = database.get_health_status().await?;
    assert!(health.is_healthy);

    Ok(())
}

// Helper function to generate random data for testing
fn generate_test_issues(
    run_id: i64,
    anti_pattern_id: i64,
    count: usize,
) -> Vec<ArchitecturalIssue> {
    (0..count)
        .map(|i| ArchitecturalIssue {
            issue_id: None,
            analysis_run_id: run_id,
            anti_pattern_type_id: anti_pattern_id,
            file_path: format!("/test/file_{}.rs", i),
            start_line: Some(10 + i as u32),
            end_line: Some(20 + i as u32),
            line_number: Some(15 + i as u32),
            column_number: Some(5),
            message: format!("Test issue {}", i),
            metadata: "{}".to_string(),
            detector_name: "TestDetector".to_string(),
            created_at: chrono::Utc::now(),
            severity: if i % 3 == 0 { "high" } else { "medium" }.to_string(),
            description: format!("Test issue description {}", i),
            code_snippet: Some(format!("fn test_{}() {{}}", i)),
            ai_explanation: None,
        })
        .collect()
}

/// Test batch operations performance
#[tokio::test]
async fn test_batch_operations() -> Result<()> {
    let config = build_sqlite_config(
        ":memory:",
        vec![],
        20,
        5,
        Duration::from_secs(10),
        Duration::from_secs(60),
        Duration::from_secs(600),
        Duration::from_secs(10),
    );

    let database = Arc::new(ScalableDatabase::new(config).await?);

    // Setup
    let project_path = std::path::Path::new("/test/batch");
    let run = database.create_analysis_run(project_path).await?;
    let run_id = run.run_id.unwrap();

    let mut anti_patterns = vec![AntiPatternType {
        anti_pattern_type_id: None,
        name: "BatchTestPattern".to_string(),
        description: "Test pattern for batch operations".to_string(),
        category: "Test".to_string(),
    }];

    database
        .store_anti_pattern_types_batch(&mut anti_patterns)
        .await?;
    let anti_pattern_id = anti_patterns[0].anti_pattern_type_id.unwrap();

    // Test batch issue insertion
    let batch_sizes = vec![10, 50, 100, 500];

    for batch_size in batch_sizes {
        let issues = generate_test_issues(run_id, anti_pattern_id, batch_size);

        let start_time = Instant::now();
        database.store_issues_batch(&issues).await?;
        let duration = start_time.elapsed();

        let issues_per_second = batch_size as f64 / duration.as_secs_f64();
        println!(
            "Batch size {}: {:.2} issues/sec ({:?} total)",
            batch_size, issues_per_second, duration
        );

        // Performance assertion - should handle at least 100 issues per second
        assert!(
            issues_per_second > 100.0,
            "Batch insert too slow for size {}: {:.2} issues/sec",
            batch_size,
            issues_per_second
        );
    }

    // Verify all data was inserted correctly
    let stats = database.get_analysis_stats(run_id).await?;
    let expected_total = batch_sizes.iter().sum::<usize>() as u32;
    assert_eq!(stats.total_issues, expected_total);

    Ok(())
}
