# Database Refactor & Detector Calibration Plan

**Created:** 2025-10-09  
**Owner:** Austin  
**Scope:** Address Risk R1 (detector calibration backlog) and Risk R2 (database refactor assignments 03–05)

## Goals
- Lock in a concrete timeline and task breakdown for completing the remaining database refactor verification assignments.
- Schedule detector calibration work so precision targets can be validated before QA (Phase A5).
- Ensure outputs feed directly into Phase A3 follow-up work and set expectations for Phase A4 readiness.

## Schedule Overview

| Week (Target) | Focus | Key Tasks | Deliverables |
|---------------|-------|-----------|--------------|
| 2025-10-13 → 2025-10-18 | Database Refactor Assignment 03 | Define repository traits, implement SQLite repositories, ensure tests in `database::repositories` pass | Updated `database-refactor-verification.md` checklist, new repository tests |
| 2025-10-20 → 2025-10-25 | Database Refactor Assignment 04 | Update application layer to depend on repositories, remove direct `Database` usage, update DI | Green `application::` tests, migration plan documented |
| 2025-10-27 → 2025-11-02 | Database Refactor Assignment 05 | Organize migrations, standardize error handling, verify dry-run tooling | Versioned migration files, passing `database::migrations` & `database::error` tests |
| 2025-11-03 → 2025-11-08 | Detector Calibration Sprint | Run calibration suite per `archive/internal-docs/DETECTOR_TOLERANCE_AUDIT.md`, adjust thresholds, document results | Calibration report, updated detector configs, precision metrics archived |

## Task Breakdown

### Database Refactor (Assignments 03–05)
- Review `database-refactor-verification.md` criteria for each assignment.
- Create Git branch `feature/db-refactor-phase2`.
- Implement repository traits and SQLite implementations (`src/database/repositories/`).
- Update application orchestrator/services to use repositories (`src/application/`).
- Restructure migrations and standardize error handling.
- Add/adjust tests in `tests/` to cover new repository and migration behavior.
- After each assignment, update checklist in `database-refactor-verification.md` and note status in tracker.

### Detector Calibration
- Prepare test repos from `archive/internal-docs/DETECTOR_TOLERANCE_AUDIT.md`.
- Automate analysis runs via scripts in `testing-uveddi/` (create new script if needed).
- Record precision/recall metrics for each detector.
- Update detector configurations in `src/analysis/detectors/**` with tuned thresholds.
- Produce calibration summary report (`reports/detector-calibration-2025-11.md`).
- Update risk register R1 with results and adjust probability/severity if warranted.

## Dependencies
- Access to sample repos in `test-codebases/` and `testing-uveddi/`.
- Current database schema/migration state documented in `database-refactor-verification.md`.
- Calibration guidelines in `archive/internal-docs/DETECTOR_TOLERANCE_AUDIT.md`.

## Milestone Tracking
- Add weekly checkpoints to `assignments/LAUNCH_TRACKER.md` working notes as tasks complete.
- Capture test outputs in `docs/release-artifacts/README.md` once calibration reports are produced.
- Update risk register after each major milestone.

## Next Actions
1. Create branch `feature/db-refactor-phase2` for upcoming database work.
2. Review and prioritize sub-tasks for Assignment 03; begin implementation week of 2025-10-13.
3. Assemble calibration repo list and tooling notes ahead of the November sprint.

---
