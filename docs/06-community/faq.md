# Frequently Asked Questions

## General Questions

### What is Uveddi?
Uveddi is an architectural analysis tool that helps identify anti-patterns and maintain design integrity in codebases.

### Which languages does Uveddi support?
Currently supports Rust, Python, and JavaScript. Additional language support is planned for future releases.

## Installation

### How do I install Uveddi?
**Alpha Release**: Currently requires building from source:
```bash
git clone https://github.com/botzrDev/uveddi.git
cd uveddi
cargo build --release --features="alpha"
```
Please see the [Installation Guide](../../docs/01-getting-started/installation.md) for complete instructions.

### What are the system requirements?
- Rust toolchain (latest stable version)
- 4GB+ RAM
- 2GB disk space

## Usage

### How do I analyze my project?
```bash
uveddi analyze ./path/to/project
```

### Can I customize the analysis?
Yes, create a `.uveddi/config.toml` file or use command-line flags. See the [Configuration Guide](../../docs/01-getting-started/configuration.md) for options.

## Troubleshooting

### Analysis fails or hangs
Ensure you have sufficient RAM, especially when AI analysis is enabled. Try running analysis on a smaller subdirectory first.

### Getting "Timeout" errors
You can try increasing the timeout for AI-related operations if you are using them.

## AI Integration

### Does Uveddi use AI?
Yes, Uveddi can use a local AI model via Ollama for deeper analysis and suggestions. This is an optional feature.

### How do I set up the local AI?
You need to have [Ollama](https://ollama.com/) installed and a model downloaded, for example:
```bash
ollama pull deepseek-coder:6.7b-instruct
```

## Contributing

### How do I contribute?
We'd love your help! Please see our [Contributing Guide](./CONTRIBUTING.md).

## Community Edition

### Is this the community edition?
Yes, this documentation and repository are for the UVEDDI Community Edition.

### What features are included?
- Local analysis engine
- Community plugin support
- Open API

### Are enterprise features available?
No, enterprise features are not included in the community edition.

### Where can I get help?
See [support.md](./support.md) for community support options.