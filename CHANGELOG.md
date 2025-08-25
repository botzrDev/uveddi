# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Updated
- **Major Dependency Updates**: Updated all dependencies to latest compatible semantic versions
  - `axum`: 0.7.9 → 0.8.4 (HTTP framework with improved async performance)
  - `chrono`: 0.4.39 → 0.4.41 (datetime handling with security fixes)
  - `criterion`: 0.5.1 → 0.7.0 (benchmarking framework with new features)
  - `lru`: 0.12.0 → 0.16.0 (LRU cache with performance improvements)  
  - `rusqlite`: 0.31.0 → 0.37.0 (SQLite bindings with API improvements)
  - `sysinfo`: 0.30.13 → 0.37.0 (system information API with breaking changes)
  - `thiserror`: 1.0.69 → 2.0.16 (error handling with improved macros)
  - `toml`: 0.8.0 → 0.9.5 (TOML parsing with better error messages)
  - `tower`: 0.4.13 → 0.5.2 (service framework with enhanced middleware)
  - `tower-http`: 0.5.2 → 0.6.6 (HTTP middleware with new features)
  - All other dependencies updated to latest patch versions

### Fixed
- **API Breaking Changes**: Updated code to handle breaking changes in dependencies
  - Fixed `sysinfo` API changes for process monitoring (`refresh_process` → `refresh_processes`)
  - Updated memory monitoring to use new `ProcessesToUpdate` API
  - Fixed `rand` API changes for random number generation
  - Maintained backward compatibility where possible

### Security
- Updated dependencies include security patches and vulnerability fixes
- All security-related dependencies updated to latest versions with CVE fixes

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
