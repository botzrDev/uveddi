# Uveddi Installation Guide

## Prerequisites

- Rust toolchain (latest stable version)
- Git
- PostgreSQL (optional, for database features)
- Ollama (optional, for local AI analysis)

## Installation Methods

### From Source

1. Clone the repository:
   ```bash
   git clone https://github.com/botzrDev/uveddi.git
   cd uveddi
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Install the CLI:
   ```bash
   cargo install --path .
   ```

### Using Cargo

```bash
cargo install uveddi
```

### Docker

```bash
docker pull ghcr.io/botzrdev/uveddi:latest
docker run -it ghcr.io/botzrdev/uveddi:latest
```

## Post-Installation

1. Verify installation:
   ```bash
   uveddi --version
   ```

2. Configure your environment:
   ```bash
   # Set up API keys if using commercial AI providers
   export OPENAI_API_KEY='your-key'
   export ANTHROPIC_API_KEY='your-key'
   ```

3. Run your first analysis:
   ```bash
   uveddi analyze ./your-project
   ```

## Troubleshooting

### Common Issues

- **Missing dependencies**: Ensure you have build essentials installed
  ```bash
  sudo apt-get install build-essential
  ```

- **Permission errors**: Use `sudo` for system-wide installation or adjust permissions

- **AI provider errors**: Verify API keys and network connectivity
