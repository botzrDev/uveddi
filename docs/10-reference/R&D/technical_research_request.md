# Technical Research Request: Rust Visualization System Implementation

## Project Context

You are researching for **Uveddi**, a Rust-based code analysis and architectural visualization tool. We're implementing Phase 2 of an enhanced visualization system that generates Mermaid.js diagrams from code analysis results and renders them to high-quality images via a Node.js microservice.

### Current Architecture Overview

- **Primary Application**: Rust-based analysis engine using Cargo workspace
- **Template System**: Tera templates for Mermaid.js generation  
- **Rendering Service**: Node.js + Playwright microservice for image generation
- **Orchestration**: Docker Compose for development and deployment
- **Anti-Pattern Focus**: God Objects, Cyclic Dependencies, Dead Code, Large Classes, Tight Coupling

### Technology Stack

```toml
# Key dependencies from Cargo.toml
tera = "1.19.1"                    # Template engine
reqwest = "0.12.22"                # HTTP client for rendering service
uuid = "1.8.0"                     # Component identification
serde = "1.0.203"                  # Serialization
tokio = "1.37.0"                   # Async runtime
```

### Current Implementation Status

- ✅ Basic template system architecture designed
- ✅ Node.js rendering service architected
- ✅ Docker Compose configuration created
- 🔄 **IN PROGRESS**: Resolving build errors and type system integration
- ❌ **BLOCKED**: Missing production-ready patterns for several areas

## Research Request

Based on the attached architectural research and current implementation challenges, I need detailed technical guidance on the following areas. **Please provide practical, production-ready patterns with code examples, not theoretical approaches.**

### 1. Rust Template System Integration Patterns

**Context**: We're using Tera templates to generate Mermaid.js diagrams dynamically based on code analysis results. Templates must be loaded at runtime from configuration.

**Research Needed**:
- **Template compilation and caching strategies** - How do production Rust applications handle Tera template compilation? Should we precompile at startup or lazy-load?
- **Error handling patterns** - What are the most robust error handling patterns for template rendering failures? How should we handle malformed templates vs data serialization errors?
- **Dynamic template registration** - Best practices for registering templates at runtime vs compile-time bundling. Our `diagrams.toml` config should drive which templates are available.
- **Performance benchmarks** - Comparative analysis of filesystem-based vs embedded template loading in containerized environments.

**Current Code Pattern**:
```rust
// src/analysis/mermaid_generator.rs
pub struct MermaidGenerator {
    template_engine: Tera,
    diagram_specs: HashMap<DiagramType, DiagramSpec>,
}

impl MermaidGenerator {
    pub fn new() -> Result<Self, MermaidGenerationError> {
        let mut tera = Tera::new("templates/diagrams/*")?;
        Self::register_builtin_templates(&mut tera)?;
        // Need guidance on production patterns here
    }
}
```

### 2. Type System and Serialization Challenges

**Context**: Complex Rust enum types must serialize cleanly for template contexts, with proper error handling for malformed data.

**Research Needed**:
- **Serde serialization patterns** for complex enums like our `ComponentType` with multiple variants
- **JSON schema validation** for template context data to prevent runtime template errors
- **Type-safe template context building** - How to ensure template data is always valid?
- **UUID serialization handling** in template contexts (our components use UUID identifiers)

**Current Challenge**:
```rust
// src/models/visualization.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentType {
    Module,
    Service,
    RustStruct { fields: Vec<FieldInfo> },
    PythonClass { bases: Vec<String>, methods: Vec<MethodInfo> },
    // ... many more variants
}

// How to ensure this serializes predictably for templates?
```

### 3. HTTP Client Architecture for Rendering Service

**Context**: Rust application needs to communicate with Node.js rendering service for image generation. Must handle failures gracefully and support high concurrency.

**Research Needed**:
- **Reqwest best practices** for microservice communication patterns
- **Connection pooling strategies** for persistent connections to rendering services
- **Circuit breaker patterns** in Rust ecosystem for handling service failures
- **Async/await patterns** for non-blocking diagram generation in high-load scenarios
- **Retry logic and backoff strategies** for transient rendering failures

**Current Implementation**:
```rust
// src/report/image_renderer.rs
pub struct ImageRenderer {
    client: reqwest::Client,
    config: RenderingConfig,
}

// Need production-grade patterns for reliability
```

### 4. Docker Compose Orchestration Specifics

**Context**: Development and production environments need different orchestration patterns. Service discovery and health checks are critical.

**Research Needed**:
- **Health check implementation patterns** for Node.js services in Docker Compose
- **Volume mounting strategies** for sharing templates between Rust app and rendering service
- **Environment variable management** patterns for multi-service configurations
- **Development vs production** Docker Compose configuration best practices
- **Service dependency ordering** and startup coordination

**Current Configuration**:
```yaml
# docker-compose.yml snippet
services:
  uveddi-app:
    build: .
    environment:
      - RENDERING_SERVICE_URL=http://rendering-service:3001
  
  rendering-service:
    build: ./rendering-service
    ports:
      - "3001:3001"
```

### 5. Integration Testing Strategies

**Context**: Complex pipeline from code analysis → template rendering → HTTP service → image generation needs comprehensive testing.

**Research Needed**:
- **Testing template rendering** with comprehensive mock data in Rust
- **Integration testing patterns** for HTTP services using tokio-test
- **Docker Compose testing** in CI/CD pipelines (GitHub Actions)
- **End-to-end testing** of the complete visualization pipeline
- **Performance testing** for template rendering under load

### 6. Build System and Deployment Integration

**Context**: Cargo workspace with optional features, containerized deployment, CI/CD integration.

**Research Needed**:
- **Cargo feature flags** for conditional compilation of visualization components
- **Asset bundling strategies** for templates in containerized Rust applications  
- **Cross-compilation considerations** for Linux containers from development machines
- **CI/CD patterns** for Rust + Node.js microservice deployments

## Expected Deliverables

For each area, please provide:

1. **Code examples** showing production-ready patterns
2. **Comparative analysis** of different approaches with trade-offs
3. **Performance considerations** and benchmarking guidance
4. **Error handling patterns** specific to each area
5. **Testing strategies** for validation
6. **Common pitfalls** and how to avoid them

## Priority

**High Priority**: Areas 1, 2, 3 (blocking current development)
**Medium Priority**: Areas 4, 5 (needed for production readiness)
**Lower Priority**: Area 6 (optimization and deployment)

## Output Format

Please structure findings as:
- **Pattern Overview** (conceptual explanation)
- **Implementation Example** (working code)
- **Trade-offs Analysis** (when to use vs not use)
- **Testing Approach** (how to validate)
- **Production Considerations** (scaling, monitoring, debugging)

Focus on **actionable, implementable guidance** rather than theoretical discussion. The research should enable immediate implementation progress on the Uveddi visualization system.
