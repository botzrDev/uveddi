//! Plugin Template Generator
//!
//! This module provides functionality to generate plugin templates and scaffolding
//! for new Uveddi WASM plugins, making it easy for developers to get started.

use crate::error::{Result, UveddiError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Plugin template generator for creating new plugins
pub struct PluginTemplateGenerator {
    templates_dir: PathBuf,
    available_templates: HashMap<String, PluginTemplate>,
}

/// Template configuration for generating plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginTemplate {
    pub name: String,
    pub description: String,
    pub category: String,
    pub files: Vec<TemplateFile>,
    pub dependencies: Vec<String>,
    pub features: Vec<String>,
    pub example_code: bool,
    pub test_files: bool,
    pub documentation: bool,
}

/// Individual file in a plugin template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateFile {
    pub path: String,
    pub content_template: String,
    pub executable: bool,
}

/// Configuration for generating a new plugin from template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginGenerationConfig {
    pub template_name: String,
    pub plugin_id: String,
    pub plugin_name: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub version: String,
    pub supported_languages: Vec<String>,
    pub output_directory: PathBuf,
    pub include_examples: bool,
    pub include_tests: bool,
    pub include_docs: bool,
    pub custom_variables: HashMap<String, String>,
}

impl PluginTemplateGenerator {
    /// Create a new plugin template generator
    pub fn new<P: AsRef<Path>>(templates_dir: P) -> Result<Self> {
        let templates_dir = templates_dir.as_ref().to_path_buf();
        
        if !templates_dir.exists() {
            fs::create_dir_all(&templates_dir)?;
        }

        let mut generator = Self {
            templates_dir,
            available_templates: HashMap::new(),
        };

        // Initialize built-in templates
        generator.initialize_builtin_templates()?;
        
        Ok(generator)
    }

    /// Initialize built-in plugin templates
    fn initialize_builtin_templates(&mut self) -> Result<()> {
        // Basic detector template
        self.available_templates.insert("detector".to_string(), PluginTemplate {
            name: "Basic Detector".to_string(),
            description: "A basic analysis detector plugin template".to_string(),
            category: "detector".to_string(),
            files: vec![
                TemplateFile {
                    path: "src/lib.rs".to_string(),
                    content_template: include_str!("templates/detector/lib.rs.template").to_string(),
                    executable: false,
                },
                TemplateFile {
                    path: "Cargo.toml".to_string(),
                    content_template: include_str!("templates/detector/Cargo.toml.template").to_string(),
                    executable: false,
                },
                TemplateFile {
                    path: "plugin.toml".to_string(),
                    content_template: include_str!("templates/detector/plugin.toml.template").to_string(),
                    executable: false,
                },
            ],
            dependencies: vec!["wit-bindgen".to_string()],
            features: vec!["basic-analysis".to_string()],
            example_code: true,
            test_files: true,
            documentation: true,
        });

        // Security scanner template
        self.available_templates.insert("security".to_string(), PluginTemplate {
            name: "Security Scanner".to_string(),
            description: "Security vulnerability scanner plugin template".to_string(),
            category: "security".to_string(),
            files: vec![
                TemplateFile {
                    path: "src/lib.rs".to_string(),
                    content_template: include_str!("templates/security/lib.rs.template").to_string(),
                    executable: false,
                },
                TemplateFile {
                    path: "Cargo.toml".to_string(),
                    content_template: include_str!("templates/security/Cargo.toml.template").to_string(),
                    executable: false,
                },
                TemplateFile {
                    path: "plugin.toml".to_string(),
                    content_template: include_str!("templates/security/plugin.toml.template").to_string(),
                    executable: false,
                },
            ],
            dependencies: vec!["wit-bindgen".to_string(), "regex".to_string()],
            features: vec!["network-access".to_string(), "security-analysis".to_string()],
            example_code: true,
            test_files: true,
            documentation: true,
        });

        // Performance analyzer template
        self.available_templates.insert("performance".to_string(), PluginTemplate {
            name: "Performance Analyzer".to_string(),
            description: "Performance analysis plugin template".to_string(),
            category: "performance".to_string(),
            files: vec![
                TemplateFile {
                    path: "src/lib.rs".to_string(),
                    content_template: include_str!("templates/performance/lib.rs.template").to_string(),
                    executable: false,
                },
                TemplateFile {
                    path: "Cargo.toml".to_string(),
                    content_template: include_str!("templates/performance/Cargo.toml.template").to_string(),
                    executable: false,
                },
                TemplateFile {
                    path: "plugin.toml".to_string(),
                    content_template: include_str!("templates/performance/plugin.toml.template").to_string(),
                    executable: false,
                },
            ],
            dependencies: vec!["wit-bindgen".to_string()],
            features: vec!["ast-analysis".to_string(), "metrics".to_string()],
            example_code: true,
            test_files: true,
            documentation: true,
        });

        Ok(())
    }

    /// List available plugin templates
    pub fn list_templates(&self) -> Vec<&PluginTemplate> {
        self.available_templates.values().collect()
    }

    /// Get a specific template by name
    pub fn get_template(&self, name: &str) -> Option<&PluginTemplate> {
        self.available_templates.get(name)
    }

    /// Generate a new plugin from template
    pub fn generate_plugin(&self, config: &PluginGenerationConfig) -> Result<PathBuf> {
        let template = self.available_templates.get(&config.template_name)
            .ok_or_else(|| UveddiError::PluginError {
                plugin: config.plugin_id.clone(),
                plugin_type: "template".to_string(),
                message: format!("Template '{}' not found", config.template_name),
                suggestion: "Use 'uveddi plugin template list' to see available templates".to_string(),
                source: None,
            })?;

        let plugin_dir = config.output_directory.join(&config.plugin_id);
        
        if plugin_dir.exists() {
            return Err(UveddiError::PluginError {
                plugin: config.plugin_id.clone(),
                plugin_type: "template".to_string(),
                message: format!("Plugin directory already exists: {}", plugin_dir.display()),
                suggestion: "Choose a different plugin ID or remove the existing directory".to_string(),
                source: None,
            });
        }

        fs::create_dir_all(&plugin_dir)?;

        // Create template variables for substitution
        let mut variables = self.create_template_variables(config);
        variables.extend(config.custom_variables.clone());

        // Generate files from template
        for file in &template.files {
            let file_path = plugin_dir.join(&file.path);
            
            // Ensure parent directory exists
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)?;
            }

            // Process template content
            let content = self.process_template(&file.content_template, &variables)?;
            fs::write(&file_path, content)?;

            // Set executable permissions if needed
            #[cfg(unix)]
            if file.executable {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&file_path)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&file_path, perms)?;
            }
        }

        // Generate additional files if requested
        if config.include_examples {
            self.generate_example_files(&plugin_dir, &variables)?;
        }

        if config.include_tests {
            self.generate_test_files(&plugin_dir, &variables)?;
        }

        if config.include_docs {
            self.generate_documentation_files(&plugin_dir, &variables)?;
        }

        // Generate build script
        self.generate_build_script(&plugin_dir, &variables)?;

        println!("✅ Plugin '{}' generated successfully at {}", config.plugin_name, plugin_dir.display());
        println!("📁 Plugin directory: {}", plugin_dir.display());
        println!("🛠️  To build: cd {} && cargo build --target wasm32-wasi --release", plugin_dir.display());
        println!("🧪 To test: cd {} && cargo test", plugin_dir.display());

        Ok(plugin_dir)
    }

    /// Create template variables for substitution
    fn create_template_variables(&self, config: &PluginGenerationConfig) -> HashMap<String, String> {
        let mut variables = HashMap::new();
        
        variables.insert("PLUGIN_ID".to_string(), config.plugin_id.clone());
        variables.insert("PLUGIN_NAME".to_string(), config.plugin_name.clone());
        variables.insert("PLUGIN_DESCRIPTION".to_string(), config.description.clone());
        variables.insert("AUTHOR".to_string(), config.author.clone());
        variables.insert("LICENSE".to_string(), config.license.clone());
        variables.insert("VERSION".to_string(), config.version.clone());
        variables.insert("SUPPORTED_LANGUAGES".to_string(), 
            config.supported_languages.iter()
                .map(|lang| format!("\"{}\"", lang))
                .collect::<Vec<_>>()
                .join(", "));
        variables.insert("PLUGIN_ID_UPPER".to_string(), config.plugin_id.to_uppercase().replace("-", "_"));
        variables.insert("PLUGIN_ID_SNAKE".to_string(), config.plugin_id.replace("-", "_"));
        variables.insert("CURRENT_YEAR".to_string(), chrono::Utc::now().year().to_string());
        variables.insert("GENERATION_TIMESTAMP".to_string(), chrono::Utc::now().to_rfc3339());
        variables.insert("UUID".to_string(), Uuid::new_v4().to_string());

        variables
    }

    /// Process template content by substituting variables
    fn process_template(&self, template: &str, variables: &HashMap<String, String>) -> Result<String> {
        let mut content = template.to_string();
        
        for (key, value) in variables {
            let placeholder = format!("{{{}}}", key);
            content = content.replace(&placeholder, value);
        }

        Ok(content)
    }

    /// Generate example files for the plugin
    fn generate_example_files(&self, plugin_dir: &Path, variables: &HashMap<String, String>) -> Result<()> {
        let examples_dir = plugin_dir.join("examples");
        fs::create_dir_all(&examples_dir)?;

        // Generate test fixtures
        let fixtures_dir = examples_dir.join("fixtures");
        fs::create_dir_all(&fixtures_dir)?;

        // Create sample files for testing
        let sample_rust = r#"// Sample Rust code for testing
pub fn example_function() -> i32 {
    let mut result = 0;
    for i in 0..10 {
        result += i;
    }
    result
}

#[deprecated]
pub fn old_function() {
    println!("This function is deprecated");
}
"#;
        fs::write(fixtures_dir.join("sample.rs"), sample_rust)?;

        let sample_js = r#"// Sample JavaScript code for testing
function exampleFunction() {
    let result = 0;
    for (let i = 0; i < 10; i++) {
        result += i;
    }
    return result;
}

// Potential security issue
function dangerousFunction(userInput) {
    eval(userInput); // This is dangerous!
}
"#;
        fs::write(fixtures_dir.join("sample.js"), sample_js)?;

        Ok(())
    }

    /// Generate test files for the plugin
    fn generate_test_files(&self, plugin_dir: &Path, variables: &HashMap<String, String>) -> Result<()> {
        let tests_dir = plugin_dir.join("tests");
        fs::create_dir_all(&tests_dir)?;

        let test_content = format!(r#"//! Integration tests for {plugin_name} plugin

use std::path::PathBuf;

#[test]
fn test_plugin_initialization() {{
    // Test plugin initialization
    // This would normally use the plugin testing framework
    assert!(true);
}}

#[test] 
fn test_analysis_functionality() {{
    // Test core analysis functionality
    // Load test fixtures and verify expected results
    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("fixtures");
    
    assert!(fixtures_dir.exists());
}}

#[test]
fn test_configuration_handling() {{
    // Test plugin configuration
    assert!(true);
}}

#[test]
fn test_error_handling() {{
    // Test error conditions
    assert!(true);
}}
"#, plugin_name = variables.get("PLUGIN_NAME").unwrap_or(&"Plugin".to_string()));

        fs::write(tests_dir.join("integration_test.rs"), test_content)?;

        Ok(())
    }

    /// Generate documentation files
    fn generate_documentation_files(&self, plugin_dir: &Path, variables: &HashMap<String, String>) -> Result<()> {
        let docs_dir = plugin_dir.join("docs");
        fs::create_dir_all(&docs_dir)?;

        let readme_content = format!(r#"# {plugin_name}

{description}

## Installation

```bash
# Build the plugin
cargo build --target wasm32-wasi --release

# Install using Uveddi
uveddi plugin install target/wasm32-wasi/release/{plugin_id_snake}_plugin.wasm plugin.toml
```

## Usage

This plugin analyzes code and provides the following capabilities:

- [List key features here]
- [Add more features]

## Configuration

The plugin can be configured using the following options:

```toml
[plugin.configuration.options]
# Add configuration options here
```

## Rules

This plugin implements the following analysis rules:

- **RULE001**: Description of rule
- **RULE002**: Description of another rule

## Development

### Building

```bash
cargo build --target wasm32-wasi --release
```

### Testing

```bash
cargo test
```

### Contributing

Contributions are welcome! Please read the contributing guidelines before submitting PRs.

## License

This project is licensed under the {license} License - see the LICENSE file for details.

## Author

{author}
"#, 
    plugin_name = variables.get("PLUGIN_NAME").unwrap_or(&"Plugin".to_string()),
    description = variables.get("PLUGIN_DESCRIPTION").unwrap_or(&"A plugin for Uveddi".to_string()),
    plugin_id_snake = variables.get("PLUGIN_ID_SNAKE").unwrap_or(&"plugin".to_string()),
    license = variables.get("LICENSE").unwrap_or(&"MIT".to_string()),
    author = variables.get("AUTHOR").unwrap_or(&"Plugin Author".to_string())
);

        fs::write(plugin_dir.join("README.md"), readme_content)?;

        // Generate API documentation
        let api_docs = format!(r#"# {plugin_name} API Documentation

## Plugin Interface

This plugin implements the standard Uveddi plugin interface defined in the WIT specification.

### Exported Functions

#### `initialize(config: PluginConfig, limits: ResourceLimits) -> Result<(), String>`

Initializes the plugin with the provided configuration and resource limits.

#### `analyze(file: SourceFile) -> Result<AnalysisResult, String>`

Analyzes a source file and returns analysis results including issues and metrics.

#### `get_info() -> PluginInfo`

Returns information about the plugin including capabilities and metadata.

#### `cleanup() -> Result<(), String>`

Cleans up plugin resources and prepares for shutdown.

## Host Functions

This plugin uses the following host functions provided by Uveddi:

- `log(level: LogLevel, message: String)` - Logging
- `read_file(path: String) -> Result<String, String>` - File reading
- `parse_ast(code: String, language: String) -> Result<AstNode, String>` - AST parsing
- [Add other host functions used]

## Configuration Schema

```toml
[plugin.configuration.options]
# Document configuration options
```

## Error Handling

The plugin handles errors gracefully and provides meaningful error messages.

## Performance Considerations

- Memory usage: [Describe memory characteristics]
- Processing time: [Describe performance characteristics]
- Resource limits: [Describe resource requirements]
"#, plugin_name = variables.get("PLUGIN_NAME").unwrap_or(&"Plugin".to_string()));

        fs::write(docs_dir.join("api.md"), api_docs)?;

        Ok(())
    }

    /// Generate build script
    fn generate_build_script(&self, plugin_dir: &Path, variables: &HashMap<String, String>) -> Result<()> {
        let build_script = r#"#!/bin/bash
set -e

echo "Building plugin..."

# Ensure we have the wasm32-wasi target
rustup target add wasm32-wasi

# Build the plugin
cargo build --target wasm32-wasi --release

# Check if build succeeded
if [ -f "target/wasm32-wasi/release/{plugin_id_snake}_plugin.wasm" ]; then
    echo "✅ Plugin built successfully!"
    echo "📦 Plugin binary: target/wasm32-wasi/release/{plugin_id_snake}_plugin.wasm"
    echo "📋 Plugin manifest: plugin.toml"
    echo ""
    echo "To install: uveddi plugin install target/wasm32-wasi/release/{plugin_id_snake}_plugin.wasm plugin.toml"
else
    echo "❌ Build failed!"
    exit 1
fi
"#.replace("{plugin_id_snake}", variables.get("PLUGIN_ID_SNAKE").unwrap_or(&"plugin".to_string()));

        let build_script_path = plugin_dir.join("build.sh");
        fs::write(&build_script_path, build_script)?;

        // Make executable on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&build_script_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&build_script_path, perms)?;
        }

        Ok(())
    }

    /// Validate plugin generation configuration
    pub fn validate_config(&self, config: &PluginGenerationConfig) -> Result<()> {
        // Check if template exists
        if !self.available_templates.contains_key(&config.template_name) {
            return Err(UveddiError::PluginError {
                plugin: config.plugin_id.clone(),
                plugin_type: "template".to_string(),
                message: format!("Template '{}' not found", config.template_name),
                suggestion: format!("Available templates: {}", 
                    self.available_templates.keys().cloned().collect::<Vec<_>>().join(", ")),
                source: None,
            });
        }

        // Validate plugin ID format
        if !config.plugin_id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(UveddiError::PluginError {
                plugin: config.plugin_id.clone(),
                plugin_type: "template".to_string(),
                message: "Plugin ID contains invalid characters".to_string(),
                suggestion: "Use only alphanumeric characters, hyphens, and underscores".to_string(),
                source: None,
            });
        }

        // Check output directory permissions
        if config.output_directory.exists() {
            let metadata = fs::metadata(&config.output_directory)?;
            if metadata.permissions().readonly() {
                return Err(UveddiError::PluginError {
                    plugin: config.plugin_id.clone(),
                    plugin_type: "template".to_string(),
                    message: "Output directory is read-only".to_string(),
                    suggestion: "Choose a writable directory or change permissions".to_string(),
                    source: None,
                });
            }
        }

        Ok(())
    }

    /// Interactive plugin generator
    pub fn interactive_generation(&self) -> Result<PluginGenerationConfig> {
        println!("🚀 Uveddi Plugin Generator");
        println!("==========================");
        println!();

        // List available templates
        println!("Available templates:");
        for (name, template) in &self.available_templates {
            println!("  {} - {}", name, template.description);
        }
        println!();

        // Get user input (this would use a proper CLI input library in practice)
        let template_name = self.prompt("Template name")?;
        let plugin_id = self.prompt("Plugin ID (kebab-case)")?;
        let plugin_name = self.prompt("Plugin Name")?;
        let description = self.prompt("Description")?;
        let author = self.prompt_with_default("Author", "Plugin Author")?;
        let license = self.prompt_with_default("License", "MIT")?;
        let version = self.prompt_with_default("Version", "1.0.0")?;
        
        let supported_languages = vec!["rust".to_string(), "javascript".to_string(), "python".to_string()];
        let output_directory = PathBuf::from("./plugins");

        Ok(PluginGenerationConfig {
            template_name,
            plugin_id,
            plugin_name,
            description,
            author,
            license,
            version,
            supported_languages,
            output_directory,
            include_examples: true,
            include_tests: true,
            include_docs: true,
            custom_variables: HashMap::new(),
        })
    }

    fn prompt(&self, message: &str) -> Result<String> {
        // In a real implementation, this would use a CLI input library
        // For now, return placeholder values
        Ok(format!("example-{}", message.to_lowercase().replace(" ", "-")))
    }

    fn prompt_with_default(&self, message: &str, default: &str) -> Result<String> {
        // In a real implementation, this would use a CLI input library
        Ok(default.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_template_generator_creation() {
        let temp_dir = TempDir::new().unwrap();
        let generator = PluginTemplateGenerator::new(temp_dir.path()).unwrap();
        
        assert!(!generator.available_templates.is_empty());
        assert!(generator.available_templates.contains_key("detector"));
    }

    #[test]
    fn test_plugin_generation() {
        let temp_dir = TempDir::new().unwrap();
        let generator = PluginTemplateGenerator::new(temp_dir.path()).unwrap();
        
        let config = PluginGenerationConfig {
            template_name: "detector".to_string(),
            plugin_id: "test-plugin".to_string(),
            plugin_name: "Test Plugin".to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            license: "MIT".to_string(),
            version: "1.0.0".to_string(),
            supported_languages: vec!["rust".to_string()],
            output_directory: temp_dir.path().to_path_buf(),
            include_examples: true,
            include_tests: true,
            include_docs: true,
            custom_variables: HashMap::new(),
        };

        let result = generator.generate_plugin(&config);
        assert!(result.is_ok());
        
        let plugin_dir = result.unwrap();
        assert!(plugin_dir.exists());
        assert!(plugin_dir.join("src/lib.rs").exists());
        assert!(plugin_dir.join("Cargo.toml").exists());
        assert!(plugin_dir.join("plugin.toml").exists());
    }

    #[test]
    fn test_config_validation() {
        let temp_dir = TempDir::new().unwrap();
        let generator = PluginTemplateGenerator::new(temp_dir.path()).unwrap();
        
        let valid_config = PluginGenerationConfig {
            template_name: "detector".to_string(),
            plugin_id: "valid-plugin".to_string(),
            plugin_name: "Valid Plugin".to_string(),
            description: "A valid plugin".to_string(),
            author: "Author".to_string(),
            license: "MIT".to_string(),
            version: "1.0.0".to_string(),
            supported_languages: vec!["rust".to_string()],
            output_directory: temp_dir.path().to_path_buf(),
            include_examples: true,
            include_tests: true,
            include_docs: true,
            custom_variables: HashMap::new(),
        };

        assert!(generator.validate_config(&valid_config).is_ok());

        let invalid_config = PluginGenerationConfig {
            template_name: "nonexistent".to_string(),
            ..valid_config.clone()
        };

        assert!(generator.validate_config(&invalid_config).is_err());
    }
}