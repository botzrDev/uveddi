use clap::{Args, Subcommand};
use std::path::PathBuf;
use crate::config::Config;
use std::fs;
use std::io::{self, Write};

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
        #[arg(long, default_value = "codeatlas.toml")]
        file: PathBuf,
    },
    /// Validate the configuration file
    Validate {
        /// Optional path to config file
        #[arg(long)]
        file: Option<PathBuf>,
    },
}

#[derive(Args)]
pub struct ConfigCommand {
    #[command(subcommand)]
    pub command: ConfigSubcommand,
}

impl ConfigCommand {
    pub fn execute(&self) -> Result<(), String> {
        match &self.command {
            ConfigSubcommand::Show { file } => {
                if let Some(path) = file {
                    match Config::from_file(path.to_str().unwrap()) {
                        Ok(cfg) => println!("{:?}", cfg),
                        Err(e) => return Err(format!("Failed to load config: {}", e)),
                    }
                } else {
                    match Config::from_env() {
                        Ok(cfg) => println!("{:?}", cfg),
                        Err(e) => return Err(format!("Failed to load config from env: {}", e)),
                    }
                }
            }
            ConfigSubcommand::Set { key, value, file } => {
                let mut config = match Config::from_file(file.to_str().unwrap()) {
                    Ok(cfg) => cfg,
                    Err(_) => Config {
                        openai_api_key: None,
                        anthropic_api_key: None,
                        gemini_api_key: None,
                        ollama_model: None,
                    },
                };
                match key.as_str() {
                    "openai_api_key" => config.openai_api_key = Some(value.clone()),
                    "anthropic_api_key" => config.anthropic_api_key = Some(value.clone()),
                    "gemini_api_key" => config.gemini_api_key = Some(value.clone()),
                    "ollama_model" => config.ollama_model = Some(value.clone()),
                    _ => return Err("Unknown config key".to_string()),
                }
                let toml = toml::to_string_pretty(&config).map_err(|e| e.to_string())?;
                let mut file_handle = fs::File::create(file).map_err(|e| e.to_string())?;
                file_handle.write_all(toml.as_bytes()).map_err(|e| e.to_string())?;
                println!("Config updated in {}", file.display());
            }
            ConfigSubcommand::Validate { file } => {
                let path = file.as_ref().map(|p| p.to_str().unwrap()).unwrap_or("codeatlas.toml");
                match Config::from_file(path) {
                    Ok(_) => println!("Config is valid."),
                    Err(e) => return Err(format!("Config validation failed: {}", e)),
                }
            }
        }
        Ok(())
    }
}
