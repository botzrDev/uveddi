# Uveddi Binary Signing & Distribution Checklist

**Version:** 1.0  
**Date:** 2025-10-09  
**Target Release:** Uveddi 1.0.0  
**Owner:** Release Engineering / Austin Green

## 📋 Overview

This document provides step-by-step procedures for signing Uveddi binaries across Linux, macOS, and Windows platforms. Binary signing ensures authenticity and integrity, allowing users to verify that binaries have not been tampered with.

---

## 🎯 Signing Requirements Summary

| Platform | Signing Method | Certificate Type | Status |
|----------|----------------|------------------|--------|
| **Linux** | GPG/PGP | Personal/Company GPG Key | ⏳ Pending |
| **macOS** | codesign | Apple Developer ID | ⏳ Pending |
| **Windows** | signtool | Authenticode Certificate | ⏳ Pending |

**Current Status:** All certificates/keys need to be obtained before 1.0.0 release.

---

## 🐧 Linux: GPG Signing

### Prerequisites

- [ ] GPG installed (`gpg --version`)
- [ ] GPG keypair generated (2048-bit RSA minimum, 4096-bit recommended)
- [ ] Key uploaded to public keyservers
- [ ] Key fingerprint published on website/documentation

### 1. Generate GPG Key (One-Time Setup)

**If you don't have a GPG key:**

```bash
# Generate new GPG key
gpg --full-generate-key --expert

# Select options:
# - RSA and RSA (default)
# - 4096 bits
# - Key does not expire (or set expiration as preferred)
# - Real name: Uveddi Release Team (or your name)
# - Email: security@uveddi.com
# - Comment: Uveddi binary signing key

# List keys to verify creation
gpg --list-secret-keys --keyid-format LONG

# Note the key ID (e.g., 3AA5C34371567BD2)
```

### 2. Export and Publish Public Key

```bash
# Export public key
gpg --armor --export security@uveddi.com > uveddi-gpg-public.asc

# Publish to keyservers
gpg --send-keys [KEY_ID]

# Verify publication
gpg --keyserver keys.openpgp.org --search-keys security@uveddi.com
```

**📍 Action Required:**
- [ ] Upload `uveddi-gpg-public.asc` to GitHub repository
- [ ] Publish key fingerprint in README.md and documentation
- [ ] Add key to website (https://uveddi.com/security-gpg.asc)

### 3. Build Binary

```bash
# Clean build environment
cargo clean

# Build release binary
cargo build --release --features cli-standard

# Binary location
ls -lh target/release/uveddi
```

### 4. Sign Binary

```bash
# Create detached signature
gpg --detach-sign --armor --output target/release/uveddi.asc target/release/uveddi

# Create checksum file
sha256sum target/release/uveddi > target/release/uveddi.sha256

# Sign checksum file
gpg --detach-sign --armor target/release/uveddi.sha256
```

### 5. Verification (Testing)

```bash
# Verify signature
gpg --verify target/release/uveddi.asc target/release/uveddi

# Expected output:
# gpg: Signature made [date]
# gpg: using RSA key [KEY_ID]
# gpg: Good signature from "Uveddi Release Team"

# Verify checksum
sha256sum --check uveddi.sha256
```

### 6. Create Distribution Package

```bash
# Create versioned directory
VERSION="1.0.0"
ARCH="x86_64-unknown-linux-gnu"
PKG_NAME="uveddi-${VERSION}-${ARCH}"

mkdir -p "dist/${PKG_NAME}"

# Copy files
cp target/release/uveddi "dist/${PKG_NAME}/"
cp target/release/uveddi.asc "dist/${PKG_NAME}/"
cp target/release/uveddi.sha256 "dist/${PKG_NAME}/"
cp target/release/uveddi.sha256.asc "dist/${PKG_NAME}/"
cp LICENSE "dist/${PKG_NAME}/"
cp README.md "dist/${PKG_NAME}/"

# Create tarball
cd dist
tar czf "${PKG_NAME}.tar.gz" "${PKG_NAME}"

# Sign tarball
gpg --detach-sign --armor "${PKG_NAME}.tar.gz"

# Generate final checksums
sha256sum "${PKG_NAME}.tar.gz" > "${PKG_NAME}.tar.gz.sha256"
```

### 7. Distribution Artifacts

Final Linux distribution includes:
- `uveddi-1.0.0-x86_64-unknown-linux-gnu.tar.gz` (binary package)
- `uveddi-1.0.0-x86_64-unknown-linux-gnu.tar.gz.asc` (GPG signature)
- `uveddi-1.0.0-x86_64-unknown-linux-gnu.tar.gz.sha256` (checksum)

---

## 🍎 macOS: Code Signing

### Prerequisites

- [ ] Apple Developer Account ($99/year)
- [ ] Developer ID Application certificate
- [ ] Xcode Command Line Tools installed
- [ ] Certificate and private key in Keychain Access

### 1. Obtain Apple Developer ID Certificate (One-Time)

**Steps:**

1. Log in to [Apple Developer Portal](https://developer.apple.com/account)
2. Navigate to Certificates, IDs & Profiles
3. Create new certificate → Developer ID Application
4. Generate Certificate Signing Request (CSR) from Keychain Access:
   - Keychain Access → Certificate Assistant → Request a Certificate from a Certificate Authority
   - Save CSR to disk
5. Upload CSR to Apple Developer Portal
6. Download certificate (.cer file)
7. Double-click to install in Keychain Access

**Verify Certificate Installation:**

```bash
# List signing identities
security find-identity -v -p codesigning

# Expected output includes:
# 1) [HASH] "Developer ID Application: Your Name (TEAM_ID)"
```

**📍 Action Required:**
- [ ] Apple Developer Account active
- [ ] Developer ID Application certificate installed
- [ ] Certificate ID documented: `_______________________`

### 2. Build Binary

```bash
# Build for macOS
cargo build --release --target x86_64-apple-darwin --features cli-standard

# Build for Apple Silicon (if applicable)
cargo build --release --target aarch64-apple-darwin --features cli-standard
```

### 3. Sign Binary

```bash
# Sign x86_64 binary
codesign --sign "Developer ID Application: Your Name (TEAM_ID)" \
    --timestamp \
    --options runtime \
    target/x86_64-apple-darwin/release/uveddi

# Sign aarch64 binary (if applicable)
codesign --sign "Developer ID Application: Your Name (TEAM_ID)" \
    --timestamp \
    --options runtime \
    target/aarch64-apple-darwin/release/uveddi
```

**Signing Options Explained:**
- `--sign`: Specifies signing identity
- `--timestamp`: Adds secure timestamp (required for Gatekeeper)
- `--options runtime`: Enables hardened runtime (required for notarization)

### 4. Verify Signature

```bash
# Verify code signature
codesign --verify --verbose target/x86_64-apple-darwin/release/uveddi

# Check signature details
codesign --display --verbose=4 target/x86_64-apple-darwin/release/uveddi

# Test Gatekeeper acceptance
spctl --assess --verbose target/x86_64-apple-darwin/release/uveddi
```

### 5. Notarization (Recommended for Distribution)

**Why Notarize:** Prevents "unidentified developer" warnings on macOS 10.15+

```bash
# Create app bundle or zip for notarization
ditto -c -k --keepParent target/x86_64-apple-darwin/release/uveddi uveddi.zip

# Submit for notarization
xcrun notarytool submit uveddi.zip \
    --apple-id "your-apple-id@example.com" \
    --team-id "TEAM_ID" \
    --password "app-specific-password" \
    --wait

# If successful, staple the notarization ticket
xcrun stapler staple target/x86_64-apple-darwin/release/uveddi

# Verify stapling
xcrun stapler validate target/x86_64-apple-darwin/release/uveddi
```

**📍 Note:** App-specific password required (generate in Apple ID account settings)

### 6. Create Distribution Package

```bash
VERSION="1.0.0"

# x86_64 (Intel)
PKG_NAME="uveddi-${VERSION}-x86_64-apple-darwin"
mkdir -p "dist/${PKG_NAME}"
cp target/x86_64-apple-darwin/release/uveddi "dist/${PKG_NAME}/"
cp LICENSE README.md "dist/${PKG_NAME}/"
cd dist && tar czf "${PKG_NAME}.tar.gz" "${PKG_NAME}" && cd ..

# aarch64 (Apple Silicon)
PKG_NAME="uveddi-${VERSION}-aarch64-apple-darwin"
mkdir -p "dist/${PKG_NAME}"
cp target/aarch64-apple-darwin/release/uveddi "dist/${PKG_NAME}/"
cp LICENSE README.md "dist/${PKG_NAME}/"
cd dist && tar czf "${PKG_NAME}.tar.gz" "${PKG_NAME}" && cd ..

# Generate checksums
cd dist
shasum -a 256 uveddi-*.tar.gz > checksums-macos.txt
```

### 7. Distribution Artifacts

- `uveddi-1.0.0-x86_64-apple-darwin.tar.gz` (Intel binary)
- `uveddi-1.0.0-aarch64-apple-darwin.tar.gz` (Apple Silicon binary)
- `checksums-macos.txt` (SHA256 checksums)

---

## 🪟 Windows: Authenticode Signing

### Prerequisites

- [ ] Code signing certificate (from DigiCert, Sectigo, etc.)
- [ ] Certificate installed in Windows Certificate Store
- [ ] Windows SDK installed (includes signtool.exe)
- [ ] Timestamp server URL configured

### 1. Obtain Code Signing Certificate (One-Time)

**Options:**

1. **Standard Code Signing Certificate** ($100-400/year)
   - Sectigo, DigiCert, GlobalSign, etc.
   - Requires business verification
   - Delivered as PFX/P12 file

2. **EV Code Signing Certificate** ($300-600/year)
   - Extended Validation
   - Hardware token (USB) required
   - No SmartScreen warnings from day one

**Installation:**

```powershell
# Import PFX certificate
certutil -f -user -p "certificate-password" -importpfx "path\to\certificate.pfx"

# Verify installation
certutil -user -store My
```

**📍 Action Required:**
- [ ] Certificate obtained from trusted CA
- [ ] Certificate installed in Current User → Personal → Certificates
- [ ] Certificate thumbprint documented: `_______________________`

### 2. Build Binary

```powershell
# Build for Windows
cargo build --release --target x86_64-pc-windows-msvc --features cli-standard

# Binary location
dir target\x86_64-pc-windows-msvc\release\uveddi.exe
```

### 3. Sign Binary

```powershell
# Locate signtool (part of Windows SDK)
# Typical location: C:\Program Files (x86)\Windows Kits\10\bin\<version>\x64\signtool.exe

# Sign with timestamp
signtool sign /a /t http://timestamp.digicert.com /fd sha256 /v target\x86_64-pc-windows-msvc\release\uveddi.exe

# Alternative: Specify certificate thumbprint explicitly
signtool sign /sha1 CERT_THUMBPRINT /t http://timestamp.digicert.com /fd sha256 /v uveddi.exe
```

**Signing Options Explained:**
- `/a`: Automatically select certificate
- `/sha1 [thumbprint]`: Specify certificate by thumbprint
- `/t [url]`: Timestamp server URL
- `/fd sha256`: File digest algorithm (SHA-256)
- `/v`: Verbose output

**Timestamp Servers:**
- DigiCert: `http://timestamp.digicert.com`
- Sectigo: `http://timestamp.sectigo.com`
- GlobalSign: `http://timestamp.globalsign.com`

### 4. Verify Signature

```powershell
# Verify signature
signtool verify /pa /v target\x86_64-pc-windows-msvc\release\uveddi.exe

# View signature details
signtool verify /pa /v /all target\x86_64-pc-windows-msvc\release\uveddi.exe

# Check in Windows Explorer
# Right-click uveddi.exe → Properties → Digital Signatures tab
```

### 5. Create Distribution Package

```powershell
$VERSION = "1.0.0"
$PKG_NAME = "uveddi-$VERSION-x86_64-pc-windows-msvc"

# Create directory structure
New-Item -ItemType Directory -Path "dist\$PKG_NAME" -Force

# Copy files
Copy-Item "target\x86_64-pc-windows-msvc\release\uveddi.exe" "dist\$PKG_NAME\"
Copy-Item "LICENSE" "dist\$PKG_NAME\"
Copy-Item "README.md" "dist\$PKG_NAME\"

# Create ZIP archive
Compress-Archive -Path "dist\$PKG_NAME" -DestinationPath "dist\$PKG_NAME.zip"

# Generate checksum
Get-FileHash "dist\$PKG_NAME.zip" -Algorithm SHA256 | Format-List > "dist\$PKG_NAME.zip.sha256"
```

### 6. Distribution Artifacts

- `uveddi-1.0.0-x86_64-pc-windows-msvc.zip` (signed binary package)
- `uveddi-1.0.0-x86_64-pc-windows-msvc.zip.sha256` (checksum)

---

## 📦 Multi-Platform Release Checklist

### Pre-Release Preparation

- [ ] All signing certificates/keys obtained and documented
- [ ] Signing environment tested (dry run with previous version or test binary)
- [ ] CI/CD pipeline configured for automated signing (optional)
- [ ] Release notes prepared
- [ ] Changelog updated

### Build & Sign Process

#### Linux
- [ ] Build binary: `cargo build --release --features cli-standard`
- [ ] Sign with GPG: `gpg --detach-sign --armor uveddi`
- [ ] Generate checksums: `sha256sum uveddi > uveddi.sha256`
- [ ] Create tarball: `tar czf uveddi-1.0.0-linux.tar.gz uveddi uveddi.asc uveddi.sha256`
- [ ] Verify signature: `gpg --verify uveddi.asc uveddi`

#### macOS (Intel)
- [ ] Build binary: `cargo build --release --target x86_64-apple-darwin`
- [ ] Sign with codesign: `codesign --sign "Developer ID" --timestamp uveddi`
- [ ] Notarize (optional but recommended): `xcrun notarytool submit`
- [ ] Staple notarization: `xcrun stapler staple uveddi`
- [ ] Create tarball: `tar czf uveddi-1.0.0-macos-x86_64.tar.gz uveddi`
- [ ] Verify: `codesign --verify --verbose uveddi`

#### macOS (Apple Silicon)
- [ ] Build binary: `cargo build --release --target aarch64-apple-darwin`
- [ ] Sign with codesign: `codesign --sign "Developer ID" --timestamp uveddi`
- [ ] Notarize (optional but recommended)
- [ ] Staple notarization
- [ ] Create tarball: `tar czf uveddi-1.0.0-macos-aarch64.tar.gz uveddi`
- [ ] Verify: `codesign --verify --verbose uveddi`

#### Windows
- [ ] Build binary: `cargo build --release --target x86_64-pc-windows-msvc`
- [ ] Sign with signtool: `signtool sign /a /t http://timestamp.digicert.com uveddi.exe`
- [ ] Verify: `signtool verify /pa /v uveddi.exe`
- [ ] Create ZIP: `Compress-Archive uveddi.exe uveddi-1.0.0-windows.zip`
- [ ] Test on clean Windows VM

### Distribution Verification

- [ ] All binaries signed and verified
- [ ] Checksums generated for all packages
- [ ] Upload to GitHub Releases with signatures
- [ ] Update download links in documentation
- [ ] Test installation from published artifacts

### Documentation

- [ ] README.md includes signature verification instructions
- [ ] GPG public key published (Linux)
- [ ] Certificate fingerprints documented (macOS/Windows)
- [ ] Security policy references signing procedures

---

## 🔐 Security Best Practices

### Certificate/Key Management

**Storage:**
- 🔒 **Private Keys:** Store securely (encrypted, hardware token, or cloud HSM)
- 🔒 **Certificates:** Back up securely (encrypted storage)
- 🔒 **Passwords:** Use password manager or secrets management system
- 🔒 **Access Control:** Limit signing authority to release manager(s)

**Key Rotation:**
- Review certificates annually
- Renew before expiration (90 days notice)
- Plan for key rotation (transition period)
- Document key fingerprints/thumbprints

**Backup & Recovery:**
- Export private keys to secure offline storage
- Document recovery procedures
- Test recovery process annually
- Maintain certificate chain documentation

### Signing Environment

**Isolation:**
- Use dedicated build/signing environment
- Clean state for each release build
- No extraneous tools or software
- Minimal network access during signing

**Verification:**
- Always verify signatures after signing
- Test on clean systems (VMs)
- Check timestamp validity
- Confirm no warnings/errors

**Audit Trail:**
- Log all signing operations
- Record certificate details for each release
- Track which binaries were signed with which certificates
- Maintain release artifact inventory

---

## 🚀 CI/CD Integration (Optional)

### GitHub Actions Example

```yaml
name: Release & Sign Binaries

on:
  release:
    types: [created]

jobs:
  build-sign-linux:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Build
        run: cargo build --release --features cli-standard
        
      - name: Import GPG Key
        run: |
          echo "${{ secrets.GPG_PRIVATE_KEY }}" | gpg --import
          gpg --list-secret-keys
          
      - name: Sign Binary
        run: |
          gpg --detach-sign --armor target/release/uveddi
          sha256sum target/release/uveddi > target/release/uveddi.sha256
          
      - name: Upload Artifacts
        uses: actions/upload-artifact@v4
        with:
          name: uveddi-linux
          path: |
            target/release/uveddi
            target/release/uveddi.asc
            target/release/uveddi.sha256

  build-sign-macos:
    runs-on: macos-latest
    steps:
      # Similar steps for macOS signing
      # Uses secrets.APPLE_DEVELOPER_ID, secrets.APPLE_ID, etc.

  build-sign-windows:
    runs-on: windows-latest
    steps:
      # Similar steps for Windows signing
      # Uses secrets.WINDOWS_CERT_PASSWORD, certificate from Azure Key Vault
```

**Secrets Required:**
- `GPG_PRIVATE_KEY`: Exported GPG private key (Linux)
- `APPLE_DEVELOPER_ID`: Apple Developer account credentials
- `WINDOWS_CERT_PASSWORD`: Code signing certificate password

---

## 📋 Post-Release Verification

### User Verification Instructions

**Linux (GPG):**
```bash
# Download public key
curl https://uveddi.com/security-gpg.asc | gpg --import

# Verify binary
gpg --verify uveddi.asc uveddi

# Verify checksum
sha256sum --check uveddi.sha256
```

**macOS:**
```bash
# Verify code signature
codesign --verify --verbose uveddi

# Check notarization (if applicable)
spctl --assess --verbose uveddi
```

**Windows:**
```powershell
# Verify signature
signtool verify /pa /v uveddi.exe

# Or: Right-click → Properties → Digital Signatures
```

---

## 🆘 Troubleshooting

### Common Issues

**Linux GPG:**
- **"Secret key not available"**: Wrong key ID or key not imported
- **"Can't check signature: No public key"**: User needs to import your public key
- **Solution:** Publish key to keyservers and document fingerprint clearly

**macOS Codesign:**
- **"no identity found"**: Certificate not installed in Keychain
- **"errSecInternalComponent"**: Try `security unlock-keychain` before signing
- **Gatekeeper rejection**: Missing `--timestamp` or `--options runtime`

**Windows Signtool:**
- **"No certificates were found"**: Certificate not in correct store location
- **Timestamp failure**: Use alternative timestamp server
- **"Invalid digital signature"**: Certificate expired or timestamp missing

---

## 📚 References

- [Apple Code Signing Guide](https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution)
- [Microsoft Authenticode Documentation](https://learn.microsoft.com/en-us/windows/win32/seccrypto/cryptography-tools)
- [GPG/PGP Best Practices](https://riseup.net/en/security/message-security/openpgp/best-practices)
- [Rust Cross-Compilation Guide](https://rust-lang.github.io/rustup/cross-compilation.html)

---

## ✅ Sign-Off

**Prepared By:** GitHub Copilot (Assignment A4)  
**Date:** 2025-10-09  
**Review Required:** Release Manager, Legal (certificate procurement)  
**Next Update:** After certificate acquisition or signing process changes

**Certificate Status Summary:**

| Platform | Status | Owner | Due Date |
|----------|--------|-------|----------|
| Linux GPG | ⏳ Pending | Austin Green | 2025-11-01 |
| macOS Developer ID | ⏳ Pending | Austin Green | 2025-11-01 |
| Windows Authenticode | ⏳ Pending | Austin Green / Ops | 2025-11-01 |

**Next Action:** Coordinate with legal/ops to obtain signing certificates by target dates.

---

**Document Control:**
- Version: 1.0
- Classification: Internal/Confidential (contains certificate details)
- Distribution: Release engineering, security team
- Review Frequency: Before each major release
