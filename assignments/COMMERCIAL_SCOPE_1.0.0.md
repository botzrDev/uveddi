# Uveddi Commercial Launch Scope – v1.0.0

**Owner:** Austin  
**Version Date:** 2025-10-09  
**Status:** Approved  

## 1. Release Intent
Deliver the Uveddi CLI 1.0.0 commercial package focused on deterministic architectural analysis for Rust, Python, JavaScript, and TypeScript repositories. The launch emphasizes reliability, actionable reporting, and supportable operations for paid subscribers.

## 2. In-Scope Capabilities
- **CLI commands:** `analyze`, `config`, `doctor`, `init`, `hooks`, `ci`, standard `help`.
- **Feature flags / build profiles:** `cli-standard` default; optional `cli-ai`, `cli-plugins`.
  - **REMOVED (A3):** `security` feature flag removed as empty/deprecated; security detectors are part of standard CLI.
- **Analysis engine:** Tree-sitter-backed multi-language parsing, anti-pattern detection, security scanning, debt scoring.
- **Outputs:** Markdown, JSON, and HTML reports; SARIF export; progress logging; Mermaid diagrams.
  - **ENABLED (A3):** HTML output format with interactive diagrams and dark/light themes included in 1.0.0.
- **Persistence:** SQLite-based caching and result storage, deterministic caching for repeat runs.
- **AI integration:** Optional Ollama support with configurable models and prompts.
- **Automation:** Git hook helpers, CI gate integration, environment diagnostics via `doctor`.
- **Documentation & enablement:** Updated README, CLI reference (`docs/CLI_REFERENCE.md`), troubleshooting guide (`docs/TROUBLESHOOTING.md`), onboarding quickstart.
- **Distribution:** Signed binaries for Linux/macOS/Windows, Docker image (production), cargo-based source build instructions.
- **Support readiness:** Email support channel, escalation path, launch-day war room guidelines.

## 3. Out-of-Scope / Deferred
- Web dashboard, REST API, and rendering service.
- Terminal UI / TUI workflows.
- SaaS-hosted AI or managed policy library (future roadmap).
- Advanced enterprise integrations (SSO, audit logging, multi-tenant analytics).
- Feature flags `web`, `tui`, `prometheus`, `chaos`, `memory-optimization`, and related subsystems.
- Automated license enforcement module (distributed separately per contract).
- Real-time telemetry/observability stack beyond lightweight opt-in diagnostics.

## 4. Launch KPIs & Targets
- **Installation success:** 95% success rate across smoke installs (Linux/macOS/Windows) during validation.
- **Analysis determinism:** ≤2% variance between repeated runs on canonical test repos.
- **Performance:** Analyze medium reference repo (<500 files) in ≤8 minutes on baseline hardware; memory usage ≤4 GB.
- **Detector precision:** Meet or exceed documented thresholds (e.g., security detectors ≥85% precision) on calibration suite.
- **Support readiness:** Response playbook with <1 business day initial response target; escalation drill completed.
- **Documentation completeness:** Quickstart executable without external guidance; zero critical doc gaps found during doc walkthrough.
- **Launch content:** Website copy, FAQ, pricing sheet, and announcement assets delivered before Phase A10.

## 5. Success Criteria
- All launch phases A1–A11 completed with verification evidence stored.
- Full automated test suite and targeted manual sweeps pass with zero critical defects open.
- Signed binaries + checksums published; Docker image pushed; release tag created.
- Support workflows operational (ticket inbox, runbooks, monitoring checklist).
- GTM assets approved and scheduled (announcement, email, pricing collateral).
- Post-launch metric instrumentation in place for adoption and issue tracking.

## 6. Dependencies & Inputs
- Detector calibration backlog (see `archive/internal-docs/DETECTOR_TOLERANCE_AUDIT.md`).
- Database abstraction work (see `database-refactor-verification.md`).
- Benchmarks and regression tests in `benches/` and `testing-uveddi/`.
- Alpha test artifacts located in `alpha-test-results/`.
- Legal/licensing review (external counsel) prior to distribution of paid binaries.

## 7. Risks & Assumptions
- Detector precision targets require completing outstanding calibration work.
- Existing Git history includes major refactors still in progress; coordination with future merges required.
- Solo bandwidth necessitates strictly sequential execution; schedule risk if phases overrun.
- Assumes no new critical defects emerge from customer pilots between now and release.
- Access to signing certificates and distribution channels is available when Phase A6 begins.

## 8. Sign-Off
- **Approved By:** Austin  
- **Date:** 2025-10-09  
- **Notes:** Scope, tracker, and risk register reviewed during Assignment A1 completion.

---
