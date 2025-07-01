
Understanding and Migrating WASI Context Management in Wasmtime


1. Executive Summary

This report addresses the compilation errors stemming from the disappearance of WasiPreview1Ctx and WasiPreview1CtxBuilder in the wasmtime-wasi crate. These types have been superseded by the ongoing evolution of the WebAssembly System Interface (WASI) and Wasmtime's robust adoption of the WebAssembly Component Model. The core resolution involves understanding the distinct API paths for WASI Preview 1 and WASI Preview 2. For modern WebAssembly components, the correct types are now wasmtime_wasi::p2::WasiCtx and wasmtime_wasi::p2::WasiCtxBuilder. For continued support of legacy WASI Preview 1 modules, wasmtime_wasi::preview1::WasiP1Ctx is available. This report provides detailed guidance, practical code examples, and essential architectural considerations to facilitate a smooth migration and effective WASI context management within wasmtime applications.

2. Understanding WASI Evolution in Wasmtime

This section lays the groundwork by explaining the fundamental shifts in the WASI specification and how Wasmtime has adapted its API to support these changes. The errors encountered by developers are not mere renames but symptoms of a deeper architectural evolution.

2.1. The Landscape of WASI: Preview 1 vs. Preview 2

The WebAssembly System Interface (WASI) defines a standardized set of APIs that enable WebAssembly modules to interact securely with host environments, providing capabilities such as filesystem access, command-line arguments, and environment variables. The WASI specification has undergone significant development, leading to distinct versions with differing approaches to system interaction. The most prominent transition relevant to the observed issue is from WASI Preview 1 (often referred to as wasi_snapshot_preview1) to WASI Preview 2 (also known as WASI 0.2).1
This evolution signifies a profound architectural shift, moving beyond a simple API update. In early 2024, the Bytecode Alliance released WASI Preview 2, a major iteration that integrates the WebAssembly Component Model and expands available APIs.1 This means that the new context types, such as
wasmtime_wasi::p2::WasiCtx and its builder, are not merely direct replacements for older types. Instead, they are fundamentally designed to operate within this new Component Model paradigm, which introduces concepts like "worlds" – cohesive sets of interfaces for specific domains.1 The errors encountered by developers, therefore, point to a fundamental incompatibility with an outdated approach, guiding users towards the new standard. A straightforward find-and-replace operation will likely not resolve the issue entirely; instead, a re-evaluation of the application's WASI integration strategy is necessary to align with the evolving conceptual model of WASI interaction.

2.2. Wasmtime's Adoption of the Component Model and WASI Preview 2

Wasmtime, developed by the Bytecode Alliance, is a high-performance WebAssembly runtime that actively drives and adopts WebAssembly standards. Its swift integration of WASI Preview 2 support underscores its commitment to the evolving WebAssembly ecosystem.1 This adoption has led to a significant restructuring of its WASI-related APIs to align with the Component Model's principles of modularity and explicit interface definitions. For instance, Wasmtime's implementation of WASI Preview 2 has been relocated to the
wasmtime_wasi::p2 module from the root of the crate in recent versions, specifically noted in the v34.0.0 release.3 This relocation directly addresses where the new WASI Preview 2 types reside.
The design of Wasmtime itself is segmented to support both traditional WebAssembly modules and the newer WebAssembly Component Model. The wasmtime crate is broadly divided into a core API for modules and a dedicated wasmtime::component module for components.4 This architectural split explains why WASI context management now follows different paths depending on whether one is working with a module or a component. While Wasmtime is clearly advancing towards WASI Preview 2 and the Component Model, it also maintains explicit support for WASI Preview 1. The presence of
wasmtime_wasi::preview1::WasiP1Ctx and the build_p1() method on WasiCtxBuilder 5 demonstrates this strategic decision. This dual-path approach allows Wasmtime to innovate and implement the latest standards (Preview 2) while simultaneously providing a clear, separate API for users who need to maintain compatibility with existing WASI Preview 1 modules. This prevents a hard-breaking change for the entire ecosystem, enabling a more gradual and controlled migration.

2.3. Impact on WASI Context Management

The evolution of WASI has profoundly impacted how host environments like Wasmtime manage the context provided to WebAssembly instances. The new design emphasizes explicit configuration, clear ownership of resources, and a more structured interaction model. This includes the central role of wasmtime::Store<T> as the primary container for host state and the introduction of traits like WasiView.2
The WasiCtxBuilder is now used to configure and construct a WasiCtx, which then holds the state for a WASI instance, encompassing elements like file descriptors, preopened directories, environment variables, and arguments.2 For WASI Preview 2, all interactions are implemented through a
WasiView trait, which provides access to WasiCtx and, by implication, to a ResourceTable. This ResourceTable is responsible for owning and managing all host-defined component model resources.8 This design pattern centralizes all host-provided context and resources within a single, unified
Store object. This means that any interaction between the WebAssembly module and the host, including WASI calls, will implicitly or explicitly pass through this Store<T> and its contained T data. This approach enhances clarity, type safety, and extensibility, as all host-specific data is managed coherently within the Store's lifetime.

3. Identifying the Current WASI Context Types

This section directly addresses the core problem by identifying the correct types to use in the current wasmtime-wasi version, differentiating between WASI Preview 2 (Component Model) and WASI Preview 1 (Legacy Modules).

3.1. The Deprecation of WasiPreview1Ctx and WasiPreview1CtxBuilder

The observation that WasiPreview1Ctx and WasiPreview1CtxBuilder no longer exist in the current wasmtime-wasi version is accurate, as stated in the user's query. These types, if they were ever explicitly named as such and directly exposed at the root of wasmtime-wasi, have been removed or relocated as part of the crate's evolution and the broader WASI specification updates. The wasmtime project has moved towards a more structured and versioned API surface for WASI. For example, the release notes for Wasmtime v34.0.0 indicate a significant internal restructuring, explicitly stating that Wasmtime's WASI Preview 2 implementation has moved to wasmtime_wasi::p2.3 This kind of internal reorganization naturally leads to older, less organized types being phased out or moved. Furthermore, the
wasi-common crate, which was Wasmtime's legacy implementation of WASI 0.1 (Preview 1), is now superseded, with maintainers advising users to upgrade to the implementation provided by the wasmtime-wasi crate.9
This deprecation reflects a deliberate design choice to enforce a more controlled and predictable lifecycle for WASI contexts. Discussions within the Wasmtime project reveal that the WasiCtx for Preview 2 does not expose direct methods for manipulating its contents after creation.10 The
WasiCtxBuilder is intended for exclusive manipulation during context construction, and WasiCtx members are made private once built.10 This indicates that older
WasiPreview1Ctx implementations might have allowed more direct, mutable access to their internal state post-creation. The current design prioritizes a "build-then-use" pattern, where the WASI context is configured entirely via the builder and then remains largely immutable. This shift improves predictability, reduces the potential for runtime errors, and aligns with more modern Rust API design principles, enhancing overall API robustness.

3.2. Introducing wasmtime_wasi::p2::WasiCtx and wasmtime_wasi::p2::WasiCtxBuilder (for Components)

For applications built to leverage the WebAssembly Component Model and the latest WASI Preview 2 specification, the canonical types for managing the WASI environment are WasiCtx and WasiCtxBuilder. These types are located within the wasmtime_wasi::p2 module.7 They are specifically designed to support the richer capabilities and structured interfaces of WASI 0.2, including the concept of "worlds" and explicit resource management.
The WasiCtxBuilder is a builder-style structure used to create a WasiCtx.7 It offers a fluent API with various methods to configure the WASI environment, such as inheriting standard I/O streams (
inherit_stdio()), inheriting command-line arguments (inherit_args()), and setting environment variables (env()) or custom arguments (arg()).11 Once configured, the
.build() method finalizes the process, producing a WasiCtx instance.7 The
WasiCtx then holds the state for a WASI instance, including file descriptors and other WASI-specific data.2 The
wasmtime_wasi::p2 module serves as Wasmtime's primary implementation for WASI Preview 2.8 These types are the standard for new development with WASI Preview 2 and the Component Model, offering a robust and well-defined API for host interaction within this evolving ecosystem.

3.3. Introducing wasmtime_wasi::preview1::WasiP1Ctx (for Legacy Modules)

Recognizing that many existing WebAssembly modules still target the older WASI Preview 1 specification, wasmtime-wasi provides dedicated support for these modules. The specific context type for WASI Preview 1 is WasiP1Ctx, which is found within the wasmtime_wasi::preview1 module.5 While the
WasiCtxBuilder from wasmtime_wasi::p2 is still utilized for its fluent configuration API, it must be finalized with the build_p1() method to produce a WasiP1Ctx instance.5 This approach maintains a consistent builder pattern while yielding the correct context type compatible with the older WASI snapshot. This separate API path ensures backward compatibility for existing WASI Preview 1 modules, allowing for a phased migration strategy.
The following table summarizes the mapping of WASI context types:
Table 1: WASI Context Type Mapping

Category
Type (Deprecated/Moved)
Current Type (WASI Preview 2 / Component Model)
Current Type (WASI Preview 1 / Legacy Modules)
Primary Use Case
Key Characteristics/Notes
Context
WasiPreview1Ctx
wasmtime_wasi::p2::WasiCtx
wasmtime_wasi::preview1::WasiP1Ctx
Legacy Module
Implicitly handled WASIp1. This type is no longer directly exposed or used in current wasmtime-wasi versions.
Builder
WasiPreview1CtxBuilder
wasmtime_wasi::p2::WasiCtxBuilder
wasmtime_wasi::p2::WasiCtxBuilder
WebAssembly Component
Designed for the Component Model, requires implementation of the WasiView trait for host state access.






(used with .build_p1())
Legacy Module
Explicit WASIp1 support, less extensible and configurable than WASI Preview 2.

This table provides a concise answer to what the new types are and where they are located, based on the WASI version. It serves as a quick reference for developers needing to identify the correct replacement types at a glance, directly addressing the core problem with a clear mapping. It also reinforces the dual-path strategy (Preview 1 vs. Preview 2) which is a critical distinction in the current Wasmtime ecosystem.

4. Practical Migration Guide: Initializing and Managing WASI Context

This section provides concrete code examples and explanations for how to correctly initialize and manage WASI contexts for both WASI Preview 2 components and WASI Preview 1 legacy modules.

4.1. Migrating to WASI Preview 2 (Component Model)

Migrating to WASI Preview 2 involves adopting the new wasmtime_wasi::p2 types and understanding their integration with wasmtime::Store<T> and the WasiView trait.

Code Example: Basic WasiCtx Initialization

To initialize a WasiCtx for a WebAssembly component, developers utilize the WasiCtxBuilder::new() method, followed by chaining various configuration methods, and finally calling .build(). This builder pattern allows for clear and concise setup of the WASI environment.7 For instance,
inherit_stdio() configures the context to inherit standard input, output, and error streams from the host process, while inherit_args() passes command-line arguments. Additional methods like env() and arg() allow for setting custom environment variables and program arguments, respectively.11

Rust


use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store, Result};
use wasmtime_wasi::p2::{WasiCtx, WasiCtxBuilder, WasiView, IoView}; // Note: IoView is also needed for ResourceTable

// Define a struct to hold your host state, including WasiCtx and ResourceTable
pub struct MyHostState {
    pub wasi_ctx: WasiCtx,
    pub resource_table: ResourceTable,
    // Add any other custom host states here
}

// Implement WasiView and IoView for your host state struct
impl IoView for MyHostState {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.resource_table
    }
}

impl WasiView for MyHostState {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi_ctx
    }
}

#[tokio::main] // If using async WASI operations
async fn main() -> Result<()> {
    let mut config = Config::new();
    config.async_support(true); // Enable async support if needed

    let engine = Engine::new(&config)?;

    // 1. Initialize WasiCtx using WasiCtxBuilder
    let wasi_ctx = WasiCtxBuilder::new()
       .inherit_stdio() // Inherit standard I/O (stdin, stdout, stderr) from the host process
       .inherit_args()  // Inherit command-line arguments passed to the host process
       .env("MY_ENV_VAR", "some_value") // Set custom environment variables
       .arg("program_arg_1") // Add custom command-line arguments for the WASM module
       .build(); // Finalize the builder to create the WasiCtx

    // 2. Create your host state struct
    let state = MyHostState {
        wasi_ctx,
        resource_table: ResourceTable::new(), // Initialize a new ResourceTable
    };

    // 3. Create a Store with your host state
    let mut store = Store::new(&engine, state);

    //... further steps to compile component and instantiate...

    Ok(())
}



Integrating WasiCtx with Store<T> and WasiView

As demonstrated in the example above, WasiCtx (and ResourceTable for Preview 2) are typically embedded within a custom struct that serves as the generic type T for wasmtime::Store<T>.7 This
T struct must then implement the WasiView trait (and IoView, which WasiView implies) to provide the wasmtime runtime and WASI functions access to the context and resource table.8 This pattern ensures that all host-specific state is managed coherently within the
Store's lifetime.
A crucial aspect of this design is the intended immutability of WasiCtx after its creation. The WasiCtx for Preview 2 does not expose public methods for modifying its internal configuration (e.g., changing preopened directories or environment variables) once it has been built.10 This means that once
WasiCtx is built and placed into Store<T>, direct modification of its internal configuration fields through the WasiView trait is not the intended pattern. The WasiView trait is designed for the runtime to access the context's state, not to reconfigure it dynamically. For significant changes to the WASI environment, the recommended approach is to create a new WasiCtx via WasiCtxBuilder and potentially a new Store instance. This promotes a more functional and predictable state management model, enhancing the overall robustness of the API.

Linking WASI Interfaces: add_to_linker_sync and add_to_linker_async

After setting up the Engine, Store, and host state, the next crucial step is to add the necessary WASI functions to a wasmtime::component::Linker. The wasmtime-wasi::p2 module provides helper functions, add_to_linker_sync and add_to_linker_async, which automatically populate the linker with all standard WASI Preview 2 interfaces.6 The choice between
_sync and _async depends on whether the Engine is configured for asynchronous support.8

Rust


//... (previous setup for MyHostState, Engine, Store)

// 4. Create a Linker
let mut linker = Linker::<MyHostState>::new(&engine);

// 5. Add WASI interfaces to the linker
// For synchronous WASI operations (default):
wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;

// For asynchronous WASI operations (if config.async_support(true) is set):
// wasmtime_wasi::p2::add_to_linker_async(&mut linker)?;

//... further steps to compile component and instantiate with the linker...


The following table provides a quick, comprehensive overview of the most common and important methods available on the WasiCtxBuilder. It helps developers understand the various ways they can customize the WASI environment without having to dig through extensive documentation.
Table 2: Essential WasiCtxBuilder Methods
Method Name
Description
Relevant Snippet IDs
new()
Creates a new WasiCtxBuilder instance with default parameters (e.g., stdin closed, no env vars).
7
inherit_stdio()
Configures stdin, stdout, and stderr to be inherited from the host process.
7
inherit_stdin()
Configures the context's stdin stream to read the host process's stdin.
11
inherit_stdout()
Configures the context's stdout stream to write to the host process's stdout.
11
inherit_stderr()
Configures the context's stderr stream to write to the host process's stderr.
11
inherit_args()
Configures the context to inherit command-line arguments from the host process.
7
arg(name)
Adds a custom command-line argument for the WebAssembly module.
11
env(key, value)
Adds a custom environment variable for the WebAssembly module.
11
build()
Finalizes the builder process and produces a WasiCtx instance (for WASI Preview 2).
7
build_p1()
Finalizes the builder process and produces a WasiP1Ctx instance (for WASI Preview 1).
5


4.2. Continuing with WASI Preview 1 (Legacy Modules)

For applications that are not yet ready to migrate to the Component Model and still rely on traditional WebAssembly modules compiled against WASI Preview 1, wasmtime-wasi provides a dedicated API path within the wasmtime_wasi::preview1 module.

Code Example: WasiP1Ctx Initialization

While still using the WasiCtxBuilder from wasmtime_wasi::p2 for its fluent API, the key difference for Preview 1 modules is the finalization method: build_p1(). This method constructs a WasiP1Ctx instance, which is compatible with the older WASI snapshot.5

Rust


use wasmtime::{Result, Engine, Linker, Module, Store};
use wasmtime_wasi::preview1::{self, WasiP1Ctx};
use wasmtime_wasi::p2::WasiCtxBuilder; // WasiCtxBuilder is still from p2, but builds p1 context

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let engine = Engine::default();
    let module = Module::from_file(&engine, "path/to/your/legacy_wasi_module.wasm")?;

    // Initialize WasiP1Ctx using WasiCtxBuilder and build_p1()
    let wasi_ctx = WasiCtxBuilder::new()
       .inherit_stdio()
       .inherit_env() // Inherit environment variables
       .args(&args) // Pass command-line arguments
       .build_p1(); // Crucial: use build_p1() for Preview 1 context

    let mut store = Store::new(&engine, wasi_ctx);
    //... (further steps to instantiate and run the module)...
    Ok(())
}



Linking WASI Preview 1 Interfaces

Similar to Preview 2, WASI interfaces for Preview 1 modules are added to the wasmtime::Linker. However, it is essential to use the add_to_linker_sync (or add_to_linker_async) function specifically from the wasmtime_wasi::preview1 module.5 Note that the
Linker's generic type T will be WasiP1Ctx in this case.

Rust


use wasmtime::{Engine, Linker, Module, Store};
use wasmtime_wasi::preview1::{self, WasiP1Ctx};
use wasmtime_wasi::p2::WasiCtxBuilder; // Builder for P1 context

fn main() -> Result<()> {
    let engine = Engine::default();
    let module = Module::from_file(&engine, "path/to/your/legacy_wasi_module.wasm")?;

    let wasi_ctx = WasiCtxBuilder::new().build_p1();
    let mut store = Store::new(&engine, wasi_ctx);

    let mut linker: Linker<WasiP1Ctx> = Linker::new(&engine);

    // Add WASI Preview 1 interfaces to the linker
    // For synchronous WASI P1 operations:
    preview1::add_to_linker_sync(&mut linker, |t| t)?;

    // For asynchronous WASI P1 operations (if engine configured for async):
    // preview1::add_to_linker_async(&mut linker, |t| t)?;

    let instance = linker.instantiate(&mut store, &module)?;
    let func = instance.get_typed_func::<(), ()>(&mut store, "_start")?;
    func.call(&mut store, ())?;

    Ok(())
}



5. Key Architectural Changes and Best Practices

Beyond the specific type names, understanding the underlying architectural shifts and adopting best practices is crucial for robust wasmtime application development.

5.1. The Role of ResourceTable in WASI Preview 2

With the advent of WASI Preview 2 and the Component Model, the concept of resource management has become more explicit and granular. The ResourceTable is a new, fundamental component within the host state (Store<T>) that is responsible for owning and managing host-defined resources.7 These resources, such as open files, network sockets, or other system handles, are exposed to WebAssembly components through a secure, table-based indexing mechanism.2 This separation from the
WasiCtx itself allows for more robust lifecycle management and fine-grained control over permissions for individual resources.
The explicit introduction of ResourceTable as a distinct entity alongside WasiCtx in WASI Preview 2 signifies a significant enhancement in how host resources are managed. In WASI Preview 1, resource handling might have been more implicitly embedded within the WasiCtx or less formally structured. By separating the ResourceTable, Wasmtime enables a clearer model of ownership and lifecycle for host-defined resources. This aligns perfectly with the Component Model's security principles, where components interact with well-defined interfaces and capabilities. It allows for more precise control over what resources a WebAssembly component can access and how, contributing to a more secure and robust sandbox. This modularity also simplifies the implementation and auditing of host-side resource management.

5.2. WasiCtx Immutability and Builder-Only Configuration

A crucial design principle in WASI Preview 2's WasiCtx is its intended immutability after creation. Unlike potentially older approaches, the WasiCtx itself does not expose public methods for modifying its internal configuration (e.g., changing preopened directories or environment variables) after it has been built. All initial configuration must be performed exclusively through the WasiCtxBuilder.10 This "build-then-use" philosophy for
WasiCtx promotes predictable behavior and reduces the potential for unexpected side effects or inconsistent state during runtime. For dynamic changes to the WASI environment, developers should consider recreating the WasiCtx via the WasiCtxBuilder and, if necessary, re-instantiating the Store or managing mutable state within the Store<T>'s T that is accessed via WasiView, but not directly modifying the WasiCtx itself.

5.3. Choosing Between WASI Preview 1 and Preview 2

The decision to use WASI Preview 1 or Preview 2 depends primarily on the WebAssembly module intended for execution. WASI Preview 2 (Component Model) is the recommended path for new development and for WebAssembly components that leverage the Component Model. It offers a more modular, secure, and extensible approach to host interaction, aligning with the future direction of WebAssembly.1 Conversely, WASI Preview 1 (Legacy Modules) is necessary for existing "core" WebAssembly modules that were compiled against the older
wasi_snapshot_preview1 specification and do not conform to the Component Model. While still supported, new features and extensive development will primarily target Preview 2.2 Developers should align their WASI context type and linking approach with the WASI version and Component Model compatibility of their WebAssembly module. New projects should prioritize WASI Preview 2 for its advanced features and future-proofing.

5.4. Considerations for Asynchronous WASI Operations

Wasmtime provides robust support for asynchronous WASI operations, a critical feature for building performant and non-blocking WebAssembly applications, especially in server-side or highly concurrent environments.1 Leveraging asynchronous WASI allows the host runtime to perform I/O operations without blocking the execution thread, thereby improving overall responsiveness and scalability. This functionality is enabled by configuring the
Engine for async support and utilizing functions like add_to_linker_async from the wasmtime-wasi::p2 module.7 For applications requiring high performance, concurrent I/O, or seamless integration with Rust's
async/await ecosystem, configuring the Engine for async support and utilizing add_to_linker_async is a recommended best practice.

6. Conclusion

The error message concerning WasiPreview1Ctx and WasiPreview1CtxBuilder is a clear indicator of the wasmtime ecosystem's active progression towards the WebAssembly Component Model and WASI Preview 2. These types have been superseded by a more structured and versioned API that reflects fundamental architectural shifts.
For new development targeting WebAssembly components, the correct and recommended approach is to utilize wasmtime_wasi::p2::WasiCtx and wasmtime_wasi::p2::WasiCtxBuilder. These types are designed to integrate seamlessly with the Component Model's "worlds" and explicit resource management via ResourceTable, necessitating the implementation of the WasiView trait for host state access.
For applications that must maintain compatibility with existing WebAssembly modules compiled against the older WASI Preview 1 specification, wasmtime_wasi::preview1::WasiP1Ctx remains available. While WasiCtxBuilder is still used for its fluent configuration API, it must be finalized with the build_p1() method to produce the correct Preview 1 context.
Understanding these distinct API paths, coupled with the underlying architectural shift towards immutable contexts and explicit resource management through ResourceTable, is paramount for successful WebAssembly development with Wasmtime. By adopting these updated patterns and best practices, developers can successfully resolve current compilation errors and build more robust, secure, and future-proof WebAssembly applications with Wasmtime. This evolution represents a significant step forward in the WebAssembly ecosystem, promising greater modularity and interoperability.
Works cited
WASI and the WebAssembly Component Model: Current Status - eunomia, accessed July 1, 2025, https://eunomia.dev/blog/2025/02/16/wasi-and-the-webassembly-component-model-current-status/
WASI (WebAssembly System Interface) - ReadmeX, accessed July 1, 2025, https://readmex.com/en-US/bytecodealliance/wasmtime/page-5b235a7a4-9cbf-41f6-acdb-f80fdf07ecf0
Releases · bytecodealliance/wasmtime - GitHub, accessed July 1, 2025, https://github.com/bytecodealliance/wasmtime/releases/
Rust - wasmtime, accessed July 1, 2025, https://docs.wasmtime.dev/api/wasmtime/
wasmtime_wasi::preview1 - Rust, accessed July 1, 2025, https://docs.wasmtime.dev/api/wasmtime_wasi/preview1/index.html
docs.rs, accessed July 1, 2025, https://docs.rs/wasmtime/34.0.1
WASI - Wasmtime, accessed July 1, 2025, https://docs.wasmtime.dev/examples-rust-wasi.html
wasmtime_wasi::p2 - Rust, accessed July 1, 2025, https://docs.wasmtime.dev/api/wasmtime_wasi/p2/index.html
wasi_common - Rust - Wasmtime, accessed July 1, 2025, https://docs.wasmtime.dev/api/wasi_common/index.html
Preview 2 `WasiCtx` is not easy to update · Issue #6584 · bytecodealliance/wasmtime, accessed July 1, 2025, https://github.com/bytecodealliance/wasmtime/issues/6584
WasiCtxBuilder in wasmtime_wasi::p2 - Rust, accessed July 1, 2025, https://docs.wasmtime.dev/api/wasmtime_wasi/p2/struct.WasiCtxBuilder.html
