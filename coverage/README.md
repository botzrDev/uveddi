# Uveddi Test Coverage Report (July 4, 2025)

## Summary
- **Total Coverage:** 39.83% (660/1657 lines covered)
- **Coverage Report:** See `coverage/tarpaulin-report.html` for a detailed, navigable report.

## Key Findings
- **Many modules have 0% coverage.**
- **Core logic in `src/analysis/engine.rs`, `src/application/mod.rs`, and most detectors are untested.**
- **Some files have partial or good coverage:**
  - `src/analysis/detectors/anti_patterns/code_duplication.rs`: 191/215
  - `src/analysis/detectors/anti_patterns/god_object.rs`: 90/110
  - `src/ast/tree_sitter/mod.rs`: 118/160
  - `src/ingestion/async_walker.rs`: 31/38
  - `src/report/mod.rs`: 93/126
  - `src/analysis/graph/dependency.rs`: 16/18
  - `src/analysis/detectors/dependency.rs`: 59/66

## Major Gaps (0% or near 0% coverage)
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

## Next Steps
- Review `coverage/tarpaulin-report.html` for a file-by-file breakdown.
- Prioritize adding tests for files/modules with 0% coverage, especially core logic and public APIs.
- Consider integration tests for end-to-end flows.

---

*This summary was generated automatically. For a full, interactive report, open `coverage/tarpaulin-report.html` in your browser.*
