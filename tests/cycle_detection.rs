//! Cycle detection tests

#[cfg(test)]
mod tests {
    use codeatlas::analysis::cycle_detector::CycleDetector;
    use codeatlas::analysis::dependency_graph::DependencyGraph;
    use codeatlas::analysis::dependency_extractor::DependencyExtractor;
    use codeatlas::ast::tree_sitter::{AstParser, SourceLanguage};
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
    fn detects_cycles_rust() {
        let dir = tempdir().unwrap();
        let a_path = create_temp_file(&dir, "a.rs", "mod b;");
        let b_path = create_temp_file(&dir, "b.rs", "mod a;");

        let mut parser = AstParser::new().unwrap();
        let mut graph = DependencyGraph::new();
        let extractor = DependencyExtractor::new();

        let a_file = parser.parse_file(&a_path).unwrap();
        let b_file = parser.parse_file(&b_path).unwrap();

        let deps = vec![a_file, b_file].iter().flat_map(|f| extractor.extract_dependencies(f)).collect();
        graph.build_from_dependencies(deps);

        let detector = CycleDetector;
        let cycles = detector.detect_cycles(&graph);
        assert_eq!(cycles.len(), 1);
    }

    #[test]
    fn detects_cycles_python() {
        let dir = tempdir().unwrap();
        let a_path = create_temp_file(&dir, "a.py", "import b");
        let b_path = create_temp_file(&dir, "b.py", "import a");

        let mut parser = AstParser::new().unwrap();
        let mut graph = DependencyGraph::new();
        let extractor = DependencyExtractor::new();

        let a_file = parser.parse_file(&a_path).unwrap();
        let b_file = parser.parse_file(&b_path).unwrap();

        let deps = vec![a_file, b_file].iter().flat_map(|f| extractor.extract_dependencies(f)).collect();
        graph.build_from_dependencies(deps);

        let detector = CycleDetector;
        let cycles = detector.detect_cycles(&graph);
        assert_eq!(cycles.len(), 1);
    }

    #[test]
    fn detects_cycles_js() {
        let dir = tempdir().unwrap();
        let a_path = create_temp_file(&dir, "a.js", "import b from './b.js';");
        let b_path = create_temp_file(&dir, "b.js", "import a from './a.js';");

        let mut parser = AstParser::new().unwrap();
        let mut graph = DependencyGraph::new();
        let extractor = DependencyExtractor::new();

        let a_file = parser.parse_file(&a_path).unwrap();
        let b_file = parser.parse_file(&b_path).unwrap();

        let deps = vec![a_file, b_file].iter().flat_map(|f| extractor.extract_dependencies(f)).collect();
        graph.build_from_dependencies(deps);

        let detector = CycleDetector;
        let cycles = detector.detect_cycles(&graph);
        assert_eq!(cycles.len(), 1);
    }

    #[test]
    fn cycle_severity_and_snippet() {
        let dir = tempdir().unwrap();
        let a_path = create_temp_file(&dir, "a.rs", "mod b;");
        create_temp_file(&dir, "b.rs", "mod a;");

        let mut parser = AstParser::new().unwrap();
        let mut graph = DependencyGraph::new();
        let extractor = DependencyExtractor::new();

        let a_file = parser.parse_file(&a_path).unwrap();
        let b_file = parser.parse_file(&dir.path().join("b.rs")).unwrap();

        let deps = vec![a_file, b_file].iter().flat_map(|f| extractor.extract_dependencies(f)).collect();
        graph.build_from_dependencies(deps);

        let detector = CycleDetector;
        let cycles = detector.detect_cycles(&graph);
        let issue = detector.issues_from_cycles(&cycles, &graph).pop().unwrap();

        assert_eq!(issue.severity, "high");
        assert!(issue.code_snippet.is_some());
    }
}
