# Contributing to Uveddi

We welcome contributions from the community! This guide will help you get started with development.

## Development Environment Setup

### Prerequisites
- Rust toolchain (latest stable version)
- Git
- Optional: Ollama for local AI analysis

### Setup Steps
1. Clone the repository:
   ```bash
   git clone git@github.com:botzrDev/uveddi.git
   cd uveddi
   ```
2. Install dependencies:
   ```bash
   cargo build
   ```
3. Set up pre-commit hooks:
   ```bash
   pre-commit install
   ```

## Git Workflow

1. **Branching Strategy**:
   - `main`: Stable production branch
   - `develop`: Integration branch for features
   - Feature branches: `feature/description`
   - Bugfix branches: `fix/description`

2. **Commit Guidelines**:
   - Follow [Conventional Commits](https://www.conventionalcommits.org/)
   - Keep commits atomic and focused
   - Include tests with new features

3. **Pull Requests**:
   - Open PRs against the `develop` branch
   - Include a clear description of changes
   - Reference related issues
   - Ensure all tests pass

## Code Style

- Follow Rustfmt configuration (see rustfmt.toml)
- Clippy should report no warnings
- Document all public APIs with Rustdoc

## Testing

- Unit tests: `cargo test`
- Integration tests: `cargo test --test integration`
- Benchmarks: `cargo bench`

## Documentation

- Update relevant documentation when adding features
- Follow the [documentation structure](docs/README.md)
- Add examples for new functionality

## Issue Reporting

When reporting issues:
- Include steps to reproduce
- Provide expected vs actual behavior
- Share relevant environment details

## Code Review Process

All contributions require:
- At least one approving review
- Passing CI checks
- Documentation updates if needed

Thank you for contributing to Uveddi!
