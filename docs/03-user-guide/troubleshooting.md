# Troubleshooting Guide

## Common Issues

### Analysis Fails Immediately
- **Symptom**: Command exits immediately with error
- **Solution**:
  ```bash
  # Check minimum requirements
  uveddi --version
  rustc --version
  
  # Run with debug logging
  RUST_LOG=debug uveddi analyze ./project
  ```

### Slow Performance
- **Symptom**: Analysis takes too long
- **Solutions**:
  ```bash
  # Limit parallel jobs
  uveddi analyze ./project --jobs 2
  
  # Exclude large files
  uveddi analyze ./project --max-file-size 1MB
  ```

### AI Provider Errors
- **Symptom**: "Failed to contact AI provider" messages
- **Solutions**:
  ```bash
  # Test provider connectivity
  curl $OLLAMA_API_URL
  
  # Switch to different provider
  uveddi analyze ./project --ai-provider anthropic
  ```

### Tree-sitter Feature
- As of vX.Y.Z, Tree-sitter is enabled by default for all supported languages. This provides more accurate parsing and analysis. If you encounter issues related to parsing, you may disable Tree-sitter via environment variable (`UVEDDI_DISABLE_TREE_SITTER=1`) or cargo feature flags.

## Error Messages

| Error | Solution |
|-------|----------|
| "Database connection failed" | Verify database is running and credentials are correct |
| "Invalid file type" | Check file extensions or use `--force` flag |
| "Analysis timeout" | Increase timeout with `--timeout 600` |
| "Missing dependencies" | Run `cargo build --features full` |

## Debugging Tips

1. Enable verbose output:
   ```bash
   RUST_LOG=trace uveddi analyze ./project
   ```

2. Generate debug report:
   ```bash
   uveddi debug-report > debug.log
   ```

3. Check system resources:
   ```bash
   # Monitor during analysis
   top -d 1
   ```

## Getting Help

1. Check known issues:
   ```bash
   uveddi known-issues
   ```

2. Create GitHub issue with:
   - Uveddi version (`uveddi --version`)
   - Debug log (`uveddi debug-report`)
   - Reproduction steps
