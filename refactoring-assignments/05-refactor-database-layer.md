# Assignment Series: Database Layer Refactor

This refactor is broken into focused, reviewable slices. Complete them in order; each slice should land in its own PR/commit to keep review scope manageable.

## Assignment 01 – Isolate Models
- Extract current `src/database/models.rs` into dedicated files under `src/database/models/` (analysis, project, cache, etc.).
- Separate domain-facing structs from persistence-only records where needed.
- Update intra-crate imports; do not change behaviour yet.

## Assignment 02 – Extract Connection Infrastructure
- Move pooling and backend-specific code from `pool.rs`/`providers/` into a new `src/database/connection/` module.
- Introduce a lightweight `DatabaseConfig` and `DatabaseConnection` abstraction while keeping the old CRUD API working.
- Adjust existing modules to reference the new module paths.

## Assignment 03 – Introduce Repository Interfaces
- Create `src/database/repositories/` with traits (`ProjectRepository`, `AnalysisRepository`, `CacheRepository`, etc.) and initial SQLite implementations.
- Refactor `Database` in `crud.rs` to delegate through these repositories without changing public signatures yet.
- Add unit tests around repositories where practical.

## Assignment 04 – Update Application Integration
- Replace direct `Database` usage in application, plugin, and infrastructure layers with the new repository interfaces or a thin service struct.
- Introduce dependency injection where necessary; ensure no crate outside `database` imports implementation details.
- Update tests/benches to reflect the new API surface.

## Assignment 05 – Migration & Error Handling Cleanup
- Move migration logic into `src/database/migrations/` with versioned files and an up/down framework driven by the repository abstractions.
- Standardise database-specific error types (e.g. `DatabaseError`) and map them into `UveddiError` at module boundaries.
- Add smoke tests for migrations and error paths.

## Assignment 06 – Remove Legacy CRUD Layer
- With repositories and integrations stable, delete the legacy `crud.rs` entry points or convert them into thin compatibility shims slated for removal.
- Update documentation, examples, and README snippets to point to the new architecture.
- Run the verification commands (`cargo deny`, `cargo test database::`, migration dry-run) and capture results for review.
