# Copilot Instructions for Uveddi

<!-- Use this file to provide workspace-specific custom instructions to Copilot. For more details, visit https://code.visualstudio.com/docs/copilot/copilot-customization#_use-a-githubcopilotinstructionsmd-file -->

This is a Rust project called Uveddi. When generating code:

## General Guidelines
- Follow Rust best practices and idioms
- Use proper error handling with `Result<T, E>` types
- Prefer using standard library types when possible
- Write clear, self-documenting code with appropriate comments
- Use `cargo fmt` formatting style
- Follow Rust naming conventions (snake_case for functions/variables, PascalCase for types)

## Dependencies
- Prefer well-maintained crates from crates.io
- Use semantic versioning in Cargo.toml
- Add dev-dependencies for testing utilities when needed

## Testing
- Write unit tests using the built-in `#[cfg(test)]` module pattern
- Use descriptive test names that explain what is being tested
- Consider integration tests in the `tests/` directory for larger features

## Documentation
- Use `///` for public API documentation
- Include examples in documentation where helpful
- Keep README.md updated with build and usage instructions
