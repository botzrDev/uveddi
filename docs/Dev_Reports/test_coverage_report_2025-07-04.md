# Uveddi Test Coverage Analysis Report

**Date:** July 4, 2025

---

## Executive Summary
- **Total Test Coverage:** 39.83% (660/1657 lines covered)
- **Coverage Tool:** [cargo-tarpaulin](https://github.com/xd009642/tarpaulin)
- **Detailed Report:** See `coverage/tarpaulin-report.html` for a navigable, line-by-line breakdown.

---

## Coverage Overview
- The project currently covers less than half of its codebase with automated tests.
- Many critical modules, including core logic and public APIs, have little to no test coverage.
- Some modules demonstrate strong coverage, indicating good test practices in isolated areas.

---

## Coverage Breakdown

### Well-Covered Files (Partial/Good Coverage)
| File                                                        | Covered / Total Lines |
|-------------------------------------------------------------|----------------------|
| src/analysis/detectors/anti_patterns/code_duplication.rs    | 191 / 215            |
| src/analysis/detectors/anti_patterns/god_object.rs          | 90 / 110             |
| src/ast/tree_sitter/mod.rs                                  | 118 / 160            |
| src/ingestion/async_walker.rs                               | 31 / 38              |
| src/report/mod.rs                                           | 93 / 126             |
| src/analysis/graph/dependency.rs                            | 16 / 18              |
| src/analysis/detectors/dependency.rs                        | 59 / 66              |

### Major Gaps (0% or Near 0% Coverage)
- `src/ai/ollama_provider.rs`
- `src/ai/prompts/context_builder.rs`
- `src/ai/self_correction/mod.rs`
- `src/analysis/cache/ast.rs`
- `src/analysis/detectors/anti_patterns/*` (most files)
- `src/analysis/engine.rs`
- `src/application/mod.rs`
- `src/cli/analyze_command.rs`, `src/cli/config_command.rs`
- `src/config/mod.rs`
- `src/database/crud.rs`, `src/database/models.rs`
- `src/ingestion/file_scanner.rs`
- `src/models/dependency_graph.rs`, `src/models/mod.rs`
- `src/semantic_search/*`
- `src/main.rs`

---

## Observations
- **Unit Test Focus:** Some modules have strong unit test coverage, but this is not consistent across the codebase.
- **Integration Test Gaps:** Many integration points and end-to-end flows are not covered.
- **Ignored Tests:** A significant number of tests are marked as ignored, especially in language-specific analysis modules. These should be reviewed and enabled if possible.
- **Public API Coverage:** Many public-facing modules and CLI commands lack direct tests.

---

## Recommendations
1. **Prioritize Core Logic:** Begin by adding tests for `src/analysis/engine.rs`, `src/application/mod.rs`, and other core modules.
2. **Increase Integration Testing:** Add tests that exercise the system end-to-end, especially for CLI and database interactions.
3. **Review Ignored Tests:** Audit ignored tests and enable or update them as appropriate.
4. **Test Public APIs:** Ensure all public functions, especially those exposed via CLI or as library APIs, are covered.
5. **Continuous Monitoring:** Integrate coverage checks into CI to prevent regressions.

---

## Next Steps
- Review the full HTML report at `coverage/tarpaulin-report.html` for a detailed, file-by-file breakdown.
- Use the coverage summary above to prioritize test writing efforts.
- Re-run coverage after each major test addition to track progress.

---

*This report was generated automatically based on the latest coverage run. For questions or help with test writing, contact the development team.*
