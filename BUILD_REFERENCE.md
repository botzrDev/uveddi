# Quick Reference: Building Uveddi

## 📍 Binary Installation Location

```bash
~/.local/bin/uveddi  ✅ CORRECT - Use this!
/usr/local/bin/uveddi  ❌ WRONG - Don't use this!
```

## 🚀 Quick Commands

```bash
# Build and install
cargo build --release --bin uveddi && cp target/release/uveddi ~/.local/bin/uveddi

# Check version
uveddi --version

# Test
uveddi analyze ./src
```

## 🔍 Verify You're Using the Right Binary

```bash
which uveddi
# Should output: /home/austingreen/.local/bin/uveddi

uveddi --version
# Should show current timestamp: uveddi 1.0.0 (2025-10-14 HH:MM:SS UTC)
```

## 🐛 Troubleshooting

### "Text file busy" error?
```bash
rm ~/.local/bin/uveddi && cp target/release/uveddi ~/.local/bin/uveddi
```

### Old version still running?
- Check timestamp in `uveddi --version`
- Compare with `ls -lh target/release/uveddi`
- Make sure you copied to `~/.local/bin/` not `/usr/local/bin/`
