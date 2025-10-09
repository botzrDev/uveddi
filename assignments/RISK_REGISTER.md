# Uveddi Commercial Launch – Risk Register

| ID | Risk Description | Source | Severity | Probability | Mitigation Plan | Owner | Target Resolution |
|----|------------------|--------|----------|-------------|-----------------|-------|-------------------|
| R1 | Detector calibration tolerances inconsistent across languages; precision targets (<85%) not yet validated. | archive/internal-docs/DETECTOR_TOLERANCE_AUDIT.md | High | Medium | Complete calibration work in Phase A3 using calibration suite; document new thresholds and rerun accuracy checks in A5. | Austin | 2025-11-22 |
| R2 | Database refactor assignments 03-05 unfinished, risking regression in persistence layer before release. | database-refactor-verification.md | High | Medium | Schedule repository + DI work early in Phase A3; maintain branch-based checkpoints; run targeted `cargo test database::*`. | Austin | 2025-11-08 |
| R3 | `uveddi config show` returns empty structure and get/set commands unverified, hurting configuration UX. | alpha-test-results/ALPHA_TEST_REPORT.md | Medium | High | Fix config read logic during Phase A3 bug sprint; add regression test in Phase A5. | Austin | 2025-11-22 |
| R4 | `uveddi ci check` lacks output/exit code clarity, blocking CI adoption. | alpha-test-results/ALPHA_TEST_REPORT.md | High | Medium | Instrument command during Phase A3; add smoke test in QA (Phase A5) and update docs in Phase A7. | Austin | 2025-11-22 |
| R5 | Detector selection flag (`--detectors`) not implemented; customers cannot scope analyses. | alpha-test-results/ALPHA_TEST_REPORT.md | Medium | Medium | Evaluate feasibility during Phase A3; if deferred, document workaround and roadmap note in Phase A7. | Austin | 2025-11-22 |
| R6 | Packaging/signing pipeline for cross-platform binaries unverified; risk of release-day failures. | Plan Phase A6 | High | Medium | Prototype build/sign flow by midpoint of Phase A6; maintain dry-run artifacts; validate on Linux/macOS/Windows smoke VMs. | Austin | 2025-11-29 |
| R7 | License enforcement tooling distributed separately; integration path unclear for subscribers. | assignments/COMMERCIAL_SCOPE_1.0.0.md | Medium | Medium | Prepare integration README + contract checklist in Phase A4; coordinate with legal for messaging in Phase A8. | Austin | 2025-11-15 |
| R8 | Support workflows (inbox, escalation, monitoring) not yet configured. | Plan Phase A9 | High | Medium | Define SOP and rehearsal during Phase A9; verify ticket flow and monitoring alerts before release week. | Austin | 2025-12-27 |
| R9 | Solo bandwidth may cause phase overruns, impacting target release window. | assignments/COMMERCIAL_SCOPE_1.0.0.md | High | Medium | Re-estimate weekly during tracker updates; prioritize critical defects; escalate slip risk in tracker notes immediately. | Austin | Ongoing |
| R10 | Signing certificates/distribution credentials availability not confirmed ahead of Phase A6. | assignments/COMMERCIAL_SCOPE_1.0.0.md | High | Low | Contact ops/legal partners during Phase A4 to confirm availability; secure credentials before Phase A6 start. | Austin | 2025-11-15 |

**Severity Legend:** High = release blocker, Medium = material customer impact, Low = minor inconvenience  
**Probability Legend:** High >60%, Medium 30-60%, Low <30%

---
**Last Updated:** 2025-10-09
