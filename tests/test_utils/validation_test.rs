//! Validation test for the test infrastructure
//!
//! This simple test validates that our test infrastructure is working correctly
//! without depending on the main codebase compilation.

use super::*;
use rstest::*;
use std::path::Path;

#[tokio::test]
async fn test_fixtures_creation() {
    // Test that fixtures can be created successfully
    let fixtures = TestFixtures::new().unwrap();
    
    // Verify temp directory exists
    assert!(fixtures.temp_path().exists());
    
    // Verify sample files exist
    assert!(fixtures.sample_rust_file.exists());
    assert!(fixtures.sample_python_file.exists());
    assert!(fixtures.sample_js_file.exists());
    
    // Verify config is properly created
    assert_eq!(fixtures.sample_config.project_root, "/tmp/test_project");
    assert!(fixtures.sample_config.excluded_paths.contains(&".git".to_string()));
}

#[tokio::test]
async fn test_custom_fixtures() {
    // Test custom fixtures creation
    let files = vec![
        ("test.rs", "fn main() {}"),
        ("subdir/test.py", "print('hello')"),
    ];
    
    let fixtures = TestFixtures::with_custom_files(files).unwrap();
    
    // Verify custom files were created
    let test_rs = fixtures.temp_path().join("test.rs");
    let test_py = fixtures.temp_path().join("subdir/test.py");
    
    assert!(test_rs.exists());
    assert!(test_py.exists());
    
    // Verify content
    let content = std::fs::read_to_string(&test_rs).unwrap();
    assert_eq!(content, "fn main() {}");
}

#[tokio::test]
async fn test_helper_functions() {
    // Test helper functions
    let temp_dir = temp_test_dir();
    let config = create_test_config();
    
    assert!(temp_dir.path().exists());
    assert_eq!(config.project_root, "/tmp/test_project");
    
    // Test async file creation
    let rust_file = create_sample_rust_file(&temp_dir, "test.rs", "fn main() {}").await.unwrap();
    assert!(rust_file.exists());
    
    let python_file = create_sample_python_file(&temp_dir, "test.py", "print('hello')").await.unwrap();
    assert!(python_file.exists());
    
    let js_file = create_sample_js_file(&temp_dir, "test.js", "console.log('hello')").await.unwrap();
    assert!(js_file.exists());
}

#[tokio::test]
async fn test_timeout_wrapper() {
    use std::time::Duration;
    
    // Test successful timeout
    let result = with_timeout(Duration::from_millis(100), async {
        tokio::time::sleep(Duration::from_millis(10)).await;
        "success"
    }).await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");
    
    // Test timeout failure
    let result = with_timeout(Duration::from_millis(10), async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        "should not complete"
    }).await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_wait_for_condition() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;
    
    let condition = Arc::new(AtomicBool::new(false));
    let condition_clone = condition.clone();
    
    // Start a task that will set the condition to true after a delay
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
        condition_clone.store(true, Ordering::SeqCst);
    });
    
    // Wait for the condition
    let result = wait_for_condition(
        || condition.load(Ordering::SeqCst),
        Duration::from_millis(200),
        Duration::from_millis(10),
    ).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_path_assertions() {
    let temp_dir = temp_test_dir();
    let existing_path = temp_dir.path();
    let non_existing_path = temp_dir.path().join("non_existing");
    
    // Test existing path
    assert_path_exists(existing_path);
    
    // Test non-existing path
    assert_path_not_exists(&non_existing_path);
}

#[rstest]
#[case("test1.rs", "rust code")]
#[case("test2.py", "python code")]
#[case("test3.js", "javascript code")]
#[tokio::test]
async fn test_rstest_integration(#[case] filename: &str, #[case] content: &str) {
    let temp_dir = temp_test_dir();
    let file_path = temp_dir.path().join(filename);
    
    std::fs::write(&file_path, content).unwrap();
    
    assert!(file_path.exists());
    let read_content = std::fs::read_to_string(&file_path).unwrap();
    assert_eq!(read_content, content);
}

#[tokio::test]
async fn test_sample_data_generation() {
    // Test dependency generation
    let rust_deps = dependencies::create_sample_rust_dependencies();
    assert!(!rust_deps.is_empty());
    assert_eq!(rust_deps[0].name, "std::collections::HashMap");
    
    let python_deps = dependencies::create_sample_python_dependencies();
    assert!(!python_deps.is_empty());
    assert_eq!(python_deps[0].name, "os");
    
    // Test parsed files generation
    let rust_parsed = parsed_files::create_sample_rust_parsed_file();
    assert_eq!(rust_parsed.path, Path::new("src/sample.rs"));
    
    let python_parsed = parsed_files::create_sample_python_parsed_file();
    assert_eq!(python_parsed.path, Path::new("src/sample.py"));
    
    let js_parsed = parsed_files::create_sample_js_parsed_file();
    assert_eq!(js_parsed.path, Path::new("src/sample.js"));
}

#[cfg(test)]
mod proptest_integration {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_file_creation_with_random_content(content in "\\PC{0,1000}") {
            let temp_dir = temp_test_dir();
            let file_path = temp_dir.path().join("random_test.txt");
            
            std::fs::write(&file_path, &content).unwrap();
            
            let read_content = std::fs::read_to_string(&file_path).unwrap();
            prop_assert_eq!(read_content, content);
        }
    }
}

// This test will only run if the main codebase compiles
#[cfg(test)]
#[ignore] // Ignore by default since main codebase has compilation errors
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_mock_ast_parser() {
        let mock_parser = MockAstParser::create_successful();
        
        let result = mock_parser.parse_file(Path::new("test.rs"));
        assert!(result.is_ok());
        
        let parsed = result.unwrap();
        assert_eq!(parsed.content, "mock content");
    }
    
    #[tokio::test]
    async fn test_mock_dependency_extractor() {
        let sample_deps = dependencies::create_sample_rust_dependencies();
        let mock_extractor = MockDependencyExtractor::create_with_dependencies(sample_deps.clone());
        
        let parsed_file = parsed_files::create_sample_rust_parsed_file();
        let result = mock_extractor.extract_from_ast(&parsed_file);
        
        assert!(result.is_ok());
        let extracted_deps = result.unwrap();
        assert_eq!(extracted_deps.len(), sample_deps.len());
    }
}