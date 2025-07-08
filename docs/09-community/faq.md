# Frequently Asked Questions

## General Questions

### What is Uveddi?
Uveddi is an architectural analysis tool that helps identify anti-patterns and maintain design integrity in codebases.

### Which languages does Uveddi support?
Currently supports Rust, Python, and JavaScript with more languages coming soon.

## Installation

### How do I install Uveddi?
```bash
cargo install uveddi
```

### What are the system requirements?
- Rust 1.70+
- 4GB+ RAM
- 2GB disk space

## Usage

### How do I analyze my project?
```bash
uveddi analyze ./path/to/project
```

### Can I customize the analysis?
Yes, create a `.uveddi/config.toml` file or use command-line flags.

## Troubleshooting

### Analysis fails with "Missing dependencies"
Run:
```bash
cargo build --features full
```

### Getting "Timeout" errors
Try increasing the timeout:
```bash
uveddi analyze ./project --timeout 600
```

## AI Integration

### Which AI providers are supported?
- OpenAI (GPT-4)
- Anthropic (Claude 3)
- Ollama (local models)

### How do I switch providers?
```bash
uveddi analyze ./project --ai-provider anthropic
```

## Advanced Topics

### Can I create custom detectors?
Yes! Uveddi supports WASM plugins for custom analysis.

### How do I contribute?
See our [Contributing Guide](development/contributing.md).
