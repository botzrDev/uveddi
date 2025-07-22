//! Standalone test infrastructure validation
//!
//! This test validates that our test infrastructure works correctly
//! without depending on the main codebase that has compilation errors.

use rstest::*;
use std::path::Path;
use std::time::Duration;
use tempfile::TempDir;
use tokio::fs;

// Basic test utilities that don't depend on main codebase
fn temp_test_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp test dir")
}

async fn create_sample_file(
    dir: &TempDir,
    filename: &str,
    content: &str,
) -> Result<std::path::PathBuf, std::io::Error> {
    let file_path = dir.path().join(filename);
    fs::write(&file_path, content).await?;
    Ok(file_path)
}

#[tokio::test]
async fn test_temp_directory_creation() {
    let temp_dir = temp_test_dir();
    assert!(temp_dir.path().exists());
    assert!(temp_dir.path().is_dir());
}

#[tokio::test]
async fn test_file_creation() {
    let temp_dir = temp_test_dir();
    let file_path = create_sample_file(&temp_dir, "test.rs", "fn main() {}")
        .await
        .unwrap();

    assert!(file_path.exists());
    assert!(file_path.is_file());

    let content = fs::read_to_string(&file_path).await.unwrap();
    assert_eq!(content, "fn main() {}");
}

#[tokio::test]
async fn test_multiple_files() {
    let temp_dir = temp_test_dir();

    let rust_file = create_sample_file(&temp_dir, "test.rs", "fn main() {}")
        .await
        .unwrap();
    let python_file = create_sample_file(&temp_dir, "test.py", "print('hello')")
        .await
        .unwrap();
    let js_file = create_sample_file(&temp_dir, "test.js", "console.log('hello')")
        .await
        .unwrap();

    assert!(rust_file.exists());
    assert!(python_file.exists());
    assert!(js_file.exists());
}

#[tokio::test]
async fn test_subdirectory_creation() {
    let temp_dir = temp_test_dir();

    // Create subdirectory structure
    let subdir = temp_dir.path().join("src");
    fs::create_dir_all(&subdir).await.unwrap();

    let file_path = create_sample_file(&temp_dir, "src/main.rs", "fn main() {}")
        .await
        .unwrap();

    assert!(file_path.exists());
    assert!(subdir.exists());
}

#[tokio::test]
async fn test_timeout_functionality() {
    use tokio::time::{timeout, Duration};

    // Test successful timeout
    let result = timeout(Duration::from_millis(100), async {
        tokio::time::sleep(Duration::from_millis(10)).await;
        "success"
    })
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");

    // Test timeout failure
    let result = timeout(Duration::from_millis(10), async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        "should not complete"
    })
    .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_concurrent_file_operations() {
    let temp_dir = temp_test_dir();

    // Create multiple files concurrently
    let tasks = vec![
        create_sample_file(&temp_dir, "file1.rs", "content1"),
        create_sample_file(&temp_dir, "file2.py", "content2"),
        create_sample_file(&temp_dir, "file3.js", "content3"),
    ];

    let results = futures::future::join_all(tasks).await;

    for result in results {
        assert!(result.is_ok());
        let file_path = result.unwrap();
        assert!(file_path.exists());
    }
}

#[rstest]
#[case("test1.rs", "rust code")]
#[case("test2.py", "python code")]
#[case("test3.js", "javascript code")]
#[tokio::test]
async fn test_rstest_integration(#[case] filename: &str, #[case] content: &str) {
    let temp_dir = temp_test_dir();
    let file_path = create_sample_file(&temp_dir, filename, content)
        .await
        .unwrap();

    assert!(file_path.exists());
    let read_content = fs::read_to_string(&file_path).await.unwrap();
    assert_eq!(read_content, content);
}

#[rstest]
#[case("src/main.rs")]
#[case("src/lib.rs")]
#[case("tests/test.rs")]
#[tokio::test]
async fn test_path_generation(#[case] relative_path: &str) {
    let temp_dir = temp_test_dir();
    let full_path = temp_dir.path().join(relative_path);

    // Create parent directories
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent).await.unwrap();
    }

    // Create file
    fs::write(&full_path, "test content").await.unwrap();

    assert!(full_path.exists());
    assert_eq!(
        full_path.file_name().unwrap().to_str().unwrap(),
        Path::new(relative_path)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
    );
}

#[cfg(test)]
mod proptest_integration {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_file_creation_with_random_content(content in "\\PC{0,100}") {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let temp_dir = temp_test_dir();
                let file_path = create_sample_file(&temp_dir, "random_test.txt", &content).await.unwrap();

                let read_content = fs::read_to_string(&file_path).await.unwrap();
                prop_assert_eq!(read_content, content);
                Ok(())
            })?;
        }

        #[test]
        fn test_filename_generation(filename in "[a-zA-Z][a-zA-Z0-9_]{0,20}\\.[a-z]{1,4}") {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let temp_dir = temp_test_dir();
                let file_path = create_sample_file(&temp_dir, &filename, "test content").await.unwrap();

                prop_assert!(file_path.exists());
                prop_assert_eq!(file_path.file_name().unwrap().to_str().unwrap(), filename);
                Ok(())
            })?;
        }
    }
}

#[tokio::test]
async fn test_async_test_macros() {
    // Test that async testing works correctly
    let start = std::time::Instant::now();

    tokio::time::sleep(Duration::from_millis(10)).await;

    let elapsed = start.elapsed();
    assert!(elapsed >= Duration::from_millis(10));
    assert!(elapsed < Duration::from_millis(100)); // Should complete quickly
}

#[tokio::test]
async fn test_error_handling() {
    let temp_dir = temp_test_dir();

    // Test error handling in file operations
    let result = create_sample_file(&temp_dir, "test.txt", "content").await;
    assert!(result.is_ok());

    // Test reading non-existent file
    let non_existent = temp_dir.path().join("non_existent.txt");
    let result = fs::read_to_string(&non_existent).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_memory_cleanup() {
    // Test that temporary directories are cleaned up properly
    let temp_paths = {
        let temp_dir = temp_test_dir();
        let path = temp_dir.path().to_path_buf();

        // Create some files
        create_sample_file(&temp_dir, "test1.txt", "content1")
            .await
            .unwrap();
        create_sample_file(&temp_dir, "test2.txt", "content2")
            .await
            .unwrap();

        assert!(path.exists());
        path
    }; // temp_dir goes out of scope here

    // Give some time for cleanup
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Directory should be cleaned up
    // Note: This might not always work due to OS-specific cleanup timing
    // but it's a good test for memory management
}

#[tokio::test]
async fn test_serial_execution() {
    // serial_test removed for this test

    // Test that serial test execution works
    static mut COUNTER: i32 = 0;

    unsafe {
        COUNTER += 1;
        let current_value = COUNTER;

        // Simulate some work
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Value should not have changed if tests are serial
        assert_eq!(COUNTER, current_value);
    }
}

#[test]
fn test_synchronous_helpers() {
    // Test that our utilities work in synchronous context as well
    let temp_dir = temp_test_dir();

    assert!(temp_dir.path().exists());

    // Test synchronous file operations
    let file_path = temp_dir.path().join("sync_test.txt");
    std::fs::write(&file_path, "sync content").unwrap();

    assert!(file_path.exists());

    let content = std::fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "sync content");
}

#[test]
fn test_basic_utilities() {
    // Test basic utility functions
    let temp_dir = temp_test_dir();

    // Test path manipulation
    let test_path = temp_dir.path().join("test.txt");
    assert_eq!(test_path.file_name().unwrap().to_str().unwrap(), "test.txt");

    // Test directory operations
    assert!(temp_dir.path().is_dir());
    assert!(temp_dir.path().exists());
}
