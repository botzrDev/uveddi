# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2025-08-18 (v1.0 Community Core)

### Added
- **v1.0 Community Core** - Production-ready release with stable core functionality
- Memory optimization enabled by default for all analysis operations
- Automatic system memory detection and configuration
- Smart memory profile selection (small/default/large) based on system resources
- `--disable-memory-optimization` flag for advanced users who need to disable optimizations
- New documentation structure
- mdBook configuration for documentation website
- Architecture documentation consolidation

### Changed
- **BREAKING**: Memory optimization is now enabled by default instead of opt-in
- Replaced `--enable-memory-optimization` with `--disable-memory-optimization` flag
- Memory profiles now auto-detect based on system RAM (16GB+ → large, 8GB+ → default, <8GB → small)
- Memory limits automatically set based on available system memory
- Updated README and CONTRIBUTING files
- Reorganized documentation directories

### Performance
- Significant performance improvements for all users through default memory optimization
- Object pooling, arena allocation, and zero-copy AST caching now active by default
- Better memory management for large codebases without user configuration

## [0.9.0-alpha] - 2025-07-29 (Legacy Alpha Release)

### Added
- Alpha release with core functionality
- CLI analysis tools with basic anti-pattern detection
- TUI interface for interactive exploration
- Local AI integration via Ollama
- Tree-sitter based code parsing
- Basic security framework with OAuth2 support
- Initial knowledge library with universal patterns

### Known Limitations (v1.0 Community Core)
- Enhanced prompt templates temporarily disabled
- OIDC authentication temporarily disabled pending API updates  
- Some advanced AI features require further development
- Performance optimizations in progress

### Security
- Fixed RSA timing attack vulnerability (RUSTSEC-2023-0071)
- Removed hardcoded secrets and SSH keys
- Implemented secure JWT configuration
- Updated vulnerable dependencies

## [2.0.0] - TBD (Future Enterprise Release)

### Planned
- Enterprise features and advanced functionality
- Additional plugin system enhancements
- Extended AI provider integrations
