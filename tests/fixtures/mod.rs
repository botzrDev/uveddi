//! Test Fixtures and Mock Data for Uveddi
//!
//! This module provides comprehensive test fixtures and mock data for
//! all components of the Uveddi system, ensuring consistent and realistic
//! test data across all test suites.

pub mod code_samples;
pub mod analysis_results;
pub mod database_fixtures;
pub mod api_fixtures;

use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::json;

pub use code_samples::*;
pub use analysis_results::*;
pub use database_fixtures::*;
pub use api_fixtures::*;

/// Comprehensive test fixture builder for creating consistent test data
pub struct TestFixtureBuilder {
    base_timestamp: DateTime<Utc>,
    project_path: String,
    language_distribution: HashMap<String, usize>,
}

impl TestFixtureBuilder {
    /// Create a new fixture builder with default settings
    pub fn new() -> Self {
        Self {
            base_timestamp: Utc::now(),
            project_path: "/test/project".to_string(),
            language_distribution: HashMap::from([
                ("rust".to_string(), 60),
                ("javascript".to_string(), 25),
                ("python".to_string(), 15),
            ]),
        }
    }

    /// Set the base timestamp for all generated data
    pub fn with_timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.base_timestamp = timestamp;
        self
    }

    /// Set the project path for generated data
    pub fn with_project_path(mut self, path: impl Into<String>) -> Self {
        self.project_path = path.into();
        self
    }

    /// Set the language distribution for generated analysis results
    pub fn with_languages(mut self, languages: HashMap<String, usize>) -> Self {
        self.language_distribution = languages;
        self
    }

    /// Build a complete test scenario with all related data
    pub fn build_scenario(&self, name: &str) -> TestScenario {
        TestScenario {
            name: name.to_string(),
            analysis_result: self.build_analysis_result(),
            architectural_issues: self.build_architectural_issues(5),
            code_samples: self.build_code_samples(),
            api_responses: self.build_api_responses(),
        }
    }

    /// Build a single analysis result
    pub fn build_analysis_result(&self) -> crate::database::models::AnalysisResult {
        let total_files: usize = self.language_distribution.values().sum();

        crate::database::models::AnalysisResult {
            id: Uuid::new_v4(),
            project_path: self.project_path.clone(),
            analysis_timestamp: self.base_timestamp,
            total_files: total_files as i32,
            analyzed_files: (total_files - 5) as i32, // Simulate some files not analyzed
            total_issues: 42,
            critical_issues: 3,
            major_issues: 15,
            minor_issues: 24,
            info_issues: 0,
            analysis_duration_ms: 15000,
            language_breakdown: self.language_distribution.clone(),
            plugin_versions: HashMap::from([
                ("core".to_string(), "1.0.0".to_string()),
                ("security".to_string(), "0.9.0".to_string()),
                ("performance".to_string(), "0.8.0".to_string()),
            ]),
            status: "completed".to_string(),
            error_message: None,
            metadata: json!({
                "test_fixture": true,
                "created_by": "TestFixtureBuilder"
            }),
        }
    }

    /// Build architectural issues for testing
    pub fn build_architectural_issues(&self, count: usize) -> Vec<crate::database::models::ArchitecturalIssue> {
        let analysis_id = Uuid::new_v4();
        let issue_types = [
            ("god_object", "critical", "GodObjectDetector"),
            ("data_clumps", "major", "DataClumpsDetector"),
            ("cyclic_dependencies", "major", "CyclicDependenciesDetector"),
            ("long_method", "minor", "LongMethodDetector"),
            ("magic_numbers", "minor", "MagicNumberDetector"),
        ];

        (0..count)
            .map(|i| {
                let (issue_type, severity, detector) = &issue_types[i % issue_types.len()];
                crate::database::models::ArchitecturalIssue {
                    id: Uuid::new_v4(),
                    analysis_result_id: analysis_id,
                    anti_pattern_type_id: (i % 5 + 1) as i32,
                    file_path: format!("src/module_{}.rs", i),
                    start_line: Some((i * 10 + 1) as i32),
                    end_line: Some((i * 10 + 20) as i32),
                    description: format!("{} detected in {}", issue_type, format!("src/module_{}.rs", i)),
                    detector_name: detector.to_string(),
                    severity: severity.to_string(),
                    recommendation: format!("Consider refactoring to address the {} anti-pattern", issue_type),
                    code_snippet: Some(format!("// Code snippet for issue {}\nstruct Example{} {{\n    // problematic code here\n}}", i, i)),
                    detected_at: self.base_timestamp,
                    metadata: Some(json!({
                        "confidence": 0.85,
                        "complexity_score": i * 2 + 10
                    })),
                }
            })
            .collect()
    }

    /// Build code samples for different scenarios
    pub fn build_code_samples(&self) -> HashMap<String, String> {
        let mut samples = HashMap::new();

        // God Object example
        samples.insert(
            "god_object_rust".to_string(),
            GOD_OBJECT_RUST.to_string(),
        );

        // Data Clumps example
        samples.insert(
            "data_clumps_python".to_string(),
            DATA_CLUMPS_PYTHON.to_string(),
        );

        // Cyclic Dependencies example
        samples.insert(
            "cyclic_deps_javascript".to_string(),
            CYCLIC_DEPENDENCIES_JS.to_string(),
        );

        // Clean code examples
        samples.insert(
            "clean_rust".to_string(),
            CLEAN_RUST_CODE.to_string(),
        );

        samples.insert(
            "clean_python".to_string(),
            CLEAN_PYTHON_CODE.to_string(),
        );

        samples
    }

    /// Build API response fixtures
    pub fn build_api_responses(&self) -> HashMap<String, serde_json::Value> {
        let mut responses = HashMap::new();

        responses.insert(
            "health_response".to_string(),
            json!({
                "status": "ok",
                "service": "uveddi-api-server-alpha",
                "version": "0.9.0-alpha",
                "timestamp": self.base_timestamp.to_rfc3339()
            }),
        );

        responses.insert(
            "project_info_response".to_string(),
            json!({
                "name": "Uveddi",
                "version": "0.9.0-alpha",
                "description": "AI-Powered Code Analysis Tool",
                "features": [
                    "Multi-language analysis",
                    "AI-powered insights",
                    "Privacy-focused local analysis"
                ]
            }),
        );

        responses.insert(
            "reports_list_response".to_string(),
            json!({
                "reports": [
                    {
                        "id": Uuid::new_v4().to_string(),
                        "timestamp": self.base_timestamp.to_rfc3339(),
                        "summary": {
                            "filesAnalyzed": 100,
                            "issuesFound": 25
                        }
                    }
                ],
                "total": 1
            }),
        );

        responses
    }
}

impl Default for TestFixtureBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A complete test scenario with all related data
#[derive(Debug)]
pub struct TestScenario {
    pub name: String,
    pub analysis_result: crate::database::models::AnalysisResult,
    pub architectural_issues: Vec<crate::database::models::ArchitecturalIssue>,
    pub code_samples: HashMap<String, String>,
    pub api_responses: HashMap<String, serde_json::Value>,
}

impl TestScenario {
    /// Get a specific code sample by name
    pub fn get_code_sample(&self, name: &str) -> Option<&String> {
        self.code_samples.get(name)
    }

    /// Get a specific API response by name
    pub fn get_api_response(&self, name: &str) -> Option<&serde_json::Value> {
        self.api_responses.get(name)
    }

    /// Get issues by severity level
    pub fn get_issues_by_severity(&self, severity: &str) -> Vec<&crate::database::models::ArchitecturalIssue> {
        self.architectural_issues
            .iter()
            .filter(|issue| issue.severity == severity)
            .collect()
    }

    /// Get issues by detector name
    pub fn get_issues_by_detector(&self, detector: &str) -> Vec<&crate::database::models::ArchitecturalIssue> {
        self.architectural_issues
            .iter()
            .filter(|issue| issue.detector_name == detector)
            .collect()
    }
}

/// Predefined test scenarios for common testing situations
pub struct TestScenarios;

impl TestScenarios {
    /// Small project scenario for quick tests
    pub fn small_project() -> TestScenario {
        TestFixtureBuilder::new()
            .with_languages(HashMap::from([
                ("rust".to_string(), 10),
                ("javascript".to_string(), 5),
            ]))
            .build_scenario("small_project")
    }

    /// Large enterprise project scenario
    pub fn large_project() -> TestScenario {
        TestFixtureBuilder::new()
            .with_languages(HashMap::from([
                ("rust".to_string(), 500),
                ("javascript".to_string(), 300),
                ("python".to_string(), 200),
                ("typescript".to_string(), 150),
                ("go".to_string(), 50),
            ]))
            .build_scenario("large_project")
    }

    /// Legacy codebase with many issues
    pub fn legacy_codebase() -> TestScenario {
        let mut builder = TestFixtureBuilder::new()
            .with_languages(HashMap::from([
                ("javascript".to_string(), 200), // Legacy JS
                ("php".to_string(), 100),
                ("java".to_string(), 150),
            ]));

        let mut scenario = builder.build_scenario("legacy_codebase");

        // Simulate many issues in legacy code
        scenario.analysis_result.total_issues = 150;
        scenario.analysis_result.critical_issues = 25;
        scenario.analysis_result.major_issues = 50;
        scenario.analysis_result.minor_issues = 75;

        scenario.architectural_issues = builder.build_architectural_issues(20);

        scenario
    }

    /// Clean, well-structured project
    pub fn clean_project() -> TestScenario {
        let mut builder = TestFixtureBuilder::new()
            .with_languages(HashMap::from([
                ("rust".to_string(), 80),
                ("typescript".to_string(), 20),
            ]));

        let mut scenario = builder.build_scenario("clean_project");

        // Clean project has fewer issues
        scenario.analysis_result.total_issues = 5;
        scenario.analysis_result.critical_issues = 0;
        scenario.analysis_result.major_issues = 1;
        scenario.analysis_result.minor_issues = 4;

        scenario.architectural_issues = builder.build_architectural_issues(2);

        scenario
    }

    /// Multi-language microservices architecture
    pub fn microservices() -> TestScenario {
        TestFixtureBuilder::new()
            .with_languages(HashMap::from([
                ("rust".to_string(), 100), // Core services
                ("go".to_string(), 80),    // API gateways
                ("javascript".to_string(), 60), // Frontend
                ("python".to_string(), 40), // ML services
                ("yaml".to_string(), 20),  // Config files
            ]))
            .build_scenario("microservices")
    }
}

/// Utility functions for test data generation
pub mod utils {
    use super::*;

    /// Generate a random UUID for testing
    pub fn random_uuid() -> Uuid {
        Uuid::new_v4()
    }

    /// Generate a timestamp in the past for testing
    pub fn past_timestamp(hours_ago: i64) -> DateTime<Utc> {
        Utc::now() - chrono::Duration::hours(hours_ago)
    }

    /// Generate a future timestamp for testing
    pub fn future_timestamp(hours_from_now: i64) -> DateTime<Utc> {
        Utc::now() + chrono::Duration::hours(hours_from_now)
    }

    /// Create a temporary test file with given content
    pub fn create_temp_file(content: &str) -> std::io::Result<tempfile::NamedTempFile> {
        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::new()?;
        temp_file.write_all(content.as_bytes())?;
        temp_file.flush()?;
        Ok(temp_file)
    }

    /// Create a temporary test directory with multiple files
    pub fn create_temp_project(files: HashMap<String, String>) -> std::io::Result<tempfile::TempDir> {
        use std::fs;
        use std::io::Write;

        let temp_dir = tempfile::TempDir::new()?;

        for (file_path, content) in files {
            let full_path = temp_dir.path().join(&file_path);

            // Create parent directories if needed
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let mut file = fs::File::create(full_path)?;
            file.write_all(content.as_bytes())?;
        }

        Ok(temp_dir)
    }

    /// Generate realistic file path for testing
    pub fn generate_file_path(language: &str, module: &str) -> String {
        match language.to_lowercase().as_str() {
            "rust" => format!("src/{}.rs", module),
            "python" => format!("{}.py", module),
            "javascript" => format!("src/{}.js", module),
            "typescript" => format!("src/{}.ts", module),
            "go" => format!("{}.go", module),
            "java" => format!("src/main/java/{}.java", module),
            "csharp" => format!("{}.cs", module),
            _ => format!("{}.txt", module),
        }
    }

    /// Calculate expected metrics for test validation
    pub fn calculate_expected_metrics(
        files: &HashMap<String, String>,
    ) -> HashMap<String, usize> {
        let mut metrics = HashMap::new();

        metrics.insert("total_files".to_string(), files.len());
        metrics.insert(
            "total_lines".to_string(),
            files.values().map(|content| content.lines().count()).sum(),
        );
        metrics.insert(
            "total_chars".to_string(),
            files.values().map(|content| content.len()).sum(),
        );

        // Language-specific metrics
        let mut rust_files = 0;
        let mut js_files = 0;
        let mut py_files = 0;

        for (path, _) in files {
            if path.ends_with(".rs") {
                rust_files += 1;
            } else if path.ends_with(".js") || path.ends_with(".ts") {
                js_files += 1;
            } else if path.ends_with(".py") {
                py_files += 1;
            }
        }

        metrics.insert("rust_files".to_string(), rust_files);
        metrics.insert("javascript_files".to_string(), js_files);
        metrics.insert("python_files".to_string(), py_files);

        metrics
    }
}