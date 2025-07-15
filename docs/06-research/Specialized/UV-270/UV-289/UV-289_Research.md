
Refactoring the AnalysisEngine: A Component-Based Architecture and Migration Strategy

Jira Issue: UV-289
Document Version: 1.0
Author: Senior Rust Architect

Introduction


Problem Statement

The AnalysisEngine struct, at over 780 lines of code, has evolved into a quintessential "god object" within the Uveddi static analysis tool. This monolithic component currently manages a disparate set of responsibilities, including Abstract Syntax Tree (AST) parsing, dependency graph construction, detector lifecycle management, WebAssembly (WASM) plugin integration, configuration, and result aggregation. This design directly violates the Single Responsibility Principle (SRP), a foundational tenet of robust software engineering.1 The consequences of this architectural anti-pattern are significant: the codebase suffers from low cohesion and high internal complexity, making it exceedingly difficult to test, maintain, and extend.2 Any modification, no matter how minor, carries a high risk of introducing regressions in seemingly unrelated functional areas. This state impedes developer velocity and poses a long-term threat to the stability and scalability of the Uveddi platform.

Strategic Objectives

The primary objective of this refactoring initiative is to dismantle the AnalysisEngine monolith and reconstitute its functionality into a collection of focused, loosely coupled components. The target architecture will be composed of "deep modules"—components that offer rich functionality through a minimal, well-designed API, thereby hiding significant internal complexity.4 This decomposition will yield several strategic benefits:
Enhanced Modularity: Each new component will have a single, well-defined responsibility, making the system easier to reason about.
Improved Testability: Isolated components can be unit-tested with mocked dependencies, drastically improving test coverage and reliability.
Increased Maintainability: Changes to one area of functionality will be localized to a single component, reducing the risk of unintended side effects.
Alignment with Idiomatic Rust: The new design will leverage Rust's powerful type system, trait-based abstractions, and ownership model to create a more robust and performant architecture that is both safe and concurrent.5

Architectural Vision

The proposed solution is a component-based architecture that draws inspiration from the principles of Hexagonal (Ports and Adapters) Architecture.6 This approach isolates the core analysis logic from external concerns like configuration, parsing, and plugins. The existing
AnalysisEngine will be preserved as a public-facing facade to ensure complete backward compatibility, a critical project constraint.
The migration from the current monolith to the new componentized architecture will be executed using the Strangler Fig pattern.8 This is a proven, risk-averse methodology for incrementally replacing pieces of a legacy system. New components will be developed and integrated one by one, gradually "strangling" the old implementation until the original
AnalysisEngine becomes a pure delegating facade. This phased approach ensures the system remains fully functional throughout the migration process, minimizing disruption and allowing for continuous delivery.

1. A Decoupled Component Architecture for Static Analysis

This section defines the target architecture, identifying the primary components to be extracted from the AnalysisEngine and specifying their structure and interfaces using idiomatic Rust.

1.1. Component Identification and Domain Boundaries

A thorough analysis of the existing AnalysisEngine reveals a conflation of multiple distinct operational domains. Adhering to the principle of Separation of Concerns, the first step is to partition the monolith's responsibilities into logical, cohesive units.9 This partitioning is based on grouping methods and data that exhibit high conceptual coherency and frequent co-usage.2
This approach is validated by established architectural patterns observed in other sophisticated static analysis tools and IDE backends, such as rust-analyzer.10 These tools often employ a pipeline or map-reduce-style architecture involving an initial indexing/parsing phase followed by a full analysis phase that leverages the indexed data. The proposed component breakdown for Uveddi mirrors this proven model, establishing a clear analysis pipeline:
Configure -> Parse (AST) -> Analyze Dependencies -> Schedule Detectors (Native + Plugin) -> Aggregate Results.
The following six components have been identified to form the new core architecture:
AstProvider: Solely responsible for parsing source files into Abstract Syntax Trees (ASTs) via tree-sitter. It will manage an in-memory cache of parsed ASTs to prevent redundant work during a single analysis run, a critical performance optimization.
DependencyGraphBuilder: Consumes ASTs from the AstProvider to construct and maintain the project's dependency graph. This isolates the complex logic of dependency resolution from the core analysis scheduling.
DetectorScheduler: The central analysis orchestrator. It traverses the project structure, queries other components for necessary data (like ASTs and configuration), and schedules the execution of all applicable static analysis detectors against the relevant code artifacts.
PluginManager: Manages the entire lifecycle of WASM-based plugins. This includes loading plugin binaries, instantiating them within a WASM runtime, and facilitating communication between the DetectorScheduler and the sandboxed plugin code. It acts as a crucial adapter to an external system.
ConfigurationService: A centralized service for accessing all tool-wide configurations. It provides a single source of truth for settings such as enabled detectors, file exclusion patterns, and plugin-specific parameters.
AnalysisAggregator: Gathers all findings, errors, and other results generated by the DetectorScheduler and PluginManager. It is responsible for collating this data into a structured format suitable for the final reporting stage.

1.2. Component Architecture Design (Deliverable A)

The new architecture will be instantiated through a dependency injection system based on the builder pattern, which provides compile-time safety and avoids the runtime overhead of traditional DI frameworks.11 The
AnalysisEngine will be retained as a facade, delegating calls to its internal components, thus ensuring backward compatibility.2

The AnalysisEngine Facade and Builder

The public API will remain unchanged. The AnalysisEngine struct will be composed of the new components, which are injected during construction via the AnalysisEngineBuilder.

Rust


use std::sync::Arc;
use crate::config::ToolConfiguration;
use crate::errors::EngineBuildError;

// The main facade, maintaining the public API. Its fields represent the new components.
pub struct AnalysisEngine {
    config_service: Arc<ConfigurationService>,
    ast_provider: Arc<dyn AstProvider>,
    dependency_builder: Arc<dyn DependencyGraphBuilder>,
    plugin_manager: PluginManagerHandle, // A handle to the actor-based PluginManager
    detector_scheduler: Arc<DetectorScheduler>,
    analysis_aggregator: Arc<AnalysisAggregator>,
}

// The builder is responsible for constructing and wiring all components.
pub struct AnalysisEngineBuilder {
    config: Option<ToolConfiguration>,
    // Other potential pre-supplied dependencies can be added here.
}

impl AnalysisEngineBuilder {
    pub fn new() -> Self {
        AnalysisEngineBuilder { config: None }
    }

    pub fn with_config(mut self, config: ToolConfiguration) -> Self {
        self.config = Some(config);
        self
    }

    /// Asynchronously builds the fully configured AnalysisEngine.
    /// This is the primary integration point for the dependency injection system.
    pub async fn build(self) -> Result<AnalysisEngine, EngineBuildError> {
        let config = self.config.ok_or(EngineBuildError::MissingConfiguration)?;

        // 1. Instantiate ConfigurationService
        let config_service = Arc::new(ConfigurationService::new(config));

        // 2. Instantiate data-centric components
        let ast_provider = Arc::new(AstProviderImpl::new());
        let dependency_builder = Arc::new(DependencyGraphBuilderImpl::new(Arc::clone(&ast_provider)));

        // 3. Spawn the PluginManager actor and get its handle
        let plugin_manager = PluginManager::new(Arc::clone(&config_service));
        let plugin_manager_handle = plugin_manager.spawn(); // Spawns a tokio task

        // 4. Instantiate remaining components
        let analysis_aggregator = Arc::new(AnalysisAggregator::new());
        let detector_scheduler = Arc::new(DetectorScheduler::new(
            Arc::clone(&config_service),
            Arc::clone(&ast_provider),
            plugin_manager_handle.clone(),
            Arc::clone(&analysis_aggregator),
        ));

        Ok(AnalysisEngine {
            config_service,
            ast_provider,
            dependency_builder,
            plugin_manager: plugin_manager_handle,
            detector_scheduler,
            analysis_aggregator,
        })
    }
}



Component Interface Traits

Each component's public contract will be defined by a trait. This abstraction is fundamental to achieving loose coupling and enabling effective mocking for unit tests.12 For components with asynchronous methods, the
async-trait crate is required.

Rust


use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use crate::ast::DependencyGraph;
use crate::errors::{AstError, DependencyError};

/// Provides access to cached Abstract Syntax Trees, parsing on-demand.
#[async_trait]
pub trait AstProvider: Send + Sync {
    /// Retrieves the AST for a given file path.
    /// If the AST is not in the cache, it parses the file and caches the result.
    async fn get_ast(&self, file_path: &Path) -> Result<Arc<tree_sitter::Tree>, AstError>;
}

/// Constructs the project's dependency graph from source files.
#[async_trait]
pub trait DependencyGraphBuilder: Send + Sync {
    /// Builds or updates the full dependency graph for the project, starting from a root path.
    async fn build_graph(&self, root_path: &Path) -> Result<DependencyGraph, DependencyError>;
}

// Note: Traits for DetectorScheduler, ConfigurationService, PluginManager (handle),
// and AnalysisAggregator would be defined similarly, each exposing a minimal,
// focused set of methods aligned with their responsibilities.



1.3. Component Responsibility Matrix (Deliverable B)

To enforce strict adherence to the SRP and prevent future architectural decay, the following matrix explicitly defines the responsibilities and boundaries of each component.
Component
Responsible For
NOT Responsible For
Dependencies (Components)
Interacts With (How)
ConfigurationService
- Providing access to all tool settings.
- Validating configuration structure.
- Parsing files.
- Executing detectors.
- Storing analysis state.
None
Provides configuration data to all other components upon request.
AstProvider
- Parsing source code into tree-sitter ASTs.
- Caching parsed ASTs in memory.
- Managing configuration.
- Building the dependency graph.
- Knowing which detectors to run.
None
Provides Arc<tree_sitter::Tree> to DetectorScheduler and DependencyGraphBuilder.
DependencyGraphBuilder
- Consuming ASTs to identify module dependencies.
- Constructing a graph representation of the project.
- Parsing source files.
- Scheduling analysis.
- Storing results.
AstProvider
Calls get_ast on AstProvider to retrieve trees for analysis.
PluginManager
- Loading/unloading WASM plugins.
- Instantiating WASM runtimes.
- Executing detectors within WASM modules.
- Translating data between the Rust host and WASM guest.
- Deciding which plugins to run on which files.
- Parsing source code.
- Aggregating final results.
ConfigurationService
Receives execution commands from DetectorScheduler; reads plugin configuration from ConfigurationService.
DetectorScheduler
- Traversing the codebase.
- Identifying which detectors (native & plugin) apply to which files.
- Invoking native detector logic.
- Sending execution requests to the PluginManager.
- Parsing code.
- Managing WASM runtimes.
- Aggregating the final report.
ConfigurationService, AstProvider, PluginManagerHandle, AnalysisAggregator
Reads config; calls get_ast; sends commands to PluginManager; calls record_finding on AnalysisAggregator.
AnalysisAggregator
- Collecting findings and errors from all sources.
- Storing intermediate results in a structured way.
- Preparing the final, collated result set for reporting.
- Executing any analysis.
- Formatting the final report output (e.g., JSON, SARIF).
- Managing configuration.
None
Receives findings from DetectorScheduler and PluginManager. Provides the final result set to the top-level AnalysisEngine facade.


2. Interfaces, Communication, and Error Propagation

This section details the dynamic aspects of the architecture: how components interact, manage state, and propagate errors within the tokio asynchronous runtime.

2.1. Asynchronous Communication and State Management

A key architectural decision in a concurrent Rust application is choosing the right primitive for communication and state sharing. The two primary patterns are shared-state concurrency via Arc<Mutex<T>> and message passing via channels.14 A naive, one-size-fits-all approach is suboptimal. A more nuanced, hybrid model is appropriate here: use message passing for I/O-bound components to avoid blocking, and use shared-state mutexes for efficient access to in-memory data structures.16 This strategy avoids common pitfalls like holding a lock across a long
.await call, which can lead to deadlocks or poor scheduler performance.17

Pattern 1: Shared State for Data-Centric Components (Arc<T>)

Components like ConfigurationService, AstProvider, and DependencyGraphBuilder are primarily data providers. They manage state that needs to be read by multiple other components concurrently. For these, a shared-state model is most efficient.
Implementation: These components will be wrapped in an Arc to allow shared ownership. Where internal mutability is needed (e.g., for the AST cache), they will use a tokio::sync::Mutex or tokio::sync::RwLock. The tokio synchronization primitives are essential, as they are designed to be safely held across .await points without blocking the thread, unlike their std::sync counterparts.17
Example (AstProviderImpl Cache):
Rust
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

// The concrete implementation of the AstProvider trait.
pub struct AstProviderImpl {
    // The cache is protected by a Tokio Mutex to allow safe concurrent access.
    cache: Arc<Mutex<HashMap<PathBuf, Arc<tree_sitter::Tree>>>>,
}

impl AstProviderImpl {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

//... in the `get_ast` implementation...
// let mut cache_guard = self.cache.lock().await;
//... logic to check cache, parse if needed, and insert...



Pattern 2: Actor Model for I/O-Bound Components (mpsc Channels)

The PluginManager is fundamentally I/O-bound. It interacts with the filesystem to load WASM binaries and communicates with a separate WASM runtime, both of which are potentially slow operations. Encapsulating this component as an actor that communicates via asynchronous messages is the ideal pattern for decoupling it from the rest of the system.18
Implementation: The PluginManager will be spawned as a dedicated tokio task that runs a message-processing loop. Other components, primarily the DetectorScheduler, will not call its methods directly. Instead, they will hold a PluginManagerHandle containing a tokio::sync::mpsc::Sender. They will send command messages to the actor. For operations that require a response, the command message will include a tokio::sync::oneshot::Sender channel, which the actor uses to send the result back to the caller.16
Example (Actor Communication Primitives):
Rust
use tokio::sync::{mpsc, oneshot};
use crate::reporting::Finding;
use std::sync::Arc;

/// Defines the commands that can be sent to the PluginManager actor.
pub enum PluginCommand {
    Execute {
        plugin_id: String,
        source_file_path: PathBuf,
        ast: Arc<tree_sitter::Tree>,
        // A one-shot channel for the actor to send the result back.
        responder: oneshot::Sender<Result<Vec<Finding>, anyhow::Error>>,
    },
    // Other commands like LoadPlugin, UnloadPlugin, etc.
}

/// A lightweight, cloneable handle for interacting with the PluginManager actor.
#[derive(Clone)]
pub struct PluginManagerHandle {
    sender: mpsc::Sender<PluginCommand>,
}

impl PluginManagerHandle {
    pub async fn execute_plugin(
        &self,
        plugin_id: String,
        source_file_path: PathBuf,
        ast: Arc<tree_sitter::Tree>,
    ) -> Result<Vec<Finding>, anyhow::Error> {
        let (responder, receiver) = oneshot::channel();
        let command = PluginCommand::Execute {
            plugin_id,
            source_file_path,
            ast,
            responder,
        };
        self.sender.send(command).await?;
        receiver.await?
    }
}



2.2. Interface Design and Dependency Injection Integration

The component traits will be designed to be minimal, adhering to the principle of a small API surface area hiding greater complexity.4 This reduces the cognitive load on developers using the component and makes the interfaces more stable over time.
The AnalysisEngineBuilder::build method serves as the single, explicit point of integration with the dependency injection system (UV-156). This manual, builder-based approach is idiomatic in Rust and is preferred over reflection-based frameworks for its compile-time safety, performance, and clarity.12
While extensive use of dynamic dispatch via Box<dyn Trait> can introduce performance overhead and ergonomic friction 19, a balanced approach is best. The
build method will construct concrete types (e.g., AstProviderImpl), benefiting from static dispatch during the setup phase. The final AnalysisEngine struct will then store these components as Arc<dyn Trait>. This provides the ideal combination of efficient construction with the runtime flexibility needed for mocking in tests and for potential future extensions where multiple implementations of a component might coexist.

2.3. Error Handling Strategy (Deliverable E)

A robust and clear error handling strategy is critical for a complex system. The canonical approach in the Rust ecosystem is a hybrid model: use thiserror to create detailed, structured error types within a library or component, and use anyhow at the application or integration layer to easily propagate errors while adding valuable context.20 Our components will be treated as internal libraries within the larger Uveddi application.

Component-Specific Errors with thiserror

Each component will define its own Error enum using the thiserror crate. This allows for precise error identification and enables consumers of the component (including tests) to match on specific failure modes.
Example (AstProvider Error Definition):
Rust
use thiserror::Error;
use std::path::PathBuf;

#
pub enum AstError {
    #[error("Failed to read source file at {path}: {source}")]
    FileReadError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #
    ParsingError { path: PathBuf },

    #[error("Unsupported language for file: {path}")]
    UnsupportedLanguage { path: PathBuf },
}



Error Propagation and Context with anyhow

When errors are propagated across component boundaries, they will be converted into anyhow::Error. The calling function will use the .context() method from the anyhow::Context trait to wrap the original error, adding a descriptive message about the high-level operation that failed. This preserves the full error chain while making debugging significantly easier.
Example (Error Propagation from DetectorScheduler):
Rust
use anyhow::Context;
use std::path::Path;

// A method within the DetectorScheduler implementation.
async fn schedule_on_file(&self, path: &Path) -> anyhow::Result<()> {
    // The specific AstError is wrapped with contextual information.
    let ast = self.ast_provider.get_ast(path).await
       .context(format!("Failed to get AST for scheduling on file '{}'", path.display()))?;

    //... further logic that might also return anyhow::Result...

    Ok(())
}



3. An Incremental Migration Roadmap (Deliverable C)

This section outlines a phased, low-risk migration plan based on the Strangler Fig pattern. This approach avoids a "big bang" rewrite, ensuring the AnalysisEngine remains functional and deployable at every stage of the refactoring process.

3.1. The Strangler Fig Pattern as a Guiding Methodology

The Strangler Fig pattern is an application modernization strategy where new functionality is incrementally built around a legacy system until the old system is "strangled" and can be decommissioned.8 In our context, the
AnalysisEngine is the legacy system, and the new components are the "vines" that will gradually take over its responsibilities.
The implementation is straightforward in Rust due to the project's backward compatibility constraint. The public methods of AnalysisEngine serve as a stable facade. The migration proceeds by:
Identifying a cohesive block of logic within an AnalysisEngine method.
Extracting this logic into a method on a new component.
Modifying the original AnalysisEngine method to simply delegate the call to the new component.2
This iterative process continues until all logic has been moved out of AnalysisEngine, leaving it as a pure facade that orchestrates the underlying components. The success of this pattern relies on having well-defined boundaries for the new components (as defined in Section 1.1) and a robust testing strategy to prevent regressions.21

3.2. Phased Migration Plan


Phase 0: Setup and Scaffolding

Effort: Low (1-2 days)
Tasks:
Define the struct and trait for all six new components (AstProvider, PluginManager, etc.).
Implement the AnalysisEngineBuilder and modify the AnalysisEngine struct to hold Option<Arc<...>> fields for each new component, all initialized to None.
Create a comprehensive backward compatibility test suite. This suite will make calls to the public AnalysisEngine API using a variety of real-world scenarios and assert the correctness of the final output. It treats the engine as a black box.
Risks: Minimal. No production logic is changed.
Testing Strategy: The new compatibility test suite must be written and must pass against the current, unmodified AnalysisEngine implementation. This suite becomes the gold standard for all subsequent phases.

Phase 1: Extract ConfigurationService

Effort: Low (2-3 days)
Tasks:
Move all logic related to reading, parsing, and providing configuration from AnalysisEngine into a new ConfigurationServiceImpl.
Update the AnalysisEngineBuilder to instantiate this component and inject it into the AnalysisEngine.
Refactor all internal calls within AnalysisEngine that accessed configuration directly to now call methods on the config_service field.
Dependencies: Phase 0.
Risks: Low. Configuration is a largely self-contained and cross-cutting concern, making it an ideal first candidate for extraction.
Testing Strategy: Write unit tests for ConfigurationServiceImpl in isolation. Run the full backward compatibility test suite to ensure no behavior has changed.

Phase 2: Extract AstProvider

Effort: Medium (3-5 days)
Tasks:
Extract all tree-sitter parsing logic and the associated caching mechanism into AstProviderImpl.
Instantiate and inject the AstProvider via the builder.
Modify all methods in AnalysisEngine that perform parsing to now delegate these calls to ast_provider.get_ast(...).
Dependencies: Phase 1.
Risks: Medium. The performance of the AST cache is critical. The interaction with tree-sitter's C libraries must be handled correctly.
Testing Strategy: Unit test AstProviderImpl using a mock filesystem. Create integration tests that verify caching behavior (e.g., assert that the underlying parse function is called only once for the same file path). Run performance benchmarks and the full compatibility suite.

Phase 3: Extract PluginManager (as an Actor)

Effort: High (5-8 days)
Tasks:
Extract all WASM plugin management logic (loading, instantiation, execution).
Implement the PluginManager actor, its PluginCommand enum, and the PluginManagerHandle.
Update the AnalysisEngineBuilder to tokio::spawn the actor task and store the handle.
Replace direct calls to WASM runtimes within AnalysisEngine with asynchronous commands sent via the PluginManagerHandle.
Dependencies: Phase 1.
Risks: High. This is the most complex extraction due to the introduction of the actor model. Asynchronous communication, error handling between tasks, and state management within the actor require careful implementation.
Testing Strategy: Unit test the PluginManager actor by creating a channel, sending it commands, and asserting its state changes or the responses it sends back. The WASM runtime itself should be mocked at this stage. Run the full compatibility suite.

Phase 4 and Beyond: Subsequent Extractions

Effort: Variable.
Tasks: Continue the Strangler Fig pattern for the remaining components in the following recommended order:
AnalysisAggregator: A relatively simple state-holding component.
DependencyGraphBuilder: A complex but well-isolated component.
DetectorScheduler: The final and most complex piece, as it orchestrates most of the other components. Its extraction will represent the culmination of the refactoring effort.
Testing Strategy: For each component, follow the pattern: write unit tests with mocks, write targeted integration tests, and validate with the full compatibility and performance suites.

Final Phase: Cleanup

Effort: Low (1 day)
Tasks: Once all logic has been delegated, the AnalysisEngine methods will be simple one-line calls to the corresponding components. Review the AnalysisEngine module and remove any now-unused private methods, helper functions, and dead code. The refactoring is officially complete.

4. Verification, Validation, and Performance

A multi-layered quality assurance strategy is essential to guarantee that the refactored system is correct, reliable, and performs as well as or better than the original.

4.1. Comprehensive Testing Strategy (Deliverable D)

Relying on a single type of test is insufficient for a change of this magnitude. The strategy must encompass unit, integration, performance, and contract testing to ensure quality at every level.23

Component Testing (Unit Tests)

Tooling: Standard Rust tests (#[test]) will be used in combination with the mockall crate for creating mock objects.25
mockall is a powerful library for mocking trait dependencies, which is why defining component interfaces as traits is critical.
Strategy: Each component implementation (e.g., DetectorSchedulerImpl) will be tested in complete isolation. All of its dependencies will be replaced with mocks generated by mockall. This allows for testing the component's internal logic without interference from other parts of the system.
Asynchronous Mocking: A key consideration when using mockall with async-trait is the order of attributes. The #[automock] (or #[cfg_attr(test, automock)]) attribute must be placed before the #[async_trait] attribute for the macros to compose correctly.25
Example (DetectorScheduler Unit Test):
Rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::MockAstProvider; // mockall generates this
    use mockall::predicate;
    use std::path::Path;

    fn create_mock_ast() -> tree_sitter::Tree {
        // Helper to create a dummy AST for testing
        //...
    }

    #[tokio::test]
    async fn scheduler_requests_ast_for_file() {
        let mut mock_ast_provider = MockAstProvider::new();

        // Set an expectation: we expect `get_ast` to be called exactly once
        // with the argument "src/main.rs". When it is, we return a mock AST.
        mock_ast_provider.expect_get_ast()
           .with(predicate::eq(Path::new("src/main.rs")))
           .times(1)
           .returning(|_| Ok(Arc::new(create_mock_ast())));

        // Instantiate the scheduler with the mock provider and other mock dependencies.
        let scheduler = DetectorScheduler::new(
            Arc::new(mock_ast_provider),
            /*... other mocks... */
        );

        // Execute the scheduler's logic.
        let result = scheduler.analyze_file(Path::new("src/main.rs")).await;

        // The test will panic if the expectation set on the mock is not met.
        assert!(result.is_ok());
    }
}



Integration Testing

Strategy: These tests verify the interaction between two or more real components. For example, an integration test would instantiate a real DetectorScheduler and a real AstProviderImpl to confirm that the scheduler can correctly request and receive an AST, and that the provider's cache is populated as expected. These tests should use real-world resources where feasible, such as a temporary directory containing test source files, rather than mocking the filesystem.23

Performance Regression Testing

Strategy: A suite of performance benchmarks will be created using criterion.rs. These benchmarks will measure the end-to-end analysis time of the AnalysisEngine on several representative codebases of varying size and complexity.
Execution: The benchmark suite will be run against the main branch before the project begins to establish a baseline. It will then be run after the completion of each migration phase. Any significant performance degradation (e.g., >5% increase in execution time) will be treated as a blocking issue that must be resolved before proceeding.

Backward Compatibility Validation

Strategy: The contract test suite developed in Phase 0 is the cornerstone of this validation. It treats the AnalysisEngine as a black box and ensures its public API produces identical results for a given set of inputs throughout the refactoring process. This test suite must pass with 100% success after every single commit to the migration branch.

4.2. Performance Impact Analysis


Component Communication Overhead

Arc<T> with tokio::sync::Mutex: The overhead for accessing shared data via this pattern is minimal. It involves an atomic reference count operation and a lock acquisition. tokio::sync::Mutex is highly optimized for the common case of low contention and will not block the OS thread, making it very efficient.17
mpsc Channels: Communication via channels is inherently more expensive than a direct function call. It typically involves at least one heap allocation for the message being sent and synchronization within the channel's internal queue. However, for the I/O-bound PluginManager, this overhead is a necessary and acceptable trade-off for the architectural benefits of decoupling, resilience, and avoiding blocked scheduler threads.

Memory Allocation Patterns

The new architecture will slightly increase memory usage due to the additional Arc allocations for sharing components. This is a small, fixed cost.
The primary memory consumer will remain the AstProvider's cache. Centralizing this cache is a significant architectural improvement. If memory consumption on extremely large projects becomes a concern, a cache eviction policy (e.g., Least Recently Used - LRU) can be implemented within the AstProvider in the future without affecting other components.

Async Operation Coordination Costs

The tokio runtime's work-stealing scheduler is designed for high-throughput I/O-bound workloads and is extremely efficient.28 The cost of spawning a few long-lived tasks for components like the
PluginManager is negligible. The primary performance cost in an async system comes from the I/O operations themselves, which the async/await model is designed to handle with maximum efficiency.

Caching Strategies

The centralized AST cache within the AstProvider is expected to be a major source of performance improvement. The current monolithic design may lead to redundant parsing of the same file in different phases of analysis. The new architecture guarantees that any given source file is parsed exactly once per analysis run, with all subsequent requests for its AST being served instantly from the in-memory cache.

Conclusion

The proposed refactoring strategy presents a comprehensive and actionable plan to dismantle the AnalysisEngine god object and replace it with a robust, maintainable, and performant component-based architecture. By decomposing the monolith into six focused components with clear responsibilities, the new design will significantly enhance testability and developer velocity while adhering to the principles of idiomatic Rust.
The adoption of a hybrid communication model—using shared-state Arc<Mutex<T>> for data-centric components and an actor model with mpsc channels for I/O-bound components—is a nuanced approach that balances performance with architectural resilience in a tokio-based asynchronous environment. The corresponding error handling strategy, which combines the strengths of thiserror for component-specific errors and anyhow for contextual propagation, will lead to a more debuggable and reliable system.
Crucially, the migration will be executed using the Strangler Fig pattern, an incremental and risk-averse methodology. This ensures that the Uveddi tool remains stable and fully functional at every stage of the process, eliminating the need for a high-risk, all-or-nothing rewrite. The multi-layered testing strategy, combining unit, integration, performance, and backward compatibility tests, provides the necessary quality assurance to execute this migration with confidence.
Executing this plan will pay down significant technical debt and position the Uveddi static analysis tool for future growth and innovation. The resulting codebase will be more modular, easier to reason about, and better aligned with the best practices of modern systems programming in Rust.
Works cited
How to implement inheritance-like feature for Rust? - Rust Users Forum, accessed July 15, 2025, https://users.rust-lang.org/t/how-to-implement-inheritance-like-feature-for-rust/31159
How do you refactor a God class? - Stack Overflow, accessed July 15, 2025, https://stackoverflow.com/questions/14870377/how-do-you-refactor-a-god-class
Large Class - Refactoring.Guru, accessed July 15, 2025, https://refactoring.guru/smells/large-class
Fundamentals: Component-Based Design - High Assurance Rust ..., accessed July 15, 2025, https://highassurance.rs/chp16_appendix/components.html
It's hard to overestimate how extremely Rust is optimized towards making refacto... | Hacker News, accessed July 15, 2025, https://news.ycombinator.com/item?id=35349835
Hexagonal architecture in Rust. Tutorial index | by Luca Corsetti ..., accessed July 15, 2025, https://medium.com/@lucorset/hexagonal-architecture-in-rust-72f8958eb26d
antoinecarton/hexagonal-rust: Hexagonal architecture in Rust - GitHub, accessed July 15, 2025, https://github.com/antoinecarton/hexagonal-rust
Strangler Fig Pattern - Azure Architecture Center | Microsoft Learn, accessed July 15, 2025, https://learn.microsoft.com/en-us/azure/architecture/patterns/strangler-fig
Refactoring to Improve Modularity and Error Handling - The Rust Programming Language, accessed July 15, 2025, https://doc.rust-lang.org/book/ch12-03-improving-error-handling-and-modularity.html
Three Architectures for a Responsive IDE - rust-analyzer, accessed July 15, 2025, https://rust-analyzer.github.io/blog/2020/07/20/three-architectures-for-responsive-ide.html
Rust Service Builder: Zero-Cost Dependency Management | by Anıl Küçükrecep | Medium, accessed July 15, 2025, https://medium.com/@anilkrcp/zero-cost-dependency-management-with-rust-eb1427ab8611
An easy way of implementing the Dependency Injection Pattern in Rust, accessed July 15, 2025, https://www.hackingwithrust.net/2023/10/08/an-easy-way-of-implementing-the-dependency-injection-pattern-in-rust/
Rust traits and dependency injection - Julio Merino (jmmv.dev), accessed July 15, 2025, https://jmmv.dev/2022/04/rust-traits-and-dependency-injection.html
Commonly used design patterns in async rust? - community, accessed July 15, 2025, https://users.rust-lang.org/t/commonly-used-design-patterns-in-async-rust/108802
Mastering Concurrency in Rust: Advanced Patterns with Async/Await and Tokio, accessed July 15, 2025, https://omid.dev/2024/06/15/mastering-concurrency-in-rust/
Sharing a resource through `Arc
Mutex in tokio::sync - Rust - Docs.rs, accessed July 15, 2025, https://docs.rs/tokio/latest/tokio/sync/struct.Mutex.html
A Pattern For Component Based Program Architecture In Rust, accessed July 15, 2025, https://vadosware.io/post/a-pattern-for-component-based-program-architecture-in-rust/
Testability: Reimagining OOP design patterns in Rust - Audun Halland, accessed July 15, 2025, https://audunhalland.github.io/blog/testability-reimagining-oop-design-patterns-in-rust/
Rust Error Handling Explained: thiserror vs anyhow (Best Practices) - YouTube, accessed July 15, 2025, https://www.youtube.com/watch?v=-c9JEexiHPE
Strangler Fig Pattern: Modernizing It Without Losing It - Swimm, accessed July 15, 2025, https://swimm.io/learn/legacy-code/strangler-fig-pattern-modernizing-it-without-losing-it
How the Strangler Fig Pattern Enables Safe and Gradual Refactoring - GoCodeo, accessed July 15, 2025, https://www.gocodeo.com/post/how-the-strangler-fig-pattern-enables-safe-and-gradual-refactoring
Master Hexagonal Architecture in Rust (parts 1 & 2) - Reddit, accessed July 15, 2025, https://www.reddit.com/r/rust/comments/1dmqqo5/master_hexagonal_architecture_in_rust_parts_1_2/
Mocking - Comprehensive Rust - Google, accessed July 15, 2025, https://google.github.io/comprehensive-rust/android/testing/mocking.html
mockall - Rust - Docs.rs, accessed July 15, 2025, https://docs.rs/mockall
Why and How to Mock Unit Tests in Rust || The MockAll Crate - YouTube, accessed July 15, 2025, https://www.youtube.com/watch?v=q4TAsQ9d5dg
Mocking Dependencies with Traits for Unit Testing in Rust | by Marcel Nunez | Medium, accessed July 15, 2025, https://medium.com/@mpnunez28/mocking-dependencies-with-traits-for-unit-testing-in-rust-ef987fafd27e
Tutorial | Tokio - An asynchronous Rust runtime, accessed July 15, 2025, https://tokio.rs/tokio/tutorial
Practical Guide to Async Rust and Tokio | by Oleg Kubrakov - Medium, accessed July 15, 2025, https://medium.com/@OlegKubrakov/practical-guide-to-async-rust-and-tokio-99e818c11965
