use crate::analysis::detectors::anti_patterns::code_duplication::CodeDuplicationDetector;
use crate::analysis::AnalysisDetector;
use crate::ast::tree_sitter::{ParsedFile, SourceLanguage};
use std::path::PathBuf;
use std::time::SystemTime;

#[test]
fn debug_code_duplication_detector() {
    // Initialize logger for debugging
    let _ = env_logger::builder().is_test(true).try_init();

    use crate::analysis::detectors::anti_patterns::code_duplication::DuplicationConfig;

    // Create a detector with more lenient settings for testing
    let config = DuplicationConfig {
        min_tokens: 10, // Lower threshold for testing
        min_lines: 2,   // Lower threshold for testing
        similarity_threshold: 0.7,
        fingerprint_length: 5,
        ignore_identifiers: true,
        ignore_literals: true,
    };

    let detector = CodeDuplicationDetector::with_config(config);

    // Test with very simple Rust code
    let rust_code = r#"
fn test_function() {
    let x = 42;
    println!("Hello {}", x);
}

fn another_function() {
    let x = 42;
    println!("Hello {}", x);
}
"#;

    let parsed_file = ParsedFile {
        path: PathBuf::from("debug_test.rs"),
        language: SourceLanguage::Rust,
        tree: None,
        source: rust_code.to_string(),
        custom_ast: None,
        modified_at: SystemTime::now(),
    };

    // Parse the file first
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_rust::language()).unwrap();

    let mut parsed_file = parsed_file;
    parsed_file.tree = Some(parser.parse(rust_code, None).unwrap());

    // Check if the tree is valid
    if let Some(tree) = &parsed_file.tree {
        println!("Tree root: {:?}", tree.root_node().kind());

        // Print all function nodes
        let mut cursor = tree.walk();
        print_nodes(&mut cursor, 0);
    }

    // Run the detector
    println!("Before running detector...");

    let issues = detector.detect_issues(&parsed_file).unwrap();

    println!("Found {} issues", issues.len());
    for issue in &issues {
        println!("Issue: {}", issue.description);
    }

    // This should find duplication now
    assert!(
        !issues.is_empty(),
        "Should detect code duplication with lenient settings"
    );
}

fn print_nodes(cursor: &mut tree_sitter::TreeCursor, depth: usize) {
    let node = cursor.node();
    let indent = "  ".repeat(depth);
    println!(
        "{}Node: {} at {}:{}-{}:{}",
        indent,
        node.kind(),
        node.start_position().row + 1,
        node.start_position().column + 1,
        node.end_position().row + 1,
        node.end_position().column + 1
    );

    if cursor.goto_first_child() {
        loop {
            print_nodes(cursor, depth + 1);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        cursor.goto_parent();
    }
}
