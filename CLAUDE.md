# Claude Development Guide for Uveddi

## 🔧 Building and Installing

### Binary Location
**IMPORTANT**: The uveddi binary must be copied to `~/.local/bin/uveddi` (NOT `/usr/local/bin`)

The user's PATH prioritizes `~/.local/bin`, so always use this location.

### Build and Install Commands

```bash
# Build release binary
cargo build --release --bin uveddi

# Copy to the correct location (no sudo needed)
cp target/release/uveddi ~/.local/bin/uveddi

# Verify installation
uveddi --version
```

### If Binary is "Text file busy"

This means the binary is currently running. Remove it first:

```bash
rm ~/.local/bin/uveddi && cp target/release/uveddi ~/.local/bin/uveddi
```

## 📍 Version Information

The binary includes a build timestamp for tracking versions:
- Format: `uveddi 1.0.0 (2025-10-14 15:34:00 UTC)`
- Set automatically by `build.rs` using `BUILD_TIMESTAMP` environment variable
- Always check version after copying: `uveddi --version`

## 🔄 Development Workflow

1. Make code changes
2. Build: `cargo build --release --bin uveddi`
3. Copy: `cp target/release/uveddi ~/.local/bin/uveddi`
4. Test: `uveddi --version` and run your command
5. Check timestamp to confirm you're using the new binary

## ⚠️ Common Mistakes

- ❌ Copying to `/usr/local/bin/uveddi` (wrong location!)
- ❌ Using `sudo` (not needed for `~/.local/bin`)
- ❌ Forgetting to check version after copying
- ❌ Not removing the old binary when it's "busy"

## ✅ Correct Pattern

```bash
# One-liner for build and install
cargo build --release --bin uveddi && \
cp target/release/uveddi ~/.local/bin/uveddi && \
echo "✅ Binary updated!" && \
uveddi --version
```

## 🧪 Testing

```bash
# Test basic functionality
uveddi --help

# Test with a small project
uveddi analyze ./src/progress

# Check progress animation
uveddi analyze ./src --progress-format terminal
```

## 📦 Progress Animation System

Location: `src/progress/mod.rs`

Key features:
- Background thread animates spinner continuously
- Updates every 100ms for smooth animation
- Phases: Discovery → Parsing → Analysis → Complete
- Simple spinner-only output (no percentages by default)

The spinner will keep animating until `progress_tracker.complete()` is called or the tracker is dropped.
