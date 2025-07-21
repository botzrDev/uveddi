//! Database operations integration tests for UV-243
//! Tests database interactions and data persistence with comprehensive coverage

use sqlx::{sqlite::SqlitePool, postgres::PgPool, Row, Transaction, Executor};
use std::env;
use std::time::{SystemTime, Duration};
use crate::monitoring::metrics::{TestMetrics, TestStatus, ResourceUsage};
use crate::database::models::{ArchitecturalIssue, AntiPatternType};
use crate::database::crud::{DatabaseCrud, TestResultRepository};
use serde_json;
use uuid::Uuid;

#[cfg(test)]
mod database_connection_tests {
    use super::*;
    use crate::integration::test_setup;
    
    #[tokio::test]
    async fn test_sqlite_database_connection() {
        test_setup::init();
        
        // Test SQLite database connection establishment
        let db_url = test_setup::setup_test_database().await
            .expect("Should get test database URL");
        
        let pool = SqlitePool::connect(&db_url).await;
        assert!(pool.is_ok(), "SQLite database connection should succeed");
        
        let pool = pool.unwrap();
        
        // Test connection is actually working
        let row: (i64,) = sqlx::query_as("SELECT 1")
            .fetch_one(&pool)
            .await
            .expect("Should execute simple query");
        
        assert_eq!(row.0, 1, "Simple query should return expected value");
        
        pool.close().await;
    }
    
    #[tokio::test] 
    async fn test_database_connection_pooling() {
        test_setup::init();
        
        let db_url = test_setup::setup_test_database().await.unwrap();
        let pool = SqlitePool::connect(&db_url).await.unwrap();
        
        // Test concurrent connections from pool
        let mut handles = Vec::new();
        
        for i in 0..5 {
            let pool_clone = pool.clone();
            let handle = tokio::spawn(async move {
                let result: Result<(i64,), sqlx::Error> = sqlx::query_as("SELECT ?")
                    .bind(i)
                    .fetch_one(&pool_clone)
                    .await;
                
                result.map(|row| row.0)
            });
            handles.push(handle);
        }
        
        // Verify all connections work concurrently
        for (i, handle) in handles.into_iter().enumerate() {
            let result = handle.await.expect("Task should complete");
            let value = result.expect("Query should succeed");
            assert_eq!(value, i as i64, "Query should return correct value");
        }
        
        pool.close().await;
    }
    
    #[tokio::test]
    async fn test_database_connection_recovery() {
        test_setup::init();
        
        let db_url = test_setup::setup_test_database().await.unwrap();
        let pool = SqlitePool::connect(&db_url).await.unwrap();
        
        // Test that pool handles connection issues gracefully
        let initial_query: Result<(i64,), _> = sqlx::query_as("SELECT 1")
            .fetch_one(&pool)
            .await;
        assert!(initial_query.is_ok(), "Initial query should succeed");
        
        // Simulate connection issue recovery by testing multiple queries
        for i in 0..10 {
            let query_result: Result<(i64,), _> = sqlx::query_as("SELECT ?")
                .bind(i)
                .fetch_one(&pool)
                .await;
            
            assert!(query_result.is_ok(), "Query {} should succeed after recovery", i);
        }
        
        pool.close().await;
    }
}

#[cfg(test)]
mod database_schema_tests {
    use super::*;
    use crate::integration::test_setup;
    
    #[tokio::test]
    async fn test_database_schema_creation() {
        test_setup::init();
        
        let db_url = test_setup::setup_test_database().await.unwrap();
        let pool = SqlitePool::connect(&db_url).await.unwrap();
        
        // Test comprehensive schema creation
        let schema_sql = r#"
            CREATE TABLE IF NOT EXISTS test_metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                execution_id TEXT NOT NULL,
                test_name TEXT NOT NULL,
                test_suite TEXT NOT NULL,
                status TEXT NOT NULL,
                duration_ms INTEGER NOT NULL,
                cpu_percent REAL NOT NULL,
                memory_mb INTEGER NOT NULL,
                disk_io_mb INTEGER NOT NULL,
                failure_category TEXT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            
            CREATE TABLE IF NOT EXISTS architectural_issues (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_path TEXT NOT NULL,
                issue_type TEXT NOT NULL,
                severity TEXT NOT NULL,
                description TEXT NOT NULL,
                line_number INTEGER,
                column_number INTEGER,
                metadata TEXT, -- JSON blob
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            
            CREATE INDEX IF NOT EXISTS idx_test_metrics_execution_id ON test_metrics(execution_id);
            CREATE INDEX IF NOT EXISTS idx_test_metrics_status ON test_metrics(status);
            CREATE INDEX IF NOT EXISTS idx_test_metrics_timestamp ON test_metrics(timestamp);
            CREATE INDEX IF NOT EXISTS idx_architectural_issues_type ON architectural_issues(issue_type);
            CREATE INDEX IF NOT EXISTS idx_architectural_issues_file ON architectural_issues(file_path);
        "#;
        
        let result = sqlx::query(schema_sql).execute(&pool).await;
        assert!(result.is_ok(), "Schema creation should succeed");
        
        // Verify tables were created
        let tables: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM sqlite_master WHERE type='table' AND name IN ('test_metrics', 'architectural_issues')"
        ).fetch_all(&pool).await.unwrap();
        
        assert_eq!(tables.len(), 2, "Both tables should be created");
        
        // Verify indexes were created  
        let indexes: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'"
        ).fetch_all(&pool).await.unwrap();
        
        assert!(indexes.len() >= 5, "All indexes should be created");
        
        pool.close().await;
    }
    
    #[tokio::test]
    async fn test_database_migration_integrity() {
        test_setup::init();
        
        let db_url = test_setup::setup_test_database().await.unwrap();
        let pool = SqlitePool::connect(&db_url).await.unwrap();
        
        // Create initial schema version
        sqlx::query(r#"
            CREATE TABLE test_data (
                id INTEGER PRIMARY KEY,
                value TEXT NOT NULL
            );
            INSERT INTO test_data (value) VALUES ('initial_data');
        "#).execute(&pool).await.unwrap();
        
        // Verify initial data
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM test_data")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(count.0, 1, "Initial data should exist");
        
        // Simulate migration: add new column
        sqlx::query(r#"
            ALTER TABLE test_data ADD COLUMN new_column TEXT DEFAULT 'migrated';
        "#).execute(&pool).await.unwrap();
        
        // Verify migration preserved data
        let migrated_data: Vec<(i64, String, Option<String>)> = sqlx::query_as(
            "SELECT id, value, new_column FROM test_data"
        ).fetch_all(&pool).await.unwrap();
        
        assert_eq!(migrated_data.len(), 1, "Data should be preserved during migration");
        let (id, value, new_col) = &migrated_data[0];
        assert_eq!(value, "initial_data", "Original data should be intact");
        assert_eq!(new_col.as_deref(), Some("migrated"), "New column should have default value");
        
        pool.close().await;
    }
    
    #[tokio::test]
    async fn test_database_schema_constraints() {
        test_setup::init();
        
        let db_url = test_setup::setup_test_database().await.unwrap();
        let pool = SqlitePool::connect(&db_url).await.unwrap();
        
        // Create table with constraints
        sqlx::query(r#"
            CREATE TABLE test_constraints (
                id INTEGER PRIMARY KEY,
                email TEXT UNIQUE NOT NULL,
                age INTEGER CHECK(age >= 0 AND age <= 150),
                status TEXT NOT NULL DEFAULT 'active',
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
        "#).execute(&pool).await.unwrap();
        
        // Test successful insert
        let insert_result = sqlx::query(
            "INSERT INTO test_constraints (email, age) VALUES (?, ?)"
        )
        .bind("test@example.com")
        .bind(25)
        .execute(&pool)
        .await;
        assert!(insert_result.is_ok(), "Valid insert should succeed");
        
        // Test unique constraint violation
        let duplicate_result = sqlx::query(
            "INSERT INTO test_constraints (email, age) VALUES (?, ?)"
        )
        .bind("test@example.com") // Same email
        .bind(30)
        .execute(&pool)
        .await;
        assert!(duplicate_result.is_err(), "Duplicate email should fail");
        
        // Test check constraint violation
        let invalid_age_result = sqlx::query(
            "INSERT INTO test_constraints (email, age) VALUES (?, ?)"
        )
        .bind("test2@example.com")
        .bind(200) // Invalid age
        .execute(&pool)
        .await;
        assert!(invalid_age_result.is_err(), "Invalid age should fail check constraint");
        
        pool.close().await;
    }
}

#[cfg(test)]
mod database_crud_tests {
    use super::*;
    use crate::integration::test_setup;
    
    #[tokio::test]
    async fn test_database_crud_operations() {
        test_setup::init();
        
        let db_url = test_setup::setup_test_database().await.unwrap();
        let pool = SqlitePool::connect(&db_url).await.unwrap();
        
        // Setup test table
        sqlx::query(r#"
            CREATE TABLE test_metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                execution_id TEXT NOT NULL,
                test_name TEXT NOT NULL,
                test_suite TEXT NOT NULL,
                status TEXT NOT NULL,
                duration_ms INTEGER NOT NULL,
                cpu_percent REAL NOT NULL,
                memory_mb INTEGER NOT NULL,
                disk_io_mb INTEGER NOT NULL,
                failure_category TEXT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            )
        "#).execute(&pool).await.unwrap();
        
        let test_metrics = TestMetrics {
            execution_id: "crud_test_001".to_string(),
            test_name: "database_crud_test".to_string(),
            test_suite: "integration".to_string(),
            status: TestStatus::Passed,
            duration_ms: 2000,
            resource_usage: ResourceUsage {
                cpu_percent: 15.5,
                memory_mb: 64,
                disk_io_mb: 5,
            },
            failure_category: None,
            timestamp: SystemTime::now(),
        };
        
        // Test CREATE (INSERT)
        let insert_result = sqlx::query(
            r#"INSERT INTO test_metrics 
               (execution_id, test_name, test_suite, status, duration_ms, cpu_percent, memory_mb, disk_io_mb, failure_category) 
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(&test_metrics.execution_id)
        .bind(&test_metrics.test_name)
        .bind(&test_metrics.test_suite)
        .bind(format!("{:?}", test_metrics.status))
        .bind(test_metrics.duration_ms as i64)
        .bind(test_metrics.resource_usage.cpu_percent)
        .bind(test_metrics.resource_usage.memory_mb as i64)
        .bind(test_metrics.resource_usage.disk_io_mb as i64)
        .bind(&test_metrics.failure_category)
        .execute(&pool)
        .await;
        
        assert!(insert_result.is_ok(), "INSERT operation should succeed");
        let insert_result = insert_result.unwrap();
        assert_eq!(insert_result.rows_affected(), 1, "Should insert exactly one row");
        
        // Test READ (SELECT)
        let select_result = sqlx::query(
            "SELECT execution_id, test_name, status, duration_ms FROM test_metrics WHERE execution_id = ?"
        )
        .bind(&test_metrics.execution_id)
        .fetch_one(&pool)
        .await;
        
        assert!(select_result.is_ok(), "SELECT operation should succeed");
        let row = select_result.unwrap();
        
        let execution_id: String = row.get("execution_id");
        let test_name: String = row.get("test_name");
        let status: String = row.get("status");
        let duration_ms: i64 = row.get("duration_ms");
        
        assert_eq!(execution_id, test_metrics.execution_id, "Execution ID should match");
        assert_eq!(test_name, test_metrics.test_name, "Test name should match");
        assert_eq!(status, "Passed", "Status should match");
        assert_eq!(duration_ms, test_metrics.duration_ms as i64, "Duration should match");
        
        // Test UPDATE
        let update_result = sqlx::query(
            "UPDATE test_metrics SET status = ?, duration_ms = ? WHERE execution_id = ?"
        )
        .bind("Failed")
        .bind(3000i64)
        .bind(&test_metrics.execution_id)
        .execute(&pool)
        .await;
        
        assert!(update_result.is_ok(), "UPDATE operation should succeed");
        assert_eq!(update_result.unwrap().rows_affected(), 1, "Should update exactly one row");
        
        // Verify update
        let updated_row = sqlx::query(
            "SELECT status, duration_ms FROM test_metrics WHERE execution_id = ?"
        )
        .bind(&test_metrics.execution_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        
        let updated_status: String = updated_row.get("status");
        let updated_duration: i64 = updated_row.get("duration_ms");
        
        assert_eq!(updated_status, "Failed", "Status should be updated");
        assert_eq!(updated_duration, 3000, "Duration should be updated");
        
        // Test DELETE
        let delete_result = sqlx::query(
            "DELETE FROM test_metrics WHERE execution_id = ?"
        )
        .bind(&test_metrics.execution_id)
        .execute(&pool)
        .await;
        
        assert!(delete_result.is_ok(), "DELETE operation should succeed");
        assert_eq!(delete_result.unwrap().rows_affected(), 1, "Should delete exactly one row");
        
        // Verify deletion
        let count_result: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM test_metrics WHERE execution_id = ?"
        )
        .bind(&test_metrics.execution_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        
        assert_eq!(count_result.0, 0, "Row should be deleted");
        
        pool.close().await;
    }
    
    #[tokio::test]
    async fn test_database_bulk_operations() {
        test_setup::init();
        
        let db_url = test_setup::setup_test_database().await.unwrap();
        let pool = SqlitePool::connect(&db_url).await.unwrap();
        
        // Setup test table
        sqlx::query(r#"
            CREATE TABLE bulk_test (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                batch_id TEXT NOT NULL,
                value INTEGER NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
        "#).execute(&pool).await.unwrap();
        
        let batch_id = Uuid::new_v4().to_string();
        let batch_size = 1000;
        
        // Test bulk insert performance
        let start_time = SystemTime::now();
        
        let mut tx = pool.begin().await.unwrap();
        
        for i in 0..batch_size {
            sqlx::query("INSERT INTO bulk_test (batch_id, value) VALUES (?, ?)")
                .bind(&batch_id)
                .bind(i)
                .execute(&mut *tx)
                .await
                .expect(&format!("Bulk insert {} should succeed", i));
        }
        
        tx.commit().await.expect("Bulk transaction should commit");
        
        let elapsed = start_time.elapsed().unwrap();
        let inserts_per_second = batch_size as f64 / elapsed.as_secs_f64();
        
        assert!(inserts_per_second > 100.0, "Bulk insert should achieve reasonable performance");
        println!("Bulk insert performance: {:.2} inserts/second", inserts_per_second);
        
        // Verify all records were inserted
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM bulk_test WHERE batch_id = ?"
        )
        .bind(&batch_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        
        assert_eq!(count.0, batch_size as i64, "All bulk records should be inserted");
        
        // Test bulk update
        let update_start = SystemTime::now();
        
        let update_result = sqlx::query(
            "UPDATE bulk_test SET value = value * 2 WHERE batch_id = ?"
        )
        .bind(&batch_id)
        .execute(&pool)
        .await;
        
        assert!(update_result.is_ok(), "Bulk update should succeed");
        assert_eq!(update_result.unwrap().rows_affected(), batch_size as u64, 
                  "Bulk update should affect all records");
        
        let update_elapsed = update_start.elapsed().unwrap();
        let updates_per_second = batch_size as f64 / update_elapsed.as_secs_f64();
        
        println!("Bulk update performance: {:.2} updates/second", updates_per_second);
        
        // Test bulk delete
        let delete_result = sqlx::query(
            "DELETE FROM bulk_test WHERE batch_id = ?"
        )
        .bind(&batch_id)
        .execute(&pool)
        .await;
        
        assert!(delete_result.is_ok(), "Bulk delete should succeed");
        assert_eq!(delete_result.unwrap().rows_affected(), batch_size as u64,
                  "Bulk delete should remove all records");
        
        pool.close().await;
    }
}

#[cfg(test)]
mod database_transaction_tests {
    use super::*;
    use crate::integration::test_setup;
    
    #[tokio::test]
    async fn test_database_transaction_rollback() {
        test_setup::init();
        
        let db_url = test_setup::setup_test_database().await.unwrap();
        let pool = SqlitePool::connect(&db_url).await.unwrap();
        
        // Setup test table
        sqlx::query(r#"
            CREATE TABLE transaction_test (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                value TEXT NOT NULL
            )
        "#).execute(&pool).await.unwrap();
        
        // Test transaction rollback
        let mut tx = pool.begin().await.unwrap();
        
        // Insert data in transaction
        sqlx::query("INSERT INTO transaction_test (value) VALUES (?)")
            .bind("test_value_1")
            .execute(&mut *tx)
            .await
            .unwrap();
        
        sqlx::query("INSERT INTO transaction_test (value) VALUES (?)")
            .bind("test_value_2")
            .execute(&mut *tx)
            .await
            .unwrap();
        
        // Verify data exists in transaction
        let count_in_tx: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM transaction_test")
            .fetch_one(&mut *tx)
            .await
            .unwrap();
        assert_eq!(count_in_tx.0, 2, "Transaction should see inserted data");
        
        // Rollback the transaction
        tx.rollback().await.unwrap();
        
        // Verify data was not committed
        let count_after_rollback: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM transaction_test")
            .fetch_one(&pool)
            .await
            .unwrap();
        
        assert_eq!(count_after_rollback.0, 0, "Rolled back data should not exist");
        
        pool.close().await;
    }
    
    #[tokio::test]
    async fn test_database_transaction_commit() {
        test_setup::init();
        
        let db_url = test_setup::setup_test_database().await.unwrap();
        let pool = SqlitePool::connect(&db_url).await.unwrap();
        
        // Setup test table
        sqlx::query(r#"
            CREATE TABLE commit_test (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                value TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
        "#).execute(&pool).await.unwrap();
        
        // Test transaction commit
        let mut tx = pool.begin().await.unwrap();
        
        sqlx::query("INSERT INTO commit_test (value) VALUES (?)")
            .bind("committed_value_1")
            .execute(&mut *tx)
            .await
            .unwrap();
        
        sqlx::query("INSERT INTO commit_test (value) VALUES (?)")
            .bind("committed_value_2")
            .execute(&mut *tx)
            .await
            .unwrap();
        
        // Commit the transaction
        tx.commit().await.unwrap();
        
        // Verify data was committed
        let committed_data: Vec<(i64, String)> = sqlx::query_as(
            "SELECT id, value FROM commit_test ORDER BY id"
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        
        assert_eq!(committed_data.len(), 2, "Committed data should exist");
        assert_eq!(committed_data[0].1, "committed_value_1", "First value should be committed");
        assert_eq!(committed_data[1].1, "committed_value_2", "Second value should be committed");
        
        pool.close().await;
    }
    
    #[tokio::test]
    async fn test_database_concurrent_transactions() {
        test_setup::init();
        
        let db_url = test_setup::setup_test_database().await.unwrap();
        let pool = SqlitePool::connect(&db_url).await.unwrap();
        
        // Setup test table
        sqlx::query(r#"
            CREATE TABLE concurrent_test (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                transaction_id TEXT NOT NULL,
                value INTEGER NOT NULL
            )
        "#).execute(&pool).await.unwrap();
        
        // Test concurrent transactions
        let mut handles = Vec::new();
        
        for tx_id in 0..5 {
            let pool_clone = pool.clone();
            let handle = tokio::spawn(async move {
                let transaction_id = format!("tx_{}", tx_id);
                let mut tx = pool_clone.begin().await.unwrap();
                
                // Each transaction inserts 10 records
                for i in 0..10 {
                    sqlx::query("INSERT INTO concurrent_test (transaction_id, value) VALUES (?, ?)")
                        .bind(&transaction_id)
                        .bind(i)
                        .execute(&mut *tx)
                        .await
                        .unwrap();
                    
                    // Small delay to increase chance of concurrency
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
                
                tx.commit().await.unwrap();
                transaction_id
            });
            handles.push(handle);
        }
        
        // Wait for all transactions to complete
        let mut completed_transactions = Vec::new();
        for handle in handles {
            let tx_id = handle.await.unwrap();
            completed_transactions.push(tx_id);
        }
        
        assert_eq!(completed_transactions.len(), 5, "All transactions should complete");
        
        // Verify all data was committed correctly
        let total_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM concurrent_test")
            .fetch_one(&pool)
            .await
            .unwrap();
        
        assert_eq!(total_count.0, 50, "All concurrent transaction data should be committed");
        
        // Verify each transaction's data integrity
        for tx_id in 0..5 {
            let transaction_id = format!("tx_{}", tx_id);
            let tx_count: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM concurrent_test WHERE transaction_id = ?"
            )
            .bind(&transaction_id)
            .fetch_one(&pool)
            .await
            .unwrap();
            
            assert_eq!(tx_count.0, 10, "Each transaction should have 10 records");
        }
        
        pool.close().await;
    }
    
    #[tokio::test]
    async fn test_database_transaction_error_handling() {
        test_setup::init();
        
        let db_url = test_setup::setup_test_database().await.unwrap();
        let pool = SqlitePool::connect(&db_url).await.unwrap();
        
        // Setup test table with constraints
        sqlx::query(r#"
            CREATE TABLE error_test (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                unique_value TEXT UNIQUE NOT NULL,
                positive_number INTEGER CHECK(positive_number > 0)
            )
        "#).execute(&pool).await.unwrap();
        
        // Test transaction with constraint violation
        let mut tx = pool.begin().await.unwrap();
        
        // Insert valid data first
        let valid_insert = sqlx::query(
            "INSERT INTO error_test (unique_value, positive_number) VALUES (?, ?)"
        )
        .bind("unique_1")
        .bind(10)
        .execute(&mut *tx)
        .await;
        assert!(valid_insert.is_ok(), "Valid insert should succeed in transaction");
        
        // Try to insert duplicate unique value (should fail)
        let duplicate_insert = sqlx::query(
            "INSERT INTO error_test (unique_value, positive_number) VALUES (?, ?)"
        )
        .bind("unique_1") // Duplicate
        .bind(20)
        .execute(&mut *tx)
        .await;
        
        assert!(duplicate_insert.is_err(), "Duplicate insert should fail");
        
        // Transaction should be rolled back due to error
        let rollback_result = tx.rollback().await;
        assert!(rollback_result.is_ok(), "Transaction rollback should succeed");
        
        // Verify no data was committed
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM error_test")
            .fetch_one(&pool)
            .await
            .unwrap();
        
        assert_eq!(count.0, 0, "No data should be committed after failed transaction");
        
        pool.close().await;
    }
}

#[cfg(test)]
mod database_performance_tests {
    use super::*;
    use crate::integration::test_setup;
    
    #[tokio::test]
    async fn test_database_query_performance() {
        test_setup::init();
        
        let db_url = test_setup::setup_test_database().await.unwrap();
        let pool = SqlitePool::connect(&db_url).await.unwrap();
        
        // Setup performance test table with indexes
        sqlx::query(r#"
            CREATE TABLE performance_test (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                indexed_field TEXT NOT NULL,
                data_field TEXT NOT NULL,
                number_field INTEGER NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            
            CREATE INDEX idx_performance_indexed_field ON performance_test(indexed_field);
            CREATE INDEX idx_performance_number_field ON performance_test(number_field);
        "#).execute(&pool).await.unwrap();
        
        // Insert test data
        let data_size = 1000;
        for i in 0..data_size {
            sqlx::query(
                "INSERT INTO performance_test (indexed_field, data_field, number_field) VALUES (?, ?, ?)"
            )
            .bind(format!("indexed_{}", i % 100)) // Create some duplicate values for realistic querying
            .bind(format!("data_value_{}", i))
            .bind(i)
            .execute(&pool)
            .await
            .unwrap();
        }
        
        // Test SELECT performance with index
        let start_time = SystemTime::now();
        let indexed_results: Vec<(i64, String)> = sqlx::query_as(
            "SELECT id, data_field FROM performance_test WHERE indexed_field = ? ORDER BY id"
        )
        .bind("indexed_50")
        .fetch_all(&pool)
        .await
        .unwrap();
        
        let indexed_elapsed = start_time.elapsed().unwrap();
        assert!(!indexed_results.is_empty(), "Indexed query should return results");
        assert!(indexed_elapsed.as_millis() < 100, "Indexed query should be fast");
        
        // Test SELECT performance with range query
        let range_start = SystemTime::now();
        let range_results: Vec<(i64,)> = sqlx::query_as(
            "SELECT id FROM performance_test WHERE number_field BETWEEN ? AND ? ORDER BY number_field"
        )
        .bind(100)
        .bind(200)
        .execute(&pool)
        .await
        .unwrap();
        
        let range_elapsed = range_start.elapsed().unwrap();
        assert_eq!(range_results.len(), 101, "Range query should return correct count");
        assert!(range_elapsed.as_millis() < 50, "Range query should be efficient");
        
        // Test aggregate query performance
        let agg_start = SystemTime::now();
        let agg_results: Vec<(String, i64)> = sqlx::query_as(
            "SELECT indexed_field, COUNT(*) FROM performance_test GROUP BY indexed_field ORDER BY COUNT(*) DESC LIMIT 10"
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        
        let agg_elapsed = agg_start.elapsed().unwrap();
        assert_eq!(agg_results.len(), 10, "Aggregate query should return top 10 results");
        assert!(agg_elapsed.as_millis() < 100, "Aggregate query should be reasonably fast");
        
        println!("Query performance - Indexed: {:?}, Range: {:?}, Aggregate: {:?}", 
                indexed_elapsed, range_elapsed, agg_elapsed);
        
        pool.close().await;
    }
    
    #[tokio::test] 
    async fn test_database_connection_scaling() {
        test_setup::init();
        
        let db_url = test_setup::setup_test_database().await.unwrap();
        
        // Test connection pool scaling under load
        let pool = SqlitePool::connect(&db_url).await.unwrap();
        
        sqlx::query("CREATE TABLE scaling_test (id INTEGER PRIMARY KEY, value TEXT)")
            .execute(&pool).await.unwrap();
        
        let concurrent_operations = 20;
        let operations_per_task = 50;
        let mut handles = Vec::new();
        
        let start_time = SystemTime::now();
        
        for task_id in 0..concurrent_operations {
            let pool_clone = pool.clone();
            let handle = tokio::spawn(async move {
                for op_id in 0..operations_per_task {
                    let value = format!("task_{}_op_{}", task_id, op_id);
                    
                    let result = sqlx::query("INSERT INTO scaling_test (value) VALUES (?)")
                        .bind(&value)
                        .execute(&pool_clone)
                        .await;
                    
                    if result.is_err() {
                        return Err(format!("Task {} operation {} failed", task_id, op_id));
                    }
                }
                Ok(task_id)
            });
            handles.push(handle);
        }
        
        // Wait for all tasks to complete
        let mut successful_tasks = 0;
        for handle in handles {
            match handle.await.unwrap() {
                Ok(_) => successful_tasks += 1,
                Err(e) => eprintln!("Task failed: {}", e),
            }
        }
        
        let total_elapsed = start_time.elapsed().unwrap();
        let total_operations = concurrent_operations * operations_per_task;
        let operations_per_second = total_operations as f64 / total_elapsed.as_secs_f64();
        
        assert_eq!(successful_tasks, concurrent_operations, "All concurrent tasks should succeed");
        assert!(operations_per_second > 100.0, "Should achieve reasonable operations per second");
        
        println!("Scaling performance: {:.2} operations/second with {} concurrent connections", 
                operations_per_second, concurrent_operations);
        
        // Verify all data was inserted correctly
        let final_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM scaling_test")
            .fetch_one(&pool).await.unwrap();
        
        assert_eq!(final_count.0, total_operations as i64, "All operations should be committed");
        
        pool.close().await;
    }
}