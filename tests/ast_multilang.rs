use std::fs::File;
use std::io::Write;
use tempfile::tempdir;
use uveddi::ast::tree_sitter::{AstParser, SourceLanguage};

#[test]
fn test_multilang_ast_parsing_and_caching() {
    let dir = tempdir().unwrap();
    let rust_path = dir.path().join("test.rs");
    let py_path = dir.path().join("test.py");
    let js_path = dir.path().join("test.js");
    File::create(&rust_path)
        .unwrap()
        .write_all(b"struct Foo { bar: i32 }\n")
        .unwrap();
    File::create(&py_path)
        .unwrap()
        .write_all(b"class Foo:\n    pass\n")
        .unwrap();
    File::create(&js_path)
        .unwrap()
        .write_all(b"class Foo {}\n")
        .unwrap();
    let mut parser = AstParser::new().unwrap();
    let rust_ast = parser.parse_file(&rust_path).unwrap();
    let py_ast = parser.parse_file(&py_path).unwrap();
    let js_ast = parser.parse_file(&js_path).unwrap();
    // Test cache hit
    let rust_ast2 = parser.parse_file(&rust_path).unwrap();
    assert_eq!(rust_ast.path, rust_ast2.path);
    assert_eq!(rust_ast.language, SourceLanguage::Rust);
    assert_eq!(py_ast.language, SourceLanguage::Python);
    assert_eq!(js_ast.language, SourceLanguage::JavaScript);
}
