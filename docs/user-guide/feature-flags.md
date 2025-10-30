# Feature Flags Guide

> **Updated for v0.9.0:** Simplified feature system with profiles, language packs, and capability features. See [Migration Guide](../migrations/feature-migration-guide.md) if upgrading from older versions.

## Overview

Uveddi uses a modern feature flag system optimized for build performance and user clarity. The system consists of three layers:

1. **Profiles** - For build selection (`minimal`, `standard`, `full`)
2. **Language Packs** - For language support (`languages-core`, `languages-web`, `languages-all`)
3. **Capabilities** - For optional functionality (`security`, `wasm-plugins`, `memory-optimization`, etc.)

## Build Profiles

### `minimal`
- **Purpose**: Ultra-fast builds for rapid iteration
- **Includes**: Core storage, parallelism, async support
- **Dependencies**: `rusqlite`, `bincode`, `rayon`, `tokio-stream`, `async-stream`
- **Binary Size**: ~2-3MB
- **Build Time**: ~15-30 seconds
- **Use Case**: Quick prototyping, CI fast checks

```bash
# Fastest possible build
cargo build --features minimal
```

### `standard` (Recommended)
- **Purpose**: Balanced development with parsing capabilities
- **Includes**: `minimal` + all language parsing + monitoring
- **Dependencies**: Adds `tree-sitter` (all languages) + `prometheus`
- **Binary Size**: ~4-6MB
- **Build Time**: ~45-90 seconds
- **Use Case**: Daily development, most analysis tasks

```bash
# Recommended for development
cargo build --features standard
```

### `full`
- **Purpose**: Production-ready with all capabilities
- **Includes**: `standard` + security + optimization + UI + plugins + web
- **Dependencies**: Adds security stack, TUI, WASM runtime, web stack, memory optimization
- **Binary Size**: ~12-18MB
- **Build Time**: ~2-4 minutes
- **Use Case**: Production deployment, comprehensive analysis

```bash
# Production build
cargo build --release --features full
```

## Language Support

### Language Packs (Recommended)

#### `languages-core`
- **Includes**: `rust-lang` + `python-lang`
- **Use Case**: Backend/systems development
- **Note**: Requires tree-sitter base for parsing

```bash
# Backend development with parsing
cargo build --features "minimal,tree-sitter,languages-core"
```

#### `languages-web`
- **Includes**: `javascript-lang` + `typescript-lang`
- **Use Case**: Frontend development
- **Note**: Requires tree-sitter base for parsing

```bash
# Frontend development with parsing
cargo build --features "minimal,tree-sitter,languages-web"
```

#### `languages-all`
- **Includes**: All language features
- **Use Case**: Full-stack development
- **Note**: Automatically included in `standard` and `full` profiles

```bash
# All languages (explicit)
cargo build --features "minimal,tree-sitter,languages-all"

# All languages (via profile)
cargo build --features standard
```

### Individual Languages

For precise control, use individual language features:

- `rust-lang` - Rust AST parsing
- `python-lang` - Python AST parsing
- `javascript-lang` - JavaScript AST parsing
- `typescript-lang` - TypeScript AST parsing

```bash
# Custom language combination
cargo build --features "minimal,tree-sitter,rust-lang,typescript-lang"
```

## Capability Features

These features add specific functionality and can be combined with any profile:

### Core Capabilities

#### `security`
- **Purpose**: Advanced security features and authentication
- **Includes**: OAuth2, JWT, RBAC, audit logging, secure HTTP client
- **Use Case**: Enterprise deployments, security-sensitive environments

```bash
cargo build --features "standard,security"
```

#### `memory-optimization`
- **Purpose**: High-performance memory management
- **Includes**: mimalloc allocator, arena allocation, memory mapping, zero-copy serialization
- **Use Case**: Large codebase analysis, performance-critical environments

```bash
cargo build --features "standard,memory-optimization"
```

#### `wasm-plugins`
- **Purpose**: WebAssembly plugin system
- **Includes**: Wasmtime runtime, WASI support, secure plugin execution
- **Use Case**: Extensible analysis, custom detectors

```bash
cargo build --features "standard,wasm-plugins"
```

### User Interface

#### `tui`
- **Purpose**: Terminal user interface
- **Includes**: Interactive TUI with forms, menus, and real-time display
- **Use Case**: Interactive analysis, development workflow integration

```bash
cargo build --features "standard,tui"
```

#### `web`
- **Purpose**: Web dashboard and HTTP API
- **Includes**: Rate limiting, WebSocket support, web server components
- **Use Case**: Team collaboration, CI/CD integration, web-based analysis

```bash
cargo build --features "standard,web"
```

### AI and Analysis

#### `ai`
- **Purpose**: Base AI functionality
- **Use Case**: Foundation for AI-powered analysis

#### `local-ai`
- **Purpose**: Local LLM integration (Ollama)
- **Includes**: `ai` + local inference capabilities
- **Use Case**: AI-enhanced analysis without external dependencies

```bash
cargo build --features "standard,local-ai"
```

#### `image-rendering`
- **Purpose**: Diagram and visualization generation
- **Use Case**: Rich report generation with charts and diagrams

```bash
cargo build --features "standard,image-rendering"
```

### Monitoring and Metrics

#### `prometheus`
- **Purpose**: Metrics collection and export
- **Included in**: `standard` and `full` profiles
- **Use Case**: Production monitoring, performance tracking

#### `chaos`
- **Purpose**: Chaos engineering and load testing
- **Use Case**: Stress testing, reliability validation

#### `regression-detection`
- **Purpose**: Performance regression analysis
- **Use Case**: CI/CD performance validation

## Common Usage Patterns

### Development Workflows

```bash
# Quick iteration (no parsing)
cargo build --features minimal
cargo run --features minimal -- analyze ./src

# Standard development
cargo build --features standard
cargo run --features standard -- analyze ./src --output-format json

# Full local testing
cargo build --features full
cargo run --features full -- analyze ./src --output-format html
```

### Language-Specific Projects

```bash
# Rust-only microservice
cargo run --features "minimal,rust-lang" -- analyze ./src

# Python data pipeline
cargo run --features "minimal,languages-core" -- analyze ./src

# React application
cargo run --features "minimal,languages-web" -- analyze ./src

# Full-stack application
cargo run --features standard -- analyze ./src
```

### Production Deployments

```bash
# Secure production build
cargo build --release --features "full"

# High-performance production
cargo build --release --features "full,memory-optimization"

# Production without security (if needed)
cargo build --release --features "standard,tui,wasm-plugins,web"
```

### CI/CD Integration

```bash
# Fast CI checks
cargo test --features minimal

# Standard CI validation
cargo test --features standard

# Comprehensive CI testing
cargo test --features "full,chaos"
```

## Performance Comparison

| Profile | Build Time | Binary Size | Capabilities |
|---------|------------|-------------|--------------|
| `minimal` | ~15-30s | ~2-3MB | Core analysis only |
| `standard` | ~45-90s | ~4-6MB | + All language parsing + metrics |
| `full` | ~2-4min | ~12-18MB | + Security + optimization + UI + plugins |

## Migration from v0.8.x

| Old Feature | New Feature | Notes |
|-------------|-------------|-------|
| `dev-minimal` | `minimal` | Direct replacement |
| `dev-core` | `standard` | Enhanced with language packs |
| `dev-full` | `standard` | Same functionality |
| `production` | `full` | All capabilities included |
| `web-full` | `web` | Simplified web features |

See the complete [Migration Guide](../migrations/feature-migration-guide.md) for detailed instructions.

## Best Practices

### For Development
1. **Start with `standard`** for most development work
2. **Use `minimal`** for quick iteration and testing
3. **Combine profiles with capabilities** as needed: `"standard,tui"`

### For Production
1. **Use `full`** for comprehensive deployments
2. **Add `memory-optimization`** for large codebases
3. **Include `security`** for enterprise environments

### For CI/CD
1. **Use `minimal`** for fast checks (linting, basic tests)
2. **Use `standard`** for comprehensive testing
3. **Use `full`** for security audits and release validation

### For Custom Builds
1. **Start with a profile** (`minimal`, `standard`, or `full`)
2. **Add specific capabilities** as needed
3. **Use language packs** instead of individual languages when possible

## Troubleshooting

### Build Issues
```bash
# Clean rebuild with specific features
cargo clean
cargo build --features standard

# Check feature resolution
cargo tree --features standard
```

### Feature Conflicts
```bash
# List active features
cargo metadata --features full | jq '.resolve.nodes[0].features'

# Test feature combinations
cargo check --features "minimal,security,tui"
```

### Performance Issues
```bash
# Profile-specific optimizations
cargo build --features "standard,memory-optimization" --profile release

# Build time optimization
cargo build --features minimal --profile dev-fast
```

For more help, see [Troubleshooting Guide](../troubleshooting/) or [report an issue](https://github.com/botzrDev/uveddi/issues).