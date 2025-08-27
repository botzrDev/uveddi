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
    "init",                   // Project initialization
    "hooks",                  // Git hooks management
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
    "tutorials",
    "workflows",
    "performance",
    "ai-integration",
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
pub fn enhance_cli_error(original_error: &str, _command_context: Option<&str>) -> CliError {
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
    r#"🚀 Uveddi - Architectural Analysis Tool

📋 Essential Commands:
  analyze, a       Analyze your code for issues and anti-patterns  
  doctor, dr       Check system health & automatically fix issues
  init             Initialize Uveddi configuration for your project
  config, cfg      Manage and validate configuration settings
  hooks            Install Git hooks for automated analysis
  help             Show detailed help and tutorials

🎯 Getting Started (< 2 minutes):
  1. Initialize your project:
     uveddi init

  2. Check system health:
     uveddi doctor --fix

  3. Run your first analysis:
     uveddi analyze ./src --progress-details

  4. Generate interactive report:
     uveddi analyze ./src --output-format html --output reports/analysis.html

🔧 Common Workflows:
  • New project setup:     uveddi init --git-hooks
  • Quick health check:    uveddi dr --fix
  • Staged files analysis: git add . && uveddi analyze --changed-files
  • CI/CD integration:     uveddi hooks install --pre-push

⚡ Pro Tips:
  • Use aliases: 'a' for analyze, 'dr' for doctor, 'cfg' for config
  • Enable AI insights: --enable-ai (requires Ollama)
  • Get smart suggestions: uveddi config validate --suggestions
  • Parallel analysis: --parallel (for large codebases)

💡 Need More Help?
  uveddi help <topic>         # Get help on specific topics
  uveddi help examples        # See detailed usage examples  
  uveddi help tutorials       # Step-by-step guides
  uveddi help workflows       # Common development workflows
  uveddi help troubleshooting # Common issues and fixes

📚 Available Help Topics:
  installation, configuration, troubleshooting, ci-cd, plugins, examples, 
  tutorials, workflows, performance, ai-integration

🔗 Resources:
  • Documentation: https://github.com/botzrDev/uveddi
  • Issues & Support: https://github.com/botzrDev/uveddi/issues
  • Quick Start Guide: uveddi help tutorials
"#.to_string()
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

Quick Setup:
  uveddi hooks install --pre-push        # Install Git hooks
  uveddi ci setup github-actions         # Generate GitHub workflow
  uveddi ci setup gitlab                # Generate GitLab CI config

GitHub Actions Example:
  name: Code Quality Check
  on: [push, pull_request]
  jobs:
    analyze:
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v3
        - name: Install Uveddi
          run: cargo install uveddi
        - name: Health Check
          run: uveddi doctor
        - name: Analyze Code
          run: uveddi analyze ./src --output-format json --output analysis.json
        - name: Upload Results
          uses: actions/upload-artifact@v3
          with:
            name: analysis-results
            path: analysis.json

Git Hooks Integration:
  # Pre-commit hook (blocks commits with critical issues)
  uveddi hooks install --pre-commit --max-issues 5

  # Pre-push hook (full repository analysis)
  uveddi hooks install --pre-push --timeout 600

Custom Integration:
  # Machine-readable output
  uveddi analyze ./src --output-format json --progress-format json

  # Exit codes: 0=success, 1=issues found, 2=critical error
  # Quality gates
  uveddi analyze ./src --fail-on critical --max-issues 10

Performance Tips for CI:
  • Use --changed-files-only for faster pre-commit checks
  • Cache analysis results with --cache-dir
  • Set appropriate timeouts with --timeout
  • Use minimal features for faster builds
"#.to_string()),

        "examples" => Some(r#"
📚 Usage Examples

Basic Analysis:
  uveddi analyze ./src                   # Analyze source directory
  uveddi a .                            # Short alias for current directory
  uveddi analyze main.rs app.py         # Analyze specific files

Output Formats:
  uveddi analyze ./src --output-format html --output report.html
  uveddi analyze ./src --output-format json --output results.json  
  uveddi analyze ./src --output-format markdown --output analysis.md

AI-Powered Analysis:
  uveddi analyze ./src --enable-ai       # AI explanations (requires Ollama)
  uveddi analyze ./src --enable-ai --ollama-model deepseek-coder:6.7b

Progress Tracking:
  uveddi analyze ./src --progress-details              # Show file names
  uveddi analyze ./src --progress-format json          # JSON progress events
  uveddi analyze ./src --progress-format silent        # Quiet mode

Configuration Management:
  uveddi init                           # Interactive project setup
  uveddi init --template rust          # Use Rust project template
  uveddi config show                   # Display current configuration
  uveddi config validate --suggestions # Get optimization suggestions
  uveddi config set ollama_model "deepseek-coder:6.7b-instruct-q4_0"

Health & Diagnostics:
  uveddi doctor                         # Full system health check
  uveddi doctor --fix                   # Auto-fix common issues
  uveddi doctor --parsers               # Check language parser availability
  uveddi doctor --ai                    # Check AI integration
  uveddi doctor --output-format json    # Machine-readable health report

Git Integration:
  uveddi hooks install                  # Install pre-commit hook
  uveddi hooks install --pre-push      # Install pre-push hook
  uveddi hooks test pre-commit          # Test hook without committing
  uveddi hooks list --detailed         # Show hook status

Performance & Memory:
  uveddi analyze ./src --parallel                    # Multi-threaded analysis
  uveddi analyze ./src --memory-optimization         # For large codebases
  uveddi analyze ./src --timeout 600                 # 10-minute timeout
  uveddi analyze ./src --memory-limit-gb 4           # Set memory limit

Real-World Workflows:
  # New project setup
  cd my-project
  uveddi init --git-hooks
  uveddi doctor --fix
  git add .
  git commit -m "Initial commit"  # Runs analysis via hook

  # Daily development
  uveddi analyze --changed-files    # Analyze only modified files
  uveddi analyze ./src --enable-ai --output-format html --output today-report.html

  # Code review preparation
  uveddi analyze ./src --strict --output-format json > review-report.json
  uveddi config validate --suggestions > config-improvements.txt

  # CI/CD pipeline
  uveddi doctor && uveddi analyze ./src --output-format json --progress-format json
"#.to_string()),

        "tutorials" => Some(r#"
🎓 Step-by-Step Tutorials

Tutorial 1: First-Time Setup (2 minutes)
  1. Install Uveddi:
     cargo install uveddi

  2. Navigate to your project:
     cd /path/to/your/project

  3. Initialize configuration:
     uveddi init
     # Follow interactive prompts to set up configuration

  4. Check system health:
     uveddi doctor --fix
     # Resolves common issues automatically

  5. Run your first analysis:
     uveddi analyze ./src --progress-details
     # Watch analysis progress with file-by-file feedback

  6. Generate a report:
     uveddi analyze ./src --output-format html --output analysis-report.html
     # Open analysis-report.html in your browser

Tutorial 2: AI-Powered Analysis Setup
  1. Install Ollama:
     # Visit https://ollama.ai for installation instructions

  2. Start Ollama service:
     ollama serve

  3. Download recommended model:
     ollama pull deepseek-coder:6.7b-instruct-q4_0

  4. Configure Uveddi for AI:
     uveddi config set ollama_model "deepseek-coder:6.7b-instruct-q4_0"

  5. Run AI-powered analysis:
     uveddi analyze ./src --enable-ai
     # Get intelligent explanations for detected issues

Tutorial 3: Git Hooks Integration
  1. Install pre-commit hook:
     uveddi hooks install --pre-commit

  2. Configure hook settings:
     # Edit .uveddi/hooks.toml to customize behavior

  3. Test the hook:
     git add some-file.rs
     uveddi hooks test pre-commit
     # Verify hook behavior without committing

  4. Make a commit:
     git add .
     git commit -m "Your commit message"
     # Hook runs automatically, blocks if issues found

Tutorial 4: Custom Configuration
  1. Create project-specific config:
     uveddi init --template rust  # or python, javascript, typescript

  2. Customize thresholds:
     # Edit uveddi.toml
     [large_classes]
     max_logical_loc = 150      # Smaller classes for your team
     max_methods = 15

  3. Validate configuration:
     uveddi config validate --suggestions
     # Get recommendations for optimization

  4. Test with new settings:
     uveddi analyze ./src
     # Analysis uses your custom thresholds

Tutorial 5: CI/CD Integration
  1. Install hooks for team workflow:
     uveddi hooks install --pre-push

  2. Generate CI config:
     uveddi ci setup github-actions
     # Creates .github/workflows/uveddi.yml

  3. Test locally:
     uveddi analyze ./src --output-format json
     # Verify CI-compatible output

  4. Commit and push:
     git add .github/workflows/uveddi.yml
     git commit -m "Add Uveddi CI integration"
     git push
     # Triggers analysis in CI environment

Next Steps:
  • Explore advanced features: uveddi help workflows
  • Optimize performance: uveddi help performance  
  • Set up team standards: uveddi help configuration
  • Troubleshoot issues: uveddi help troubleshooting
"#.to_string()),

        "workflows" => Some(r#"
🔄 Common Development Workflows

Daily Development Workflow:
  # Morning routine
  uveddi doctor --fix                   # Health check & auto-fix
  git pull origin main                  # Get latest changes
  
  # During development
  uveddi analyze --changed-files        # Quick check on modified files
  git add .
  git commit -m "Feature: new component" # Pre-commit hook runs automatically
  
  # Before pushing
  uveddi analyze ./src --enable-ai --output-format html --output daily-report.html
  git push origin feature-branch       # Pre-push hook runs if configured

Code Review Workflow:
  # Prepare for review
  uveddi analyze ./src --strict --confidence-threshold 0.9
  uveddi config validate --suggestions > config-review.txt
  
  # Generate review materials
  uveddi analyze ./src --output-format markdown --output review-summary.md
  uveddi analyze ./src --output-format json --output detailed-results.json
  
  # Share with team
  git add review-summary.md detailed-results.json
  git commit -m "Add analysis results for review"

New Project Workflow:
  # Project initialization
  mkdir my-new-project && cd my-new-project
  git init
  uveddi init --git-hooks               # Interactive setup with Git integration
  
  # First analysis
  # ... write some code ...
  uveddi doctor                         # Verify everything works
  uveddi analyze ./src                  # First analysis run
  git add .
  git commit -m "Initial project setup" # Hooks validate the commit

Team Onboarding Workflow:
  # New team member setup
  git clone project-repo
  cd project-repo
  uveddi doctor --fix                   # Auto-configure environment
  
  # Learn project standards
  cat .uveddi/hooks.toml               # Review team hook configuration
  cat uveddi.toml                      # Review project analysis settings
  
  # Test setup
  uveddi hooks test pre-commit          # Verify hooks work
  uveddi analyze ./src --enable-ai      # Run analysis with explanations

Release Preparation Workflow:
  # Pre-release analysis
  uveddi analyze ./src --strict --max-issues 0 --output-format json > release-quality.json
  
  # Generate release documentation
  uveddi analyze ./src --output-format html --output release-analysis.html
  
  # Verify CI integration
  uveddi hooks test pre-push            # Ensure hooks pass
  git tag v1.0.0
  git push origin v1.0.0               # Triggers full CI analysis

Debugging Workflow:
  # When analysis reports false positives
  uveddi analyze problematic-file.rs --verbose --debug
  
  # Adjust configuration
  uveddi config set dead_code.confidence_threshold 0.9
  uveddi config validate --suggestions
  
  # Re-run with new settings
  uveddi analyze problematic-file.rs
  
  # Update ignore patterns if needed
  # Edit uveddi.toml to add ignore patterns

Performance Optimization Workflow:
  # Baseline measurement
  time uveddi analyze ./src
  
  # Enable optimizations
  uveddi analyze ./src --parallel --memory-optimization
  
  # For very large codebases
  uveddi analyze ./src --timeout 1800 --memory-limit-gb 8
  
  # Selective analysis
  uveddi analyze ./src/core ./src/api  # Analyze specific directories
  uveddi analyze --changed-files       # Only analyze modified files

Continuous Integration Workflow:
  # Local pre-commit (fast)
  uveddi hooks install --pre-commit --changed-files-only --timeout 60
  
  # Pre-push validation (thorough)  
  uveddi hooks install --pre-push --timeout 300
  
  # CI pipeline (comprehensive)
  # GitHub Actions runs: uveddi analyze ./src --output-format json --progress-format json
  
  # Post-merge analysis (reporting)
  # Generate reports for dashboard/metrics collection

Maintenance Workflow:
  # Weekly maintenance
  uveddi doctor                         # Check for system issues
  uveddi config validate --suggestions # Review configuration optimizations
  
  # Update ignored patterns based on new files
  # Review and update .uveddi/hooks.toml settings
  
  # Monthly: Review analysis trends and adjust thresholds
  # Compare current analysis with historical results
"#.to_string()),

        "performance" => Some(r#"
⚡ Performance Optimization

Build Performance (60-80% faster compilation):
  # Use development feature sets for faster builds
  cargo build --features=dev-minimal    # Ultra-fast (16.8s)
  cargo build --features=dev-core       # Balanced (13.0s)
  cargo build --features=dev-rust-only  # Single language (70-85% faster)

Analysis Performance:
  # Parallel processing
  uveddi analyze ./src --parallel                    # Multi-threaded analysis
  uveddi analyze ./src --parallel --jobs 8           # Specify thread count

  # Memory optimization
  uveddi analyze ./src --memory-optimization         # Enable memory optimizations
  uveddi analyze ./src --memory-limit-gb 4           # Set memory limit
  uveddi analyze ./src --memory-profile large        # Large project profile

  # Selective analysis
  uveddi analyze ./src/core ./src/api                # Specific directories
  uveddi analyze --changed-files                     # Only modified files
  uveddi analyze ./src --depth 5                     # Limit analysis depth

  # Timeout management
  uveddi analyze ./src --timeout 600                 # 10-minute timeout
  uveddi analyze ./src --timeout 0                   # No timeout (use carefully)

Caching Strategies:
  # Enable caching for repeated analysis
  uveddi analyze ./src --cache-enabled
  uveddi analyze ./src --cache-dir .cache/uveddi

  # Incremental analysis
  uveddi analyze ./src --incremental                 # Only analyze changes

Large Codebase Optimization:
  # For codebases > 100k LOC
  uveddi analyze ./src \
    --parallel \
    --memory-optimization \
    --memory-limit-gb 8 \
    --timeout 1800 \
    --cache-enabled

  # Divide and conquer approach
  uveddi analyze ./src/backend --output backend-analysis.json
  uveddi analyze ./src/frontend --output frontend-analysis.json
  
CI/CD Performance:
  # Pre-commit (fast feedback)
  uveddi hooks install --pre-commit --changed-files-only --timeout 60

  # Use minimal features in CI
  cargo build --features=dev-minimal    # Faster CI builds
  uveddi analyze ./src --progress-format silent --timeout 300

Memory Management:
  # Monitor memory usage
  uveddi analyze ./src --memory-profile --verbose

  # Large project settings
  uveddi analyze ./src \
    --memory-limit-gb 8 \
    --memory-optimization \
    --memory-profile large

  # WSL optimization (Windows Subsystem for Linux)
  export CARGO_TARGET_DIR=/tmp/uveddi-target
  export CARGO_BUILD_JOBS=4
  export RUSTFLAGS="-C link-arg=-fuse-ld=lld"

Troubleshooting Slow Analysis:
  # Profile analysis performance
  uveddi analyze ./src --profile --verbose

  # Check for bottlenecks
  uveddi doctor --performance           # System performance check

  # Reduce scope
  uveddi analyze ./src --exclude-patterns "test/**,node_modules/**,target/**"

  # Skip expensive features
  uveddi analyze ./src --no-ai --no-diagram-generation

Benchmark Your Setup:
  # Measure baseline
  time uveddi analyze ./src

  # Test optimizations
  time uveddi analyze ./src --parallel --memory-optimization

  # Compare feature sets
  time cargo build --features=dev-minimal
  time cargo build --features=production

Performance Monitoring:
  # Track analysis metrics
  uveddi analyze ./src --benchmark --output-format json | jq '.performance'
  
  # Monitor resource usage
  /usr/bin/time -v uveddi analyze ./src
  
  # Set up alerts for CI timeouts
  uveddi analyze ./src --timeout 300 || echo "Analysis timeout - consider optimization"
"#.to_string()),

        "ai-integration" => Some(r#"
🤖 AI Integration Guide

Quick Setup:
  1. Install Ollama:
     # macOS: brew install ollama
     # Linux: curl -fsSL https://ollama.ai/install.sh | sh
     # Windows: Download from https://ollama.ai

  2. Start Ollama:
     ollama serve

  3. Install recommended model:
     ollama pull deepseek-coder:6.7b-instruct-q4_0

  4. Configure Uveddi:
     uveddi config set ollama_model "deepseek-coder:6.7b-instruct-q4_0"

  5. Test AI integration:
     uveddi doctor --ai

Available Models (by use case):
  # Code analysis (recommended)
  ollama pull deepseek-coder:6.7b-instruct-q4_0
  
  # General purpose
  ollama pull codellama:7b-instruct
  
  # Lightweight (faster, less accurate)
  ollama pull llama2:7b-chat
  
  # Large projects (slower, more detailed)
  ollama pull deepseek-coder:33b-instruct

AI-Powered Analysis:
  # Basic AI analysis
  uveddi analyze ./src --enable-ai

  # AI with specific model
  uveddi analyze ./src --enable-ai --ollama-model deepseek-coder:33b-instruct

  # AI explanations in HTML report
  uveddi analyze ./src --enable-ai --output-format html --output ai-report.html

  # JSON output with AI insights
  uveddi analyze ./src --enable-ai --output-format json | jq '.ai_insights'

Configuration Options:
  # In uveddi.toml
  [ai]
  provider = "ollama"
  model = "deepseek-coder:6.7b-instruct-q4_0"
  api_url = "http://localhost:11434"
  timeout_seconds = 120
  max_context_length = 4096

  # Environment variables
  export OLLAMA_API_URL="http://localhost:11434"
  export OLLAMA_MODEL="deepseek-coder:6.7b-instruct-q4_0"

Advanced Features:
  # Context-aware explanations
  uveddi analyze problematic-file.rs --enable-ai --verbose

  # AI-powered suggestions
  uveddi analyze ./src --enable-ai --suggestions-only

  # Batch AI analysis
  for file in $(find ./src -name "*.rs"); do
    uveddi analyze "$file" --enable-ai --output-format json >> ai-batch-results.json
  done

Performance Tuning:
  # Faster AI (smaller model)
  uveddi config set ollama_model "llama2:7b-chat"
  
  # More accurate AI (larger model)  
  uveddi config set ollama_model "deepseek-coder:33b-instruct"
  
  # Timeout for large codebases
  uveddi analyze ./src --enable-ai --ai-timeout 300

Troubleshooting AI Issues:
  # Check Ollama status
  ollama list                           # Show installed models
  curl http://localhost:11434/api/tags  # API endpoint test

  # Diagnosis
  uveddi doctor --ai --verbose         # Detailed AI health check
  
  # Common fixes
  ollama pull deepseek-coder:6.7b-instruct-q4_0  # Ensure model is available
  ollama serve                          # Restart Ollama service
  
  # Test connectivity
  curl -X POST http://localhost:11434/api/generate \
    -H "Content-Type: application/json" \
    -d '{"model": "deepseek-coder:6.7b-instruct-q4_0", "prompt": "test"}'

Remote AI Setup:
  # Use remote Ollama instance
  uveddi config set ai.api_url "http://remote-server:11434"
  
  # With authentication (if required)
  export OLLAMA_API_KEY="your-api-key"
  uveddi analyze ./src --enable-ai

Integration with IDEs:
  # VS Code: Generate AI explanations for selected code
  uveddi analyze selected-file.rs --enable-ai --output-format markdown

  # Vim/Neovim: Integrate with quickfix
  uveddi analyze ./src --enable-ai --output-format json | jq -r '.issues[] | "file:\(.file_path) line:\(.line_number) \(.ai_explanation)"'

Best Practices:
  • Use AI sparingly in CI/CD (adds significant time)
  • Cache AI results for repeated analysis of same code
  • Choose model size based on your hardware and speed requirements
  • Use AI explanations for learning and code reviews
  • Combine AI insights with traditional static analysis
  • Review AI suggestions critically - they're not always correct
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