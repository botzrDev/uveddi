//! Output configuration for report format, file paths, and template options

use crate::error::UveddiError;
use std::path::PathBuf;

/// Configuration for output format and file handling
#[derive(Debug, Clone)]
pub struct OutputConfig {
    /// The desired output format for the analysis report (e.g., "json", "markdown", "html")
    pub format: String,
    /// An optional path to a file where the report should be saved
    pub file_path: Option<PathBuf>,
    /// Template options for custom report formatting
    pub template_options: TemplateOptions,
}

/// Template configuration options for report generation
#[derive(Debug, Clone)]
pub struct TemplateOptions {
    /// Custom CSS styles for HTML reports
    pub custom_css: Option<PathBuf>,
    /// Custom JavaScript for HTML reports
    pub custom_js: Option<PathBuf>,
    /// Custom template file path
    pub template_file: Option<PathBuf>,
    /// Enable syntax highlighting in reports
    pub syntax_highlighting: bool,
    /// Include source code snippets in reports
    pub include_snippets: bool,
    /// Maximum lines to include in code snippets
    pub max_snippet_lines: u32,
}

impl OutputConfig {
    /// Create a new output configuration
    pub fn new(format: String, file_path: Option<PathBuf>) -> Self {
        Self {
            format,
            file_path,
            template_options: TemplateOptions::default(),
        }
    }

    /// Validate the output configuration
    pub fn validate(&self) -> Result<(), UveddiError> {
        // Validate output format
        match self.format.as_str() {
            "json" | "markdown" | "html" => {},
            _ => {
                return Err(UveddiError::config_error(
                    &format!("Unsupported output format: {}", self.format),
                    "output format validation",
                ));
            }
        }

        // Validate file path if provided
        if let Some(ref path) = self.file_path {
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    return Err(UveddiError::PathError {
                        path: parent.display().to_string(),
                        reason: "Output directory does not exist".to_string(),
                        suggestion: "Create the directory or choose an existing path".to_string(),
                    });
                }
            }
        }

        // Validate template options
        self.template_options.validate()?;

        Ok(())
    }

    /// Merge with another output configuration
    pub fn merge_with(mut self, other: OutputConfig) -> Self {
        // Override format if other is not default
        if other.format != "json" {
            self.format = other.format;
        }

        // Use other's file path if provided
        self.file_path = other.file_path.or(self.file_path);

        // Merge template options
        self.template_options = self.template_options.merge_with(other.template_options);

        self
    }

    /// Get the appropriate file extension for the output format
    pub fn get_file_extension(&self) -> &str {
        match self.format.as_str() {
            "json" => "json",
            "markdown" => "md",
            "html" => "html",
            _ => "txt",
        }
    }

    /// Generate a default output file name if none is provided
    pub fn generate_default_filename(&self, base_name: &str) -> PathBuf {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("{}_{}.{}", base_name, timestamp, self.get_file_extension());
        PathBuf::from(filename)
    }

    /// Check if HTML-specific features are enabled
    pub fn is_html_format(&self) -> bool {
        self.format == "html"
    }

    /// Check if interactive features should be enabled
    pub fn supports_interactive_features(&self) -> bool {
        self.is_html_format() && (
            self.template_options.custom_js.is_some() ||
            self.template_options.syntax_highlighting
        )
    }
}

impl TemplateOptions {
    /// Create new template options with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate template options
    pub fn validate(&self) -> Result<(), UveddiError> {
        // Validate custom CSS file exists if provided
        if let Some(ref css_path) = self.custom_css {
            if !css_path.exists() {
                return Err(UveddiError::PathError {
                    path: css_path.display().to_string(),
                    reason: "Custom CSS file does not exist".to_string(),
                    suggestion: "Verify the CSS file path is correct".to_string(),
                });
            }
        }

        // Validate custom JavaScript file exists if provided
        if let Some(ref js_path) = self.custom_js {
            if !js_path.exists() {
                return Err(UveddiError::PathError {
                    path: js_path.display().to_string(),
                    reason: "Custom JavaScript file does not exist".to_string(),
                    suggestion: "Verify the JavaScript file path is correct".to_string(),
                });
            }
        }

        // Validate custom template file exists if provided
        if let Some(ref template_path) = self.template_file {
            if !template_path.exists() {
                return Err(UveddiError::PathError {
                    path: template_path.display().to_string(),
                    reason: "Custom template file does not exist".to_string(),
                    suggestion: "Verify the template file path is correct".to_string(),
                });
            }
        }

        // Validate snippet line limit
        if self.max_snippet_lines == 0 {
            return Err(UveddiError::config_error(
                "Maximum snippet lines must be greater than 0",
                "template options validation",
            ));
        }

        Ok(())
    }

    /// Merge with another template options configuration
    pub fn merge_with(mut self, other: TemplateOptions) -> Self {
        self.custom_css = other.custom_css.or(self.custom_css);
        self.custom_js = other.custom_js.or(self.custom_js);
        self.template_file = other.template_file.or(self.template_file);

        // Override boolean flags if other explicitly sets them
        if other.syntax_highlighting {
            self.syntax_highlighting = other.syntax_highlighting;
        }
        if other.include_snippets {
            self.include_snippets = other.include_snippets;
        }
        if other.max_snippet_lines != 50 { // 50 is default
            self.max_snippet_lines = other.max_snippet_lines;
        }

        self
    }

    /// Enable all interactive features for rich HTML reports
    pub fn enable_interactive(&mut self) -> &mut Self {
        self.syntax_highlighting = true;
        self.include_snippets = true;
        self
    }

    /// Configure for minimal, fast reports
    pub fn configure_minimal(&mut self) -> &mut Self {
        self.syntax_highlighting = false;
        self.include_snippets = false;
        self.max_snippet_lines = 10;
        self
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: "json".to_string(),
            file_path: None,
            template_options: TemplateOptions::default(),
        }
    }
}

impl Default for TemplateOptions {
    fn default() -> Self {
        Self {
            custom_css: None,
            custom_js: None,
            template_file: None,
            syntax_highlighting: true,
            include_snippets: true,
            max_snippet_lines: 50,
        }
    }
}