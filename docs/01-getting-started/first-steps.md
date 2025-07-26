# First Steps with Uveddi Alpha

## Alpha Release Status

**⚠️ Important**: This is an alpha release. The CLI interface is fully functional, but the analysis engine is in development. You will see "Analysis execution failed" errors - this is expected behavior.

## Getting Started

1. Build the alpha release:
   ```bash
   cd uveddi
   cargo build --release --features="alpha"
   ```

2. Check available commands:
   ```bash
   ./target/release/uveddi --help
   ```

## Testing the CLI Interface

### View Available Options
```bash
# See all analysis options (comprehensive CLI)
./target/release/uveddi analyze --help

# See configuration commands
./target/release/uveddi config --help
```

### Test CLI Functionality (Expected to show analysis errors)
```bash
# Test basic command structure - will show expected "execution failed" error
./target/release/uveddi analyze /path/to/your/project --output-format=markdown

# Test JSON output format
./target/release/uveddi analyze /path/to/your/project --output-format=json --output=report.json

# Test with dead code detection options
./target/release/uveddi analyze /path/to/your/project --dead-code-confidence=0.8

# Test with large class detection options
./target/release/uveddi analyze /path/to/your/project --large-classes-max-methods=5
```

## What Works in Alpha

### ✅ Fully Functional
- Complete CLI argument parsing and validation
- Comprehensive help system (`--help` on all commands)
- Configuration commands (`uveddi config show`, `set`, `validate`)
- All output format options (`--output-format=json|markdown|text`)
- Error handling with clear, helpful messages
- Memory optimization flags
- AI integration flags (when engine is complete)
- Image rendering options

### ⚠️ Alpha/Development Status
- **Analysis Engine**: Shows "Analysis execution failed" - engine in development
- **TUI Interface**: Available as separate test binary: `cargo run --bin tui_test --features="tui"`

## Expected Alpha Behavior

When you run analysis commands, you will see:
```bash
Error: Analysis error in unknown:0: Unexpected error: Analysis execution failed
  → Context: generic operation
  → Suggestion: Check logs for more details
  → Context: analyze command
  → Suggestion: Check file encoding and language support
```

**This is expected** - the CLI infrastructure is complete and robust, but the analysis engine is still being developed.

## Testing the Complete CLI

Test all the CLI options to verify the interface:
```bash
# Memory optimization options
./target/release/uveddi analyze ./project --enable-memory-optimization --memory-limit-gb=4

# AI integration options (ready for when engine is complete)
./target/release/uveddi analyze ./project --enable-ai --ollama-model=deepseek-coder

# Image rendering options
./target/release/uveddi analyze ./project --enable-image-rendering --rendering-service-url=http://localhost:3001

# Configuration testing
./target/release/uveddi config show

# Config validation (will show expected "file not found" error)
./target/release/uveddi config validate
```

## Next Steps for Alpha Testing

1. **Verify CLI completeness**: Test all argument combinations
2. **Check help system**: Ensure all `--help` outputs are comprehensive
3. **Test error handling**: Try invalid arguments to see error quality
4. **Report issues**: Any CLI crashes or unclear error messages
5. **Await analysis engine**: Engine development is the next milestone