
Comprehensive Test Feature Gating for UV-215


Executive Summary

This report provides a comprehensive strategy for resolving the challenges outlined in task UV-215 within the Uveddi project. The current testing framework's inability to gracefully handle the disabled tree-sitter feature has led to critical issues, including test panics and a lack of maintainability. The root cause is an absence of conditional compilation for tests and a failure to define the system's behavior when the feature is inactive.
The solution presented herein is a holistic approach grounded in idiomatic Rust practices. It leverages Rust's powerful conditional compilation attributes (#[cfg]) to gate not only production code but also the corresponding tests. Central to this strategy is the implementation of the "dummy shim" architectural pattern. This pattern ensures graceful degradation by providing a well-defined, non-panicking API surface when the tree-sitter feature is disabled, transforming potential runtime failures into predictable, testable behavior.
This report details the best practices for feature management in Cargo.toml, strategies for implementing conditional test execution, and a clear methodology for creating stub validation tests that verify the system's resilience. Furthermore, it establishes robust guidelines for documentation and informative test skipping to enhance clarity and long-term maintainability.
The expected outcome is the successful implementation of a resilient, flexible, and maintainable testing framework that fully supports the tree-sitter feature toggle. This will eliminate test panics, improve developer experience, and serve as a durable engineering blueprint for all future feature gating work within the Uveddi project, ensuring a higher standard of software quality and reliability.

1. Foundational Principles of Feature Gating in Rust

Effective feature gating is more than a conditional if statement; it is a compile-time strategy that shapes a crate's architecture, dependencies, and public API. A deep understanding of the underlying mechanics in Cargo and rustc is paramount for building flexible and maintainable software.

1.1. The Role of Features in a Modular Architecture

In Rust, features are a mechanism managed by Cargo to facilitate conditional compilation and handle optional dependencies.1 They allow developers to define a set of named flags in
Cargo.toml that can be used to include or exclude sections of code at compile time.1 This capability is fundamental to building modular and scalable systems.
Beyond simple code toggling, features are integral to modern development workflows like progressive delivery. They enable teams to ship new, potentially incomplete functionality behind a feature gate, ensuring the new code path remains inactive in production environments.3 This decouples development cycles from release schedules, allowing components to be merged and integrated without being blocked by dependencies or final implementation details.3 For the Uveddi project, this means the
tree-sitter feature can be developed, tested, and merged into the main branch without activating it for all builds, thereby increasing development velocity and reducing integration friction.

1.2. The Mechanics of Cargo.toml

The Cargo.toml manifest is the control center for feature management. Correct configuration here is the first step toward a robust feature-gated system.
Defining Features: Features are declared within the [features] table. A feature that simply acts as a boolean flag without enabling other dependencies is defined with an empty array, for example: tree-sitter =.1
Optional Dependencies: A key use case for features is managing optional dependencies. To associate a dependency with a feature, the dependency must first be marked as optional = true in the [dependencies] section. This prevents Cargo from compiling it by default. Then, in the [features] table, the feature is defined to activate the dependency using the dep: prefix.1 For the
tree-sitter feature, this would look like:
Ini, TOML
[dependencies]
tree-sitter-parser = { version = "0.20", optional = true }

[features]
tree-sitter = ["dep:tree-sitter-parser"]


Feature Hierarchies: Features can depend on other features. If feature-A requires the functionality provided by feature-B, it can be defined as feature-A =.1 This creates a directed graph of feature dependencies that Cargo resolves.
The default Feature: Cargo provides a special feature named default. When a package is built or added as a dependency, the features listed in the default set are enabled automatically.1 This behavior can be opted out of using the
--no-default-features command-line flag or by specifying default-features = false in a dependency declaration in another crate's Cargo.toml.1 This control is critical for testing, as it allows for the explicit validation of code paths with and without the default feature set enabled.

1.3. The Additive Feature Philosophy and Unification

A core design principle of Cargo's feature system is that features are additive.2 This means that during dependency resolution, Cargo constructs a single build plan for each crate. If any package in the dependency tree requires a feature to be enabled for a particular crate, that feature will be enabled for all other packages that also depend on that crate. This process is known as "feature unification".2
This has a critical implication: creating mutually exclusive features is a significant anti-pattern in Rust.1 For example, if a crate defines
backend-a and backend-b features that gate conflicting implementations, there is no mechanism to prevent a downstream user's dependency graph from enabling both simultaneously. This would likely result in compilation errors or unpredictable behavior.
For the UV-215 task, the tree-sitter feature must be designed as a purely additive enhancement. The code gated behind #[cfg(feature = "tree-sitter")] provides new functionality, while the code under #[cfg(not(feature = "tree-sitter"))] provides a compatible, non-conflicting baseline behavior.

1.4. Core Conditional Compilation Tools

Rust provides two primary mechanisms for conditional compilation, each with a distinct purpose and behavior.6
The #[cfg] Attribute: The #[cfg] attribute is the workhorse of feature gating. It instructs the compiler to conditionally include or exclude the annotated item—be it a module, function, struct, impl block, or even a single expression.7 If the configuration predicate within the attribute evaluates to false, the compiler removes the associated code from the Abstract Syntax Tree (AST)
before performing type-checking or borrow-checking.8 This is why
#[cfg] is essential for gating code that relies on optional dependencies; if the feature is disabled, the code that uses the missing types is completely removed, preventing compilation errors.
The cfg!() Macro: In contrast, the cfg!() macro evaluates a configuration predicate to a boolean literal (true or false) at compile time.6 It is used within function bodies, typically in an
if statement. A crucial difference is that cfg!() does not remove any code from the AST. Both the if and else blocks must contain syntactically and semantically valid code that can be successfully type-checked, even if one of the blocks is guaranteed to be dead code.6 Its use is therefore limited to situations where both code paths are always valid, regardless of which features are enabled.
Combinators: Both #[cfg] and cfg!() support the all(), any(), and not() operators to construct complex boolean predicates. This allows for fine-grained control, such as #[cfg(all(target_os = "linux", feature = "tree-sitter"))].7
The distinction between these two tools is fundamental to solving UV-215.

Feature
#[cfg(predicate)]
if cfg!(predicate) {... }
Evaluation Time
Compile-time (AST transformation)
Compile-time (evaluates to true or false)
Code Scope
Removes the annotated item (module, function, struct, expression) if predicate is false.
The code inside both if and else blocks must be syntactically and semantically valid, even if one branch is dead code.
Effect on Binary
Reduces binary size by excluding code and dependencies.
Does not alter binary size on its own; relies on the optimizer for dead code elimination.
Primary Use Case
Gating features, platform-specific code, optional dependencies.
Minor conditional logic within a function where both paths are always valid.
Source(s)
6
6


2. Implementation Strategy for the tree-sitter Feature

A successful implementation requires a dual approach: gating the feature-specific logic while simultaneously providing a well-defined alternative for when the feature is disabled. This ensures the application remains robust and predictable across all build configurations.

2.1. Gating Application Logic

The primary mechanism for gating the tree-sitter functionality is the #[cfg] attribute. All code that directly depends on the tree-sitter crate or its associated types must be annotated with #[cfg(feature = "tree-sitter")].1 This applies to:
Module declarations (e.g., #[cfg(feature = "tree-sitter")] mod ast_parser;).
Struct and enum definitions.
impl blocks.
Individual functions.
use statements that import types from the optional dependency.
To avoid the proliferation of #[cfg] attributes throughout the codebase—a pattern that harms readability and maintainability—it is best to apply the attribute at the highest logical level possible.8 For instance, if all functionality related to AST parsing is contained within a single module, applying
#[cfg(feature = "tree-sitter")] to the mod declaration is preferable to annotating every item within that module. This practice encapsulates the feature-specific logic cleanly, as demonstrated in projects like Wasmtime, which gates entire modules like runtime and component at their top level.8

2.2. The "Dummy Shim" Pattern for Graceful Degradation

The core problem described in UV-215—test panics when tree-sitter is disabled—is a symptom of a system that fails to degrade gracefully. When a component's dependency is unavailable, the system should continue to operate with reduced functionality rather than crashing.12 The solution is to architect the system to handle this state explicitly.
The "dummy shim" pattern achieves this by providing a parallel, non-functional implementation of the API surface under the #[cfg(not(feature = "tree-sitter"))] attribute.8 This ensures that for every public item available when the feature is
on, a corresponding item with the same name and a compatible type signature exists when the feature is off.
The behavior of this shim implementation should be consistent with the feature being unavailable:
A function that would normally perform parsing and return a Result<Ast, Error> should instead immediately return an appropriate error, such as Err(Error::FeatureUnavailable("tree-sitter")). This makes the disabled state an explicit and reportable condition rather than a cause for a panic.8
A function that performs a side effect but returns () should become a no-op with an empty body.8
A struct that holds state related to the feature can be defined as a zero-sized type (ZST), such as struct Parser;. This satisfies the type checker and allows other parts of the code to instantiate and hold the type, but it carries no data and has zero runtime overhead.8
A function that returns an Option<T> should return None.17
This pattern, also used effectively in the Wasmtime project with its shared_memory.rs and shared_memory_disabled.rs modules 8, is more than a testing convenience. It is the direct implementation of a resilient architecture. By defining the behavior for the disabled code path, the system is designed to be robust against configuration changes, fulfilling the requirement for graceful degradation. The tests for this path are therefore not merely "stub validation" but are critical
resilience tests.

2.3. Managing Optional Dependencies in Cargo.toml

To support the conditional compilation of the tree-sitter logic, the Cargo.toml file must be configured to treat its dependencies as optional. As outlined in Section 1.2, this involves two key steps:
Declare Optional Dependencies: In the [dependencies] section, the tree-sitter crate and any associated parser crates must be marked with optional = true.1
Ini, TOML
[dependencies]
tree-sitter = { version = "0.20", optional = true }


Link Feature to Dependency: In the [features] table, the tree-sitter feature must be defined to enable its corresponding dependency via the dep: syntax.1
Ini, TOML
[features]
tree-sitter = ["dep:tree-sitter"]


This configuration ensures that the tree-sitter dependency is only downloaded, compiled, and linked when a user or a build process explicitly enables the tree-sitter feature (e.g., via cargo build --features tree-sitter). For builds where the feature is disabled, the dependency is completely ignored, leading to faster compile times, smaller binary sizes, and a reduced dependency footprint.2

3. A Robust Testing Framework for Gated Features

A comprehensive testing strategy for a feature-gated crate must validate all possible code paths: the full-functionality path when the feature is enabled, and the graceful degradation path when it is disabled. This requires careful structuring of tests and the CI pipeline.

3.1. Conditional Execution of Feature-Dependent Tests

All tests that exercise the live tree-sitter functionality must themselves be conditionally compiled. This is achieved by applying the #[cfg(feature = "tree-sitter")] attribute to the test functions or their enclosing modules.4

Rust


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "tree-sitter")]
    fn test_parsing_with_tree_sitter_enabled() {
        // Test logic that depends on the tree-sitter crate...
    }
}


A crucial and often misunderstood aspect of Rust's testing system is the difference in how #[cfg(test)] is applied. The test configuration flag is enabled by rustc only for the crate being tested, which means it is active for unit tests (test modules inside the src/ directory). However, it is not enabled for the crate's code when compiling integration tests (files inside the tests/ directory).18 The compiler treats integration tests as a separate, external crate that depends on the main library crate. This design choice ensures that integration tests only validate the public API, just as a real-world consumer would.
This has a significant implication for test helper functions. If a helper function or module is defined within src/ and gated with #[cfg(test)], it will be completely invisible to integration tests in the tests/ directory. The idiomatic solution is to create a dedicated, non-default feature for test utilities, such as test-utils. This feature can then be enabled during test runs via cargo test --features test-utils, making the helpers available to both unit and integration tests without polluting the production API.20

3.2. Stub Validation: Testing for Graceful Degradation

Equally important are the tests that validate the "dummy shim" implementation. These tests ensure that the system behaves correctly and degrades gracefully when the tree-sitter feature is disabled. These tests must be gated with the opposite condition: #[cfg(not(feature = "tree-sitter"))].
A typical stub validation test would verify that calling a feature-gated function results in the expected "feature unavailable" error or no-op behavior.

Rust


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(feature = "tree-sitter"))]
    fn test_parser_returns_feature_unavailable_when_disabled() {
        let parser = Parser::new(); // The dummy ZST struct
        let result = parser.parse("fn main() {}");
        assert!(matches!(result, Err(Error::FeatureUnavailable(msg)) if msg == "tree-sitter"));
    }
}


These tests are essential for preventing regressions. Without them, a future code change could inadvertently re-introduce panics or other incorrect behavior into the disabled code path. The following table provides a clear guide for implementing the stubbing logic required for these tests.

Function Signature
Disabled-Feature Behavior
Example
Source(s)
Returns Result<T, E>
Return Err(FeatureUnavailableError)
Err(Error::FeatureUnavailable("tree-sitter"))
8
Returns Option<T>
Return None
None
17
Returns an Iterator
Return an empty iterator
std::iter::empty()
(Logical extension)
Returns () (side effect)
No-op (empty function body)
fn do_thing() {}
8
Is a struct
Define as a zero-sized type (ZST)
struct Parser;
8


3.3. Comprehensive CI for Feature Combinations

Because features are additive, a crate with N independent features can theoretically be built in 2N different configurations.2 While testing every single combination is often impractical, a robust CI pipeline must validate the most important ones to prevent breakages caused by feature interactions.
For the Uveddi project and the tree-sitter feature, the following CI jobs are recommended:
Default Build and Test: cargo test --workspace
This validates the default configuration of the crate. It tests the code path that most users will encounter.
No-Default-Features Build and Test: cargo test --workspace --no-default-features
This is critical for UV-215. It explicitly disables the default feature set and runs the tests for the graceful degradation path, including the stub validation tests.4
All-Features Build and Test: cargo test --workspace --all-features
This enables every feature defined in the Cargo.toml. It is the single most important job for ensuring that features are truly additive and do not conflict with one another.1
Specific-Feature Build and Test: cargo test --workspace --features "tree-sitter"
This job isolates and tests the tree-sitter feature specifically, ensuring it works correctly even if it is not part of the default set.
This multi-faceted CI strategy provides high confidence that all major code paths are correct and that feature interactions will not lead to unexpected build failures.

4. Enhancing Maintainability and Developer Experience

Beyond correct implementation, a successful feature-gating strategy must prioritize long-term maintainability and a clear developer experience. This is achieved through informative test output and comprehensive, feature-aware documentation.

4.1. Graceful Test Skipping and Informative Messaging

It is important to distinguish between two ways of "skipping" tests in Rust:
Compile-Time Exclusion (#[cfg]): As discussed, this removes tests from the binary entirely. The test runner is not even aware they exist. This is the correct approach for tests that are fundamentally incompatible with a given build configuration (e.g., a test for a disabled feature).
Runtime Ignoring (#[ignore]): The #[ignore] attribute marks a test function to be compiled but not executed by default.23 These tests can be run explicitly with
cargo test -- --include-ignored. The #[ignore] attribute can also take a string argument to provide a reason for skipping, which the test runner will display.23
Rust
#[test]
#
fn test_expensive_operation() {
    //...
}

When run, the output will include: test_expensive_operation... ignored: This test is very slow...
While #[ignore] is less relevant for the specific problem of the tree-sitter feature flag, it is a vital tool for a mature testing suite. For example, if a test required a specific, large tree-sitter grammar file to be downloaded, it could be marked #[ignore] to avoid running it in every CI cycle. The community is also exploring more dynamic, imperative ways to skip tests from within the test body itself, as seen in the pre-RFC for skippable tests, which proposes a test::skip(reason) function.26 For now, a test can dynamically check a condition and, if it cannot run, simply return early while printing an informative message to standard output (which can be viewed with
cargo test -- --nocapture).27

4.2. Documentation for Clarity and Longevity

One of the most significant challenges with feature gating is ensuring the public documentation accurately reflects the conditional nature of the API. By default, rustdoc generates documentation for a single configuration (usually the host platform with default features), which can render the documentation for feature-gated items incomplete or misleading.28
There are two primary solutions to this problem, with a clear recommendation for projects that value long-term maintainability.
The Stable Solution (#[cfg(doc)]): rustdoc sets a special doc configuration flag during its build process. By combining this with the feature flag, #[cfg(any(feature = "tree-sitter", doc))], developers can ensure an item is always compiled during the documentation build, making it visible in the output.28 While this solves the problem of
visibility, it fails to solve the problem of clarity. A user viewing the documentation will see the function but will have no indication that it is only available when a specific feature is enabled.
The Superior (Unstable) Solution (#[doc(cfg)]): The Rust compiler provides an unstable attribute, #[doc(cfg(...))], which is the definitive solution to this problem.30 This attribute explicitly tells
rustdoc about the configuration required for an item. rustdoc then renders a prominent banner in the documentation, such as "This is only available when the tree-sitter feature is enabled." This provides unambiguous information to the user at the point of use. To use this, a nightly toolchain is required for documentation generation, and the feature must be enabled with #![feature(doc_cfg)] at the crate root.30
Given the Uveddi project's focus on reliability and maintainability, the minor overhead of using a nightly toolchain exclusively for documentation generation is a worthwhile trade-off for the immense clarity #[doc(cfg)] provides. The existence of this attribute, and the related doc_auto_cfg proposal which aims to infer these banners automatically 30, demonstrates a clear direction from the Rust project: feature-gated APIs should be explicitly documented as such.
The following table summarizes the recommended configuration for producing high-quality, feature-aware documentation on platforms like docs.rs.

Configuration Item
Value / Example
Purpose
Source(s)
Cargo.toml Metadata
[package.metadata.docs.rs] all-features = true rustdoc-args = ["--cfg", "docsrs"]
Tells docs.rs to build with all features enabled and to set the docsrs configuration flag.
32
Stable doc Attribute
#[cfg(any(feature = "tree-sitter", doc))]
Ensures an item is always compiled when rustdoc runs, making it visible in the output.
28
Unstable doc(cfg) Attribute
#[doc(cfg(feature = "tree-sitter"))]
Annotates the item in the documentation with a banner explaining its feature requirement. Requires nightly.
30
Combined Attributes
#[cfg(any(feature = "tree-sitter", doc))] #[doc(cfg(feature = "tree-sitter"))]
The complete solution: ensures visibility and provides a clear feature-gate banner in the generated docs.
30
Doctests
cargo test --doc --features "tree-sitter"
Doctests for feature-gated code must be run with the required features enabled manually.
34


5. Integration Roadmap and Governance for UV-215

A structured implementation plan and clear guidelines for future feature management will ensure the successful resolution of UV-215 and establish a sustainable pattern for the Uveddi project.

5.1. A Phased Refactoring Plan

The following phased approach is recommended to minimize disruption and ensure a methodical implementation.
Phase 1: Cargo.toml Configuration.
Modify Cargo.toml to declare the tree-sitter dependency and any related parser dependencies as optional = true.
Define the tree-sitter feature in the [features] table, ensuring it enables the necessary dependencies with the dep: prefix.
Phase 2: Implement the "Dummy Shim".
For each function, struct, and type alias that is part of the public or internal API for tree-sitter, create the corresponding stub implementation under the #[cfg(not(feature = "tree-sitter"))] attribute.
This phase should be completed first to establish a non-panicking baseline that compiles successfully with --no-default-features.
Phase 3: Gate Production Code.
Apply the #[cfg(feature = "tree-sitter")] attribute to the real implementations of the tree-sitter logic.
At the end of this phase, the entire crate should compile cleanly both with and without the tree-sitter feature enabled.
Phase 4: Refactor and Augment Tests.
Apply #[cfg(feature = "tree-sitter")] to all existing tests that depend on the tree-sitter functionality.
Create a new suite of "stub validation" tests under #[cfg(all(test, not(feature = "tree-sitter")))] to verify the graceful degradation behavior implemented in Phase 2.
Review test modules for any common setup or helper logic that can be abstracted, being mindful of the cfg(test) limitations for integration tests.
Phase 5: Implement Documentation.
Add #![feature(doc_cfg)] to the crate root.
Apply the #[doc(cfg(feature = "tree-sitter"))] attribute to all relevant public items.
Configure the [package.metadata.docs.rs] table in Cargo.toml to ensure docs.rs builds the documentation correctly.

5.2. Long-Term Feature Lifecycle Management

To prevent "feature creep" 2 and maintain a clean codebase, a clear policy for the entire lifecycle of a feature is necessary.
Feature Hygiene: A regular process should be established to review existing feature flags. Once a feature has been fully rolled out and is considered a permanent part of the application, the feature flag and the corresponding #[cfg] attributes should be removed.3 This simplifies the code by removing the now-unnecessary conditional compilation paths.
Deprecation and Removal: When a feature is to be removed, it should follow a clear deprecation cycle. First, the feature should be removed from the default set in Cargo.toml. In a subsequent major version release, the feature flag can be removed entirely, making the functionality either mandatory or fully removed. Removing a feature or placing existing public code behind a new feature flag is a breaking change and must be communicated accordingly through semantic versioning.1
Governance: While compile-time features do not require the complex governance of runtime feature flag services, changes to features in Cargo.toml should be subject to rigorous code review. These changes directly impact the crate's public API, dependency graph, and build configurations, and should be treated with the same level of scrutiny as changes to production code.

6. Conclusion and Recommendations

The challenges presented by UV-215 are symptoms of an incomplete feature implementation that lacks resilience and clear documentation. By adopting the comprehensive strategy detailed in this report, the Uveddi project can resolve these immediate issues and establish a robust, maintainable framework for future development.
The key recommendations are:
Embrace Conditional Compilation Fully: Utilize #[cfg(feature =...)] and #[cfg(not(feature =...))] to create two distinct, complete, and valid code paths for the tree-sitter functionality.
Architect for Graceful Degradation: Implement the "dummy shim" pattern to provide a predictable, non-panicking behavior when the tree-sitter feature is disabled. This transforms a potential failure mode into a testable, well-defined state.
Test All Configurations: Configure the CI pipeline to test the default, no-default, and all-features configurations. This is essential for verifying feature additivity and ensuring that both the enabled and disabled code paths remain correct over time.
Prioritize Documentation Clarity: Adopt the unstable #[doc(cfg)] attribute to provide unambiguous, compiler-verified documentation for all feature-gated APIs. The long-term benefit of clear documentation for developers and users far outweighs the minor inconvenience of using a nightly toolchain for rustdoc generation.
By implementing these recommendations, the Uveddi project will not only solve the specific problems of UV-215 but will also elevate its engineering practices, leading to more reliable, flexible, and maintainable software.
Works cited
Features - The Cargo Book - Rust Documentation, accessed July 20, 2025, https://doc.rust-lang.org/cargo/reference/features.html
Item 26: Be wary of feature creep - Effective Rust, accessed July 20, 2025, https://effective-rust.com/features.html
Best practices for Feature Gates | Statsig Docs, accessed July 20, 2025, https://docs.statsig.com/feature-flags/best-practices/
Enabling/disabling default crate features for integration tests - help, accessed July 20, 2025, https://users.rust-lang.org/t/enabling-disabling-default-crate-features-for-integration-tests/12402
Guidance on optional functionality: crates vs features - Rust Users Forum, accessed July 20, 2025, https://users.rust-lang.org/t/guidance-on-optional-functionality-crates-vs-features/85694
cfg - Rust By Example - Rust Documentation, accessed July 20, 2025, https://doc.rust-lang.org/rust-by-example/attribute/cfg.html
#[cfg] Conditional Compilation in Rust - Mastering Backend, accessed July 20, 2025, https://masteringbackend.com/posts/cfg-conditional-compilation-in-rust
Conditional Compilation - Wasmtime, accessed July 20, 2025, https://docs.wasmtime.dev/contributing-conditional-compilation.html
Conditional compilation - The Rust Reference, accessed July 20, 2025, https://dev-doc.rust-lang.org/beta/reference/conditional-compilation.html
Conditional compilation - The Rust Reference, accessed July 20, 2025, https://doc.rust-lang.org/reference/conditional-compilation.html
What's your stance towards conditional compilation? : r/rust - Reddit, accessed July 20, 2025, https://www.reddit.com/r/rust/comments/xvkcns/whats_your_stance_towards_conditional_compilation/
production-readiness-checklist/docs/concepts/graceful-degradation.md at master - GitHub, accessed July 20, 2025, https://github.com/mercari/production-readiness-checklist/blob/master/docs/concepts/graceful-degradation.md
Graceful Degradation: Preventing Complete System Failures, accessed July 20, 2025, https://www.thecoder.cafe/p/graceful-degradation
How does one write function stubs for testing Rust modules? - Stack Overflow, accessed July 20, 2025, https://stackoverflow.com/questions/66603516/how-does-one-write-function-stubs-for-testing-rust-modules
Graceful error handling - Rustfinity, accessed July 20, 2025, https://www.rustfinity.com/practice/rust/challenges/graceful-error-handling/description
Error Handling Best Practices in Rust: A Comprehensive Guide to Building Resilient Applications | by Syed Murtza | Medium, accessed July 20, 2025, https://medium.com/@Murtza/error-handling-best-practices-in-rust-a-comprehensive-guide-to-building-resilient-applications-46bdf6fa6d9d
Error Handling - The Rust Programming Language - MIT, accessed July 20, 2025, https://web.mit.edu/rust-lang_v1.25/arch/amd64_ubuntu1404/share/doc/rust/html/book/first-edition/error-handling.html
#[cfg(test)] doesn't take affect when running integration tests in the /tests dir : r/rust - Reddit, accessed July 20, 2025, https://www.reddit.com/r/rust/comments/ny6k3f/cfgtest_doesnt_take_affect_when_running/
`#[cfg(test)]` not working inside function bodies? - Rust Users Forum, accessed July 20, 2025, https://users.rust-lang.org/t/cfg-test-not-working-inside-function-bodies/80174
Can I make an object public for integration tests and/or benchmarks only? - Stack Overflow, accessed July 20, 2025, https://stackoverflow.com/questions/47698194/can-i-make-an-object-public-for-integration-tests-and-or-benchmarks-only
Implicitly enable feature only for tests - help - The Rust Programming Language Forum, accessed July 20, 2025, https://users.rust-lang.org/t/implicitly-enable-feature-only-for-tests/100109
Is it possible to enable a rust feature only in test? - Stack Overflow, accessed July 20, 2025, https://stackoverflow.com/questions/68618789/is-it-possible-to-enable-a-rust-feature-only-in-test
Testing - The Rust Reference, accessed July 20, 2025, https://doc.rust-lang.org/reference/attributes/testing.html
How to skip expensive tests with cargo test? - rust - Reddit, accessed July 20, 2025, https://www.reddit.com/r/rust/comments/3i1nki/how_to_skip_expensive_tests_with_cargo_test/
Provide ignore message when the test ignored · Issue #10250 · rust-lang/cargo - GitHub, accessed July 20, 2025, https://github.com/rust-lang/cargo/issues/10250
Pre-rfc: Skippable tests - libs - Rust Internals, accessed July 20, 2025, https://internals.rust-lang.org/t/pre-rfc-skippable-tests/14611
rust - Skipping unit tests or at least showing warnings from them - Stack Overflow, accessed July 20, 2025, https://stackoverflow.com/questions/29241148/skipping-unit-tests-or-at-least-showing-warnings-from-them
Advanced features - The rustdoc book, accessed July 20, 2025, https://doc.rust-lang.org/rustdoc/advanced-features.html
Advanced features - The rustdoc book, accessed July 20, 2025, https://rustwiki.org/en/rustdoc/advanced-features.html
Unstable features - The rustdoc book, accessed July 20, 2025, https://doc.rust-lang.org/beta/rustdoc/unstable-features.html
Is it a misuse or a bug with feature `doc_cfg`? - Rust Users Forum, accessed July 20, 2025, https://users.rust-lang.org/t/is-it-a-misuse-or-a-bug-with-feature-doc-cfg/112509
Use `docsrs` attribute to indicate feature-gated items in documentation · Issue #986 · rust-random/rand - GitHub, accessed July 20, 2025, https://github.com/rust-random/rand/issues/986
Getting Features to Show Up in Your Rust Docs - VADOSWARE, accessed July 20, 2025, https://vadosware.io/post/getting-features-to-show-up-in-your-rust-docs/
How do you run doc tests in feature gated implementations? - Stack Overflow, accessed July 20, 2025, https://stackoverflow.com/questions/42742369/how-do-you-run-doc-tests-in-feature-gated-implementations
cfg(test) is not set during doctests · Issue #45599 · rust-lang/rust - GitHub, accessed July 20, 2025, https://github.com/rust-lang/rust/issues/45599
