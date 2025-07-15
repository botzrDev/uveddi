#[cfg(feature = "tui")]
// TUI Form Validation and Data Conversion Tests
//
// This module tests the form validation logic and data conversion
// between TUI form inputs and backend command structures.
// Focuses on ensuring data integrity and proper error handling.
use std::path::PathBuf;

use uveddi::cli::analyze_command::AnalyzeCommand;

/// Mock form data structure that simulates TUI form inputs
#[derive(Debug, Clone)]
struct MockAnalyzeFormData {
    pub path: String,
    pub output_format: String,
    pub output_file: Option<String>,
    pub enable_ai: bool,
    pub ollama_api_url: Option<String>,
    pub ollama_model: Option<String>,
    pub dead_code_confidence: Option<String>, // String to simulate user input
    pub dead_code_library_mode: bool,
    pub dead_code_ignore_patterns: Option<String>, // Comma-separated string
    pub dead_code_keep_alive: Option<String>,      // Comma-separated string
    pub large_classes_max_loc: Option<String>,     // String to simulate user input
    pub large_classes_max_methods: Option<String>,
    pub large_classes_max_fields: Option<String>,
    pub large_classes_max_complexity: Option<String>,
    pub large_classes_max_lcom: Option<String>,
    pub large_classes_ignore_patterns: Option<String>,
    pub large_classes_min_severity: Option<String>,
}

impl MockAnalyzeFormData {
    /// Create a new form with default values
    pub fn new() -> Self {
        Self {
            path: String::new(),
            output_format: "markdown".to_string(),
            output_file: None,
            enable_ai: false,
            ollama_api_url: None,
            ollama_model: None,
            dead_code_confidence: None,
            dead_code_library_mode: false,
            dead_code_ignore_patterns: None,
            dead_code_keep_alive: None,
            large_classes_max_loc: None,
            large_classes_max_methods: None,
            large_classes_max_fields: None,
            large_classes_max_complexity: None,
            large_classes_max_lcom: None,
            large_classes_ignore_patterns: None,
            large_classes_min_severity: None,
        }
    }

    /// Convert form data to AnalyzeCommand with validation
    pub fn to_analyze_command(&self) -> Result<AnalyzeCommand, String> {
        // Validate required fields
        if self.path.trim().is_empty() {
            return Err("Path is required".to_string());
        }

        let path = std::path::PathBuf::from(&self.path);
        if !path.exists() {
            return Err("The specified path does not exist".to_string());
        }

        // Parse numeric fields with validation
        let dead_code_confidence = if let Some(ref conf_str) = self.dead_code_confidence {
            if conf_str.trim().is_empty() {
                None
            } else {
                match conf_str.parse::<f64>() {
                    Ok(val) => {
                        if val < 0.0 || val > 1.0 {
                            return Err(
                                "Dead code confidence must be between 0.0 and 1.0".to_string()
                            );
                        }
                        Some(val)
                    }
                    Err(_) => return Err("Dead code confidence must be a valid number".to_string()),
                }
            }
        } else {
            None
        };

        let large_classes_max_loc = if let Some(ref loc_str) = self.large_classes_max_loc {
            if loc_str.trim().is_empty() {
                None
            } else {
                match loc_str.parse::<u32>() {
                    Ok(val) => {
                        if val == 0 {
                            return Err("Maximum LOC must be greater than 0".to_string());
                        }
                        Some(val)
                    }
                    Err(_) => return Err("Maximum LOC must be a valid positive number".to_string()),
                }
            }
        } else {
            None
        };

        let large_classes_max_methods = if let Some(ref methods_str) =
            self.large_classes_max_methods
        {
            if methods_str.trim().is_empty() {
                None
            } else {
                match methods_str.parse::<u32>() {
                    Ok(val) => {
                        if val == 0 {
                            return Err("Maximum methods must be greater than 0".to_string());
                        }
                        Some(val)
                    }
                    Err(_) => {
                        return Err("Maximum methods must be a valid positive number".to_string())
                    }
                }
            }
        } else {
            None
        };

        let large_classes_max_fields = if let Some(ref fields_str) = self.large_classes_max_fields {
            if fields_str.trim().is_empty() {
                None
            } else {
                match fields_str.parse::<u32>() {
                    Ok(val) => {
                        if val == 0 {
                            return Err("Maximum fields must be greater than 0".to_string());
                        }
                        Some(val)
                    }
                    Err(_) => {
                        return Err("Maximum fields must be a valid positive number".to_string())
                    }
                }
            }
        } else {
            None
        };

        let large_classes_max_complexity = if let Some(ref complexity_str) =
            self.large_classes_max_complexity
        {
            if complexity_str.trim().is_empty() {
                None
            } else {
                match complexity_str.parse::<u32>() {
                    Ok(val) => {
                        if val == 0 {
                            return Err("Maximum complexity must be greater than 0".to_string());
                        }
                        Some(val)
                    }
                    Err(_) => {
                        return Err("Maximum complexity must be a valid positive number".to_string())
                    }
                }
            }
        } else {
            None
        };

        let large_classes_max_lcom = if let Some(ref lcom_str) = self.large_classes_max_lcom {
            if lcom_str.trim().is_empty() {
                None
            } else {
                match lcom_str.parse::<f64>() {
                    Ok(val) => {
                        if val < 0.0 || val > 1.0 {
                            return Err("Maximum LCOM must be between 0.0 and 1.0".to_string());
                        }
                        Some(val)
                    }
                    Err(_) => return Err("Maximum LCOM must be a valid number".to_string()),
                }
            }
        } else {
            None
        };

        let large_classes_min_severity = if let Some(ref severity_str) =
            self.large_classes_min_severity
        {
            if severity_str.trim().is_empty() {
                None
            } else {
                match severity_str.parse::<u32>() {
                    Ok(val) => {
                        if val > 100 {
                            return Err("Minimum severity must be between 0 and 100".to_string());
                        }
                        Some(val)
                    }
                    Err(_) => return Err("Minimum severity must be a valid number".to_string()),
                }
            }
        } else {
            None
        };

        // Parse comma-separated pattern lists
        let parse_patterns = |input: &Option<String>| -> Option<Vec<String>> {
            input.as_ref().and_then(|s| {
                if s.trim().is_empty() {
                    None
                } else {
                    Some(
                        s.split(',')
                            .map(|p| p.trim().to_string())
                            .filter(|p| !p.is_empty())
                            .collect(),
                    )
                }
            })
        };

        let dead_code_ignore_patterns = parse_patterns(&self.dead_code_ignore_patterns);
        let dead_code_keep_alive = parse_patterns(&self.dead_code_keep_alive);
        let large_classes_ignore_patterns = parse_patterns(&self.large_classes_ignore_patterns);

        // Validate output format
        let valid_formats = ["text", "json", "markdown"];
        if !valid_formats.contains(&self.output_format.as_str()) {
            return Err(format!(
                "Invalid output format: {}. Valid formats are: {}",
                self.output_format,
                valid_formats.join(", ")
            ));
        }

        // Create the command
        Ok(AnalyzeCommand {
            path,
            output_format: self.output_format.clone(),
            output: self.output_file.as_ref().map(std::path::PathBuf::from),
            enable_ai: self.enable_ai,
            ollama_api_url: self.ollama_api_url.clone(),
            ollama_model: self.ollama_model.clone(),
            dead_code_confidence,
            dead_code_library_mode: self.dead_code_library_mode,
            dead_code_ignore_patterns,
            dead_code_keep_alive,
            large_classes_max_loc,
            large_classes_max_methods,
            large_classes_max_fields,
            large_classes_max_complexity,
            large_classes_max_lcom,
            large_classes_ignore_patterns,
            large_classes_min_severity,
            enable_memory_optimization: false,
            memory_limit_gb: None,
            memory_profile: None,
        })
    }
}

/// Helper function to create a test file for validation
async fn create_test_file() -> std::io::Result<PathBuf> {
    let test_dir = std::path::PathBuf::from("./tmp/form_validation_test");
    tokio::fs::create_dir_all(&test_dir).await?;

    let test_file = test_dir.join("test.rs");
    tokio::fs::write(&test_file, "fn main() { println!(\"test\"); }").await?;

    Ok(test_file)
}

#[tokio::test]
async fn test_form_validation_empty_path() {
    let form = MockAnalyzeFormData::new();
    let result = form.to_analyze_command();

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Path is required");
}

#[tokio::test]
async fn test_form_validation_nonexistent_path() {
    let mut form = MockAnalyzeFormData::new();
    form.path = "/nonexistent/path".to_string();

    let result = form.to_analyze_command();

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "The specified path does not exist");
}

#[tokio::test]
async fn test_form_validation_valid_path() {
    let test_file = create_test_file().await.unwrap();
    let mut form = MockAnalyzeFormData::new();
    form.path = test_file.to_string_lossy().to_string();

    let result = form.to_analyze_command();

    assert!(result.is_ok());
    let command = result.unwrap();
    assert_eq!(command.path, test_file);

    // Cleanup
    tokio::fs::remove_dir_all("./tmp/form_validation_test")
        .await
        .ok();
}

#[tokio::test]
async fn test_form_validation_invalid_confidence() {
    let test_file = create_test_file().await.unwrap();
    let mut form = MockAnalyzeFormData::new();
    form.path = test_file.to_string_lossy().to_string();
    form.dead_code_confidence = Some("1.5".to_string()); // Invalid: > 1.0

    let result = form.to_analyze_command();

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Dead code confidence must be between 0.0 and 1.0"
    );

    // Cleanup
    tokio::fs::remove_dir_all("./tmp/form_validation_test")
        .await
        .ok();
}

#[tokio::test]
async fn test_form_validation_invalid_confidence_format() {
    let test_file = create_test_file().await.unwrap();
    let mut form = MockAnalyzeFormData::new();
    form.path = test_file.to_string_lossy().to_string();
    form.dead_code_confidence = Some("not_a_number".to_string());

    let result = form.to_analyze_command();

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Dead code confidence must be a valid number"
    );

    // Cleanup
    tokio::fs::remove_dir_all("./tmp/form_validation_test")
        .await
        .ok();
}

#[tokio::test]
async fn test_form_validation_valid_confidence() {
    let test_file = create_test_file().await.unwrap();
    let mut form = MockAnalyzeFormData::new();
    form.path = test_file.to_string_lossy().to_string();
    form.dead_code_confidence = Some("0.8".to_string());

    let result = form.to_analyze_command();

    assert!(result.is_ok());
    let command = result.unwrap();
    assert_eq!(command.dead_code_confidence, Some(0.8));

    // Cleanup
    tokio::fs::remove_dir_all("./tmp/form_validation_test")
        .await
        .ok();
}

#[tokio::test]
async fn test_form_validation_invalid_output_format() {
    let test_file = create_test_file().await.unwrap();
    let mut form = MockAnalyzeFormData::new();
    form.path = test_file.to_string_lossy().to_string();
    form.output_format = "invalid_format".to_string();

    let result = form.to_analyze_command();

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid output format"));

    // Cleanup
    tokio::fs::remove_dir_all("./tmp/form_validation_test")
        .await
        .ok();
}

#[tokio::test]
async fn test_form_validation_zero_numeric_fields() {
    let test_file = create_test_file().await.unwrap();
    let mut form = MockAnalyzeFormData::new();
    form.path = test_file.to_string_lossy().to_string();
    form.large_classes_max_loc = Some("0".to_string());

    let result = form.to_analyze_command();

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Maximum LOC must be greater than 0");

    // Cleanup
    tokio::fs::remove_dir_all("./tmp/form_validation_test")
        .await
        .ok();
}

#[tokio::test]
async fn test_form_validation_pattern_parsing() {
    let test_file = create_test_file().await.unwrap();
    let mut form = MockAnalyzeFormData::new();
    form.path = test_file.to_string_lossy().to_string();
    form.dead_code_ignore_patterns = Some("test, spec,mock,  generated  ".to_string());

    let result = form.to_analyze_command();

    assert!(result.is_ok());
    let command = result.unwrap();
    let patterns = command.dead_code_ignore_patterns.unwrap();
    assert_eq!(patterns, vec!["test", "spec", "mock", "generated"]);

    // Cleanup
    tokio::fs::remove_dir_all("./tmp/form_validation_test")
        .await
        .ok();
}

#[tokio::test]
async fn test_form_validation_empty_patterns() {
    let test_file = create_test_file().await.unwrap();
    let mut form = MockAnalyzeFormData::new();
    form.path = test_file.to_string_lossy().to_string();
    form.dead_code_ignore_patterns = Some("   ".to_string()); // Only whitespace

    let result = form.to_analyze_command();

    assert!(result.is_ok());
    let command = result.unwrap();
    assert!(command.dead_code_ignore_patterns.is_none());

    // Cleanup
    tokio::fs::remove_dir_all("./tmp/form_validation_test")
        .await
        .ok();
}

#[tokio::test]
async fn test_form_validation_severity_range() {
    let test_file = create_test_file().await.unwrap();
    let mut form = MockAnalyzeFormData::new();
    form.path = test_file.to_string_lossy().to_string();
    form.large_classes_min_severity = Some("101".to_string()); // Invalid: > 100

    let result = form.to_analyze_command();

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Minimum severity must be between 0 and 100"
    );

    // Cleanup
    tokio::fs::remove_dir_all("./tmp/form_validation_test")
        .await
        .ok();
}

#[tokio::test]
async fn test_form_validation_lcom_range() {
    let test_file = create_test_file().await.unwrap();
    let mut form = MockAnalyzeFormData::new();
    form.path = test_file.to_string_lossy().to_string();
    form.large_classes_max_lcom = Some("1.5".to_string()); // Invalid: > 1.0

    let result = form.to_analyze_command();

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Maximum LCOM must be between 0.0 and 1.0"
    );

    // Cleanup
    tokio::fs::remove_dir_all("./tmp/form_validation_test")
        .await
        .ok();
}

#[tokio::test]
async fn test_form_validation_complete_valid_form() {
    let test_file = create_test_file().await.unwrap();
    let mut form = MockAnalyzeFormData::new();
    form.path = test_file.to_string_lossy().to_string();
    form.output_format = "json".to_string();
    form.output_file = Some("output.json".to_string());
    form.enable_ai = true;
    form.ollama_api_url = Some("http://localhost:11434".to_string());
    form.ollama_model = Some("deepseek-coder:6.7b".to_string());
    form.dead_code_confidence = Some("0.9".to_string());
    form.dead_code_library_mode = true;
    form.dead_code_ignore_patterns = Some("test,spec".to_string());
    form.dead_code_keep_alive = Some("main,init".to_string());
    form.large_classes_max_loc = Some("500".to_string());
    form.large_classes_max_methods = Some("25".to_string());
    form.large_classes_max_fields = Some("20".to_string());
    form.large_classes_max_complexity = Some("60".to_string());
    form.large_classes_max_lcom = Some("0.8".to_string());
    form.large_classes_ignore_patterns = Some("generated,autogen".to_string());
    form.large_classes_min_severity = Some("30".to_string());

    let result = form.to_analyze_command();

    assert!(result.is_ok());
    let command = result.unwrap();

    // Verify all fields were parsed correctly
    assert_eq!(command.path, test_file);
    assert_eq!(command.output_format, "json");
    assert_eq!(
        command.output,
        Some(std::path::PathBuf::from("output.json"))
    );
    assert!(command.enable_ai);
    assert_eq!(
        command.ollama_api_url,
        Some("http://localhost:11434".to_string())
    );
    assert_eq!(
        command.ollama_model,
        Some("deepseek-coder:6.7b".to_string())
    );
    assert_eq!(command.dead_code_confidence, Some(0.9));
    assert!(command.dead_code_library_mode);
    assert_eq!(
        command.dead_code_ignore_patterns,
        Some(vec!["test".to_string(), "spec".to_string()])
    );
    assert_eq!(
        command.dead_code_keep_alive,
        Some(vec!["main".to_string(), "init".to_string()])
    );
    assert_eq!(command.large_classes_max_loc, Some(500));
    assert_eq!(command.large_classes_max_methods, Some(25));
    assert_eq!(command.large_classes_max_fields, Some(20));
    assert_eq!(command.large_classes_max_complexity, Some(60));
    assert_eq!(command.large_classes_max_lcom, Some(0.8));
    assert_eq!(
        command.large_classes_ignore_patterns,
        Some(vec!["generated".to_string(), "autogen".to_string()])
    );
    assert_eq!(command.large_classes_min_severity, Some(30));

    // Cleanup
    tokio::fs::remove_dir_all("./tmp/form_validation_test")
        .await
        .ok();
}

#[tokio::test]
async fn test_form_validation_optional_fields_empty() {
    let test_file = create_test_file().await.unwrap();
    let mut form = MockAnalyzeFormData::new();
    form.path = test_file.to_string_lossy().to_string();
    // Leave all optional fields as None or empty

    let result = form.to_analyze_command();

    assert!(result.is_ok());
    let command = result.unwrap();

    // Verify optional fields are None
    assert_eq!(command.dead_code_confidence, None);
    assert_eq!(command.dead_code_ignore_patterns, None);
    assert_eq!(command.dead_code_keep_alive, None);
    assert_eq!(command.large_classes_max_loc, None);
    assert_eq!(command.large_classes_max_methods, None);
    assert_eq!(command.large_classes_max_fields, None);
    assert_eq!(command.large_classes_max_complexity, None);
    assert_eq!(command.large_classes_max_lcom, None);
    assert_eq!(command.large_classes_ignore_patterns, None);
    assert_eq!(command.large_classes_min_severity, None);

    // Cleanup
    tokio::fs::remove_dir_all("./tmp/form_validation_test")
        .await
        .ok();
}

/// Test boundary values for numeric inputs
#[tokio::test]
async fn test_form_validation_boundary_values() {
    let test_file = create_test_file().await.unwrap();

    // Test minimum valid confidence
    let mut form = MockAnalyzeFormData::new();
    form.path = test_file.to_string_lossy().to_string();
    form.dead_code_confidence = Some("0.0".to_string());
    assert!(form.to_analyze_command().is_ok());

    // Test maximum valid confidence
    form.dead_code_confidence = Some("1.0".to_string());
    assert!(form.to_analyze_command().is_ok());

    // Test minimum invalid confidence
    form.dead_code_confidence = Some("-0.1".to_string());
    assert!(form.to_analyze_command().is_err());

    // Test maximum invalid confidence
    form.dead_code_confidence = Some("1.1".to_string());
    assert!(form.to_analyze_command().is_err());

    // Test minimum valid severity
    form.dead_code_confidence = None;
    form.large_classes_min_severity = Some("0".to_string());
    assert!(form.to_analyze_command().is_ok());

    // Test maximum valid severity
    form.large_classes_min_severity = Some("100".to_string());
    assert!(form.to_analyze_command().is_ok());

    // Test invalid severity
    form.large_classes_min_severity = Some("101".to_string());
    assert!(form.to_analyze_command().is_err());

    // Cleanup
    tokio::fs::remove_dir_all("./tmp/form_validation_test")
        .await
        .ok();
}
