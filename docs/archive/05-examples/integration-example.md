# Integration Example: Uveddi with CI/CD Pipeline

## Overview
This example shows how to integrate Uveddi into a GitHub Actions workflow to enforce architectural standards.

## Basic Setup

1. Create `.github/workflows/architecture-analysis.yml`:
```yaml
name: Architectural Analysis
on: [push, pull_request]

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: stable
          override: true
      - run: |
          git clone https://github.com/botzrDev/uveddi.git
          cd uveddi
          cargo build --release --features="alpha"
      - run: uveddi analyze ./src --format markdown --output analysis.md
      - uses: actions/upload-artifact@v3
        if: always()
        with:
          name: architectural-analysis
          path: analysis.md
```

## Advanced Integration

### Fail on Critical Issues
```yaml
- run: uveddi analyze ./src --fail-on critical --format json
```

### Custom Rules
```yaml
- run: |
    uveddi analyze ./src \
      --config .uveddi/ci-rules.toml \
      --output analysis.json
```

### Slack Notifications
```yaml
- uses: rtCamp/action-slack-notify@v2
  if: failure()
  env:
    SLACK_WEBHOOK: ${{ secrets.SLACK_WEBHOOK }}
    SLACK_MESSAGE: "Architectural issues detected in ${{ github.repository }}"
    SLACK_COLOR: "danger"
```

## Example Config

`.uveddi/ci-rules.toml`:
```toml
[rules.cyclic_dependencies]
severity = "critical"

[rules.god_objects]
max_methods = 10
severity = "warning"

[analysis]
exclude = ["**/test/**", "**/examples/**"]
```

## Best Practices

1. Run analysis on both push and PR events
2. Cache Uveddi installation between runs
3. Store analysis reports as artifacts
4. Set appropriate failure thresholds
5. Include architecture review in PR checklist
