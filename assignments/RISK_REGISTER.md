# Uveddi Commercial Launch – Risk Register

| ID | Risk Description | Source | Severity | Probability | Mitigation Plan | Owner | Target Resolution |
|----|------------------|--------|----------|-------------|-----------------|-------|-------------------|
| R1 | Detector calibration tolerances inconsistent across languages; precision targets (<85%) not yet validated. | archive/internal-docs/DETECTOR_TOLERANCE_AUDIT.md | High | Medium | Execute scheduled calibration sprint (2025-11-03 → 2025-11-08) per `assignments/DB_CALIBRATION_PLAN.md`; publish calibration report and updated thresholds before Phase A5. | Austin | 2025-11-22 |
| R2 | Database refactor assignments 03-05 unfinished, risking regression in persistence layer before release. *(Closed 2025-10-09 – DB assignments 01-06 complete)* | database-refactor-verification.md | Low | Low | Repository integration, migrations, and CRUD cleanup finalized; monitor only for regressions during calibration/QA. | Austin | 2025-10-09 |
| R3 | `uveddi config show` returns empty structure and get/set commands unverified, hurting configuration UX. *(Mitigated A3 – tests added)* | alpha-test-results/ALPHA_TEST_REPORT.md | Low | Low | Fix delivered 2025-10-09 with new integration tests; monitor regressions via CI. | Austin | 2025-10-09 |
| R4 | `uveddi ci check` lacks output/exit code clarity, blocking CI adoption. *(Mitigated A3 – parsing fix & tests)* | alpha-test-results/ALPHA_TEST_REPORT.md | Medium | Low | JSON parsing & gate behavior validated 2025-10-09; add CI smoke during Phase A5 for ongoing coverage. | Austin | 2025-10-09 |
| R5 | Detector selection flag (`--detectors`) not implemented; customers cannot scope analyses. | alpha-test-results/ALPHA_TEST_REPORT.md | Medium | Medium | Evaluate feasibility during Phase A3; if deferred, document workaround and roadmap note in Phase A7. | Austin | 2025-11-22 |
| R6 | Packaging/signing pipeline for cross-platform binaries unverified; risk of release-day failures. | Plan Phase A6 | High | Medium | Prototype build/sign flow by midpoint of Phase A6; maintain dry-run artifacts; validate on Linux/macOS/Windows smoke VMs. | Austin | 2025-11-29 |
| R7 | License enforcement tooling distributed separately; integration path unclear for subscribers. *(Mitigated A4 – plan documented)* | assignments/COMMERCIAL_SCOPE_1.0.0.md | Low | Low | Manual verification process documented in `docs/release-artifacts/license-enforcement-plan.md` for 1.0.0; automated enforcement planned Q1 2026; customer workflow defined. Residual: Implementation required post-launch. | Austin | 2025-10-09 (Mitigated) |
| R8 | Support workflows (inbox, escalation, monitoring) not yet configured. | Plan Phase A9 | High | Medium | Define SOP and rehearsal during Phase A9; verify ticket flow and monitoring alerts before release week. | Austin | 2025-12-27 |
| R9 | Solo bandwidth may cause phase overruns, impacting target release window. | assignments/COMMERCIAL_SCOPE_1.0.0.md | High | Medium | Re-estimate weekly during tracker updates; prioritize critical defects; escalate slip risk in tracker notes immediately. | Austin | Ongoing |
| R10 | Signing certificates/distribution credentials availability not confirmed ahead of Phase A6. *(Mitigated A4 – process documented)* | assignments/COMMERCIAL_SCOPE_1.0.0.md | Medium | Low | Signing procedures fully documented in `docs/release-artifacts/signing-checklist.md`; certificate procurement tracked with target dates (2025-11-01); dry-run process validated. Residual: Actual certificate acquisition pending ops/legal coordination. | Austin | 2025-11-01 (Target) |
| R11 | Required docs (`docs/CLI_REFERENCE.md`, `docs/TROUBLESHOOTING.md`) missing; could slip past Phase A3 and leave launch docs incomplete. *(Closed A3 – docs published)* | docs/release-artifacts/scope-implementation-diff-2025-10-09.md | Low | Low | Documentation delivered 2025-10-09; keep doc drift checks in future phases. | Austin | 2025-10-09 |

**Severity Legend:** High = release blocker, Medium = material customer impact, Low = minor inconvenience  
**Probability Legend:** High >60%, Medium 30-60%, Low <30%

---

## Assignment A4 Risk Mitigation Summary (2025-10-09)

**R7 - License Enforcement:**
- **Status:** Mitigated → Downgraded from Medium/Medium to Low/Low
- **Mitigation Evidence:**
  - ✅ License enforcement plan documented (`docs/release-artifacts/license-enforcement-plan.md`)
  - ✅ Manual verification workflow defined for 1.0.0 release
  - ✅ Automated enforcement roadmap created (Q1 2026 target)
  - ✅ Customer onboarding process documented
  - ✅ Contract-based enforcement approach validated
- **Residual Risk:** Implementation of automated enforcement deferred to post-1.0.0
- **Justification:** Manual process acceptable for limited early customer base with contract agreements

**R10 - Signing Certificates:**
- **Status:** Partially Mitigated → Downgraded from High/Low to Medium/Low
- **Mitigation Evidence:**
  - ✅ Comprehensive signing checklist created (`docs/release-artifacts/signing-checklist.md`)
  - ✅ Linux GPG signing process documented
  - ✅ macOS codesign process documented with notarization steps
  - ✅ Windows Authenticode process documented
  - ✅ Certificate requirements identified for all platforms
  - ✅ Procurement targets set (2025-11-01)
- **Residual Risk:** Physical certificate acquisition pending coordination with ops/legal
- **Justification:** Process documented; certificates can be obtained within target timeline

**Cross-Cutting Deliverables (A4):**
- ✅ Dependency license audit completed (`reports/license-audit-2025-11.md`)
- ✅ Security contact information published (`SECURITY_CONTACT.md`)
- ✅ Security policy documented (`SECURITY_POLICY.md`)
- ✅ Compliance gap analysis performed (`reports/compliance-gap-analysis-2025-10-09.md`)
- ⏳ README security documentation links (in progress)

---
**Last Updated:** 2025-10-09
