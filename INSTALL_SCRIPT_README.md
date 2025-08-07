# Uveddi Installation Script

## Overview

The `install.sh` script provides a one-line installation method for Uveddi:

```bash
curl -sSL https://uveddi.org/install.sh | bash
```

## What the Script Does

1. **System Detection**: Automatically detects Linux, macOS, or Windows (WSL2)
2. **Dependency Installation**: 
   - Installs Rust toolchain if missing (via rustup)
   - Installs build essentials (gcc, pkg-config, libssl-dev) if needed
3. **Repository Setup**: Clones the Uveddi repository to `~/.uveddi`
4. **Build Process**: Compiles Uveddi with alpha features enabled
5. **Binary Installation**: Installs the binary to `~/.local/bin/uveddi`
6. **PATH Configuration**: Adds `~/.local/bin` to your PATH automatically
7. **Verification**: Tests that the installation works properly

## Installation Locations

- **Repository**: `~/.uveddi` (kept for future updates)
- **Binary**: `~/.local/bin/uveddi` 
- **PATH**: Added to `~/.bashrc`, `~/.zshrc`, or `~/.profile`

## Prerequisites

The script automatically handles most dependencies, but you may need:

- **Internet connection** (for downloading Rust and repository)
- **sudo access** (for installing system packages on Linux)

## System Support

- ✅ **Linux** (Ubuntu, Debian, RHEL, CentOS, Fedora)
- ✅ **macOS** (with Xcode command line tools)  
- ⚠️ **Windows** (WSL2 recommended)

## Features

- **Robust error handling** with clear error messages
- **Progress indicators** for long operations
- **Automatic dependency management**
- **System-specific package manager detection**
- **PATH configuration for all major shells**
- **Build verification** to ensure working installation

## Alpha Release Status

✅ **Ready for Production**: All alpha-blocking issues have been resolved:

1. ✅ Database storage failures fixed
2. ✅ Performance scalability improved with memory optimization  
3. ✅ Error messages are now specific and actionable
4. ✅ Template engine stability issues resolved

The script installs a fully functional alpha version suitable for testing and evaluation.

## Troubleshooting

If installation fails:

1. Check the error message - the script provides specific guidance
2. Ensure you have internet connectivity
3. For permission issues, make sure you have sudo access
4. For build issues, verify you have adequate disk space (2GB+)
5. Report issues at: https://github.com/botzrDev/uveddi/issues

## Manual Installation Alternative

If the script doesn't work for your system:

```bash
# Clone repository
git clone -b alpha https://github.com/botzrDev/uveddi.git
cd uveddi

# Build with alpha features
cargo build --release --features="alpha"

# Test installation
./target/release/uveddi --help
```

## Validation

The script includes comprehensive validation and can be tested with:

```bash
# Download and validate without installing
curl -sSL https://uveddi.org/install.sh > install.sh
bash -n install.sh  # Syntax check
```

## Security

- Script uses `set -euo pipefail` for robust error handling
- All downloads use HTTPS
- Repository is cloned from the official GitHub repository
- No arbitrary code execution from external sources

## Deployment

For deployment to production:

1. Upload `install.sh` to `https://uveddi.org/install.sh`
2. Ensure the file is served with proper MIME type (`text/plain` or `application/x-shellscript`)
3. Test the full installation process
4. Update frontend links to use the working installation command