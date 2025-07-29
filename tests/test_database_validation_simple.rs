//! Simple test for database input validation integration

use tempfile::TempDir;
use uveddi::database::models::{AntiPatternType, ArchitecturalIssue};
use uveddi::database::Database;

#[test]
fn test_database_validation_with_valid_data() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let mut db = Database::new(Some(&db_path)).unwrap();

    // Create a test analysis run first
    let analysis_run = db
        .create_analysis_run(std::path::Path::new("./test"))
        .unwrap();
    let analysis_run_id = analysis_run.run_id.unwrap();

    // Create a test anti-pattern type
    let mut anti_pattern_type = AntiPatternType {
        anti_pattern_type_id: None,
        name: "test-pattern".to_string(),
        description: "Test pattern for validation".to_string(),
        category: "test".to_string(),
    };
    db.store_anti_pattern_type(&mut anti_pattern_type).unwrap();
    let anti_pattern_type_id = anti_pattern_type.anti_pattern_type_id.unwrap();

    // Test valid insertion should work
    let valid_issues = vec![ArchitecturalIssue {
        issue_id: None,
        analysis_run_id,
        anti_pattern_type_id,
        file_path: "src/main.rs".to_string(),
        start_line: Some(10),
        end_line: Some(20),
        severity: "medium".to_string(),
        description: "This is a safe message".to_string(),
        code_snippet: Some("fn main() { }".to_string()),
        ai_explanation: Some("This is a valid explanation".to_string()),
    }];

    let result = db.store_issues(&valid_issues);
    assert!(
        result.is_ok(),
        "Valid issues should be stored successfully: {:?}",
        result.err()
    );
}

#[test]
fn test_database_validation_with_malicious_sql() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let mut db = Database::new(Some(&db_path)).unwrap();

    // Create test data
    let analysis_run = db
        .create_analysis_run(std::path::Path::new("./test"))
        .unwrap();
    let analysis_run_id = analysis_run.run_id.unwrap();
    let mut anti_pattern_type = AntiPatternType {
        anti_pattern_type_id: None,
        name: "test-pattern".to_string(),
        description: "Test pattern for validation".to_string(),
        category: "test".to_string(),
    };
    db.store_anti_pattern_type(&mut anti_pattern_type).unwrap();
    let anti_pattern_type_id = anti_pattern_type.anti_pattern_type_id.unwrap();

    // Test SQL injection prevention in database
    let malicious_issues = vec![ArchitecturalIssue {
        issue_id: None,
        analysis_run_id,
        anti_pattern_type_id,
        file_path: "'; DROP TABLE users; --".to_string(),
        start_line: Some(10),
        end_line: Some(20),
        severity: "high".to_string(),
        description: "'; DELETE FROM analysis_runs; --".to_string(),
        code_snippet: Some("'; EXEC xp_cmdshell('malicious'); --".to_string()),
        ai_explanation: Some("' UNION SELECT * FROM secrets --".to_string()),
    }];

    let result = db.store_issues(&malicious_issues);
    assert!(result.is_err(), "Malicious inputs should be rejected");

    // Verify the error is a security error
    match result.unwrap_err() {
        uveddi::error::UveddiError::SecurityError { .. } => {
            // Expected security error
        }
        _ => panic!("Expected SecurityError for malicious input"),
    }
}

#[test]
fn test_database_validation_with_invalid_ranges() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let mut db = Database::new(Some(&db_path)).unwrap();

    // Create test data
    let analysis_run = db
        .create_analysis_run(std::path::Path::new("./test"))
        .unwrap();
    let analysis_run_id = analysis_run.run_id.unwrap();
    let mut anti_pattern_type = AntiPatternType {
        anti_pattern_type_id: None,
        name: "test-pattern".to_string(),
        description: "Test pattern for validation".to_string(),
        category: "test".to_string(),
    };
    db.store_anti_pattern_type(&mut anti_pattern_type).unwrap();
    let anti_pattern_type_id = anti_pattern_type.anti_pattern_type_id.unwrap();

    // Test invalid line number ranges
    let invalid_issues = vec![ArchitecturalIssue {
        issue_id: None,
        analysis_run_id,
        anti_pattern_type_id,
        file_path: "src/main.rs".to_string(),
        start_line: Some(-1), // Invalid line number
        end_line: Some(20),
        severity: "medium".to_string(),
        description: "This is a safe message".to_string(),
        code_snippet: Some("fn main() { }".to_string()),
        ai_explanation: Some("This is a valid explanation".to_string()),
    }];

    let result = db.store_issues(&invalid_issues);
    assert!(result.is_err(), "Invalid line numbers should be rejected");
}
