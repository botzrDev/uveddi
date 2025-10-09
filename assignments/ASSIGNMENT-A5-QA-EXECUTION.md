# Assignment A5 – QA Execution

**Status:** Not Started  
**Owner:** Solo Dev (acting QA Lead)  
**Branch:** `release/1.0.0`

## Objective
Validate that Uveddi 1.0.0 meets functional, performance, and stability expectations before packaging. This assignment formalizes automated and manual test passes, regression analysis, and release candidate sign‑off.

## Deliverables
1. **Comprehensive QA Report** documenting executed suites, pass/fail counts, and environment details (`reports/qa/qa-summary-2025-11-22.md`).
2. **Regression Matrix** covering critical user flows, CLI commands, detector behaviors, and database paths (`reports/qa/regression-matrix.csv`).
3. **Defect Log & Triage Notes** with status for each finding, linked to issues or TODOs (`reports/qa/defect-log-2025-11.csv`).
4. **Coverage Snapshot** consolidating code coverage, detector scenario coverage, and manual test checklists.
5. **Release Candidate Sign-Off**: decision memo confirming readiness or listing blocking defects.

## Tasks
1. **Test Inventory Refresh**
   - Audit automated suites (unit, integration, CLI, detector calibration) and ensure they are runnable via scripts.
   - Update `tests/README.md` with latest invocation instructions.
2. **Automated Test Execution**
   - Run `cargo test --all-features`, key integration suites, and CLI regression tests.
   - Capture logs/artifacts to `reports/qa/logs/`.
3. **Manual Scenario Sweep**
   - Execute high-value workflows (init → analyze → report export, detector calibration smoke, migration up/down).
   - Record observations and screenshots if applicable.
4. **Bug Triage & Fix Coordination**
   - Log defects with severity, reproduction steps, and ownership.
   - For critical/high bugs, either patch immediately or document mitigation with target resolution date.
5. **Performance & Resource Spot Checks**
   - Run targeted benchmarks or profiling (e.g., cache warm-up, large repo analysis).
   - Compare results with baseline metrics from earlier milestones.
6. **Final Verification & Memo**
   - Summarize executed coverage, outstanding issues, and go/no-go recommendation.
   - Update tracker and risk register with QA outcomes.

## Acceptance Criteria
- All planned automated suites executed with documented pass/fail results.
- No open critical/high severity defects; medium items have mitigation/owners.
- Manual regression checklist completed with evidence.
- Coverage snapshot stored and linked from the QA report.
- QA sign-off memo delivered with clear release recommendation.

## Verification Steps
1. Reviewer opens `reports/qa/qa-summary-2025-11-22.md` to confirm scope and results.
2. Reviewer confirms `reports/qa/regression-matrix.csv` includes all critical flows with statuses.
3. Reviewer checks defect log for triage notes and linked issues.
4. Reviewer validates coverage artifacts (e.g., `reports/qa/coverage/lcov.info`) exist.
5. Risk register updated with QA outcomes; tracker reflects assignment completion.

## Dependencies & Notes
- Requires DB refactor assignments (DB-04..DB-08) to be complete and merged.
- Coordinate with detector calibration sprint outputs for updated test scenarios.
- Ensure environments used for QA are reproducible (document config, data seeds).
