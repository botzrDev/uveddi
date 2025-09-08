# Uveddi Reference Documentation

This section provides comprehensive reference documentation for Uveddi - the architectural analysis tool for modern codebases.

## Table of Contents

### 📋 [CLI Reference](cli-reference.md)
Complete command-line interface reference including:
- All available commands and subcommands
- Command-line flags and options
- Usage examples and patterns
- Performance optimization flags
- Integration with CI/CD systems

### 🌐 [API Reference](api-reference.md)
Complete API documentation covering:
- REST API endpoints (current status and roadmap)
- Rust library API for advanced integrations
- Plugin development APIs
- WebSocket interfaces
- Authentication and authorization

### ⚙️ [Configuration Reference](configuration-reference.md)
Comprehensive configuration documentation:
- Configuration file formats (TOML, JSON)
- Environment variables
- Command-line overrides
- Memory optimization settings
- AI provider configurations
- Plugin settings and permissions

### 🚨 [Error Codes Reference](error-codes.md)
Error handling and troubleshooting guide:
- Categorized error codes with descriptions
- Common solutions and workarounds
- Debugging techniques
- Logging configuration
- Support and issue reporting

---

## Quick Reference

### Essential Commands
```bash
# Basic analysis
uveddi analyze ./src --output-format html

# With AI insights
uveddi analyze ./src --enable-ai --ollama-model deepseek-coder:6.7b

# Memory optimized (default)
uveddi analyze ./src --memory-profile large --memory-limit-gb 8

# Check system status
uveddi doctor

# Get help
uveddi --help
```

### Key Configuration Paths
- **Global Config**: `~/.config/uveddi/config.toml`
- **Project Config**: `./uveddi.toml`
- **Plugin Directory**: `~/.uveddi/plugins/`
- **Cache Directory**: `~/.cache/uveddi/`

### Important Environment Variables
```bash
export UVEDDI_LOG=debug              # Enable debug logging
export UVEDDI_AI_ENABLED=true        # Enable AI features
export OLLAMA_API_URL=http://localhost:11434
export UVEDDI_MEMORY_LIMIT_GB=8      # Override memory limit
```

---

## Project Status (v1.0.0-prerelease)

### ✅ Working Features
- **Multi-language AST parsing** (Rust, Python, JavaScript, TypeScript)
- **10+ anti-pattern detectors** (God objects, circular dependencies, dead code, etc.)
- **Multiple output formats** (HTML, JSON, Markdown, Text)
- **Memory optimization** (automatic system detection and tuning)
- **Plugin architecture** (WASM-based extensibility)
- **CLI interface** (comprehensive command-line tool)

### 🚧 Development Status
- **Build Issues**: Currently has compilation errors preventing usage
- **API Server**: Partially functional with limited endpoints
- **Web Dashboard**: Implementation in progress
- **CI/CD Integration**: Command structure ready, needs build fixes

### 🎯 Next Milestones
1. **Fix Build System**: Resolve compilation errors and dependency issues
2. **Stabilize API**: Complete REST endpoint implementations
3. **Plugin Ecosystem**: Release official plugin development kit
4. **Performance Optimization**: Benchmark and optimize for large codebases

---

## Getting Help

### Documentation
- **User Guide**: [../user-guide/](../user-guide/)
- **Development Guide**: [../development/](../development/)
- **Examples**: [../examples/](../examples/)

### Community Support
- **GitHub Issues**: https://github.com/botzrDev/uveddi/issues
- **Discussion Forum**: https://github.com/botzrDev/uveddi/discussions
- **Discord Community**: https://discord.gg/uveddi

### Reporting Issues
Use the debug report generator for comprehensive issue reporting:
```bash
uveddi debug-report > debug.log
```

Include the generated debug.log when reporting issues for faster resolution.

---

## Reference Navigation

Use the links above to navigate to specific reference sections, or use the search functionality to find specific information across all reference documents.

Each reference document includes:
- **Comprehensive coverage** of the topic area
- **Working examples** and code snippets
- **Troubleshooting sections** for common issues
- **Cross-references** to related documentation

Happy analyzing! 🦀