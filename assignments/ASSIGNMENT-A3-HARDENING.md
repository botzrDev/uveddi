# Assignment A3 – Hardening Sprint & Cleanup

**Status:** Not Started  
**Target Window:** 2025-10-19 → 2025-11-08  
**Owner:** Austin (solo dev)  

## Objective
Stabilize the CLI for the 1.0.0 launch by closing product-surface gaps, addressing high-priority defects, and setting up the refactor workstreams that unblock later phases. This sprint also delivers the documentation artifacts identified in Assignment A2 and resolves open decisions around security and HTML output.

## Deliverables
1. ✅ Documentation:  
   - `docs/CLI_REFERENCE.md` – Command-by-command guide derived from the CLI help snapshot.  
   - `docs/TROUBLESHOOTING.md` – Troubleshooting appendix covering common issues and doctor workflows.
2. ✅ Decisions & Scope Updates:  
   - Security feature flag decision recorded (remove, implement, or document marker) with scope/readme updated.  
   - HTML output format explicitly captured in scope (approved or marked experimental).
3. ✅ Defect Fixes & Tests:  
   - `uveddi config show` / get / set behavior corrected with integration test coverage.  
   - `uveddi ci check` output/exit code clarified and tested.  
   - Detector selection flag decision (implement `--detectors` or document roadmap).  
4. ✅ Refactor Kickoff:  
   - Database refactor assignments 03–05 plan finalized; actionable TODO list created in tracker.  
   - Detector calibration backlog triaged with schedule for calibration runs.  
5. ✅ Verification Evidence:  
   - Test runs (`cargo test`, targeted integration tests) recorded in tracker.  
   - Updated `docs/release-artifacts/` README with new artifacts and decisions.  
   - Tracker/assignment status updated to reflect completion.

## Tasks
1. **Documentation Creation**
   - Draft `docs/CLI_REFERENCE.md` using `docs/release-artifacts/cli-help-2025-10-09.md` as source. Include usage examples and common option sets.  
   - Author `docs/TROUBLESHOOTING.md` covering install issues, parser failures, AI connectivity, caching resets, and `uveddi doctor` workflows.  
   - Cross-link both docs from README and tracker.

2. **Security & HTML Decisions**
   - Inspect `Cargo.toml` and code for `security` feature usage. Decide to (a) remove flag, (b) make it gate detectors, or (c) document marker. Apply changes to scope doc, README, and feature matrix.  
   - Review HTML output implementation; either add to scope (`COMMERCIAL_SCOPE_1.0.0.md` Section 2) or mark as experimental in docs with warning banner.

3. **Bug Fixes & Tests**
   - Fix `config` command read/write path so `show`, `get`, `set` operate on actual config file (`uveddi.example.toml` baseline). Add integration test under `tests/`.  
   - Enhance `ci check` to print summary and return non-zero on failure; create regression test (may stub using fixture project).  
   - Evaluate feasibility of a `--detectors` option (or alternative). If scope feasible, implement with documentation; otherwise document roadmap item in release notes/feature matrix.

4. **Refactor Planning**
   - Break down database refactor assignments (03–05) into actionable tasks with owners/dates in tracker.  
   - Review calibration backlog (R1) and schedule actual calibration sprints, noting required repos or tooling.

5. **Verification**
   - Run `cargo fmt` and `cargo clippy --all-targets` to ensure cleanliness.  
   - Execute `cargo test` plus any new integration tests (config, CI). Capture output snippets in tracker/assignment.  
   - Update `docs/release-artifacts/README.md` with new documentation references and decisions.

## Acceptance Criteria
- Both new docs exist, cross-linked, and reviewed for accuracy.  
- README and scope document updated to match final decisions (security feature + HTML output).  
- Config and CI command fixes merged with passing tests; documented in diff report/notes.  
- Tracker shows scheduled plan for database refactor and calibration.  
- Assignment marked **Complete** with verification evidence (test logs, doc links, decision notes).

## Verification Steps
1. PM reviews new docs for completeness; confirms README links functional.  
2. Run regression tests and attach outputs to tracker.  
3. Confirm risk register updated if new risks identified or existing ones mitigated (R1–R4).  
4. Ensure launch tracker Phase A3 row updated with completion date and artifacts.

## Dependencies & Notes
- Requires CLI snapshot and feature matrix from A2 (already available).  
- Coordinate security/HTML decisions before touching scope doc to avoid rework.  
- Any large implementation changes (e.g., detector selection) should be scoped carefully to avoid destabilizing release timeline.

---
**Last Updated:** 2025-10-09
