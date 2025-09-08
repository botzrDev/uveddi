# Dependency Injection User Guide

## Introduction

Uveddi's analysis engine now supports **dependency injection**, allowing you to customize which detectors are used and how they're configured. This guide shows you how to use these features in your projects.

## Quick Start

### Default Usage (No Changes Required)

Your existing code continues to work exactly as before:

```rust
use uveddi::analysis::AnalysisEngine;

let engine = AnalysisEngine::new()?;
let (issues, graph) = engine.analyze(Path::new("src/")).await?;
```

### Using Configuration Files

Create a configuration file to customize detector behavior:

**analysis.toml**:
```toml
# Cache settings
cache_size = 1000
cache_path = "uveddi_cache.db"
enable_plugins = true

# Detector configurations
[detectors.god_object]
threshold_methods = 5
threshold_fields = 8

[detectors.code_duplication]
# Uses default settings

[detectors.dead_code]
# Uses default settings
```

**Your code**:
```rust
use uveddi::analysis::{AnalysisConfig, AnalysisEngineBuilder};

// Load configuration and create engine
let engine = AnalysisEngineBuilder::new()
    .from_config_file(Path::new("analysis.toml"))?
    .build().await?;

let (issues, graph) = engine.analyze(Path::new("src/")).await?;
```

## Configuration Options

### Detector Configuration

Each detector can be configured with custom parameters:

```toml
[detectors.god_object]
threshold_methods = 10    # Maximum methods before flagging
threshold_fields = 15     # Maximum fields before flagging

[detectors.code_duplication]
# No configuration options currently

[detectors.dead_code]
# No configuration options currently

[detectors.large_classes]
# No configuration options currently

[detectors.tight_coupling]
# No configuration options currently
```

### Cache Configuration

Control how analysis results are cached:

```toml
# File-based cache
cache_size = 1000
cache_path = "custom_cache.db"

# Or use in-memory cache (for testing)
# Omit cache_path or set to null
cache_size = 500
```

### Plugin Configuration

Enable or disable WASM plugin support:

```toml
enable_plugins = true   # Enable WASM plugins
# enable_plugins = false  # Disable plugins (default)
```

## Builder Pattern Usage

The builder pattern provides a fluent interface for configuring engines:

### Basic Builder Usage

```rust
use uveddi::analysis::AnalysisEngineBuilder;

let engine = AnalysisEngineBuilder::new()
    .enable_plugins()
    .with_memory_cache()
    .build().await?;
```

### Custom Detectors

Add your own detector implementations:

```rust
use uveddi::analysis::{AnalysisEngineBuilder, GodObjectDetector};

let engine = AnalysisEngineBuilder::new()
    .add_detector(Box::new(GodObjectDetector::new(15, 20))) // Custom thresholds
    .add_detector(Box::new(MyCustomDetector::new()))
    .build().await?;
```

### Configuration Override

Load configuration but override specific settings:

```rust
let engine = AnalysisEngineBuilder::new()
    .from_config_file(Path::new("production.toml"))?
    .with_memory_cache() // Override cache setting for testing
    .build().await?;
```

## Environment-Specific Configurations

Create different configuration files for different environments:

### Development Configuration

**dev.toml**:
```toml
cache_size = 100
enable_plugins = false

[detectors.god_object]
threshold_methods = 15    # More relaxed for development
threshold_fields = 20
```

### Production Configuration

**production.toml**:
```toml
cache_size = 5000
cache_path = "/var/cache/uveddi/cache.db"
enable_plugins = true

[detectors.god_object]
threshold_methods = 5     # Strict for production
threshold_fields = 8
```

### Usage

```rust
let config_file = if cfg!(debug_assertions) {
    "dev.toml"
} else {
    "production.toml"
};

let engine = AnalysisEngineBuilder::new()
    .from_config_file(Path::new(config_file))?
    .build().await?;
```

## Plugin Integration

### Automatic Plugin Loading

When plugins are enabled, they're automatically loaded:

```rust
let engine = AnalysisEngineBuilder::new()
    .enable_plugins()
    .build().await?; // Plugins loaded automatically
```

### Manual Plugin Management

For more control over plugin loading:

```rust
let mut engine = AnalysisEngine::new_with_plugins().await?;

// Load all available plugins
let plugin_count = engine.add_plugin_detectors().await?;
println!("Loaded {} plugin detectors", plugin_count);

// Later: reload plugins (e.g., after installing new ones)
let new_count = engine.reload_plugin_detectors().await?;
println!("Reloaded {} plugin detectors", new_count);

// Remove plugins if needed
let removed_count = engine.remove_plugin_detectors();
println!("Removed {} plugin detectors", removed_count);
```

## Advanced Usage

### Programmatic Configuration

Create configurations in code:

```rust
use uveddi::analysis::{AnalysisConfig, DetectorConfig};

let mut config = AnalysisConfig::default();

// Customize god object detector
config.add_detector("god_object".to_string(), 
    DetectorConfig::new()
        .with_param("threshold_methods", 10)
        .with_param("threshold_fields", 15));

// Enable plugins
config.enable_plugins = true;

let engine = config.create_engine().await?;
```

### Registry-Based Management

For complex detector management scenarios:

```rust
use uveddi::analysis::{DetectorRegistry, DetectorFactory};

let mut registry = DetectorRegistry::new();

// Load different detector sets
registry.load_defaults();
registry.load_from_config(&custom_configs)?;

// Add plugin detectors if available
if let Some(plugin_engine) = get_plugin_engine() {
    registry.load_plugin_detectors(plugin_engine).await?;
}

// Create engine with all detectors
let detectors = registry.get_all_detectors();
let engine = AnalysisEngine::with_detectors(detectors, None, false)?;
```

## Testing with Dependency Injection

### Mock Detectors for Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;

    mock! {
        TestDetector {}
        impl AnalysisDetector for TestDetector {
            fn detect_issues(&self, file: &ParsedFile) -> Result<Vec<ArchitecturalIssue>, UveddiError>;
            fn get_anti_pattern_types(&self) -> Vec<AntiPatternType>;
            fn get_detector_name(&self) -> &'static str;
        }
    }

    #[test]
    fn test_with_mock_detector() {
        let mut mock = MockTestDetector::new();
        mock.expect_detect_issues()
            .returning(|_| Ok(vec![]));
        mock.expect_get_detector_name()
            .returning(|| "test-detector");

        let detectors: Vec<Box<dyn AnalysisDetector + Send + Sync>> = vec![
            Box::new(mock)
        ];

        let engine = AnalysisEngine::with_detectors(detectors, None, false)?;
        // Test with controlled detector behavior
    }
}
```

### Isolated Test Configurations

```rust
#[test]
fn test_analysis_with_custom_config() {
    let engine = AnalysisEngineBuilder::new()
        .add_detector(Box::new(TestSpecificDetector::new()))
        .with_memory_cache() // Isolated cache
        .build().await?;

    // Test analysis with specific detector configuration
}
```

## Best Practices

### 1. Use Configuration Files for Production

- Keep detector settings in version-controlled configuration files
- Use environment-specific configurations
- Document configuration changes in your project

### 2. Use Builder Pattern for Complex Setup

- Builder pattern is ideal for programmatic configuration
- Chain methods for readable configuration
- Override configuration file settings when needed

### 3. Handle Plugin Failures Gracefully

```rust
match engine.add_plugin_detectors().await {
    Ok(count) => println!("Loaded {} plugins", count),
    Err(e) => {
        eprintln!("Plugin loading failed: {}", e);
        // Continue with built-in detectors
    }
}
```

### 4. Use Memory Cache for Testing

```rust
// In tests, always use memory cache
let engine = AnalysisEngineBuilder::new()
    .with_memory_cache()
    .build().await?;
```

## Migration from Legacy API

### Step 1: Assess Current Usage

If you're using:
```rust
let engine = AnalysisEngine::new()?;
```

**You don't need to change anything!** This continues to work.

### Step 2: Identify Customization Needs

If you need to:
- Change detector thresholds → Use configuration files
- Add custom detectors → Use builder pattern
- Support plugins → Enable plugins in configuration
- Test with mocks → Use dependency injection

### Step 3: Gradual Migration

Start with configuration files:
```rust
// Before
let engine = AnalysisEngine::new()?;

// After
let engine = AnalysisEngineBuilder::new()
    .from_config_file(Path::new("analysis.toml"))?
    .build().await?;
```

Then add custom features as needed.

## Troubleshooting

### Configuration Not Loading

- Check file path is correct
- Validate TOML syntax
- Ensure all detector names are recognized

### Plugin Issues

- Verify `enable_plugins = true` in configuration
- Check plugin directory exists and has valid plugins
- Review plugin loading logs

### Performance Issues

- Adjust cache size based on your project size
- Consider using file-based cache for large projects
- Monitor detector performance and adjust configurations

## Related Documentation

- [Configuration Reference](./configuration-options.md)
- [Plugin Development Guide](../05-development/plugin_development_guide.md)
- [Testing Strategy](../10-reference/testing_strategy.md)

---

This guide covers the most common use cases for dependency injection in Uveddi. For advanced scenarios or custom implementations, refer to the developer documentation.