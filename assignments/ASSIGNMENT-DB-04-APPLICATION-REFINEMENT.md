# Assignment DB-04 – Application Layer Repository Integration

**Status:** Not Started  
**Owner:** Senior Backend Developer  
**Branch:** `feature/db-refactor-phase2`

## Objective
Complete the migration of the application layer so it relies exclusively on the repository interfaces introduced in the database refactor. Remove the legacy `Database` struct from `src/application` and ensure all orchestrators, services, and workflows consume repositories via dependency injection.

## Deliverables
1. `Database` struct no longer imported or instantiated within `src/application/**`.
2. `AnalysisOrchestrator` and related services/use-cases operate through `RepositoryManager` (or equivalent DI container).
3. Updated tests (`cargo test application::`) pass without touching the legacy database API.
4. `archive/development/database-refactor-verification.md` updated with Assignment 04 marked **Complete**.
5. Verification notes added to `assignments/IMPLEMENTATION_SUMMARY.md` and `assignments/LAUNCH_TRACKER.md`.

## Tasks
1. **Code Audit**
   - Search for `use crate::database::Database` and `Database::` within `src/application`.
   - Identify constructor paths where `Database` is still injected.
2. **Refactor Constructors**
   - Update constructors to accept repository dependencies (`RepositoryManager`, specific repositories, or service structs).
   - Adjust initialization in `main.rs` / CLI command wiring to pass repositories.
3. **Workflow Updates**
   - Modify orchestrators and services to call repository methods directly.
   - Remove helper methods that only wrap `Database`.
4. **Config & DI Wiring**
   - Ensure the bootstrap process creates repositories once and shares them across components.
   - Update any feature-gated paths (AI, security) to use the new injection style.
5. **Test & Verification**
   - Run `cargo fmt` and `cargo clippy --all-targets`.
   - Execute `cargo test application::` and integration suites touching application orchestration.
   - Capture output snippets for verification logs.
6. **Documentation**
   - Update `database-refactor-verification.md` checklist for Assignment 04.
   - Note completion in launch tracker working notes.

## Acceptance Criteria
- No remaining references to `Database` within `src/application`.
- Application tests pass without regression.
- Repository injections follow agreed architecture (single source of truth via DI container).
- Documentation and tracker updated to reflect completion.

## Verification Steps
1. Reviewer runs `rg "Database" src/application` to ensure zero matches (aside from comments if any).
2. Reviewer confirms test logs attached or referenced.
3. Plan documents updated (`database-refactor-verification.md`, `IMPLEMENTATION_SUMMARY.md`, tracker).

## Dependencies & Notes
- Depends on repository layer being stable (confirmed in Assignment 03).
- Coordinate with calibration tooling team if DI changes affect testing scripts.
- Keep branch clean; use small commits for easier review.

---
**Last Updated:** 2025-10-09
