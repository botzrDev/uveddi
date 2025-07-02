//! Dependency extraction tests for all supported languages (Rust, Python, JS/TS)

#[cfg(test)]
mod tests {
    use uveddi::analysis::dependency_extractor::DependencyExtractor;
    use uveddi::ast::tree_sitter::AstParser;
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::Write;

    fn create_temp_file(dir: &tempfile::TempDir, name: &str, content: &str) -> std::path::PathBuf {
        let file_path = dir.path().join(name);
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "{}", content).unwrap();
        file_path
    }

    #[test]
    fn rust_dependency_extraction_basic() {
        let dir = tempdir().unwrap();
        let file_path = create_temp_file(&dir, "a.rs", "mod b;\nuse std::fs;\n");
        let mut parser = AstParser::new().unwrap();
        let parsed = parser.parse_file(&file_path).unwrap();
        let extractor = DependencyExtractor::new().unwrap();
        let deps = extractor.extract_from_ast(&parsed).unwrap();
        let dep_names: Vec<_> = deps.iter().map(|d| d.to_module.as_str()).collect();
        assert!(dep_names.contains(&"b"));
    }

    #[test]
    fn python_dependency_extraction_basic() {
        let dir = tempdir().unwrap();
        let file_path = create_temp_file(&dir, "a.py", "import os\nimport sys\n");
        let mut parser = AstParser::new().unwrap();
        let parsed = parser.parse_file(&file_path).unwrap();
        let extractor = DependencyExtractor::new().unwrap();
        let deps = extractor.extract_from_ast(&parsed).unwrap();
        let dep_names: Vec<_> = deps.iter().map(|d| d.to_module.as_str()).collect();
        assert!(dep_names.contains(&"os"));
        assert!(dep_names.contains(&"sys"));
    }

    #[test]
    fn js_dependency_extraction_basic() {
        let dir = tempdir().unwrap();
        let file_path = create_temp_file(&dir, "a.js", "import b from './b.js';\nimport fs from 'fs';\n");
        let mut parser = AstParser::new().unwrap();
        let parsed = parser.parse_file(&file_path).unwrap();
        let extractor = DependencyExtractor::new().unwrap();
        let deps = extractor.extract_from_ast(&parsed).unwrap();
        let dep_names: Vec<_> = deps.iter().map(|d| d.to_module.as_str()).collect();
        assert_eq!(dep_names.len(), 2);
        assert!(dep_names.contains(&"./b.js"));
        assert!(dep_names.contains(&"fs"));
    }

    // TODO: Add more tests for edge cases, error handling, and cross-language scenarios
}
