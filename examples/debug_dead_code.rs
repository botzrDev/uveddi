use uveddi::analysis::detectors::anti_patterns::DeadCodeDetector;
use uveddi::analysis::AnalysisDetector;
use uveddi::ast::tree_sitter::{AstParser, SourceLanguage};
use std::path::PathBuf;

fn main() {
    env_logger::init();
    
    // Test Python
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
        .parse_content(python_code, &PathBuf::from("app.py"), SourceLanguage::Python)
        .expect("Failed to parse Python code");

    let detector = DeadCodeDetector::with_default_config();
    
    // Debug: Extract symbols and references separately
    let symbols = detector.extract_symbols(&parsed_file).unwrap();
    let references = detector.extract_references(&parsed_file).unwrap();
    
    println!("=== SYMBOLS FOUND ===");
    for symbol in &symbols {
        println!("Symbol: {} (exported: {}, type: {:?}, confidence: {:.2})", 
                 symbol.name, symbol.is_exported, symbol.symbol_type, symbol.confidence);
    }
    
    println!("\n=== REFERENCES FOUND ===");
    for reference in &references {
        println!("Reference: {}", reference);
    }
    
    println!("\n=== ISSUES DETECTED ===");
    let issues = detector
        .detect_issues(&parsed_file)
        .expect("Failed to detect issues");
        
    for issue in &issues {
        println!("Issue: {}", issue.description);
        println!("  Contains 'used_function': {}", issue.description.contains("used_function"));
        println!("  Contains 'unused_function': {}", issue.description.contains("unused_function"));
        println!("  Contains 'main': {}", issue.description.contains("main"));
    }
    
    println!("\n=== TEST ASSERTIONS ===");
    println!("Issues count: {}", issues.len());
    println!("Should detect unused_function: {}", 
             issues.iter().any(|issue| issue.description.contains("unused_function")));
    println!("Should NOT detect used_function: {}", 
             !issues.iter().any(|issue| issue.description.contains("used_function")));
    println!("Should detect _private_unused: {}", 
             issues.iter().any(|issue| issue.description.contains("_private_unused")));
}