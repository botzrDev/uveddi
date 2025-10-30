# Knowledge Plugin Development Guide

## Overview

The Uveddi Knowledge Plugin System (UV-338) enables external developers and organizations to extend the AI Knowledge Library with custom anti-patterns, domain-specific knowledge, and specialized detection methods. This guide provides comprehensive instructions for developing, testing, and deploying knowledge plugins.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Plugin Architecture](#plugin-architecture)
3. [Plugin Types](#plugin-types)
4. [Development Framework](#development-framework)
5. [Security and Permissions](#security-and-permissions)
6. [Performance Requirements](#performance-requirements)
7. [Testing Your Plugin](#testing-your-plugin)
8. [Deployment and Distribution](#deployment-and-distribution)
9. [Best Practices](#best-practices)
10. [Examples](#examples)

## Getting Started

### Prerequisites

- Rust 1.70+ with async/await support
- Understanding of Uveddi's knowledge schema
- Familiarity with anti-pattern detection concepts

### Quick Start

1. **Create a new plugin project:**
```bash
cargo new my-knowledge-plugin --lib
cd my-knowledge-plugin
```

2. **Add Uveddi dependencies to `Cargo.toml`:**
```toml
[dependencies]
uveddi = { path = "../uveddi" }
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
```

3. **Implement the basic plugin structure:**
```rust
use uveddi::plugins::knowledge::*;
use uveddi::plugins::development::*;
use async_trait::async_trait;

pub struct MyKnowledgePlugin {
    metadata: KnowledgePluginMetadata,
    patterns: Vec<PatternKnowledge>,
    initialized: bool,
}

#[async_trait]
impl KnowledgePlugin for MyKnowledgePlugin {
    // Implementation details...
}
```

## Plugin Architecture

### Core Components

The knowledge plugin system consists of several key components:

- **KnowledgePluginSystem**: Main orchestrator for plugin management
- **KnowledgePlugin trait**: Core interface all plugins must implement
- **Plugin Registry**: Manages plugin discovery and metadata
- **Security Manager**: Enforces security policies and sandboxing
- **Performance Monitor**: Tracks and validates plugin performance
- **Integration Manager**: Merges plugin knowledge with core library

### Plugin Lifecycle

1. **Discovery**: Plugin is discovered in configured directories
2. **Validation**: Security and performance validation
3. **Registration**: Plugin is registered in the system
4. **Initialization**: Plugin initializes its knowledge base
5. **Active**: Plugin serves knowledge requests
6. **Shutdown**: Plugin cleans up resources

## Plugin Types

### 1. Anti-Pattern Plugins

Define custom anti-patterns for specific domains or technologies.

```rust
use uveddi::plugins::knowledge::*;

#[async_trait]
impl AntiPatternPlugin for MyAntiPatternPlugin {
    async fn get_custom_patterns(&self) -> Result<Vec<CustomPatternDefinition>, PluginError> {
        // Return custom anti-pattern definitions
    }

    async fn validate_pattern(
        &self,
        pattern: &PatternKnowledge,
    ) -> Result<ValidationResult, PluginError> {
        // Validate pattern against domain knowledge
    }
}
```

### 2. Language Plugins

Extend support for additional programming languages.

```rust
async fn get_language_knowledge(
    &self,
    language: SourceLanguage,
) -> Result<Option<LanguageKnowledge>, PluginError> {
    match language {
        SourceLanguage::Go => Ok(Some(self.go_knowledge.clone())),
        _ => Ok(None),
    }
}
```

### 3. Framework Plugins

Provide framework-specific knowledge and patterns.

```rust
async fn get_framework_knowledge(
    &self,
    framework: &str,
) -> Result<Option<FrameworkKnowledge>, PluginError> {
    match framework {
        "Vue.js" => Ok(Some(self.vue_knowledge.clone())),
        "Angular" => Ok(Some(self.angular_knowledge.clone())),
        _ => Ok(None),
    }
}
```

### 4. Enterprise Plugins

Implement organization-specific patterns and compliance rules.

```rust
#[async_trait]
impl EnterprisePlugin for MyEnterprisePlugin {
    async fn get_organization_patterns(
        &self,
        org_id: &str,
    ) -> Result<Vec<PatternKnowledge>, PluginError> {
        // Return organization-specific patterns
    }

    async fn get_compliance_knowledge(
        &self,
        compliance_framework: &str,
    ) -> Result<ComplianceKnowledge, PluginError> {
        // Return compliance-specific knowledge
    }
}
```

## Development Framework

### Plugin Macro

Use the `define_knowledge_plugin!` macro for easy plugin creation:

```rust
use uveddi::define_knowledge_plugin;

define_knowledge_plugin!(
    name: "My Custom Plugin",
    version: "1.0.0",
    author: "Your Name",
    description: "Description of your plugin",
    plugin_type: KnowledgePluginType::AntiPattern,
    supported_languages: [SourceLanguage::Rust, SourceLanguage::Python],
    implementation: MyKnowledgePlugin
);
```

### Pattern Definition

Define custom patterns using the schema:

```rust
use uveddi::ai::knowledge::schema::*;
use uveddi::ai::knowledge::compression::CompressedString;

fn create_custom_pattern() -> PatternKnowledge {
    PatternKnowledge {
        id: "my_custom_pattern".to_string(),
        name: "My Custom Anti-Pattern".to_string(),
        definition: CompressedString::new("Description of the anti-pattern"),
        symptoms: vec![
            CompressedString::new("Symptom 1"),
            CompressedString::new("Symptom 2"),
        ],
        impact: ImpactLevel::High,
        category: AntiPatternCategory::Performance,
        detection_methods: vec![
            DetectionMethod {
                method_type: DetectionMethodType::Structural,
                description: CompressedString::new("How to detect this pattern"),
                thresholds: vec![("metric_name".to_string(), 5.0)],
                confidence: 0.85,
            }
        ],
        solutions: vec![
            SolutionPattern {
                name: "Refactoring Solution".to_string(),
                description: CompressedString::new("How to fix the pattern"),
                implementation_steps: vec![
                    CompressedString::new("Step 1: Analyze the code"),
                    CompressedString::new("Step 2: Apply refactoring"),
                ],
                benefits: vec![
                    CompressedString::new("Improved performance"),
                    CompressedString::new("Better maintainability"),
                ],
                effort_level: EffortLevel::Medium,
            }
        ],
        examples: CodeExamples {
            primary: vec![],
            variations: HashMap::new(),
        },
        language_variations: HashMap::new(),
        related_patterns: vec!["related_pattern_id".to_string()],
        tags: vec!["performance".to_string(), "custom".to_string()],
        frequency_score: 0.7,
        detection_confidence: 0.85,
    }
}
```

## Security and Permissions

### Permission System

Plugins must declare their required permissions:

```rust
KnowledgePluginPermissions {
    network: NetworkPermissions {
        allow_http: false,
        allow_https: true,
        allowed_domains: vec!["api.mycompany.com".to_string()],
        allowed_ports: vec![443],
    },
    filesystem: FilesystemPermissions {
        allow_read: true,
        allow_write: false,
        allowed_paths: vec![PathBuf::from("/etc/myapp/patterns")],
    },
    system: SystemPermissions {
        allow_process_execution: false,
        allow_env_access: false,
        max_memory_mb: 50,
        max_execution_time_ms: 2000,
    },
    data: DataPermissions {
        allow_knowledge_read: true,
        allow_knowledge_write: false,
        allow_analysis_data: true,
    },
}
```

### Security Best Practices

1. **Minimal Permissions**: Request only the permissions you need
2. **Input Validation**: Validate all external inputs
3. **Resource Limits**: Respect memory and execution time limits
4. **Error Handling**: Handle errors gracefully without exposing sensitive information
5. **Audit Logging**: Log security-relevant operations

## Performance Requirements

### Performance Targets

- **Initialization**: < 1000ms
- **Pattern Retrieval**: < 100ms
- **Memory Usage**: < 100MB per plugin
- **Integration Impact**: < 10% overhead on core system

### Performance Optimization

1. **Lazy Loading**: Load patterns only when needed
2. **Caching**: Cache frequently accessed data
3. **Compression**: Use compressed strings for large text data
4. **Batch Operations**: Process multiple patterns together
5. **Async Operations**: Use async/await for I/O operations

### Performance Monitoring

```rust
impl KnowledgePlugin for MyPlugin {
    async fn health_check(&self) -> Result<PluginHealthStatus, PluginError> {
        let start = std::time::Instant::now();
        
        // Perform health check operations
        let patterns = self.get_patterns().await?;
        
        let elapsed = start.elapsed();
        
        if elapsed.as_millis() > 100 {
            Ok(PluginHealthStatus::Warning(
                format!("Slow pattern retrieval: {}ms", elapsed.as_millis())
            ))
        } else {
            Ok(PluginHealthStatus::Healthy)
        }
    }
}
```

## Testing Your Plugin

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_plugin_initialization() {
        let mut plugin = MyKnowledgePlugin::new(create_test_metadata());
        assert!(plugin.initialize().await.is_ok());
    }

    #[tokio::test]
    async fn test_pattern_retrieval() {
        let mut plugin = MyKnowledgePlugin::new(create_test_metadata());
        plugin.initialize().await.unwrap();
        
        let patterns = plugin.get_patterns().await.unwrap();
        assert!(!patterns.is_empty());
    }

    #[tokio::test]
    async fn test_performance_requirements() {
        let mut plugin = MyKnowledgePlugin::new(create_test_metadata());
        plugin.initialize().await.unwrap();
        
        let start = std::time::Instant::now();
        let _patterns = plugin.get_patterns().await.unwrap();
        let elapsed = start.elapsed();
        
        assert!(elapsed.as_millis() < 100, "Pattern retrieval too slow");
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_plugin_integration() {
    let config = KnowledgePluginSystemConfig::default();
    let mut plugin_system = KnowledgePluginSystem::new(config);
    
    // Test plugin installation and activation
    // Test knowledge integration
    // Test context-aware retrieval
}
```

## Deployment and Distribution

### Plugin Manifest

Create a `plugin.toml` file:

```toml
[plugin]
id = "my-plugin-1.0.0"
name = "My Knowledge Plugin"
version = "1.0.0"
author = "Your Name"
description = "Description of your plugin"
api_version = "1.0.0"

[plugin.type]
type = "AntiPattern"
supported_languages = ["Rust", "Python"]

[plugin.capabilities]
provides_patterns = true
provides_detectors = true
provides_solutions = true

[plugin.permissions]
allow_network = false
allow_filesystem_read = true
max_memory_mb = 50
max_execution_time_ms = 2000
```

### Distribution

1. **Package your plugin**: Create a distributable package
2. **Documentation**: Include comprehensive documentation
3. **Examples**: Provide usage examples
4. **Tests**: Include test suite
5. **License**: Specify license terms

## Best Practices

### Code Quality

1. **Follow Rust conventions**: Use `rustfmt` and `clippy`
2. **Error handling**: Use proper error types and handling
3. **Documentation**: Document all public APIs
4. **Testing**: Achieve high test coverage
5. **Performance**: Profile and optimize critical paths

### Knowledge Quality

1. **Accurate patterns**: Ensure pattern definitions are accurate
2. **Clear descriptions**: Write clear, actionable descriptions
3. **Practical solutions**: Provide implementable solutions
4. **Evidence-based**: Base patterns on real-world evidence
5. **Regular updates**: Keep knowledge current

### Compatibility

1. **API versioning**: Use semantic versioning
2. **Backward compatibility**: Maintain compatibility when possible
3. **Migration guides**: Provide migration documentation
4. **Deprecation notices**: Give advance notice of changes

## Examples

### Complete Plugin Example

See `examples/knowledge_plugin_example.rs` for a complete working example that demonstrates:

- Plugin system initialization
- Multiple plugin types
- Knowledge integration
- Context-aware retrieval
- Performance monitoring

### Plugin Templates

The repository includes templates for different plugin types:

- `templates/antipattern_plugin.rs`: Anti-pattern plugin template
- `templates/enterprise_plugin.rs`: Enterprise plugin template
- `templates/framework_plugin.rs`: Framework plugin template
- `templates/language_plugin.rs`: Language plugin template

## Support and Community

### Getting Help

1. **Documentation**: Check the comprehensive documentation
2. **Examples**: Review the provided examples
3. **Tests**: Look at the test suite for usage patterns
4. **Issues**: Report issues on the GitHub repository
5. **Discussions**: Join community discussions

### Contributing

1. **Plugin registry**: Submit your plugin to the community registry
2. **Core improvements**: Contribute to the plugin system itself
3. **Documentation**: Help improve documentation
4. **Examples**: Share examples and tutorials

## Conclusion

The Uveddi Knowledge Plugin System provides a powerful, secure, and performant way to extend the AI Knowledge Library. By following this guide and best practices, you can create high-quality plugins that enhance the analysis capabilities for your specific domain or organization.

For more information, see:
- [API Documentation](../api/rust-documentation.md)
- [Architecture Overview](../04-architecture/overview.md)
- [Security Model](../04-architecture/security-model.md)
- [Performance Guidelines](../04-architecture/performance.md)