# Feature Flags Guide

## Overview

Uveddi uses feature flags to enable modular compilation and reduce binary size. This allows you to include only the functionality you need, resulting in faster compilation times and smaller binaries.

## Available Features

### Core Features

#### `default`
- **Includes**: `["local-ai", "image-rendering", "tree-sitter"]`
- **Purpose**: Complete feature set for most use cases
- **Binary Size**: ~15MB
- **Build Time**: ~2 minutes

```toml
[dependencies]
uveddi = "0.1"
# or explicitly
uveddi = { version = "0.1", features = ["default"] }
```

#### `analysis`
- **Includes**: Core analysis functionality without tree-sitter
- **Purpose**: Lightweight analysis without AST parsing
- **Binary Size**: ~5MB
- **Build Time**: ~30 seconds

```toml
[dependencies]
uveddi = { version = "0.1", features = ["analysis"] }
```

#### `tree-sitter`
- **Includes**: AST parsing for Rust, Python, JavaScript, TypeScript
- **Dependencies**: `tree-sitter`, `tree-sitter-rust`, `tree-sitter-python`, `tree-sitter-javascript`, `tree-sitter-typescript`
- **Purpose**: Multi-language AST parsing and analysis
- **Binary Size**: Adds ~3MB
- **Build Time**: Adds ~30 seconds

```toml
[dependencies]
uveddi = { version = "0.1", features = ["tree-sitter"] }
```

### AI Integration Features

#### `ai`
- **Includes**: Base AI functionality
- **Dependencies**: `reqwest`, `async-trait`
- **Purpose**: Foundation for AI-powered analysis
- **Binary Size**: Adds ~2MB
- **Build Time**: Adds ~15 seconds

```toml
[dependencies]
uveddi = { version = "0.1", features = ["ai"] }
```

#### `local-ai`
- **Includes**: `ai` + Ollama integration
- **Dependencies**: All `ai` dependencies
- **Purpose**: Local AI analysis using Ollama
- **Binary Size**: Adds ~2MB
- **Build Time**: Adds ~15 seconds

```toml
[dependencies]
uveddi = { version = "0.1", features = ["local-ai"] }
```

### Specialized Features

#### `image-rendering`
- **Includes**: Diagram generation service
- **Dependencies**: `reqwest`, `base64`
- **Purpose**: Generate visual diagrams and charts
- **Binary Size**: Adds ~1MB
- **Build Time**: Adds ~10 seconds

```toml
[dependencies]
uveddi = { version = "0.1", features = ["image-rendering"] }
```

#### `wasm-plugins`
- **Includes**: WebAssembly plugin system
- **Dependencies**: `wasmtime`, `wasmtime-wasi`, `wit-bindgen`
- **Purpose**: Load and execute WebAssembly plugins
- **Binary Size**: Adds ~4MB
- **Build Time**: Adds ~45 seconds

```toml
[dependencies]
uveddi = { version = "0.1", features = ["wasm-plugins"] }
```

#### `tui`
- **Includes**: Terminal user interface
- **Dependencies**: `ratatui`, `crossterm`, `tui-input`
- **Purpose**: Interactive terminal interface
- **Binary Size**: Adds ~1MB
- **Build Time**: Adds ~20 seconds

```toml
[dependencies]
uveddi = { version = "0.1", features = ["tui"] }
```

## Feature Combinations

### Common Use Cases

| Use Case | Features | Binary Size | Build Time | Description |
|----------|----------|-------------|------------|-------------|
| **Minimal Analysis** | `["analysis"]` | ~5MB | ~30s | Basic analysis without AST parsing |
| **Full Local Setup** | `["default"]` | ~15MB | ~2m | Complete feature set with AI and rendering |
| **AST Analysis Only** | `["tree-sitter"]` | ~8MB | ~1m | Multi-language AST parsing and analysis |
| **AI-Powered Analysis** | `["tree-sitter", "local-ai"]` | ~10MB | ~1.5m | AST analysis with AI explanations |
| **Plugin Development** | `["wasm-plugins", "tree-sitter"]` | ~12MB | ~1.5m | Develop and test WebAssembly plugins |
| **TUI Only** | `["tui", "tree-sitter"]` | ~9MB | ~1m | Terminal interface for interactive analysis |
| **Server Deployment** | `["analysis", "image-rendering"]` | ~6MB | ~40s | Headless analysis with diagram generation |

### Example Configurations

#### Development Environment
```toml
[dependencies]
uveddi = { version = "0.1", features = ["default"] }
```
- Complete development experience
- All features available for testing
- AI explanations and visual diagrams

#### CI/CD Pipeline
```toml
[dependencies]
uveddi = { version = "0.1", features = ["analysis", "tree-sitter"] }
```
- Fast compilation for automated testing
- Full analysis capabilities
- No AI dependencies (reduces external service requirements)

#### Production Server
```toml
[dependencies]
uveddi = { version = "0.1", features = ["analysis", "image-rendering"] }
```
- Minimal binary size for deployment
- Core analysis functionality
- Diagram generation for reports

#### Plugin Development
```toml
[dependencies]
uveddi = { version = "0.1", features = ["wasm-plugins", "tree-sitter"] }
```
- WebAssembly plugin system
- AST parsing for plugin development
- No AI or rendering dependencies

## Performance Impact

### Compile Time Impact

| Feature | Additional Build Time | Reason |
|---------|----------------------|--------|
| `tree-sitter` | +30s | Language parser compilation |
| `wasm-plugins` | +45s | Wasmtime runtime compilation |
| `local-ai` | +15s | HTTP client dependencies |
| `image-rendering` | +10s | Base64 encoding dependencies |
| `tui` | +20s | Terminal rendering libraries |

### Runtime Performance

#### Memory Usage
- **`tree-sitter`**: Higher memory usage due to AST caching
- **`wasm-plugins`**: Moderate overhead for sandboxed execution
- **`local-ai`**: Network request overhead
- **`image-rendering`**: Temporary memory for image processing

#### CPU Usage
- **`tree-sitter`**: CPU-intensive AST parsing
- **`wasm-plugins`**: Moderate CPU overhead for WebAssembly execution
- **`local-ai`**: Network I/O bound (depends on Ollama response time)
- **`image-rendering`**: Network I/O bound (depends on rendering service)

## Feature Dependencies

### Dependency Tree
```
default
├── local-ai
│   └── ai
│       ├── reqwest
│       └── async-trait
├── image-rendering
│   ├── reqwest
│   └── base64
└── tree-sitter
    ├── tree-sitter
    ├── tree-sitter-rust
    ├── tree-sitter-python
    ├── tree-sitter-javascript
    └── tree-sitter-typescript

wasm-plugins
├── wasmtime
├── wasmtime-wasi
└── wit-bindgen

tui
├── ratatui
├── crossterm
└── tui-input
```

### Shared Dependencies
- **`reqwest`**: Used by both `ai` and `image-rendering`
- **`tokio`**: Always included (core async runtime)
- **`serde`**: Always included (serialization)
- **`anyhow`**: Always included (error handling)

## Best Practices

### 1. Choose Minimal Feature Set
Start with the smallest feature set that meets your needs:

```toml
# Start minimal
uveddi = { version = "0.1", features = ["analysis"] }

# Add features as needed
uveddi = { version = "0.1", features = ["analysis", "tree-sitter"] }
```

### 2. Development vs Production
Use different feature sets for development and production:

```toml
# Development
[dependencies]
uveddi = { version = "0.1", features = ["default"] }

# Production
[dependencies]
uveddi = { version = "0.1", features = ["analysis", "tree-sitter"] }
```

### 3. CI/CD Optimization
Optimize for build speed in CI/CD:

```toml
# Fast CI builds
[dependencies]
uveddi = { version = "0.1", features = ["analysis"] }

# Full testing
[dependencies]
uveddi = { version = "0.1", features = ["default"] }
```

## Feature-Specific Configuration

### Tree-sitter Configuration
```rust
use uveddi::analysis::AnalysisEngine;
use uveddi::ast::tree_sitter::SourceLanguage;

let engine = AnalysisEngine::builder()
    .with_supported_languages(vec![
        SourceLanguage::Rust,
        SourceLanguage::Python,
    ])
    .build()?;
```

### AI Configuration
```rust
use uveddi::ai::ollama_provider::OllamaProvider;
use uveddi::ai::engine::AiEngine;

let provider = OllamaProvider::new("http://localhost:11434")?;
let ai_engine = AiEngine::new(Box::new(provider))?;
```

### Plugin Configuration
```rust
use uveddi::plugins::engine::PluginEngine;
use uveddi::plugins::types::PluginConfig;

let plugin_engine = PluginEngine::new()?;
let config = PluginConfig {
    plugin_path: "path/to/plugin.wasm".into(),
    memory_limit: 64 * 1024 * 1024, // 64MB
    timeout: std::time::Duration::from_secs(30),
    ..Default::default()
};
```

## Troubleshooting

### Common Issues

#### 1. Missing Feature Compilation Error
```
error: failed to resolve: could not find `tree_sitter` in `uveddi::ast`
```
**Solution**: Enable the `tree-sitter` feature:
```toml
uveddi = { version = "0.1", features = ["tree-sitter"] }
```

#### 2. Ollama Connection Error
```
Error: Network error: Connection refused
```
**Solution**: Ensure Ollama is running and the `local-ai` feature is enabled:
```bash
ollama serve
```

#### 3. Plugin Loading Error
```
Error: Plugin error: WebAssembly runtime not available
```
**Solution**: Enable the `wasm-plugins` feature:
```toml
uveddi = { version = "0.1", features = ["wasm-plugins"] }
```

#### 4. Build Time Issues
If builds are taking too long, consider reducing features:
```toml
# Instead of default
uveddi = { version = "0.1", features = ["analysis", "tree-sitter"] }
```

### Feature Verification

You can verify which features are enabled at runtime:

```rust
use uveddi::config::FeatureFlags;

let flags = FeatureFlags::current();
println!("Tree-sitter enabled: {}", flags.tree_sitter);
println!("AI enabled: {}", flags.ai);
println!("TUI enabled: {}", flags.tui);
```

## Future Features

### Planned Features
- **`cloud-ai`**: Cloud-based AI providers (OpenAI, Anthropic)
- **`database-postgres`**: PostgreSQL database backend
- **`metrics`**: Advanced metrics collection and reporting
- **`web-ui`**: Web-based user interface
- **`distributed`**: Distributed analysis across multiple machines

### Contributing New Features
When adding new features:
1. Add feature flag to `Cargo.toml`
2. Use conditional compilation: `#[cfg(feature = "feature-name")]`
3. Update this documentation
4. Add feature-specific tests
5. Update CI/CD configurations

## Migration Guide

### Upgrading from v0.1.x
No breaking changes in feature flags. New features are additive.

### Upgrading Dependencies
Feature flags are designed to be stable. Dependency updates should not require changes to feature configuration.

---

For more information about configuring and using these features, see the [Configuration Guide](configuration-options.md) and [API Reference](../03-api-reference/).