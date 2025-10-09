# Assignment DB-08 – Migration Test Stabilization & CLI Coverage

**Status:** Not Started  
**Owner:** Senior Backend Developer (Tooling QA)  
**Branch:** `feature/db-refactor-phase2`

## Objective
Repair the failing migration CLI tests introduced in DB-07, ensure the registry/version expectations are consistent, and add direct coverage for the `MigrateCommand::execute` paths. This assignment moves DB-07 from “in progress” to “complete” and restores confidence in the migration tooling.

## Deliverables
1. Green `tests/cli/migrate_command.rs` suite (no panics, no unused-import warnings).
2. Consistent migration versioning between registry definitions and tests (sequential ids vs. date-based ids resolved).
3. New async tests that exercise each `MigrateCommand` subcommand (`Plan`, `Up`, `Down`, `Status`) via `execute()`.
4. Updated documentation (`archive/development/database-refactor-verification.md`, `assignments/IMPLEMENTATION_SUMMARY.md`, `assignments/LAUNCH_TRACKER.md`) reflecting the fix and test coverage.

## Tasks
1. **Version Alignment**
   - Decide whether to keep sequential versions (1..=7) or switch to date-based ids.
   - Update either `create_standard_registry()` or the test expectations accordingly.
   - Cover both version number assertions and migration name checks to prevent regressions.
2. **Harness Enhancements**
   - Refactor `MigrationTestHarness` if needed to share setup/teardown for CLI tests.
   - Ensure temporary DB paths can be passed cleanly into CLI command execution.
3. **CLI Command Tests**
   - Instantiate `MigrateCommand` for each subcommand and call `execute()` using the harness database path.
   - Capture and assert on stdout/stderr (use `assert_cmd`, `tokio::io::duplex`, or `test-log` if needed) to verify user-facing output.
   - Confirm that the command handles already-up-to-date and rollback scenarios gracefully.
4. **Stabilize Assertions**
   - Revisit flaky checks (e.g., timestamp formatting in status output). Use regex/substring comparisons if necessary to avoid brittle expectations.
   - Remove or justify any `println!` debugging left in the test file.
5. **Validation**
   - Run `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test --test migrate_command -- --nocapture`, and a representative subset of the full suite.
   - Resolve the unused import warning in the test module.
6. **Documentation & Tracker**
   - Update verification docs with actual command output snippets and confirm DB-07 can be marked complete.
   - Note the follow-up work in `assignments/IMPLEMENTATION_SUMMARY.md` and `assignments/LAUNCH_TRACKER.md`.

## Acceptance Criteria
- Migration tests pass locally (no failures or warnings) and do not require manual environment tweaks.
- Tests validate both the runner and CLI layers.
- Version numbering story is documented (e.g., inline comment or doc update).
- Tracker and verification docs reflect the restored green status.

## Verification Steps
1. `cargo test --test migrate_command -- --nocapture` → reports all tests passed.
2. `rg "database::crud::Database" src/bin` → still zero matches.
3. Reviewer observes updated docs noting the fix, with links to the specific test names executed.
4. Optional: run `cargo test cli::` (or other relevant group) to ensure no regressions.

## Dependencies & Notes
- Builds directly on the work from DB-07; coordinate with the engineer responsible for that assignment.
- If switching to date-based version numbers, ensure existing migration history is preserved and no runtime regressions occur.
- Keep commits focused (version fix vs. CLI coverage) to simplify review.

---
**Last Updated:** 2025-10-09
