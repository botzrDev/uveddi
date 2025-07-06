//! God Object anti-pattern detection tests
//!
//! This module tests detection of classes/structs with too many responsibilities.

#[cfg(test)]
mod tests {
    #[test]
    fn test_god_object_positive() {
        use crate::analysis::detectors::anti_patterns::god_object::GodObjectDetector;
        use crate::analysis::AnalysisDetector;
        use crate::ast::tree_sitter::{AstParser, SourceLanguage};
        use std::path::PathBuf;

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
            issues.iter().any(|issue| issue.description.contains("GodObject")),
            "Should detect GodObject struct"
        );
    }

    #[test]
    fn test_god_object_negative() {
        use crate::analysis::detectors::anti_patterns::god_object::GodObjectDetector;
        use crate::analysis::AnalysisDetector;
        use crate::ast::tree_sitter::{AstParser, SourceLanguage};
        use std::path::PathBuf;

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
        assert!(issues.is_empty(), "Should not detect any god object issues for well-designed struct");
    }
}
