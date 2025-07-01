//! God Object anti-pattern detection tests

#[cfg(test)]
mod tests {
    use uveddi::analysis::anti_patterns::god_object_detector::GodObjectDetector;
    use uveddi::analysis::AnalysisDetector;
    use uveddi::ast::tree_sitter::{AstParser};
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn detects_god_object_rust() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("god_object.rs");
        let code = r#"
        struct GodObject {
            a: i32,
            b: i32,
            c: i32,
        }
        impl GodObject {
            fn m1(&self) {}
            fn m2(&self) {}
            fn m3(&self) {}
            fn m4(&self) {}
            fn m5(&self) {}
            fn m6(&self) {}
            fn m7(&self) {}
            fn m8(&self) {}
            fn m9(&self) {}
            fn m10(&self) {}
            fn m11(&self) {}
            fn m12(&self) {}
            fn m13(&self) {}
            fn m14(&self) {}
            fn m15(&self) {}
            fn m16(&self) {}
            fn m17(&self) {}
            fn m18(&self) {}
            fn m19(&self) {}
            fn m20(&self) {}
            fn m21(&self) {}
        }
        "#;
        File::create(&file_path).unwrap().write_all(code.as_bytes()).unwrap();
        let mut parser = AstParser::new().unwrap();
        let parsed = parser.parse_file(&file_path).unwrap();
        let detector = GodObjectDetector::new(20, 10); // Example thresholds
        let issues = detector.detect_issues(&parsed).unwrap();
        assert!(!issues.is_empty(), "Should detect at least one God Object");
        let issue = &issues[0];
        assert!(issue.description.contains("God Object"));
        assert_eq!(issue.file_path, file_path.to_string_lossy());
    }

    #[test]
    fn detects_god_object_python() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("god_object.py");
        let code = r#"
class GodObject:
    def __init__(self):
        self.a = 1
        self.b = 2
        self.c = 3
        self.d = 4
        self.e = 5
        self.f = 6
        self.g = 7
        self.h = 8
        self.i = 9
        self.j = 10
        self.k = 11
        self.l = 12
        self.m = 13
        self.n = 14
        self.o = 15
        self.p = 16
        self.q = 17
        self.r = 18
        self.s = 19
        self.t = 20
    def m1(self): pass
    def m2(self): pass
    def m3(self): pass
    def m4(self): pass
    def m5(self): pass
    def m6(self): pass
    def m7(self): pass
    def m8(self): pass
    def m9(self): pass
    def m10(self): pass
    def m11(self): pass
    def m12(self): pass
    def m13(self): pass
    def m14(self): pass
    def m15(self): pass
    def m16(self): pass
    def m17(self): pass
    def m18(self): pass
    def m19(self): pass
    def m20(self): pass
    def m21(self): pass
"#;
        File::create(&file_path).unwrap().write_all(code.as_bytes()).unwrap();
        let mut parser = AstParser::new().unwrap();
        let parsed = parser.parse_file(&file_path).unwrap();
        let detector = GodObjectDetector::new(20, 10);
        let issues = detector.detect_issues(&parsed).unwrap();
        assert!(!issues.is_empty(), "Should detect at least one God Object");
        let issue = &issues[0];
        assert!(issue.description.contains("God Object"));
        assert_eq!(issue.file_path, file_path.to_string_lossy());
    }

    #[test]
    fn detects_god_object_js() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("god_object.js");
        let code = r#"
class GodObject {
    constructor() {
        this.a = 1;
        this.b = 2;
        this.c = 3;
        this.d = 4;
        this.e = 5;
        this.f = 6;
        this.g = 7;
        this.h = 8;
        this.i = 9;
        this.j = 10;
        this.k = 11;
        this.l = 12;
        this.m = 13;
        this.n = 14;
        this.o = 15;
        this.p = 16;
        this.q = 17;
        this.r = 18;
        this.s = 19;
        this.t = 20;
    }
    m1() {}
    m2() {}
    m3() {}
    m4() {}
    m5() {}
    m6() {}
    m7() {}
    m8() {}
    m9() {}
    m10() {}
    m11() {}
    m12() {}
    m13() {}
    m14() {}
    m15() {}
    m16() {}
    m17() {}
    m18() {}
    m19() {}
    m20() {}
    m21() {}
}
"#;
        File::create(&file_path).unwrap().write_all(code.as_bytes()).unwrap();
        let mut parser = AstParser::new().unwrap();
        let parsed = parser.parse_file(&file_path).unwrap();
        let detector = GodObjectDetector::new(20, 10);
        let issues = detector.detect_issues(&parsed).unwrap();
        assert!(!issues.is_empty(), "Should detect at least one God Object");
        let issue = &issues[0];
        assert!(issue.description.contains("God Object"));
        assert_eq!(issue.file_path, file_path.to_string_lossy());
    }

    #[test]
    fn god_object_severity_and_snippet() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("god_object.rs");
        let code = r#"
        struct GodObject {
            a: i32,
            b: i32,
            c: i32,
        }
        impl GodObject {
            fn m1(&self) {}
            fn m2(&self) {}
            fn m3(&self) {}
            fn m4(&self) {}
            fn m5(&self) {}
            fn m6(&self) {}
            fn m7(&self) {}
            fn m8(&self) {}
            fn m9(&self) {}
            fn m10(&self) {}
            fn m11(&self) {}
            fn m12(&self) {}
            fn m13(&self) {}
            fn m14(&self) {}
            fn m15(&self) {}
            fn m16(&self) {}
            fn m17(&self) {}
            fn m18(&self) {}
            fn m19(&self) {}
            fn m20(&self) {}
            fn m21(&self) {}
        }
        "#;
        File::create(&file_path).unwrap().write_all(code.as_bytes()).unwrap();
        let mut parser = AstParser::new().unwrap();
        let parsed = parser.parse_file(&file_path).unwrap();
        let detector = GodObjectDetector::new(20, 10);
        let issues = detector.detect_issues(&parsed).unwrap();
        let issue = &issues[0];
        assert_eq!(issue.severity, "Medium");
        assert!(issue.code_snippet.is_some());
    }
}
