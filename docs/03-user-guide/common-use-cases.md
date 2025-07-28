# Common Use Cases (Alpha Release)

⚠️ **Alpha Status**: These examples show the CLI interface structure. Analysis engine is in development and will show "execution failed" errors.

## Basic Codebase Analysis

```bash
# Alpha: CLI works, analysis shows expected errors
./target/release/uveddi analyze ./project --output-format=json --output=analysis.json
```

Available CLI flags (ready for when analysis engine is complete):
- `--output-format`: Format (json, markdown, text)
- `--output`: Output file path
- `--dead-code-confidence`: Confidence threshold (0.0-1.0)
- `--enable-ai`: Enable AI analysis (when engine ready)

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
