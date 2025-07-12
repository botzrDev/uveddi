
A Technical Report on the Implementation of the UV-42 Abstract Syntax Tree Caching System


Executive Summary

This report provides a comprehensive technical guide for the design and implementation of the UV-42 Abstract Syntax Tree (AST) caching system, a foundational component for the Uveddi infrastructure sprint. The primary objectives of this system are to enable efficient parallel processing, substantially reduce memory pressure when analyzing large codebases, and achieve an overall analysis performance improvement exceeding 60%.
To meet these ambitious goals, this report recommends a hybrid architectural pattern that synthesizes best practices from industry-leading tools. The proposed architecture consists of three distinct layers: a content-addressable storage (CAS) layer for persistent, on-disk artifacts; a concurrent, in-memory indexing layer for fast lookups; and a lazy-access layer that leverages memory-mapped files to minimize memory consumption.
Key technology and strategy recommendations include:
Architectural Model: A hybrid of a Bazel-style Content-Addressable Store (CAS) for immutable, on-disk AST blobs and a Clang-style lazy-loading mechanism using memory-mapped files for cache reads. This combination ensures correctness, shareability, and extreme memory efficiency.
Cache Invalidation: The use of a high-speed, non-cryptographic hashing algorithm (xxHash) for file content hashing is recommended. This approach is demonstrably more robust and performant for file integrity verification than modification-time tracking.
Concurrency: The in-memory cache index should be implemented as a sharded Least Recently Used (LRU) cache. Each shard will be protected by a std::sync::RwLock to maximize throughput in the expected read-heavy workload. This design minimizes lock contention and scales effectively with multiple processing threads.
Serialization: The rkyv framework is the recommended choice for serializing ASTs. Its support for true zero-copy deserialization is the critical enabling technology for the memory-mapping strategy, allowing the system to access large ASTs without allocating proportional heap memory. This choice is backed by extensive performance benchmarks.
Observability: The system should be instrumented using the OpenTelemetry standard. This provides a vendor-agnostic framework for collecting high-fidelity, correlated telemetry (metrics, traces, logs), enabling deep performance analysis and debugging.
This report provides detailed analysis, data-driven comparisons, and actionable implementation plans for each component of the UV-42 system. Adherence to these recommendations will ensure the final implementation is performant, scalable, and robust, meeting all specified project requirements and performance targets.

I. Architectural Blueprints for High-Performance Caching

A successful caching system is not a monolithic block of logic but a well-architected set of layered components, each with a clear responsibility. The design of the UV-42 system can draw critical lessons from the architectures of mature, high-performance developer tools that have solved similar problems at scale.

1.1 Analysis of Industry Implementations: Lessons from the Titans

An examination of systems like the Clang compiler, the Bazel build system, and modern language servers reveals a set of recurring principles that form the foundation of a robust caching strategy.

1.1.1 Clang: The Gold Standard for Memory Efficiency

The Clang compiler's design philosophy is relentlessly driven by performance, with a specific focus on minimizing the cost of processing its precompiled artifacts, such as Precompiled Headers (PCH) and Modules.1 Its primary innovation is the concept of
lazy deserialization of AST files.
When Clang loads a serialized AST file, it does not immediately parse the entire file into memory. Instead, it reads only a small metadata block to establish the on-disk locations of important data structures like the type table and declaration table.1 The actual AST nodes—representing functions, variables, types, and other language constructs—are only deserialized from the file when they are first referenced by the code being compiled. This "on-demand" approach makes the performance cost of using a cached AST directly proportional to the amount of code actually
used from the cache, not the total size of the cached artifact.1
Clang's Modules feature is a direct evolution of this concept, generalizing the linear chain of PCH dependencies into a full directed acyclic graph (DAG).1 This more advanced system uses the same lazy-loading mechanics but adds sophisticated logic for managing complex dependencies, merging declarations from different modules, and controlling name visibility.1 For UV-42, the core takeaway from Clang is the immense memory savings and performance gain achieved by deferring the cost of deserialization until the last possible moment.

1.1.2 Bazel: The Paragon of Correctness and Distributed Caching

The Bazel build system prioritizes correctness and reproducibility, which is reflected in its caching architecture. The fundamental unit of work in Bazel is an action—a command with explicitly declared inputs, outputs, and environment settings.3 Caching is keyed by an
action key, a deterministic hash of this action metadata, ensuring that identical actions produce cache hits.3
Bazel's remote caching is architected as a two-tiered system 4:
An Action Cache (AC), which is a map from an action's hash to its result metadata (e.g., the hashes of its output files).
A Content-Addressable Store (CAS), which stores the actual output files (build artifacts, logs, etc.) as opaque blobs, addressed by the hash of their content.
This separation of action metadata from content storage is a powerful architectural pattern. It ensures correctness, prevents data duplication, and enables the efficient sharing of build artifacts across an entire development organization.4 Furthermore, Bazel employs multiple cache layers, including a sophisticated in-memory cache called
Skyframe that caches the build's action graph. The fact that this highly optimized in-memory cache is ephemeral and lost upon server restart underscores the critical importance of a robust, persistent on-disk or remote cache layer for overall performance.6

1.1.3 Language Servers (TypeScript/Deno): The Move Towards Layered Caching

Language Servers (LS) are built around the constant use of ASTs to provide features like autocomplete, diagnostics, and "go to definition." However, the Language Server Protocol (LSP) itself intentionally abstracts this away, forbidding the direct exposure of the AST to the client editor.8 The server is solely responsible for managing the ASTs internally.8
The evolution of Deno's language server provides a particularly relevant case study. Initial implementations often re-parsed source files multiple times for different kinds of requests, leading to significant performance bottlenecks.10 The Deno team's solution was to introduce a dedicated
cache layer between their Rust-based server logic and the TypeScript compiler (tsc) process.11 This layer caches file contents and module resolution information. By doing so, it dramatically reduced the amount of data that needed to be passed between the Rust and JavaScript runtimes and intelligently avoided re-processing remote dependencies (e.g., from
npm: or URLs) that were known to be unchanged.11 This journey highlights the value of treating the cache not as a simple data structure, but as an explicit architectural layer designed to mediate communication and eliminate redundant work.

1.2 Recommended Architectural Pattern for UV-42: A Hybrid Approach

The most advanced systems do not treat caching as a single, monolithic feature. Instead, they architect it as a layered system with a clear separation of concerns. Bazel separates the definition of an action from the storage of its output (AC vs. CAS). Clang separates the on-disk representation of an AST from its in-memory working set. Deno's LS evolution involved inserting a cache as a distinct layer to solve a specific inter-process communication bottleneck.
This pattern of layering and separation of concerns is the key to managing the complexity of the UV-42 requirements. A single, monolithic cache class attempting to handle on-disk persistence, memory mapping, concurrent access, and LRU eviction would be extraordinarily difficult to design, test, and maintain.
Therefore, the recommended architecture for UV-42 is a hybrid model that synthesizes the strengths of these industry-leading systems:
A Bazel-style Content-Addressable Store (CAS) for On-Disk Persistence: The foundation of the cache will be a CAS. When an AST is generated for a given source file, it will be serialized into a binary blob. This blob will be stored on disk in a file whose name is derived from the blob's content hash. This provides immutable, content-addressable storage, ensuring correctness and enabling future possibilities for distributed caching. The primary cache key for a source file will be a composite key, derived from its unique path and its own content hash.
A Clang-style Lazy Loading Mechanism for Cache Reads: The system will not naively load entire AST blobs from the CAS into application memory upon a cache hit. Instead, for large ASTs that exceed a configurable threshold, the system will use memory-mapped files to map the blob from the CAS directly into the process's virtual address space. This approach leverages the operating system's page cache to manage memory, drastically reducing the application's resident memory footprint for large codebases, directly addressing a core project requirement.
This hybrid model provides the correctness and immutability of Bazel's CAS while achieving the profound memory efficiency of Clang's lazy-loading philosophy, creating a robust and scalable foundation for the UV-42 system.

II. The Cache Core: Concurrent Data Structures and Eviction

The heart of the UV-42 system is its in-memory index, which is responsible for tracking cached items, managing their lifecycle, and providing fast, thread-safe access. The design of this core component must prioritize both algorithmic efficiency and high concurrency to meet the project's performance targets.

2.1 Implementing a High-Throughput LRU Cache: The Canonical Approach

To satisfy the requirement for an LRU (Least Recently Used) eviction policy, the implementation must be able to perform lookups, insertions, and deletions in constant time. A naive implementation using a simple list or array would result in linear-time complexity for at least one of these operations, which is unacceptable for a high-performance system.
The canonical and most efficient data structure for an LRU cache combines two distinct components 12:
A HashMap: This provides average-case $O(1)$ time complexity for lookups (get), insertions (put), and deletions (remove). The map's key is the cache key (e.g., the composite key of file path and content hash), and its value is a direct pointer or reference to a node within a linked list.
A Doubly Linked List: This list maintains the usage order of the items. The head of the list represents the most recently used item, while the tail represents the least recently used item. Because it is a doubly linked list, a node can be unlinked from any position and moved to the head in $O(1)$ time, provided one holds a pointer to it.
The operations work in concert:
On a get (cache hit): The HashMap is used to find the corresponding node in the linked list in $O(1)$. That node is then moved to the head of the list, also in $O(1)$, to mark it as the most recently used item.14
On a put (new item): A new node is created and added to the head of the list. A new entry is added to the HashMap pointing to this node. Both are $O(1)$ operations. If the cache is at capacity, the node at the tail of the list is removed, and its corresponding key is removed from the HashMap, which are also $O(1)$ operations.13
Using a VecDeque or a standard array is an inferior choice because the crucial operation of moving an arbitrary element from the middle of the sequence to the front (which occurs on every cache hit) is an $O(n)$ operation, as it requires shifting subsequent elements.15

2.2 Concurrency Model: Sharding to Conquer Contention

A single, global lock protecting the entire LRU cache (HashMap + list) would create a severe performance bottleneck in a multi-threaded analysis environment. Every cache access, whether read or write, would be serialized, defeating the purpose of parallel processing.
The industry-standard pattern to overcome this is sharding. This technique is employed by high-performance concurrent Rust crates like concurrent_lru and threadsafe-lru.16 The cache is partitioned into a fixed number of
N independent segments, or shards. Each shard is its own smaller, self-contained LRU cache, protected by its own dedicated lock.
When an operation for a given key arrives, a hash function is applied to the key to determine which shard it belongs to (e.g., shard_index = hash(key) % N). The thread then acquires the lock for only that specific shard to perform its operation. This design dramatically improves concurrency, as threads operating on keys that map to different shards do not contend for the same lock and can proceed in parallel. The number of shards should be configurable and is typically set to a value related to the number of available CPU cores.

2.3 A Multi-Tiered Locking Strategy

The choice of synchronization primitive is not a single, global decision. A sophisticated system like UV-42 should employ a multi-tiered locking strategy, using different primitives for different parts of the architecture based on their specific access patterns. The primary contention point will be access to the LRU shards, where the expected access pattern is read-heavy (get operations) and write-light (put operations). RwLock is explicitly designed for this read-heavy pattern, allowing multiple concurrent readers.18 Therefore, each shard of the LRU cache should be wrapped in an
Arc<RwLock<Shard>>.
However, other shared data may exist, such as global cache configuration (size limits, eviction policy settings) or metrics collectors. This data is read by every operation but written to very rarely. While an RwLock is viable, a lock-free structure like ArcSwap is an even better fit for this "read-often, write-rarely" pattern.20 It allows all threads to read the current configuration with zero locking overhead, while updates are handled via an atomic pointer swap. This leads to a more nuanced and performant design:
RwLock for the high-contention, read-mostly data plane (the cache shards) and ArcSwap for the low-contention, read-heavy control plane (global configuration), optimizing performance at each level.
The following table compares the recommended Rust concurrency primitives and their intended roles within the UV-42 architecture.

Primitive
Granularity
Recommended Use Case in UV-42
Contention Profile
Rationale
$Arc<RwLock<T>>$
Shard-level
Protecting each individual LRU cache shard.
High concurrency, read-mostly. Allows parallel get operations within a shard.
The primary access pattern for a cache is get (read). RwLock maximizes throughput for this common case by allowing multiple readers to access the data simultaneously.18
$Arc<Mutex<T>>$
Component-level
Guarding components with short, frequent, mixed read/write access patterns (e.g., a metrics aggregator before export).
Simpler and potentially lower overhead than RwLock when writes are common or critical sections are brief, as it avoids managing separate reader/writer states.19
To be used judiciously where the read-mostly assumption of RwLock does not hold or when simplicity is paramount.
$ArcSwap<T>$
Global-level
Holding global, rarely-changing configuration data (e.g., cache size limits, file paths).
Lock-free for readers; writers are lock-free but more expensive. Perfect for "read-often, write-rarely" data.
Provides wait-free reads for configuration data, eliminating any potential bottleneck on accessing global settings during high-throughput operations.20


III. The Storage Layer: Persistence, Memory-Mapping, and Serialization

The storage layer is responsible for the durable persistence of serialized ASTs. Its design must be efficient in terms of both disk I/O and memory utilization, especially when handling the large ASTs typical of complex codebases.

3.1 File-Based Persistence Strategy: A CAS Layout

The on-disk cache will be located in a configurable root directory. To ensure correctness and avoid filesystem bottlenecks, the storage layout will be modeled after a Content-Addressable Store (CAS), similar to those used by Bazel and Git.4
When a new AST is cached, it is first serialized into a byte blob. The SHA-256 or xxHash of this blob's content is computed. The blob is then stored in a file path derived from this hash. A common and effective strategy is to use the first few characters of the hash to create a subdirectory, which prevents any single directory from containing an excessive number of files and degrading filesystem performance.22 For example, a blob with hash
d4e8e16... would be stored at <cache_root>/cas/d4/e8e16....
The in-memory LRU index will map the primary cache key (e.g., a hash of file_path + file_content_hash) to this CAS content hash, providing the link needed to retrieve the correct AST blob from disk.

3.2 Leveraging Memory-Mapped Files for Large ASTs

A core requirement for UV-42 is to reduce memory pressure. For large ASTs, which can easily exceed 100 MB, naively reading the entire file from disk into a heap-allocated buffer is inefficient and defeats this goal. The solution is to use memory-mapped files.
Mechanism and Benefits: Memory-mapping (mmap) is an operating system feature that allows a file on disk to be mapped directly into an application's virtual address space.23 Instead of issuing explicit
read() system calls, the application can access the file's contents as if it were an in-memory array. The OS transparently handles loading the necessary data from disk into the system's page cache on-demand when a "page fault" occurs. This approach offers two profound benefits for UV-42:
Reduced Memory Pressure: An application can mmap a 100 MB AST file, but its actual resident memory usage will only increase by the size of the pages that are actively being accessed. This is the same principle that makes Clang's PCH and Module caches so memory-efficient.1
Elimination of Double-Buffering: Standard file I/O involves copying data first from the disk into a kernel-space buffer (the page cache) and then again from the kernel buffer into a user-space buffer. Memory-mapping eliminates the second copy, allowing the application to work directly with the kernel's page cache, reducing CPU overhead and improving I/O performance.23
Safety and Ergonomics in Rust: In Rust, mmap operations are unsafe because the underlying file could be modified by another process at any time, which would violate Rust's strict aliasing and immutability guarantees for slices like &[u8].25 This risk can be mitigated by:
Using read-only, private mappings (MAP_PRIVATE) to ensure changes are not propagated and to get copy-on-write behavior.
Ensuring the UV-42 cache process has exclusive ownership of its cache directory.
Building safe abstractions that encapsulate the unsafe code, using raw pointers internally while exposing a safe, ergonomic API to the rest of the application.25
The mmap-cache crate provides a relevant design pattern, using a highly compressed Finite State Transducer (fst::Map) as a memory-efficient on-disk index that maps keys to byte offsets within a separate, large memory-mapped values file.27 This demonstrates a viable strategy for managing the index itself if it were to grow too large for main memory.

3.3 Choosing a Serialization Format for ASTs

The selection of a serialization format is not an independent decision; it is the critical link that enables the memory-mapping strategy to fulfill the core requirement of reducing memory pressure. A format that requires a full deserialization step into a new heap allocation would completely negate the benefits of mmap.
The essential feature is zero-copy deserialization. This means being able to access the structured data within a serialized buffer without first parsing it into a separate, heap-allocated Rust object. When a mmap operation provides a &[u8] slice of the on-disk data, a zero-copy format allows the application to cast this slice into a traversable, structured view of the AST that simply borrows from the underlying mapped memory. This is the only way to truly realize the memory-saving benefits of mmap for this use case.
Frameworks like bincode and postcard, while extremely fast for full serialization and deserialization, do not support this. They would require allocating memory for the entire AST and copying the data out of the mapped buffer, thus failing to reduce memory pressure.28 Therefore, the choice must be between true zero-copy formats like Flatbuffers, Cap'n Proto, and
rkyv.
Based on comprehensive Rust serialization benchmarks, rkyv emerges as the superior choice. It consistently demonstrates performance on par with or exceeding other top-tier frameworks, produces a very compact binary representation, and provides an ergonomic, idiomatic Rust API for true zero-copy access.28
The following table summarizes the analysis, filtering the options through the lens of UV-42's specific architectural needs.

Framework
Serialize Perf.
Deserialize Perf.
Zero-Copy Access
Serialized Size
Rationale for UV-42
bincode
Excellent
Excellent
No
Very Compact
Not Recommended. Despite high performance for full (de)serialization, its lack of zero-copy access makes it unsuitable for the memory-mapped lazy loading strategy.28
postcard
Excellent
Excellent
No
Very Compact
Not Recommended. Similar to bincode, it is a fantastic general-purpose format but does not meet the specific zero-copy requirement for large ASTs.28
Flatbuffers
Good
Good
Yes
Larger
A viable option. It is a mature zero-copy format. However, its API can be less ergonomic ("inside-out" building), and benchmarks show it's often slower and larger than rkyv.28
rkyv
Excellent
Excellent
Yes
Very Compact
Highly Recommended. Offers the best of all worlds: performance competitive with bincode, a compact representation, and true zero-copy deserialization. Its API is more idiomatic for Rust. It is the ideal choice for the UV-42 architecture.28


IV. Invalidation, Integrity, and Multi-Language Integration

A correct and reliable cache must be able to accurately determine when its entries are stale and must integrate cleanly with the multi-language parsing pipeline it serves.

4.1 Robust File Change Detection: Content Hashing is King

The user requirements specify both "modification time tracking" and "hash-based invalidation." While modification timestamps (mtime) are simple to check, they are notoriously brittle and unsuitable for a robust caching system. File mtimes are not preserved by version control systems like Git during operations like checkout or clone, and they can be inconsistent across different filesystems or in CI/CD environments.31
Content-based hashing is the superior and correct approach, as demonstrated by build systems like Bazel and best practices for CI caching with tools like ESLint.3 This method involves computing a hash of the file's actual content. A cache entry is considered valid only if the source file's current content hash matches the hash stored with the cached data. This guarantees that the cache entry corresponds to the exact version of the file content, eliminating a whole class of subtle and hard-to-debug caching bugs.
For the hashing algorithm itself, a distinction must be made between cryptographic and non-cryptographic hashes 33:
Cryptographic Hashes (e.g., SHA-256): These are designed to be secure against malicious adversaries trying to create intentional collisions. This security comes at the cost of computational performance.34
Non-Cryptographic Hashes (e.g., xxHash): These are designed for maximum speed while maintaining excellent statistical properties, meaning the probability of accidental collisions is astronomically low. They are widely used for checksums and file integrity checks where the threat model is accidental data corruption, not a targeted attack.33
For the use case of file integrity checking within a build and analysis tool, the primary concern is speed and detecting accidental changes. Therefore, the extreme performance of xxHash, which can operate at memory-bus speeds, makes it the ideal choice over the slower SHA-256.35

4.2 Designing a Multi-Language Parser Pipeline

The UV-42 cache must be language-agnostic to support Rust, Python, and JavaScript. This is achieved by defining a clear contract between the analysis engine and the cache. The cache's responsibility is simple: it stores and retrieves opaque byte blobs ([u8]) associated with a primary key. It has no knowledge of the internal structure of these blobs.
The analysis pipeline will operate as follows:
The analysis engine identifies a source file to be processed (e.g., main.rs, utils.py).
It computes the file's content hash using xxHash.
It queries the UV-42 cache with a composite key, such as (file_path, content_hash).
On a Cache Miss: The engine invokes the appropriate language-specific parser (e.g., the syn crate for Rust, or a dedicated Python/JS parser). The parser produces its native AST data structure. This AST is then serialized into a byte blob using the chosen format (rkyv). This blob is finally passed to the cache's put() method to be stored.
On a Cache Hit: The cache returns the serialized rkyv byte blob. This blob can then be passed directly to the language-specific analysis tools. Crucially, because rkyv is a zero-copy format, these tools can access the AST data directly from the returned byte slice without needing to perform a full parse or allocate a new AST structure on the heap.
This design cleanly decouples the generic caching logic from the domain-specific parsing and analysis logic, creating a flexible and extensible system that fulfills the multi-language requirement.

V. Instrumenting for Insight: Observability and Metrics

To ensure the UV-42 cache meets its performance targets and remains healthy in production, it must be instrumented with comprehensive, high-quality telemetry. Simply collecting a few metrics is insufficient; the observability strategy must be designed to provide deep, actionable insights into the cache's behavior.

5.1 A Unified Observability Strategy with OpenTelemetry

The recommended framework for instrumentation is OpenTelemetry (OTel). OTel is a vendor-agnostic, open standard for generating and collecting telemetry data, including metrics, logs, and traces.36 By using the OTel SDK, the UV-42 system is decoupled from any specific monitoring backend (e.g., Prometheus, Datadog, New Relic), providing maximum flexibility for the Uveddi infrastructure.37
The most powerful feature of OTel is its ability to correlate signals. For example, an exemplar can link a spike in a cache latency metric directly to the specific traces of the slow requests that occurred at that time, dramatically reducing the time required for root cause analysis.38 The recommended architecture involves the UV-42 system using the OTel SDK to generate metrics, which are then pushed to a central
OpenTelemetry Collector. The Collector can then process, aggregate, and export this data to one or more backends, including a Prometheus instance for storage and querying.37

5.2 Designing for Actionable Insights

Simply collecting metrics is not enough. A flat metric like cache_hits is far less useful than a metric with attributes that can answer questions like "What is the hit rate for Python files specifically?" or "Is lookup latency higher for cache misses?". The instrumentation plan must go beyond just defining metric names; it must define a rich set of attributes (also known as labels or tags) that allow for deep, multi-dimensional analysis. This transforms the observability system from a simple monitoring tool into a powerful diagnostic and performance analysis platform.
The attributes should reflect the architecture of the cache and the context of its operations. Key attributes for UV-42 metrics should include:
language: To segment performance by language (rust, python, javascript).
operation: To distinguish between different cache actions (get, put, invalidate).
result: To differentiate outcomes (hit, miss, error).
shard_id: To identify performance hotspots or imbalances across cache shards.

5.3 Recommended OpenTelemetry Metrics for UV-42

The following table provides a concrete, actionable set of metrics for instrumenting the UV-42 cache, following OpenTelemetry semantic conventions and specifying the appropriate instrument type, unit, and key attributes for multi-dimensional analysis.

Metric Name (Semantic Convention)
OTel Instrument
Unit
Key Attributes
Purpose
uv42.cache.operation.duration
Histogram
ms
`operation=[get
put
uv42.cache.operations.total
Counter
{operation}
`operation=[get
put
uv42.cache.memory.usage
Asynchronous Gauge
By
shard_id
Tracks resident memory usage of the in-memory index, crucial for monitoring against the <500MB target.42
uv42.cache.items.total
Asynchronous Gauge
{item}
shard_id
Tracks the number of items in the cache. Useful for understanding cache fullness and eviction pressure.43
uv42.cache.evictions.total
Counter
{item}
shard_id
Tracks the number of items evicted due to the LRU policy. A high or rapidly increasing rate may indicate the cache is too small for the workload.43
uv42.cache.storage.size
Asynchronous Gauge
By
-
Tracks the total on-disk size of the CAS. Important for capacity planning.


VI. Validation Strategy: Benchmarking and Testing Plan

A rigorous benchmarking and testing plan is essential to validate that the UV-42 implementation meets its stringent performance and scalability targets. The plan must be transparent, reproducible, and designed to simulate realistic production workloads.

6.1 Methodology for Performance Validation

All benchmark tests, configurations, hardware specifications, and datasets must be version-controlled to ensure that results are reproducible and comparable across different builds and implementation phases.44 The testing environment should closely mimic the production environment in terms of CPU, memory, disk I/O characteristics, and network configuration to ensure the results are representative of real-world performance.44
For implementation, the criterion crate in Rust is recommended for micro-benchmarking specific, performance-critical functions (e.g., hashing algorithms, serialization/deserialization routines). For end-to-end system testing, a custom multi-threaded load generator should be developed to simulate the concurrent access patterns of the Uveddi analysis engine.

6.2 Workload Simulation

The benchmarks must avoid small or overly simplistic synthetic datasets, which often reside entirely in CPU caches and do not reflect real-world performance.44
Realistic Dataset: A large, real-world codebase, such as a clone of a major open-source project (e.g., rust-lang/rust, torvalds/linux, or tensorflow/tensorflow), should be used as the input source. This provides a natural and realistic distribution of file sizes, code complexity, and directory structures.
Mixed Read/Write Load: Benchmarks should not test get and put operations in isolation. The load generator must simulate a realistic analysis workflow, which will likely consist of a high percentage of read operations (checking for cached ASTs) and a smaller, but concurrent, percentage of write operations (parsing and storing new ASTs after a file change).44
Cache State: Tests must be conducted under various cache states:
Cold Start: Measuring performance with an empty cache to quantify the worst-case scenario.
Warm Cache: Measuring performance after a "warm-up" phase where the cache is populated with frequently accessed items.45 This reflects steady-state performance.
Full Cache: Measuring performance when the cache is at capacity to test the overhead of the eviction logic.
Concurrency Simulation: The load generator must be multi-threaded to test the cache's performance under high contention. Key metrics like throughput and latency should be measured as the number of concurrent clients increases (e.g., 1, 4, 8, 16, 32, 64 threads) to verify that the sharded architecture is scaling as expected.

6.3 Failure Mode and Scalability Testing

Beyond raw performance, the cache's robustness and correctness must be validated.
Cache Corruption Test: A test should be designed to intentionally corrupt or delete a file within the on-disk CAS. The system's behavior should be verified: it must not crash or return corrupted data. The expected behavior is for the system to treat the corrupted entry as a cache miss and regenerate it.
Eviction Policy Verification: A specific test sequence should be designed to validate that the LRU eviction policy is functioning correctly. This can be done by accessing a known set of items in a predictable order and then inserting a new item that exceeds capacity, verifying that the least recently used item is the one that gets evicted.
Scalability Testing: A series of benchmarks should be run with progressively larger datasets (e.g., 1,000 files, 10,000 files, 100,000 files, 1,000,000 files) to measure how memory usage and disk space scale. The results should be plotted to ensure scaling is predictable and to identify any potential performance cliffs or resource leaks.

VII. Conclusion and Phased Implementation Roadmap

The UV-42 AST caching system is a critical infrastructure project with ambitious performance goals. The analysis presented in this report indicates that these goals are achievable through a principled, layered architecture that synthesizes best-in-class solutions from across the industry. The recommended design—a sharded, RwLock-based LRU cache indexing into a content-addressable store, with rkyv-serialized ASTs accessed via memory-mapping—directly addresses every key requirement, from concurrency and memory pressure to multi-language support and observability.
The key technological choices are mutually reinforcing: the rkyv serialization format enables the memory-mapping strategy, which in turn satisfies the requirement to handle large ASTs efficiently. The sharded concurrency model allows the system to leverage modern multi-core processors, and the robust content-hashing approach ensures correctness. Finally, instrumenting with OpenTelemetry provides the necessary insight to operate and optimize the system in a complex production environment.
To de-risk the project and ensure incremental progress, the following phased implementation roadmap is proposed:
Phase 1: Core Functionality and Single-Language Prototype
Implement the fundamental, single-threaded LRU cache data structure (HashMap + Doubly Linked List).
Develop the on-disk Content-Addressable Store (CAS) layout.
Integrate the rkyv serialization framework and the xxHash algorithm for file hashing.
Build a complete proof-of-concept pipeline for a single language (e.g., Rust), demonstrating cache miss (parse, serialize, store) and cache hit (load, access) scenarios.
Phase 2: Concurrency and Performance Benchmarking
Introduce the sharding architecture, wrapping each shard in an Arc<RwLock<T>> to enable concurrent access.
Develop the comprehensive benchmarking suite and load generator as described in Section VI.
Execute benchmarks to validate performance against single-threaded targets and ensure the concurrent implementation scales correctly.
Phase 3: Scalability and Observability
Implement the memory-mapping logic for accessing large ASTs from the CAS, triggered by a configurable size threshold.
Instrument the entire system with the OpenTelemetry metrics defined in Section V.
Set up an OTel Collector and a monitoring backend (e.g., Prometheus/Grafana) to visualize and analyze cache performance.
Phase 4: Multi-Language Support and Production Integration
Develop the parser integration wrappers for Python and JavaScript, ensuring they adhere to the same serialization contract.
Conduct final end-to-end testing with mixed-language codebases.
Integrate the completed UV-42 cache into the broader Uveddi analysis pipeline for deployment.
Works cited
Precompiled Header and Modules Internals — Clang 21.0.0git ..., accessed July 12, 2025, https://clang.llvm.org/docs/PCHInternals.html
Modules — Clang 21.0.0git documentation, accessed July 12, 2025, https://clang.llvm.org/docs/Modules.html
Bazel Glossary, accessed July 12, 2025, https://bazel.build/reference/glossary
Remote Caching | Bazel, accessed July 12, 2025, https://bazel.build/remote/caching
Bazel's Remote Caching and Remote Execution Explained - BuildBuddy, accessed July 12, 2025, https://www.buildbuddy.io/blog/bazels-remote-caching-and-remote-execution-explained/
The Many Caches of Bazel - EngFlow Blog, accessed July 12, 2025, https://blog.engflow.com/2024/05/13/the-many-caches-of-bazel/
Estimating the effort to build a Bazel CI/CD - Aspect Blog, accessed July 12, 2025, https://blog.aspect.build/estimating-bazel-cicd
How do I parse TS to symbols using a Language Server Protocol? - Stack Overflow, accessed July 12, 2025, https://stackoverflow.com/questions/56961479/how-do-i-parse-ts-to-symbols-using-a-language-server-protocol
How does a language server for a text editor manage looking up function definitions and such so quickly? : r/ProgrammingLanguages - Reddit, accessed July 12, 2025, https://www.reddit.com/r/ProgrammingLanguages/comments/zmky16/how_does_a_language_server_for_a_text_editor/
lsp: replace code lens navigation tree with swc "collector" · Issue #11032 · denoland/deno, accessed July 12, 2025, https://github.com/denoland/deno/issues/11032
How We Made the Deno Language Server Ten Times Faster | Deno, accessed July 12, 2025, https://deno.com/blog/optimizing-our-lsp
Cache It or Lose It: The Power of LRU | by Abhijith M S - Medium, accessed July 12, 2025, https://medium.com/@ams_132/cache-it-or-lose-it-the-power-of-lru-b669b43e527b
LRU Cache Implementation Guide: How It Works, FAQs, and Applications - Final Round AI, accessed July 12, 2025, https://www.finalroundai.com/blog/lru-cache-implementation-guide-how-it-works-faqs-and-applications
LRU Cache Implementation using Doubly Linked List - GeeksforGeeks, accessed July 12, 2025, https://www.geeksforgeeks.org/dsa/lru-cache-implementation-using-double-linked-lists/
Why Use A Doubly Linked List and HashMap for a LRU Cache Instead of a Deque?, accessed July 12, 2025, https://stackoverflow.com/questions/54730706/why-use-a-doubly-linked-list-and-hashmap-for-a-lru-cache-instead-of-a-deque
concurrent_lru - crates.io: Rust Package Registry, accessed July 12, 2025, https://crates.io/crates/concurrent_lru
threadsafe-lru - crates.io: Rust Package Registry, accessed July 12, 2025, https://crates.io/crates/threadsafe-lru
Understanding Mutex and RwLock in Rust | by loudsilence - Medium, accessed July 12, 2025, https://medium.com/@loudsilence/understanding-mutex-and-rwlock-in-rust-55974cc163c0
When or why should I use a Mutex over an RwLock? - Red And Green, accessed July 12, 2025, https://redandgreen.co.uk/mutex-v-rwlock/rust-programming/
arc_swap::docs::performance - Rust, accessed July 12, 2025, https://docs.rs/arc-swap/latest/arc_swap/docs/performance/index.html
Mastering Rust Arc and Mutex: A Comprehensive Guide to Safe Shared State in Concurrent Programming | by Syed Murtza | May, 2025 | Medium, accessed July 12, 2025, https://medium.com/@Murtza/mastering-rust-arc-and-mutex-a-comprehensive-guide-to-safe-shared-state-in-concurrent-programming-1913cd17e08d
Recommendations for cache-type database - help - Rust Users Forum, accessed July 12, 2025, https://users.rust-lang.org/t/recommendations-for-cache-type-database/90336
Understanding when and how to use Memory Mapped Files | by Abhijit Mondal, accessed July 12, 2025, https://mecha-mind.medium.com/understanding-when-and-how-to-use-memory-mapped-files-b94707df30e9
Pretokenized Headers (PTH) — Clang 8 documentation, accessed July 12, 2025, https://bcain-llvm.readthedocs.io/projects/clang/en/latest/PTHInternals/
Is there no safe way to use mmap in Rust?, accessed July 12, 2025, https://users.rust-lang.org/t/is-there-no-safe-way-to-use-mmap-in-rust/70338
Memory mapped files in Rust : r/rust - Reddit, accessed July 12, 2025, https://www.reddit.com/r/rust/comments/sn4zl4/memory_mapped_files_in_rust/
mmap_cache - Rust - Docs.rs, accessed July 12, 2025, https://docs.rs/mmap-cache
djkoloski/rust_serialization_benchmark: Benchmarks for ... - GitHub, accessed July 12, 2025, https://github.com/djkoloski/rust_serialization_benchmark
Benchmarking Data Serialization: JSON vs. Protobuf vs. Flatbuffers | by Harshil Jani | Jun, 2025 | Medium, accessed July 12, 2025, https://medium.com/@harshiljani2002/benchmarking-data-serialization-json-vs-protobuf-vs-flatbuffers-3218eecdba77
Performance Comparison of Messaging Protocols and Serialization Formats for Digital Twins in IoV, accessed July 12, 2025, https://dl.ifip.org/db/conf/networking/networking2020/1570620395.pdf
Eslint Cache in CI - Enoch, accessed July 12, 2025, https://enochchau.com/blog/2022/eslint-cache-ci/
Speeding up ESLint—Even on CI - Nicolas Charpentier, accessed July 12, 2025, https://www.charpeni.com/blog/speeding-up-eslint-even-on-ci
HMAC-SHA256 vs xxHash - A Comprehensive Comparison - MojoAuth, accessed July 12, 2025, https://mojoauth.com/compare-hashing-algorithms/hmac-sha256-vs-xxhash/
SHA-256 vs xxHash - SSOJet, accessed July 12, 2025, https://ssojet.com/compare-hashing-algorithms/sha-256-vs-xxhash/
Anybody know of a resource explaining the differences between these checksum algorithms at a fairly basic level (pros, cons, etc)? : r/DataHoarder - Reddit, accessed July 12, 2025, https://www.reddit.com/r/DataHoarder/comments/19c1i83/anybody_know_of_a_resource_explaining_the/
Unlocking Observability with OpenTelemetry: Seamless Integration with Your Favorite Tools, accessed July 12, 2025, https://scalardynamic.com/resources/articles/14-opentelemetry-integration-observability-tools
A Deep Dive into OpenTelemetry and Prometheus Metrics | Better Stack Community, accessed July 12, 2025, https://betterstack.com/community/guides/observability/opentelemetry-metrics-vs-prometheus-metrics/
OpenTelemetry Metrics 101 - New Relic, accessed July 12, 2025, https://newrelic.com/fr/blog/best-practices/opentelemetry-metrics
New to OpenTelemetry Metrics? Start Here | Better Stack Community, accessed July 12, 2025, https://betterstack.com/community/guides/observability/opentelemetry-metrics/
Prometheus vs. OpenTelemetry Metrics: A Complete Guide | TigerData - TimescaleDB, accessed July 12, 2025, https://www.tigerdata.com/blog/prometheus-vs-opentelemetry-metrics-a-complete-guide
OpenTelemetry Metrics [with examples] - Uptrace, accessed July 12, 2025, https://uptrace.dev/opentelemetry/metrics
Metrics - OpenTelemetry, accessed July 12, 2025, https://opentelemetry.io/docs/concepts/signals/metrics/
Observability | Best Practices for Using Prometheus to Monitor Memcached - Alibaba Cloud, accessed July 12, 2025, https://www.alibabacloud.com/blog/observability-%7C-best-practices-for-using-prometheus-to-monitor-memcached_600895
Best practices for database benchmarking - Aerospike, accessed July 12, 2025, https://aerospike.com/blog/best-practices-for-database-benchmarking/
Caching Best Practices | Amazon Web Services, accessed July 12, 2025, https://aws.amazon.com/caching/best-practices/
