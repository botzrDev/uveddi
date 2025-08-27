//! Help command implementation
//!
//! Provides enhanced help with context-aware suggestions, examples,
//! and topic-based assistance for better user experience.

use crate::cli::enhanced_help::{generate_quick_help, generate_topic_help};
use crate::error::UveddiError;
use clap::Args;

#[derive(Args, Debug)]
pub struct HelpCommand {
    /// Help topic to display
    ///
    /// Available topics: installation, configuration, troubleshooting, ci-cd, plugins, examples
    pub topic: Option<String>,

    /// Show verbose help with all available options
    #[arg(long)]
    pub verbose: bool,
}

impl HelpCommand {
    pub async fn execute(&self) -> Result<(), UveddiError> {
        if let Some(topic) = &self.topic {
            self.show_topic_help(topic).await
        } else {
            self.show_general_help().await
        }
    }

    async fn show_general_help(&self) -> Result<(), UveddiError> {
        println!("{}", generate_quick_help());
        
        if self.verbose {
            println!("\n🔍 Detailed Command Information:");
            println!("================================");
            
            self.show_command_details().await?;
        }
        
        Ok(())
    }

    async fn show_topic_help(&self, topic: &str) -> Result<(), UveddiError> {
        if let Some(help_text) = generate_topic_help(topic) {
            println!("{}", help_text);
        } else {
            println!("❌ Unknown help topic: '{}'", topic);
            println!("\n💡 Available topics:");
            println!("   • installation    - Setup and installation guide");
            println!("   • configuration   - Configuration file help");
            println!("   • troubleshooting - Common issues and fixes");
            println!("   • ci-cd          - CI/CD integration guide");
            println!("   • plugins        - Plugin system documentation");
            println!("   • examples       - Usage examples and tutorials");
            println!("\n📚 Use 'uveddi help <topic>' for specific help");
            
            return Err(UveddiError::config_error(&format!("Unknown help topic: {}", topic), "cli"));
        }
        
        Ok(())
    }

    async fn show_command_details(&self) -> Result<(), UveddiError> {
        println!(r#"
📝 Analyze Command (uveddi analyze, uveddi a):
  Primary code analysis command with comprehensive options
  
  Basic Usage:
    uveddi analyze ./src                    # Basic analysis
    uveddi a ./src --output-format json     # Short form with JSON
    
  Advanced Options:
    --enable-ai                            # AI-powered insights
    --progress-format terminal             # Rich progress display
    --security                             # Security vulnerability scan
    --memory-optimization                  # For large codebases
    --timeout 600                          # 10-minute timeout

🏥 Doctor Command (uveddi doctor, uveddi dr):
  System health diagnostics and auto-fix capabilities
  
  Usage:
    uveddi doctor                          # Full health check
    uveddi dr --fix                        # Auto-fix issues
    uveddi doctor --parsers                # Check only parsers
    uveddi doctor --ai                     # Check only AI integration
    --output-format json                   # Machine-readable output

⚙️  Config Command (uveddi config, uveddi cfg):
  Configuration management
  
  Usage:
    uveddi config show                     # Display current config
    uveddi cfg set key value               # Update setting
    uveddi config validate                 # Check config validity
    uveddi config reset                    # Restore defaults

🚀 Serve Command:
  Launch web dashboard and services
  
  Usage:
    uveddi serve                           # Start with defaults
    uveddi serve --port 8888 --development # Dev mode

🖥️  TUI Command:
  Terminal user interface for interactive analysis
  
  Usage:
    uveddi tui                             # Launch TUI

🔧 Plugin Command (requires wasm-plugins feature):
  WebAssembly plugin management
  
  Usage:
    uveddi plugin list                     # List installed plugins
    uveddi plugin install plugin.wasm     # Install new plugin
    uveddi plugin info <id>                # Plugin details

📈 CI Command:
  CI/CD integration helpers
  
  Usage:
    uveddi ci setup github-actions        # Generate GitHub Actions
    uveddi ci setup gitlab                # Generate GitLab CI
"#);
        
        Ok(())
    }
}

/// Generate contextual help based on user's current situation
pub fn generate_contextual_help(context: &str) -> String {
    match context {
        "first_run" => r#"
👋 Welcome to Uveddi!

This appears to be your first time running Uveddi. Here's how to get started:

1. 🏥 Check System Health:
   uveddi doctor
   
2. 📊 Analyze Your Code:
   uveddi analyze ./src
   
3. ⚙️  Configure Settings:
   uveddi config show

💡 Quick Tips:
   • Use 'uveddi help <topic>' for detailed guidance
   • Add --progress-details to see file-by-file progress
   • Try --enable-ai for intelligent explanations (requires Ollama)

🚀 Ready to analyze? Run: uveddi analyze ./your-project-path
"#.to_string(),

        "analysis_failed" => r#"
🔧 Analysis Failed - Troubleshooting Steps:

1. 🏥 Run Health Check:
   uveddi doctor
   
2. 🔍 Check File Permissions:
   uveddi doctor --system --fix
   
3. 📝 Try Simpler Analysis:
   uveddi analyze ./src --output-format text --progress-format silent
   
4. 🐛 Get Detailed Error Info:
   uveddi analyze ./src --verbose

💡 Common Solutions:
   • Increase timeout: --timeout 600
   • Disable AI: Remove --enable-ai flag  
   • Check disk space and memory availability
   • Verify all source files are accessible

📚 More help: uveddi help troubleshooting
"#.to_string(),

        "config_error" => r#"
⚙️  Configuration Issue - Quick Fixes:

1. 🔍 Check Current Config:
   uveddi config show
   
2. ✅ Validate Configuration:
   uveddi config validate
   
3. 🔄 Reset to Defaults:
   uveddi config reset
   
4. 📂 Check Config Files:
   ~/.config/uveddi/config.toml
   ./uveddi.toml

💡 Common Config Issues:
   • Invalid file paths in config
   • Malformed TOML syntax
   • Missing required sections
   • Conflicting environment variables

📚 More help: uveddi help configuration
"#.to_string(),

        _ => generate_quick_help()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_contextual_help() {
        let help = generate_contextual_help("first_run");
        assert!(help.contains("Welcome to Uveddi"));
        assert!(help.contains("doctor"));
    }
    
    #[test]
    fn test_help_command_creation() {
        let help_cmd = HelpCommand {
            topic: Some("troubleshooting".to_string()),
            verbose: true,
        };
        assert_eq!(help_cmd.topic, Some("troubleshooting".to_string()));
        assert!(help_cmd.verbose);
    }
}