# Assignment A2 – Product Surface Freeze

**Status:** Not Started  
**Target Window:** 2025-10-12 → 2025-10-18  
**Owner:** Senior Developer (You)  

## Objective
Audit and lock the public-facing product surface for the 1.0.0 commercial release. Confirm the CLI command set, feature flag matrix, and documentation accurately reflect the approved scope so downstream hardening work proceeds against a stable baseline.

## Deliverables
- Current-state CLI command snapshot (`uveddi --help` and per-command `--help`) saved to `docs/release-artifacts/cli-help-2025-10-YY.md`.
- Feature flag & build profile matrix documented in `docs/release-artifacts/feature-flags-1.0.md`, highlighting required/default/optional flags plus unresolved gaps.
- Diff report summarizing discrepancies between implementation and the approved scope (`assignments/COMMERCIAL_SCOPE_1.0.0.md`) or existing docs; include recommended fixes and owners.
- Updated documentation drafts (README/CLI reference) or filed tickets for items outside your remit.

## Tasks
1. **Command Surface Audit**
   - Run `cargo run -- --help` and each subcommand `cargo run -- <cmd> --help` using the `cli-standard` feature set.
   - Capture output, normalize formatting, and store in the release artifacts directory.
   - Compare against README, release notes, and scope document; flag mismatches (missing commands, outdated options, renamed flags).
2. **Feature Flag Mapping**
   - Inspect `Cargo.toml` feature definitions, `conditional_compilation.txt`, and any gating macros in `src/`.
   - Build a matrix covering: default features, commercial-required toggles, optional add-ons (AI, plugins, security), and deprecated flags.
   - Identify dead or undocumented features; recommend removal or documentation updates.
3. **Documentation Alignment**
   - Update drafts directly where quick fixes are possible (e.g., README command list, feature flag section).
   - For larger doc impacts, create TODO entries or tickets with clear descriptions.
4. **Sign-Off Packet**
   - Summarize findings and decisions in a short memo (add to `docs/release-artifacts/README.md` or tracker notes).
   - Ensure all artifacts are linked in the launch tracker verification section.

## Acceptance Criteria
- CLI help snapshot stored and referenced in tracker.
- Feature flag matrix lists feature name, purpose, default state, and customer-facing status.
- All discrepancies documented with recommended actions; critical gaps highlighted for Phase A3.
- README/CLI reference either updated or have open tasks referencing required fixes.
- Assignment status updated to **Complete** with verification pointers.

## Verification Steps
1. PM reviews stored CLI help outputs for completeness.
2. Scope vs implementation diff reviewed; no undisclosed gaps remain.
3. Tracker (Phase A2) updated with artifact links and status.

## Notes & Dependencies
- Coordinate with PM before removing or renaming commands/flags.
- Reference historical gating docs in `archive/development/` if context needed.
- Any new tickets should live in the shared launch tracker with due dates.

---
**Last Updated:** 2025-10-09
