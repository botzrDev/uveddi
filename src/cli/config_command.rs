// NOTE: UV-112, UV-115 - Layer boundary compliance confirmed July 2025. This module only interacts with the Application layer per architecture.

use crate::config::Config;
use clap::{Args, Subcommand};
use log::{error, info, warn};
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
                    let path_str = path.to_str()
                        .ok_or_else(|| {
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
                        },
                        Err(e) => {
                            error!("Failed to load config from {}: {}", path_str, e);
                            return Err(crate::error::UveddiError::config_error(
                                &format!("Failed to load config: {e}"),
                                "config file",
                            ))
                        }
                    }
                } else {
                    match Config::from_env() {
                        Ok(cfg) => {
                            info!("Successfully loaded config from environment variables");
                            println!("{cfg:?}");
                        },
                        Err(e) => {
                            error!("Failed to load config from environment variables: {}", e);
                            return Err(crate::error::UveddiError::config_error(
                                &format!("Failed to load config from env: {e}"),
                                "environment variables",
                            ))
                        }
                    }
                }
            }
            ConfigSubcommand::Set { key, value, file } => {
                let file_str = file.to_str()
                    .ok_or_else(|| {
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
            ConfigSubcommand::Validate { file } => {
                let path = if let Some(file_path) = file.as_ref() {
                    file_path.to_str()
                        .ok_or_else(|| crate::error::UveddiError::config_error(
                            "Invalid UTF-8 in file path",
                            "config file path",
                        ))?
                } else {
                    "uveddi.toml"
                };
                match Config::from_file(path) {
                    Ok(_) => println!("Config is valid."),
                    Err(e) => {
                        return Err(crate::error::UveddiError::config_error(
                            &format!("Config validation failed: {e}"),
                            "config validation",
                        ))
                    }
                }
            }
        }
        Ok(())
    }
}
