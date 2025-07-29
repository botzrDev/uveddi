# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- New documentation structure
- mdBook configuration for documentation website
- Architecture documentation consolidation

### Changed
- Updated README and CONTRIBUTING files
- Reorganized documentation directories

## [0.9.0-alpha] - 2025-07-29

### Added
- Alpha release with core functionality
- CLI analysis tools with basic anti-pattern detection
- TUI interface for interactive exploration
- Local AI integration via Ollama
- Tree-sitter based code parsing
- Basic security framework with OAuth2 support
- Initial knowledge library with universal patterns

### Known Limitations (Alpha)
- Enhanced prompt templates temporarily disabled
- OIDC authentication temporarily disabled pending API updates  
- Some advanced AI features require further development
- Performance optimizations in progress

### Security
- Fixed RSA timing attack vulnerability (RUSTSEC-2023-0071)
- Removed hardcoded secrets and SSH keys
- Implemented secure JWT configuration
- Updated vulnerable dependencies

## [1.0.0] - TBD (Future Release)

### Added
- Initial release of Uveddi CLI
- Core analysis engine with Rust, Python, JavaScript support
- AI integration with Ollama and commercial providers
- Plugin system using WASM

### Fixed
- Various stability improvements
- Performance optimizations for large codebases
