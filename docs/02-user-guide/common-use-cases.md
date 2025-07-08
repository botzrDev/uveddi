# Common Use Cases

## Basic Codebase Analysis

```bash
uveddi analyze ./project --output analysis.json
```

Key flags:
- `--focus`: Limit to specific issue types (cyclic_dependencies, god_objects, etc.)
- `--format`: Output format (json, markdown, html)
- `--fail-on`: Exit with error if issues exceed threshold

## CI/CD Integration

Example GitHub Actions workflow:
```yaml
name: Architectural Analysis
on: [push, pull_request]

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: botzrdev/uveddi-action@v1
        with:
          path: ./src
          fail-on: critical
          format: markdown
```

## Custom Rule Configuration

Create `.uveddi/config.toml`:
```toml
[rules.cyclomatic_complexity]
enabled = true
threshold = 15

[rules.dependency_distance]
enabled = true
max_distance = 3
```

## Plugin Development

Basic plugin structure:
```rust
use uveddi_plugin::*;

#[plugin]
pub struct MyCustomDetector;

impl Detector for MyCustomDetector {
    fn analyze(&self, ctx: &AnalysisContext) -> Vec<Issue> {
        // Custom analysis logic
        vec![]
    }
}
```

## Advanced Analysis

Combine with other tools:
```bash
# Run clippy and Uveddi together
cargo clippy && uveddi analyze . --output combined_report.md
```

## IDE Integration

VS Code settings:
```json
{
  "uveddi.enable": true,
  "uveddi.autoAnalyze": true,
  "uveddi.analysisOnSave": true
}
