//! Enhanced CLI help and error handling
//!
//! Provides context-aware error messages, command suggestions,
//! and improved help text for better user experience.

use std::fmt;
use levenshtein::levenshtein;

/// Enhanced CLI error with context and suggestions
#[derive(Debug, Clone)]
pub struct CliError {
    pub message: String,
    pub context: Option<String>,
    pub suggestions: Vec<String>,
    pub help_topic: Option<String>,
}

impl CliError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            context: None,
            suggestions: Vec::new(),
            help_topic: None,
        }
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestions.push(suggestion.into());
        self
    }

    pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions.extend(suggestions);
        self
    }

    pub fn with_help_topic(mut self, topic: impl Into<String>) -> Self {
        self.help_topic = Some(topic.into());
        self
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "❌ {}", self.message)?;
        
        if let Some(context) = &self.context {
            writeln!(f, "📍 Context: {}", context)?;
        }
        
        if !self.suggestions.is_empty() {
            writeln!(f, "\n💡 Suggestions:")?;
            for suggestion in &self.suggestions {
                writeln!(f, "   • {}", suggestion)?;
            }
        }
        
        if let Some(help_topic) = &self.help_topic {
            writeln!(f, "\n📚 For more help: uveddi help {}", help_topic)?;
        }
        
        Ok(())
    }
}

impl std::error::Error for CliError {}

/// Available Uveddi commands for suggestion matching
const AVAILABLE_COMMANDS: &[&str] = &[
    "analyze", "a",           // Main analysis command + alias
    "doctor", "dr",           // Health check command + alias  
    "config", "cfg",          // Configuration command + alias
    "serve",                  // Web server command
    "tui",                    // Terminal UI command
    "plugin",                 // Plugin management
    "ci",                     // CI/CD integration
    "ui",                     // UI command
    "help",                   // Help command
];

/// Available help topics
const HELP_TOPICS: &[&str] = &[
    "installation",
    "configuration", 
    "troubleshooting",
    "ci-cd",
    "plugins",
    "examples",
];

/// Suggest similar commands when user makes a typo
pub fn suggest_command(input: &str) -> Vec<String> {
    let mut suggestions = AVAILABLE_COMMANDS
        .iter()
        .map(|cmd| (cmd, levenshtein(input, cmd)))
        .collect::<Vec<_>>();
    
    // Sort by edit distance
    suggestions.sort_by_key(|&(_, distance)| distance);
    
    // Return suggestions with distance <= 3 and at most 3 suggestions
    suggestions
        .iter()
        .take(3)
        .filter_map(|&(cmd, distance)| {
            if distance <= 3 {
                Some(cmd.to_string())
            } else {
                None
            }
        })
        .collect()
}

/// Generate contextual error message for common CLI mistakes
pub fn enhance_cli_error(original_error: &str, command_context: Option<&str>) -> CliError {
    let lower_error = original_error.to_lowercase();
    
    // Handle "command not found" errors
    if lower_error.contains("no such subcommand") || lower_error.contains("unrecognized subcommand") {
        if let Some(invalid_cmd) = extract_invalid_command(&lower_error) {
            let suggestions = suggest_command(&invalid_cmd);
            
            return CliError::new(format!("Unknown command: '{}'", invalid_cmd))
                .with_context("Uveddi supports analysis, configuration, and health checking commands")
                .with_suggestions(suggestions.into_iter()
                    .map(|s| format!("Try 'uveddi {}'", s))
                    .collect())
                .with_help_topic("commands");
        }
    }
    
    // Handle file not found errors
    if lower_error.contains("no such file") || lower_error.contains("not found") {
        return CliError::new("File or directory not found")
            .with_context("Check that the path exists and you have read permissions")
            .with_suggestion("Use absolute paths if relative paths aren't working")
            .with_suggestion("Run 'uveddi doctor' to check file permissions")
            .with_help_topic("troubleshooting");
    }
    
    // Handle permission errors
    if lower_error.contains("permission denied") || lower_error.contains("access denied") {
        return CliError::new("Permission denied")
            .with_context("Insufficient permissions to access the requested resource")
            .with_suggestion("Check file and directory permissions")
            .with_suggestion("Run 'uveddi doctor --fix' to attempt automatic fixes")
            .with_suggestion("Try running with appropriate permissions")
            .with_help_topic("troubleshooting");
    }
    
    // Handle configuration errors
    if lower_error.contains("config") || lower_error.contains("configuration") {
        return CliError::new("Configuration error")
            .with_context("There was a problem with the configuration settings")
            .with_suggestion("Run 'uveddi config validate' to check configuration")
            .with_suggestion("Use 'uveddi config reset' to restore default settings")
            .with_suggestion("Check ~/.config/uveddi/ for configuration files")
            .with_help_topic("configuration");
    }
    
    // Handle AI/Ollama related errors
    if lower_error.contains("ollama") || lower_error.contains("ai") || lower_error.contains("model") {
        return CliError::new("AI service error")
            .with_context("Problem connecting to or using AI services")
            .with_suggestion("Check if Ollama is running with 'ollama serve'")
            .with_suggestion("Run 'uveddi doctor --ai' to diagnose AI issues")
            .with_suggestion("Try running without --enable-ai to bypass AI features")
            .with_help_topic("ai-integration");
    }
    
    // Handle timeout errors
    if lower_error.contains("timeout") || lower_error.contains("timed out") {
        return CliError::new("Operation timed out")
            .with_context("The analysis took too long to complete")
            .with_suggestion("Increase timeout with --timeout <seconds>")
            .with_suggestion("Analyze a smaller subset of your codebase")
            .with_suggestion("Use --progress-format terminal to monitor progress")
            .with_help_topic("performance");
    }
    
    // Handle memory errors
    if lower_error.contains("memory") || lower_error.contains("out of memory") {
        return CliError::new("Memory limit exceeded")
            .with_context("The analysis required more memory than available")
            .with_suggestion("Use --memory-limit-gb to increase memory limit")
            .with_suggestion("Enable --memory-optimization for large codebases") 
            .with_suggestion("Analyze smaller sections of your codebase separately")
            .with_help_topic("performance");
    }
    
    // Default enhanced error
    CliError::new(original_error)
        .with_context("An unexpected error occurred")
        .with_suggestion("Run with --verbose for more detailed error information")
        .with_suggestion("Check 'uveddi doctor' for system health")
        .with_help_topic("troubleshooting")
}

/// Extract the invalid command from error message
fn extract_invalid_command(error: &str) -> Option<String> {
    // Look for patterns like "no such subcommand 'foo'" or "'bar' isn't a valid subcommand"
    if let Some(start) = error.find("subcommand '") {
        let start = start + 12; // length of "subcommand '"
        if let Some(end) = error[start..].find('\'') {
            return Some(error[start..start + end].to_string());
        }
    }
    
    if let Some(start) = error.find('\'') {
        if let Some(end) = error[start + 1..].find('\'') {
            return Some(error[start + 1..start + 1 + end].to_string());
        }
    }
    
    None
}

/// Generate help text for common use cases
pub fn generate_quick_help() -> String {
    format!(r#"
Uveddi - Code Analysis Tool

🚀 Quick Start:
  uveddi analyze ./src                    # Analyze your code
  uveddi analyze ./src --enable-ai        # With AI insights
  uveddi doctor                          # Check system health

📊 Commands:
  analyze (a)     Perform code analysis
  doctor (dr)     Run health diagnostics  
  config (cfg)    Manage configuration
  serve           Start web dashboard
  help            Show detailed help

💡 Examples:
  uveddi a ./src --output-format json     # JSON output
  uveddi dr --fix                         # Auto-fix issues
  uveddi cfg set ollama.model deepseek-coder

📚 Get Help:
  uveddi help <topic>                     # Detailed help
  uveddi <command> --help                 # Command help
  
Available topics: installation, configuration, troubleshooting, ci-cd

🔧 Troubleshooting:
  • Run 'uveddi doctor' to diagnose issues
  • Use --verbose for detailed error information  
  • Check https://github.com/botzrDev/uveddi for documentation
"#)
}

/// Generate help for a specific topic
pub fn generate_topic_help(topic: &str) -> Option<String> {
    match topic {
        "installation" => Some(r#"
📦 Installation Help

System Requirements:
  • Rust 1.70+ (for building from source)
  • Git (for development workflow)
  • Optional: Ollama (for AI features)

Installation Options:
  # From crates.io
  cargo install uveddi

  # From source (latest features)
  git clone https://github.com/botzrDev/uveddi.git
  cd uveddi
  cargo build --release --features=production

  # Development build (faster compilation)
  cargo build --features=dev-minimal

Verification:
  uveddi doctor                          # Check installation health
  uveddi --version                       # Verify version
"#.to_string()),

        "configuration" => Some(r#"
⚙️  Configuration Help

Configuration Files:
  • ~/.config/uveddi/config.toml         # User settings
  • ./uveddi.toml                        # Project settings
  • .env                                 # Environment variables

Quick Setup:
  uveddi init                            # Interactive setup
  uveddi config show                     # View current config
  uveddi config set key value            # Update setting

Common Settings:
  [analysis]
  languages = ["rust", "python", "javascript"]
  max_depth = 10

  [ai]
  provider = "ollama"
  model = "deepseek-coder:6.7b"

  [output]
  format = "html"
  include_timing = true
"#.to_string()),

        "troubleshooting" => Some(r#"
🔧 Troubleshooting Help

Common Issues:
  1. "Command not found"
     → Check PATH or use full path to binary

  2. "Permission denied"
     → Run 'uveddi doctor --fix' for auto-repair
     → Check file permissions manually

  3. "Analysis timeout"
     → Use --timeout <seconds> for large codebases
     → Try --memory-optimization

  4. "AI not working"
     → Check Ollama with 'ollama serve'
     → Run 'uveddi doctor --ai'

Diagnostic Commands:
  uveddi doctor                          # Full system check
  uveddi doctor --parsers                # Check language support
  uveddi doctor --ai                     # Check AI integration
  uveddi config validate                 # Verify configuration
"#.to_string()),

        "ci-cd" => Some(r#"
🚀 CI/CD Integration Help

GitHub Actions:
  uveddi ci setup github-actions         # Generate workflow

GitLab CI:
  uveddi ci setup gitlab                # Generate .gitlab-ci.yml

Custom Integration:
  # JSON output for parsing
  uveddi analyze ./src --progress-format json --output-format json

  # Exit codes
  # 0 = success, 1 = issues found, 2 = critical error

Quality Gates:
  uveddi analyze ./src --max-critical 0 --max-debt-score 50

Example Workflow:
  steps:
    - run: uveddi doctor                 # Health check
    - run: uveddi analyze ./src --output-format json
    - run: upload results to code quality dashboard
"#.to_string()),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_suggestions() {
        assert_eq!(suggest_command("analze"), vec!["analyze"]); // typo
        assert_eq!(suggest_command("doctr"), vec!["doctor"]); // typo
        assert!(suggest_command("totally-wrong-command").is_empty()); // too different
    }

    #[test]
    fn test_error_enhancement() {
        let error = enhance_cli_error("no such subcommand 'analze'", None);
        assert!(error.message.contains("analze"));
        assert!(!error.suggestions.is_empty());
    }

    #[test]
    fn test_help_generation() {
        let help = generate_quick_help();
        assert!(help.contains("analyze"));
        assert!(help.contains("doctor"));
    }
}
"#.to_string()),