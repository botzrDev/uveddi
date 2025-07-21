// Unit test module for UV-243 Testing Infrastructure

pub mod core_analysis;
pub mod monitoring_comprehensive;

// Common test utilities for unit tests
pub mod test_utils {
    use std::sync::Once;
    
    static INIT: Once = Once::new();
    
    /// Initialize test environment once
    pub fn init() {
        INIT.call_once(|| {
            // Initialize logging for tests if needed
            let _ = env_logger::try_init();
        });
    }
}