
# Uveddi Copilot & AI Agent Instructions

## Project Architecture & Key Patterns
- **Layered Architecture:**
  - CLI (`src/cli/`): User interface, command parsing (no direct infra access)
  - Application (`src/main.rs`): Orchestrates workflows, config, error reporting
  - Analysis (`src/analysis/`): Core business logic, detectors, engines
  - Infrastructure (`src/ast/`, `src/ai/`, `src/database/`, `src/plugins/`, `src/api/`): AST, AI, DB, plugin, API, service orchestration
  - Platform: OS, filesystem, network
- **Strict Downward Dependencies:** Each layer only depends on the one below; use traits for interfaces, never concrete types.
- **Error Handling:** All errors flow up via the unified `UveddiError` type; add context at each layer.
- **Shared Types:** Use `src/api/types.rs` for cross-module types to prevent cycles.
- **Plugin System:** WASM-based, plugins communicate via defined API only, no direct FS/network access.

## Developer Workflows
- **Fast Dev Build:** `cargo build --features=dev-core --profile=dev-fast`
- **Production Build:** `cargo build --release --features=production`
- **Run Analysis:** `uveddi analyze <path> [--output-format json|html|markdown] [--output <file>]`
- **TUI:** `cargo run --features=community --bin tui_test`
- **Test All:** `cargo test --features=dev-core --profile=dev-fast`
- **Comprehensive Test Scripts:** See `scripts/` for full/coverage/performance/optimization test runners.
- **Config:** Use `uveddi.toml` or `uveddi config set/show/validate`.
- **AI Integration:** Requires Ollama running; set `OLLAMA_API_URL` and `OLLAMA_MODEL` env vars.

## Project-Specific Conventions
- **Feature Flags:** Use for modular builds (see `Cargo.toml` and docs for `dev-core`, `dev-minimal`, `production`, etc.).
- **Output Formats:** Always support `--output-format` and `--output` for CI/CD and automation.
- **Service Orchestration:** All services (API, rendering, frontend) start with `serve` command; readiness/health checks are automatic.
- **Testing:**
  - Unit: `tests/unit/`  
  - Integration: `tests/integration/`
  - TUI: `tests/tui_*.rs`
  - Security: `tests/security/`
  - Performance: `tests/performance/`
  - Coverage: `tests/coverage/`
- **CI/CD:** Use `cargo run --features=dev-minimal --profile=dev-fast -- analyze . --output-format json --output report.json` for fast CI builds.
- **Jira Integration:**
  - Reference issues in commits/branches: `UV-XXX` (e.g., UV-81)
  - Use `Closes UV-XXX` in PRs for auto-transition
  - See below for commit/branch/PR templates

## Integration & Extensibility
- **Plugins:** Place in `src/plugins/`, WASM only, follow API contract
- **AI Providers:** Add in `src/ai/providers/`, implement `LlmProvider` trait
- **Rendering Service:** Node.js, see `rendering-service/` for diagram/HTML generation
- **Frontend:** React dashboard (optional, dev mode only)
- **External Docs:**
  - User: `docs/03-user-guide/`
  - Dev: `docs/04-development/`
  - API: `docs/08-api/`
  - Architecture: `docs/01-architecture/`

## Example: Adding a Detector
1. Implement in `src/analysis/` (e.g., `god_object_detector.rs`)
2. Register in `DetectorRegistry`
3. Add tests in `tests/unit/` and `tests/integration/`
4. Document in `docs/` and update `README.md`

## Example: New Service Integration
1. Add orchestration logic in `src/service_orchestration/`
2. Expose API in `src/api/`
3. Add health checks, readiness, and error context

## Troubleshooting
- **Build slow?** Use `dev-minimal` or single-language features
- **Memory issues?** Enable `memory-optimization` or use `dev-fast` profile
- **TUI/Rendering issues?** Check terminal compatibility, run Playwright install scripts
- **AI not working?** Verify Ollama/model, check env vars
- **Circular deps?** Use shared types in `src/api/types.rs`

---

## Jira Integration Guidelines

### Issue Reference Format
- Reference Jira issues in commits: `UV-XXX` (e.g., UV-81)
- Include issue key in branch names: `feature/UV-81-build-system-fixes`
- Reference multiple issues: `Fixes UV-81, UV-96`

### Commit Message Standards
```bash
# Format: type(scope): description (ISSUE-KEY)
fix(visualization): resolve build system failures (UV-81)
feat(detector): implement magic values detection (UV-102)
docs(api): update visualization guide (UV-93)
```

### PR Description Template
```
Resolves UV-95: Missing ComponentType match arms

## Changes
- Added Class and Function match arms
- Updated pattern matching logic
- Added test cases

## Acceptance Criteria
- [x] All ComponentType variants handled
- [x] No compilation warnings
- [x] Tests pass

Closes UV-95
Related to UV-98
```

---

For more, see `README.md`, `CLAUDE.md`, and `docs/01-architecture/overview.md`.

  🎯 Jira Integration Instructions for GitHub Copilot                                ┃ │
│ ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛ │
│                                                                                                                      │
│  ## Jira Integration Guidelines                                                                                      │
│                                                                                                                      │
│  ### Issue Reference Format                                                                                          │
│  - Always reference Jira issues in commits using format: `UV-XXX` (e.g., UV-81, UV-96)                               │
│  - Include issue key in branch names: `feature/UV-81-build-system-fixes`                                             │
│  - Reference multiple issues when applicable: `Fixes UV-81, UV-96`                                                   │
│                                                                                                                      │
│  ### Commit Message Standards                                                                                        │
│  ```bash                                                                                                             │
│  # Format: type(scope): description (ISSUE-KEY)                                                                      │
│  fix(visualization): resolve build system failures (UV-81)                                                           │
│  feat(detector): implement magic values detection (UV-102)                                                           │
│  docs(api): update visualization guide (UV-93)                                                                       │
│                                                                                                                      │
│                                                                                                                      │
│                                              Jira Workflow Integration                                               │
│                                                                                                                      │
│  • To Do → In Progress: Start work, create branch                                                                    │
│  • In Progress → In Review: Create PR with issue reference                                                           │
│  • In Review → Done: Merge PR, auto-transition issue                                                                 │
│                                                                                                                      │
│                                                Issue Status Tracking                                                 │
│                                                                                                                      │
│  • Use Closes UV-XXX in PR descriptions for auto-transition                                                          │
│  • Reference related issues: Related to UV-98 (parent epic)                                                          │
│  • Link blocking issues: Blocked by UV-81                                                                            │
│                                                                                                                      │
│                                               Sprint Planning Context                                                │
│                                                                                                                      │
│  • Always check current sprint assignments before starting work                                                      │
│  • Prioritize P0 (Highest) issues in active sprint                                                                   │
│  • Consider issue dependencies and epic relationships                                                                │
│  • Estimate effort in Jira time tracking format                                                                      │
│                                                                                                                      │
│                                           Code Comments with Jira Context                                            │
│                                                                                                                      │
│                                                                                                                      │
│  // TODO: UV-102 - Implement magic values threshold configuration                                                    │
│  // FIXME: UV-97 - Add feature flag for tree-sitter dependency                                                       │
│  // NOTE: UV-81 - This resolves the missing visualization models                                                     │
│                                                                                                                      │
│                                                                                                                      │
│                                                Documentation Updates                                                 │
│                                                                                                                      │
│  • Update relevant docs when closing issues                                                                          │
│  • Reference Jira issue in documentation changes                                                                     │
│  • Maintain traceability between code and requirements                                                               │
│                                                                                                                      │
│                                              Epic and Subtask Handling                                               │
│                                                                                                                      │
│  • Understand epic relationships (e.g., UV-98 parent epic)                                                           │
│  • Complete subtasks before marking epic as done                                                                     │
│  • Track progress at both task and epic levels                                                                       │
│                                                                                                                      │
│                                                                                                                      │
│                                                                                                                      │
│  ---                                                                                                                 │
│                                                                                                                      │
│  ## 🔧 **Additional Copilot Prompts**                                                                                │
│                                                                                                                      │
│  Add these specific prompts for better Jira integration:                                                             │
│                                                                                                                      │
│  ```markdown                                                                                                         │
│  ### Jira-Aware Development Prompts                                                                                  │
│                                                                                                                      │
│  When suggesting code changes:                                                                                       │
│  - "Check if this change relates to any open Jira issues"                                                            │
│  - "Suggest appropriate Jira issue references for this commit"                                                       │
│  - "Identify if this fix resolves multiple related issues"                                                           │
│                                                                                                                      │
│  When reviewing code:                                                                                                │
│  - "Verify Jira issue references are correct and complete"                                                           │
│  - "Check if acceptance criteria from Jira are met"                                                                  │
│  - "Suggest additional test cases based on Jira requirements"                                                        │
│                                                                                                                      │
│  When planning work:                                                                                                 │
│  - "Review current sprint issues before suggesting new features"                                                     │
│  - "Consider Jira issue priorities and dependencies"                                                                 │
│  - "Estimate effort in Jira-compatible time formats"                                                                 │
│                                                                                                                      │
│                                                                                                                      │
│ ──────────────────────────────────────────────────────────────────────────────────────────────────────────────────── │
│                                                                                                                      │
│                                             📋 Quick Reference Commands                                              │
│                                                                                                                      │
│                                                                                                                      │
│  ### Jira Integration Quick Commands                                                                                 │
│                                                                                                                      │
│  # Check current sprint issues                                                                                       │
│  "Show me current sprint priorities from Jira context"                                                               │
│                                                                                                                      │
│  # Commit with proper Jira reference                                                                                 │
│  git commit -m "fix(build): resolve compilation errors (UV-81)                                                       │
│                                                                                                                      │
│  - Fix missing visualization models                                                                                  │
│  - Update import paths                                                                                               │
│  - Add required struct fields                                                                                        │
│                                                                                                                      │
│  Closes UV-81"                                                                                                       │
│                                                                                                                      │
│  # Branch naming convention                                                                                          │
│  git checkout -b feature/UV-95-component-type-match-arms                                                             │
│                                                                                                                      │
│  # PR description template                                                                                           │
│  "Resolves UV-95: Missing ComponentType match arms                                                                   │
│                                                                                                                      │
│  ## Changes                                                                                                          │
│  - Added Class and Function match arms                                                                               │
│  - Updated pattern matching logic                                                                                    │
│  - Added test cases                                                                                                  │
│                                                                                                                      │
│  ## Acceptance Criteria                                                                                              │
│  - [x] All ComponentType variants handled                                                                            │
│  - [x] No compilation warnings                                                                                       │
│  - [x] Tests pass                                                                                                    │
│                                                                                                                      │
│  Closes UV-95                                                                                                        │
│  Related to UV-98"                                                                                                   │
│                                              
