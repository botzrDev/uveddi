//! Git hooks management command
//!
//! Provides CLI interface for installing, managing, and configuring
//! Git hooks for automated Uveddi analysis.

use crate::error::UveddiError;
use crate::hooks::{HookConfig, HookType};
use crate::hooks::manager::HookManager;
use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum HooksSubcommand {
    /// Install Git hooks for automated analysis
    Install {
        /// Repository path (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
        /// Force overwrite existing hooks
        #[arg(long)]
        force: bool,
        /// Enable pre-commit hook
        #[arg(long)]
        pre_commit: Option<bool>,
        /// Enable pre-push hook
        #[arg(long)]
        pre_push: Option<bool>,
        /// Enable commit-msg validation
        #[arg(long)]
        commit_msg: Option<bool>,
        /// Analysis timeout in seconds
        #[arg(long)]
        timeout: Option<u64>,
        /// Analyze only changed files
        #[arg(long)]
        changed_files_only: Option<bool>,
        /// Minimum confidence threshold
        #[arg(long)]
        min_confidence: Option<f64>,
        /// Maximum issues before blocking
        #[arg(long)]
        max_issues: Option<u32>,
    },
    /// Uninstall Uveddi Git hooks
    Uninstall {
        /// Repository path (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    /// List installed Git hooks
    List {
        /// Repository path (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
        /// Show detailed information
        #[arg(long)]
        detailed: bool,
    },
    /// Test Git hooks without committing
    Test {
        /// Repository path (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
        /// Hook type to test
        #[arg(value_enum)]
        hook_type: TestHookType,
        /// Test with specific files
        #[arg(long)]
        files: Option<Vec<PathBuf>>,
    },
    /// Show hook configuration
    Config {
        /// Repository path (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
}

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum TestHookType {
    PreCommit,
    PrePush,
    CommitMsg,
}

impl From<TestHookType> for HookType {
    fn from(test_type: TestHookType) -> Self {
        match test_type {
            TestHookType::PreCommit => HookType::PreCommit,
            TestHookType::PrePush => HookType::PrePush,
            TestHookType::CommitMsg => HookType::CommitMsg,
        }
    }
}

#[derive(Args)]
pub struct HooksCommand {
    #[command(subcommand)]
    pub command: HooksSubcommand,
}

impl HooksCommand {
    pub async fn execute(&self) -> Result<(), UveddiError> {
        match &self.command {
            HooksSubcommand::Install {
                path,
                force,
                pre_commit,
                pre_push,
                commit_msg,
                timeout,
                changed_files_only,
                min_confidence,
                max_issues,
            } => {
                self.install_hooks(
                    path.clone().unwrap_or_else(|| PathBuf::from(".")),
                    *force,
                    *pre_commit,
                    *pre_push,
                    *commit_msg,
                    *timeout,
                    *changed_files_only,
                    *min_confidence,
                    *max_issues,
                ).await
            }
            HooksSubcommand::Uninstall { path } => {
                self.uninstall_hooks(path.clone().unwrap_or_else(|| PathBuf::from("."))).await
            }
            HooksSubcommand::List { path, detailed } => {
                self.list_hooks(path.clone().unwrap_or_else(|| PathBuf::from(".")), *detailed).await
            }
            HooksSubcommand::Test { path, hook_type, files } => {
                self.test_hook(
                    path.clone().unwrap_or_else(|| PathBuf::from(".")),
                    hook_type.clone(),
                    files.clone(),
                ).await
            }
            HooksSubcommand::Config { path } => {
                self.show_config(path.clone().unwrap_or_else(|| PathBuf::from("."))).await
            }
        }
    }

    async fn install_hooks(
        &self,
        repo_path: PathBuf,
        force: bool,
        pre_commit: Option<bool>,
        pre_push: Option<bool>,
        commit_msg: Option<bool>,
        timeout: Option<u64>,
        changed_files_only: Option<bool>,
        min_confidence: Option<f64>,
        max_issues: Option<u32>,
    ) -> Result<(), UveddiError> {
        // Check if it's a Git repository
        if !crate::hooks::is_git_repository(&repo_path) {
            return Err(UveddiError::config_error(&format!(
                "Not a Git repository: {}",
                repo_path.display()
            ), "cli"));
        }

        println!("🔧 Installing Uveddi Git hooks...");
        println!("Repository: {}", repo_path.display());

        // Load existing config or create default
        let mut config = HookManager::load_config(&repo_path).unwrap_or_default();

        // Override config with command-line options
        if let Some(pc) = pre_commit {
            config.pre_commit = pc;
        }
        if let Some(pp) = pre_push {
            config.pre_push = pp;
        }
        if let Some(cm) = commit_msg {
            config.commit_msg = cm;
        }
        if let Some(t) = timeout {
            config.timeout_seconds = t;
        }
        if let Some(cf) = changed_files_only {
            config.changed_files_only = cf;
        }
        if let Some(mc) = min_confidence {
            config.min_confidence = mc;
        }
        if let Some(mi) = max_issues {
            config.max_issues = Some(mi);
        }

        let manager = HookManager::new(repo_path, config)?;
        let result = manager.install_hooks(force).await?;

        // Display results
        if !result.installed_hooks.is_empty() {
            println!("✅ Installed hooks:");
            for hook in &result.installed_hooks {
                println!("   • {:?}", hook);
            }
        }

        if !result.updated_hooks.is_empty() {
            println!("🔄 Updated hooks:");
            for hook in &result.updated_hooks {
                println!("   • {:?}", hook);
            }
        }

        if !result.errors.is_empty() {
            println!("❌ Errors:");
            for error in &result.errors {
                println!("   • {}", error);
            }
        }

        if let Some(config_path) = result.config_path {
            println!("📝 Configuration saved to: {}", config_path.display());
        }

        if result.installed_hooks.is_empty() && result.updated_hooks.is_empty() {
            println!("⚠️  No hooks were installed. Check the configuration.");
        } else {
            println!("\n🎉 Git hooks installation completed!");
            println!("\n💡 Next steps:");
            println!("   • Test hooks with: uveddi hooks test pre-commit");
            println!("   • View configuration: uveddi hooks config");
            println!("   • Make a test commit to see hooks in action");
        }

        Ok(())
    }

    async fn uninstall_hooks(&self, repo_path: PathBuf) -> Result<(), UveddiError> {
        if !crate::hooks::is_git_repository(&repo_path) {
            return Err(UveddiError::config_error(&format!(
                "Not a Git repository: {}",
                repo_path.display()
            ), "cli"));
        }

        println!("🗑️  Uninstalling Uveddi Git hooks...");
        println!("Repository: {}", repo_path.display());

        let config = HookManager::load_config(&repo_path).unwrap_or_default();
        let manager = HookManager::new(repo_path, config)?;
        let removed_hooks = manager.uninstall_hooks().await?;

        if removed_hooks.is_empty() {
            println!("ℹ️  No Uveddi hooks found to uninstall");
        } else {
            println!("✅ Uninstalled hooks:");
            for hook in &removed_hooks {
                println!("   • {:?}", hook);
            }
            println!("\n🎉 Hooks uninstalled successfully!");
        }

        Ok(())
    }

    async fn list_hooks(&self, repo_path: PathBuf, detailed: bool) -> Result<(), UveddiError> {
        if !crate::hooks::is_git_repository(&repo_path) {
            return Err(UveddiError::config_error(&format!(
                "Not a Git repository: {}",
                repo_path.display()
            ), "cli"));
        }

        println!("📋 Git hooks status");
        println!("Repository: {}", repo_path.display());
        println!();

        let config = HookManager::load_config(&repo_path).unwrap_or_default();
        let manager = HookManager::new(repo_path, config.clone())?;
        let hooks = manager.list_hooks().await?;

        for (hook_type, is_uveddi, hook_path) in hooks {
            let status_icon = if hook_path.exists() {
                if is_uveddi {
                    "✅"
                } else {
                    "⚠️ "
                }
            } else {
                "❌"
            };

            let status_text = if hook_path.exists() {
                if is_uveddi {
                    "Uveddi hook installed"
                } else {
                    "Non-Uveddi hook exists"
                }
            } else {
                "Not installed"
            };

            println!("{} {:?}: {}", status_icon, hook_type, status_text);

            if detailed {
                println!("   Path: {}", hook_path.display());
                if hook_path.exists() && is_uveddi {
                    println!("   Size: {} bytes", hook_path.metadata().map(|m| m.len()).unwrap_or(0));
                    if let Ok(metadata) = hook_path.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            println!("   Modified: {:?}", modified);
                        }
                    }
                }
                println!();
            }
        }

        if detailed {
            println!("📊 Configuration:");
            println!("   Pre-commit: {}", if config.pre_commit { "enabled" } else { "disabled" });
            println!("   Pre-push: {}", if config.pre_push { "enabled" } else { "disabled" });
            println!("   Commit-msg: {}", if config.commit_msg { "enabled" } else { "disabled" });
            println!("   Timeout: {} seconds", config.timeout_seconds);
            println!("   Changed files only: {}", config.changed_files_only);
            println!("   Min confidence: {}", config.min_confidence);
            if let Some(max) = config.max_issues {
                println!("   Max issues: {}", max);
            } else {
                println!("   Max issues: unlimited");
            }
        }

        Ok(())
    }

    async fn test_hook(
        &self,
        repo_path: PathBuf,
        hook_type: TestHookType,
        files: Option<Vec<PathBuf>>,
    ) -> Result<(), UveddiError> {
        if !crate::hooks::is_git_repository(&repo_path) {
            return Err(UveddiError::config_error(&format!(
                "Not a Git repository: {}",
                repo_path.display()
            ), "cli"));
        }

        println!("🧪 Testing {:?} hook...", hook_type);
        println!("Repository: {}", repo_path.display());

        let config = HookManager::load_config(&repo_path).unwrap_or_default();
        let hook_type_enum: HookType = hook_type.into();

        // Get test files
        let test_files = if let Some(files) = files {
            files
        } else {
            match hook_type_enum {
                HookType::PreCommit => {
                    println!("🔍 Getting staged files...");
                    crate::hooks::get_staged_files(&repo_path)?
                }
                _ => {
                    println!("🔍 Using current directory for analysis...");
                    vec![]
                }
            }
        };

        if matches!(hook_type_enum, HookType::PreCommit) && test_files.is_empty() {
            println!("⚠️  No staged files found. Stage some files first or specify --files");
            return Ok(());
        }

        // Execute hook analysis
        let result = crate::hooks::execute_analysis_for_hook(
            &repo_path,
            &config,
            &hook_type_enum,
            if test_files.is_empty() { None } else { Some(&test_files) },
        ).await?;

        // Display results
        println!("\n📊 Test Results:");
        println!("   Status: {}", if result.success { "✅ PASS" } else { "❌ FAIL" });
        println!("   Files analyzed: {}", result.files_analyzed);
        println!("   Issues found: {}", result.issues_found);
        println!("   Critical issues: {}", result.critical_issues);
        println!("   Execution time: {:.2}s", result.execution_time);

        if !result.message.is_empty() {
            println!("   Message: {}", result.message);
        }

        if !result.suggestions.is_empty() {
            println!("   Suggestions:");
            for suggestion in &result.suggestions {
                println!("     • {}", suggestion);
            }
        }

        if result.success {
            println!("\n🎉 Hook test passed! This commit/push would be allowed.");
        } else {
            println!("\n🚫 Hook test failed! This commit/push would be blocked.");
            println!("💡 Fix the issues above or use --no-verify to skip hooks");
        }

        Ok(())
    }

    async fn show_config(&self, repo_path: PathBuf) -> Result<(), UveddiError> {
        if !crate::hooks::is_git_repository(&repo_path) {
            return Err(UveddiError::config_error(&format!(
                "Not a Git repository: {}",
                repo_path.display()
            ), "cli"));
        }

        println!("⚙️  Uveddi Hooks Configuration");
        println!("Repository: {}", repo_path.display());
        println!();

        let config_path = repo_path.join(".uveddi").join("hooks.toml");
        if config_path.exists() {
            println!("📝 Configuration file: {}", config_path.display());
            let config = HookManager::load_config(&repo_path)?;
            let toml_content = toml::to_string_pretty(&config)
                .map_err(|e| UveddiError::config_error(&format!("Failed to serialize config: {}", e), "cli"))?;
            println!("\n{}", toml_content);
        } else {
            println!("📝 Configuration file: Not found (using defaults)");
            let config = HookConfig::default();
            let toml_content = toml::to_string_pretty(&config)
                .map_err(|e| UveddiError::config_error(&format!("Failed to serialize config: {}", e), "cli"))?;
            println!("\n{}", toml_content);
            println!("\n💡 Run 'uveddi hooks install' to create configuration file");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_command_creation() {
        let cmd = HooksCommand {
            command: HooksSubcommand::List {
                path: None,
                detailed: false,
            },
        };

        assert!(matches!(cmd.command, HooksSubcommand::List { .. }));
    }

    #[test]
    fn test_test_hook_type_conversion() {
        let test_type = TestHookType::PreCommit;
        let hook_type: HookType = test_type.into();
        assert!(matches!(hook_type, HookType::PreCommit));
    }
}