
# Code Review TODO List (2025-07-02)

This document lists the action items identified during the code review on 2025-07-02.

### High Priority

-   [ ] **Centralize Configuration:**
    -   **Task:** Implement a single, centralized configuration module.
    -   **Files:** `src/config/mod.rs`, and various other modules.
    -   **Reason:** To simplify configuration management and improve scalability.

-   [ ] **Standardize Error Handling:**
    -   **Task:** Refactor all functions to return `Result<T, E>` and handle errors consistently in the `main` function.
    -   **Files:** `src/error.rs`, various modules.
    -   **Reason:** To make error handling robust and predictable.

-   [ ] **Implement Database Transactions:**
    -   **Task:** Wrap all database write operations in transactions.
    -   **Files:** `src/database/mod.rs`.
    -   **Reason:** To ensure data consistency.

-   [ ] **Pin Dependency Versions:**
    -   **Task:** Replace wildcard versions in `Cargo.toml` with specific, locked versions.
    -   **Files:** `Cargo.toml`.
    -   **Reason:** To ensure reproducible builds and prevent unexpected breakages.

### Medium Priority

-   [ ] **Refactor `main.rs` Orchestration:**
    -   **Task:** Move business process orchestration logic from `main.rs` into the `src/application` module.
    -   **Files:** `src/main.rs`, `src/application/mod.rs`.
    -   **Reason:** To improve separation of concerns and reusability of core logic.

-   [ ] **Introduce Traits for Infrastructure Layer:**
    -   **Task:** Create and implement traits for the Database, AI, and AST services.
    -   **Files:** `src/analysis/engine.rs`, `src/database/mod.rs`, `src/ai/mod.rs`, `src/ast/mod.rs`.
    -   **Reason:** To decouple the analysis engine from concrete implementations, improving testability and flexibility.

-   [ ] **Integrate Security Checks:**
    -   **Task:** Actively use the checks from `src/security.rs` within the application's core logic.
    -   **Files:** `src/security.rs`, `src/analysis/dependency_extractor.rs`.
    -   **Reason:** To make security checks effective.

-   [ ] **Create Test Utilities Module:**
    -   **Task:** Create a `test_utils` module to share common test setup logic.
    -   **Files:** `tests/`.
    -   **Reason:** To reduce code duplication in integration tests (DRY).

### Low Priority

-   [ ] **Optimize Dependency Extraction:**
    -   **Task:** Investigate and implement a more efficient algorithm for dependency extraction.
    -   **Files:** `src/analysis/dependency_extractor.rs`.
    -   **Reason:** To improve performance on large codebases.

-   [ ] **Improve Inline Documentation:**
    -   **Task:** Add more detailed comments to internal functions and modules.
    -   **Files:** Throughout the codebase.
    -   **Reason:** To improve maintainability and ease of onboarding for new developers.

-   [ ] **Remove Unused Benchmarks:**
    -   **Task:** Delete benchmark files for features that are not yet implemented.
    -   **Files:** `benches/`.
    -   **Reason:** To reduce code clutter and confusion (YAGNI).
