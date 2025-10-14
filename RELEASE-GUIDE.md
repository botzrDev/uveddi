# Uveddi Release Guide

**Quick Reference for Future Releases**

This guide provides the exact commands and workflow for releasing new versions of Uveddi, based on the successful 0.0.1 → 0.0.2 release process.

---

## Table of Contents

1. [Quick Release Checklist](#quick-release-checklist)
2. [Pre-Release Validation](#pre-release-validation)
3. [Release Commands](#release-commands)
4. [Post-Release Verification](#post-release-verification)
5. [Version Numbering Guide](#version-numbering-guide)
6. [Common Scenarios](#common-scenarios)
7. [Troubleshooting](#troubleshooting)

---

## Quick Release Checklist

Before starting a release, ensure:

- [ ] All code changes are complete and tested
- [ ] All detectors are working (run validation tests)
- [ ] Documentation is up-to-date
- [ ] `cargo build --release` completes successfully
- [ ] `cargo test` passes (or known failures are documented)
- [ ] Git working directory is ready (all changes committed or staged)
- [ ] You know which version number to use (MAJOR.MINOR.PATCH)

---

## Pre-Release Validation

### 1. Verify Current Version

```bash
# Check current version in Cargo.toml
grep '^version' Cargo.toml

# Check binary version (if built)
./target/release/uveddi -V
```

### 2. Check Git Status

```bash
# View current branch
git branch --show-current

# Check for uncommitted changes
git status

# View recent commits
git log --oneline -5
```

### 3. Run Tests

```bash
# Run all tests
cargo test

# Run specific detector tests
cargo test --lib

# Run integration tests
cargo test --test '*'
```

### 4. Build and Test Binary

```bash
# Clean build
cargo clean
cargo build --release

# Test version display
./target/release/uveddi -V
./target/release/uveddi --version

# Run a quick analysis to verify functionality
./target/release/uveddi analyze tests/ --output-format markdown
```

---

## Release Commands

### For PATCH Release (0.0.1 → 0.0.2)

Bug fixes, small improvements, documentation updates.

```bash
# 1. Update version in Cargo.toml
sed -i 's/^version = "0.0.1"/version = "0.0.2"/' Cargo.toml

# 2. Verify the change
grep '^version' Cargo.toml

# 3. Stage all changes
git add Cargo.toml [other-modified-files...]

# 4. Create commit
git commit -m "fix: [Brief description of changes]

[Detailed description of what was fixed/improved]

Key changes:
- [Change 1]
- [Change 2]
- [Change 3]

Bump version: 0.0.1 → 0.0.2

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"

# 5. Create annotated tag
git tag -a v0.0.2 -m "Version 0.0.2 - [Brief Title]

Changes:
- [Change 1]
- [Change 2]

Release Date: $(date +%Y-%m-%d)"

# 6. Rebuild from clean state
cargo clean && cargo build --release

# 7. Verify clean build
./target/release/uveddi -V
# Should show: uveddi 0.0.2 (xxxxxxxx-clean)

# 8. Check detailed version
./target/release/uveddi --version

# 9. Push to remote
git push origin $(git branch --show-current) --tags
```

---

### For MINOR Release (0.0.2 → 0.1.0)

New features, significant enhancements (backward compatible).

```bash
# 1. Update version in Cargo.toml
sed -i 's/^version = "0.0.2"/version = "0.1.0"/' Cargo.toml

# 2. Verify the change
grep '^version' Cargo.toml

# 3. Stage all changes
git add Cargo.toml [other-modified-files...]

# 4. Create commit
git commit -m "feat: [Brief description of new feature]

[Detailed description of the new feature/enhancement]

New features:
- [Feature 1]
- [Feature 2]
- [Feature 3]

Enhancements:
- [Enhancement 1]
- [Enhancement 2]

Bump version: 0.0.2 → 0.1.0

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"

# 5. Create annotated tag
git tag -a v0.1.0 -m "Version 0.1.0 - [Feature Name]

New Features:
- [Feature 1]
- [Feature 2]

Enhancements:
- [Enhancement 1]

Release Date: $(date +%Y-%m-%d)"

# 6. Rebuild and verify
cargo clean && cargo build --release
./target/release/uveddi -V
./target/release/uveddi --version

# 7. Push to remote
git push origin $(git branch --show-current) --tags
```

---

### For MAJOR Release (0.x.x → 1.0.0)

Breaking changes, first stable release, major rewrites.

```bash
# 1. Create new release branch (optional but recommended)
git checkout -b release/1.0.0

# 2. Update version in Cargo.toml
sed -i 's/^version = "0.x.x"/version = "1.0.0"/' Cargo.toml

# 3. Update documentation for stable release
# - Update README.md
# - Update CHANGELOG.md
# - Review all documentation

# 4. Stage all changes
git add Cargo.toml [other-modified-files...]

# 5. Create commit
git commit -m "feat: Version 1.0.0 stable release

[Comprehensive description of the stable release]

Major changes:
- [Breaking change 1]
- [Breaking change 2]

New features:
- [Feature 1]
- [Feature 2]

Migration guide:
- [Migration step 1]
- [Migration step 2]

Bump version: 0.x.x → 1.0.0

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com)"

# 6. Create annotated tag with comprehensive notes
git tag -a v1.0.0 -m "Version 1.0.0 - First Stable Release

This is the first stable release of Uveddi!

Major Features:
- [Feature 1]
- [Feature 2]

Breaking Changes:
- [Breaking change 1]
- [Breaking change 2]

Release Date: $(date +%Y-%m-%d)"

# 7. Rebuild and verify
cargo clean && cargo build --release
./target/release/uveddi -V
./target/release/uveddi --version

# 8. Run full test suite
cargo test --all-features

# 9. Push to remote
git push origin release/1.0.0 --tags
```

---

## Post-Release Verification

After pushing the release, verify everything is correct:

### 1. Check Remote Repository

```bash
# Verify tag was pushed
git ls-remote --tags origin | grep v0.0.2

# Verify commit was pushed
git log --oneline -1
```

### 2. Verify Binary

```bash
# Build fresh from clean state
cargo clean
cargo build --release

# Check version matches tag
./target/release/uveddi -V
# Should show: uveddi 0.0.2 (xxxxxxxx-clean)

# Check git hash matches current commit
git rev-parse --short=8 HEAD
# Should match hash in binary version
```

### 3. Test Functionality

```bash
# Run quick analysis
./target/release/uveddi analyze tests/ --output-format markdown

# Verify all detectors work
./target/release/uveddi analyze tests/detector_validation_extended.rs --output-format markdown
```

### 4. Update Release Notes (Optional)

Create a CHANGELOG.md entry or GitHub release notes:

```markdown
## [0.0.2] - 2025-10-14

### Added
- Build versioning system with git commit tracking
- Feature flag display in version output
- Enhanced version display with build information

### Changed
- Version display now shows commit hash and git status
- Improved build-time metadata capture

### Fixed
- None

### Documentation
- Added BUILD-VERSIONING-SYSTEM.md
- Added VERSION-RELEASE-WORKFLOW.md
```

---

## Version Numbering Guide

Use [Semantic Versioning](https://semver.org/): `MAJOR.MINOR.PATCH`

### When to Bump PATCH (0.0.1 → 0.0.2)

- Bug fixes
- Performance improvements
- Documentation updates
- Code cleanup/refactoring (no API changes)
- Small enhancements that don't add features

**Examples:**
- Fixed detector not running
- Improved error messages
- Updated documentation
- Optimized performance

### When to Bump MINOR (0.0.2 → 0.1.0)

- New features (backward compatible)
- New detectors
- New output formats
- Significant enhancements to existing features
- New CLI commands

**Examples:**
- Added AI-powered suggestions
- New detector for cyclomatic complexity
- Added JSON output format
- Plugin system

### When to Bump MAJOR (0.x.x → 1.0.0 or 1.x.x → 2.0.0)

- Breaking API changes
- Removed features
- Changed CLI interface (breaking)
- Major architecture changes
- First stable public release (0.x.x → 1.0.0)

**Examples:**
- Changed command-line argument names
- Removed deprecated detectors
- Changed output format structure
- Rewrote core analysis engine

### Pre-1.0.0 Releases

For versions `0.x.x`, you can be more flexible:
- `0.0.x`: Bug fixes and small changes
- `0.x.0`: New features (may include breaking changes)

---

## Common Scenarios

### Scenario 1: Quick Bug Fix

```bash
# Fix the bug, test it, then:
sed -i 's/^version = "0.0.1"/version = "0.0.2"/' Cargo.toml
git add Cargo.toml [fixed-files...]
git commit -m "fix: Correct [bug description]

[Details about the fix]

Fixes #123

Bump version: 0.0.1 → 0.0.2"
git tag v0.0.2
cargo clean && cargo build --release
./target/release/uveddi -V
git push origin $(git branch --show-current) --tags
```

### Scenario 2: New Feature

```bash
# Implement feature, test it, then:
sed -i 's/^version = "0.0.2"/version = "0.1.0"/' Cargo.toml
git add Cargo.toml [feature-files...]
git commit -m "feat: Add [feature name]

[Feature description and usage]

Bump version: 0.0.2 → 0.1.0"
git tag v0.1.0
cargo clean && cargo build --release
./target/release/uveddi --version
git push origin $(git branch --show-current) --tags
```

### Scenario 3: Documentation Only

```bash
# Update docs, then:
sed -i 's/^version = "0.0.1"/version = "0.0.2"/' Cargo.toml
git add Cargo.toml [doc-files...]
git commit -m "docs: [Documentation improvement]

[What was documented/improved]

Bump version: 0.0.1 → 0.0.2"
git tag v0.0.2
cargo build --release
git push origin $(git branch --show-current) --tags
```

### Scenario 4: Multiple Changes

```bash
# After implementing multiple fixes/features:
sed -i 's/^version = "0.1.0"/version = "0.2.0"/' Cargo.toml
git add Cargo.toml [all-modified-files...]
git commit -m "feat: Multiple improvements for v0.2.0

This release includes several enhancements and fixes:

Features:
- [Feature 1]
- [Feature 2]

Fixes:
- [Fix 1]
- [Fix 2]

Documentation:
- [Doc update 1]

Bump version: 0.1.0 → 0.2.0"
git tag -a v0.2.0 -m "Version 0.2.0 - [Release name]

Features:
- [Feature 1]
- [Feature 2]

Fixes:
- [Fix 1]
- [Fix 2]

Release Date: $(date +%Y-%m-%d)"
cargo clean && cargo build --release
./target/release/uveddi --version
git push origin $(git branch --show-current) --tags
```

---

## Troubleshooting

### Binary Shows "modified" After Release

**Problem:** Binary shows `uveddi 0.0.2 (xxxxxxxx-modified)` instead of `clean`

**Cause:** Built before committing changes, or uncommitted files exist

**Solution:**
```bash
# Check for uncommitted changes
git status

# If clean, rebuild
cargo clean
cargo build --release
./target/release/uveddi -V
```

### Version Doesn't Update in Binary

**Problem:** Binary still shows old version after bumping Cargo.toml

**Cause:** Incremental compilation cache

**Solution:**
```bash
cargo clean
rm -rf target/
cargo build --release
./target/release/uveddi -V
```

### Git Hash Doesn't Match

**Problem:** Hash in binary doesn't match current commit

**Cause:** Binary was built before commit

**Solution:**
```bash
# Always commit FIRST, then rebuild
git commit -m "..."
cargo clean
cargo build --release
```

### Tag Already Exists

**Problem:** `git tag v0.0.2` fails because tag exists

**Solution:**
```bash
# Delete local tag
git tag -d v0.0.2

# Delete remote tag (if pushed)
git push origin :refs/tags/v0.0.2

# Recreate tag
git tag -a v0.0.2 -m "..."
```

### Pushed Wrong Version

**Problem:** Pushed release but need to fix something

**Solution:**
```bash
# For minor fixes, create a patch release
sed -i 's/^version = "0.0.2"/version = "0.0.3"/' Cargo.toml
git add Cargo.toml [fixes...]
git commit -m "fix: Correct issue in v0.0.2

Bump version: 0.0.2 → 0.0.3"
git tag v0.0.3
cargo clean && cargo build --release
git push origin $(git branch --show-current) --tags
```

---

## Commit Message Templates

### Bug Fix
```
fix: [Short description]

[Detailed description of the bug and fix]

Fixes #[issue-number]

Bump version: [old] → [new]
```

### New Feature
```
feat: [Short description]

[Detailed description of the feature]

Key features:
- [Feature aspect 1]
- [Feature aspect 2]

Usage:
[How to use the feature]

Bump version: [old] → [new]
```

### Documentation
```
docs: [Short description]

[What was documented]

Bump version: [old] → [new]
```

### Performance
```
perf: [Short description]

[What was optimized and why]

Performance improvements:
- [Improvement 1]
- [Improvement 2]

Bump version: [old] → [new]
```

### Refactoring
```
refactor: [Short description]

[What was refactored and why]

No functional changes.

Bump version: [old] → [new]
```

---

## Build Versioning System

Every Uveddi binary includes:

- **BUILD_TIMESTAMP**: When the binary was compiled
- **GIT_HASH**: 8-character commit hash
- **GIT_BRANCH**: Branch name at build time
- **GIT_DIRTY**: Working directory status (clean/modified)
- **BUILD_FEATURES**: Enabled Cargo features

### Verifying Binary Freshness

```bash
# Quick check
./target/release/uveddi -V
# Shows: uveddi 0.0.2 (c0f041a9-clean)

# Detailed check
./target/release/uveddi --version
# Shows full build information

# Compare with git
git rev-parse --short=8 HEAD
# Should match hash in binary
```

### Clean vs Modified Status

- **clean**: Binary built from committed code (release-ready)
- **modified**: Binary has uncommitted changes (development build)

**Always ensure "clean" status before releasing!**

---

## Quick Command Reference

```bash
# Check current version
grep '^version' Cargo.toml

# Update version (patch)
sed -i 's/version = "X.Y.Z"/version = "X.Y.Z+1"/' Cargo.toml

# Build clean release
cargo clean && cargo build --release

# Check binary version
./target/release/uveddi -V

# Create tag
git tag -a vX.Y.Z -m "Version X.Y.Z"

# Push with tags
git push origin $(git branch --show-current) --tags

# Verify git hash matches
git rev-parse --short=8 HEAD
```

---

## Release Checklist Template

Copy this for each release:

```
Release: v[X.Y.Z]
Date: [YYYY-MM-DD]

Pre-Release:
- [ ] All tests pass
- [ ] Documentation updated
- [ ] Clean build succeeds
- [ ] Version number decided

Release:
- [ ] Updated Cargo.toml version
- [ ] Staged all files
- [ ] Created commit with proper message
- [ ] Created annotated tag
- [ ] Rebuilt from clean state
- [ ] Verified "clean" status in binary
- [ ] Verified git hash matches

Post-Release:
- [ ] Pushed to remote with tags
- [ ] Verified tag on remote
- [ ] Tested fresh build
- [ ] Updated CHANGELOG (if applicable)
- [ ] Created GitHub release (if applicable)
```

---

## Notes

- **Always rebuild after committing** to get "clean" status
- **Use semantic versioning** consistently
- **Tag every release** for easy tracking
- **Write descriptive commit messages** for future reference
- **Test before releasing** - run `cargo test` and manual tests
- **Document breaking changes** clearly in commit messages

---

**Last Updated:** 2025-10-14
**Based on Release:** v0.0.2
**Next Review:** After v1.0.0
