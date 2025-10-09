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

## Assignment A5 Risk Assessment & Updates (2025-11-22)

### New Risks Identified During QA

| ID | Risk Description | Source | Severity | Probability | Mitigation Plan | Owner | Target Resolution |
|----|------------------|--------|----------|-------------|-----------------|-------|-------------------|
| R12 | Security detectors using unsupported regex features (lookahead/lookbehind); 35+ tests failing, production accuracy unknown. | QA Execution (DEF-002) | **CRITICAL** | High | Rewrite regex patterns without lookahead/lookbehind; use character classes and word boundaries; document security features as BETA for v1.0.0; fix in v1.0.1 patch. | Security Team | 2025-12-06 (v1.0.1) |
| R13 | Integration test suite blocked by compilation errors; 3+ test files reference removed modules (web, TUI, database). | QA Execution (DEF-001) | High | High | Update test imports to current module structure; remove tests for removed features; re-establish integration test coverage in v1.0.1. | Dev Team | 2025-12-06 (v1.0.1) |
| R14 | Detector calibration shows false negatives (God Object, Long Methods); thresholds too conservative. | QA Execution (DEF-006, DEF-007) | Medium | Medium | Review and adjust detection thresholds; add CLI configuration options; test against real-world codebases; update in v1.1.0. | Calibration Team | 2026-01-15 (v1.1.0) |
| R15 | Database test pollution causing failures on repeated runs; tables persist between test executions. | QA Execution (DEF-003) | Medium | Medium | Implement test isolation with unique table names or cleanup hooks; use tempfile for test databases; fix in v1.0.1. | QA Team | 2025-12-06 (v1.0.1) |
| R16 | 10.1% unit test failure rate (77/762); combination of regex issues, state pollution, and implementation changes. | QA Execution Summary | High | High | Prioritize critical fixes (R12, R13, R15); accept remaining failures with documentation for v1.0.0; systematic improvement in v1.0.1 and v1.1.0. | QA Team | Phased (see plan) |

### Risk Updates from QA Execution

**R1 - Detector Calibration:**
- **Updated Status:** VALIDATED with GAPS
- **QA Findings:**
  - Magic Values detector: ✅ Working correctly
  - God Object detector: ⚠️ Conservative thresholds (DEF-006)
  - Long Methods detector: ⚠️ Conservative thresholds (DEF-007)
- **Impact:** Partial validation; some detectors need threshold adjustment
- **Action:** Defer comprehensive calibration to v1.1.0; document known limitations for v1.0.0

**R5 - Detector Selection Flag:**
- **Updated Status:** DEFERRED (Confirmed)
- **QA Impact:** Not blocking; users can run full analysis
- **Documentation:** Added to release notes as roadmap item

**R6 - Packaging/Signing Pipeline:**
- **QA Readiness Check:**
  - ✅ Release binary builds successfully (2m 34s)
  - ✅ Binary executes correctly
  - ✅ Version reporting accurate
- **Status:** Ready for Phase A6 execution

**R8 - Support Workflows:**
- **QA Input Provided:**
  - Known limitations documented
  - Defect log created for support reference
  - Common issues identified
- **Status:** QA artifacts feed into A9 support planning

**R9 - Solo Bandwidth:**
- **A5 Impact:** QA execution completed in 6 hours (within estimate)
- **Slip Risk:** Moderate - critical defects require v1.0.1 patch work
- **Mitigation:** Prioritize DEF-001, DEF-002 for immediate post-release

### QA-Driven Release Strategy Adjustments

**Release Recommendation:** CONDITIONAL GO with v1.0.1 Commitment

**Required Conditions for v1.0.0:**
1. ✅ Document security detector limitations (BETA status) in release notes
2. ✅ Document known test failures and detector calibration gaps
3. ✅ Create GitHub tracking issues for DEF-001, DEF-002, R12, R13
4. ✅ Commit to v1.0.1 patch within 2 weeks (target: 2025-12-06)
5. ✅ Provide user guidance on interpreting results

**v1.0.1 Patch Scope (2 weeks):**
- [ ] Fix R12: Security detector regex patterns (DEF-002) - CRITICAL
- [ ] Fix R13: Integration test compilation (DEF-001) - HIGH
- [ ] Fix R15: Database test isolation (DEF-003) - HIGH
- [ ] TypeScript language detection (DEF-008) - MEDIUM

**v1.1.0 Enhancement Scope (1-2 months):**
- [ ] Address R14: Detector calibration (DEF-006, DEF-007)
- [ ] Update R1: Complete calibration sprint
- [ ] Resolve R16: Systematic test suite improvement
- [ ] Achieve >80% code coverage target

### Risk Severity Reassessment Post-QA

| Risk | Pre-QA Severity | Post-QA Severity | Trend | Rationale |
|------|-----------------|------------------|-------|-----------|
| R1 | High | Medium | ↓ Improved | Partial validation complete; gaps identified and scoped |
| R5 | Medium | Low | ↓ Improved | Confirmed as non-blocking; documentation plan in place |
| R6 | High | Medium | ↓ Improved | Binary builds validated; packaging risks reduced |
| R8 | High | Medium | ↓ Improved | QA artifacts provide support foundation |
| R9 | High | Medium | → Stable | A5 on schedule; patch work adds moderate load |
| R12 | NEW | **CRITICAL** | ⚠️ NEW | Security accuracy unknown; requires immediate attention |
| R13 | NEW | High | ⚠️ NEW | Integration coverage gap; needs systematic fix |

### Overall Risk Posture

**Pre-QA Risk Level:** MEDIUM-HIGH (detector calibration, packaging unknowns)
**Post-QA Risk Level:** **MEDIUM** (security detector accuracy primary concern)

**Key Changes:**
- ✅ Binary stability validated
- ✅ Core workflows confirmed functional
- ⚠️ New critical risk (R12) identified but mitigatable
- ⚠️ Test suite gaps documented with fix plan
- ✅ Release strategy adjusted to conditional approach

**Release Decision:** Proceed with CONDITIONAL GO per QA sign-off memo

---

**QA Execution Evidence:**
- Comprehensive QA Report: [reports/qa/qa-summary-2025-11-22.md](../reports/qa/qa-summary-2025-11-22.md)
- Defect Log: [reports/qa/defect-log-2025-11.csv](../reports/qa/defect-log-2025-11.csv)
- Release Sign-Off Memo: [reports/qa/release-candidate-signoff-memo.md](../reports/qa/release-candidate-signoff-memo.md)

---
**Last Updated:** 2025-11-22
