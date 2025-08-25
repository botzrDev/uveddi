# Contributing to Uveddi

We welcome contributions from the community! This guide will help you get started with development.

## Development Environment Setup

### Prerequisites
- Rust toolchain (latest stable version)
- Git
- At least 4GB RAM (8GB+ recommended for development)
- Optional: Ollama for local AI analysis

> **Performance Note**: Uveddi automatically enables memory optimization by default. During development, you can disable it with `--disable-memory-optimization` if needed for debugging.

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

## 🚀 Good First Issues

New to Uveddi? Start here! We've curated beginner-friendly tasks to help you get familiar with the codebase.

### Finding Your First Task
1. Browse issues labeled [`good-first-issue`](https://github.com/botzrDev/uveddi/labels/good-first-issue)
2. Check our [Good First Issues Guide](docs/09-community/GOOD_FIRST_ISSUES.md)
3. Review the [Contribution Difficulty Matrix](docs/09-community/CONTRIBUTION_MATRIX.md)

### Difficulty Levels
- 🟢 **Beginner**: 1-4 hours, minimal context needed
- 🟡 **Intermediate**: 4-8 hours, some project knowledge required  
- 🔴 **Advanced**: 8+ hours, deep understanding needed

### Getting Started Checklist
- [ ] Read this contributing guide completely
- [ ] Set up your development environment
- [ ] Run tests to ensure everything works: `cargo test`
- [ ] Pick a task labeled `good-first-issue`
- [ ] Comment on the issue to claim it
- [ ] Ask questions if anything is unclear

### Need Help?
- **Questions**: Comment on your chosen issue
- **Community**: Join our GitHub Discussions
- **Mentorship**: Request a mentor for guidance
- **Stuck?**: Don't hesitate to ask for help!

## Code Style

### Formatting
- We use rustfmt to enforce consistent style. Run `cargo fmt --all` before pushing.
- CI runs `cargo fmt --all -- --check` and will fail PRs that are not formatted.
- Install pre-commit hooks to auto-format and lint:
  - `pre-commit install`
  - Hooks include: cargo fmt, cargo clippy, cargo check.


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
