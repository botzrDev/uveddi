//! Dead Code anti-pattern detection tests
//!
//! This module tests detection of unused functions, variables, or modules.

#[cfg(test)]
mod tests {
    use crate::analysis::detectors::anti_patterns::DeadCodeDetector;
    use crate::analysis::AnalysisDetector;
    use crate::ast::tree_sitter::{AstParser, SourceLanguage};
    use std::path::PathBuf;

    #[test]
    fn test_dead_code_rust_unused_function() {
        let rust_code = r#"
fn used_function() {
    println!("This function is used");
}

fn unused_function() {
    println!("This function is never called");
}

fn main() {
    used_function();
}
"#;

        let mut parser = AstParser::new().expect("Failed to create parser");
        let parsed_file = parser
            .parse_content(rust_code, &PathBuf::from("test.rs"), SourceLanguage::Rust)
            .expect("Failed to parse Rust code");

        let detector = DeadCodeDetector::with_default_config();
        let issues = detector
            .detect_issues(&parsed_file)
            .expect("Failed to detect issues");

        // Should detect unused_function as dead code
        assert!(!issues.is_empty(), "Should detect dead code");
        assert!(
            issues.iter().any(|issue| issue.description.contains("unused_function")),
            "Should detect unused_function as dead code"
        );
        
        // Should not detect used_function or main as dead code
        assert!(
            !issues.iter().any(|issue| issue.description.contains("used_function")),
            "Should not detect used_function as dead code"
        );
        assert!(
            !issues.iter().any(|issue| issue.description.contains("main")),
            "Should not detect main as dead code"
        );
    }

    #[test]
    fn test_dead_code_python_unused_function() {
        let python_code = r#"
def used_function():
    print("This function is used")

def unused_function():
    print("This function is never called")

def _private_unused():
    print("Private unused function")

if __name__ == "__main__":
    used_function()
"#;

        let mut parser = AstParser::new().expect("Failed to create parser");
        let parsed_file = parser
            .parse_content(python_code, &PathBuf::from("test.py"), SourceLanguage::Python)
            .expect("Failed to parse Python code");

        let detector = DeadCodeDetector::with_default_config();
        let issues = detector
            .detect_issues(&parsed_file)
            .expect("Failed to detect issues");

        // Should detect unused functions
        assert!(!issues.is_empty(), "Should detect dead code");
        
        // Check that unused_function is detected
        let unused_detected = issues.iter().any(|issue| issue.description.contains("unused_function"));
        assert!(unused_detected, "Should detect unused_function as dead code");
    }

    #[test]
    fn test_dead_code_javascript_unused_function() {
        let js_code = r#"
function usedFunction() {
    console.log("This function is used");
}

function unusedFunction() {
    console.log("This function is never called");
}

function main() {
    usedFunction();
}

main();
"#;

        let mut parser = AstParser::new().expect("Failed to create parser");
        let parsed_file = parser
            .parse_content(js_code, &PathBuf::from("test.js"), SourceLanguage::JavaScript)
            .expect("Failed to parse JavaScript code");

        let detector = DeadCodeDetector::with_default_config();
        let issues = detector
            .detect_issues(&parsed_file)
            .expect("Failed to detect issues");

        // Should detect unused function
        assert!(!issues.is_empty(), "Should detect dead code");
        assert!(
            issues.iter().any(|issue| issue.description.contains("unusedFunction")),
            "Should detect unusedFunction as dead code"
        );
    }

    #[test]
    fn test_dead_code_exported_symbols() {
        let rust_code = r#"
pub fn exported_function() {
    println!("This is exported");
}

fn private_unused() {
    println!("This is private and unused");
}
"#;

        let mut parser = AstParser::new().expect("Failed to create parser");
        let parsed_file = parser
            .parse_content(rust_code, &PathBuf::from("lib.rs"), SourceLanguage::Rust)
            .expect("Failed to parse Rust code");

        let detector = DeadCodeDetector::with_default_config();
        let issues = detector
            .detect_issues(&parsed_file)
            .expect("Failed to detect issues");

        // Should detect private_unused but not exported_function
        let private_detected = issues.iter().any(|issue| issue.description.contains("private_unused"));
        let exported_detected = issues.iter().any(|issue| issue.description.contains("exported_function"));
        
        assert!(private_detected, "Should detect private unused function");
        assert!(!exported_detected, "Should not detect exported function as dead code");
    }

    #[test]
    fn test_dead_code_confidence_scoring() {
        let rust_code = r#"
fn unused_function() {
    println!("Unused");
}
"#;

        let mut parser = AstParser::new().expect("Failed to create parser");
        let parsed_file = parser
            .parse_content(rust_code, &PathBuf::from("test.rs"), SourceLanguage::Rust)
            .expect("Failed to parse Rust code");

        let detector = DeadCodeDetector::with_default_config();
        let issues = detector
            .detect_issues(&parsed_file)
            .expect("Failed to detect issues");

        assert!(!issues.is_empty(), "Should detect dead code");
        
        // Check that severity is assigned based on confidence
        let issue = &issues[0];
        assert!(
            issue.severity == "High" || issue.severity == "Medium" || issue.severity == "Low",
            "Should have valid severity level"
        );
    }

    #[test]
    fn test_dead_code_no_false_positives() {
        let rust_code = r#"
fn helper() {
    println!("Helper function");
}

fn main() {
    helper();
}
"#;

        let mut parser = AstParser::new().expect("Failed to create parser");
        let parsed_file = parser
            .parse_content(rust_code, &PathBuf::from("test.rs"), SourceLanguage::Rust)
            .expect("Failed to parse Rust code");

        let detector = DeadCodeDetector::with_default_config();
        let issues = detector
            .detect_issues(&parsed_file)
            .expect("Failed to detect issues");

        // Should not detect any dead code since all functions are used
        assert!(issues.is_empty(), "Should not detect any dead code when all functions are used");
    }
}
