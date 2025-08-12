# Uveddi Alpha Release Setup

This document explains how to set up GitHub Releases for Uveddi alpha testing with binary distribution and authenticated access.

## Overview

The setup provides:
- ✅ Cross-platform binary builds (Linux, macOS, Windows)
- ✅ Authenticated downloads via GitHub Personal Access Tokens
- ✅ No source code exposure for alpha testers
- ✅ Fast installation (no compilation required)
- ✅ Professional alpha testing experience

## Release Process

### 1. Creating a Release

To create a new release with binaries:

```bash
# Tag the current commit
git tag v0.9.0-alpha.1
git push origin v0.9.0-alpha.1

# Or manually trigger the workflow with a custom tag
# Go to GitHub Actions -> Release workflow -> Run workflow
```

The workflow will automatically:
- Build binaries for 6 platforms:
  - `x86_64-unknown-linux-gnu` (Linux x64)
  - `x86_64-unknown-linux-musl` (Linux x64 static)
  - `aarch64-unknown-linux-gnu` (Linux ARM64)
  - `x86_64-apple-darwin` (macOS Intel)
  - `aarch64-apple-darwin` (macOS Apple Silicon)
  - `x86_64-pc-windows-msvc` (Windows x64)
- Create a GitHub release with all binaries attached
- Mark the release as a prerelease

### 2. Platform-Specific Binary Names

Binaries are packaged as:
- Unix/Linux/macOS: `uveddi-{target}.tar.gz`
- Windows: `uveddi-{target}.zip`

## Alpha Tester Setup

### For Alpha Team (Distribution)

1. **Grant Repository Access:**
   - Add alpha testers as collaborators to `botzrDev/uveddi`
   - Or create a GitHub team with `Contents: Read` permission

2. **Provide Installation Instructions:**
   Send alpha testers the following instructions:

### For Alpha Testers (Installation)

#### Step 1: Get a GitHub Personal Access Token

1. Go to [GitHub Settings > Personal Access Tokens](https://github.com/settings/tokens)
2. Click "Generate new token (classic)"
3. Give it a descriptive name: "Uveddi Alpha Access"
4. Select scopes:
   - ✅ `Contents` (read access to repository contents)
5. Click "Generate token"
6. **Copy the token immediately** (you won't be able to see it again)

#### Step 2: Install Uveddi

Run the following command with your token:

```bash
GITHUB_TOKEN=your_token_here curl -sSL https://raw.githubusercontent.com/botzrDev/uveddi/alpha/install-release.sh | bash
```

**Alternative: Download and run locally**
```bash
# Download the script first
curl -O https://raw.githubusercontent.com/botzrDev/uveddi/alpha/install-release.sh
chmod +x install-release.sh

# Run with your token
GITHUB_TOKEN=your_token_here ./install-release.sh
```

#### Step 3: Verify Installation

```bash
uveddi --version
uveddi --help
```

## Installation Script Features

The `install-release.sh` script:

### Automatic Platform Detection
- Detects OS (Linux, macOS, Windows)
- Detects architecture (x86_64, aarch64)
- Downloads the correct binary automatically

### Security Features
- Requires GitHub authentication
- Validates token permissions
- Provides clear error messages for access issues

### User Experience
- Installs to `~/.local/bin` by default (customizable with `INSTALL_DIR`)
- Checks if install directory is in PATH
- Provides guidance for PATH setup
- Verifies installation success

### Error Handling
- Clear error messages for missing tokens
- Helpful instructions for getting access
- Platform-specific guidance
- Rate limiting awareness

## Hosting the Install Script

### Option 1: GitHub Raw (Current)
```bash
curl -sSL https://raw.githubusercontent.com/botzrDev/uveddi/alpha/install-release.sh | bash
```

### Option 2: Custom Domain (Recommended)
1. Set up `https://uveddi.org/install-release.sh` to serve the script
2. Update the script to use the custom domain in examples
3. This provides a more professional appearance and easier URLs

## Benefits of This Approach

### For the Uveddi Team
- **No source code exposure** to alpha testers
- **Version control** over who has access
- **Easy revocation** by removing collaborator access
- **Professional appearance** with custom installation experience
- **Analytics** through GitHub's download statistics

### For Alpha Testers
- **Fast installation** (no compilation required)
- **Cross-platform support** with automatic detection
- **Simple authentication** with GitHub tokens they already understand
- **Clear error messages** when access is denied
- **Standard tooling** using GitHub's release system

## Troubleshooting

### Common Issues for Alpha Testers

1. **"Not Found" or Access Denied**
   - Verify the GitHub token has `Contents` read permission
   - Ensure they've been added as a repository collaborator
   - Check if the token has expired

2. **"Rate Limit Exceeded"**
   - GitHub has API rate limits (60 requests/hour for unauthenticated, 5000/hour for authenticated)
   - Wait and try again later
   - This shouldn't happen with authenticated requests

3. **Platform Not Supported**
   - We support Linux (x64, ARM64), macOS (Intel, Apple Silicon), and Windows (x64)
   - For other platforms, they'll need to build from source

4. **Binary Not Found in PATH**
   - The script installs to `~/.local/bin` by default
   - They need to add this to their PATH or use a different `INSTALL_DIR`

## Security Considerations

1. **Token Security:**
   - Tokens should be treated as passwords
   - Recommend setting expiration dates on tokens
   - Tokens should only have minimal required permissions

2. **Repository Access:**
   - Only grant `Contents: Read` permission
   - Regularly audit collaborator access
   - Remove access when alpha testing ends

3. **Binary Verification:**
   - Consider adding checksums to releases in the future
   - The script downloads over HTTPS with authentication

## Future Enhancements

1. **Checksum Verification:**
   - Add SHA256 checksums to releases
   - Verify downloads in the install script

2. **Auto-Update Capability:**
   - Add `uveddi update` command
   - Check for newer releases automatically

3. **Metrics and Analytics:**
   - Track download counts
   - Monitor installation success rates
   - Gather feedback on platform usage

4. **Custom Domain:**
   - Host install script at `https://uveddi.org/install-release.sh`
   - Provides more professional URLs for alpha testers