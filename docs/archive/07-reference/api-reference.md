# API Reference

This section will document all public APIs exposed by Uveddi, including CLI, Rust library, and (future) HTTP endpoints.

## CLI API

See [CLI Commands](./cli-commands.md) for a full list of commands and options.

## Rust Library API

Uveddi exposes a Rust API for advanced integrations and plugin development. See the crate-level documentation and examples below:

```rust
use uveddi::analysis::AnalysisEngine;

let engine = AnalysisEngine::new(config)?;
let results = engine.analyze_project("./src").await?;
for issue in results.anti_patterns {
    println!("Found {}: {}", issue.pattern_type, issue.description);
}
```

- For more details, see the [Rust API docs](https://docs.rs/uveddi).

## HTTP API (Planned)

- REST and GraphQL endpoints are planned for a future release.
- See the [Alpha Release Notes](../alpha-release-notes.md) for roadmap details.

---

> **Note:** Please open an issue if you need additional API documentation or have specific integration needs.
