# Version Release Workflow for Uveddi

**Best Practices for Version Bumps and Commits**

---

## Quick Reference

```bash
# For v0.0.2 release (current situation):
./scripts/release.sh 0.0.2 "Add build versioning system with git info and feature detection"

# Or manual steps below...
```

---

## Complete Workflow for Version 0.0.2

### Step 1: Update Version in Cargo.toml

```bash
# Open Cargo.toml and change version
sed -i 's/^version = "0.0.1"/version = "0.0.2"/' Cargo.toml

# Verify the change
grep '^version' Cargo.toml
```

**Output should be:**
```toml
version = "0.0.2"
```

---

### Step 2: Review Your Changes

```bash
# See what files changed
git status

# See detailed changes
git diff

# See untracked files
git ls-files --others --exclude-standard
```

**Current changes for v0.0.2:**
- `build.rs` - Added git info and feature capture (+108 lines)
- `src/main.rs` - Enhanced version display (+26 lines)
- `BUILD-VERSIONING-SYSTEM.md` - New documentation (untracked)

---

### Step 3: Build and Test with New Version

```bash
# Clean previous build
cargo clean

# Build with new version
cargo build --release

# Verify version shows 0.0.2
./target/release/uveddi -V
# Should show: uveddi 0.0.2 (xxxxxxxx-modified)

# Test detailed version
./target/release/uveddi --version
# Should show: Uveddi 0.0.2
```

---

### Step 4: Stage Your Changes

```bash
# Stage modified files
git add Cargo.toml
git add build.rs
git add src/main.rs

# Stage new documentation
git add BUILD-VERSIONING-SYSTEM.md

# Review what's staged
git status
```

**Should show:**
```
Changes to be committed:
  modified:   Cargo.toml
  modified:   build.rs
  modified:   src/main.rs
  new file:   BUILD-VERSIONING-SYSTEM.md
```

---

### Step 5: Create Commit

```bash
# Commit with semantic versioning message
git commit -m "feat: Add comprehensive build versioning system

- Capture git commit, branch, and dirty status at build time
- Display build timestamp in version output
- Show enabled Cargo features in --version output
- Add short version format: uveddi -V shows commit hash
- Add detailed version format: uveddi --version shows full build info
- Document versioning system in BUILD-VERSIONING-SYSTEM.md

Build versioning now includes:
- BUILD_TIMESTAMP: When binary was compiled
- GIT_HASH: 8-character commit hash
- GIT_BRANCH: Branch name at build time
- GIT_DIRTY: Working directory status (clean/modified)
- BUILD_FEATURES: Comma-separated list of enabled features

Usage:
- uveddi -V: Quick version with commit hash
- uveddi --version: Detailed build information

This allows users to verify if their binary is up-to-date and
see exactly which features are compiled in.

Bump version: 0.0.1 → 0.0.2

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Step 6: Verify Commit

```bash
# Check commit was created
git log --oneline -1

# See full commit details
git show --stat

# Verify you're still on correct branch
git branch --show-current
```

---

### Step 7: Rebuild from Clean State

```bash
# Clean build artifacts
cargo clean

# Build release from committed code
cargo build --release

# Verify version is now "clean" (not "modified")
./target/release/uveddi -V
# Should show: uveddi 0.0.2 (xxxxxxxx-clean)
#                                      ^^^^^
#                                      Clean status!

# Check detailed version
./target/release/uveddi --version
```

**You should see:**
```
Uveddi 0.0.2

Build Information:
  Built:    2025-10-14 21:15:30 UTC
  Commit:   abc12345 (release/0.0.1)
  Status:   clean                      ← ✅ Shows "clean"!
  Features: tree-sitter,security,ast-cache,analysis-cache,rust,python,javascript,typescript
```

---

### Step 8: Tag the Release (Optional but Recommended)

```bash
# Create annotated tag
git tag -a v0.0.2 -m "Version 0.0.2 - Build Versioning System

Features:
- Git commit and branch tracking
- Build timestamp capture
- Feature flag display
- Clean/modified status detection

Release Date: 2025-10-14"

# Verify tag was created
git tag -l

# Show tag details
git show v0.0.2
```

---

### Step 9: Push to Remote

```bash
# Push commits
git push origin release/0.0.1

# Push tags
git push origin v0.0.2

# Or push everything at once
git push origin release/0.0.1 --tags
```

---

## Semantic Versioning Guide

### Version Format: MAJOR.MINOR.PATCH

**0.0.2** means:
- **MAJOR (0)**: Pre-release / alpha / beta
- **MINOR (0)**: Feature additions
- **PATCH (2)**: Bug fixes and small improvements

### When to Bump Which Number:

#### MAJOR (1.0.0, 2.0.0)
- Breaking API changes
- Major rewrites
- First stable public release (0.x.x → 1.0.0)

**Examples:**
- `0.9.5` → `1.0.0` (first stable release)
- `1.5.3` → `2.0.0` (breaking changes)

#### MINOR (0.1.0, 0.2.0)
- New features (backward compatible)
- Significant enhancements
- New detectors or major functionality

**Examples:**
- `0.0.2` → `0.1.0` (added AI support)
- `0.1.0` → `0.2.0` (added plugin system)

#### PATCH (0.0.2, 0.0.3)
- Bug fixes
- Performance improvements
- Documentation updates
- Small enhancements

**Examples:**
- `0.0.1` → `0.0.2` (build versioning system)
- `0.0.2` → `0.0.3` (fix detector bug)

---

## Commit Message Format

### Template:

```
<type>: <subject>

<body>

<footer>
```

### Types:

- **feat**: New feature
- **fix**: Bug fix
- **docs**: Documentation changes
- **style**: Code style changes (formatting)
- **refactor**: Code refactoring
- **perf**: Performance improvements
- **test**: Adding tests
- **chore**: Maintenance tasks

### Examples:

#### Feature Addition (MINOR bump)
```bash
git commit -m "feat: Add AI-powered code suggestions

- Integrate Ollama for local AI inference
- Add suggestion generation for detected issues
- Implement confidence scoring for suggestions

Bump version: 0.0.2 → 0.1.0"
```

#### Bug Fix (PATCH bump)
```bash
git commit -m "fix: Correct duplicate detection threshold

- Increase min_tokens from 30 to 50
- Fix false positives in small code blocks
- Add test cases for edge cases

Fixes #123

Bump version: 0.0.2 → 0.0.3"
```

#### Documentation (PATCH bump)
```bash
git commit -m "docs: Add comprehensive detector validation report

- Document all 8 detectors
- Include accuracy metrics
- Add usage examples

Bump version: 0.0.2 → 0.0.3"
```

---

## Branch Strategy

### For Current Release (0.0.x)

```bash
# Stay on release/0.0.1 branch for patches
git checkout release/0.0.1

# Make changes, commit with version bump
git commit -m "feat: ..."

# Tag each release
git tag v0.0.2
```

### For Minor Release (0.1.x)

```bash
# Create new release branch
git checkout -b release/0.1.0

# Update version in Cargo.toml
sed -i 's/version = "0.0.2"/version = "0.1.0"/' Cargo.toml

# Commit and tag
git commit -m "feat: ..."
git tag v0.1.0
```

### For Major Release (1.0.0)

```bash
# Create new release branch
git checkout -b release/1.0.0

# Update version in Cargo.toml
sed -i 's/version = "0.x.x"/version = "1.0.0"/' Cargo.toml

# Update documentation for public release
# Run full test suite
# Generate release notes

# Commit and tag
git commit -m "feat: Version 1.0.0 stable release"
git tag v1.0.0
```

---

## Automated Script (Optional)

Create `scripts/release.sh`:

```bash
#!/bin/bash
# Uveddi Version Release Script

set -e  # Exit on error

VERSION=$1
MESSAGE=$2

if [ -z "$VERSION" ] || [ -z "$MESSAGE" ]; then
    echo "Usage: ./scripts/release.sh <version> <message>"
    echo "Example: ./scripts/release.sh 0.0.2 'Add build versioning'"
    exit 1
fi

echo "🚀 Releasing Uveddi v$VERSION"
echo ""

# 1. Update version in Cargo.toml
echo "📝 Updating Cargo.toml..."
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
grep "^version" Cargo.toml

# 2. Clean build
echo "🧹 Cleaning previous build..."
cargo clean

# 3. Build release
echo "🔨 Building release..."
cargo build --release

# 4. Test version
echo "✅ Testing version..."
./target/release/uveddi -V

# 5. Stage changes
echo "📦 Staging changes..."
git add Cargo.toml

# 6. Commit
echo "💾 Creating commit..."
git commit -m "feat: $MESSAGE

Bump version: → $VERSION

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"

# 7. Tag
echo "🏷️  Creating tag v$VERSION..."
git tag -a "v$VERSION" -m "Version $VERSION: $MESSAGE"

# 8. Rebuild clean
echo "🔨 Rebuilding from clean state..."
cargo clean
cargo build --release

# 9. Verify
echo "✅ Verifying clean build..."
./target/release/uveddi --version | grep -E "Status:|Version:"

echo ""
echo "✅ Release v$VERSION complete!"
echo ""
echo "Next steps:"
echo "  git push origin $(git branch --show-current)"
echo "  git push origin v$VERSION"
```

Make it executable:
```bash
chmod +x scripts/release.sh
```

---

## Checklist for v0.0.2 Release

- [ ] Update `Cargo.toml` version to `0.0.2`
- [ ] Run `cargo build --release` and test
- [ ] Stage all modified files (`git add`)
- [ ] Create descriptive commit message
- [ ] Verify commit with `git show`
- [ ] Rebuild from clean state
- [ ] Verify version shows "clean" status
- [ ] Tag release with `git tag v0.0.2`
- [ ] Push to remote
- [ ] Update release notes/CHANGELOG (if exists)

---

## Verifying Your Release

After completing the workflow:

```bash
# 1. Check git log
git log --oneline -1

# 2. Check tags
git tag -l | tail -5

# 3. Verify binary version
./target/release/uveddi --version

# 4. Confirm clean status
./target/release/uveddi -V
# Should show: uveddi 0.0.2 (xxxxxxxx-clean)

# 5. Compare with git
git rev-parse --short=8 HEAD
# Should match commit hash in binary
```

---

## Troubleshooting

### "Modified" status after commit

**Problem:** Binary still shows "modified" status

**Cause:** Build artifacts from before commit

**Solution:**
```bash
cargo clean
cargo build --release
./target/release/uveddi -V  # Now shows "clean"
```

---

### Version doesn't update

**Problem:** Binary still shows old version

**Cause:** Cargo incremental compilation cache

**Solution:**
```bash
cargo clean
rm -rf target/
cargo build --release
```

---

### Git hash doesn't match

**Problem:** Binary hash doesn't match current commit

**Cause:** Binary was built before commit

**Solution:**
```bash
# Always rebuild AFTER committing
git commit -m "..."
cargo clean
cargo build --release
```

---

## Best Practices

### DO ✅

- Update version in Cargo.toml FIRST
- Build and test BEFORE committing
- Rebuild AFTER committing to get clean status
- Use semantic versioning
- Write descriptive commit messages
- Tag releases
- Document breaking changes

### DON'T ❌

- Commit without updating Cargo.toml version
- Skip testing after version bump
- Release with "modified" status
- Use vague commit messages
- Forget to push tags
- Break semantic versioning

---

## For Your Current Situation (v0.0.2)

Run these commands:

```bash
# 1. Update version
sed -i 's/^version = "0.0.1"/version = "0.0.2"/' Cargo.toml

# 2. Add all changes
git add Cargo.toml build.rs src/main.rs BUILD-VERSIONING-SYSTEM.md

# 3. Commit
git commit -m "feat: Add comprehensive build versioning system

- Capture git commit, branch, and dirty status at build time
- Display build timestamp in version output
- Show enabled Cargo features in --version output

Bump version: 0.0.1 → 0.0.2"

# 4. Tag
git tag v0.0.2

# 5. Rebuild clean
cargo clean && cargo build --release

# 6. Verify
./target/release/uveddi --version

# 7. Push
git push origin release/0.0.1 --tags
```

Done! 🎉
