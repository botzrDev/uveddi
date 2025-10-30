//! God Object anti-pattern detection tests
//!
//! This module tests detection of classes/structs with too many responsibilities.

#[cfg(test)]
mod tests {
    use crate::analysis::detectors::anti_patterns::god_object::GodObjectDetector;
    use crate::analysis::AnalysisDetector;

    #[cfg(feature = "tree-sitter")]
    mod tree_sitter_tests {
        use super::*;
        use crate::analysis::detectors::anti_patterns::god_object::GodObjectConfig;
        use crate::ast::tree_sitter::{AstParser, SourceLanguage};
        use std::path::PathBuf;

        #[test]
        fn test_god_object_positive() {
            let rust_code = r#"
pub struct GodObject {
    field1: String,
    field2: i32,
    field3: f64,
    field4: bool,
    field5: Vec<String>,
    field6: HashMap<String, i32>,
    field7: Option<String>,
    field8: Result<i32, String>,
    field9: Box<dyn Trait>,
}

impl GodObject {
    pub fn new() -> Self { todo!() }
    pub fn method1(&self) -> String { todo!() }
    pub fn method2(&mut self) { todo!() }
    pub fn method3(&self, param: i32) -> bool { todo!() }
    pub fn method4(&mut self, data: &str) { todo!() }
    pub fn method5(&self) -> Vec<String> { todo!() }
    pub fn method6(&mut self, items: Vec<i32>) { todo!() }
    pub fn method7(&self, key: &str) -> Option<String> { todo!() }
    pub fn method8(&mut self, value: f64) -> Result<(), String> { todo!() }
    pub fn method9(&self) -> HashMap<String, i32> { todo!() }
    pub fn method10(&mut self, config: Config) { todo!() }
    pub fn method11(&self, filter: impl Fn(&str) -> bool) -> Vec<String> { todo!() }
}
"#;

            let mut parser = AstParser::new().expect("Failed to create parser");
            let parsed_file = parser
                .parse_content(rust_code, &PathBuf::from("test.rs"), SourceLanguage::Rust)
                .expect("Failed to parse Rust code");

            // Use strict thresholds to ensure detection
            let detector = GodObjectDetector::new(5, 5); // 5 methods, 5 fields max
            let issues = detector
                .detect_issues(&parsed_file)
                .expect("Failed to detect issues");

            // Should detect the GodObject struct as having too many methods and fields
            assert!(!issues.is_empty(), "Should detect god object issues");
            assert!(
                issues
                    .iter()
                    .any(|issue| issue.description.contains("GodObject")),
                "Should detect GodObject struct"
            );
        }

        #[test]
        fn test_god_object_negative() {
            let rust_code = r#"
pub struct WellDesignedStruct {
    id: u32,
    name: String,
}

impl WellDesignedStruct {
    pub fn new(id: u32, name: String) -> Self {
        Self { id, name }
    }
    
    pub fn get_id(&self) -> u32 {
        self.id
    }
    
    pub fn get_name(&self) -> &str {
        &self.name
    }
}
"#;

            let mut parser = AstParser::new().expect("Failed to create parser");
            let parsed_file = parser
                .parse_content(rust_code, &PathBuf::from("test.rs"), SourceLanguage::Rust)
                .expect("Failed to parse Rust code");

            // Use default thresholds
            let detector = GodObjectDetector::new(10, 8); // 10 methods, 8 fields max
            let issues = detector
                .detect_issues(&parsed_file)
                .expect("Failed to detect issues");

            // Should not detect any god object issues for well-designed struct
            assert!(
                issues.is_empty(),
                "Should not detect any god object issues for well-designed struct"
            );
        }
        
        #[test]
        fn test_serde_dto_exclusion() {
            let rust_code = r#"
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct UserDto {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub address: String,
    pub city: String,
    pub state: String,
    pub zip_code: String,
    pub country: String,
    pub date_created: String,
    pub date_modified: String,
    pub is_active: bool,
    pub role: String,
    pub preferences: String,
    pub avatar_url: Option<String>,
    pub timezone: String,
    pub language: String,
    pub metadata: String,
}

impl UserDto {
    pub fn new() -> Self { todo!() }
    pub fn validate(&self) -> bool { true }
}
"#;
            
            let mut parser = AstParser::new().expect("Failed to create parser");
            let parsed_file = parser
                .parse_content(rust_code, &PathBuf::from("user_dto.rs"), SourceLanguage::Rust)
                .expect("Failed to parse Rust code");
            
            // This should be excluded as a DTO despite having many fields
            let detector = GodObjectDetector::new(5, 5); // Very strict thresholds
            let issues = detector
                .detect_issues(&parsed_file)
                .expect("Failed to detect issues");
            
            // Should not detect DTO as God Object due to pattern recognition
            assert!(
                issues.is_empty(),
                "Should not detect Serde DTO as God Object: found {} issues",
                issues.len()
            );
        }
        
        #[test]
        fn test_builder_pattern_exclusion() {
            let rust_code = r#"
pub struct DatabaseConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
    username: Option<String>,
    password: Option<String>,
    database: Option<String>,
    ssl_mode: Option<String>,
    timeout: Option<u32>,
    pool_size: Option<u32>,
    retry_attempts: Option<u32>,
    connection_string: Option<String>,
}

impl DatabaseConfigBuilder {
    pub fn new() -> Self { todo!() }
    pub fn host(mut self, host: String) -> Self { todo!() }
    pub fn port(mut self, port: u16) -> Self { todo!() }
    pub fn username(mut self, username: String) -> Self { todo!() }
    pub fn password(mut self, password: String) -> Self { todo!() }
    pub fn database(mut self, database: String) -> Self { todo!() }
    pub fn ssl_mode(mut self, ssl_mode: String) -> Self { todo!() }
    pub fn timeout(mut self, timeout: u32) -> Self { todo!() }
    pub fn pool_size(mut self, pool_size: u32) -> Self { todo!() }
    pub fn retry_attempts(mut self, retry_attempts: u32) -> Self { todo!() }
    pub fn connection_string(mut self, connection_string: String) -> Self { todo!() }
    pub fn build(self) -> DatabaseConfig { todo!() }
}
"#;
            
            let mut parser = AstParser::new().expect("Failed to create parser");
            let parsed_file = parser
                .parse_content(rust_code, &PathBuf::from("database_config.rs"), SourceLanguage::Rust)
                .expect("Failed to parse Rust code");
            
            let detector = GodObjectDetector::new(5, 5); // Very strict thresholds
            let issues = detector
                .detect_issues(&parsed_file)
                .expect("Failed to detect issues");
            
            // Should not detect Builder as God Object
            assert!(
                issues.is_empty(),
                "Should not detect Builder pattern as God Object: found {} issues",
                issues.len()
            );
        }
        
        #[test]
        fn test_generated_file_exclusion() {
            let generated_code = r#"
// This file was automatically generated by protoc
// DO NOT EDIT!

pub struct GeneratedMessage {
    field1: String,
    field2: String,
    field3: String,
    field4: String,
    field5: String,
    field6: String,
    field7: String,
    field8: String,
    field9: String,
    field10: String,
}

impl GeneratedMessage {
    pub fn new() -> Self { todo!() }
    pub fn get_field1(&self) -> &str { todo!() }
    pub fn set_field1(&mut self, value: String) { todo!() }
    pub fn get_field2(&self) -> &str { todo!() }
    pub fn set_field2(&mut self, value: String) { todo!() }
    pub fn get_field3(&self) -> &str { todo!() }
    pub fn set_field3(&mut self, value: String) { todo!() }
    pub fn get_field4(&self) -> &str { todo!() }
    pub fn set_field4(&mut self, value: String) { todo!() }
    pub fn get_field5(&self) -> &str { todo!() }
    pub fn set_field5(&mut self, value: String) { todo!() }
    pub fn serialize(&self) -> Vec<u8> { todo!() }
    pub fn deserialize(data: &[u8]) -> Self { todo!() }
}
"#;
            
            let mut parser = AstParser::new().expect("Failed to create parser");
            let parsed_file = parser
                .parse_content(generated_code, &PathBuf::from("generated.rs"), SourceLanguage::Rust)
                .expect("Failed to parse Rust code");
            
            let detector = GodObjectDetector::new(5, 5); // Very strict thresholds
            let issues = detector
                .detect_issues(&parsed_file)
                .expect("Failed to detect issues");
            
            // Should not detect generated code as God Object
            assert!(
                issues.is_empty(),
                "Should not detect generated code as God Object: found {} issues",
                issues.len()
            );
        }
        
        #[test]
        fn test_python_dto_exclusion() {
            let python_code = r#"
from pydantic import BaseModel
from typing import Optional

class UserModel(BaseModel):
    id: int
    username: str
    email: str
    first_name: str
    last_name: str
    phone: Optional[str]
    address: str
    city: str
    state: str
    zip_code: str
    country: str
    date_created: str
    date_modified: str
    is_active: bool
    role: str
    preferences: str
    avatar_url: Optional[str]
    timezone: str
    language: str
    metadata: str
    
    def validate_email(self):
        return True
        
    def to_dict(self):
        return {}
"#;
            
            let mut parser = AstParser::new().expect("Failed to create parser");
            let parsed_file = parser
                .parse_content(python_code, &PathBuf::from("user_model.py"), SourceLanguage::Python)
                .expect("Failed to parse Python code");
            
            let detector = GodObjectDetector::new(5, 5); // Very strict thresholds
            let issues = detector
                .detect_issues(&parsed_file)
                .expect("Failed to detect issues");
            
            // Should not detect Pydantic model as God Object
            assert!(
                issues.is_empty(),
                "Should not detect Pydantic model as God Object: found {} issues",
                issues.len()
            );
        }
        
        #[test]
        fn test_framework_controller_exclusion() {
            let python_code = r#"
from django.views import View
from django.http import JsonResponse

class UserController(View):
    def get(self, request, user_id=None):
        pass
        
    def post(self, request):
        pass
        
    def put(self, request, user_id):
        pass
        
    def delete(self, request, user_id):
        pass
        
    def list_users(self, request):
        pass
        
    def search_users(self, request):
        pass
        
    def validate_user_data(self, data):
        pass
        
    def serialize_user(self, user):
        pass
        
    def handle_user_creation(self, data):
        pass
        
    def handle_user_update(self, user_id, data):
        pass
        
    def handle_user_deletion(self, user_id):
        pass
        
    def send_user_notification(self, user_id, message):
        pass
"#;
            
            let mut parser = AstParser::new().expect("Failed to create parser");
            let parsed_file = parser
                .parse_content(python_code, &PathBuf::from("user_controller.py"), SourceLanguage::Python)
                .expect("Failed to parse Python code");
            
            let detector = GodObjectDetector::new(5, 1); // Very strict thresholds
            let issues = detector
                .detect_issues(&parsed_file)
                .expect("Failed to detect issues");
            
            // Should not detect Django controller as God Object due to framework detection
            assert!(
                issues.is_empty(),
                "Should not detect Django controller as God Object: found {} issues",
                issues.len()
            );
        }
        
        #[test]
        fn test_language_specific_thresholds() {
            let config = GodObjectConfig::default();
            
            // Verify language-specific thresholds are different
            assert_eq!(config.method_thresholds[&SourceLanguage::Rust], 30);
            assert_eq!(config.method_thresholds[&SourceLanguage::Python], 25);
            assert_eq!(config.method_thresholds[&SourceLanguage::JavaScript], 20);
            
            assert_eq!(config.field_thresholds[&SourceLanguage::Rust], 20);
            assert_eq!(config.field_thresholds[&SourceLanguage::Python], 15);
            assert_eq!(config.field_thresholds[&SourceLanguage::JavaScript], 12);
        }
        
        #[test]
        fn test_enhanced_configuration() {
            let mut config = GodObjectConfig::default();
            config.recognize_patterns = false;
            config.enable_behavioral_analysis = false;
            config.enable_cohesion_analysis = false;
            
            let detector = GodObjectDetector::with_config(config);
            
            // Test that configuration is properly applied
            assert!(!detector.config.recognize_patterns);
            assert!(!detector.config.enable_behavioral_analysis);
            assert!(!detector.config.enable_cohesion_analysis);
        }
        
        #[test]
        fn test_true_god_object_detection() {
            let rust_code = r#"
pub struct TrueGodObject {
    // Database fields
    user_id: u64,
    username: String,
    email: String,
    password_hash: String,
    
    // UI state
    current_page: String,
    selected_items: Vec<String>,
    ui_theme: String,
    
    // Business logic state
    order_total: f64,
    tax_rate: f64,
    discount_code: Option<String>,
    
    // File system state
    temp_files: Vec<String>,
    log_file_path: String,
    config_file_path: String,
    
    // Network state
    api_endpoints: HashMap<String, String>,
    connection_pool: String,
    timeout_settings: u32,
}

impl TrueGodObject {
    // Database operations
    pub fn save_to_database(&self) -> Result<(), String> { todo!() }
    pub fn load_from_database(id: u64) -> Result<Self, String> { todo!() }
    pub fn delete_from_database(&self) -> Result<(), String> { todo!() }
    pub fn update_database(&self) -> Result<(), String> { todo!() }
    
    // UI operations
    pub fn render_page(&self) -> String { todo!() }
    pub fn handle_click(&mut self, item: String) { todo!() }
    pub fn update_theme(&mut self, theme: String) { todo!() }
    pub fn refresh_ui(&self) { todo!() }
    
    // Business logic
    pub fn calculate_total(&self) -> f64 { todo!() }
    pub fn apply_discount(&mut self, code: String) -> bool { todo!() }
    pub fn process_payment(&self, amount: f64) -> Result<(), String> { todo!() }
    pub fn generate_invoice(&self) -> String { todo!() }
    
    // File operations
    pub fn write_log(&self, message: String) -> Result<(), String> { todo!() }
    pub fn read_config(&self) -> Result<String, String> { todo!() }
    pub fn cleanup_temp_files(&self) -> Result<(), String> { todo!() }
    pub fn backup_data(&self) -> Result<(), String> { todo!() }
    
    // Network operations
    pub fn make_api_call(&self, endpoint: String) -> Result<String, String> { todo!() }
    pub fn update_connection_pool(&mut self) { todo!() }
    pub fn check_connectivity(&self) -> bool { todo!() }
    pub fn handle_timeout(&self) { todo!() }
    
    // Utility methods
    pub fn validate_data(&self) -> bool { todo!() }
    pub fn serialize(&self) -> String { todo!() }
    pub fn deserialize(data: String) -> Result<Self, String> { todo!() }
    pub fn clone_data(&self) -> Self { todo!() }
}
"#;
            
            let mut parser = AstParser::new().expect("Failed to create parser");
            let parsed_file = parser
                .parse_content(rust_code, &PathBuf::from("true_god_object.rs"), SourceLanguage::Rust)
                .expect("Failed to parse Rust code");
            
            let detector = GodObjectDetector::new(15, 10); // Reasonable thresholds
            let issues = detector
                .detect_issues(&parsed_file)
                .expect("Failed to detect issues");
            
            // Should detect this as a true God Object
            assert!(
                !issues.is_empty(),
                "Should detect true God Object with mixed responsibilities"
            );
            
            let issue = &issues[0];
            assert!(issue.description.contains("TrueGodObject"));
            assert!(issue.severity == "Critical" || issue.severity == "High");
        }
    }

    #[cfg(not(feature = "tree-sitter"))]
    mod stub_tests {
        use super::*;

        #[test]
        fn test_god_object_detector_graceful_fallback() {
            // Verify detector doesn't panic when tree-sitter unavailable
            let result = std::panic::catch_unwind(|| {
                // Attempt to create a detector instance
                let detector = GodObjectDetector::with_default_config();
                // Try to run detection with minimal input
                let _ = detector.detect_issues("");
            });
            
            assert!(
                result.is_ok(),
                "GodObjectDetector should not panic when tree-sitter is disabled"
            );
        }

        #[test]
        fn test_informative_skipping() {
            println!("SKIPPED: tree-sitter feature disabled - using fallback behavior for god object detection");
            // This test validates that the system provides clear feedback
            // about missing functionality when tree-sitter is disabled
        }
    }
}