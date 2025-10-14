# Uveddi Build Versioning System

**Status:** ✅ **Fully Implemented and Working**
**Date:** 2025-10-14

---

## Overview

Uveddi now has a comprehensive build versioning system that captures:
- **Build timestamp** - When the binary was compiled
- **Git commit hash** - Exact source code version (8-character short hash)
- **Git branch** - Which branch was built from
- **Git status** - Whether the build has uncommitted changes
- **Enabled features** - Which Cargo features are compiled in

This allows you to instantly verify if your installed binary is the most recent build and see exactly what features are enabled.

---

## Usage

### Quick Version Check (-V)
```bash
$ uveddi -V
uveddi 0.0.1 (c0f041a9-modified)
```

**Shows:**
- Version number
- Git commit hash (8 chars)
- Git status (clean or modified)

**Perfect for:** Quick version check, comparing binaries

---

### Detailed Version Info (--version)
```bash
$ uveddi --version
uveddi Uveddi 0.0.1

Build Information:
  Built:    2025-10-14 21:07:51 UTC
  Commit:   c0f041a9 (release/0.0.1)
  Status:   modified
  Features: tree-sitter,security,ast-cache,analysis-cache,rust,python,javascript,typescript

Repository: https://github.com/botzrDev/uveddi
```

**Shows:**
- Full version name
- Build timestamp (UTC)
- Git commit + branch
- Git working directory status
- All enabled Cargo features
- Repository URL

**Perfect for:** Debugging, bug reports, comparing feature sets

---

## How It Works

### 1. Build-Time Capture (build.rs)

The `build.rs` script runs during compilation and captures:

#### Git Information
```rust
fn capture_git_info() {
    // Captures:
    // - GIT_HASH: 8-character short commit hash
    // - GIT_BRANCH: Current branch name
    // - GIT_DIRTY: "clean" or "modified"
}
```

#### Feature Flags
```rust
fn capture_feature_flags() {
    // Checks which features are enabled and creates comma-separated list
    // Example: "tree-sitter,security,ast-cache,rust,python,javascript,typescript"
}
```

#### Build Timestamp
```rust
// Format: "YYYY-MM-DD HH:MM:SS UTC"
let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
```

### 2. Compile-Time Embedding (main.rs)

Version info is embedded as compile-time constants using `env!()`:

```rust
fn get_version_string() -> &'static str {
    concat!(
        env!("CARGO_PKG_VERSION"),
        " (", env!("GIT_HASH"), "-", env!("GIT_DIRTY"), ")"
    )
}

fn get_long_version() -> &'static str {
    concat!(
        "Uveddi ", env!("CARGO_PKG_VERSION"), "\n",
        "Built:    ", env!("BUILD_TIMESTAMP"), "\n",
        "Commit:   ", env!("GIT_HASH"), " (", env!("GIT_BRANCH"), ")\n",
        "Status:   ", env!("GIT_DIRTY"), "\n",
        "Features: ", env!("BUILD_FEATURES")
    )
}
```

### 3. Display via Clap

```rust
#[derive(Parser)]
#[command(version = get_version_string())]        // -V output
#[command(long_version = get_long_version())]     // --version output
struct Cli { ... }
```

---

## Verifying You Have the Latest Build

### Method 1: Check Commit Hash

```bash
# Get current git commit
$ git rev-parse --short=8 HEAD
c0f041a9

# Check binary version
$ uveddi -V
uveddi 0.0.1 (c0f041a9-modified)
```

✅ **Hashes match** = Binary is built from current commit
❌ **Hashes differ** = Binary is outdated, rebuild needed

---

### Method 2: Check Git Status

```bash
$ uveddi -V
uveddi 0.0.1 (c0f041a9-clean)     # No uncommitted changes
```

vs

```bash
$ uveddi -V
uveddi 0.0.1 (c0f041a9-modified)  # Has uncommitted changes
```

**"modified"** = Binary was built with uncommitted changes
**"clean"** = Binary matches committed source exactly

---

### Method 3: Check Build Timestamp

```bash
$ uveddi --version | grep Built
  Built:    2025-10-14 21:07:51 UTC

$ stat -c %y ./target/release/uveddi
2025-10-14 21:07:54.123456789 +0000
```

Compare timestamps to verify binary age.

---

### Method 4: Check Enabled Features

```bash
$ uveddi --version | grep Features
  Features: tree-sitter,security,ast-cache,analysis-cache,rust,python,javascript,typescript
```

Verify which features are enabled in your binary:
- **tree-sitter** - AST parsing for all languages
- **security** - Security vulnerability detection
- **ast-cache** - AST caching for performance
- **analysis-cache** - Analysis result caching
- **rust, python, javascript, typescript** - Language support

---

## CI/CD Integration

### GitHub Actions Example

```yaml
- name: Build and capture version
  run: |
    cargo build --release
    ./target/release/uveddi --version > version.txt

- name: Upload version artifact
  uses: actions/upload-artifact@v3
  with:
    name: version-info
    path: version.txt
```

### Docker Image Labeling

```dockerfile
# Build stage
FROM rust:1.75 as builder
WORKDIR /build
COPY . .
RUN cargo build --release

# Extract version info
RUN ./target/release/uveddi --version > /version.txt

# Runtime stage
FROM debian:bookworm-slim
COPY --from=builder /build/target/release/uveddi /usr/local/bin/
COPY --from=builder /version.txt /version.txt

# Add labels
RUN export BUILD_INFO=$(cat /version.txt | grep "Commit:" | awk '{print $2}') && \
    echo "LABEL version=${BUILD_INFO}" >> /tmp/labels

LABEL org.opencontainers.image.version="0.0.1"
```

---

## Development Workflow

### Building a Release

```bash
# 1. Ensure clean working directory
$ git status
On branch release/1.0.0
nothing to commit, working tree clean

# 2. Build release binary
$ cargo build --release

# 3. Verify version
$ ./target/release/uveddi --version
uveddi Uveddi 0.0.1

Build Information:
  Built:    2025-10-14 21:07:51 UTC
  Commit:   c0f041a9 (release/1.0.0)
  Status:   clean                           ← ✅ Clean status!
  Features: tree-sitter,security,...

# 4. Binary is release-ready!
```

---

### Debugging a Custom Build

```bash
# Build with different features
$ cargo build --release --features="cli-ai,wasm-plugins"

# Check what was built
$ ./target/release/uveddi --version
  Features: tree-sitter,security,ai,local-ai,wasm-plugins,...
            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
            Confirms AI and plugin features are enabled!
```

---

## Environment Variables Set by Build

| Variable | Example | Description |
|----------|---------|-------------|
| `BUILD_TIMESTAMP` | `2025-10-14 21:07:51 UTC` | When binary was compiled |
| `GIT_HASH` | `c0f041a9` | 8-char short commit hash |
| `GIT_BRANCH` | `release/1.0.0` | Branch name at build time |
| `GIT_DIRTY` | `clean` or `modified` | Working directory status |
| `BUILD_FEATURES` | `tree-sitter,security,...` | Comma-separated feature list |
| `CARGO_PKG_VERSION` | `0.0.1` | Version from Cargo.toml |

---

## Troubleshooting

### "unknown" Git Information

**Problem:**
```bash
$ uveddi --version
  Commit:   unknown (unknown)
```

**Cause:** Git not available or not in a git repository

**Solution:**
```bash
# Ensure git is installed
$ git --version

# Ensure you're in the git repo
$ git rev-parse --show-toplevel
```

---

### Mismatched Versions After Rebuild

**Problem:** Version doesn't change after rebuilding

**Cause:** Incremental compilation cache

**Solution:**
```bash
# Clean build cache
$ cargo clean

# Rebuild
$ cargo build --release
```

---

### Binary Shows "modified" on Clean Checkout

**Problem:** Git status shows "modified" even on clean checkout

**Cause:** Uncommitted `target/` directory or ignored files

**Solution:**
```bash
# Check what's modified
$ git status --porcelain

# If target/ is shown, update .gitignore
$ echo "target/" >> .gitignore
```

---

## Best Practices

### For Development

1. **Always check version before testing:**
   ```bash
   $ uveddi -V
   ```

2. **Rebuild after git operations:**
   ```bash
   $ git checkout main
   $ cargo build --release
   $ uveddi --version | grep Commit
   ```

3. **Use build timestamp to track latest:**
   ```bash
   $ ls -l ./target/release/uveddi
   $ uveddi --version | grep Built
   ```

---

### For Production

1. **Always build from clean checkout:**
   ```bash
   $ git status  # Ensure clean
   $ cargo build --release
   $ uveddi --version | grep Status
   ```

2. **Tag releases with git:**
   ```bash
   $ git tag v1.0.0
   $ cargo build --release
   $ uveddi --version
   ```

3. **Document features in release notes:**
   ```bash
   $ uveddi --version | grep Features
   ```

---

## Files Modified

| File | Purpose |
|------|---------|
| `build.rs` | Captures git info and features at build time |
| `src/main.rs` | Displays version information via CLI |
| `Cargo.toml` | Defines version and features |

---

## Future Enhancements

Potential additions for v1.1+:

1. **Binary Size Information**
   ```
   Binary:   12.4 MB (optimized)
   ```

2. **Compiler Version**
   ```
   Rustc:    1.75.0 (stable)
   ```

3. **Build Profile**
   ```
   Profile:  release (opt-level=3, lto=fat)
   ```

4. **Dependency Versions**
   ```
   Deps:     tree-sitter=0.20.0, tokio=1.35.0
   ```

---

## Conclusion

✅ **Build versioning system is fully functional!**

You can now:
- Instantly verify if your binary is up-to-date
- See exact git commit and branch
- Know which features are enabled
- Track build timestamps for debugging

**Quick Commands to Remember:**
```bash
uveddi -V                    # Quick version check
uveddi --version             # Detailed build info
uveddi --version | grep Commit   # Check git commit
uveddi --version | grep Features # Check enabled features
```

---

**System Validated:** 2025-10-14
**Status:** ✅ Production Ready
**Next Review:** After v1.0.0 release
