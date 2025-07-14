//! CLI commands for WASM plugin management
// NOTE: UV-112, UV-115 - Layer boundary compliance confirmed July 2025. This module only interacts with the Application layer per architecture.

#[cfg(feature = "wasm-plugins")]
use crate::plugins::{WasmPluginEngine, PluginManifest, PluginId};
use clap::{Args, Subcommand};
use std::path::PathBuf;

/// Plugin management commands
#[derive(Args, Debug)]
pub struct PluginCommand {
    #[command(subcommand)]
    pub action: PluginAction,
}

#[derive(Subcommand, Debug)]
pub enum PluginAction {
    /// List installed plugins
    List,
    /// Install a plugin from a file
    Install {
        /// Path to the plugin WASM binary
        binary: PathBuf,
        /// Path to the plugin manifest (plugin.toml)
        manifest: PathBuf,
    },
    /// Uninstall a plugin
    Uninstall {
        /// Plugin name or ID
        plugin: String,
    },
    /// Show plugin information
    Info {
        /// Plugin name or ID
        plugin: String,
    },
    /// Show plugin statistics
    Stats,
    /// Monitor plugin resource usage
    Monitor,
    /// Verify a plugin without installing
    Verify {
        /// Path to the plugin WASM binary
        binary: PathBuf,
        /// Path to the plugin manifest (plugin.toml)
        manifest: PathBuf,
    },
}

impl PluginCommand {
    /// Execute the plugin command
    pub async fn execute(self) -> Result<(), crate::error::UveddiError> {
        match self.action {
            PluginAction::List => self.list_plugins().await,
            PluginAction::Install {
                ref binary,
                ref manifest,
            } => self.install_plugin(binary.clone(), manifest.clone()).await,
            PluginAction::Uninstall { ref plugin } => self.uninstall_plugin(plugin.clone()).await,
            PluginAction::Info { ref plugin } => self.show_plugin_info(plugin.clone()).await,
            PluginAction::Stats => self.show_plugin_stats().await,
            PluginAction::Monitor => self.monitor_plugins().await,
            PluginAction::Verify {
                ref binary,
                ref manifest,
            } => self.verify_plugin(binary.clone(), manifest.clone()).await,
        }
    }

    async fn list_plugins(&self) -> Result<(), crate::error::UveddiError> {
        #[cfg(feature = "wasm-plugins")]
        {
            let engine = WasmPluginEngine::new().await?;
            let stats = engine.get_registry_stats();

            println!("Plugin Registry Statistics:");
            println!("  Total plugins: {}", stats.total_plugins);
            println!("  Ready plugins: {}", stats.ready_plugins);
            println!("  Error plugins: {}", stats.error_plugins);
            println!(
                "  Supported languages: {}",
                stats.supported_languages.join(", ")
            );
            println!(
                "  Supported anti-patterns: {}",
                stats.supported_anti_patterns.join(", ")
            );

            if stats.total_plugins > 0 {
                println!("\nLoaded plugins:");
                for plugin_id in engine.list_loaded_plugins().await {
                    println!("  - {}", plugin_id);
                }
            }
        }

        #[cfg(not(feature = "wasm-plugins"))]
        {
            println!(
                "WASM plugins not enabled. Compile with --features wasm-plugins to use plugins."
            );
        }

        Ok(())
    }

    async fn install_plugin(
        &self,
        binary_path: PathBuf,
        manifest_path: PathBuf,
    ) -> Result<(), crate::error::UveddiError> {
        #[cfg(feature = "wasm-plugins")]
        {
            println!(
                "Installing plugin from {} and {}...",
                binary_path.display(),
                manifest_path.display()
            );

            // Read binary file
            let binary = tokio::fs::read(&binary_path)
                .await
                .map_err(|e| crate::error::UveddiError::IoError(e))?;

            // Read and parse manifest
            let manifest_content = tokio::fs::read_to_string(&manifest_path)
                .await
                .map_err(|e| crate::error::UveddiError::IoError(e))?;
            let manifest: PluginManifest = toml::from_str(&manifest_content)
                .map_err(|e| crate::error::UveddiError::PluginError(crate::plugins::errors::PluginError::Configuration(e.to_string())))?;

            // Install plugin
            let mut engine = WasmPluginEngine::new().await?;
            let plugin_id = engine.install_plugin(manifest.clone(), binary).await?;

            println!(
                "Successfully installed plugin '{}' with ID: {}",
                manifest.name, plugin_id
            );
            println!("  Version: {}", manifest.version);
            println!("  Author: {}", manifest.author);
            println!("  Description: {}", manifest.description);
            println!(
                "  Supported languages: {}",
                manifest.supported_languages.join(", ")
            );
            println!(
                "  Anti-pattern types: {}",
                manifest.anti_pattern_types.join(", ")
            );
        }

        #[cfg(not(feature = "wasm-plugins"))]
        {
            println!(
                "WASM plugins not enabled. Compile with --features wasm-plugins to use plugins."
            );
        }

        Ok(())
    }

    async fn uninstall_plugin(&self, plugin_name: String) -> Result<(), crate::error::UveddiError> {
        #[cfg(feature = "wasm-plugins")]
        {
            println!("Uninstalling plugin '{}'...", plugin_name);

            let mut engine = WasmPluginEngine::new().await?;

            // Find plugin by name
            let stats = engine.get_registry_stats();
            // For simplicity, we'll create a plugin ID from the name
            // In a real implementation, you'd search the registry properly
            let plugin_id = PluginId::from_name(&plugin_name);

            engine.uninstall_plugin(&plugin_id).await?;
            println!("Successfully uninstalled plugin '{}'", plugin_name);
        }

        #[cfg(not(feature = "wasm-plugins"))]
        {
            println!(
                "WASM plugins not enabled. Compile with --features wasm-plugins to use plugins."
            );
        }

        Ok(())
    }

    async fn show_plugin_info(&self, plugin_name: String) -> Result<(), crate::error::UveddiError> {
        #[cfg(feature = "wasm-plugins")]
        {
            let engine = WasmPluginEngine::new().await?;

            // Find plugin by name and show detailed information
            println!("Plugin Information for '{}':", plugin_name);
            println!("  Status: Not implemented - would show detailed plugin info");

            // In a real implementation, you would:
            // 1. Search the registry for the plugin
            // 2. Load its manifest
            // 3. Show detailed information including permissions, capabilities, etc.
        }

        #[cfg(not(feature = "wasm-plugins"))]
        {
            println!(
                "WASM plugins not enabled. Compile with --features wasm-plugins to use plugins."
            );
        }

        Ok(())
    }

    async fn show_plugin_stats(&self) -> Result<(), crate::error::UveddiError> {
        #[cfg(feature = "wasm-plugins")]
        {
            let engine = WasmPluginEngine::new().await?;

            // Get all loaded plugins and their stats
            let loaded_plugins = engine.list_loaded_plugins().await;
            if !loaded_plugins.is_empty() {
                println!("Plugin Statistics:");
                for plugin_id in loaded_plugins {
                    if let Some(stats) = engine.get_plugin_stats(&plugin_id).await {
                    println!("  Plugin: {}", plugin_id);
                    println!("    Invocations: {}", stats.invocations);
                    println!(
                        "    Total execution time: {}ms",
                        stats.total_execution_time_ms
                    );
                    println!(
                        "    Average execution time: {:.2}ms",
                        stats.avg_execution_time_ms
                    );
                    println!("    Total fuel consumed: {}", stats.total_fuel_consumed);
                    println!("    Peak memory usage: {} bytes", stats.peak_memory_usage);
                    println!("    Error count: {}", stats.error_count);
                    if let Some(ref error) = stats.last_error {
                        println!("    Last error: {}", error);
                    }
                    println!();
                    }
                }
            } else {
                println!("No plugin statistics available (plugin engine not initialized)");
            }
        }

        #[cfg(not(feature = "wasm-plugins"))]
        {
            println!(
                "WASM plugins not enabled. Compile with --features wasm-plugins to use plugins."
            );
        }

        Ok(())
    }

    async fn monitor_plugins(&self) -> Result<(), crate::error::UveddiError> {
        #[cfg(feature = "wasm-plugins")]
        {
            let mut engine = WasmPluginEngine::new().await?;

            println!("Monitoring plugin resource usage...");
            let resource_report = engine.monitor_resources().await?;

            println!("Resource Report:");
            println!(
                "  Total memory usage: {} bytes",
                resource_report.total_memory_usage
            );
            println!(
                "  Total fuel consumed: {}",
                resource_report.total_fuel_consumed
            );

            for (plugin_id, plugin_report) in resource_report.plugin_reports {
                println!("  Plugin: {}", plugin_id);
                println!("    Memory usage: {} bytes", plugin_report.memory_usage);
                println!("    Fuel consumed: {}", plugin_report.fuel_consumed);
                println!("    Execution time: {:?}", plugin_report.execution_time);
                println!("    Handle count: {}", plugin_report.handle_count);
                println!("    Handle memory: {} bytes", plugin_report.handle_memory);
            }
        }

        #[cfg(not(feature = "wasm-plugins"))]
        {
            println!(
                "WASM plugins not enabled. Compile with --features wasm-plugins to use plugins."
            );
        }

        Ok(())
    }

    async fn verify_plugin(
        &self,
        binary_path: PathBuf,
        manifest_path: PathBuf,
    ) -> Result<(), crate::error::UveddiError> {
        #[cfg(feature = "wasm-plugins")]
        {
            use crate::plugins::{PluginVerifier, SecurityPolicy};

            println!(
                "Verifying plugin from {} and {}...",
                binary_path.display(),
                manifest_path.display()
            );

            // Read binary file
            let binary = tokio::fs::read(&binary_path)
                .await
                .map_err(|e| crate::error::UveddiError::IoError(e))?;

            // Read and parse manifest
            let manifest_content = tokio::fs::read_to_string(&manifest_path)
                .await
                .map_err(|e| crate::error::UveddiError::IoError(e))?;
            let manifest: PluginManifest = toml::from_str(&manifest_content)
                .map_err(|e| crate::error::UveddiError::PluginError(crate::plugins::errors::PluginError::Configuration(e.to_string())))?;

            // Verify plugin
            let verifier = PluginVerifier::new();
            let policy = SecurityPolicy::restrictive();

            match verifier.verify_plugin(&binary, &manifest, &policy).await {
                Ok(report) => {
                    println!("Verification completed!");
                    println!(
                        "  Plugin: {} v{}",
                        report.plugin_name, report.plugin_version
                    );
                    println!("  Status: {:?}", report.overall_status);
                    println!("  Binary hash: {}", report.plugin_hash);
                    println!(
                        "  Verification timestamp: {}",
                        report.verification_timestamp
                    );

                    println!("\nStatic Analysis:");
                    println!(
                        "  Vulnerabilities found: {}",
                        report.static_analysis.vulnerabilities.len()
                    );
                    for vuln in &report.static_analysis.vulnerabilities {
                        println!(
                            "    - {}: {} ({:?})",
                            vuln.id, vuln.description, vuln.severity
                        );
                    }

                    println!("\nManifest Validation:");
                    println!("  Valid: {}", report.manifest_validation.is_valid);
                    if !report.manifest_validation.errors.is_empty() {
                        println!("  Errors:");
                        for error in &report.manifest_validation.errors {
                            println!("    - {}", error);
                        }
                    }
                    if !report.manifest_validation.warnings.is_empty() {
                        println!("  Warnings:");
                        for warning in &report.manifest_validation.warnings {
                            println!("    - {}", warning);
                        }
                    }

                    println!("\nCapability Audit:");
                    println!(
                        "  Requested permissions: {}",
                        report.capability_audit.requested_permissions.len()
                    );
                    println!(
                        "  Granted permissions: {}",
                        report.capability_audit.granted_permissions.len()
                    );
                    println!(
                        "  Denied permissions: {}",
                        report.capability_audit.denied_permissions.len()
                    );
                }
                Err(e) => {
                    println!("Verification failed: {}", e);
                    return Err(crate::error::UveddiError::PluginError(crate::plugins::errors::PluginError::Configuration(e.to_string())));
                }
            }
        }

        #[cfg(not(feature = "wasm-plugins"))]
        {
            println!(
                "WASM plugins not enabled. Compile with --features wasm-plugins to use plugins."
            );
        }

        Ok(())
    }
}
