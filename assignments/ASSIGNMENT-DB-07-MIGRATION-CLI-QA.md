# Assignment DB-07 – Migration CLI QA & Demo Alignment

**Status:** Not Started  
**Target Window:** 2025-10-10 → 2025-10-14  
**Owner:** Senior Backend Developer (Tooling QA)  
**Branch:** `feature/db-refactor-phase2`

## Objective
Validate the new `migrate` CLI command end-to-end and clean up developer-facing demos that still reference the removed `database::crud` layer. This assignment closes the testing gap flagged after DB-05, ensures migrations remain reliable across workflows, and prevents drift in sample code.

## Deliverables
1. Integration test coverage for `migrate plan`, `migrate up`, `migrate status`, and `migrate down` flows (e.g., `tests/cli/migrate_command.rs`).
2. Temporary-database harness that runs migrations in isolation without polluting the repo (use `tempfile`, `tokio::fs`, or similar).
3. Updated `src/bin/simple_cycle_demo.rs` messaging to reference `ScalableDatabase` / repository architecture (remove `database::crud::Database` references).
4. Documentation updates summarizing new coverage (`archive/development/database-refactor-verification.md`, `assignments/IMPLEMENTATION_SUMMARY.md`, `assignments/LAUNCH_TRACKER.md`).

## Tasks
1. **Harness Setup**
   - Create a utility in the new test file to provision a temp SQLite path (e.g., `tempfile::NamedTempFile` or `tempfile::TempDir`).
   - Ensure each test cleans up its database (drop file or directory after assertions).
   - Configure tracing/log capture if needed so assertions focus on deterministic output.
2. **Plan & Status Coverage**
   - Write a `tokio::test` that constructs `MigrateCommand { subcommand: Plan { … } }`, calls `execute()`, captures stdout, and asserts that planned migrations list all versioned files (20251001–20251007).
   - Add a complementary `Status` test that first applies migrations, then validates `Current Version` and applied list formatting.
3. **Apply & Rollback Coverage**
   - Test the `Up` path end-to-end: verify that after running, `status` reports no pending migrations and the database file exists with expected size (>0 bytes).
   - Test the `Down` path: apply migrations in setup, roll back to a lower version (e.g., 20251003), and confirm the status reflects the rollback and that `plan` shows remaining versions.
4. **Demo Cleanup**
   - Update `src/bin/simple_cycle_demo.rs` narrative so code comments/examples reference `ScalableDatabase`/repository patterns instead of `database::crud::Database`.
   - Run `rg "database::crud::Database" src/bin` to confirm no lingering references.
5. **Documentation & Tracking**
   - Mark Assignment DB-05 verification checklist as covered by listing new tests.
   - Add summary notes (test names, command outputs) to `IMPLEMENTATION_SUMMARY.md` and `LAUNCH_TRACKER.md`.
   - Log any follow-up issues discovered (e.g., flaky migrations) in the tracker.
6. **Validation**
   - Execute `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, and the new CLI test(s) (`cargo test --test migrate_command` if you isolate them).
   - Capture command outputs needed for verification and attach paths in docs.

## Acceptance Criteria
- New CLI tests pass consistently on CI-equivalent runs and fail if migrations break.
- No references to `database::crud::Database` remain in `src/bin/simple_cycle_demo.rs`.
- `cargo test cli::migrate` (or equivalent command) passes alongside existing suites.
- Documentation and tracker entries cite the added tests and link to artifacts/output.

## Verification Steps
1. Reviewer runs `cargo test --test migrate_command` (or the consolidated CLI test suite) and sees all scenarios green.
2. Reviewer executes `rg "database::crud::Database" src/bin` and confirms zero matches.
3. Reviewer checks `archive/development/database-refactor-verification.md` and `assignments/IMPLEMENTATION_SUMMARY.md` for updated sections referencing the new coverage.
4. Reviewer spot-checks captured stdout/stderr snippets to ensure messaging matches CLI expectations.

## Dependencies & Notes
- Builds on DB-05 completion (migration runner/dry-run available).
- Reuses `feature/db-refactor-phase2` branch; keep commits focused (tests vs. demo cleanup).
- Leave the six pre-existing database test failures untouched; document if behavior changes.
- If you encounter non-deterministic timestamps in status output, sanitize before asserting (e.g., regex replace timestamps prior to comparison).

---
**Last Updated:** 2025-10-09
