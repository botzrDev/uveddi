# Uveddi Release Build Quick Reference

Quick reference for building, signing, and distributing Uveddi releases.

## Prerequisites

```bash
# Rust toolchain
rustup update
rustup target add x86_64-unknown-linux-musl

# Build tools (Linux)
sudo apt-get install pkg-config libssl-dev musl-tools

# Optional: GPG for signing
gpg --full-generate-key

# Optional: Security scanners
# Trivy: https://trivy.dev/latest/getting-started/installation/
# Grype: https://github.com/anchore/grype#installation
```

## Quick Build Commands

### Build All Binaries

```bash
# Build everything
./scripts/build_release.sh

# Build without signing
./scripts/build_release.sh --skip-sign

# Build without tests
UVEDDI_SKIP_TESTS=true ./scripts/build_release.sh

# Build specific target
UVEDDI_BUILD_TARGET=x86_64-unknown-linux-gnu ./scripts/build_release.sh
```

### Build Container

```bash
# Build with security scan
./scripts/build_container.sh --scan

# Build and push to registry
./scripts/build_container.sh --scan --push

# Build Alpine variant
UVEDDI_VARIANT=alpine ./scripts/build_container.sh

# Build multi-platform
./scripts/build_container.sh --platform linux/amd64,linux/arm64
```

### Sign and Verify

```bash
# Generate checksums only
./scripts/sign_artifacts.sh checksums

# Sign artifacts (requires GPG)
./scripts/sign_artifacts.sh sign

# Verify everything
./scripts/sign_artifacts.sh verify

# Complete workflow
./scripts/sign_artifacts.sh all
```

## Complete Release Workflow

```bash
# 1. Prepare branch
git checkout release/1.0.0
git pull origin release/1.0.0

# 2. Build all artifacts
./scripts/build_release.sh

# 3. Build containers
./scripts/build_container.sh --scan

# 4. Sign everything
export UVEDDI_GPG_KEY=<your-key-id>
./scripts/sign_artifacts.sh all

# 5. Verify
./scripts/sign_artifacts.sh verify

# 6. Test installation
./scripts/install.sh

# 7. Create release
gh release create v1.0.0 \
  --title "Uveddi v1.0.0" \
  --notes-file RELEASE_NOTES_v1.0.0.md \
  dist/archives/* \
  dist/checksums/SHA256SUMS* \
  dist/checksums/SHA512SUMS* \
  dist/signatures/*.asc \
  dist/UVEDDI_RELEASE_KEY.asc

# 8. Push containers
docker push ghcr.io/botzrdev/uveddi:1.0.0
docker push ghcr.io/botzrdev/uveddi:latest
```

## Testing Installation

### Linux

```bash
# In Docker container
docker run -it --rm ubuntu:22.04 bash -c \
  "apt update && apt install -y curl ca-certificates && \
   curl -fsSL https://github.com/botzrDev/uveddi/releases/download/v1.0.0/install.sh | bash"
```

### macOS

```bash
# Test locally
./scripts/install.sh

# Or download from release
curl -fsSL https://github.com/botzrDev/uveddi/releases/download/v1.0.0/install.sh | bash
```

### Windows

```powershell
# Test PowerShell script
.\scripts\install.ps1

# Or download from release
irm https://github.com/botzrDev/uveddi/releases/download/v1.0.0/install.ps1 | iex
```

## Verification

### Verify Checksums

```bash
# Download and verify
wget https://github.com/botzrDev/uveddi/releases/download/v1.0.0/uveddi-1.0.0-x86_64-unknown-linux-gnu.tar.gz
wget https://github.com/botzrDev/uveddi/releases/download/v1.0.0/SHA256SUMS

sha256sum -c <<< "$(grep uveddi-1.0.0-x86_64-unknown-linux-gnu.tar.gz SHA256SUMS)"
```

### Verify Signatures

```bash
# Import public key
wget https://github.com/botzrDev/uveddi/releases/download/v1.0.0/UVEDDI_RELEASE_KEY.asc
gpg --import UVEDDI_RELEASE_KEY.asc

# Verify checksum file
wget https://github.com/botzrDev/uveddi/releases/download/v1.0.0/SHA256SUMS.asc
gpg --verify SHA256SUMS.asc SHA256SUMS

# Verify individual artifact
wget https://github.com/botzrDev/uveddi/releases/download/v1.0.0/uveddi-1.0.0-x86_64-unknown-linux-gnu.tar.gz.asc
gpg --verify uveddi-1.0.0-x86_64-unknown-linux-gnu.tar.gz.asc uveddi-1.0.0-x86_64-unknown-linux-gnu.tar.gz
```

## Directory Structure

After successful build:

```
dist/
├── binaries/                          # Raw binaries
│   ├── uveddi-1.0.0-x86_64-unknown-linux-gnu
│   ├── uveddi-1.0.0-x86_64-unknown-linux-musl
│   ├── uveddi-1.0.0-aarch64-unknown-linux-gnu
│   ├── uveddi-1.0.0-x86_64-apple-darwin
│   ├── uveddi-1.0.0-aarch64-apple-darwin
│   └── uveddi-1.0.0-x86_64-pc-windows-gnu.exe
├── archives/                          # Distribution packages
│   ├── uveddi-1.0.0-x86_64-unknown-linux-gnu.tar.gz
│   ├── uveddi-1.0.0-x86_64-unknown-linux-musl.tar.gz
│   ├── uveddi-1.0.0-aarch64-unknown-linux-gnu.tar.gz
│   ├── uveddi-1.0.0-x86_64-apple-darwin.tar.gz
│   ├── uveddi-1.0.0-aarch64-apple-darwin.tar.gz
│   ├── uveddi-1.0.0-x86_64-pc-windows-gnu.zip
│   ├── SHA256SUMS
│   ├── SHA512SUMS
│   └── *.asc (signatures)
├── checksums/                         # Checksum files
│   ├── SHA256SUMS
│   ├── SHA512SUMS
│   ├── MD5SUMS
│   └── *.sha256 (individual)
├── signatures/                        # GPG signatures
│   ├── *.tar.gz.asc
│   ├── *.zip.asc
│   ├── SHA256SUMS.asc
│   └── SHA512SUMS.asc
├── BUILD_MANIFEST.txt                 # Build details
├── VERIFICATION.md                    # Verification guide
└── UVEDDI_RELEASE_KEY.asc            # Public key
```

## Environment Variables

### Build Configuration

```bash
export UVEDDI_VERSION=1.0.0                    # Version to build
export UVEDDI_BUILD_TARGET=<target>            # Specific target
export UVEDDI_SKIP_TESTS=true                  # Skip tests
export UVEDDI_GPG_KEY=<key-id>                 # GPG key for signing
export SOURCE_DATE_EPOCH=$(git log -1 --format=%ct)  # Reproducible builds
```

### Container Configuration

```bash
export UVEDDI_REGISTRY=ghcr.io/botzrdev        # Container registry
export UVEDDI_VARIANT=distroless               # distroless or alpine
```

### Installation Configuration

```bash
export UVEDDI_VERSION=1.0.0                    # Version to install
export UVEDDI_INSTALL_DIR=~/.local/bin         # Install directory
export UVEDDI_VERIFY_SIGNATURE=true            # Verify GPG signatures
export UVEDDI_BASE_URL=https://...             # Custom download URL
```

## Troubleshooting

### Build fails with "target not found"

```bash
rustup target add <target>
```

### Windows cross-compile fails

```bash
# Install MinGW
sudo apt-get install mingw-w64
```

### Container build fails

```bash
# Create buildx builder
docker buildx create --name uveddi-builder --use
docker buildx inspect --bootstrap
```

### GPG signing fails

```bash
# List available keys
gpg --list-secret-keys --keyid-format LONG

# Specify key
export UVEDDI_GPG_KEY=<key-id>
```

### Security scan fails

```bash
# Install Trivy
wget -qO - https://aquasecurity.github.io/trivy-repo/deb/public.key | sudo apt-key add -
echo "deb https://aquasecurity.github.io/trivy-repo/deb $(lsb_release -sc) main" | sudo tee /etc/apt/sources.list.d/trivy.list
sudo apt-get update && sudo apt-get install trivy

# Install Grype
curl -sSfL https://raw.githubusercontent.com/anchore/grype/main/install.sh | sh -s -- -b /usr/local/bin
```

## CI/CD Integration

### GitHub Actions

```yaml
name: Release Build

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Build Release
        run: ./scripts/build_release.sh --skip-sign
        
      - name: Build Container
        run: ./scripts/build_container.sh --scan
        
      - name: Upload Artifacts
        uses: actions/upload-artifact@v3
        with:
          name: release-artifacts
          path: dist/
```

## Documentation

- **Manifest:** [docs/release-artifacts/manifest-1.0.0.md](../docs/release-artifacts/manifest-1.0.0.md)
- **Checklist:** [docs/release-artifacts/distribution-checklist-1.0.0.md](../docs/release-artifacts/distribution-checklist-1.0.0.md)
- **Assignment:** [assignments/ASSIGNMENT-A6-PACKAGING-DISTRIBUTION.md](../assignments/ASSIGNMENT-A6-PACKAGING-DISTRIBUTION.md)

## Support

- Issues: https://github.com/botzrDev/uveddi/issues
- Security: security@uveddi.io
- Documentation: https://uveddi.io/docs
