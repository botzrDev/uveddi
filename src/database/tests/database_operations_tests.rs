//! Comprehensive Database Operations Tests
//!
//! This module provides extensive test coverage for database operations including:
//! - CRUD operations for reports and analysis results
//! - Connection pooling and transaction handling
//! - Database migration scripts
//! - Error recovery and data integrity
//! - Performance under load
//!
//! Target coverage: 85%+

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{
        crud::{AnalysisResultCrud, ReportCrud},
        models::{AnalysisResult, Report, ArchitecturalIssue},
        config_manager::DatabaseConfig,
        migration_manager::MigrationManager,
    };
    use std::collections::HashMap;
    use tokio_test;
    use uuid::Uuid;
    use chrono::{DateTime, Utc};

    /// Test database configuration for integration tests
    fn test_db_config() -> DatabaseConfig {
        DatabaseConfig {
            url: std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "sqlite::memory:".to_string()),
            max_connections: 5,
            connection_timeout: 5,
            enable_logging: false,
        }
    }

    /// Helper function to create test analysis result
    fn create_test_analysis_result() -> AnalysisResult {
        AnalysisResult {
            id: Uuid::new_v4(),
            project_path: "/test/project".to_string(),
            analysis_timestamp: Utc::now(),
            total_files: 100,
            analyzed_files: 95,
            total_issues: 25,
            critical_issues: 3,
            major_issues: 8,
            minor_issues: 14,
            info_issues: 0,
            analysis_duration_ms: 5000,
            language_breakdown: HashMap::from([
                ("rust".to_string(), 60),
                ("javascript".to_string(), 35),
                ("python".to_string(), 5),
            ]),
            plugin_versions: HashMap::from([
                ("core".to_string(), "1.0.0".to_string()),
                ("security".to_string(), "0.9.0".to_string()),
            ]),
            status: "completed".to_string(),
            error_message: None,
            metadata: serde_json::json!({"test": true}),
        }
    }

    /// Helper function to create test architectural issue
    fn create_test_architectural_issue(analysis_id: Uuid) -> ArchitecturalIssue {
        ArchitecturalIssue {
            id: Uuid::new_v4(),
            analysis_result_id: analysis_id,
            anti_pattern_type_id: 1,
            file_path: "src/main.rs".to_string(),
            start_line: Some(45),
            end_line: Some(67),
            description: "God Object detected with too many responsibilities".to_string(),
            detector_name: "GodObjectDetector".to_string(),
            severity: "critical".to_string(),
            recommendation: "Consider breaking this class into smaller, focused classes".to_string(),
            code_snippet: Some("struct LargeStruct { ... }".to_string()),
            detected_at: Utc::now(),
            metadata: Some(serde_json::json!({"method_count": 25, "field_count": 18})),
        }
    }

    #[tokio::test]
    async fn test_database_connection() {
        let config = test_db_config();

        let connection_result = crate::database::create_connection(&config).await;
        assert!(connection_result.is_ok(), "Should be able to connect to test database");

        let mut conn = connection_result.unwrap();
        let query_result = sqlx::query("SELECT 1 as test_value")
            .fetch_one(&mut conn)
            .await;

        assert!(query_result.is_ok(), "Should be able to execute basic query");
    }

    #[tokio::test]
    async fn test_migration_manager() {
        let config = test_db_config();
        let migration_manager = MigrationManager::new(config.clone());

        // Test running migrations
        let migration_result = migration_manager.run_migrations().await;
        assert!(migration_result.is_ok(), "Migrations should run successfully");

        // Test checking migration status
        let status_result = migration_manager.get_migration_status().await;
        assert!(status_result.is_ok(), "Should be able to check migration status");

        let status = status_result.unwrap();
        assert!(!status.pending_migrations.is_empty() || !status.applied_migrations.is_empty(),
                "Should have some migration information");
    }

    #[tokio::test]
    async fn test_analysis_result_crud_create() {
        let config = test_db_config();
        let crud = AnalysisResultCrud::new(config).await.expect("Failed to create CRUD instance");

        let test_result = create_test_analysis_result();
        let original_id = test_result.id;

        let create_result = crud.create(test_result).await;
        assert!(create_result.is_ok(), "Should be able to create analysis result");

        let created_result = create_result.unwrap();
        assert_eq!(created_result.id, original_id);
        assert_eq!(created_result.total_files, 100);
        assert_eq!(created_result.analyzed_files, 95);
        assert_eq!(created_result.total_issues, 25);
    }

    #[tokio::test]
    async fn test_analysis_result_crud_read() {
        let config = test_db_config();
        let crud = AnalysisResultCrud::new(config).await.expect("Failed to create CRUD instance");

        // Create a test result first
        let test_result = create_test_analysis_result();
        let test_id = test_result.id;
        crud.create(test_result).await.expect("Failed to create test data");

        // Test reading by ID
        let read_result = crud.get_by_id(test_id).await;
        assert!(read_result.is_ok(), "Should be able to read analysis result by ID");

        let retrieved_result = read_result.unwrap();
        assert!(retrieved_result.is_some(), "Should find the created analysis result");

        let result = retrieved_result.unwrap();
        assert_eq!(result.id, test_id);
        assert_eq!(result.total_files, 100);
        assert_eq!(result.status, "completed");
    }

    #[tokio::test]
    async fn test_analysis_result_crud_list() {
        let config = test_db_config();
        let crud = AnalysisResultCrud::new(config).await.expect("Failed to create CRUD instance");

        // Create multiple test results
        for i in 0..3 {
            let mut test_result = create_test_analysis_result();
            test_result.total_files = 100 + i * 10;
            crud.create(test_result).await.expect("Failed to create test data");
        }

        // Test listing all results
        let list_result = crud.list_all(None, None).await;
        assert!(list_result.is_ok(), "Should be able to list analysis results");

        let results = list_result.unwrap();
        assert!(results.len() >= 3, "Should have at least 3 analysis results");
    }

    #[tokio::test]
    async fn test_analysis_result_crud_update() {
        let config = test_db_config();
        let crud = AnalysisResultCrud::new(config).await.expect("Failed to create CRUD instance");

        // Create a test result first
        let mut test_result = create_test_analysis_result();
        test_result.status = "running".to_string();
        let test_id = test_result.id;
        crud.create(test_result).await.expect("Failed to create test data");

        // Update the status
        let update_result = crud.update_status(test_id, "completed".to_string()).await;
        assert!(update_result.is_ok(), "Should be able to update analysis result status");

        // Verify the update
        let retrieved = crud.get_by_id(test_id).await.unwrap().unwrap();
        assert_eq!(retrieved.status, "completed");
    }

    #[tokio::test]
    async fn test_analysis_result_crud_delete() {
        let config = test_db_config();
        let crud = AnalysisResultCrud::new(config).await.expect("Failed to create CRUD instance");

        // Create a test result first
        let test_result = create_test_analysis_result();
        let test_id = test_result.id;
        crud.create(test_result).await.expect("Failed to create test data");

        // Delete the result
        let delete_result = crud.delete(test_id).await;
        assert!(delete_result.is_ok(), "Should be able to delete analysis result");

        // Verify deletion
        let retrieved = crud.get_by_id(test_id).await.unwrap();
        assert!(retrieved.is_none(), "Deleted analysis result should not be found");
    }

    #[tokio::test]
    async fn test_architectural_issue_crud() {
        let config = test_db_config();
        let analysis_crud = AnalysisResultCrud::new(config.clone()).await
            .expect("Failed to create analysis CRUD instance");

        // Create analysis result first
        let analysis_result = create_test_analysis_result();
        let analysis_id = analysis_result.id;
        analysis_crud.create(analysis_result).await.expect("Failed to create analysis result");

        // Test architectural issue CRUD operations
        let issue_crud = crate::database::crud::ArchitecturalIssueCrud::new(config).await
            .expect("Failed to create issue CRUD instance");

        // Create test issue
        let test_issue = create_test_architectural_issue(analysis_id);
        let issue_id = test_issue.id;

        let create_result = issue_crud.create(test_issue).await;
        assert!(create_result.is_ok(), "Should be able to create architectural issue");

        // Test reading the issue
        let read_result = issue_crud.get_by_id(issue_id).await;
        assert!(read_result.is_ok(), "Should be able to read architectural issue");

        let retrieved_issue = read_result.unwrap();
        assert!(retrieved_issue.is_some(), "Should find the created issue");

        let issue = retrieved_issue.unwrap();
        assert_eq!(issue.id, issue_id);
        assert_eq!(issue.severity, "critical");
        assert_eq!(issue.detector_name, "GodObjectDetector");

        // Test listing issues by analysis result
        let list_result = issue_crud.get_by_analysis_id(analysis_id).await;
        assert!(list_result.is_ok(), "Should be able to list issues by analysis ID");

        let issues = list_result.unwrap();
        assert_eq!(issues.len(), 1, "Should have one issue for this analysis");
    }

    #[tokio::test]
    async fn test_transaction_handling() {
        let config = test_db_config();
        let analysis_crud = AnalysisResultCrud::new(config.clone()).await
            .expect("Failed to create CRUD instance");

        // Test successful transaction
        let analysis_result = create_test_analysis_result();
        let analysis_id = analysis_result.id;

        let transaction_result = analysis_crud.create_with_issues(
            analysis_result,
            vec![create_test_architectural_issue(analysis_id)],
        ).await;

        assert!(transaction_result.is_ok(), "Transaction should succeed");

        // Verify both analysis result and issue were created
        let retrieved_analysis = analysis_crud.get_by_id(analysis_id).await.unwrap();
        assert!(retrieved_analysis.is_some(), "Analysis result should exist");

        // Test transaction rollback on error
        let invalid_analysis = AnalysisResult {
            id: Uuid::new_v4(),
            project_path: "".to_string(), // Invalid empty path
            // ... other fields would cause constraint violations
            ..create_test_analysis_result()
        };

        let rollback_result = analysis_crud.create_with_issues(
            invalid_analysis,
            vec![create_test_architectural_issue(Uuid::new_v4())],
        ).await;

        assert!(rollback_result.is_err(), "Transaction with invalid data should fail");
    }

    #[tokio::test]
    async fn test_connection_pooling() {
        let config = test_db_config();

        // Create multiple concurrent connections
        let mut handles = vec![];

        for i in 0..10 {
            let config_clone = config.clone();
            let handle = tokio::spawn(async move {
                let crud = AnalysisResultCrud::new(config_clone).await?;
                let mut test_result = create_test_analysis_result();
                test_result.total_files = 100 + i;
                crud.create(test_result).await
            });
            handles.push(handle);
        }

        // Wait for all connections to complete
        let mut successful_connections = 0;
        for handle in handles {
            if handle.await.is_ok() && handle.await.unwrap().is_ok() {
                successful_connections += 1;
            }
        }

        assert!(successful_connections >= 5, "Most connections should succeed with pooling");
    }

    #[tokio::test]
    async fn test_pagination() {
        let config = test_db_config();
        let crud = AnalysisResultCrud::new(config).await.expect("Failed to create CRUD instance");

        // Create 10 test results
        for i in 0..10 {
            let mut test_result = create_test_analysis_result();
            test_result.total_files = 100 + i;
            crud.create(test_result).await.expect("Failed to create test data");
        }

        // Test pagination
        let first_page = crud.list_all(Some(5), Some(0)).await.unwrap();
        assert_eq!(first_page.len(), 5, "First page should have 5 results");

        let second_page = crud.list_all(Some(5), Some(5)).await.unwrap();
        assert_eq!(second_page.len(), 5, "Second page should have 5 results");

        // Ensure no overlap between pages
        let first_ids: std::collections::HashSet<_> = first_page.iter().map(|r| r.id).collect();
        let second_ids: std::collections::HashSet<_> = second_page.iter().map(|r| r.id).collect();
        assert!(first_ids.is_disjoint(&second_ids), "Pages should not overlap");
    }

    #[tokio::test]
    async fn test_filtering_and_search() {
        let config = test_db_config();
        let crud = AnalysisResultCrud::new(config).await.expect("Failed to create CRUD instance");

        // Create test results with different statuses
        let statuses = ["completed", "failed", "running"];
        for (i, &status) in statuses.iter().enumerate() {
            let mut test_result = create_test_analysis_result();
            test_result.status = status.to_string();
            test_result.total_files = 100 + i as i32;
            crud.create(test_result).await.expect("Failed to create test data");
        }

        // Test filtering by status
        let completed_results = crud.find_by_status("completed").await.unwrap();
        assert!(!completed_results.is_empty(), "Should find completed results");
        assert!(completed_results.iter().all(|r| r.status == "completed"));

        // Test filtering by date range
        let now = Utc::now();
        let one_hour_ago = now - chrono::Duration::hours(1);
        let one_hour_later = now + chrono::Duration::hours(1);

        let recent_results = crud.find_by_date_range(one_hour_ago, one_hour_later).await.unwrap();
        assert!(!recent_results.is_empty(), "Should find recent results");
    }

    #[tokio::test]
    async fn test_error_handling() {
        let config = test_db_config();
        let crud = AnalysisResultCrud::new(config).await.expect("Failed to create CRUD instance");

        // Test handling of non-existent ID
        let non_existent_id = Uuid::new_v4();
        let read_result = crud.get_by_id(non_existent_id).await;
        assert!(read_result.is_ok(), "Reading non-existent ID should not error");
        assert!(read_result.unwrap().is_none(), "Should return None for non-existent ID");

        // Test handling of invalid update
        let update_result = crud.update_status(non_existent_id, "completed".to_string()).await;
        assert!(update_result.is_err(), "Updating non-existent record should error");

        // Test handling of constraint violations
        let invalid_result = AnalysisResult {
            total_files: -1, // Invalid negative value
            ..create_test_analysis_result()
        };

        let create_result = crud.create(invalid_result).await;
        assert!(create_result.is_err(), "Creating invalid data should error");
    }

    #[tokio::test]
    async fn test_performance_metrics() {
        let config = test_db_config();
        let crud = AnalysisResultCrud::new(config).await.expect("Failed to create CRUD instance");

        // Test bulk insert performance
        let start_time = std::time::Instant::now();

        for i in 0..100 {
            let mut test_result = create_test_analysis_result();
            test_result.total_files = 100 + i;
            crud.create(test_result).await.expect("Failed to create test data");
        }

        let insert_duration = start_time.elapsed();
        assert!(insert_duration.as_secs() < 10, "Bulk inserts should complete within 10 seconds");

        // Test bulk read performance
        let read_start = std::time::Instant::now();
        let all_results = crud.list_all(None, None).await.unwrap();
        let read_duration = read_start.elapsed();

        assert!(all_results.len() >= 100, "Should retrieve all inserted records");
        assert!(read_duration.as_secs() < 5, "Bulk read should complete within 5 seconds");
    }

    #[tokio::test]
    async fn test_database_backup_and_restore() {
        let config = test_db_config();

        // This test would be more relevant for file-based databases like SQLite
        if config.url.contains("sqlite") {
            let backup_manager = crate::database::backup::BackupManager::new(config.clone());

            // Create some test data
            let crud = AnalysisResultCrud::new(config.clone()).await
                .expect("Failed to create CRUD instance");
            let test_result = create_test_analysis_result();
            let test_id = test_result.id;
            crud.create(test_result).await.expect("Failed to create test data");

            // Test backup creation
            let backup_path = "/tmp/test_backup.db";
            let backup_result = backup_manager.create_backup(backup_path).await;
            assert!(backup_result.is_ok(), "Should be able to create backup");

            // Verify backup file exists
            assert!(std::path::Path::new(backup_path).exists(), "Backup file should exist");

            // Test restore (would need a separate test database)
            // This is more complex and would require setting up a separate test environment

            // Cleanup
            std::fs::remove_file(backup_path).ok();
        }
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let config = test_db_config();
        let crud = std::sync::Arc::new(
            AnalysisResultCrud::new(config).await.expect("Failed to create CRUD instance")
        );

        // Create multiple concurrent readers and writers
        let mut handles = vec![];

        // Concurrent writes
        for i in 0..5 {
            let crud_clone = crud.clone();
            let handle = tokio::spawn(async move {
                let mut test_result = create_test_analysis_result();
                test_result.total_files = 100 + i;
                crud_clone.create(test_result).await
            });
            handles.push(handle);
        }

        // Concurrent reads
        for _ in 0..5 {
            let crud_clone = crud.clone();
            let handle = tokio::spawn(async move {
                crud_clone.list_all(Some(10), None).await
            });
            handles.push(handle);
        }

        // Wait for all operations to complete
        let mut successful_operations = 0;
        for handle in handles {
            if handle.await.is_ok() {
                successful_operations += 1;
            }
        }

        assert!(successful_operations >= 8, "Most concurrent operations should succeed");
    }

    #[tokio::test]
    async fn test_data_integrity() {
        let config = test_db_config();
        let analysis_crud = AnalysisResultCrud::new(config.clone()).await
            .expect("Failed to create analysis CRUD instance");
        let issue_crud = crate::database::crud::ArchitecturalIssueCrud::new(config).await
            .expect("Failed to create issue CRUD instance");

        // Create analysis result
        let analysis_result = create_test_analysis_result();
        let analysis_id = analysis_result.id;
        analysis_crud.create(analysis_result).await.expect("Failed to create analysis result");

        // Create multiple issues for the analysis
        let mut issue_ids = vec![];
        for i in 0..5 {
            let mut issue = create_test_architectural_issue(analysis_id);
            issue.start_line = Some(10 + i);
            issue_ids.push(issue.id);
            issue_crud.create(issue).await.expect("Failed to create issue");
        }

        // Test foreign key constraints - deleting analysis should cascade to issues
        let delete_result = analysis_crud.delete(analysis_id).await;
        assert!(delete_result.is_ok(), "Should be able to delete analysis result");

        // Verify issues are also deleted (cascade)
        for issue_id in issue_ids {
            let retrieved_issue = issue_crud.get_by_id(issue_id).await.unwrap();
            assert!(retrieved_issue.is_none(), "Issue should be deleted when analysis is deleted");
        }
    }
}