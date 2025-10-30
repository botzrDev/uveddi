# GPT Dev Prompt: Fix run_app() Deprecation Issue

## Problem Statement

The Uveddi CLI application is currently broken because `src/main.rs` calls the deprecated `uveddi::application::run_app()` function, which has been removed/deprecated in favor of direct CLI command handling. This prevents all CLI commands (`--version`, `--help`, `analyze`, `serve`, etc.) from functioning.

**Current Error:**
```
Error: Configuration error: run_app has been moved to main.rs CLI handling
→ Location: deprecated function
→ Suggestion: Check the configuration documentation for valid options
```

**Affected File:** `src/main.rs` (line 110)

## Objective

Replace the deprecated `run_app()` call with modern CLI handling that:
1. Parses command-line arguments using `clap`
2. Directly executes the appropriate command (analyze, serve, config, etc.)
3. Maintains async/await patterns properly
4. Preserves existing logging and error handling
5. Works with the current feature flag system

## Context

### Current Architecture

The codebase has:
- **CLI module**: `src/cli/` with command definitions
- **Commands**: Located in `src/cli/commands/`
  - `analyze.rs` - Analysis command
  - `serve.rs` - Web server command
  - `config.rs` - Configuration management
  - Others as needed
- **Feature flags**: `minimal`, `standard`, `full`
- **Async runtime**: Using `tokio::main` wrapper

### Relevant Code Locations

1. **Main entry point**: `/workspaces/uveddi/src/main.rs` (lines 62-118)
2. **CLI module**: `/workspaces/uveddi/src/cli/mod.rs`
3. **CLI commands**: `/workspaces/uveddi/src/cli/commands/*.rs`
4. **Application module**: `/workspaces/uveddi/src/application/mod.rs` (contains deprecated `run_app()`)

## Requirements

### Must Have

1. **Parse CLI arguments** using `clap` derive API
2. **Execute commands directly** without calling `run_app()`
3. **Maintain async patterns** - all command handlers are async
4. **Preserve error handling** - use `color_eyre` for error reporting
5. **Keep logging intact** - existing tracing/logging should continue to work
6. **Support all commands**:
   - `uveddi --version` - Show version
   - `uveddi --help` - Show help
   - `uveddi analyze <path>` - Run analysis
   - `uveddi serve` - Start web server
   - `uveddi config` - Manage configuration
   - Any other commands in `src/cli/commands/`

### Should Have

1. Clear error messages for invalid arguments
2. Consistent output formatting
3. Exit codes: 0 for success, 1 for errors
4. Proper async cleanup on exit

### Nice to Have

1. Progress indicators for long-running commands
2. Colored output for better UX
3. Command aliases or shortcuts

## Implementation Guide

### Step 1: Examine Existing CLI Structure

First, read these files to understand the current CLI structure:

```bash
# Read the CLI module structure
cat src/cli/mod.rs

# Read command implementations
cat src/cli/commands/analyze.rs
cat src/cli/commands/serve.rs
cat src/cli/commands/config.rs

# Read the deprecated application module to understand what run_app() was doing
cat src/application/mod.rs | head -200
```

Look for:
- How commands are defined (clap derive macros?)
- Command handler function signatures
- How configuration is passed to commands
- Any shared setup/teardown logic

### Step 2: Define CLI Structure in main.rs

Replace the `run_app()` call with proper CLI handling. The pattern should be:

```rust
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "uveddi")]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Analyze a codebase for architectural issues
    Analyze {
        /// Path to the directory or file to analyze
        #[arg(value_name = "PATH")]
        path: std::path::PathBuf,

        /// Output format (json, html, markdown, sarif)
        #[arg(long, default_value = "json")]
        output_format: String,

        /// Output file path
        #[arg(long, short = 'o')]
        output: Option<std::path::PathBuf>,

        /// Enable AI-powered analysis
        #[arg(long)]
        enable_ai: bool,

        /// Ollama model to use for AI analysis
        #[arg(long)]
        ollama_model: Option<String>,

        // Add other flags from src/cli/commands/analyze.rs
    },

    /// Start the web server dashboard
    Serve {
        /// Port to bind the server to
        #[arg(long, default_value = "8080")]
        port: u16,

        /// Host address to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        // Add other flags from src/cli/commands/serve.rs
    },

    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    // Add other commands found in src/cli/commands/
}

#[derive(clap::Subcommand, Debug)]
enum ConfigAction {
    /// Show current configuration
    Show,

    /// Set a configuration value
    Set {
        key: String,
        value: String,
    },

    /// Get a configuration value
    Get {
        key: String,
    },
}
```

### Step 3: Implement Command Execution

In the main function, replace the `run_app()` call with:

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // ... existing setup code (color_eyre, logging) ...

    info!("Starting Uveddi application");

    // Parse CLI arguments
    let cli = Cli::parse();

    // Execute the appropriate command
    let result = match cli.command {
        Commands::Analyze { path, output_format, output, enable_ai, ollama_model, .. } => {
            info!("Running analyze command on path: {:?}", path);

            // Import the analyze command handler
            use uveddi::cli::commands::analyze;

            // Create configuration from CLI args
            let config = analyze::AnalyzeConfig {
                path,
                output_format,
                output,
                enable_ai,
                ollama_model,
                // ... map other fields
            };

            // Execute the analyze command
            analyze::execute(config).await
        }

        Commands::Serve { port, host, .. } => {
            info!("Starting web server on {}:{}", host, port);

            use uveddi::cli::commands::serve;

            let config = serve::ServeConfig {
                port,
                host,
                // ... map other fields
            };

            serve::execute(config).await
        }

        Commands::Config { action } => {
            use uveddi::cli::commands::config;

            match action {
                ConfigAction::Show => config::show().await,
                ConfigAction::Set { key, value } => config::set(&key, &value).await,
                ConfigAction::Get { key } => config::get(&key).await,
            }
        }

        // Handle other commands...
    };

    // Handle result
    match result {
        Ok(_) => {
            info!("Command completed successfully");
            Ok(())
        }
        Err(e) => {
            error!(error = ?e, "Command failed");
            Err(color_eyre::eyre::eyre!(e))
        }
    }
}
```

### Step 4: Check Command Handler Signatures

For each command in `src/cli/commands/`, verify the handler function signature:

```rust
// Expected pattern:
pub async fn execute(config: AnalyzeConfig) -> Result<()> {
    // Implementation
}
```

If handlers don't exist or have different signatures, you may need to:
1. Create wrapper functions that match this pattern
2. Update the command modules to export the handlers
3. Ensure all handlers return `Result<()>` or compatible error type

### Step 5: Handle Special Cases

**Version Flag:**
The `#[command(version)]` attribute should automatically handle `--version`.

**Help Flag:**
The `#[command(about, long_about = None)]` attributes handle `--help`.

**Global Flags:**
If there are global flags (like `--verbose`, `--quiet`), add them to the `Cli` struct:

```rust
#[derive(Parser, Debug)]
struct Cli {
    /// Enable verbose logging
    #[arg(long, short = 'v', global = true)]
    verbose: bool,

    /// Suppress non-error output
    #[arg(long, short = 'q', global = true)]
    quiet: bool,

    #[command(subcommand)]
    command: Commands,
}
```

### Step 6: Test the Implementation

After making changes:

```bash
# Build
cargo build --bin uveddi --features standard

# Test basic commands
./target/debug/uveddi --version
./target/debug/uveddi --help
./target/debug/uveddi analyze --help
./target/debug/uveddi serve --help

# Test actual execution
./target/debug/uveddi analyze ./src/cli --output-format json --output /tmp/test.json
./target/debug/uveddi serve --port 9999 &
curl http://localhost:9999/health
```

## Common Pitfalls

### Issue 1: Command handlers not found

**Error:** `cannot find function execute in module uveddi::cli::commands::analyze`

**Solution:** Check if the command modules export the handler functions:
```rust
// In src/cli/commands/analyze.rs
pub async fn execute(config: AnalyzeConfig) -> Result<()> {
    // ...
}
```

### Issue 2: Type mismatches

**Error:** `expected struct Result, found enum Result`

**Solution:** Ensure consistent error types. Use type aliases if needed:
```rust
use color_eyre::eyre::Result;
// Or
type Result<T> = color_eyre::eyre::Result<T>;
```

### Issue 3: Async runtime issues

**Error:** `cannot call async function in a non-async function`

**Solution:** Ensure command handlers are properly awaited and main is `#[tokio::main]`.

### Issue 4: Missing clap derive

**Error:** `cannot find derive macro Parser in this scope`

**Solution:** Ensure clap is imported with derive feature:
```rust
use clap::{Parser, Subcommand};
```

Check `Cargo.toml`:
```toml
clap = { version = "4.x", features = ["derive"] }
```

## Validation

The fix is successful when:

1. ✅ `uveddi --version` shows version "1.0.0"
2. ✅ `uveddi --help` displays command list
3. ✅ `uveddi analyze --help` shows analyze options
4. ✅ `uveddi analyze ./src/cli --output-format json --output /tmp/test.json` completes without errors
5. ✅ `/tmp/test.json` exists and contains valid JSON
6. ✅ `uveddi serve --port 9999` starts server successfully
7. ✅ No deprecation warnings about `run_app()`
8. ✅ Exit code is 0 on success, non-zero on failure

## Debugging Tips

### Enable debug logging
```bash
RUST_LOG=debug ./target/debug/uveddi analyze ./src
```

### Check backtrace
```bash
RUST_BACKTRACE=full ./target/debug/uveddi analyze ./src
```

### Validate clap structure
Add this to test clap parsing:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        // Test version
        let args = vec!["uveddi", "--version"];
        let result = Cli::try_parse_from(args);
        assert!(result.is_ok() || result.is_err()); // --version exits early

        // Test analyze
        let args = vec!["uveddi", "analyze", "./src"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(matches!(cli.command, Commands::Analyze { .. }));
    }
}
```

## Expected File Changes

### Files to Modify

1. **src/main.rs** (primary changes)
   - Remove `run_app()` call (line 110)
   - Add CLI struct definitions
   - Add command execution logic
   - Estimated: +100-150 lines, -10 lines

2. **src/cli/mod.rs** (possibly)
   - May need to export command modules
   - Ensure command handlers are public

3. **src/cli/commands/*.rs** (possibly)
   - Ensure handler functions are exported
   - May need to adjust function signatures

### Files NOT to Modify

- Do NOT modify `/workspaces/uveddi/Cargo.toml` (version already updated)
- Do NOT modify `/workspaces/uveddi/CHANGELOG.md` (already complete)
- Do NOT modify scripts or documentation
- Do NOT modify `src/application/mod.rs` (leave deprecated code as-is for now)

## Success Criteria

### Minimum Viable Fix

- [x] `uveddi --version` works
- [x] `uveddi --help` works
- [x] `uveddi analyze <path>` runs successfully
- [x] No errors or panics during normal operation

### Complete Fix

- [x] All CLI commands work correctly
- [x] Error messages are clear and helpful
- [x] Exit codes are correct (0 = success, non-zero = failure)
- [x] Async cleanup happens properly
- [x] All tests in `scripts/test-v1-release.sh` pass

## Additional Resources

### Clap Documentation
- Derive API: https://docs.rs/clap/latest/clap/_derive/index.html
- Examples: https://github.com/clap-rs/clap/tree/master/examples

### Relevant Uveddi Code Patterns

Look for existing patterns in:
```bash
# Find clap usage
rg "use clap" src/

# Find async command handlers
rg "pub async fn.*execute" src/cli/

# Find Result type usage
rg "Result<\(\)>" src/cli/
```

## Prompt for GPT

---

**TASK:** Fix the deprecated `run_app()` call in Uveddi's main.rs

**CONTEXT:** I'm preparing Uveddi v1.0 for release. All documentation, scripts, and version numbers are updated. The only blocking issue is that `src/main.rs:110` calls `uveddi::application::run_app()` which is deprecated, causing all CLI commands to fail.

**REQUIREMENTS:**
1. Read `src/main.rs`, `src/cli/mod.rs`, and files in `src/cli/commands/` to understand the current structure
2. Replace the `run_app()` call (line 110) with proper clap-based CLI parsing
3. Implement direct command execution for: analyze, serve, config, and any other commands
4. Maintain async/await patterns and error handling
5. Ensure `uveddi --version`, `uveddi --help`, and `uveddi analyze ./src` all work correctly

**CONSTRAINTS:**
- Do NOT modify Cargo.toml, CHANGELOG.md, or documentation files
- Preserve existing logging and error handling setup
- Keep the `#[tokio::main]` async wrapper
- Use `color_eyre::eyre::Result` for error types
- Follow existing code style and patterns

**VALIDATION:**
After implementing, test with:
```bash
cargo build --bin uveddi --features standard
./target/debug/uveddi --version  # Should show "uveddi 1.0.0"
./target/debug/uveddi analyze ./src/cli --output-format json --output /tmp/test.json
```

**DELIVERABLES:**
1. Updated `src/main.rs` with working CLI handling
2. Any necessary changes to `src/cli/` modules to export handlers
3. Brief explanation of changes made

Please implement this fix following the detailed guide in this document. Start by reading the existing code structure, then implement the solution step by step.

---

## Notes for the Developer

- The codebase is at `/workspaces/uveddi/`
- Current branch: `finalversionv1.2`
- Rust version: 1.90.0
- All builds currently succeed with 0 errors (only warnings)
- The CLI structure likely already exists in `src/cli/`, so you may just need to wire it up in main.rs
- Look for existing `AnalyzeArgs`, `ServeArgs`, etc. structs that may already be defined

Good luck! This is the final piece needed for the v1.0 release. 🚀
