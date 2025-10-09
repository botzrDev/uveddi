#!/usr/bin/env bash
# Uveddi Artifact Signing and Verification Script
#
# Usage:
#   ./scripts/sign_artifacts.sh sign      # Sign all artifacts in dist/
#   ./scripts/sign_artifacts.sh verify    # Verify all signatures
#   ./scripts/sign_artifacts.sh checksums # Generate checksums only
#
# Environment Variables:
#   UVEDDI_GPG_KEY: GPG key ID for signing (default: auto-detect)
#   UVEDDI_DIST_DIR: Distribution directory (default: ./dist)

set -euo pipefail

# ============================================================================
# Configuration
# ============================================================================

readonly DIST_DIR="${UVEDDI_DIST_DIR:-$(pwd)/dist}"
readonly ARCHIVES_DIR="${DIST_DIR}/archives"
readonly CHECKSUMS_DIR="${DIST_DIR}/checksums"
readonly SIGNATURES_DIR="${DIST_DIR}/signatures"
readonly GPG_KEY="${UVEDDI_GPG_KEY:-}"

# Colors
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly NC='\033[0m'

# ============================================================================
# Logging
# ============================================================================

log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*" >&2
}

log_header() {
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}$*${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
}

# ============================================================================
# Checksum Functions
# ============================================================================

generate_checksums() {
    log_header "Generating Checksums"
    
    if [ ! -d "${ARCHIVES_DIR}" ]; then
        log_error "Archives directory not found: ${ARCHIVES_DIR}"
        return 1
    fi
    
    mkdir -p "${CHECKSUMS_DIR}"
    cd "${ARCHIVES_DIR}"
    
    # Count files to process
    local file_count=0
    for file in *.tar.gz *.zip 2>/dev/null; do
        [ -e "$file" ] || continue
        ((file_count++))
    done
    
    if [ $file_count -eq 0 ]; then
        log_warning "No archive files found to checksum"
        return 0
    fi
    
    log_info "Processing ${file_count} files"
    
    # Generate SHA256 checksums
    if command -v sha256sum &> /dev/null; then
        log_info "Generating SHA256 checksums"
        sha256sum *.tar.gz *.zip 2>/dev/null | tee "${CHECKSUMS_DIR}/SHA256SUMS" | while read sum file; do
            log_success "SHA256: ${file}"
        done
        
        # Also create individual checksum files
        for file in *.tar.gz *.zip 2>/dev/null; do
            [ -e "$file" ] || continue
            sha256sum "$file" > "${CHECKSUMS_DIR}/${file}.sha256"
        done
    else
        log_error "sha256sum not found"
        return 1
    fi
    
    # Generate SHA512 checksums (optional but recommended)
    if command -v sha512sum &> /dev/null; then
        log_info "Generating SHA512 checksums"
        sha512sum *.tar.gz *.zip 2>/dev/null | tee "${CHECKSUMS_DIR}/SHA512SUMS" | while read sum file; do
            log_success "SHA512: ${file}"
        done
        
        # Also create individual checksum files
        for file in *.tar.gz *.zip 2>/dev/null; do
            [ -e "$file" ] || continue
            sha512sum "$file" > "${CHECKSUMS_DIR}/${file}.sha512"
        done
    else
        log_warning "sha512sum not found, skipping SHA512"
    fi
    
    # Generate MD5 checksums (legacy support)
    if command -v md5sum &> /dev/null; then
        log_info "Generating MD5 checksums (legacy)"
        md5sum *.tar.gz *.zip 2>/dev/null > "${CHECKSUMS_DIR}/MD5SUMS"
    fi
    
    log_success "Checksums generated in ${CHECKSUMS_DIR}"
    
    # Display summary
    echo ""
    log_info "Checksum files:"
    ls -lh "${CHECKSUMS_DIR}" | tail -n +2 | awk '{printf "  %-15s %s\n", $5, $9}'
}

verify_checksums() {
    log_header "Verifying Checksums"
    
    if [ ! -d "${CHECKSUMS_DIR}" ]; then
        log_error "Checksums directory not found: ${CHECKSUMS_DIR}"
        return 1
    fi
    
    cd "${ARCHIVES_DIR}"
    
    local all_valid=true
    
    # Verify SHA256
    if [ -f "${CHECKSUMS_DIR}/SHA256SUMS" ]; then
        log_info "Verifying SHA256 checksums"
        if sha256sum -c "${CHECKSUMS_DIR}/SHA256SUMS" 2>&1 | while read line; do
            if echo "$line" | grep -q "OK$"; then
                log_success "$(echo "$line" | cut -d: -f1)"
            elif echo "$line" | grep -q "FAILED"; then
                log_error "$(echo "$line" | cut -d: -f1)"
                all_valid=false
            fi
        done; then
            log_success "SHA256 verification passed"
        else
            log_error "SHA256 verification failed"
            all_valid=false
        fi
    fi
    
    # Verify SHA512
    if [ -f "${CHECKSUMS_DIR}/SHA512SUMS" ]; then
        log_info "Verifying SHA512 checksums"
        if sha512sum -c "${CHECKSUMS_DIR}/SHA512SUMS" 2>&1 | while read line; do
            if echo "$line" | grep -q "OK$"; then
                log_success "$(echo "$line" | cut -d: -f1)"
            elif echo "$line" | grep -q "FAILED"; then
                log_error "$(echo "$line" | cut -d: -f1)"
                all_valid=false
            fi
        done; then
            log_success "SHA512 verification passed"
        else
            log_error "SHA512 verification failed"
            all_valid=false
        fi
    fi
    
    if [ "$all_valid" = true ]; then
        log_success "All checksums verified successfully"
        return 0
    else
        log_error "Some checksums failed verification"
        return 1
    fi
}

# ============================================================================
# GPG Signing Functions
# ============================================================================

get_gpg_key() {
    local key="${GPG_KEY}"
    
    if [ -z "${key}" ]; then
        # Try to auto-detect key
        key="$(gpg --list-secret-keys --keyid-format LONG 2>/dev/null | grep -m1 'sec' | awk '{print $2}' | cut -d'/' -f2 || true)"
    fi
    
    if [ -z "${key}" ]; then
        log_error "No GPG key found"
        log_info "Available keys:"
        gpg --list-secret-keys --keyid-format LONG 2>/dev/null || true
        log_info ""
        log_info "Set UVEDDI_GPG_KEY environment variable or create a GPG key:"
        log_info "  gpg --full-generate-key"
        return 1
    fi
    
    echo "${key}"
}

sign_artifacts() {
    log_header "Signing Artifacts"
    
    if ! command -v gpg &> /dev/null; then
        log_error "GPG not found. Please install GnuPG."
        return 1
    fi
    
    local gpg_key="$(get_gpg_key)" || return 1
    
    log_info "Using GPG key: ${gpg_key}"
    
    # Verify key exists and get details
    if ! gpg --list-keys "${gpg_key}" &> /dev/null; then
        log_error "GPG key ${gpg_key} not found"
        return 1
    fi
    
    log_info "Key details:"
    gpg --list-keys "${gpg_key}" 2>/dev/null | grep -A2 "^pub" || true
    echo ""
    
    mkdir -p "${SIGNATURES_DIR}"
    
    # Sign archives
    cd "${ARCHIVES_DIR}"
    for file in *.tar.gz *.zip 2>/dev/null; do
        [ -e "$file" ] || continue
        
        log_info "Signing: ${file}"
        
        # Create detached ASCII-armored signature
        gpg --detach-sign --armor --local-user "${gpg_key}" --output "${SIGNATURES_DIR}/${file}.asc" "${file}"
        
        log_success "Signed: ${file} -> ${file}.asc"
    done
    
    # Sign checksum files
    cd "${CHECKSUMS_DIR}"
    for file in SHA256SUMS SHA512SUMS MD5SUMS 2>/dev/null; do
        [ -e "$file" ] || continue
        
        log_info "Signing: ${file}"
        gpg --detach-sign --armor --local-user "${gpg_key}" --output "${SIGNATURES_DIR}/${file}.asc" "${file}"
        log_success "Signed: ${file} -> ${file}.asc"
    done
    
    log_success "All artifacts signed"
    
    # Display summary
    echo ""
    log_info "Signature files:"
    ls -lh "${SIGNATURES_DIR}" | tail -n +2 | awk '{printf "  %-15s %s\n", $5, $9}'
    
    # Export public key for verification
    local pubkey_file="${DIST_DIR}/UVEDDI_RELEASE_KEY.asc"
    gpg --armor --export "${gpg_key}" > "${pubkey_file}"
    log_success "Public key exported: ${pubkey_file}"
    
    echo ""
    log_info "To verify signatures, users should:"
    echo "  1. Import the public key: gpg --import UVEDDI_RELEASE_KEY.asc"
    echo "  2. Verify a file: gpg --verify <file>.asc <file>"
}

verify_signatures() {
    log_header "Verifying Signatures"
    
    if ! command -v gpg &> /dev/null; then
        log_error "GPG not found"
        return 1
    fi
    
    if [ ! -d "${SIGNATURES_DIR}" ]; then
        log_error "Signatures directory not found: ${SIGNATURES_DIR}"
        return 1
    fi
    
    local all_valid=true
    
    # Verify archives
    cd "${ARCHIVES_DIR}"
    for sig in "${SIGNATURES_DIR}"/*.tar.gz.asc "${SIGNATURES_DIR}"/*.zip.asc 2>/dev/null; do
        [ -e "$sig" ] || continue
        
        local file="$(basename "${sig}" .asc)"
        
        if [ ! -f "${file}" ]; then
            log_warning "File not found for signature: ${file}"
            continue
        fi
        
        log_info "Verifying: ${file}"
        
        if gpg --verify "${sig}" "${file}" 2>&1 | grep -q "Good signature"; then
            log_success "Valid signature: ${file}"
        else
            log_error "Invalid signature: ${file}"
            all_valid=false
        fi
    done
    
    # Verify checksums
    cd "${CHECKSUMS_DIR}"
    for sig in "${SIGNATURES_DIR}"/SHA*.asc "${SIGNATURES_DIR}"/MD5SUMS.asc 2>/dev/null; do
        [ -e "$sig" ] || continue
        
        local file="$(basename "${sig}" .asc)"
        
        if [ ! -f "${file}" ]; then
            log_warning "File not found for signature: ${file}"
            continue
        fi
        
        log_info "Verifying: ${file}"
        
        if gpg --verify "${sig}" "${file}" 2>&1 | grep -q "Good signature"; then
            log_success "Valid signature: ${file}"
        else
            log_error "Invalid signature: ${file}"
            all_valid=false
        fi
    done
    
    if [ "$all_valid" = true ]; then
        log_success "All signatures verified successfully"
        return 0
    else
        log_error "Some signatures failed verification"
        return 1
    fi
}

# ============================================================================
# Verification Document
# ============================================================================

generate_verification_doc() {
    log_header "Generating Verification Documentation"
    
    local doc="${DIST_DIR}/VERIFICATION.md"
    
    cat > "${doc}" << 'EOF'
# Uveddi Release Artifact Verification Guide

This document explains how to verify the authenticity and integrity of Uveddi release artifacts.

## Quick Start

```bash
# 1. Import the Uveddi release signing key
gpg --import UVEDDI_RELEASE_KEY.asc

# 2. Verify checksum file signature
gpg --verify SHA256SUMS.asc SHA256SUMS

# 3. Verify artifact checksums
sha256sum -c SHA256SUMS

# 4. Verify individual artifact signature (optional)
gpg --verify uveddi-1.0.0-x86_64-unknown-linux-gnu.tar.gz.asc uveddi-1.0.0-x86_64-unknown-linux-gnu.tar.gz
```

## Detailed Verification Steps

### Step 1: Import the Signing Key

```bash
gpg --import UVEDDI_RELEASE_KEY.asc
```

Expected output:
```
gpg: key <KEY_ID>: public key "Uveddi Release <release@uveddi.io>" imported
gpg: Total number processed: 1
gpg:               imported: 1
```

### Step 2: Verify Checksum File Signature

This verifies that the checksum file itself hasn't been tampered with:

```bash
gpg --verify SHA256SUMS.asc SHA256SUMS
```

Expected output:
```
gpg: Signature made <DATE>
gpg:                using RSA key <KEY_ID>
gpg: Good signature from "Uveddi Release <release@uveddi.io>"
```

### Step 3: Verify File Checksums

Once you've verified the checksum file is authentic, verify your downloaded files:

```bash
sha256sum -c SHA256SUMS
```

Or for a specific file:

```bash
sha256sum -c <<< "$(grep uveddi-1.0.0-x86_64-unknown-linux-gnu.tar.gz SHA256SUMS)"
```

### Step 4: Verify Individual Artifact Signatures (Optional)

Each artifact also has its own detached signature:

```bash
gpg --verify <filename>.asc <filename>
```

Example:
```bash
gpg --verify uveddi-1.0.0-x86_64-unknown-linux-gnu.tar.gz.asc uveddi-1.0.0-x86_64-unknown-linux-gnu.tar.gz
```

## Verification on Different Platforms

### Linux

```bash
# SHA256
sha256sum -c SHA256SUMS

# SHA512
sha512sum -c SHA512SUMS
```

### macOS

```bash
# SHA256
shasum -a 256 -c SHA256SUMS

# SHA512
shasum -a 512 -c SHA512SUMS
```

### Windows (PowerShell)

```powershell
# Verify SHA256 checksum
$expectedHash = (Get-Content SHA256SUMS | Select-String "uveddi.*zip").Line.Split()[0]
$actualHash = (Get-FileHash -Algorithm SHA256 uveddi-1.0.0-x86_64-pc-windows-gnu.zip).Hash
if ($expectedHash -eq $actualHash.ToLower()) { 
    Write-Host "Checksum verified!" -ForegroundColor Green 
} else { 
    Write-Host "Checksum mismatch!" -ForegroundColor Red 
}
```

## Troubleshooting

### "Can't check signature: No public key"

You need to import the signing key first:
```bash
gpg --import UVEDDI_RELEASE_KEY.asc
```

### "WARNING: This key is not certified with a trusted signature!"

This is normal for the first import. You can optionally trust the key:
```bash
gpg --edit-key <KEY_ID>
gpg> trust
# Choose trust level (5 = ultimate trust)
gpg> quit
```

### Checksum Mismatch

If checksums don't match:
1. Re-download the file (may have been corrupted during download)
2. Verify you're using the correct checksum file for the version
3. If problem persists, report it as a security issue

## Security Contact

If you discover a security issue with Uveddi releases:
- Email: security@uveddi.io
- Report: https://github.com/botzrDev/uveddi/security/advisories

## Additional Information

- Release announcements: https://github.com/botzrDev/uveddi/releases
- Documentation: https://uveddi.io/docs
- Security policy: https://github.com/botzrDev/uveddi/security/policy
EOF
    
    log_success "Verification guide generated: ${doc}"
}

# ============================================================================
# Main
# ============================================================================

usage() {
    cat << EOF
Uveddi Artifact Signing and Verification Script

Usage:
  $0 <command>

Commands:
  checksums    Generate SHA256/SHA512 checksums
  sign         Sign all artifacts with GPG
  verify       Verify all signatures and checksums
  doc          Generate verification documentation
  all          Run checksums, sign, verify, and doc

Environment Variables:
  UVEDDI_GPG_KEY     GPG key ID for signing
  UVEDDI_DIST_DIR    Distribution directory (default: ./dist)

Examples:
  $0 all
  UVEDDI_GPG_KEY=ABC123 $0 sign
  $0 verify

EOF
}

main() {
    if [ $# -eq 0 ]; then
        usage
        exit 1
    fi
    
    local command="$1"
    
    case "${command}" in
        checksums)
            generate_checksums
            ;;
        sign)
            sign_artifacts
            ;;
        verify)
            verify_checksums
            verify_signatures
            ;;
        doc)
            generate_verification_doc
            ;;
        all)
            generate_checksums
            sign_artifacts
            verify_checksums
            verify_signatures
            generate_verification_doc
            ;;
        *)
            log_error "Unknown command: ${command}"
            usage
            exit 1
            ;;
    esac
}

main "$@"
