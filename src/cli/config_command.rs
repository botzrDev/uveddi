// NOTE: UV-112, UV-115 - Layer boundary compliance confirmed July 2025. This module only interacts with the Application layer per architecture.

use crate::config::Config;
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
    pub fn execute(&self) -> crate::error::Result<()> {
        match &self.command {
            ConfigSubcommand::Show { file } => {
                if let Some(path) = file {
                    match Config::from_file(path.to_str().unwrap()) {
                        Ok(cfg) => println!("{cfg:?}"),
                        Err(e) => return Err(crate::error::UveddiError::ConfigError(format!("Failed to load config: {e}"))),
                    }
                } else {
                    match Config::from_env() {
                        Ok(cfg) => println!("{cfg:?}"),
                        Err(e) => return Err(crate::error::UveddiError::ConfigError(format!("Failed to load config from env: {e}"))),
                    }
                }
            }
            ConfigSubcommand::Set { key, value, file } => {
                let mut config = match Config::from_file(file.to_str().unwrap()) {
                    Ok(cfg) => cfg,
                    Err(_) => Config {
                        ollama_model: None,
                        dead_code: None,
                        large_classes: None,
                    },
                };
                match key.as_str() {
                    "ollama_model" => config.ollama_model = Some(value.clone()),
                    _ => return Err(crate::error::UveddiError::ConfigError("Unknown config key".to_string())),
                }
                let toml = toml::to_string_pretty(&config).map_err(|e| crate::error::UveddiError::ConfigError(e.to_string()))?;
                let mut file_handle = fs::File::create(file).map_err(|e| crate::error::UveddiError::IoError(e))?;
                file_handle
                    .write_all(toml.as_bytes())
                    .map_err(|e| crate::error::UveddiError::IoError(e))?;
                println!("Config updated in {}", file.display());
            }
            ConfigSubcommand::Validate { file } => {
                let path = file
                    .as_ref()
                    .map(|p| p.to_str().unwrap())
                    .unwrap_or("uveddi.toml");
                match Config::from_file(path) {
                    Ok(_) => println!("Config is valid."),
                    Err(e) => return Err(crate::error::UveddiError::ConfigError(format!("Config validation failed: {e}"))),
                }
            }
        }
        Ok(())
    }
}
