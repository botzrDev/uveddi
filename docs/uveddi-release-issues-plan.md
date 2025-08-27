# Uveddi Release 1.0 – Must-Do vs Post-Release Issue Plan

This document classifies the current open Jira issues in the Uveddi (UV) project into what must be completed before a public production release versus what can be deferred. It is based on the repository state and the content of the open Jira issues.

## Headline numbers
- Open issues reviewed: ~117
- Must-do for public release: 12
- Can wait until after release: ~105

## Must-do issues for public release
The following items are required to ship a stable, secure, buildable v1 with accurate docs and CI gates. Where epics are listed, the scope here is the minimal subset needed for a safe public release.

1) UV-267 Critical Security & Stability Fixes (Epic)
   - Highest priority; foundational security/stability items required for public usage.

2) UV-98 Complete Anti-Pattern Detector Implementation (Epic)
   - Do the “fix all critical build issues” portion so the code compiles and core analysis/diagrams run. Non-blocking detector work can slip.

3) UV-179 Implement Comprehensive Test Suite for Rendering Service
   - Unit tests for the Node rendering-service are missing; needed for reliability.

4) UV-189 Enhanced Integration Testing – HTTP Client and Rendering Service
   - Validate cross-service timeouts, retries, error propagation, and happy-path.

5) UV-301 Implement Configuration Validation
   - Users need clear, fail-fast config validation (ties to current async validation pitfalls).

6) UV-149 Code Quality & Security Hardening (Epic)
   - Complete CRITICAL items only: remove unwrap panics on prod paths; path sanitization; input validation at boundaries.

7) UV-274 Add Security Test Suite
   - Minimal security coverage: auth, input validation, TLS/timeout on HTTP client, path traversal guards.

8) UV-236 Phase 1.3: GitHub Actions CI/CD Integration
   - Ensure CI gates: compile, tests, basic clippy, security scan, and packaging.

9) UV-293 Enhance Documentation Consistency
   - Remove inflated/inaccurate claims (coverage, speed, WASM readiness) and align docs with current reality.

10) UV-191 UV-203 – Clean Up Warnings and Lint
   - Reduce to a safe baseline (warn-as-error on core crates) to keep release quality.

11) UV-211 Critical Test Infrastructure Recovery (Epic)
   - Stabilize test infra so CI is trustworthy; only unblock consistent runs now.

12) UV-88 VIZ-010: API Documentation
   - Ensure REST/GraphQL docs align with actual endpoints and error codes (minimum viable, accurate reference).

## Strong candidates to defer until after release
- Performance and scalability
  - UV-285 (Tree-sitter query perf)
  - UV-281/UV-282/UV-283 (memory/batch/indexes)
  - UV-206/UV-207/UV-209 (perf suite, large codebase stability)
  - UV-284 (performance monitoring)
- Advanced features and integrations
  - UV-239/UV-237 (websocket infra, QuestDB)
  - UV-204/UV-201/UV-200/UV-199 (advanced resilience/caching and language expansions)
  - UV-205/UV-208 (parallel/incremental analysis)
- Community/UX/Docs depth
  - UV-240 (React dashboard MVP)
  - UV-33 (installation guide depth)
  - UV-29/UV-30 (contributor and viz docs depth)
  - UV-41 (severity scoring)
  - UV-70/UV-71/UV-72 (AI prompt efficiency/model recommendations/extra models)
  - UV-76/UV-77/UV-75 (tutorials/marketplace/best practices DB)
- Longer-term epics (broad scope, non-blocking)
  - UV-270/UV-269/UV-235/UV-196/UV-113/UV-111/UV-110/UV-109/UV-53/UV-52/UV-51/UV-50
  - Community Hub series: UV-126–UV-134

## JQL filters you can save in Jira
- Must-do for Release 1.0
  - `project = UV AND statusCategory != Done AND issuekey in (UV-267, UV-98, UV-179, UV-189, UV-301, UV-149, UV-274, UV-236, UV-293, UV-191, UV-211, UV-88)`

- Post-release backlog (open but not must-do)
  - `project = UV AND statusCategory != Done AND issuekey not in (UV-267, UV-98, UV-179, UV-189, UV-301, UV-149, UV-274, UV-236, UV-293, UV-191, UV-211, UV-88)`

## CI gates required for the release branch
- Rust
  - cargo build --workspace --all-features
  - cargo test --workspace
  - cargo clippy -- -D warnings (allow specific justified exceptions as needed)
  - cargo audit (allowlist only where strictly justified)
- Node services (api-server, rendering-service)
  - npm ci; npm test; npm audit (fail on high/critical)
- Frontend
  - npm ci; npm run build; npm test; npm audit
- Docs
  - Verify openapi.yaml matches REST/GraphQL; build docs site; broken links check
- Packaging
  - Build release artifacts, Docker images (non-root, minimal base), publish to registry

## Action checklist (immediate)
- [ ] Add a "release-blocker" label to the 12 must-do issues
- [ ] Create a saved filter “Uveddi Release 1.0 – Must Do” using the JQL above
- [ ] Move non-critical subtasks from must-do epics into a “post-release” label
- [ ] Make CI checks required on the release branch (see gates above)
- [ ] Open a docs PR to align claims and API reference (ties to UV-293 and UV-88)

## Notes on scope and interpretation
- Epics are broad; treat the items above as the minimal sub-scope to safely ship v1.0.
- Items explicitly planned under “Advanced - Post Release” are deferrable by default unless they directly impact build/security/docs/CI accuracy.
