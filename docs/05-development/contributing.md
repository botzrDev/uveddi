# Contributing to Uveddi

## Getting Started

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone git@github.com:your-username/uveddi.git
   ```
3. Set up development environment:
   ```bash
   cd uveddi
   ./scripts/setup-dev-environment.sh
   ```

## Development Workflow

### Branch Naming
Use the format: `{type}/{short-description}`
- `feat/`: New features
- `fix/`: Bug fixes
- `docs/`: Documentation changes
- `refactor/`: Code refactoring

### Commit Messages
Follow [Conventional Commits](https://www.conventionalcommits.org/):
```
feat: add new analysis rule for god objects
fix(parser): handle edge case in Python imports
```

### Testing Requirements
- All changes must include tests
- Run tests with:
  ```bash
  cargo test --all-features
  ```

## Code Standards

### Rust Guidelines
- Follow Rust API Guidelines
- Use `rustfmt` and `clippy`
- Document all public APIs with examples

### Documentation
- Update relevant documentation for any changes
- Add examples for new features
- Keep CHANGELOG.md updated

## Pull Requests

1. Create a draft PR early for discussion
2. Request review when ready
3. Address all review comments
4. Ensure CI passes before merging

## Issue Reporting

When reporting issues:
- Include Uveddi version (`uveddi --version`)
- Describe expected vs actual behavior
- Provide reproduction steps
- Attach relevant logs (`RUST_LOG=debug` output)

## Community Guidelines

- Be respectful and inclusive
- Help others when possible
- Follow the [Code of Conduct](CODE_OF_CONDUCT.md)
