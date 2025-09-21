# Security Subsystem Remediation Plan (Option 2)

## 1. Objective
Restore the entire security analysis stack (configuration, OWASP, taint analysis, validation, agent orchestration) to a compilable, testable, and documented state without removing existing features. Deliver a clean `cargo check`/`cargo test` and updated docs that reflect the reconstructed architecture.

## 2. Guiding Principles
- Prefer incremental reconstruction over wholesale rewrites; reuse surviving logic where possible.
- Keep public APIs stable, matching the shapes used throughout tests and documentation.
- Enforce compile-time safety first, then functional correctness, then performance.
- Maintain thorough observability: logs, metrics, and meaningful error propagation.
- Gate new work behind feature flags when the end-to-end path is incomplete.

## 3. Workstreams & Phased Execution

### Phase 0: Discovery & Architecture Sign-off
- Inventory every module under `src/analysis/detectors/security` and capture current compile errors.
- Produce dependency graph of modules (config, owaps, taint, validation, agents, CLI touchpoints).
- Align on target architecture diagram and module boundaries; document contracts (traits, data models).
- Decide on temporary feature flags (e.g., `security_full`) for incremental integration.

### Phase 1: Core Types & Settings Restoration
- Finalize `SecurityConfig`, `FalsePositiveConfig`, `MultiAgentConfig`, `TaintAnalysisConfig`, `LanguageConfig` to match tests and runtime usage.
- Ensure serde derives and `Default` impls exist where config loading relies on them.
- Reconcile all config consumers (CLI, orchestrator, validation, detectors) to the restored structures.
- Update docs (`docs/07-reference/...`) and examples to mirror the authoritative types.

### Phase 2: Configuration Detector Stack
- Rebuild pattern-matching infrastructure with the new in-crate `PatternMatcher` trait.
- Fix all misconfiguration analyzers (credentials, defaults, permissions, YAML/TOML/ENV analyzers) to reference the new trait and config helpers.
- Restore language-support utilities (shared `create_config_issue`, parsing helpers) and ensure they are publicly accessible.
- Write/repair unit tests for each checker (`debug_checker`, `session_checker`, `database_checker`, etc.).
- Add targeted integration tests covering representative config files.

### Phase 3: OWASP Detector Suite
- Reintroduce minimal viable analysis pipeline: category detectors, vulnerability detectors, scanners, orchestrator.
- Replace removed analysis/reporting pipelines with lightweight substitutes or feature-gate them until ready.
- Fix scanner trait implementations (no orphan methods, consistent signatures) and ensure they return valid `OwaspVulnerability` instances.
- Align re-exports and module wiring in `owasp/mod.rs`, `security/mod.rs`, and dependent crates.
- Author regression tests: per-detector unit tests and high-level orchestrator async tests.

### Phase 4: Taint Analysis Engine
- Stabilize `TaintAnalysisEngine` and supporting modules (sources, sinks, propagation, language adapters).
- Resolve exhaustive match warnings/errors by handling every `SourceLanguage` variant.
- Audit recursion/async usage (e.g., ensure helper routines are non-recursive or boxed).
- Validate data-flow graph construction and issue conversion pathways with realistic fixtures.
- Add fixtures for Rust, Python, JS, TS to cover cross-language behavior.

### Phase 5: Validation & False-Positive Mitigation
- Reconnect `ValidationEngine`, `FalsePositiveMitigator`, `BayesianOptimizer`, and calculators to the restored config types.
- Ensure async filtering stages compile and handle edge cases (empty issues, high volume, mixed severities).
- Restore tests under `tests/unit/security/validation_test.rs` and add new cases for the rebuilt heuristic/contextual/statistical filters.

### Phase 6: Multi-Agent Orchestrator & CLI Integration
- Bring `SecurityOrchestrator` and agents (`TaintAnalysisAgent`, `ConfigAnalysisAgent`, `DependencyAgent`, `ValidationAgent`) back to working order with corrected channel usage and task metadata.
- Verify the CLI and API layers compile (`SecurityDetector::minimal`, `for_ci_cd`, etc.), adjusting feature wiring where necessary.
- Update documentation, release notes, and CLI help to match the revived detector behavior.

### Phase 7: Final Integration & Quality Gates
- Execute full `cargo fmt`, `cargo clippy`, and `cargo test --all`.
- Run targeted profiling/benchmarking if available (e.g., `criterion` suites) to ensure no major regressions.
- Produce final architecture/update document summarizing changes and future maintenance guidelines.

## 4. Testing Strategy
- Unit tests for every restored module (per checker, per scanner, per agent).
- Async integration tests covering end-to-end security detector flows.
- Golden-file tests for config analyzers using sample YAML/TOML/ENV fixtures.
- Snapshot tests for OWASP vulnerability outputs (serialized JSON) to guard regressions.
- Continuous `cargo check --all-targets --all-features` and `cargo clippy --all-targets` in CI.

## 5. Tooling & Automation
- Introduce a dedicated `security` workspace feature to gate incremental builds.
- Add GitHub Actions matrix jobs focusing on security modules once they compile.
- Enable logging/tracing assertions in tests to guarantee observability hygiene.

## 6. Risk Assessment & Mitigations
- **Scope creep**: lock the plan to compilation + baseline functional parity before performance tuning.
- **Hidden API consumers**: search repo/tests for wildcard usages of removed types before changing signatures.
- **Complex async orchestration**: add tracing spans and synthetic tests to detect deadlocks or channel leaks early.
- **Timeline pressure**: sequence phases so that partially restored modules can be feature-gated to keep master buildable.

## 7. Deliverables & Exit Criteria
- Clean repository state where `cargo check`, `cargo test`, and `cargo clippy` succeed.
- Updated documentation (reference + dev guides) reflecting the revived architecture.
- Comprehensive test suite covering config, OWASP, taint, validation, and agent orchestration paths.
- Change log entry outlining the remediation and any deprecations or new feature flags.

## 8. Rough Timeline (adjust per bandwidth)
| Phase | Duration (est.) | Notes |
| --- | --- | --- |
| 0 | 0.5–1 day | Discovery & alignment |
| 1 | 0.5 day | Config scaffolding |
| 2 | 1.5 days | Config detector restoration |
| 3 | 2–3 days | OWASP stack revival |
| 4 | 2 days | Taint engine stabilization |
| 5 | 1 day | Validation pipeline |
| 6 | 1.5 days | Agents & CLI integration |
| 7 | 0.5 day | Final verification |

## 9. Dependencies & Coordination
- Confirm ownership of security modules (assign DRI for each phase).
- Coordinate with documentation and release managers for updated guides.
- Align with CI maintainers to adjust pipelines as feature gates change.
- Communicate progress via weekly updates or stand-ups, flagging blockers early.
