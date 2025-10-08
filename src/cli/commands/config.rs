// NOTE: UV-112, UV-115 - Layer boundary compliance confirmed July 2025. This module only interacts with the Application layer per architecture.

use crate::config::Config;
use crate::core::logging::{error, info};
use clap::{Args, Subcommand};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum ConfigSubcommand {
    /// Show the current configuration (from file or environment)
    Show {
        /// Optional path to config file
        #[arg(long)]
        file: Option<PathBuf>,
    },
    /// Set a configuration value in the config file
    Set {
        /// Config key (e.g. openai_api_key)
        key: String,
        /// Config value
        value: String,
        /// Path to config file
        #[arg(long, default_value = "uveddi.toml")]
        file: PathBuf,
    },
    /// Validate the configuration file with smart suggestions
    Validate {
        /// Optional path to config file
        #[arg(long)]
        file: Option<PathBuf>,
        /// Show detailed suggestions for optimization
        #[arg(long)]
        suggestions: bool,
        /// Output format for validation results
        #[arg(long, default_value = "human")]
        format: String,
    },
}

#[derive(Args)]
pub struct ConfigCommand {
    #[command(subcommand)]
    pub command: ConfigSubcommand,
}

impl ConfigCommand {
    pub async fn execute(&self) -> crate::error::Result<()> {
        match &self.command {
            ConfigSubcommand::Show { file } => {
                if let Some(path) = file {
                    let path_str = path.to_str().ok_or_else(|| {
                        error!("Invalid UTF-8 in config file path: {:?}", path);
                        crate::error::UveddiError::config_error(
                            "Invalid UTF-8 in file path",
                            "config file path",
                        )
                    })?;
                    match Config::from_file(path_str) {
                        Ok(cfg) => {
                            info!("Successfully loaded config from {}", path_str);
                            println!("{cfg:?}");
                        }
                        Err(e) => {
                            error!("Failed to load config from {}: {}", path_str, e);
                            return Err(crate::error::UveddiError::config_error(
                                &format!("Failed to load config: {e}"),
                                "config file",
                            ));
                        }
                    }
                } else {
                    match Config::from_env() {
                        Ok(cfg) => {
                            info!("Successfully loaded config from environment variables");
                            println!("{cfg:?}");
                        }
                        Err(e) => {
                            error!("Failed to load config from environment variables: {}", e);
                            return Err(crate::error::UveddiError::config_error(
                                &format!("Failed to load config from env: {e}"),
                                "environment variables",
                            ));
                        }
                    }
                }
            }
            ConfigSubcommand::Set { key, value, file } => {
                let file_str = file.to_str().ok_or_else(|| {
                    error!("Invalid UTF-8 in config file path: {:?}", file);
                    crate::error::UveddiError::config_error(
                        "Invalid UTF-8 in file path",
                        "config file path",
                    )
                })?;
                let mut config = match Config::from_file(file_str) {
                    Ok(cfg) => cfg,
                    Err(_) => Config {
                        ollama_model: None,
                        dead_code: None,
                        large_classes: None,
                    },
                };
                match key.as_str() {
                    "ollama_model" => config.ollama_model = Some(value.clone()),
                    _ => {
                        return Err(crate::error::UveddiError::config_error(
                            "Unknown config key",
                            "config key validation",
                        ))
                    }
                }
                let toml = toml::to_string_pretty(&config).map_err(|e| {
                    crate::error::UveddiError::config_error(&e.to_string(), "config serialization")
                })?;
                let mut file_handle = fs::File::create(file).map_err(|e| {
                    crate::error::UveddiError::io_error(
                        "creating config file",
                        &file.display().to_string(),
                        e,
                    )
                })?;
                file_handle.write_all(toml.as_bytes()).map_err(|e| {
                    crate::error::UveddiError::io_error(
                        "writing config file",
                        &file.display().to_string(),
                        e,
                    )
                })?;
                info!("Config successfully updated in {}", file.display());
                println!("Config updated in {}", file.display());
            }
            ConfigSubcommand::Validate {
                file,
                suggestions,
                format,
            } => {
                let path_buf = if let Some(file_path) = file.as_ref() {
                    file_path.clone()
                } else {
                    std::path::PathBuf::from("uveddi.toml")
                };

                // Use smart validation system
                let validation_result = crate::config::validation::validate_config_file(&path_buf)
                    .await
                    .map_err(|e| {
                        crate::error::UveddiError::config_error(
                            &format!("Validation failed: {}", e),
                            "config validation",
                        )
                    })?;

                match format.as_str() {
                    "json" => {
                        let json =
                            serde_json::to_string_pretty(&validation_result).map_err(|e| {
                                crate::error::UveddiError::config_error(
                                    &format!("JSON serialization failed: {}", e),
                                    "output formatting",
                                )
                            })?;
                        println!("{}", json);
                    }
                    "human" | _ => {
                        self.display_validation_results(&validation_result, *suggestions);
                    }
                }

                if !validation_result.is_valid {
                    return Err(crate::error::UveddiError::config_error(
                        "Configuration validation failed",
                        "config validation",
                    ));
                }
            }
        }
        Ok(())
    }

    fn display_validation_results(
        &self,
        result: &crate::config::validation::ValidationResult,
        show_suggestions: bool,
    ) {
        use crate::config::validation::{ErrorSeverity, ImpactLevel, Priority};

        // Summary
        println!("🔍 Configuration Validation Summary");
        println!("==================================");
        println!(
            "Status: {}",
            if result.is_valid {
                "✅ Valid"
            } else {
                "❌ Invalid"
            }
        );
        println!("Performance Score: {:.1}/100", result.performance_score);
        println!("Completeness Score: {:.1}/100", result.completeness_score);
        println!();

        // Errors
        if !result.errors.is_empty() {
            println!("❌ Errors ({}):", result.errors.len());
            for error in &result.errors {
                let icon = match error.severity {
                    ErrorSeverity::Critical => "🚨",
                    ErrorSeverity::High => "🔴",
                    ErrorSeverity::Medium => "🟡",
                    ErrorSeverity::Low => "🟢",
                };
                println!("  {} [{}] {}", icon, error.field, error.message);
                if let Some(ref fix) = error.fix_suggestion {
                    println!("    💡 Fix: {}", fix);
                }
            }
            println!();
        }

        // Warnings
        if !result.warnings.is_empty() {
            println!("⚠️  Warnings ({}):", result.warnings.len());
            for warning in &result.warnings {
                let icon = match warning.impact {
                    ImpactLevel::High => "🔶",
                    ImpactLevel::Medium => "🔸",
                    ImpactLevel::Low => "🔹",
                };
                println!("  {} [{}] {}", icon, warning.field, warning.message);
                println!("    📋 Recommendation: {}", warning.recommendation);
            }
            println!();
        }

        // Suggestions (if requested or if high priority)
        let relevant_suggestions: Vec<_> = if show_suggestions {
            result.suggestions.iter().collect()
        } else {
            result
                .suggestions
                .iter()
                .filter(|s| matches!(s.priority, Priority::High))
                .collect()
        };

        if !relevant_suggestions.is_empty() {
            println!("💡 Suggestions ({}):", relevant_suggestions.len());
            for suggestion in &relevant_suggestions {
                let priority_icon = match suggestion.priority {
                    Priority::High => "🔥",
                    Priority::Medium => "📈",
                    Priority::Low => "💭",
                };
                let category_icon = match suggestion.category {
                    crate::config::validation::SuggestionCategory::Performance => "⚡",
                    crate::config::validation::SuggestionCategory::Accuracy => "🎯",
                    crate::config::validation::SuggestionCategory::Security => "🔒",
                    crate::config::validation::SuggestionCategory::Maintenance => "🔧",
                    crate::config::validation::SuggestionCategory::Compatibility => "🔗",
                };

                println!("  {} {} {}", priority_icon, category_icon, suggestion.title);
                println!("    {}", suggestion.description);

                if let Some(ref before) = suggestion.before {
                    println!("    Before: {}", before);
                }
                println!("    After:  {}", suggestion.after);
                println!("    Benefit: {}", suggestion.benefit);
                println!();
            }

            if !show_suggestions && result.suggestions.len() > relevant_suggestions.len() {
                println!(
                    "💭 {} more suggestions available. Use --suggestions to see all.",
                    result.suggestions.len() - relevant_suggestions.len()
                );
                println!();
            }
        }

        // Final recommendations
        if result.is_valid {
            if result.performance_score < 70.0 {
                println!(
                    "🎯 Consider optimizing for better performance (score: {:.1})",
                    result.performance_score
                );
            }
            if result.completeness_score < 80.0 {
                println!(
                    "📋 Consider adding more configuration options (completeness: {:.1}%)",
                    result.completeness_score
                );
            }
            if result.performance_score >= 80.0 && result.completeness_score >= 80.0 {
                println!("🎉 Excellent configuration! Your setup looks great.");
            }
        } else {
            println!("🔧 Fix the errors above before running analysis.");
        }
    }
}
