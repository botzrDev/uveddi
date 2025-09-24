// Integration test module for Assignment 05 Migration & Error Handling

pub mod migration_tests;

// Common integration test utilities
pub mod test_setup {
    use std::sync::Once;
    use tempfile::TempDir;
    use std::path::PathBuf;
    
    static INIT: Once = Once::new();
    
    /// Initialize integration test environment
    pub fn init() {
        INIT.call_once(|| {
            // Initialize logging for integration tests
            let _ = tracing_subscriber::fmt::try_init();
        });
    }
    
    /// Create a temporary test directory
    pub fn create_test_dir() -> (TempDir, PathBuf) {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let path = temp_dir.path().to_path_buf();
        (temp_dir, path)
    }
    
    /// Setup test database for integration tests
    pub async fn setup_test_database() -> Result<String, Box<dyn std::error::Error>> {
        // This would set up a test database connection
        // For now, return a mock connection string
        Ok("sqlite::memory:".to_string())
    }
}