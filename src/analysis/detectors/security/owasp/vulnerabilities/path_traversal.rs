//! Path Traversal Vulnerability Detector
//!
//! Specialized detector for path traversal (directory traversal) vulnerabilities
//! that allow attackers to access files outside the intended directory structure.

use crate::analysis::detectors::security::owasp::types::{
    OwaspCategory, OwaspCategoryDetector, OwaspVulnerability,
};
use crate::analysis::detectors::security::types::{
    SecurityIssueType, SecurityLocation, SecuritySeverity, VulnerabilityMetadata,
};
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;

/// Path traversal pattern definition
#[derive(Debug, Clone)]
pub struct PathTraversalPattern {
    pub pattern: String,
    pub description: String,
    pub confidence: f64,
    pub severity: SecuritySeverity,
    pub traversal_type: PathTraversalType,
    pub context: String,
}

#[derive(Debug, Clone)]
pub enum PathTraversalType {
    DirectoryTraversal,
    FileInclusion,
    ZipSlip,
    SymlinkTraversal,
    UrlTraversal,
    RelativePathManipulation,
}

impl PathTraversalType {
    fn description(&self) -> &'static str {
        match self {
            PathTraversalType::DirectoryTraversal => "Directory traversal vulnerability",
            PathTraversalType::FileInclusion => "File inclusion vulnerability",
            PathTraversalType::ZipSlip => "Zip slip vulnerability",
            PathTraversalType::SymlinkTraversal => "Symbolic link traversal vulnerability",
            PathTraversalType::UrlTraversal => "URL path traversal vulnerability",
            PathTraversalType::RelativePathManipulation => {
                "Relative path manipulation vulnerability"
            }
        }
    }
}

/// Specialized path traversal detector
pub struct PathTraversalDetector {
    patterns: HashMap<SourceLanguage, Vec<PathTraversalPattern>>,
}

impl PathTraversalDetector {
    pub fn new() -> Self {
        Self {
            patterns: Self::init_patterns(),
        }
    }

    fn init_patterns() -> HashMap<SourceLanguage, Vec<PathTraversalPattern>> {
        let mut patterns = HashMap::new();

        patterns.insert(SourceLanguage::Rust, Self::rust_patterns());
        patterns.insert(SourceLanguage::Python, Self::python_patterns());
        patterns.insert(SourceLanguage::JavaScript, Self::javascript_patterns());
        patterns.insert(SourceLanguage::TypeScript, Self::javascript_patterns());

        patterns
    }

    fn rust_patterns() -> Vec<PathTraversalPattern> {
        vec![
            PathTraversalPattern {
                pattern: r#"File::open\(.*\+.*\)"#.to_string(),
                description: "File path constructed with user input".to_string(),
                confidence: 0.8,
                severity: SecuritySeverity::High,
                traversal_type: PathTraversalType::DirectoryTraversal,
                context: "File::open with concatenation".to_string(),
            },
            PathTraversalPattern {
                pattern: r#"fs::read\(.*format!"#.to_string(),
                description: "File read with formatted path".to_string(),
                confidence: 0.85,
                severity: SecuritySeverity::High,
                traversal_type: PathTraversalType::DirectoryTraversal,
                context: "fs::read with formatting".to_string(),
            },
            PathTraversalPattern {
                pattern: r#"Path::new\(.*\+.*\)"#.to_string(),
                description: "Path construction with string concatenation".to_string(),
                confidence: 0.75,
                severity: SecuritySeverity::Medium,
                traversal_type: PathTraversalType::RelativePathManipulation,
                context: "Path::new with concatenation".to_string(),
            },
            PathTraversalPattern {
                pattern: r#"include_str!\(.*\+.*\)"#.to_string(),
                description: "Include file with dynamic path".to_string(),
                confidence: 0.9,
                severity: SecuritySeverity::High,
                traversal_type: PathTraversalType::FileInclusion,
                context: "include_str! macro".to_string(),
            },
            PathTraversalPattern {
                pattern: r#"\.\./"#.to_string(),
                description: "Hardcoded directory traversal sequence".to_string(),
                confidence: 0.6,
                severity: SecuritySeverity::Medium,
                traversal_type: PathTraversalType::DirectoryTraversal,
                context: "Hardcoded traversal".to_string(),
            },
        ]
    }

    fn python_patterns() -> Vec<PathTraversalPattern> {
        vec![
            PathTraversalPattern {
                pattern: r#"open\(.*\+.*\)"#.to_string(),
                description: "File open with string concatenation".to_string(),
                confidence: 0.8,
                severity: SecuritySeverity::High,
                traversal_type: PathTraversalType::DirectoryTraversal,
                context: "open() with concatenation".to_string(),
            },
            PathTraversalPattern {
                pattern: r#"open\(f".*"#.to_string(),
                description: "File open with f-string formatting".to_string(),
                confidence: 0.85,
                severity: SecuritySeverity::High,
                traversal_type: PathTraversalType::DirectoryTraversal,
                context: "open() with f-string".to_string(),
            },
            PathTraversalPattern {
                pattern: r#"os\.path\.join\(.*input.*\)"#.to_string(),
                description: "Path join with user input".to_string(),
                confidence: 0.7,
                severity: SecuritySeverity::Medium,
                traversal_type: PathTraversalType::RelativePathManipulation,
                context: "os.path.join with input".to_string(),
            },
            PathTraversalPattern {
                pattern: r#"zipfile\.extractall\(.*\+.*\)"#.to_string(),
                description: "Zip extraction with dynamic path".to_string(),
                confidence: 0.9,
                severity: SecuritySeverity::Critical,
                traversal_type: PathTraversalType::ZipSlip,
                context: "zipfile.extractall".to_string(),
            },
            PathTraversalPattern {
                pattern: r#"os\.symlink\(.*\+.*\)"#.to_string(),
                description: "Symbolic link creation with user input".to_string(),
                confidence: 0.85,
                severity: SecuritySeverity::High,
                traversal_type: PathTraversalType::SymlinkTraversal,
                context: "os.symlink".to_string(),
            },
            PathTraversalPattern {
                pattern: r#"__import__\(.*\+.*\)"#.to_string(),
                description: "Dynamic import with user input".to_string(),
                confidence: 0.9,
                severity: SecuritySeverity::Critical,
                traversal_type: PathTraversalType::FileInclusion,
                context: "__import__ function".to_string(),
            },
        ]
    }

    fn javascript_patterns() -> Vec<PathTraversalPattern> {
        vec![
            PathTraversalPattern {
                pattern: r#"fs\.readFile\(.*\+.*\)"#.to_string(),
                description: "File read with string concatenation".to_string(),
                confidence: 0.8,
                severity: SecuritySeverity::High,
                traversal_type: PathTraversalType::DirectoryTraversal,
                context: "fs.readFile with concatenation".to_string(),
            },
            PathTraversalPattern {
                pattern: r#"fs\.readFile\(`.*\$\{.*\}.*`\)"#.to_string(),
                description: "File read with template literals".to_string(),
                confidence: 0.85,
                severity: SecuritySeverity::High,
                traversal_type: PathTraversalType::DirectoryTraversal,
                context: "fs.readFile with template literals".to_string(),
            },
            PathTraversalPattern {
                pattern: r#"path\.join\(.*req\..*\)"#.to_string(),
                description: "Path join with request parameters".to_string(),
                confidence: 0.8,
                severity: SecuritySeverity::High,
                traversal_type: PathTraversalType::RelativePathManipulation,
                context: "path.join with request data".to_string(),
            },
            PathTraversalPattern {
                pattern: r#"require\(.*\+.*\)"#.to_string(),
                description: "Dynamic require with user input".to_string(),
                confidence: 0.9,
                severity: SecuritySeverity::Critical,
                traversal_type: PathTraversalType::FileInclusion,
                context: "require() function".to_string(),
            },
            PathTraversalPattern {
                pattern: r#"express\.static\(.*\+.*\)"#.to_string(),
                description: "Static file serving with dynamic path".to_string(),
                confidence: 0.85,
                severity: SecuritySeverity::High,
                traversal_type: PathTraversalType::DirectoryTraversal,
                context: "Express static middleware".to_string(),
            },
            PathTraversalPattern {
                pattern: r#"res\.sendFile\(.*\+.*\)"#.to_string(),
                description: "Send file with concatenated path".to_string(),
                confidence: 0.9,
                severity: SecuritySeverity::High,
                traversal_type: PathTraversalType::DirectoryTraversal,
                context: "Express sendFile".to_string(),
            },
        ]
    }

    fn analyze_line(
        &self,
        line: &str,
        line_number: usize,
        patterns: &[PathTraversalPattern],
    ) -> Vec<OwaspVulnerability> {
        let mut vulnerabilities = Vec::new();

        for pattern in patterns {
            if let Ok(regex) = regex::Regex::new(&pattern.pattern) {
                if regex.is_match(line) {
                    let location = SecurityLocation::new(
                        PathBuf::from("unknown"), // Will be updated by caller
                        line_number,
                        0,
                    );

                    let mut metadata = VulnerabilityMetadata::new();
                    metadata.add_metadata(
                        "traversal_type".to_string(),
                        pattern.traversal_type.description().to_string(),
                    );
                    metadata.add_metadata("context".to_string(), pattern.context.clone());
                    metadata.add_metadata("pattern_matched".to_string(), pattern.pattern.clone());
                    metadata.add_metadata("line_content".to_string(), line.trim().to_string());

                    // Add additional analysis for path traversal indicators
                    if line.contains("../") || line.contains("..\\") {
                        metadata.add_metadata(
                            "contains_traversal_sequence".to_string(),
                            "true".to_string(),
                        );
                    }

                    let remediation =
                        Self::generate_remediation(&pattern.traversal_type, &pattern.context);

                    let vulnerability = OwaspVulnerability::new(
                        OwaspCategory::BrokenAccessControl, // Path traversal is often categorized under access control
                        SecurityIssueType::PathTraversal,
                        format!("Path Traversal: {}", pattern.traversal_type.description()),
                        pattern.description.clone(),
                        location,
                    )
                    .with_severity(pattern.severity)
                    .with_confidence(pattern.confidence)
                    .with_remediation(remediation)
                    .with_metadata(metadata);

                    vulnerabilities.push(vulnerability);
                }
            }
        }

        vulnerabilities
    }

    fn generate_remediation(traversal_type: &PathTraversalType, context: &str) -> String {
        let base_advice = match traversal_type {
            PathTraversalType::DirectoryTraversal => {
                "Validate and sanitize file paths. Use a whitelist of allowed directories."
            }
            PathTraversalType::FileInclusion => {
                "Use a whitelist of allowed files for inclusion. Avoid dynamic file inclusion."
            }
            PathTraversalType::ZipSlip => {
                "Validate extraction paths and ensure they stay within the intended directory."
            }
            PathTraversalType::SymlinkTraversal => {
                "Validate symbolic link targets and restrict link creation permissions."
            }
            PathTraversalType::UrlTraversal => {
                "Validate URL paths and implement proper access controls."
            }
            PathTraversalType::RelativePathManipulation => {
                "Use absolute paths or properly normalize relative paths before use."
            }
        };

        let context_advice = match context {
            c if c.contains("concatenation") => {
                " Avoid string concatenation for path construction."
            }
            c if c.contains("template") || c.contains("f-string") => {
                " Use safe path joining methods instead of string interpolation."
            }
            c if c.contains("join") => " Validate path components before joining.",
            c if c.contains("static") => " Configure static file serving with proper restrictions.",
            _ => "",
        };

        format!(
            "{}{} Consider using a security library for path validation and canonicalization.",
            base_advice, context_advice
        )
    }
}

#[async_trait]
impl OwaspCategoryDetector for PathTraversalDetector {
    async fn detect(&self, file: &ParsedFile) -> Result<Vec<OwaspVulnerability>, AnalysisError> {
        let patterns = match self.patterns.get(&file.language) {
            Some(patterns) => patterns,
            None => return Ok(Vec::new()),
        };

        let mut vulnerabilities = Vec::new();

        for (line_number, line) in file.content.lines().enumerate() {
            let mut line_vulnerabilities = self.analyze_line(line, line_number + 1, patterns);

            // Update file path in location
            for vuln in &mut line_vulnerabilities {
                vuln.location.file_path = file.file_path.to_path_buf();
            }

            vulnerabilities.extend(line_vulnerabilities);
        }

        Ok(vulnerabilities)
    }
}

impl Default for PathTraversalDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_path_traversal_rust() {
        let detector = PathTraversalDetector::new();
        let content = r#"
            let file = File::open(base_path + &user_file);
            let content = fs::read(format!("/data/{}", filename));
            let path = Path::new(root + "/" + user_input);
        "#;

        let file = ParsedFile {
            file_path: Arc::new(PathBuf::from("test.rs")),
            language: SourceLanguage::Rust,
            content: content.to_string(),
            tree: None,
        };

        let vulnerabilities = detector.detect(&file).await.unwrap();
        assert!(vulnerabilities.len() >= 3);
        assert!(vulnerabilities
            .iter()
            .any(|v| v.description.contains("File::open")));
    }

    #[tokio::test]
    async fn test_path_traversal_python() {
        let detector = PathTraversalDetector::new();
        let content = r#"
            with open(f"/data/{filename}", 'r') as f:
                data = f.read()
            zipfile.extractall(target_dir + user_path)
            os.symlink(source + dest_link)
        "#;

        let file = ParsedFile {
            file_path: Arc::new(PathBuf::from("test.py")),
            language: SourceLanguage::Python,
            content: content.to_string(),
            tree: None,
        };

        let vulnerabilities = detector.detect(&file).await.unwrap();
        assert!(vulnerabilities.len() >= 2);
        assert!(vulnerabilities.iter().any(|v| v
            .metadata
            .get_metadata("traversal_type")
            .unwrap()
            .contains("Zip slip")));
    }

    #[tokio::test]
    async fn test_path_traversal_javascript() {
        let detector = PathTraversalDetector::new();
        let content = r#"
            fs.readFile(`/uploads/${req.params.filename}`, callback);
            app.use(express.static(baseDir + userDir));
            res.sendFile(path.join(__dirname, req.query.file));
        "#;

        let file = ParsedFile {
            file_path: Arc::new(PathBuf::from("test.js")),
            language: SourceLanguage::JavaScript,
            content: content.to_string(),
            tree: None,
        };

        let vulnerabilities = detector.detect(&file).await.unwrap();
        assert!(vulnerabilities.len() >= 3);
        assert!(vulnerabilities
            .iter()
            .any(|v| v.description.contains("Express")));
    }

    #[test]
    fn test_remediation_generation() {
        let remediation = PathTraversalDetector::generate_remediation(
            &PathTraversalType::ZipSlip,
            "zipfile.extractall",
        );
        assert!(remediation.contains("extraction paths"));
        assert!(remediation.contains("intended directory"));
    }
}
