# Assignment A6 – Packaging & Distribution

**Status:** Not Started  
**Owner:** Solo Dev (Release Engineer)  
**Branch:** `release/1.0.0`

## Objective
Produce, sign, and validate all Uveddi 1.0.0 distribution artifacts. This includes binary builds for supported platforms, container images, installer scripts, and distribution documentation ready for publishing.

## Deliverables
1. **Build Artifact Set**
   - Linux/macOS/Windows binaries in `dist/` with checksums (SHA256, optional SHA512).
   - Docker/OCI image pushed to staging registry with digest documented.
2. **Installer & Upgrade Scripts**
   - CLI installer (`scripts/install.sh`, `scripts/install.ps1`), upgrade guide, and uninstall instructions.
3. **Artifact Manifest**
   - `docs/release-artifacts/manifest-1.0.0.md` listing files, hashes, signing info, and storage locations.
4. **Signing & Verification Evidence**
   - Signed binaries and detached signatures (`.sig`/`.asc`) plus verification commands.
5. **Distribution Readiness Checklist**
   - Completed checklist covering download mirrors, CDN caching (if applicable), and release-notes linkage.

## Tasks
1. **Build Pipeline Preparation**
   - Configure reproducible build scripts (e.g., `scripts/build_release.sh`).
   - Ensure cargo profiles and feature flags align with production configuration (`--features production-secure`).
2. **Artifact Production**
   - Compile platform binaries, package with required resources (config templates, documentation).
   - Build container image with minimal base, run security scans (trivy/grype).
3. **Signing & Hashing**
   - Sign binaries using documented PGP or platform certificates.
   - Generate checksums and store in `dist/checksums/`.
4. **Installer Authoring**
   - Update shell/PowerShell installers to fetch the correct version and verify signatures.
   - Document offline/off-network installation steps.
5. **Verification & Smoke Tests**
   - Install artifacts on clean environments (VMs/containers) to confirm functionality.
   - Record verification logs and screenshots where relevant.
6. **Manifest & Documentation**
   - Populate artifact manifest with download URLs, hashes, signature fingerprints.
   - Update README/download instructions to point to new assets.

## Acceptance Criteria
- All target platform artifacts build reproducibly with documented commands.
- Each artifact has matching checksum and signature; verification instructions provided.
- Installers succeed on clean environments; rollback/uninstall instructions validated.
- Container image passes security scan with no critical vulnerabilities.
- Manifest and checklist reviewed and stored under version control.

## Verification Steps
1. Reviewer runs checksum verification command from manifest to confirm integrity.
2. Reviewer spot-checks installer scripts for version accuracy and signature verification.
3. Reviewer validates container image digest and scan report stored in `reports/security/`.
4. Manifest and checklist are accessible in `docs/release-artifacts/`.
5. Tracker and risk register updated with packaging status.

## Dependencies & Notes
- Requires QA sign-off (Assignment A5) before finalizing artifacts.
- Coordinate with security/compliance documentation for signing instructions (Assignment A4).
- Ensure build environments are clean; capture Dockerfiles/VM snapshots if needed.
