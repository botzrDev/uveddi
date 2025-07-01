
Architecting a Secure and Performant Plugin System for Uveddi with WebAssembly

Executive Summary
Uveddi requires a plugin architecture that enables safe, third-party extensibility without compromising the core application's security or performance. This report provides a comprehensive architectural blueprint for achieving this using WebAssembly (WASM). The analysis concludes with a set of strategic recommendations designed to provide Uveddi with a robust, scalable, and future-proof plugin ecosystem.
The primary recommendation is the adoption of the Wasmtime runtime. This choice is predicated on its security-first design philosophy, transparent development practices, leadership in emerging WASM standards, and the stable backing of the Bytecode Alliance. While other runtimes may offer higher peak performance in specific benchmarks, Wasmtime provides the optimal balance of security, performance, and long-term strategic alignment for an enterprise-grade system.
For inter-module communication, this report recommends architecting around the WebAssembly Component Model. This emerging standard automates the complex and error-prone task of data exchange across the host-guest boundary, offering both high performance and superior developer experience. For the specialized task of handling large, complex data structures like Abstract Syntax Trees (ASTs), a hybrid approach is advised: using the Component Model for API control flow and the Apache Arrow IPC format for structuring bulk data within shared memory buffers.
The security of the plugin system will be ensured through a multi-layered, defense-in-depth framework. This framework begins with a capability-based security model founded on the "deny-by-default" principle, implemented via the WebAssembly System Interface (WASI). This is reinforced by a rigorous plugin verification pipeline that includes static analysis of WASM bytecode, digital signature verification for authenticity and integrity, and strict manifest validation. At runtime, plugins will be further constrained by deterministic resource limits, including instruction-counting ("fuel") to prevent denial-of-service attacks and strict memory caps.
Finally, to meet the stringent performance requirement of operating within 30% of native code overhead, a holistic optimization strategy is essential. This strategy includes designing "chunky" APIs to minimize boundary-crossing costs, implementing a persistent Ahead-of-Time (AOT) compilation cache to ensure near-instantaneous plugin startup, and leveraging Rust's memory efficiency for plugin development. By following these integrated recommendations, Uveddi can build a powerful extensibility platform that fosters community contribution while rigorously upholding the integrity and performance of its core application.

Section 1: Foundational Runtime Analysis: A Comparative Study of Rust-Compatible WASM Runtimes

The selection of a WebAssembly runtime is the most critical architectural decision for the Uveddi plugin system. It dictates the foundation upon which all security, performance, and interoperability features will be built. This section provides a multi-faceted comparative analysis of the three leading Rust-compatible runtimes: Wasmtime, Wasmer, and WasmEdge. The evaluation focuses on performance characteristics, security posture, and the broader ecosystem to provide a definitive, data-driven recommendation for Uveddi.

1.1. Performance Benchmarking: Execution Speed, Compilation, and Memory

The performance of a WASM runtime is not a single metric but a complex interplay of execution speed, compilation strategy, and memory consumption. Achieving the Uveddi success criterion of plugin execution within 30% overhead of native Rust code requires a nuanced understanding of these factors.

1.1.1. Analysis of Execution Speed

Execution speed is highly dependent on the workload and the compiler backend used by the runtime.
Compiler Backends: The primary distinction is between LLVM-based backends, which perform extensive optimizations for the highest possible runtime speed, and Cranelift-based backends, which prioritize faster compilation times for a good-enough level of runtime performance.1 Wasmtime exclusively uses Cranelift, while Wasmer offers pluggable backends, including Cranelift (its "Universal" engine) and LLVM.2 WasmEdge also leverages an LLVM-based AOT compiler, positioning itself for high-performance use cases.3
Computational Throughput: For raw, compute-intensive tasks, runtimes utilizing an LLVM backend consistently demonstrate the highest performance, often achieving speeds that are very close to native code. Benchmarks show that Wasmer's LLVM backend can reach 95% of native speed on workloads like Coremark 5 and is generally 1.2x to 2.1x slower than native on other tasks.1 This makes LLVM-based runtimes an attractive option for long-running, computationally demanding plugins.
Balanced Performance: Cranelift-based runtimes, such as Wasmtime and Wasmer's Universal engine, offer a more balanced profile. While their peak execution speed is typically lower—often 1.5x to 4.6x slower than native code—they remain highly performant for a wide range of tasks.1 Independent benchmarks focusing on real-world, complex code like the
libsodium cryptographic library show that Wasmtime and Wasmer-Cranelift perform almost identically.7 This suggests that for many non-trivial workloads, the performance difference between the two main Cranelift implementations is negligible. In simple, direct comparisons like a Fibonacci calculation, Wasmtime has shown a slight advantage over Wasmer (Cranelift) in both execution time and memory usage.8
Achieving the Target: The goal of staying within a 30% performance overhead (equivalent to being no more than 1.43x slower than native) is ambitious. It is most likely achievable with an optimizing compiler like LLVM.1 However, this choice comes with a significant trade-off in compilation time, which directly impacts plugin load times and must be considered as part of the overall performance budget.

1.1.2. Analysis of Compilation Strategies (AOT vs. JIT)

The method and timing of compilation are critical for a plugin system, directly influencing startup latency and predictability.
Ahead-of-Time (AOT) vs. Just-in-Time (JIT): AOT compilation involves translating the entire WASM module to native machine code before it is executed for the first time. This approach, favored by server-side runtimes like Wasmtime and Wasmer, results in predictable and stable performance from the outset, as the expensive compilation step is done upfront.9 In contrast, JIT compilation occurs during program execution. While this allows for dynamic optimizations based on runtime behavior, it introduces a "warm-up" period where initial execution is slower, and it consumes more memory to store intermediate representations and profiling data.8 For the Uveddi plugin system, where predictable startup is crucial, AOT is the superior strategy.
Compilation Speed: There is a significant performance differential between compiler backends. Cranelift is designed for speed, enabling very fast module compilation. LLVM, while producing more optimized code, is orders of magnitude slower to compile.1 This makes Cranelift particularly well-suited for environments where plugins are loaded dynamically and startup time is a critical user-facing metric.
AOT Caching: A critical optimization for any AOT-based plugin system is the caching of compiled artifacts. Both Wasmtime and Wasmer support serializing a compiled module to a file. On the first load of a plugin, the system performs the AOT compilation and saves the result. On all subsequent loads of that same plugin, the system can bypass the compilation step entirely by deserializing the cached native code, drastically reducing startup time.1 This pattern is essential for mitigating the "cold start" penalty of AOT compilation, especially if using a slower backend like LLVM.

1.1.3. Analysis of Memory Overhead

WASM execution inherently introduces memory overhead compared to a native process, a factor that must be managed in a multi-plugin environment.
Runtime Memory Footprint: A simple native Rust program might consume under 2 MB of memory, whereas its WASM counterpart running in Wasmtime could require 12 MB, and in Wasmer, 24 MB.8 This overhead stems from the runtime's need to manage the sandbox, linear memory, and other associated data structures. Uveddi must account for this per-plugin overhead in its capacity planning and resource limiting.
Binary Size: The choice of source language for plugins also affects their footprint. Rust is an excellent choice as it produces highly optimized, small WASM binaries with minimal runtime dependencies. In contrast, languages like Swift can produce binaries that are over four times larger for the same simple program, due to the need to bundle a more substantial language runtime.12

1.2. Security Model and Posture: Sandboxing, Vulnerability Management, and WASI

For Uveddi, the security guarantees of the runtime are paramount. The analysis reveals significant differences in the security philosophy and practices of the leading runtimes.
Wasmtime's Security-First Approach: Wasmtime, developed under the governance of the Bytecode Alliance, has an explicit and demonstrable focus on security and correctness.13
Defense-in-Depth: Beyond the standard WASM sandbox, Wasmtime implements multiple additional layers of protection. These include placing a 2 GB guard region before a module's linear memory to protect against potential compiler bugs that could lead to out-of-bounds access, using guard pages on native thread stacks to detect overflows, and systematically zeroing memory after an instance is dropped to prevent data leakage between plugins.16
Supply Chain Security: Wasmtime is a notable adopter of cargo vet, a tool for methodically auditing every third-party dependency. This practice provides strong protection against supply chain attacks, a critical concern for any enterprise system.13
Formal Verification: The project actively collaborates with academic researchers to formally verify the correctness of security-critical components of the runtime and the Cranelift compiler. This pursuit of provable correctness is a strong indicator of a commitment to security.13
Vulnerability Disclosure: Wasmtime maintains a public, transparent, and detailed log of all security advisories on GitHub and has a documented CVE history.17 This transparency, while appearing to show more "vulnerabilities," is actually a sign of a mature and responsible security process, which is highly desirable for an enterprise partner.7
Wasmer's Security Features: Wasmer provides the foundational security features expected of a modern WASM runtime. It isolates workloads into their own processes with dedicated memory, network, and thread spaces, creating a strong sandbox.19 It also supports the definition of fine-grained, capability-based permissions to limit a module's access to host resources.19 However, its security posture from an enterprise perspective has notable weaknesses. The project's security disclosure process is opaque, directing reporters to a private email address and lacking a public, easily auditable history of vulnerabilities and their resolutions.13 This lack of transparency introduces a significant risk, as it makes independent assessment of the runtime's security track record difficult.
WasmEdge's Security Model: As a Cloud Native Computing Foundation (CNCF) project, WasmEdge provides a well-defined execution sandbox and follows standard secure development practices, including a formal disclosure timeline.22 Its primary architectural distinction is that it is written in C++.25 While a well-written C++ application can be secure, the language lacks the compile-time memory safety guarantees inherent to Rust, which increases the potential for vulnerabilities within the runtime itself.

1.3. Ecosystem, API, and Strategic Alignment

The long-term viability of the Uveddi plugin system depends on the health of the chosen runtime's ecosystem and its alignment with future standards.
Community and Corporate Backing: The organizational structure behind a runtime is a strong indicator of its long-term trajectory.
Wasmtime is backed by the Bytecode Alliance, a non-profit foundation with members including Mozilla, Fastly, Intel, and Red Hat.3 This diverse, collaborative backing suggests a focus on creating a stable, open standard rather than a specific commercial product, reducing the risk of vendor lock-in or sudden strategic pivots.
Wasmer is a commercial entity. While this drives innovation in features and broad language support, it also ties the future of the runtime to the success and strategic direction of a single company.8
WasmEdge is a CNCF sandbox project, giving it strong credibility and support within the cloud-native community.22
API Stability and Ease of Embedding: All three runtimes can be embedded in Rust.
Wasmtime's Rust API is specifically designed to be ergonomic and safe, guaranteeing that an embedder does not need to use unsafe code to interact with the runtime.16 Its C API is also noted for its stability, which is a positive sign for the overall API design philosophy.7 The project provides clear and comprehensive embedding examples.27
Wasmer also provides extensive language bindings and is designed to be embeddable.2 However, its history suggests a greater propensity for API changes as the product evolves.32
WasmEdge's C++ core means that its Rust SDK is a wrapper around a C FFI boundary.25 This introduces an additional layer of complexity and potential for impedance mismatch compared to the pure-Rust runtimes.
Alignment with Emerging Standards: The WebAssembly ecosystem is rapidly evolving. Aligning with emerging standards like the WebAssembly Component Model is crucial for future-proofing the architecture.
Wasmtime and the Bytecode Alliance are the primary drivers of the Component Model and the WASI Preview 2 specification.34 Choosing Wasmtime inherently aligns Uveddi with the core group defining the future of interoperable WASM.
Wasmer has historically taken a more pragmatic approach, sometimes creating its own non-standard extensions, such as WASIX, to meet immediate user needs that the standards process has not yet addressed (e.g., adding a fork() syscall).35 While this provides more features today, it creates a risk of fragmentation and divergence from the main standards track.

1.4. Runtime Comparison Summary

The choice of runtime is not merely a technical selection but a strategic commitment to a particular development philosophy and ecosystem. A purely performance-based comparison is misleading. For instance, claims of superior speed often hinge on using an LLVM backend, which comes with a severe compile-time penalty—a critical factor for a dynamic plugin system.1 When comparing like-for-like compiler backends (Cranelift), Wasmtime and Wasmer show very similar performance profiles.7
Therefore, the decision must be based on a more holistic view that prioritizes the non-functional requirements essential for an enterprise system like Uveddi. The stark contrast in security posture becomes a deciding factor. Wasmtime's commitment to a security-first development process—evidenced by its defense-in-depth mechanisms, supply chain auditing, formal verification efforts, and, most importantly, transparent vulnerability disclosure—provides a level of assurance that is critical for a platform that will run untrusted third-party code. This stands in contrast to the more opaque security processes of other runtimes. Aligning with Wasmtime also means aligning with the community-driven, standards-first approach of the Bytecode Alliance, which minimizes long-term architectural risk and ensures compatibility with the future of WebAssembly.
Criterion
Wasmtime
Wasmer
WasmEdge
Primary Backing
Bytecode Alliance (Mozilla, Fastly, Intel) 3
Wasmer, Inc. (Commercial Entity) 8
CNCF (Sandbox Project) 22
Core Language
Rust 13
Rust 13
C++ 25
Security Posture
Excellent: Security-first design, transparent disclosure, formal verification, cargo vet supply chain security.13
Adequate but Opaque: Strong sandboxing but lacks transparent vulnerability disclosure process.13
Good: Standard secure development practices, but C++ core introduces higher intrinsic risk than Rust.22
Performance (Cranelift)
Excellent: Fast compilation, good runtime speed, low memory overhead.7
Excellent: Similar performance profile to Wasmtime when using the same Cranelift backend.7
N/A
Performance (LLVM)
N/A
Excellent (Runtime) / Poor (Compile Time): Near-native execution at the cost of very slow compilation.1
Excellent (Runtime) / Poor (Compile Time): Similar profile to Wasmer-LLVM.3
API & Embedding
Excellent: Ergonomic and safe Rust API, stable C API, well-documented.7
Good: Wide language support, but historically more API churn.2
Fair: Rust SDK is a wrapper over C FFI, adding complexity.25
Component Model Support
Leader: Actively driving the standard.34
Follower: Adopting standards but also creating non-standard extensions (WASIX).35
Lagging: Focus on custom extensions over Component Model integration.7
Strategic Risk
Low: Aligned with community-driven standards and a broad industry alliance.
Medium: Dependent on the roadmap and success of a single commercial vendor.
Medium: Risk of divergence from core WASM standards due to focus on cloud-native extensions.


1.5. Recommendation for Uveddi

Based on this comprehensive analysis, Wasmtime is the unequivocally recommended runtime for the Uveddi plugin system.
This recommendation is justified by Wasmtime's superior and transparent security posture, which is a non-negotiable requirement for a system that executes third-party code. Its leadership in developing and adopting the WebAssembly Component Model ensures that Uveddi will be building on a future-proof foundation. The backing of the Bytecode Alliance provides stability and mitigates the risks associated with dependency on a single vendor. While Wasmtime forgoes the absolute peak performance offered by an LLVM backend, its Cranelift compiler provides an excellent balance of fast plugin load times and high-quality runtime performance that is well-suited to the dynamic nature of a plugin architecture. This combination of security, standards alignment, and balanced performance makes Wasmtime the most prudent and powerful choice for Uveddi.

Section 2: Inter-Module Communication: Protocols and Patterns for Efficient Data Exchange

Once a runtime is chosen, the next critical architectural challenge is defining how the Rust host application and WASM plugins will communicate. Efficient and type-safe data exchange is essential, especially for a domain like code analysis which involves passing complex data structures such as Abstract Syntax Trees (ASTs). This section specifies the optimal protocols and patterns to achieve this, balancing performance with developer experience.

2.1. The Challenge: Crossing the WASM Boundary

The fundamental challenge of host-guest communication in WebAssembly stems from its security-oriented design. The WASM virtual machine is a simple, stack-based machine with a sandboxed, linear memory space.
A Numeric-Only Boundary: The core WASM specification only allows primitive numeric types—specifically 32-bit and 64-bit integers and floats—to be passed directly as function arguments or return values.36
Manual Memory Marshalling: To exchange any complex data type, such as a string, a vector, or a structured object, the caller and callee must cooperate to transfer data through the WASM module's linear memory. This typically involves the caller (e.g., the host) allocating a buffer in the callee's (the plugin's) memory, writing the serialized data into that buffer, and then passing a pointer (an integer offset) and a length to the callee function. This process is manual, inefficient, and highly prone to error.38
The Performance Bottleneck: This boundary-crossing, with its associated serialization and memory copying, is a primary source of performance overhead. For APIs that require frequent, small interactions ("chatty" APIs), this overhead can easily dominate the actual computational work, making it impossible to meet high-performance goals.6

2.2. Serialization Format Analysis for Code Analysis Data

The choice of serialization format directly impacts the performance of the data exchange. For Uveddi, which deals with potentially large and complex ASTs, this choice is critical.
Traditional Formats (JSON, Protobuf):
JSON: While human-readable and easy to debug, JSON is a text-based format that suffers from significant parsing overhead and a verbose, large memory footprint. It is unsuitable for performance-critical data exchange.43
Protocol Buffers (Protobuf): As a binary, schema-driven format, Protobuf offers a substantial improvement over JSON. It produces much smaller payloads and is significantly faster to serialize and deserialize. It is a battle-tested and robust choice for many RPC systems.43 However, it still fundamentally relies on a serialize-copy-deserialize workflow, which involves CPU-intensive steps and memory allocation that can be a bottleneck for very large datasets.44
Zero-Copy Deserialization Formats: This class of formats is designed to eliminate the costly deserialization step. Data is serialized into a specific binary layout that allows the reader to access fields directly from the byte buffer without parsing the entire structure into new memory allocations. This is ideal for read-heavy workloads, which is common in analysis plugins.
Flatbuffers and rkyv: These are prominent examples of zero-copy formats. They enable extremely fast read access, as they essentially provide a pointer to the data in place.43 The trade-off is often a more complex and less intuitive API for writing or building the data structures, as they are optimized for read speed, not write simplicity.43
Apache Arrow: Arrow is a specification for a language-agnostic, columnar in-memory data format. It is specifically designed for high-performance, large-scale data processing and analytics.46 Its key advantage for Uveddi is its columnar layout, which is highly efficient for querying and accessing subsets of large, structured data like an AST. For example, a plugin could efficiently iterate over all "function declaration" nodes without needing to traverse or deserialize the entire tree. Using Arrow's Inter-Process Communication (IPC) format provides a standardized, zero-copy-capable way to share this complex data between the Rust host and a WASM plugin.46

2.3. The Strategic Solution: The WebAssembly Component Model

While optimizing the serialization format is a valid approach, the WebAssembly Component Model offers a more fundamental and strategic solution to the data exchange problem. It aims to make cross-boundary communication a seamless, toolchain-level concern rather than a manual, application-level one.
A Standard for Interoperability: The Component Model is a forward-looking proposal that extends core WASM by defining a standard way for components to interact, regardless of their source language.47 It is the centerpiece of the WASI Preview 2 standard and is being driven by the Bytecode Alliance.34
High-Level Interfaces with WIT: The model introduces the WebAssembly Interface Definition Language (WIT). WIT allows developers to define the interface of a component using high-level types like strings, lists, records, and variants, instead of just raw numbers.47 A WIT file acts as the contract between the host and the plugin.
Automated Glue Code Generation: The true power of the Component Model lies in its tooling. A tool like wit-bindgen takes a WIT definition and automatically generates all the necessary "glue code" for a specific language (like Rust).34 This generated code handles the complex and error-prone tasks of memory management, serialization, and data marshalling across the WASM boundary, abstracting it away from the plugin developer and the host implementer.47
Standardized Memory Management: The Component Model's Canonical ABI specifies how memory for data exchange is managed. For instance, if a host needs to pass a string to a guest, the host can call a canonical realloc function, which is guaranteed to be exported by the component, to allocate the necessary space within the guest's linear memory. The host then writes the string data into this allocated buffer.51 This standardized approach eliminates the need for each plugin to invent its own memory management conventions for host communication.
The Component Model represents a paradigm shift. Instead of manually optimizing data transfer with a specific serialization format, developers can define a clean, high-level interface and let the toolchain generate highly optimized, low-level glue code. This provides both superior developer experience and high performance.

2.4. Protocol Comparison and Hybrid Strategy

The choice of a communication protocol is not merely about picking the fastest format; it is a foundational architectural decision that defines the performance ceiling, developer experience, and long-term maintainability of the entire plugin system. A naive approach using traditional serialization like JSON would impose a severe performance penalty due to parsing and copying, making it difficult to meet Uveddi's performance goals.43 A zero-copy format like Apache Arrow offers a direct performance optimization, but requires manual integration and management.46
The WebAssembly Component Model provides the most robust and future-proof solution. By operating at a higher level of abstraction, it solves the data transfer problem systemically, providing both performance and excellent developer ergonomics.34 It is the clear direction in which the entire WebAssembly ecosystem is moving.
However, given that the Component Model is still an emerging standard, and that Apache Arrow offers unparalleled efficiency for the specific domain of large-scale data analysis, a hybrid strategy is recommended for Uveddi during the transition period. This strategy leverages the strengths of both approaches:
Control Plane: Use the Component Model and WIT to define the primary plugin API. This includes functions for registration, configuration, and invoking analysis tasks. This provides a clean, type-safe, and ergonomic interface for all standard interactions.
Data Plane: For the transfer of very large, performance-critical data payloads like full ASTs, the WIT interface should be designed to pass handles (e.g., simple integer IDs or pointers) that refer to large data blocks residing in the plugin's linear memory. These data blocks should be structured using the highly efficient Apache Arrow IPC format.
This two-tiered protocol gives Uveddi the best of both worlds: the developer-friendly, standardized, and automated glue code of the Component Model for the majority of API interactions, combined with the raw, specialized performance of Apache Arrow for its most demanding data analysis tasks.

Approach
Performance (Overhead)
Payload Size
Developer Experience (DX)
Maturity & Future-Proofing
JSON Serialization
High (Parsing & Copying)
Large
Simple (Debugging), Poor (Boilerplate)
Mature, but not for high-performance
Protobuf Serialization
Medium (Serialization & Copying)
Small
Good (Schema-driven)
Very Mature
Bincode (Rust-specific)
Low
Very Small
Excellent (in Rust)
Poor (No cross-language spec)
Zero-Copy (Flatbuffers/rkyv)
Very Low (No Deserialization)
Medium
Fair (Complex write API)
Mature
Zero-Copy (Apache Arrow)
Very Low (Optimized for Analytics)
Medium
Good (Rich ecosystem)
Very Mature (in data science)
WASM Component Model (WIT)
Low to Very Low (Automated glue code)
Varies
Excellent (High-level, type-safe)
Excellent (The future standard)


2.5. Protocol Specification for Uveddi

It is recommended that Uveddi adopt the WebAssembly Component Model as the primary framework for all plugin interface definitions. For handling large code analysis data, this should be supplemented by using the Apache Arrow IPC format for the data plane.
A sample WIT file (plugin.wit) demonstrating this hybrid approach could look as follows:

Code snippet


// file: plugin.wit
package uveddi:plugins

// Define the world that our plugins will implement.
// This world imports host functionality and exports plugin functionality.
world code-analyzer {
  // Import host functions that plugins can call.
  import logging: func(level: string, message: string)
  import get-config-value: func(key: string) -> option<string>

  // Define a handle to a large, Arrow-formatted data buffer.
  // This is an opaque handle managed by the host.
  resource ast-handle {
    // Methods on the handle could be defined here if needed.
  }

  // A record to represent a single analysis finding.
  record issue {
    file-path: string,
    start-line: u32,
    end-line: u32,
    code: string,
    message: string,
  }

  // The main export of the plugin.
  // The host calls this function to trigger an analysis.
  // It passes a handle to an AST buffer. The plugin returns a list of issues.
  export analyze: func(tree: borrow<ast-handle>) -> list<issue>

  // A function for the host to load an AST into the plugin's memory
  // and get a handle back. The buffer is in Arrow IPC format.
  export load-ast: func(ast-buffer: list<u8>) -> ast-handle

  // A function for the host to release the memory associated with an AST handle.
  export free-ast: func(handle: ast-handle)
}


This interface demonstrates the hybrid model:
Standard interactions like logging and configuration use simple, high-level WIT types (string, option<string>).
The large AST is not passed directly. Instead, the host calls load-ast with the Arrow-formatted byte buffer. The plugin manages this buffer internally and returns an opaque ast-handle.
The main analyze function operates on this efficient handle, avoiding repeated data transfer.
This design is both performant and clean, providing a clear and robust contract for all plugin developers.

Section 3: A Multi-Layered Security Architecture for the Plugin Ecosystem

A secure plugin system is non-negotiable. It requires a defense-in-depth strategy that protects the Uveddi application and its users at every stage of the plugin lifecycle, from submission and verification to execution and resource management. This section details a multi-layered security architecture founded on the principle of least authority.

3.1. The Principle of Least Authority: A Capability-Based Security Model

The foundational security principle for the plugin system must be "deny-by-default." A plugin should have no inherent authority to access system resources; it must be explicitly granted every permission it needs to perform its function.52 This is known as capability-based security.
WASI as the Enforcement Mechanism: The WebAssembly System Interface (WASI) is the standard for implementing capability-based security for WASM.53 It fundamentally inverts the traditional security model. Instead of a process running with ambient authority (e.g., the user's permissions) and trying to access resources like files by their name, a WASI-compliant module starts with zero authority.56 It cannot see the host filesystem or network. The host application must explicitly grant it a "capability"—an unforgeable handle (akin to a file descriptor)—to a specific resource. The plugin can then only perform operations on or relative to that handle.54
Implementation with Wasmtime: Wasmtime provides a rich and ergonomic API for configuring these capabilities at instantiation time. The wasmtime_wasi::WasiCtxBuilder is the primary tool for this. The host application can use it to precisely define the plugin's environment, including:
Filesystem Access: Granting read-only or read-write access to specific host directories, which are then mapped into the plugin's virtual filesystem.29
Environment Variables: Passing a curated list of environment variables, preventing leakage of sensitive host information.29
Networking: Providing pre-opened socket handles for specific, allowed network endpoints.54
The Role of the Component Model: The Component Model naturally extends this paradigm. The interfaces defined in a WIT file act as a set of capabilities. A plugin component can only interact with the host through the functions it explicitly imports in its world definition.48 The runtime enforces this contract, ensuring that a plugin cannot call an undeclared host function. This provides a high-level, language-agnostic mechanism for enforcing the principle of least authority at the API level.37
The logic within the Uveddi host that configures the WasiCtxBuilder for each plugin is therefore the most security-critical component of the entire architecture. It is the gatekeeper that mints and bestows all authority. This code must be subject to the highest level of scrutiny and testing to ensure it correctly and minimally grants capabilities based on a verified plugin manifest.

3.1.1. Implementation Guidelines for Capability Scoping

The following Rust code snippets, using the wasmtime and wasmtime_wasi crates, demonstrate how to enforce different levels of privilege for a plugin.
1. Instantiating a Plugin with No System Access (Pure Computation):

Rust


use wasmtime::{Engine, Linker, Module, Store};
use wasmtime_wasi::WasiCtx;

//... engine and module setup...
let mut linker = Linker::new(&engine);
wasmtime_wasi::add_to_linker(&mut linker, |s: &mut WasiCtx| s)?;

// Create a WASI context with no permissions.
let wasi = wasmtime_wasi::WasiCtxBuilder::new().build();
let mut store = Store::new(&engine, wasi);

// Instantiate. This plugin can perform calculations but cannot access I/O.
let instance = linker.instantiate(&mut store, &module)?;


2. Instantiating a Plugin with Read-Only Access to a Specific Directory:

Rust


use wasmtime::{Engine, Linker, Module, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};
use cap_std::fs::Dir;

//... engine and module setup...
let mut linker = Linker::new(&engine);
wasmtime_wasi::add_to_linker(&mut linker, |s: &mut WasiCtx| s)?;

// Open the host directory we want to grant access to.
let preopened_dir = Dir::open_ambient_dir("/path/to/project", cap_std::ambient_authority())?;

// Create a WASI context, granting read-only access to the pre-opened directory
// and mapping it to '/src' inside the plugin's virtual filesystem.
let wasi = WasiCtxBuilder::new()
   .preopened_dir(preopened_dir, wasi_common::file::FdFlags::empty(), "/src")?
   .build();
let mut store = Store::new(&engine, wasi);

// This plugin can read from '/src' but cannot write or access any other path.
let instance = linker.instantiate(&mut store, &module)?;



3.2. Proactive Threat Mitigation: Plugin Verification Framework

Before a plugin is ever executed, it must pass through a rigorous verification pipeline to proactively identify and block threats. This pipeline should be an automated part of the plugin submission and publication process.
Static Analysis of WASM Bytecode: Many vulnerabilities, especially those inherited from unsafe source languages like C or C++, can be detected by statically analyzing the WASM bytecode itself.58 Tools like
Wasmati construct a Code Property Graph (CPG) from the binary and use it to query for vulnerability patterns such as integer overflows, buffer overflows, or dangerous uses of imported functions.58 The Uveddi plugin ingestion pipeline must integrate such a scanner and automatically reject any plugin that contains high-severity vulnerabilities.
Code Signing and Integrity Verification: To guarantee the authenticity (who created it) and integrity (it hasn't been tampered with) of a plugin, a digital signature scheme is essential.61
Embedded Signature Approach: A practical and self-contained method is to embed an ECDSA signature within a custom section of the .wasm file.62 When a plugin is published, the author signs the hash of the binary with their private key. The Uveddi host, upon loading the plugin, can then verify this signature against the author's public key, which could be fetched from a trusted registry. This ensures the plugin is from a known source and has not been modified.61
Registry-Based (TUF) Approach: For a more mature ecosystem, a framework like The Update Framework (TUF) offers stronger protection against supply chain attacks, such as a compromised plugin registry. In this model, the plugin binary is stored in an OCI-compliant registry, while its cryptographic digest and signature are stored on a separate, highly secured trust server (e.g., Notary).64 The client verifies the integrity by cross-referencing both sources. For initial implementation, the embedded signature is sufficient, but the architecture should be planned with a potential migration to TUF in mind.
Manifest Validation: Every plugin must be accompanied by a machine-readable manifest file (e.g., plugin.toml) that serves as a declaration of intent.
This manifest must explicitly enumerate all permissions the plugin requires, such as filesystem paths, network hosts, or environment variables. Example: permissions = ["fs:read:/src", "net:connect:api.github.com"].
The host application must parse this manifest during the verification stage. The permissions declared in the manifest become the upper bound of what the host is allowed to grant the plugin at runtime. This prevents a user from accidentally granting a plugin more permissions than its author intended and is a standard best practice in secure plugin systems like Chrome Extensions.65

3.3. Runtime Defense: Resource Limiting and Denial-of-Service Prevention

Even a verified plugin may contain bugs or malicious logic designed to disrupt the host application. Runtime defenses are necessary to contain the impact of a misbehaving plugin.
CPU and Execution Time Limiting: A primary threat is a plugin entering an infinite loop, consuming 100% of a CPU core and causing a denial-of-service (DoS).66
Instruction Counting ("Fuel"): The most robust and deterministic method for preventing this is instruction counting, often called "gas" or "fuel" metering. Before execution, the host allocates a certain amount of "fuel" to the plugin instance. The runtime is instrumented to decrement this fuel counter as it executes WASM instructions. If the fuel runs out, the execution is deterministically trapped.68 Wasmtime provides this mechanism via
Config::consume_fuel(true) and Store::set_fuel().70 While it introduces a moderate performance overhead, its determinism is invaluable for security.
Epoch-Based Timeouts: A lower-overhead alternative is epoch-based interruption. The host maintains an "epoch" counter that is periodically incremented by a timer in a separate thread. The WASM code is instrumented with checks against this epoch counter. If the deadline is passed, execution traps.70 This method is effective at stopping long-running computations but is non-deterministic and may not catch very tight, fast infinite loops.
Recommendation: For its predictability and stronger guarantee against DoS, the fuel-based mechanism is recommended. The amount of fuel allocated per invocation should be a configurable policy within Uveddi.
Memory Capping: A plugin must be prevented from consuming unbounded amounts of memory.67
The WebAssembly standard allows a module's linear memory to be defined with an initial and a maximum size, measured in 64KiB pages.72
The Uveddi host must enforce a sensible maximum limit during plugin instantiation. For example, MemoryType::new(1, Some(4096)) would give the plugin an initial 64KiB but cap its growth at 256MB (4096 pages). This prevents a single plugin from exhausting host memory.
Logging and Output Limiting: To prevent a plugin from flooding log files or standard output (a "log bomb" DoS), the host must intercept and rate-limit the data written to these streams. A per-invocation cap, such as the 16 KiB limit used by Google Cloud's Wasm service, is a reasonable practice.71
This defense-in-depth approach, combining proactive verification with robust runtime containment, creates a security posture where a failure in one layer is likely to be caught by another. This layered model is essential for building the trust required to foster a vibrant and safe third-party plugin ecosystem.

Section 4: Performance Optimization and Best Practices

Meeting the success criterion of keeping plugin overhead within 30% of native Rust code requires a holistic performance strategy. This is not just about the raw CPU speed of WASM execution but also encompasses the costs of compilation, host-guest communication, and memory management. The performance target must be viewed as a system-wide budget, and optimizations must be applied at each stage of the plugin lifecycle.

4.1. Minimizing Host-Guest Call Overhead

The boundary between the host application and the WASM plugin is a significant performance bottleneck. Every call across this boundary incurs overhead from context switching, data marshalling, and type validation.41
Design "Chunky" not "Chatty" APIs: The most effective way to mitigate this overhead is through API design. Instead of creating "chatty" interfaces that require many small, frequent calls to get work done (e.g., a separate call for each token in a file), APIs should be designed to be "chunky." A chunky API performs a complete, meaningful unit of work in a single call, thereby amortizing the fixed cost of the boundary crossing over a much larger amount of useful computation.6 For example, an API function like
analyze_syntax_tree(tree_buffer) is vastly more efficient than an API that requires the host to traverse the tree and call a plugin function for each node.
Leverage Zero-Copy Techniques: As detailed in Section 2, the cost of serializing and copying data into and out of the plugin's linear memory is a major component of call overhead. Adopting the WebAssembly Component Model, which automates the generation of optimized marshalling code, is the primary strategy for reducing this cost.51 For very large data payloads, supplementing this with a format like Apache Arrow, which allows for direct, zero-copy access to the data in the buffer, will further enhance performance.46

4.2. Compilation and Caching Strategies

Plugin startup time is a critical component of the user experience. A slow-loading plugin can make the entire application feel unresponsive. The key to fast startup is an intelligent Ahead-of-Time (AOT) compilation strategy.
AOT Compilation as the Standard: For any production environment, plugins must be AOT compiled to native machine code. This provides fast and, more importantly, predictable execution performance, eliminating the "warm-up" latency and overhead of JIT compilation.1
Implement a Persistent AOT Cache: The single most important optimization for plugin load time is to implement a cache for AOT-compiled artifacts. The workflow should be as follows:
When a specific version of a plugin is requested for the first time, the host uses the runtime (e.g., Wasmtime) to perform the AOT compilation.
The resulting native code artifact is saved to a persistent cache on disk, indexed by a hash of the original .wasm file.
On all subsequent requests for that same plugin, the host checks the cache. If a valid artifact exists, it loads it directly, completely bypassing the expensive compilation step.
Performance Impact: This caching mechanism transforms the performance characteristics of the plugin system. The "cold start" time for a new plugin might be in the hundreds of milliseconds, dominated by compilation. However, the "warm start" time for a cached plugin can be reduced to single-digit milliseconds, dominated only by the time it takes to read the artifact from disk and map it into memory. Both Wasmtime and Wasmer provide the necessary APIs for this pattern, typically through Module::serialize and Module::deserialize (or Engine::precompile_module in Wasmtime).1

4.3. Memory Management Best Practices

Efficient memory management reduces the overall footprint of the application and prevents performance degradation due to memory pressure.
Standardized Allocation via the Component Model: As discussed in Section 2, the Component Model provides a canonical ABI that includes a standardized realloc function exported by the component.51 The host should use this function to allocate memory within the plugin's linear memory for passing data. This avoids forcing each plugin to bundle its own memory manager and provides a clean, consistent mechanism for memory operations across the boundary.
Acknowledge Grow-Only Memory: A crucial characteristic of the current WebAssembly memory model is that linear memory can grow but can never shrink.76 A plugin that has a transient high-water mark for memory usage will retain that large memory allocation for its entire lifetime. This can lead to a bloated memory footprint for the host application. Plugin developers should be educated on this limitation and encouraged to design their plugins to be memory-frugal.
Future-Proofing for memory.discard: The memory.discard proposal is designed to address the grow-only limitation. It will introduce an instruction that allows a module to inform the host that a range of memory pages is no longer in use. The host can then release the underlying physical memory back to the operating system, reducing the application's true memory footprint.76 While this feature is not yet finalized, designing plugins with memory locality in mind (i.e., allocating transient data contiguously) can help prepare the ecosystem to take advantage of it when it becomes available.
The Rust Advantage: Encouraging or requiring plugins to be written in Rust provides significant performance benefits. Rust's compile-time memory safety guarantees and lack of a heavy, garbage-collected runtime result in WASM modules that are smaller, faster, and have more predictable memory usage patterns than modules compiled from languages like Go, Swift, or anything requiring a large runtime.12
There is an inherent and unavoidable tension between achieving maximum performance and ensuring robust security and determinism. The fastest possible execution would involve no sandboxing, no bounds checks, and no resource metering. However, for the Uveddi plugin system, security is paramount. Therefore, certain performance trade-offs are necessary costs of enabling a safe, extensible ecosystem. For example, the deterministic fuel-based timeout mechanism is computationally more expensive than the non-deterministic epoch-based one, but it provides a stronger guarantee against DoS attacks.70 The architecture must allow for these trade-offs to be made consciously and, where appropriate, configurably. For highly trusted, internally developed plugins, it might be acceptable to increase fuel limits to maximize performance. For untrusted community plugins, a stricter, more conservative limit is required. The key is to build a system that can support this policy-driven configuration.

Section 5: Conclusion and Implementation Roadmap

This report has provided a comprehensive architectural analysis for building a secure, performant, and scalable plugin system for Uveddi using WebAssembly. The findings indicate that with a carefully considered architecture, it is possible to achieve the goals of safe third-party extensibility while maintaining system integrity and meeting stringent performance targets.

5.1. Summary of Architectural Recommendations

The following core recommendations form the blueprint for the Uveddi plugin architecture:
Runtime: Adopt Wasmtime as the core WASM runtime. Its security-first design, transparent development process, leadership in emerging standards (Component Model), and the stable backing of the Bytecode Alliance make it the most robust and strategically sound choice for an enterprise system.
Communication Protocol: Standardize all plugin interfaces using the WebAssembly Component Model and its Interface Definition Language (WIT). This provides a high-level, type-safe, and future-proof foundation for host-guest communication. For the specialized transfer of large data payloads like ASTs, supplement this by passing handles to memory buffers formatted with the Apache Arrow IPC specification to achieve maximum data plane performance.
Security Framework: Implement a multi-stage, defense-in-depth security pipeline that operates throughout the plugin lifecycle. This pipeline must include:
Verification: Pre-execution checks involving static analysis for vulnerabilities, code signing for authenticity, and strict manifest validation for declared permissions.
Sandboxing: A strict capability-based model at instantiation, configured by the host based on the verified manifest, using WASI and Wasmtime's APIs.
Containment: Deterministic runtime resource limits, including fuel-based instruction counting to prevent CPU-based DoS attacks and hard memory caps to prevent memory exhaustion.
Performance Strategy: Pursue a holistic performance optimization strategy focused on minimizing all sources of overhead. This includes:
API Design: Creating "chunky" APIs that minimize the frequency of expensive boundary crossings.
AOT Caching: Implementing a persistent cache for AOT-compiled artifacts to ensure near-instantaneous "warm starts" for plugins.
Language Choice: Promoting Rust as the primary language for plugin development to leverage its performance and memory efficiency.

5.2. Phased Implementation Plan

To translate these architectural recommendations into a functional system, the following phased implementation plan is proposed for the Uveddi engineering team.
Phase 1: Core Runtime & Communication Proof of Concept (PoC)
Objective: Validate the core technical choices and establish a baseline for performance.
Key Tasks:
Integrate the wasmtime and wasmtime-wasi crates into a branch of the Uveddi application.
Define a simple plugin interface using WIT (e.g., a basic linter).
Develop a prototype Rust plugin that implements this WIT interface, compiling it to a WASM component.
Implement the host-side logic to load, instantiate, and call the component using wit-bindgen-generated bindings.
Establish a benchmark suite to measure the end-to-end overhead of a simple plugin call, creating a performance baseline.
Phase 2: Security Framework Implementation
Objective: Build the foundational security layers for sandboxing and runtime containment.
Key Tasks:
Implement the capability-based security model. The host logic must be built to configure the WasiCtxBuilder based on a set of permissions (e.g., no filesystem access by default).
Develop the plugin.toml manifest format and the host-side logic to parse and validate it. The permissions granted at instantiation must be constrained by the manifest.
Integrate Wasmtime's fuel metering to enforce configurable execution limits on plugin invocations.
Enforce a configurable maximum memory limit for all plugin instances.
Phase 3: Plugin Lifecycle Management & Verification
Objective: Create the infrastructure for securely managing plugins from submission to deployment.
Key Tasks:
Design and build the infrastructure for developers to submit and publish plugins (e.g., a simple web portal or CLI tool).
Integrate a static analysis tool (e.g., Wasmati) into the submission pipeline to automatically scan for vulnerabilities.
Implement the chosen code signing and verification workflow (e.g., embedded ECDSA signatures). The host must verify the signature before any other step.
Develop a plugin registry to store metadata, public keys, and verified plugin binaries.
Phase 4: Performance Optimization & Ecosystem Launch
Objective: Finalize performance optimizations and prepare for a public launch.
Key Tasks:
Implement the persistent AOT cache to drastically reduce plugin load times.
Refine the performance benchmark suite to cover a wider range of realistic use cases.
Author and publish comprehensive documentation and best-practice guides for third-party plugin developers.
Launch a beta program for the Uveddi plugin ecosystem, inviting initial community contributions.
Works cited
Benchmarking WebAssembly Runtimes | by Brandon Fish | Wasmer, accessed June 28, 2025, https://blog.wasmer.io/benchmarking-webassembly-runtimes-18497ce0d76e
Wasmer vs Wasmtime, accessed June 28, 2025, https://wasmer.io/wasmer-vs-wasmtime
Comparing WebAssembly Runtimes: Wasmer vs. Wasmtime vs. Wasmedge — Unveiling the Power of Wasm | by Ashish Singh | Medium, accessed June 28, 2025, https://medium.com/@siashish/comparing-webassembly-runtimes-wasmer-vs-wasmtime-vs-wasmedge-unveiling-the-power-of-wasm-ff1ecf2e64cd
WasmEdge Features | WasmEdge Developer Guides, accessed June 28, 2025, https://wasmedge.org/docs/start/wasmedge/features
Announcing Wasmer 6.0 - closer to Native speeds! · Blog, accessed June 28, 2025, https://wasmer.io/posts/announcing-wasmer-6-closer-to-native-speeds
“Near-Native Performance”: Wasm is often described as having “near-native perf... | Hacker News, accessed June 28, 2025, https://news.ycombinator.com/item?id=30156437
Performance of WebAssembly runtimes in 2023 - Frank DENIS ..., accessed June 28, 2025, https://00f.net/2023/01/04/webassembly-benchmark-2023/
Performance Comparison Analysis: Wasmer vs. WASMTime | by Mohammadreza Ashouri, accessed June 28, 2025, https://ashourics.medium.com/performance-comparison-analysis-wasmer-vs-wasmtime-48c6f51b536f
Comparing AOT and JIT Compilers: Understanding the Differences and Making an Informed Choice — Programming concepts - Amin Nezampour, accessed June 28, 2025, https://aminnez.com/programming-concepts/jit-vs-aot-compiler-pros-cons
For my project, is a jit or compiler better? : r/rust - Reddit, accessed June 28, 2025, https://www.reddit.com/r/rust/comments/1aq4ayw/for_my_project_is_a_jit_or_compiler_better/
jit - What are the advantages of just-in-time compilation versus ahead-of-time compilation?, accessed June 28, 2025, https://stackoverflow.com/questions/2106380/what-are-the-advantages-of-just-in-time-compilation-versus-ahead-of-time-compila
The Six Ways of Optimizing WebAssembly - InfoQ, accessed June 28, 2025, https://www.infoq.com/articles/six-ways-optimize-webassembly/
Choosing a WebAssembly Run-Time - Colin Breck, accessed June 28, 2025, https://blog.colinbreck.com/choosing-a-webassembly-run-time/
Wasmtime, accessed June 28, 2025, https://wasmtime.dev/
Bytecode Alliance — Security and Correctness in Wasmtime, accessed June 28, 2025, https://bytecodealliance.org/articles/security-and-correctness-in-wasmtime
Security - Wasmtime, accessed June 28, 2025, https://docs.wasmtime.dev/security.html
Security Policy - bytecodealliance/wasmtime - GitHub, accessed June 28, 2025, https://github.com/bytecodealliance/wasmtime/security
Wasmtime CVEs and Security Vulnerabilities - OpenCVE, accessed June 28, 2025, https://www.opencve.io/cve?vendor=bytecodealliance&product=wasmtime
Architecture of Wasmer Edge, accessed June 28, 2025, https://docs.wasmer.io/edge/architecture
Wasmer: Universal applications using WebAssembly, accessed June 28, 2025, https://wasmer.io/
wasmer/docs/SECURITY.md at main - GitHub, accessed June 28, 2025, https://github.com/wasmerio/wasmer/blob/main/docs/SECURITY.md
Self Assessment - CNCF TAG Security - Cloud Native Computing Foundation, accessed June 28, 2025, https://tag-security.cncf.io/community/assessments/projects/wasmedge/self-assessment/
WasmEdge: Transforming Cloud and Edge Computing with WebAssembly - Adyog, accessed June 28, 2025, https://blog.adyog.com/2024/09/16/wasmedge-transforming-cloud-and-edge-computing-with-webassembly/
WasmEdge is a lightweight, high-performance, and extensible WebAssembly runtime for cloud native, edge, and decentralized applications. It powers serverless apps, embedded functions, microservices, smart contracts, and IoT devices. - GitHub, accessed June 28, 2025, https://github.com/WasmEdge/WasmEdge
Any Rust Developer currently developing for WasmEdge as a target? : r/rust - Reddit, accessed June 28, 2025, https://www.reddit.com/r/rust/comments/1fb4qq2/any_rust_developer_currently_developing_for/
relation to wasmtime? · Issue #142 · wasmerio/wasmer - GitHub, accessed June 28, 2025, https://github.com/wasmerio/wasmer/issues/142
wasmtime - Rust - Docs.rs, accessed June 28, 2025, https://docs.rs/wasmtime
Rust - Wasmtime, accessed June 28, 2025, https://docs.wasmtime.dev/lang-rust.html
Rust & Wasm: Embed Wasmtime in your Rust app | by Nikhil Gupta - Medium, accessed June 28, 2025, https://guptanikhil.medium.com/rust-wasm-embed-wasmtime-in-your-rust-app-51c4da4231f6
WebAssembly runtimes compared - LogRocket Blog, accessed June 28, 2025, https://blog.logrocket.com/webassembly-runtimes-compared/
wasmerio/wasmer-rust-example: Example of WebAssembly ... - GitHub, accessed June 28, 2025, https://github.com/wasmerio/wasmer-rust-example
Explain difference vs. wasmtime · Issue #2259 · wasmerio/wasmer - GitHub, accessed June 28, 2025, https://github.com/wasmerio/wasmer/issues/2259
WasmEdge Rust SDK | WasmEdge Developer Guides, accessed June 28, 2025, https://wasmedge.org/docs/embed/rust/intro/
Plugins with Rust and WASI Preview 2, accessed June 28, 2025, https://benw.is/posts/plugins-with-rust-and-wasi
WASI and the WebAssembly Component Model: Current Status - eunomia, accessed June 28, 2025, https://eunomia.dev/blog/2025/02/16/wasi-and-the-webassembly-component-model-current-status/
Rust WASM Plugins Example - Reddit, accessed June 28, 2025, https://www.reddit.com/r/rust/comments/1hvaz5f/rust_wasm_plugins_example/
Revolutionizing Distributed Software with WebAssembly Component Model - Medium, accessed June 28, 2025, https://medium.com/wasi-articles/revolutionizing-distributed-software-with-webassembly-component-model-412574d2881a
Efficient Data Exchange between WebAssembly Modules - MDPI, accessed June 28, 2025, https://www.mdpi.com/1999-5903/16/9/341
(PDF) Efficient Data Exchange between WebAssembly Modules - ResearchGate, accessed June 28, 2025, https://www.researchgate.net/publication/384194651_Efficient_Data_Exchange_between_WebAssembly_Modules
I was understanding WASM all wrong! | by Yuji Isobe - Medium, accessed June 28, 2025, https://medium.com/@yujiisobe/i-was-understanding-wasm-all-wrong-e4bcab8d077c
wasm-bindgen benchmarks - Rust and WebAssembly, accessed June 28, 2025, https://rustwasm.github.io/wasm-bindgen/benchmarks/
Why is webAssembly function almost 300 time slower than same JS function, accessed June 28, 2025, https://stackoverflow.com/questions/48173979/why-is-webassembly-function-almost-300-time-slower-than-same-js-function
Benchmarking Data Serialization: JSON vs. Protobuf vs. Flatbuffers ..., accessed June 28, 2025, https://medium.com/@harshiljani2002/benchmarking-data-serialization-json-vs-protobuf-vs-flatbuffers-3218eecdba77
FlatBuffers vs. Protobuf - serialization - Stack Overflow, accessed June 28, 2025, https://stackoverflow.com/questions/54478659/flatbuffers-vs-protobuf
rkyv is faster than {bincode, capnp, cbor, flatbuffers, postcard, prost, serde_json} : r/rust - Reddit, accessed June 28, 2025, https://www.reddit.com/r/rust/comments/m2yxb1/rkyv_is_faster_than_bincode_capnp_cbor/
A Study on using a Rust-based dynamic Module system in ..., accessed June 28, 2025, https://zuinnote.eu/blog/?p=2037
Using WebAssembly for Extension Development - Visual Studio Code, accessed June 28, 2025, https://code.visualstudio.com/blogs/2024/05/08/wasm
What is the WebAssembly Component Model? - F5 Networks, accessed June 28, 2025, https://www.f5.com/company/blog/what-is-the-webassembly-component-model
The WebAssembly Component Model - Fermyon, accessed June 28, 2025, https://www.fermyon.com/blog/webassembly-component-model
Inside the WebAssembly Component Model | by Enrico Piovesan | WASM Radar - Medium, accessed June 28, 2025, https://medium.com/wasm-radar/inside-the-webassembly-component-model-5b5ef3c423f9
Efficient memory passing between WASM and host · Issue #314 · WebAssembly/component-model - GitHub, accessed June 28, 2025, https://github.com/WebAssembly/component-model/issues/314
Zero Trust Distributed Computing with WebAssembly and wasmCloud, accessed June 28, 2025, https://wasmcloud.com/blog/zero-trust-security/
What is WASI? - Fastly, accessed June 28, 2025, https://www.fastly.com/learning/serverless/what-is-wasi
WASI: secure capability based networking - JDriven Blog, accessed June 28, 2025, https://jdriven.com/blog/2022/08/WASI-capability-based-networking
WASI's Capability-based Security Model - Yuki Nakata, accessed June 28, 2025, https://www.chikuwa.it/blog/2023/capability/
WASI Introduction - Wasm By Example, accessed June 28, 2025, https://wasmbyexample.dev/example-redirect?exampleName=wasi-introduction&programmingLanguage=all
component-model/design/high-level/UseCases.md at main - GitHub, accessed June 28, 2025, https://github.com/WebAssembly/component-model/blob/main/design/high-level/UseCases.md
Wasmati: An Efficient Static Vulnerability Scanner for WebAssembly - ResearchGate, accessed June 28, 2025, https://www.researchgate.net/publication/360215840_Wasmati_An_Efficient_Static_Vulnerability_Scanner_for_WebAssembly
Discovering Vulnerabilities in WebAssembly with Code Property Graphs - SysSec @ DPSS.INESC-ID, accessed June 28, 2025, https://syssec.dpss.inesc-id.pt/projects/tr-wasmati.pdf
Wasmati: An Efficient Static Vulnerability Scanner for WebAssembly - arXiv, accessed June 28, 2025, https://arxiv.org/pdf/2204.12575
Let's explore code signing with WebAssembly, accessed June 28, 2025, https://nishtahir.com/lets-explore-code-signing-with-webassembly/
WASM Module Signature/Verification · Issue #1185 · WebAssembly/design - GitHub, accessed June 28, 2025, https://github.com/WebAssembly/design/issues/1185
frehberg/wasm-sign: WebAssembly signing and verification tool - GitHub, accessed June 28, 2025, https://github.com/frehberg/wasm-sign
Securely distributing and signing WebAssembly modules using OCI and TUF | radu's blog, accessed June 28, 2025, https://radu-matei.com/blog/wasm-oci-tuf/
Stay secure | Chrome Extensions - Chrome for Developers, accessed June 28, 2025, https://developer.chrome.com/docs/extensions/develop/security-privacy/stay-secure
Gorgeous Shadow Locust - Missing Execution Timeout (WASM VM) · Issue #149 · sherlock-audit/2024-12-seda-protocol-judging - GitHub, accessed June 28, 2025, https://github.com/sherlock-audit/2024-12-seda-protocol-judging/issues/149
WebAssembly: The Promise and Perils of Native Code on the Web | by Shreyash Sharma, accessed June 28, 2025, https://medium.com/@shreyash.sharma78/webassembly-the-promise-and-perils-of-native-code-on-the-web-93eeebfc1e2
Gas Metering for Wasm Programs - Alexander Gryaznov, accessed June 28, 2025, https://agryaznov.com/posts/wasm-gas-metering/
Experiment with wasm-global based gas metering · Issue #4410 · near/nearcore - GitHub, accessed June 28, 2025, https://github.com/near/nearcore/issues/4410
Interrupting Wasm Execution - Wasmtime, accessed June 28, 2025, https://docs.wasmtime.dev/examples-interrupting-wasm.html
Quotas and limits | Service Extensions - Google Cloud, accessed June 28, 2025, https://cloud.google.com/service-extensions/docs/quotas
Memory limits in webassembly - browser - Stack Overflow, accessed June 28, 2025, https://stackoverflow.com/questions/40417774/memory-limits-in-webassembly
4. WebAssembly Memory - WebAssembly: The Definitive Guide [Book] - O'Reilly Media, accessed June 28, 2025, https://www.oreilly.com/library/view/webassembly-the-definitive/9781492089834/ch04.html
Performance in the spotlight: WebAssembly profiling for everyone - Dispatch, accessed June 28, 2025, https://dispatch.run/blog/performance-in-the-spotlight-webassembly-profiling-for-everyone
Wasmer 2.0, It's a big deal! · Blog, accessed June 28, 2025, https://wasmer.io/posts/wasmer-2_0
memory-control/proposals/memory-control/Overview.md at main · WebAssembly/memory-control - GitHub, accessed June 28, 2025, https://github.com/WebAssembly/memory-control/blob/main/proposals/memory-control/Overview.md
