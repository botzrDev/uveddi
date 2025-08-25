//! Interactive initialization command
//!
//! Provides guided setup with project templates and smart configuration
//! generation to reduce "time to first success" from minutes to seconds.

use crate::config::{Config, DeadCodeConfig, LargeClassConfig, LanguageThresholds};
use crate::core::UveddiError;
use clap::Args;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use dirs;

#[derive(Args, Debug)]
pub struct InitCommand {
    /// Path to initialize Uveddi configuration
    #[arg(default_value = ".")]
    pub path: PathBuf,
    
    /// Skip interactive prompts and use defaults
    #[arg(long)]
    pub non_interactive: bool,
    
    /// Project template to use
    #[arg(long, value_enum)]
    pub template: Option<ProjectTemplate>,
    
    /// Force overwrite existing configuration
    #[arg(long)]
    pub force: bool,
    
    /// Generate Git hooks for automated analysis
    #[arg(long)]
    pub git_hooks: bool,
}

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum ProjectTemplate {
    /// Rust project (Cargo-based)
    Rust,
    /// Python project (pip/poetry-based)
    Python,
    /// JavaScript/Node.js project
    JavaScript,
    /// TypeScript project
    TypeScript,
    /// Web project (mixed JS/TS/HTML/CSS)
    Web,
    /// Library project (multi-language)
    Library,
    /// Monorepo with multiple languages
    Monorepo,
    /// Custom configuration
    Custom,
}

pub struct ProjectAnalyzer {
    path: PathBuf,
}

#[derive(Debug)]
pub struct ProjectInfo {
    pub languages: Vec<String>,
    pub project_type: ProjectTemplate,
    pub has_tests: bool,
    pub has_docs: bool,
    pub estimated_size: ProjectSize,
    pub package_managers: Vec<String>,
    pub build_tools: Vec<String>,
}

#[derive(Debug)]
pub enum ProjectSize {
    Small,   // < 1k LOC
    Medium,  // 1k-10k LOC
    Large,   // 10k-100k LOC
    Huge,    // 100k+ LOC
}

impl InitCommand {
    pub async fn execute(&self) -> Result<(), UveddiError> {
        println!("🚀 Uveddi Project Initialization");
        println!("=================================\n");
        
        // Check if configuration already exists
        let config_path = self.path.join("uveddi.toml");
        if config_path.exists() && !self.force {
            return Err(UveddiError::Config(format!(
                "Configuration already exists at {}. Use --force to overwrite.",
                config_path.display()
            )));
        }
        
        // Analyze the project
        let analyzer = ProjectAnalyzer::new(&self.path);
        let project_info = analyzer.analyze().await?;
        
        println!("📊 Project Analysis Complete:");
        println!("   Languages: {}", project_info.languages.join(", "));
        println!("   Type: {:?}", project_info.project_type);
        println!("   Size: {:?}", project_info.estimated_size);
        if !project_info.package_managers.is_empty() {
            println!("   Package Managers: {}", project_info.package_managers.join(", "));
        }
        println!();
        
        // Determine template
        let template = if let Some(template) = &self.template {
            template.clone()
        } else if self.non_interactive {
            project_info.project_type
        } else {
            self.prompt_for_template(&project_info)?
        };
        
        // Generate configuration
        let config = if self.non_interactive {
            self.generate_config_from_template(&template, &project_info)
        } else {
            self.interactive_config_generation(&template, &project_info)?
        };
        
        // Write configuration
        self.write_config(&config, &config_path)?;
        
        // Setup Git hooks if requested
        if self.git_hooks {
            self.setup_git_hooks().await?;
        }
        
        // Generate project-specific documentation
        self.generate_project_docs(&template, &project_info)?;
        
        println!("✅ Uveddi initialization complete!");
        println!("📝 Configuration saved to: {}", config_path.display());
        println!("\n🚀 Ready to analyze! Try:");
        println!("   uveddi analyze {}", self.path.display());
        if self.git_hooks {
            println!("   Git hooks installed - analysis will run automatically on commits");
        }
        
        Ok(())
    }
    
    fn prompt_for_template(&self, project_info: &ProjectInfo) -> Result<ProjectTemplate, UveddiError> {
        println!("📋 Select Project Template:");
        println!("   1. Rust project");
        println!("   2. Python project");
        println!("   3. JavaScript project");  
        println!("   4. TypeScript project");
        println!("   5. Web project (mixed)");
        println!("   6. Library project");
        println!("   7. Monorepo");
        println!("   8. Custom configuration");
        println!("   9. Auto-detect (recommended)");
        
        print!("\nChoose template [9]: ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).map_err(|e| {
            UveddiError::Config(format!("Failed to read input: {}", e))
        })?;
        
        let choice = input.trim();
        
        match choice {
            "1" => Ok(ProjectTemplate::Rust),
            "2" => Ok(ProjectTemplate::Python),
            "3" => Ok(ProjectTemplate::JavaScript),
            "4" => Ok(ProjectTemplate::TypeScript),
            "5" => Ok(ProjectTemplate::Web),
            "6" => Ok(ProjectTemplate::Library),
            "7" => Ok(ProjectTemplate::Monorepo),
            "8" => Ok(ProjectTemplate::Custom),
            "9" | "" => Ok(project_info.project_type.clone()),
            _ => {
                println!("Invalid choice, using auto-detected template");
                Ok(project_info.project_type.clone())
            }
        }
    }
    
    fn interactive_config_generation(&self, template: &ProjectTemplate, project_info: &ProjectInfo) -> Result<Config, UveddiError> {
        let mut config = self.generate_config_from_template(template, project_info);
        
        println!("🔧 Configuration Options:");
        
        // AI Model Configuration
        if self.prompt_yes_no("Enable AI-powered analysis", true)? {
            let model = self.prompt_for_ai_model()?;
            config.ollama_model = Some(model);
        }
        
        // Dead Code Detection
        if self.prompt_yes_no("Configure dead code detection", true)? {
            config.dead_code = Some(self.prompt_dead_code_config()?);
        }
        
        // Large Classes Detection
        if self.prompt_yes_no("Configure large classes detection", true)? {
            config.large_classes = Some(self.prompt_large_classes_config(template, project_info)?);
        }
        
        Ok(config)
    }
    
    fn prompt_yes_no(&self, prompt: &str, default: bool) -> Result<bool, UveddiError> {
        let default_str = if default { "Y/n" } else { "y/N" };
        print!("{} [{}]: ", prompt, default_str);
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).map_err(|e| {
            UveddiError::Config(format!("Failed to read input: {}", e))
        })?;
        
        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => Ok(true),
            "n" | "no" => Ok(false),
            "" => Ok(default),
            _ => Ok(default),
        }
    }
    
    fn prompt_for_ai_model(&self) -> Result<String, UveddiError> {
        println!("\n🤖 AI Model Selection:");
        println!("   1. deepseek-coder:6.7b-instruct-q4_0 (recommended for code analysis)");
        println!("   2. codellama:7b-instruct (general purpose)");
        println!("   3. llama2:7b-chat (lightweight)");
        println!("   4. Custom model");
        
        print!("\nChoose AI model [1]: ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).map_err(|e| {
            UveddiError::Config(format!("Failed to read input: {}", e))
        })?;
        
        match input.trim() {
            "1" | "" => Ok("deepseek-coder:6.7b-instruct-q4_0".to_string()),
            "2" => Ok("codellama:7b-instruct".to_string()),
            "3" => Ok("llama2:7b-chat".to_string()),
            "4" => {
                print!("Enter custom model name: ");
                io::stdout().flush().unwrap();
                let mut custom_input = String::new();
                io::stdin().read_line(&mut custom_input).map_err(|e| {
                    UveddiError::Config(format!("Failed to read input: {}", e))
                })?;
                Ok(custom_input.trim().to_string())
            },
            _ => Ok("deepseek-coder:6.7b-instruct-q4_0".to_string()),
        }
    }
    
    fn prompt_dead_code_config(&self) -> Result<DeadCodeConfig, UveddiError> {
        println!("\n💀 Dead Code Detection Configuration:");
        
        let confidence_threshold = if self.prompt_yes_no("Use default confidence threshold (0.8)", true)? {
            Some(0.8)
        } else {
            print!("Enter confidence threshold (0.0-1.0): ");
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input).map_err(|e| {
                UveddiError::Config(format!("Failed to read input: {}", e))
            })?;
            input.trim().parse().ok()
        };
        
        let library_mode = self.prompt_yes_no("Enable library mode (analyze exported symbols)", false)?;
        
        Ok(DeadCodeConfig {
            confidence_threshold,
            library_mode: Some(library_mode),
            ignore_patterns: Some(vec![
                "test/**".to_string(),
                "tests/**".to_string(),
                "*_test.rs".to_string(),
                "*_test.py".to_string(),
                "*.test.js".to_string(),
                "*.test.ts".to_string(),
            ]),
            keep_alive_patterns: Some(vec![
                "main".to_string(),
                "lib".to_string(),
                "pub".to_string(),
            ]),
        })
    }
    
    fn prompt_large_classes_config(&self, template: &ProjectTemplate, project_info: &ProjectInfo) -> Result<LargeClassConfig, UveddiError> {
        println!("\n📏 Large Classes Detection Configuration:");
        
        let (default_loc, default_methods) = match template {
            ProjectTemplate::Rust => (200, 20),
            ProjectTemplate::Python => (300, 30),
            ProjectTemplate::JavaScript | ProjectTemplate::TypeScript => (250, 25),
            ProjectTemplate::Web => (400, 35),
            _ => (250, 25),
        };
        
        let max_logical_loc = if self.prompt_yes_no(&format!("Use default LOC threshold ({})", default_loc), true)? {
            Some(default_loc)
        } else {
            print!("Enter maximum logical LOC: ");
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input).map_err(|e| {
                UveddiError::Config(format!("Failed to read input: {}", e))
            })?;
            input.trim().parse().ok()
        };
        
        let max_methods = if self.prompt_yes_no(&format!("Use default methods threshold ({})", default_methods), true)? {
            Some(default_methods)
        } else {
            print!("Enter maximum methods count: ");
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input).map_err(|e| {
                UveddiError::Config(format!("Failed to read input: {}", e))
            })?;
            input.trim().parse().ok()
        };
        
        Ok(LargeClassConfig {
            max_logical_loc,
            max_methods,
            max_fields: Some(50),
            max_cyclomatic_complexity: Some(20),
            max_cognitive_complexity: Some(25),
            max_lcom_score: Some(0.8),
            max_coupling: Some(30),
            ignore_patterns: Some(vec![
                "test/**".to_string(),
                "tests/**".to_string(),
                "generated/**".to_string(),
            ]),
            language_overrides: Some(self.generate_language_overrides(template)),
        })
    }
    
    fn generate_language_overrides(&self, template: &ProjectTemplate) -> HashMap<String, LanguageThresholds> {
        let mut overrides = HashMap::new();
        
        match template {
            ProjectTemplate::Web | ProjectTemplate::Monorepo => {
                overrides.insert("javascript".to_string(), LanguageThresholds {
                    max_logical_loc: Some(200),
                    max_methods: Some(20),
                    max_fields: Some(30),
                    max_cyclomatic_complexity: Some(15),
                    max_cognitive_complexity: Some(20),
                    max_lcom_score: Some(0.7),
                    max_coupling: Some(25),
                });
                
                overrides.insert("typescript".to_string(), LanguageThresholds {
                    max_logical_loc: Some(250),
                    max_methods: Some(25),
                    max_fields: Some(35),
                    max_cyclomatic_complexity: Some(18),
                    max_cognitive_complexity: Some(22),
                    max_lcom_score: Some(0.75),
                    max_coupling: Some(28),
                });
            },
            _ => {}
        }
        
        overrides
    }
    
    fn generate_config_from_template(&self, template: &ProjectTemplate, project_info: &ProjectInfo) -> Config {
        let ai_model = match project_info.estimated_size {
            ProjectSize::Small => Some("deepseek-coder:6.7b-instruct-q4_0".to_string()),
            ProjectSize::Medium => Some("deepseek-coder:6.7b-instruct-q4_0".to_string()),
            ProjectSize::Large => Some("codellama:7b-instruct".to_string()),
            ProjectSize::Huge => Some("llama2:7b-chat".to_string()),
        };
        
        let dead_code_config = DeadCodeConfig {
            confidence_threshold: Some(0.8),
            library_mode: Some(matches!(template, ProjectTemplate::Library)),
            ignore_patterns: Some(self.generate_ignore_patterns(template)),
            keep_alive_patterns: Some(self.generate_keep_alive_patterns(template)),
        };
        
        let large_classes_config = LargeClassConfig {
            max_logical_loc: Some(match template {
                ProjectTemplate::Rust => 200,
                ProjectTemplate::Python => 300,
                ProjectTemplate::JavaScript | ProjectTemplate::TypeScript => 250,
                ProjectTemplate::Web => 400,
                ProjectTemplate::Library => 150,
                ProjectTemplate::Monorepo => 350,
                ProjectTemplate::Custom => 250,
            }),
            max_methods: Some(match template {
                ProjectTemplate::Rust => 20,
                ProjectTemplate::Python => 30,
                ProjectTemplate::JavaScript | ProjectTemplate::TypeScript => 25,
                ProjectTemplate::Web => 35,
                ProjectTemplate::Library => 15,
                ProjectTemplate::Monorepo => 40,
                ProjectTemplate::Custom => 25,
            }),
            max_fields: Some(50),
            max_cyclomatic_complexity: Some(20),
            max_cognitive_complexity: Some(25),
            max_lcom_score: Some(0.8),
            max_coupling: Some(30),
            ignore_patterns: Some(self.generate_ignore_patterns(template)),
            language_overrides: Some(self.generate_language_overrides(template)),
        };
        
        Config {
            ollama_model: ai_model,
            dead_code: Some(dead_code_config),
            large_classes: Some(large_classes_config),
        }
    }
    
    fn generate_ignore_patterns(&self, template: &ProjectTemplate) -> Vec<String> {
        let mut patterns = vec![
            "test/**".to_string(),
            "tests/**".to_string(),
            "node_modules/**".to_string(),
            "target/**".to_string(),
            "dist/**".to_string(),
            "build/**".to_string(),
        ];
        
        match template {
            ProjectTemplate::Rust => {
                patterns.extend(vec![
                    "target/**".to_string(),
                    "**/target/**".to_string(),
                    "Cargo.lock".to_string(),
                ]);
            },
            ProjectTemplate::Python => {
                patterns.extend(vec![
                    "__pycache__/**".to_string(),
                    "*.pyc".to_string(),
                    ".venv/**".to_string(),
                    "venv/**".to_string(),
                ]);
            },
            ProjectTemplate::JavaScript | ProjectTemplate::TypeScript | ProjectTemplate::Web => {
                patterns.extend(vec![
                    "node_modules/**".to_string(),
                    "dist/**".to_string(),
                    "build/**".to_string(),
                    ".next/**".to_string(),
                ]);
            },
            _ => {}
        }
        
        patterns
    }
    
    fn generate_keep_alive_patterns(&self, template: &ProjectTemplate) -> Vec<String> {
        match template {
            ProjectTemplate::Rust => vec![
                "main".to_string(),
                "lib".to_string(),
                "pub fn".to_string(),
                "pub struct".to_string(),
                "pub enum".to_string(),
            ],
            ProjectTemplate::Python => vec![
                "__main__".to_string(),
                "__init__".to_string(),
                "def main".to_string(),
                "class ".to_string(),
            ],
            ProjectTemplate::JavaScript | ProjectTemplate::TypeScript => vec![
                "export".to_string(),
                "module.exports".to_string(),
                "function main".to_string(),
                "class ".to_string(),
            ],
            _ => vec!["main".to_string(), "export".to_string(), "public".to_string()],
        }
    }
    
    fn write_config(&self, config: &Config, path: &Path) -> Result<(), UveddiError> {
        let toml_content = toml::to_string_pretty(config)
            .map_err(|e| UveddiError::Config(format!("Failed to serialize config: {}", e)))?;
        
        // Add header comment
        let header = format!(
            r#"# Uveddi Configuration File
# Generated by: uveddi init
# Generated on: {}
# 
# This file configures Uveddi's analysis behavior for your project.
# You can modify these settings or regenerate this file with 'uveddi init --force'
# 
# For more information, see: https://github.com/botzrDev/uveddi

"#,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );
        
        let full_content = format!("{}{}", header, toml_content);
        
        fs::write(path, full_content)
            .map_err(|e| UveddiError::Config(format!("Failed to write config file: {}", e)))?;
        
        Ok(())
    }
    
    async fn setup_git_hooks(&self) -> Result<(), UveddiError> {
        let git_dir = self.path.join(".git");
        if !git_dir.exists() {
            println!("⚠️  No Git repository found. Skipping Git hooks setup.");
            return Ok(());
        }
        
        let hooks_dir = git_dir.join("hooks");
        fs::create_dir_all(&hooks_dir)
            .map_err(|e| UveddiError::Config(format!("Failed to create hooks directory: {}", e)))?;
        
        // Pre-commit hook
        let pre_commit_content = r#"#!/bin/sh
# Uveddi pre-commit hook
# Automatically runs code analysis on staged files

echo "🔍 Running Uveddi analysis..."
uveddi analyze . --progress-format silent

# Check exit code
if [ $? -ne 0 ]; then
    echo "❌ Uveddi analysis found issues. Commit blocked."
    echo "💡 Review the issues and fix them, or use 'git commit --no-verify' to skip."
    exit 1
fi

echo "✅ Uveddi analysis passed!"
exit 0
"#;
        
        let pre_commit_path = hooks_dir.join("pre-commit");
        fs::write(&pre_commit_path, pre_commit_content)
            .map_err(|e| UveddiError::Config(format!("Failed to write pre-commit hook: {}", e)))?;
        
        // Make executable (Unix systems)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&pre_commit_path)
                .map_err(|e| UveddiError::Config(format!("Failed to read hook permissions: {}", e)))?
                .permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&pre_commit_path, perms)
                .map_err(|e| UveddiError::Config(format!("Failed to set hook permissions: {}", e)))?;
        }
        
        println!("🔧 Git pre-commit hook installed successfully!");
        
        Ok(())
    }
    
    fn generate_project_docs(&self, template: &ProjectTemplate, project_info: &ProjectInfo) -> Result<(), UveddiError> {
        let docs_dir = self.path.join("docs").join("uveddi");
        fs::create_dir_all(&docs_dir)
            .map_err(|e| UveddiError::Config(format!("Failed to create docs directory: {}", e)))?;
        
        // Generate README
        let readme_content = self.generate_readme_content(template, project_info);
        fs::write(docs_dir.join("README.md"), readme_content)
            .map_err(|e| UveddiError::Config(format!("Failed to write README: {}", e)))?;
        
        // Generate analysis guide
        let guide_content = self.generate_analysis_guide(template);
        fs::write(docs_dir.join("ANALYSIS_GUIDE.md"), guide_content)
            .map_err(|e| UveddiError::Config(format!("Failed to write analysis guide: {}", e)))?;
        
        println!("📚 Project documentation generated in docs/uveddi/");
        
        Ok(())
    }
    
    fn generate_readme_content(&self, template: &ProjectTemplate, project_info: &ProjectInfo) -> String {
        format!(r#"# Uveddi Analysis Configuration

This project has been set up with Uveddi for automated code analysis.

## Project Information
- **Type**: {:?}
- **Languages**: {}
- **Estimated Size**: {:?}
- **Has Tests**: {}
- **Has Documentation**: {}

## Quick Start

```bash
# Run analysis
uveddi analyze .

# Run with AI explanations
uveddi analyze . --enable-ai

# Generate HTML report
uveddi analyze . --output-format html --output reports/analysis.html

# Check system health
uveddi doctor

# View configuration
uveddi config show
```

## Configuration

The project configuration is stored in `uveddi.toml`. You can:

- Edit the file directly
- Use `uveddi config set key value` to update settings
- Regenerate with `uveddi init --force`

## Git Integration

{}

## Troubleshooting

If you encounter issues:

1. Run health check: `uveddi doctor`
2. Check configuration: `uveddi config validate`
3. View detailed help: `uveddi help troubleshooting`

For more information, see the [Uveddi documentation](https://github.com/botzrDev/uveddi).
"#,
            template,
            project_info.languages.join(", "),
            project_info.estimated_size,
            project_info.has_tests,
            project_info.has_docs,
            if self.git_hooks {
                "Git pre-commit hooks are installed and will run analysis automatically."
            } else {
                "Git hooks not installed. Run `uveddi init --git-hooks` to enable automated analysis."
            }
        )
    }
    
    fn generate_analysis_guide(&self, template: &ProjectTemplate) -> String {
        match template {
            ProjectTemplate::Rust => include_str!("../templates/guides/rust_analysis_guide.md"),
            ProjectTemplate::Python => include_str!("../templates/guides/python_analysis_guide.md"),
            ProjectTemplate::JavaScript => include_str!("../templates/guides/javascript_analysis_guide.md"),
            ProjectTemplate::TypeScript => include_str!("../templates/guides/typescript_analysis_guide.md"),
            _ => include_str!("../templates/guides/general_analysis_guide.md"),
        }.to_string()
    }
}

impl ProjectAnalyzer {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
        }
    }
    
    pub async fn analyze(&self) -> Result<ProjectInfo, UveddiError> {
        let languages = self.detect_languages().await?;
        let project_type = self.determine_project_type(&languages).await?;
        let has_tests = self.detect_tests().await?;
        let has_docs = self.detect_docs().await?;
        let estimated_size = self.estimate_size().await?;
        let package_managers = self.detect_package_managers().await?;
        let build_tools = self.detect_build_tools().await?;
        
        Ok(ProjectInfo {
            languages,
            project_type,
            has_tests,
            has_docs,
            estimated_size,
            package_managers,
            build_tools,
        })
    }
    
    async fn detect_languages(&self) -> Result<Vec<String>, UveddiError> {
        let mut languages = Vec::new();
        
        // Check for common language files
        let language_patterns = [
            ("rust", &["*.rs", "Cargo.toml"]),
            ("python", &["*.py", "requirements.txt", "pyproject.toml", "setup.py"]),
            ("javascript", &["*.js", "package.json"]),
            ("typescript", &["*.ts", "*.tsx", "tsconfig.json"]),
            ("html", &["*.html"]),
            ("css", &["*.css", "*.scss", "*.sass"]),
            ("go", &["*.go", "go.mod"]),
            ("java", &["*.java", "pom.xml", "build.gradle"]),
            ("c", &["*.c", "*.h"]),
            ("cpp", &["*.cpp", "*.cc", "*.cxx", "*.hpp"]),
        ];
        
        for (lang, patterns) in language_patterns {
            if self.has_files_matching(patterns).await? {
                languages.push(lang.to_string());
            }
        }
        
        if languages.is_empty() {
            languages.push("unknown".to_string());
        }
        
        Ok(languages)
    }
    
    async fn has_files_matching(&self, patterns: &[&str]) -> Result<bool, UveddiError> {
        for pattern in patterns {
            let glob_pattern = self.path.join(pattern);
            if let Ok(entries) = glob::glob(&glob_pattern.to_string_lossy()) {
                if entries.count() > 0 {
                    return Ok(true);
                }
            }
            
            // Also check recursively for code files
            if pattern.starts_with("*.") {
                let recursive_pattern = self.path.join("**").join(pattern);
                if let Ok(entries) = glob::glob(&recursive_pattern.to_string_lossy()) {
                    if entries.count() > 0 {
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }
    
    async fn determine_project_type(&self, languages: &[String]) -> Result<ProjectTemplate, UveddiError> {
        // Multi-language projects
        if languages.len() > 2 {
            return Ok(ProjectTemplate::Monorepo);
        }
        
        // Web projects
        if languages.contains(&"html".to_string()) || 
           languages.contains(&"css".to_string()) ||
           (languages.contains(&"javascript".to_string()) && languages.contains(&"typescript".to_string())) {
            return Ok(ProjectTemplate::Web);
        }
        
        // Single language projects
        if languages.contains(&"rust".to_string()) {
            // Check if it's a library
            if self.path.join("src").join("lib.rs").exists() {
                Ok(ProjectTemplate::Library)
            } else {
                Ok(ProjectTemplate::Rust)
            }
        } else if languages.contains(&"python".to_string()) {
            Ok(ProjectTemplate::Python)
        } else if languages.contains(&"typescript".to_string()) {
            Ok(ProjectTemplate::TypeScript)
        } else if languages.contains(&"javascript".to_string()) {
            Ok(ProjectTemplate::JavaScript)
        } else {
            Ok(ProjectTemplate::Custom)
        }
    }
    
    async fn detect_tests(&self) -> Result<bool, UveddiError> {
        let test_patterns = [
            "test/**",
            "tests/**",
            "**/*test*",
            "**/*spec*",
        ];
        
        for pattern in test_patterns {
            let glob_pattern = self.path.join(pattern);
            if let Ok(mut entries) = glob::glob(&glob_pattern.to_string_lossy()) {
                if entries.next().is_some() {
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }
    
    async fn detect_docs(&self) -> Result<bool, UveddiError> {
        let doc_patterns = [
            "README*",
            "docs/**",
            "doc/**",
            "*.md",
        ];
        
        for pattern in doc_patterns {
            let glob_pattern = self.path.join(pattern);
            if let Ok(mut entries) = glob::glob(&glob_pattern.to_string_lossy()) {
                if entries.next().is_some() {
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }
    
    async fn estimate_size(&self) -> Result<ProjectSize, UveddiError> {
        let mut line_count = 0;
        let mut file_count = 0;
        
        let code_patterns = ["**/*.rs", "**/*.py", "**/*.js", "**/*.ts", "**/*.go", "**/*.java"];
        
        for pattern in code_patterns {
            let glob_pattern = self.path.join(pattern);
            if let Ok(entries) = glob::glob(&glob_pattern.to_string_lossy()) {
                for entry in entries.flatten() {
                    if let Ok(content) = fs::read_to_string(&entry) {
                        line_count += content.lines().count();
                        file_count += 1;
                    }
                }
            }
        }
        
        match line_count {
            0..=1000 => Ok(ProjectSize::Small),
            1001..=10000 => Ok(ProjectSize::Medium),
            10001..=100000 => Ok(ProjectSize::Large),
            _ => Ok(ProjectSize::Huge),
        }
    }
    
    async fn detect_package_managers(&self) -> Result<Vec<String>, UveddiError> {
        let mut managers = Vec::new();
        
        let manager_files = [
            ("cargo", "Cargo.toml"),
            ("npm", "package.json"),
            ("yarn", "yarn.lock"),
            ("pip", "requirements.txt"),
            ("poetry", "pyproject.toml"),
            ("pipenv", "Pipfile"),
            ("go", "go.mod"),
            ("maven", "pom.xml"),
            ("gradle", "build.gradle"),
        ];
        
        for (manager, file) in manager_files {
            if self.path.join(file).exists() {
                managers.push(manager.to_string());
            }
        }
        
        Ok(managers)
    }
    
    async fn detect_build_tools(&self) -> Result<Vec<String>, UveddiError> {
        let mut tools = Vec::new();
        
        let build_files = [
            ("make", "Makefile"),
            ("cmake", "CMakeLists.txt"),
            ("webpack", "webpack.config.js"),
            ("rollup", "rollup.config.js"),
            ("vite", "vite.config.js"),
            ("docker", "Dockerfile"),
        ];
        
        for (tool, file) in build_files {
            if self.path.join(file).exists() {
                tools.push(tool.to_string());
            }
        }
        
        Ok(tools)
    }
}