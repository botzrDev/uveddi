# Assignment DB-06 – Legacy CRUD Layer Decommission

**Status:** Not Started  
**Target Window:** 2025-10-27 → 2025-11-02  
**Owner:** Backend Developer (Repository Migration)  
**Branch:** `feature/db-refactor-phase2`

## Objective
Finish the database refactor by removing or minimizing the legacy `crud.rs` layer, ensuring all data access flows through repository interfaces. This eliminates redundant abstractions, reduces technical debt, and prepares the codebase for calibration and QA phases.

## Deliverables
1. `src/database/crud.rs` removed entirely **or** reduced to thin compatibility shims (<200 LOC) with deprecation notice.
2. All call sites migrated to use repository APIs (`RepositoryManager`, specific repository traits).
3. Full database and integration test suites passing (`cargo test database::`, `cargo test --test integration`).
4. Updated documentation and trackers (`database-refactor-verification.md`, `IMPLEMENTATION_SUMMARY.md`, launch tracker note).

## Tasks
1. **Usage Audit**
   - Run `rg "crud::" src/` to identify remaining dependencies.
   - Document high-risk call sites (orchestrator, services, CLI commands) requiring migration.
2. **Repository Migration**
   - For each call site, replace legacy CRUD calls with repository equivalents.
   - Ensure transactions, error handling, and logging behavior preserved.
3. **Shim or Delete**
   - If immediate deletion is possible, remove `crud.rs` and update module exports.
   - Otherwise, retain minimal shim functions that delegate to repositories, annotate with `#[deprecated(note = "...")]`, and add TODO with removal plan.
4. **Test & Verification**
   - Run `cargo fmt` and `cargo clippy --all-targets -- -D warnings`.
   - Execute `cargo test database::` and relevant integration tests.
   - If new tests required (e.g., regression coverage), add under `tests/`.
5. **Documentation**
   - Update `archive/development/database-refactor-verification.md` marking Assignment 06 complete.
   - Append verification notes to `assignments/IMPLEMENTATION_SUMMARY.md`.
   - Add completion entry to launch tracker working notes.

## Acceptance Criteria
- No production code imports `src/database/crud.rs` (except optional deprecated shims).
- Repository layer is sole abstraction for database access.
- Build & test suites pass; no clippy warnings.
- Documentation/tracker reflects completion with references to commits/tests run.

## Verification Steps
1. Reviewer runs `rg "crud::" src/` to confirm zero usage (or only deprecated shims slated for removal).
2. Reviewer executes `cargo test database::` and `cargo test --test integration` to confirm green suites.
3. Reviewer checks `database-refactor-verification.md` and `IMPLEMENTATION_SUMMARY.md` for updated status.

## Dependencies & Notes
- Requires Assignment DB-04 to be complete (application layer using repositories).
- Coordinate with teams depending on legacy CRUD API (e.g., calibration tooling) to ensure migration is transparent.
- Keep commits focused per subsystem for easier review/reverts if needed.

---
**Last Updated:** 2025-10-09
