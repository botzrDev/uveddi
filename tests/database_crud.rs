use uveddi::database::crud::Database;
use uveddi::database::models::AntiPatternType;
use std::path::Path;
use std::fs;

fn cleanup_database() {
    // Clean up main database files that might be created during tests
    if Path::new("uveddi.db").exists() {
        let _ = fs::remove_file("uveddi.db");
    }
}

#[test]
fn test_database_creation() {
    cleanup_database();
    let _db = Database::new();
    assert!(_db.is_ok());
    cleanup_database();
}

#[test]
fn test_get_or_create_project_id_new_project() {
    cleanup_database();
    let db = Database::new().unwrap();
    
    let project_path = Path::new("/test/project/path");
    let project_id = db.get_or_create_project_id(project_path);
    
    assert!(project_id.is_ok());
    let id = project_id.unwrap();
    assert!(id > 0);
    
    cleanup_database();
}

#[test]
fn test_create_analysis_run() {
    cleanup_database();
    let db = Database::new().unwrap();
    
    let project_path = Path::new("/test/project");
    let analysis_run = db.create_analysis_run(project_path);
    
    assert!(analysis_run.is_ok());
    let run = analysis_run.unwrap();
    
    assert!(run.run_id.is_some());
    assert!(run.project_id > 0);
    assert_eq!(run.status, "running");
    assert_eq!(run.analysis_config, "{}");
    assert!(run.end_time.is_none());
    
    cleanup_database();
}

#[test]
fn test_store_anti_pattern_type() {
    cleanup_database();
    let db = Database::new().unwrap();
    
    let mut anti_pattern_type = AntiPatternType {
        anti_pattern_type_id: None,
        name: "Test Pattern".to_string(),
        description: "A test anti-pattern".to_string(),
        category: "Testing".to_string(),
    };
    
    let result = db.store_anti_pattern_type(&mut anti_pattern_type);
    assert!(result.is_ok());
    assert!(anti_pattern_type.anti_pattern_type_id.is_some());
    
    cleanup_database();
}
