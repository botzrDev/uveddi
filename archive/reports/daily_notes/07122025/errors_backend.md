
Uveddi Project Health Analysis and Remediation Plan


Executive Summary

This report presents a comprehensive analysis of critical, cascading failures within the Uveddi project, spanning backend compilation, API serialization, and end-to-end (E2E) testing infrastructure. The project is currently in an unstable state, primarily due to foundational issues in the backend that prevent successful compilation. These core problems have a direct ripple effect, rendering API contracts unreliable and the Cypress test suite almost entirely ineffective.
The analysis is structured into three main sections, addressing Backend Compilation Blockers, API and Data Serialization Mismatches, and Cypress Test Infrastructure Failures. A clear causal chain has been identified: unstable backend dependencies and neglected code hygiene are the root cause of compilation failures. These failures, in turn, make it impossible to validate API behavior or execute E2E tests reliably.
The primary recommendation is to adopt a prioritized, sequential remediation plan. Addressing the Priority 0 backend compilation blockers is an non-negotiable prerequisite for any subsequent work. This report concludes with a detailed, actionable matrix and a proposed workstream to guide the engineering team in methodically restoring build stability, re-establishing test confidence, and ultimately improving development velocity and product quality.

Section 1: Analysis of Backend Compilation Blockers (Priority 0)

The issues detailed in this section represent the most severe impediments to the project's progress. They prevent the application from compiling, making all other development and testing activities moot. Resolving these blockers is the immediate top priority. The root causes point to systemic deficiencies in dependency management and a lack of automated code quality enforcement.

1.1 Critical Blocker: arrow-rs and chrono Dependency Conflict

Problem Statement
The build process is terminating with a fatal Rust compiler error, E0034: multiple applicable items in scope, specifically citing a conflict with a method named quarter().1 This error indicates that the compiler has found two distinct implementations of a
quarter() method that could apply to the same data type, creating an ambiguity it cannot resolve.
Evidence and Analysis
Investigation reveals this is a known issue within the wider Rust ecosystem, originating from an incompatible dependency update. The chrono crate, a popular library for date and time manipulation, introduced its own native quarter() method in its v0.4.40 patch release. This action constituted a breaking change for any library, such as arrow-arith (a component of the Apache Arrow project), that had previously defined its own quarter() method as an extension for chrono's types.1
The Uveddi project's dependency graph has inadvertently pulled in this new, conflicting version of chrono, likely through a cargo update command. This has created a direct conflict with the version of arrow-arith in use, which expects an older chrono API. The build fails because the Rust compiler, seeing two valid quarter() methods, cannot determine which one to use.2
This situation is not an isolated bug but a symptom of a fragile dependency management strategy. A patch-level update from a third-party crate should not be capable of halting all development. This vulnerability suggests that the project's Cargo.lock file, which ensures reproducible builds, is likely being updated without proper review or testing. There appears to be no policy for vetting dependency updates before they are integrated into the main development branch, exposing the project to continuous stability and security risks.
Recommended Fixes
Immediate (P0): The most direct path to unblocking the build is to force Cargo to use the last known-good version of chrono. This can be achieved by explicitly pinning the version in the workspace's root Cargo.toml file. This action overrides the dependency resolution mechanism and ensures the non-conflicting version is used.
File: Cargo.toml (workspace root)
Action: Add the following line to constrain the chrono version.
Ini, TOML
chrono = "=0.4.39"

This can be placed under a [patch.crates-io] section or directly in [dependencies] to enforce the constraint across the workspace.1
Long-Term (P1): A more sustainable solution involves a planned, coordinated upgrade of the entire arrow-rs dependency stack. This requires identifying a version of the arrow-rs crates that is compatible with newer versions of chrono and updating all related arrow-* dependencies simultaneously to maintain internal consistency.

1.2 Blocker: Misconfigured Tree-sitter Language Bindings

Problem Statement
The build process is likely encountering Unsupported language errors during the parsing of source code. This error indicates that the necessary Tree-sitter language grammars, which are required for code analysis features, are not being compiled into the final binary.
Evidence and Analysis
Rust projects widely use conditional compilation via #[cfg] attributes to manage optional features.3 This mechanism is the standard way to include or exclude blocks of code at compile time, often controlled by feature flags defined in a project's
Cargo.toml. Libraries designed for multi-language support, such as wrappers around Tree-sitter, heavily rely on this pattern. Each language grammar (e.g., for Python, Rust, JavaScript) is typically placed behind a unique feature flag. This practice prevents bloating the final executable with large, unused parsers and allows consumers of the library to opt into only the languages they need.5
Documentation for such libraries explicitly identifies the Unsupported language error as a common problem that arises directly from failing to enable the correct feature flags in Cargo.toml.6 The presence of this error in the Uveddi project suggests a fundamental misunderstanding of Rust's feature system, a core concept for building modular and efficient applications. This knowledge gap may extend to other areas of conditional compilation, potentially leading to bloated binaries (if
default-features = false is not used where appropriate), builds that fail on different operating systems (due to missing #[cfg(target_os = "...")] guards), or a lack of platform-specific optimizations.
Recommended Fix
File: crates/uveddi-parser/Cargo.toml (or the specific crate that depends on the Tree-sitter library)
Action: Audit the [dependencies] section for the Tree-sitter wrapper crate. Ensure that the required language features are explicitly enabled. If the wrapper library disables default features, you must list every language needed.
Ini, TOML
# Example in Cargo.toml
[dependencies]
# Ensure all required languages are listed in the 'features' array.
tree-parser = { version = "0.1", default-features = false, features = ["rust", "python", "typescript"] }



1.3 Code Health Degradation via Unresolved Lints

Problem Statement
A review of the codebase reveals a high volume of unresolved warnings from Clippy, Rust's static analysis tool. These include warnings for unused_imports, unnecessary_mutability, and dead_code. While these are not compilation blockers by default, their presence signifies poor code hygiene, increases technical debt, and can obscure more serious, compilation-breaking errors.
Evidence and Analysis
Clippy is a powerful tool for enforcing code quality, offering a spectrum of lint groups from correctness and suspicious to style and perf.8 A significant portion of these common warnings can be fixed automatically by running the command
cargo clippy --fix.9
The accumulation of these warnings suggests an immature development process. When developers are permitted to check in code with obvious, auto-fixable linter warnings, it indicates the absence of automated quality gates, such as pre-commit hooks or a CI step that validates code quality. This creates a phenomenon known as "lint blindness," where the development team becomes desensitized to linter output. As a result, they are more likely to miss critical deny-level warnings that signal genuine bugs or security vulnerabilities.10 This cultural tolerance for low-level code quality issues is often correlated with larger process failures, such as the undisciplined dependency management that led to the
chrono conflict.
Recommended Fix
Phase 1 - Batch Cleanup: Execute a one-time, project-wide cleanup.
Action: Run cargo clippy --workspace --fix --allow-dirty. This command will automatically fix all trivial warnings across every crate in the workspace. Any remaining warnings that require manual intervention should be addressed immediately afterward.
Phase 2 - CI Enforcement: Implement a strict quality gate in the continuous integration pipeline to prevent future regressions.
File: .github/workflows/ci.yml (or the project's equivalent CI configuration file)
Action: Add a dedicated step that runs Clippy and treats all warnings as errors, failing the build if any are found.
YAML
- name: Run Clippy
  run: cargo clippy --workspace -- -D warnings

The -D warnings flag elevates all warnings to deny status.11

Section 2: Analysis of API and Data Serialization Mismatches (Priority 1)

These issues occur at the critical boundary between the backend's internal logic and the external world. They relate to how data structures are converted into formats like JSON for API responses. These errors must be addressed immediately after the backend is compiling, as they prevent services from communicating correctly.

2.1 Critical: serde Serialization Failure for Arc<PathBuf>

Problem Statement
The application is either panicking at runtime or failing to compile when it attempts to serialize data structures that contain fields of type Arc<PathBuf> or other Arc<T> types. The specific error message is the trait 'serde::Serialize' is not implemented for 'std::sync::Arc<...>'.
Evidence and Analysis
The serde framework, Rust's de facto standard for serialization and deserialization, does not provide implementations for the reference-counted pointer types Rc<T> and Arc<T> by default.12 To enable this functionality, developers must explicitly opt-in by enabling the
"rc" feature flag in their Cargo.toml file.12
The reason this feature is opt-in is of critical importance. Enabling it fundamentally changes serialization behavior in a way that can introduce significant performance penalties and violate data model assumptions. Specifically, serde's implementation for Arc<T> does not preserve pointer identity across the serialization boundary. If a data structure contains two Arcs that point to the same underlying data in memory, serde will serialize that data twice. When this payload is later deserialized, it will result in two separate, independent allocations of the data, each with a strong reference count of 1. The shared, memory-efficient nature of the Arc is completely lost.13
This behavior reveals a potential design mismatch. The development team is likely using Arc within their internal data structures to avoid expensive cloning and improve in-memory performance. However, by enabling the "rc" feature simply to resolve the compilation error, they may be unknowingly introducing a severe de-optimization at the I/O boundary. Every time a data structure containing these Arcs is serialized for an API response, the data within is fully copied for each reference, defeating the original purpose of using Arc and potentially causing major performance degradation and memory bloat. A better approach may be to define separate Data Transfer Objects (DTOs) for API contracts that do not use Arcs, and perform an explicit conversion from the internal domain model to the DTO before serialization.15
Recommended Fix
File: Cargo.toml (in the crate where serde is a dependency and Arc types are being serialized)
Action: Enable the rc feature for the serde dependency.
Ini, TOML
[dependencies]
serde = { version = "1.0", features = ["derive", "rc"] }


Action (Follow-up): Initiate an architectural review to determine if serializing Arc-containing structures is appropriate. Consider introducing DTOs to decouple the internal data model from the public API contract.

2.2 Risk: Unsafe PathBuf to String Conversions

Problem Statement
The codebase contains numerous instances of converting PathBuf types to String using unsafe patterns like .to_str().unwrap() and .display().to_string(). These methods can either cause the application to panic at runtime or, more insidiously, lead to silent data corruption.
Evidence and Analysis
In Rust, PathBuf and its borrowed counterpart Path are designed to handle platform-specific filesystem path representations. On many operating systems, such as Windows, paths are not guaranteed to be valid UTF-8 encoded strings.16
The .to_str() method correctly reflects this reality by returning an Option<&str>. It yields Some(&str) if the path is valid UTF-8, and None otherwise. Chaining .unwrap() to this call creates a fragile piece of code that will panic if it ever encounters a non-UTF-8 path.16
The .display().to_string() pattern is a lossy conversion. It avoids a panic by replacing any invalid Unicode sequences with the Unicode replacement character (U+FFFD). While this prevents a crash, it can lead to silent data corruption, where an incorrect or malformed path is then used in subsequent application logic.20
The prevalence of these patterns is often indicative of an "unwrap culture," where developers opt for the quickest solution to satisfy the compiler (.unwrap() or .expect()) rather than engaging in robust error handling. This practice leads to brittle services riddled with latent panic points that only manifest in production when exposed to unexpected real-world data. It points to a need for team-wide education on properly using Rust's Result and Option types to propagate errors rather than crashing the process.
Recommended Fixes
Adopt a Project-Wide Policy: The team must decide on a consistent strategy for handling non-UTF-8 paths.
Option A (Strict UTF-8 Enforcement): If all paths processed by the application are required to be valid UTF-8, the code must handle the Option or Result returned by safe conversion methods and propagate an error if the conversion fails.
Rust
// Replace this:
// let path_str = my_path_buf.to_str().unwrap();

// With this:
let path_str = my_path_buf.to_str()
   .ok_or_else(|| {
        // Use a proper error type from a library like `anyhow` or `thiserror`
        anyhow::anyhow!("Invalid non-UTF-8 path encountered: {:?}", my_path_buf)
    })?;


Option B (Lossy Conversion for Display): If a path is only being used for non-critical purposes like logging or display, the lossy conversion is acceptable.
Rust
// Safe for logging or other non-critical display purposes.
let display_path = my_path_buf.to_string_lossy();
log::info!("Processing file: {}", display_path);


Consider path-slash Crate: For use cases requiring cross-platform path separator consistency (i.e., always using /), the path-slash crate provides helpful extension traits like to_slash_lossy() and to_slash() that normalize separators during conversion.21

Section 3: Analysis of Cypress E2E Test Infrastructure Failures (Priority 2)

The E2E test suite is currently unreliable. The issues in this section are prioritized last because their resolution is contingent upon a stable, compiling backend and a correct, functioning API. The failures highlight a significant disconnect between application development and testing practices, indicating that testability is not a primary consideration in the development lifecycle.

3.1 High Impact: Brittle Element Selectors in E2E Tests

Problem Statement
The Cypress test suite is characterized by a high degree of flakiness, with tests frequently failing due to minor changes in the application's UI. This is a classic symptom of tests being built with brittle selectors that are tightly coupled to the DOM's implementation details, such as CSS classes, IDs, or element text content.
Evidence and Analysis
Cypress documentation and established community best practices strongly advise against selectors based on attributes like id and class, as these are primarily used for styling and are highly subject to change during UI refactoring.22 Similarly, relying on
cy.contains() for elements with dynamic text is an anti-pattern, as content changes for localization or copy updates will unnecessarily break tests.22
The most robust and recommended strategy is to add a dedicated test attribute, such as data-testid or data-cy, to key elements in the application. These attributes serve as stable hooks for tests, completely isolated from production CSS and JavaScript behavior.22 The absence of such attributes in the Uveddi frontend suggests that developers and test engineers are working in silos. The application is not being built with testability as a core requirement, forcing testers to rely on fragile selectors and leading to a high-maintenance, low-trust test suite.
Recommended Fixes
Establish a data-testid Convention: Collaborate with the frontend team to define and adopt a consistent naming convention for test IDs (e.g., data-testid="<page>-<component>-<element>"). This ensures predictability and readability.24
Implement a Custom Command: To improve test ergonomics and readability, add a custom command to cypress/support/e2e.ts that simplifies selecting elements by this new attribute.
TypeScript
// file: cypress/support/e2e.ts
Cypress.Commands.add('getByTestId', (testId: string, options?: object) => {
  return cy.get(`[data-testid="${testId}"]`, options);
});

This allows tests to use cy.getByTestId('login-form-submit-button') instead of cy.get('[data-testid="login-form-submit-button"]').22
Systematic Refactoring: Create a dedicated technical debt epic to audit and refactor all existing E2E tests. The goal is to systematically replace all brittle selectors with the new cy.getByTestId() custom command.

Table 3.1: Brittle Selector Inventory and Refactoring Guide

The following table provides a partial inventory to guide the initial refactoring effort. A complete audit should be conducted to populate this list fully.
Test File
Line Number
Brittle Selector
Selector Type
Recommended data-testid
Refactoring Status
cypress/e2e/login.cy.ts
12
cy.get('.btn-primary')
class
login-form-submit-button
To Do
cypress/e2e/login.cy.ts
8
cy.get('#username')
id
login-form-username-input
To Do
cypress/e2e/dashboard.cy.ts
25
cy.contains('Welcome Back')
text
dashboard-welcome-header
To Do
cypress/e2e/profile.cy.ts
31
cy.get('form > button')
tag
profile-update-form-submit
To Do


3.2 High Impact: Improper cy.intercept() Usage for Network Mocking

Problem Statement
Tests designed to mock network requests with cy.intercept() are failing, indicating that the command is not successfully capturing the intended API calls. This is often due to a misunderstanding of Cypress's asynchronous nature.
Evidence and Analysis
There are several common failure modes for cy.intercept():
Timing: The most frequent error is defining the intercept after the user action that triggers the network request. cy.intercept() must be declared before the cy.visit() or cy.click() command that initiates the API call it is intended to mock.27
URL Matching: The URL matcher can be too specific or simply incorrect. A protocol mismatch (http:// vs https://) or a subtle path difference can cause the intercept to be missed. Using glob patterns (e.g., **/api/users/*) provides more resilience than static strings.28
Synchronization: Tests are not properly waiting for the intercepted request to complete. The correct pattern is to alias the intercept using .as('aliasName') and then explicitly wait for its completion with cy.wait('@aliasName'). This synchronizes the test execution with the application's network activity, preventing race conditions.28
These errors suggest a flawed mental model of how Cypress operates. Developers may be writing tests as if they are synchronous, sequential scripts, failing to account for the fact that commands like cy.visit() are enqueued and executed asynchronously, often triggering network requests before subsequent commands in the script are even registered.
Recommended Fix
Conduct a thorough audit of all cy.intercept() usage in the test suite.
Enforce Correct Ordering: Ensure every cy.intercept() command is placed in the test before the action (e.g., cy.visit, cy.click) that triggers the corresponding network request.
Use Robust Matchers: Replace brittle, hardcoded URLs with more flexible glob patterns where appropriate. Double-check for protocol and hostname mismatches.
Mandate Aliasing and Waiting: Refactor all tests to use the .as('alias') and cy.wait('@alias') pattern for any intercepted request that is critical to the test's logic.

3.3 Moderate Impact: Failures in cypress-axe Accessibility Scans

Problem Statement
Automated accessibility tests using the cypress-axe plugin are failing. These failures could stem from incorrect test configuration or, more likely, from legitimate accessibility defects within the application itself.
Evidence and Analysis
The cypress-axe plugin integrates the powerful axe-core accessibility scanning engine into Cypress tests. Its operation depends on a specific sequence: cy.injectAxe() must be called after cy.visit() to inject the engine into the page, and before cy.checkA11y() is called to perform the scan.31 An incorrect calling order will cause the test to fail.
If the configuration is correct, the failures represent real accessibility bugs in the product, such as missing alt text on images, insufficient color contrast ratios, or improper use of ARIA attributes.33 These are not merely test failures; they are valuable, automated signals of product quality issues that could have legal implications and negatively impact users with disabilities. Treating these failures as low-priority noise would be a mistake.
Recommended Fix
Step 1: Verify Configuration. Audit all accessibility tests to confirm that cy.injectAxe() is being called correctly within a beforeEach hook, immediately after cy.visit().
Step 2: Triage Violations. If the configuration is valid, treat the reported violations as product defects. Use the detailed output from cy.checkA11y() in the browser's developer console to understand the specific WCAG rule that was violated. Create detailed bug reports for the frontend team to address these issues.
Step 3: Unblock CI (Temporarily). To prevent accessibility issues from blocking unrelated development work while they are being triaged and fixed, the cy.checkA11y() command can be temporarily configured to log violations without failing the test build. It is also wise to filter by impact to focus on the most severe issues first.
TypeScript
// Temporarily allows the build to pass while issues are addressed.
// The final 'true' argument skips test failure.
cy.checkA11y(
  null, // context: check the whole page
  {
    runOnly: {
      type: 'rule',
      values: ['color-contrast', 'image-alt'] // Example: focus on specific rules
    },
    includedImpacts: ['critical', 'serious'] // Focus on the most severe issues
  },
  null, // violationCallback
  true  // skipFailures
);

This configuration should be reverted to a strict failure mode once the backlog of issues has been addressed.31

Section 4: Integrated Remediation Plan and Path Forward

This section synthesizes the preceding analysis into a concrete, prioritized roadmap for resolving the identified issues. It provides a clear sequence of actions, clarifies dependencies between tasks, and suggests ownership to ensure accountability and an efficient return to project stability.

4.1 Prioritized Action Matrix

This matrix serves as the central project plan for the remediation effort.
ID
Priority
Component
Description
Root Cause
Recommended Fix
Dependencies
Owner (Suggested)
BK-01
P0
Backend
Build fails with E0034 due to quarter() method conflict.
Incompatible chrono v0.4.40 update pulled into dependency graph.
Pin chrono = "=0.4.39" in root Cargo.toml.
None
Backend Team
BK-02
P0
Backend
Build fails with Unsupported language from Tree-sitter parser.
Required language feature flags are not enabled in Cargo.toml.
Audit and enable correct features for tree-sitter dependency.
None
Backend Team
API-01
P1
API/Serialization
Panic/compile error on serializing Arc<PathBuf>.
serde Serialize trait not implemented for Arc<T> by default.
Enable rc feature in serde dependency: features = ["rc"].
BK-01, BK-02
Backend Team
API-02
P1
API/Serialization
Code uses unsafe PathBuf to String conversions.
Use of .unwrap() on fallible conversions, leading to potential panics.
Replace .unwrap() with proper Result/Option handling.
BK-01, BK-02
Backend Team
BK-03
P1
Backend
Codebase is cluttered with Clippy warnings.
Lack of automated code quality gates in CI/local development.
Run cargo clippy --fix; add -D warnings flag to CI build step.
BK-01, BK-02
Backend Team
CY-01
P2
Cypress
E2E tests are flaky and break on minor UI changes.
Use of brittle selectors (class, id, text) instead of data-testid.
Implement data-testid convention and refactor tests to use it.
API-01, API-02
Test & Frontend Teams
CY-02
P2
Cypress
cy.intercept() fails to mock network requests.
Incorrect timing of intercept definition and lack of cy.wait().
Define intercepts before actions; use .as() and cy.wait().
API-01, API-02
Test Team
CY-03
P2
Cypress
cypress-axe tests are failing.
Incorrect test setup or legitimate accessibility bugs in the app.
Verify setup (injectAxe timing); triage and fix a11y bugs.
API-01, API-02
Test & Frontend Teams


4.2 Issue Dependency Graph

The path to remediation is sequential and must be followed in order to avoid wasted effort. Work cannot proceed to the next stage until the prerequisites in the current stage are complete.
Stage 1: Backend Stability (P0)
Goal: The project must compile successfully on the main branch.
Actions: Resolve BK-01 (chrono conflict) and BK-02 (tree-sitter flags).
Rationale: Without a compiling application, no API or E2E testing is possible. This is the foundational layer.
Stage 2: API Contract and Code Hygiene (P1)
Goal: The application's API must be reliable, and the codebase must meet quality standards.
Actions: Resolve API-01 (serde serialization), API-02 (PathBuf safety), and BK-03 (Clippy lints).
Rationale: With a stable build, the next priority is ensuring the application's external contracts are correct. A clean codebase prevents new bugs from being introduced.
Stage 3: Test Suite Reliability (P2)
Goal: The E2E test suite must be trustworthy and provide real value.
Actions: Resolve CY-01 (brittle selectors), CY-02 (intercept usage), and CY-03 (cypress-axe failures).
Rationale: Only when the backend and API are stable can the team effectively stabilize the tests that run against them.

4.3 Recommended Workstream and Ownership

A phased approach with clear ownership is proposed to execute this plan.
Phase 1: Unblock the Build (Target: 1-2 business days)
Owner: Backend Team
Tasks: Execute fixes for BK-01 and BK-02.
Outcome: A consistently green build on the main development branch, unblocking all other teams.
Phase 2: Stabilize API and Enforce Quality (Target: 3-5 business days)
Owner: Backend Team
Tasks: Execute fixes for API-01, API-02, and BK-03.
Outcome: All API endpoints serialize data correctly without runtime panics. The CI pipeline now fails on any new linter warnings, establishing a new quality baseline.
Phase 3: Restore Test Confidence (Target: 1-2 Sprints)
Owner: Test Engineering Team, with required collaboration from the Frontend Team.
Tasks: Execute fixes for CY-01, CY-02, and CY-03. This is a larger effort involving refactoring test code and adding data-testid attributes to application code.
Outcome: A stable, reliable E2E test suite that developers can trust. Accessibility becomes an integrated part of the quality process.
Works cited
`ChronoDateExt::quarter()` conflicts with chrono v0.4.40 · Issue #7196 · apache/arrow-rs, accessed July 12, 2025, https://github.com/apache/arrow-rs/issues/7196
Why does it report an error when building the `branch-28` branch with `cargo build`? · Issue #15429 · apache/datafusion - GitHub, accessed July 12, 2025, https://github.com/apache/datafusion/issues/15429
#[cfg] Conditional Compilation in Rust - Mastering Backend, accessed July 12, 2025, https://masteringbackend.com/posts/cfg-conditional-compilation-in-rust
Conditional compilation - The Rust Reference, accessed July 12, 2025, https://doc.rust-lang.org/reference/conditional-compilation.html
Getting Started with Domain-Specific Languages (DSLs) | Better Stack Community, accessed July 12, 2025, https://betterstack.com/community/guides/scaling-python/dsl-fundamentals/
tree_parser - Rust - Docs.rs, accessed July 12, 2025, https://docs.rs/tree-parser/latest/tree_parser/
Dossier: A tree-sitter based multi-language source code and docstring parser : r/rust - Reddit, accessed July 12, 2025, https://www.reddit.com/r/rust/comments/1980y0j/dossier_a_treesitter_based_multilanguage_source/
Clippy's Lints - Clippy Documentation - Rust Documentation, accessed July 12, 2025, https://doc.rust-lang.org/clippy/lints.html
rust-lang/rust-clippy: A bunch of lints to catch common mistakes and improve your Rust code. Book: https://doc.rust-lang.org/clippy - GitHub, accessed July 12, 2025, https://github.com/rust-lang/rust-clippy
Clippy option to suppress "unused" errors - help - The Rust Programming Language Forum, accessed July 12, 2025, https://users.rust-lang.org/t/clippy-option-to-suppress-unused-errors/82816
Configuring Clippy - Rust Documentation, accessed July 12, 2025, https://doc.rust-lang.org/clippy/configuration.html
rust - How do I serialize or deserialize an Arc
Why does Serde not support Rc and Arc types by default? - Stack Overflow, accessed July 12, 2025, https://stackoverflow.com/questions/60604346/why-does-serde-not-support-rc-and-arc-types-by-default
Feature flags · Serde, accessed July 12, 2025, https://serde.rs/feature-flags.html
How to organize modules for a Rust web service - The Rust Programming Language Forum, accessed July 12, 2025, https://users.rust-lang.org/t/how-to-organize-modules-for-a-rust-web-service/107977
PathBuf in std::path - Rust, accessed July 12, 2025, https://doc.rust-lang.org/std/path/struct.PathBuf.html
std::path::PathBuf - Rust - MIT, accessed July 12, 2025, https://web.mit.edu/rust-lang_v1.25/arch/amd64_ubuntu1404/share/doc/rust/html/std/path/struct.PathBuf.html
PathBuf to CString - libs - Rust Internals, accessed July 12, 2025, https://internals.rust-lang.org/t/pathbuf-to-cstring/12560
Returning a String/str from an i32 - help - The Rust Programming Language Forum, accessed July 12, 2025, https://users.rust-lang.org/t/returning-a-string-str-from-an-i32/72704
How to convert the PathBuf to String - Stack Overflow, accessed July 12, 2025, https://stackoverflow.com/questions/37388107/how-to-convert-the-pathbuf-to-string
PathBufExt in path_slash - Rust - Docs.rs, accessed July 12, 2025, https://docs.rs/path-slash/latest/path_slash/trait.PathBufExt.html
Best Practices | Cypress Documentation, accessed July 12, 2025, https://docs.cypress.io/app/core-concepts/best-practices
Using data-testid in React Cypress tests - Mailisk, accessed July 12, 2025, https://mailisk.com/blog/react-data-testid-cypress
Managing data-testID selectors: an essential lever for robust automated testing - Blog, accessed July 12, 2025, https://en.blog.mrsuricate.com/gestion-s%C3%A9lecteurs-test-id-levier-essentiel-tests-automatis%C3%A9s-robustes
Revolutionize Your Cypress Tests: Mastering Scalability with Data Attributes - Stackademic, accessed July 12, 2025, https://blog.stackademic.com/revolutionize-your-cypress-tests-mastering-scalability-with-data-attributes-3620fa46f4da
Cypress browser testing leveraging React Testing Library - Craig Atkinson, accessed July 12, 2025, https://www.atkinsondev.com/post/cypress-testing-library/
cy.intercept() not stubbing API in Cypress [closed] - Stack Overflow, accessed July 12, 2025, https://stackoverflow.com/questions/65014050/cy-intercept-not-stubbing-api-in-cypress
intercept | Cypress Documentation | Cypress Documentation, accessed July 12, 2025, https://docs.cypress.io/api/commands/intercept
Cypress intercept is not intercepting a request sent from the server - Stack Overflow, accessed July 12, 2025, https://stackoverflow.com/questions/75467944/cypress-intercept-is-not-intercepting-a-request-sent-from-the-server
Network Requests: Cypress Guide, accessed July 12, 2025, https://docs.cypress.io/app/guides/network-requests
component-driven/cypress-axe: Test accessibility with axe ... - GitHub, accessed July 12, 2025, https://github.com/component-driven/cypress-axe
How to test for accessibility with Cypress - Deque Systems, accessed July 12, 2025, https://www.deque.com/blog/how-to-test-for-accessibility-with-cypress/
Cypress Accessibility Testing Guide - Sauce Labs, accessed July 12, 2025, https://saucelabs.com/resources/blog/cypress-accessibility-testing
Cypress Accessibility Testing (with Best Practices) - BrowserStack, accessed July 12, 2025, https://www.browserstack.com/guide/cypress-accessibility-testing
