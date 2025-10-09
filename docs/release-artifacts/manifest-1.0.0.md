# Uveddi v1.0.0 Release Artifact Manifest

**Release Date:** TBD  
**Build Date:** TBD  
**Git Commit:** TBD  
**Release Branch:** `release/1.0.0`

## Overview

This document provides a comprehensive manifest of all Uveddi v1.0.0 release artifacts, including binaries, containers, checksums, and signatures. All artifacts have been built using reproducible build processes and cryptographically signed.

## Quick Links

- **Release Page:** https://github.com/botzrDev/uveddi/releases/tag/v1.0.0
- **Installation Guide:** [Installation Instructions](#installation-instructions)
- **Verification Guide:** [Verification Instructions](#verification-instructions)
- **Security Contact:** security@uveddi.io

## Distribution Artifacts

### Binary Releases

All binary releases include:
- Uveddi executable
- LICENSE file
- README.md
- CHANGELOG.md

#### Linux

| Platform | Architecture | File | Size | SHA256 | Download |
|----------|--------------|------|------|--------|----------|
| Linux (GNU) | x86_64 | `uveddi-1.0.0-x86_64-unknown-linux-gnu.tar.gz` | TBD | TBD | [Download](https://github.com/botzrDev/uveddi/releases/download/v1.0.0/uveddi-1.0.0-x86_64-unknown-linux-gnu.tar.gz) |
| Linux (musl) | x86_64 | `uveddi-1.0.0-x86_64-unknown-linux-musl.tar.gz` | TBD | TBD | [Download](https://github.com/botzrDev/uveddi/releases/download/v1.0.0/uveddi-1.0.0-x86_64-unknown-linux-musl.tar.gz) |
| Linux (GNU) | ARM64 | `uveddi-1.0.0-aarch64-unknown-linux-gnu.tar.gz` | TBD | TBD | [Download](https://github.com/botzrDev/uveddi/releases/download/v1.0.0/uveddi-1.0.0-aarch64-unknown-linux-gnu.tar.gz) |

#### macOS

| Platform | Architecture | File | Size | SHA256 | Download |
|----------|--------------|------|------|--------|----------|
| macOS | x86_64 (Intel) | `uveddi-1.0.0-x86_64-apple-darwin.tar.gz` | TBD | TBD | [Download](https://github.com/botzrDev/uveddi/releases/download/v1.0.0/uveddi-1.0.0-x86_64-apple-darwin.tar.gz) |
| macOS | ARM64 (Apple Silicon) | `uveddi-1.0.0-aarch64-apple-darwin.tar.gz` | TBD | TBD | [Download](https://github.com/botzrDev/uveddi/releases/download/v1.0.0/uveddi-1.0.0-aarch64-apple-darwin.tar.gz) |

#### Windows

| Platform | Architecture | File | Size | SHA256 | Download |
|----------|--------------|------|------|--------|----------|
| Windows | x86_64 | `uveddi-1.0.0-x86_64-pc-windows-gnu.zip` | TBD | TBD | [Download](https://github.com/botzrDev/uveddi/releases/download/v1.0.0/uveddi-1.0.0-x86_64-pc-windows-gnu.zip) |

### Container Images

| Variant | Tag | Digest | Platforms | Size |
|---------|-----|--------|-----------|------|
| Distroless | `ghcr.io/botzrdev/uveddi:1.0.0` | `sha256:TBD` | linux/amd64 | TBD |
| Distroless Latest | `ghcr.io/botzrdev/uveddi:latest` | `sha256:TBD` | linux/amd64 | TBD |
| Alpine | `ghcr.io/botzrdev/uveddi:1.0.0-alpine` | `sha256:TBD` | linux/amd64 | TBD |
| Alpine Latest | `ghcr.io/botzrdev/uveddi:latest-alpine` | `sha256:TBD` | linux/amd64 | TBD |

**Pull Commands:**
```bash
# Distroless (recommended for production)
docker pull ghcr.io/botzrdev/uveddi:1.0.0

# Alpine (with shell for debugging)
docker pull ghcr.io/botzrdev/uveddi:1.0.0-alpine
```

### Source Code

| Type | File | Size | SHA256 | Download |
|------|------|------|--------|----------|
| Source tarball | `uveddi-1.0.0-source.tar.gz` | TBD | TBD | [Download](https://github.com/botzrDev/uveddi/archive/refs/tags/v1.0.0.tar.gz) |
| Source ZIP | `uveddi-1.0.0-source.zip` | TBD | TBD | [Download](https://github.com/botzrDev/uveddi/archive/refs/tags/v1.0.0.zip) |

## Checksums

All checksums are available in the release artifacts:

- **SHA256SUMS:** Contains SHA256 checksums for all binary releases
- **SHA512SUMS:** Contains SHA512 checksums for all binary releases
- **MD5SUMS:** Contains MD5 checksums (legacy support)

### Checksum Files

| File | Purpose | Signature |
|------|---------|-----------|
| `SHA256SUMS` | Primary checksum verification | `SHA256SUMS.asc` |
| `SHA512SUMS` | Secondary checksum verification | `SHA512SUMS.asc` |
| `MD5SUMS` | Legacy checksum verification | `MD5SUMS.asc` |

## Signatures

All artifacts are signed with the Uveddi Release GPG key:

**Key ID:** TBD  
**Fingerprint:** TBD  
**Key File:** `UVEDDI_RELEASE_KEY.asc`

### Signature Files

Each artifact has a corresponding detached GPG signature:
- Binary archives: `<filename>.asc`
- Checksum files: `SHA256SUMS.asc`, `SHA512SUMS.asc`

## Build Information

### Build Environment

- **Builder:** Uveddi Release Pipeline
- **Rust Version:** 1.75+
- **Build Target:** reproducible builds with musl for Linux
- **Build Features:** `cli-standard` (production-secure configuration)
- **Cargo Profile:** `release` (optimized)

### Build Configuration

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

### Reproducibility

All binaries are built with reproducible build settings:
- `SOURCE_DATE_EPOCH` set from git commit timestamp
- Static linking where possible (musl targets)
- Deterministic linker flags
- Stripped debug symbols

## Installation Instructions

### Quick Install (Linux/macOS)

```bash
# Using the installer script (recommended)
curl -fsSL https://uveddi.io/install.sh | bash

# Or with wget
wget -qO- https://uveddi.io/install.sh | bash
```

### Quick Install (Windows)

```powershell
# Using PowerShell
irm https://uveddi.io/install.ps1 | iex
```

### Manual Installation

#### Linux/macOS

1. Download the appropriate archive for your platform
2. Verify the checksum (see [Verification Instructions](#verification-instructions))
3. Extract the archive:
   ```bash
   tar -xzf uveddi-1.0.0-<platform>.tar.gz
   ```
4. Move the binary to your PATH:
   ```bash
   sudo mv uveddi-1.0.0-<platform>/uveddi /usr/local/bin/
   chmod +x /usr/local/bin/uveddi
   ```
5. Verify installation:
   ```bash
   uveddi --version
   ```

#### Windows

1. Download the Windows ZIP archive
2. Verify the checksum (see [Verification Instructions](#verification-instructions))
3. Extract the archive to a directory (e.g., `C:\Program Files\Uveddi`)
4. Add the directory to your PATH environment variable
5. Verify installation:
   ```powershell
   uveddi --version
   ```

### Container Installation

```bash
# Run directly
docker run --rm ghcr.io/botzrdev/uveddi:1.0.0 --version

# Or create an alias
echo 'alias uveddi="docker run --rm -v \$(pwd):/data ghcr.io/botzrdev/uveddi:1.0.0"' >> ~/.bashrc
source ~/.bashrc

# Use the alias
uveddi analyze /path/to/code
```

## Verification Instructions

### Verify Checksums

#### Linux/macOS

```bash
# Download the archive and checksum file
wget https://github.com/botzrDev/uveddi/releases/download/v1.0.0/uveddi-1.0.0-x86_64-unknown-linux-gnu.tar.gz
wget https://github.com/botzrDev/uveddi/releases/download/v1.0.0/SHA256SUMS

# Verify the checksum
sha256sum -c <<< "$(grep uveddi-1.0.0-x86_64-unknown-linux-gnu.tar.gz SHA256SUMS)"
```

#### Windows (PowerShell)

```powershell
# Download the archive
$url = "https://github.com/botzrDev/uveddi/releases/download/v1.0.0/uveddi-1.0.0-x86_64-pc-windows-gnu.zip"
Invoke-WebRequest -Uri $url -OutFile "uveddi-1.0.0-x86_64-pc-windows-gnu.zip"

# Download checksums
Invoke-WebRequest -Uri "https://github.com/botzrDev/uveddi/releases/download/v1.0.0/SHA256SUMS" -OutFile "SHA256SUMS"

# Verify
$expected = (Get-Content SHA256SUMS | Select-String "uveddi.*zip").Line.Split()[0]
$actual = (Get-FileHash -Algorithm SHA256 uveddi-1.0.0-x86_64-pc-windows-gnu.zip).Hash
if ($expected -eq $actual.ToLower()) {
    Write-Host "✓ Checksum verified!" -ForegroundColor Green
} else {
    Write-Host "✗ Checksum mismatch!" -ForegroundColor Red
}
```

### Verify Signatures

#### Import the Release Key

```bash
# Download the public key
wget https://github.com/botzrDev/uveddi/releases/download/v1.0.0/UVEDDI_RELEASE_KEY.asc

# Import the key
gpg --import UVEDDI_RELEASE_KEY.asc
```

#### Verify Checksum File Signature

```bash
# Download signature
wget https://github.com/botzrDev/uveddi/releases/download/v1.0.0/SHA256SUMS.asc

# Verify
gpg --verify SHA256SUMS.asc SHA256SUMS
```

#### Verify Individual Artifact

```bash
# Download artifact and signature
wget https://github.com/botzrDev/uveddi/releases/download/v1.0.0/uveddi-1.0.0-x86_64-unknown-linux-gnu.tar.gz
wget https://github.com/botzrDev/uveddi/releases/download/v1.0.0/uveddi-1.0.0-x86_64-unknown-linux-gnu.tar.gz.asc

# Verify
gpg --verify uveddi-1.0.0-x86_64-unknown-linux-gnu.tar.gz.asc uveddi-1.0.0-x86_64-unknown-linux-gnu.tar.gz
```

### Complete Verification

For complete verification (checksums + signatures):

```bash
# Download all necessary files
wget https://github.com/botzrDev/uveddi/releases/download/v1.0.0/uveddi-1.0.0-x86_64-unknown-linux-gnu.tar.gz
wget https://github.com/botzrDev/uveddi/releases/download/v1.0.0/SHA256SUMS
wget https://github.com/botzrDev/uveddi/releases/download/v1.0.0/SHA256SUMS.asc
wget https://github.com/botzrDev/uveddi/releases/download/v1.0.0/UVEDDI_RELEASE_KEY.asc

# Import key
gpg --import UVEDDI_RELEASE_KEY.asc

# Verify checksum file signature
gpg --verify SHA256SUMS.asc SHA256SUMS

# Verify artifact checksum
sha256sum -c <<< "$(grep uveddi-1.0.0-x86_64-unknown-linux-gnu.tar.gz SHA256SUMS)"
```

## Upgrade Instructions

### From Previous Versions

1. Backup your configuration:
   ```bash
   cp ~/.config/uveddi/config.toml ~/.config/uveddi/config.toml.backup
   ```

2. Install the new version (same as installation steps above)

3. Run migration (if needed):
   ```bash
   uveddi migrate --from 0.x.x --to 1.0.0
   ```

4. Verify the upgrade:
   ```bash
   uveddi --version
   uveddi check-config
   ```

### Rollback

To rollback to a previous version:

1. Download the previous version from releases
2. Install using the same process
3. Restore configuration backup if needed

## Uninstallation

### Linux/macOS

```bash
# Using installer script
curl -fsSL https://uveddi.io/install.sh | bash -s uninstall

# Manual removal
rm -f /usr/local/bin/uveddi
rm -rf ~/.config/uveddi
```

### Windows

```powershell
# Using installer script
irm https://uveddi.io/install.ps1 | iex -Uninstall

# Manual removal
Remove-Item "C:\Program Files\Uveddi\uveddi.exe"
Remove-Item -Recurse "$env:APPDATA\uveddi"
```

### Container

```bash
# Remove images
docker rmi ghcr.io/botzrdev/uveddi:1.0.0
docker rmi ghcr.io/botzrdev/uveddi:latest
```

## Storage Locations

### Primary Distribution

- **GitHub Releases:** https://github.com/botzrDev/uveddi/releases/tag/v1.0.0
- **Container Registry:** ghcr.io/botzrdev/uveddi

### Mirrors (TBD)

Mirror locations will be announced after release.

### CDN (TBD)

CDN endpoints for faster downloads will be configured post-release.

## Security

### Vulnerability Scanning

All container images have been scanned with:
- **Trivy:** No CRITICAL vulnerabilities
- **Grype:** No CRITICAL vulnerabilities

Scan reports available in: `reports/security/`

### Security Advisories

- **Security Policy:** https://github.com/botzrDev/uveddi/security/policy
- **Security Contact:** security@uveddi.io
- **Report Vulnerability:** https://github.com/botzrDev/uveddi/security/advisories/new

### Known Issues

No security issues at release time.

## Support

- **Documentation:** https://uveddi.io/docs
- **Issue Tracker:** https://github.com/botzrDev/uveddi/issues
- **Discussions:** https://github.com/botzrDev/uveddi/discussions
- **Email Support:** support@uveddi.io

## License

Uveddi is released under the MIT License. See LICENSE file for details.

## Changelog

For detailed changes in this release, see:
- [CHANGELOG.md](../../CHANGELOG.md)
- [Release Notes](../../RELEASE_NOTES_v1.0.0.md)

## Acknowledgments

Special thanks to all contributors who made this release possible.

---

**Document Version:** 1.0.0  
**Last Updated:** TBD  
**Maintained By:** Uveddi Release Team
