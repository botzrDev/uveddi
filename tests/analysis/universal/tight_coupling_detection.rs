//! Tight Coupling anti-pattern detection tests
//!
//! This module tests detection of tight coupling between components.

#[cfg(test)]
mod tests {
    use crate::analysis::detectors::anti_patterns::tight_coupling::TightCouplingDetector;
    use crate::analysis::AnalysisDetector;
    use crate::ast::tree_sitter_impl::{ParsedFile, SourceLanguage, AstParser};
    use std::path::Path;
    use std::sync::Arc;

    fn create_test_rust_file_with_high_coupling() -> ParsedFile {
        let source = r#"
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::net::TcpStream;
use std::thread;
use std::sync::Mutex;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use tokio::runtime::Runtime;
use reqwest::Client;

struct HighlyCoupledStruct {
    file_handler: File,
    network_client: TcpStream,
    cache: HashMap<String, String>,
    runtime: Runtime,
    http_client: Client,
    mutex: Mutex<i32>,
}

impl HighlyCoupledStruct {
    fn new() -> Self {
        let file = File::open("test.txt").unwrap();
        let stream = TcpStream::connect("127.0.0.1:8080").unwrap();
        let cache = HashMap::new();
        let runtime = Runtime::new().unwrap();
        let client = Client::new();
        let mutex = Mutex::new(0);
        
        Self {
            file_handler: file,
            network_client: stream,
            cache,
            runtime,
            http_client: client,
            mutex,
        }
    }
    
    fn process_data(&self) {
        // Multiple external dependencies in one method
        let mut buffer = Vec::new();
        self.file_handler.read_to_end(&mut buffer);
        self.network_client.write_all(&buffer);
        self.cache.insert("key".to_string(), "value".to_string());
        self.runtime.block_on(async {
            self.http_client.get("http://example.com").send().await;
        });
        let _guard = self.mutex.lock().unwrap();
    }
}
"#;
        
        let parser = AstParser::new();
        parser.parse_source(source, SourceLanguage::Rust, Path::new("test_high_coupling.rs"))
            .expect("Failed to parse test source")
    }

    fn create_test_rust_file_with_low_coupling() -> ParsedFile {
        let source = r#"
trait DataProcessor {
    fn process(&self, data: &str) -> String;
}

struct SimpleProcessor;

impl DataProcessor for SimpleProcessor {
    fn process(&self, data: &str) -> String {
        data.to_uppercase()
    }
}

struct DataHandler {
    processor: Box<dyn DataProcessor>,
}

impl DataHandler {
    fn new(processor: Box<dyn DataProcessor>) -> Self {
        Self { processor }
    }
    
    fn handle_data(&self, input: &str) -> String {
        self.processor.process(input)
    }
}
"#;
        
        let parser = AstParser::new();
        parser.parse_source(source, SourceLanguage::Rust, Path::new("test_low_coupling.rs"))
            .expect("Failed to parse test source")
    }

    #[test]
    fn test_tight_coupling_positive() {
        let parsed_file = create_test_rust_file_with_high_coupling();
        let detector = TightCouplingDetector::default();
        
        let issues = detector.detect_issues(&parsed_file).unwrap();
        
        // Should detect high coupling due to many dependencies
        assert!(!issues.is_empty(), "Expected to find tight coupling issues in highly coupled code");
        
        // Verify issue details
        let issue = &issues[0];
        assert_eq!(issue.severity, "Critical");
        assert!(issue.description.contains("dependencies"));
        assert!(issue.description.contains("threshold"));
    }

    #[test]
    fn test_tight_coupling_negative() {
        let parsed_file = create_test_rust_file_with_low_coupling();
        let detector = TightCouplingDetector::default();
        
        let issues = detector.detect_issues(&parsed_file).unwrap();
        
        // Should not detect tight coupling in well-designed code
        assert!(issues.is_empty(), "Expected no tight coupling issues in loosely coupled code");
    }

    #[test]
    fn test_detector_configuration() {
        let detector = TightCouplingDetector::default();
        
        // Verify detector is properly configured
        assert_eq!(detector.get_detector_name(), "TightCouplingDetector");
        
        let anti_pattern_types = detector.get_anti_pattern_types();
        assert_eq!(anti_pattern_types.len(), 1);
        assert_eq!(anti_pattern_types[0].name, "Tight Coupling");
        assert_eq!(anti_pattern_types[0].category, "structural");
    }

    #[test]
    fn test_language_specific_analysis() {
        let detector = TightCouplingDetector::default();
        
        // Test with Rust file
        let rust_file = create_test_rust_file_with_high_coupling();
        let rust_issues = detector.detect_issues(&rust_file).unwrap();
        
        // Should work with Rust files
        assert!(!rust_issues.is_empty(), "Should detect issues in Rust files");
        
        // Verify file path is captured
        assert!(rust_issues[0].file_path.contains("test_high_coupling.rs"));
    }
}
