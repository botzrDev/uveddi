# Using Plugins

This guide covers how to use plugins in Uveddi to extend analysis capabilities. Plugins allow you to add custom detectors, rules, and analysis features without modifying the core application.

## Overview

Uveddi's plugin system allows you to:

- Install community plugins for additional analysis capabilities
- Use specialized detectors for specific frameworks or domains
- Apply organization-specific coding standards and policies
- Extend language support beyond the built-in languages

## Plugin Types

### Built-in Plugin Categories

- **Performance Analyzers**: Detect performance bottlenecks and optimization opportunities
- **Security Scanners**: Identify security vulnerabilities and compliance issues
- **Code Quality**: Analyze complexity, maintainability, and code smells
- **Framework-Specific**: Specialized analysis for specific frameworks (React, Vue, etc.)
- **Enterprise**: Organization-specific rules and compliance checking

## Installing Plugins

### From Plugin Registry

```bash
# Search for available plugins
uveddi plugin search security

# Get plugin information
uveddi plugin info security-scanner

# Install a plugin
uveddi plugin install security-scanner

# Install specific version
uveddi plugin install security-scanner@2.1.0
```

### From Local Files

```bash
# Install from local WASM file
uveddi plugin install ./my-plugin.wasm

# Install from URL
uveddi plugin install https://example.com/plugin.wasm
```

### From Git Repository

```bash
# Install and build from source
uveddi plugin install git+https://github.com/user/plugin.git
```

## Managing Plugins

### List Installed Plugins

```bash
# List all installed plugins
uveddi plugin list

# Show detailed information
uveddi plugin list --verbose

# Show only active plugins
uveddi plugin list --active
```

### Plugin Status and Control

```bash
# Check plugin status
uveddi plugin status my-plugin

# Enable a plugin
uveddi plugin enable my-plugin

# Disable a plugin
uveddi plugin disable my-plugin

# Remove a plugin
uveddi plugin remove my-plugin
```

### Updating Plugins

```bash
# Update all plugins
uveddi plugin update

# Update specific plugin
uveddi plugin update security-scanner

# Check for available updates
uveddi plugin outdated
```

## Using Plugins in Analysis

### Running Analysis with Plugins

```bash
# Use all enabled plugins
uveddi analyze ./src

# Use specific plugins only
uveddi analyze ./src --plugins security-scanner,performance-analyzer

# Exclude specific plugins
uveddi analyze ./src --exclude-plugins deprecated-detector

# Run with plugin-specific options
uveddi analyze ./src --plugin-config security-scanner:strict=true
```

### Plugin Configuration

Create a `.uveddi/plugins.toml` file in your project root:

```toml
# Global plugin configuration
[plugins]
# Enable plugins by default
auto_enable = true

# Plugin-specific configurations
[plugins.security-scanner]
enabled = true
strict_mode = true
ignore_patterns = ["**/test/**", "**/vendor/**"]
severity_threshold = "medium"

[plugins.performance-analyzer]
enabled = true
complexity_threshold = 15
memory_analysis = true

[plugins.code-quality]
enabled = false  # Explicitly disabled
```

### Environment-Based Configuration

```bash
# Development environment
UVEDDI_PLUGINS_PROFILE=dev uveddi analyze ./src

# Production environment  
UVEDDI_PLUGINS_PROFILE=prod uveddi analyze ./src

# CI environment with strict rules
UVEDDI_PLUGINS_PROFILE=ci uveddi analyze ./src
```

## Plugin Permissions and Security

### Understanding Plugin Permissions

Plugins run in a secure sandbox and must declare their required permissions:

- **File Access**: Read/write permissions for specific directories
- **Network Access**: Access to external APIs or services
- **System Information**: Access to system metrics or environment variables
- **Database Access**: Read/write to analysis database

### Reviewing Plugin Permissions

```bash
# View plugin permissions before installation
uveddi plugin inspect security-scanner

# View permissions of installed plugin
uveddi plugin permissions security-scanner
```

### Security Best Practices

1. **Review Permissions**: Always review what permissions a plugin requests
2. **Trusted Sources**: Install plugins from trusted sources
3. **Regular Updates**: Keep plugins updated to latest versions
4. **Monitor Usage**: Monitor plugin resource usage and behavior
5. **Sandbox Limits**: Plugins are limited by memory and execution time

## Advanced Usage

### Plugin Profiles

Create different plugin profiles for different contexts:

```toml
# ~/.uveddi/profiles.toml
[profiles.development]
plugins = ["code-quality", "performance-analyzer"]
auto_fix = true
strict_mode = false

[profiles.production]
plugins = ["security-scanner", "compliance-checker"]
auto_fix = false
strict_mode = true
fail_on_error = true

[profiles.code_review]
plugins = ["all"]
generate_suggestions = true
detailed_reports = true
```

Use profiles:

```bash
# Use development profile
uveddi analyze ./src --profile development

# Use production profile for CI
uveddi analyze ./src --profile production
```

### Plugin Chaining and Dependencies

Some plugins work better when used together:

```toml
[plugins.advanced-security]
enabled = true
depends_on = ["security-scanner", "dependency-tracker"]
enhanced_analysis = true
```

### Custom Plugin Configuration

Create plugin-specific configuration files:

```bash
# .uveddi/plugins/security-scanner.toml
[rules]
sql_injection = { enabled = true, severity = "high" }
xss_detection = { enabled = true, severity = "high" }
hardcoded_secrets = { enabled = true, severity = "critical" }

[ignore]
patterns = ["test/**", "node_modules/**"]
files = ["config.example.js"]

[reporting]
include_examples = true
suggest_fixes = true
```

## Troubleshooting

### Common Issues

#### Plugin Won't Load

```bash
# Check plugin compatibility
uveddi plugin validate ./my-plugin.wasm

# View detailed error information
uveddi plugin install ./my-plugin.wasm --verbose

# Check system requirements
uveddi plugin requirements my-plugin
```

#### Performance Issues

```bash
# Monitor plugin performance
uveddi plugin monitor --duration 60s

# Profile specific plugin
uveddi plugin profile security-scanner

# Check resource usage
uveddi plugin stats
```

#### Configuration Problems

```bash
# Validate plugin configuration
uveddi plugin config validate

# Reset plugin configuration
uveddi plugin config reset my-plugin

# View effective configuration
uveddi plugin config show my-plugin
```

### Getting Help

```bash
# Get help for plugin commands
uveddi plugin help

# Get plugin-specific help
uveddi plugin help security-scanner

# View plugin documentation
uveddi plugin docs security-scanner
```

## Integration with CI/CD

### GitHub Actions

```yaml
name: Code Analysis
on: [push, pull_request]

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Uveddi
        run: cargo install uveddi
        
      - name: Install plugins
        run: |
          uveddi plugin install security-scanner
          uveddi plugin install performance-analyzer
          
      - name: Run analysis
        run: uveddi analyze ./src --output-format json --output analysis-results.json
        
      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: analysis-results
          path: analysis-results.json
```

### Jenkins Pipeline

```groovy
pipeline {
    agent any
    
    stages {
        stage('Setup') {
            steps {
                sh 'cargo install uveddi'
                sh 'uveddi plugin install security-scanner performance-analyzer'
            }
        }
        
        stage('Analysis') {
            steps {
                sh 'uveddi analyze ./src --profile ci'
            }
            
            post {
                always {
                    archiveArtifacts artifacts: '**/uveddi-report.*'
                }
            }
        }
    }
}
```

### GitLab CI

```yaml
stages:
  - analysis

code_analysis:
  stage: analysis
  image: rust:latest
  script:
    - cargo install uveddi
    - uveddi plugin install security-scanner
    - uveddi analyze ./src --profile production
  artifacts:
    reports:
      junit: uveddi-results.xml
    paths:
      - uveddi-report.html
  rules:
    - if: $CI_PIPELINE_SOURCE == "merge_request_event"
```

## Best Practices

### Plugin Selection

1. **Start Small**: Begin with essential plugins and add more as needed
2. **Quality over Quantity**: Choose well-maintained plugins from trusted sources
3. **Performance Impact**: Consider the performance impact of multiple plugins
4. **Compatibility**: Ensure plugins work well together
5. **Documentation**: Choose plugins with good documentation

### Configuration Management

1. **Version Control**: Keep plugin configurations in version control
2. **Environment Separation**: Use different configurations for different environments  
3. **Documentation**: Document why specific plugins and configurations are used
4. **Regular Review**: Regularly review and update plugin configurations
5. **Testing**: Test plugin configurations before deploying to production

### Maintenance

1. **Regular Updates**: Keep plugins updated to latest versions
2. **Security Monitoring**: Monitor for security updates and vulnerabilities
3. **Performance Monitoring**: Track plugin performance over time
4. **Cleanup**: Remove unused or deprecated plugins
5. **Backup**: Backup plugin configurations and customizations

## Next Steps

- [Available Plugins](available-plugins.md) - Browse available plugins
- [Plugin Development](../../development/plugins/development-guide.md) - Create your own plugins
- [Configuration Reference](../configuration-reference.md) - Complete configuration options
- [Security Guidelines](../../security/plugin-security.md) - Plugin security best practices