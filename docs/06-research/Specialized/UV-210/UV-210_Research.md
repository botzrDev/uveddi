
UV-210: A High-Performance Memory Architecture for the Uveddi Static Analysis Engine


Executive Summary

This document presents a comprehensive technical architecture and implementation plan to address critical memory performance issues within the Uveddi static analysis tool. Current analysis of large codebases reveals severe memory pressure exceeding 16GB, high allocation overhead, and significant memory fragmentation, which collectively degrade performance and limit scalability. The proposed architecture is designed to meet aggressive performance targets, including a reduction of peak memory usage to under 8GB, a 50%+ improvement in allocation speed, and a 70% reduction in allocation frequency.
The cornerstone of this proposal is a multi-layered memory management strategy that moves beyond a reliance on the general-purpose system allocator. It introduces a tiered approach tailored to the specific lifetime and usage patterns of objects within the analysis engine:
A High-Performance Global Allocator: We recommend replacing the default system allocator with mimalloc to immediately improve throughput and reduce fragmentation in multi-threaded scenarios.
Concurrent Object Pools: For frequently reused, long-lived objects such as detector configurations, we will implement a sharded, thread-safe object pool. This strategy amortizes creation costs and drastically reduces allocation churn by recycling object instances.
Per-Analysis Arena Allocation: For the vast number of transient objects created during the analysis of a single file, we will employ arena allocation using the bumpalo crate. This offers near-zero-cost allocation and deallocation, fundamentally solving the problems of allocation overhead and fragmentation for the hottest code paths.
Furthermore, this report details advanced optimizations for the AST cache, advocating for a transition from bincode serialization to a zero-copy approach using memory-mapped files (memmap2) and the rkyv framework. This will transform the cache from a memory-saving utility into a primary performance accelerator by enabling instantaneous, allocation-free access to cached ASTs.
The architecture is designed for observability, with a detailed set of metrics for monitoring memory usage, and for flexibility, through a configurable allocation strategy framework. A phased, three-day implementation roadmap is provided, prioritizing foundational changes and data-driven refactoring of performance-critical components. This plan is supported by a comprehensive validation strategy using criterion for benchmarking and heaptrack for profiling to ensure all quantitative and qualitative goals are met.
Upon implementation, this memory architecture will not only resolve the immediate performance bottlenecks but also provide a robust and scalable foundation for the future evolution of the Uveddi static analysis engine, aligning it with the best practices of industry-leading compilers and developer tools.

Part I: Foundational Analysis and Industry Precedent

To design a robust memory architecture for Uveddi, it is imperative to first analyze how similar high-performance systems solve analogous problems. Compilers, build systems, and other developer tools operate on vast, complex data structures derived from source code, making them ideal models for Uveddi's own challenges. The consistent pattern across these systems is a deliberate move away from generic memory management in favor of specialized, context-aware strategies that exploit the predictable lifetimes of data within their operational domains.

Section 1: Memory Management Patterns in High-Performance Compilers and Tools

A detailed examination of best-in-class tools like the Rust compiler (rustc), Clang/LLVM, and the Bazel build system reveals a set of core principles that will form the foundation of our proposed architecture.

1.1 The Arena and Interning Model of rustc

The Rust compiler (rustc) is a masterclass in managing memory for a large, complex compilation pipeline. During compilation, it generates an enormous number of data structures, such as types, traits, and intermediate representations. If managed naively with a general-purpose allocator, the overhead would be prohibitive. Instead, rustc employs a sophisticated strategy centered on arenas and interning.1
The central data structure in the compiler is the "typing context," or tcx, which has a lifetime parameter 'tcx associated with it. This lifetime represents a single compilation session. All long-lived data structures created during that session are allocated within a global, context-specific arena tied to this 'tcx lifetime.1 This is a form of arena allocation where objects are "bump-allocated" from a large, pre-allocated memory region. The key advantage is that deallocation becomes trivial: when the compilation is finished, the entire arena is discarded at once, obviating the need to track and free millions of individual objects.2 This model perfectly matches the lifecycle of compiler-internal data, which is only needed for the duration of a single compilation.
Complementing arena allocation is interning. For types that are frequently duplicated, such as ty::TyKind (the internal representation of a type), rustc maintains a canonical set.1 When a new type is constructed, the compiler first checks if an identical type has already been interned. If so, it reuses the existing pointer; otherwise, it allocates a new one in the arena and stores it. This ensures that any two identical types are represented by the same pointer, enabling extremely fast equality checks (a simple pointer comparison) and drastically reducing memory duplication.1 This technique is also applied to other structures like generic arguments (
GenericArgs) and trait references (TraitRef).1
To manage these systems in a highly concurrent environment, rustc leverages thread-local storage to handle interning caches, which avoids the ergonomic and performance costs of pervasive locking around these frequently accessed data structures.1

1.2 AST Lifecycle Management in Clang/LLVM

The Clang C++ compiler, built on the LLVM framework, faces similar challenges. Its solution is architecturally convergent with rustc's, centered on the ASTContext class.4 This class acts as a central repository for all long-lived AST nodes, types, and declarations generated during the parsing and semantic analysis of a single translation unit (e.g., a C++ source file).
The ASTContext internally uses a llvm::BumpPtrAllocator, a fast arena allocator.4 To allocate AST nodes, Clang developers use a special C++ feature called "placement new" with the syntax
new (Context) IntegerLiteral(...). This syntax overrides the global new operator and directs the allocation to the BumpPtrAllocator managed by the provided ASTContext instance.5 This approach yields several benefits:
Performance: Allocations are extremely fast, often just a pointer increment and an alignment check, avoiding the overhead of a general-purpose malloc.
Locality: AST nodes for a given file are allocated contiguously in memory, improving cache performance during tree traversals.2
Simplified Lifetime Management: Developers do not need to manually delete AST nodes. The memory for all nodes is managed by the ASTContext, which frees the entire arena upon its destruction at the end of the translation unit's compilation.5
This pattern demonstrates that for tree-like structures such as an AST, where all nodes share the lifetime of the tree's root, arena allocation is the industry-standard, high-performance solution.3 LLVM's broader architecture reinforces this performance-first philosophy, even providing intrinsics for garbage collection integration (
@llvm.gcroot) for languages that require it, while maintaining an informal memory model that prioritizes optimization.9

1.3 Transitive Data Management in Build Systems: A Bazel Case Study

The Bazel build system provides another valuable perspective, particularly in managing data that flows transitively through a dependency graph. A naive approach of concatenating lists of files or flags from dependencies leads to quadratic ($O(N^2)$) complexity in both time and memory. Bazel solves this with a specialized data structure called a depset.12
A depset represents a collection of transitive information as a nested graph, enabling efficient sharing of common data. For example, if target C depends on B, and both B and D depend on A, the information from A is stored only once and referenced by both B and D within the depset structure. This is conceptually analogous to interning and avoids the redundant storage that would occur with simple list concatenation.12 The use of
depsets is mandatory in Bazel rules for all transitive data to maintain performance.
Furthermore, Bazel's design includes sophisticated mechanisms for managing its own memory usage. It provides flags like --discard_analysis_cache to trade incremental build speed for lower memory consumption and an experimental feature called Skyfocus that allows users to define a "working set" of files, enabling Bazel to discard irrelevant state and reduce its memory footprint.13 This demonstrates a mature understanding that memory usage in developer tools is not a fixed cost but a tunable parameter that can be adapted to different workloads and constraints. Bazel's memory profiler and guidance on using
ctx.actions.args() to defer the expansion of depsets further underscore the deep investment in memory efficiency required for a high-performance build system.12

1.4 Implications for Static Analysis Tooling

The analysis of these three industry-leading tools reveals a clear and consistent set of architectural principles that are directly applicable to Uveddi.
First, high-performance developer tools do not rely on general-purpose allocators for performance-critical paths. They universally adopt specialized memory management strategies. For Uveddi to achieve its performance goals, it must transition from its current reliance on standard Vec and HashMap heap allocations to a more sophisticated model. The "Core Infrastructure & Performance Foundation Sprint" is therefore not a minor refactoring but a fundamental architectural evolution to align Uveddi with the best practices of its peers.
Second, the "unit of work" is the natural boundary for memory lifetimes. In rustc, it is the compilation session ('tcx). In Clang, it is the translation unit (ASTContext). In games, it is the frame.15 For Uveddi, the analysis of a single source file is a perfect analog. All the temporary data generated during the analysis of one file—AST nodes, intermediate results, lists of issues—can be discarded once that analysis is complete. This insight reframes the problem from a global memory reduction challenge to a more tractable one of isolating and rapidly reclaiming the memory associated with these transient, per-file analysis tasks. A per-file or per-task arena is the logical and most effective architecture to achieve this.

Part II: Core Memory Optimization Architecture for Uveddi

Based on the foundational analysis, we propose a multi-layered memory architecture for Uveddi. This architecture is designed to address the distinct allocation patterns and object lifetimes observed within the application, moving from a single, general-purpose allocator to a tiered system of specialized allocators. This tiered strategy directly targets the identified bottlenecks of allocation overhead, memory fragmentation, and peak memory pressure.

Section 2: A Tiered Allocation Strategy: Global, Pooled, and Arena-Based

The proposed architecture consists of three tiers, each chosen to match a specific object lifetime profile within Uveddi. This provides a clear and prescriptive model for developers: the lifetime of an object dictates which allocator should be used.
Global Allocator: For objects with a lifetime tied to the application itself.
Object Pool: For objects with an inter-analysis lifetime, reused across the analysis of multiple files.
Arena Allocator: For objects with an intra-analysis lifetime, existing only for the duration of a single file's analysis.
This structure ensures that we apply the most efficient allocation strategy for each specific use case, maximizing performance and minimizing waste.

2.1 Foundational Performance: Selecting a High-Throughput Global Allocator

While specialized allocators will handle the bulk of performance-critical allocations, the underlying global allocator remains important for all other allocations and for providing the memory blocks that our specialized allocators will manage. The default system allocator (malloc on Linux, HeapAlloc on Windows) is a generalist, designed for a wide range of applications. For a highly concurrent, allocation-intensive program like Uveddi, it can become a bottleneck due to lock contention and susceptibility to memory fragmentation.17
Switching to a high-performance global allocator is a low-effort, high-impact foundational improvement. The two leading candidates in the Rust ecosystem are jemalloc and mimalloc.
jemalloc: Originally developed for FreeBSD, jemalloc is engineered to reduce fragmentation and enhance performance in multi-threaded applications. It achieves this through techniques like thread-specific caches to reduce lock contention and size-class-based management to improve memory utilization.17 The
tikv-jemallocator crate provides an easy integration path.18
mimalloc: A newer allocator from Microsoft Research, mimalloc is designed for exceptional performance and security. It often outperforms jemalloc in benchmarks, particularly in terms of raw allocation speed, and can have a lower memory overhead.18 Anecdotal evidence from the Rust community shows significant performance gains and reductions in memory-related errors (like OOM) after switching to
mimalloc.19
Recommendation: The Uveddi project should adopt mimalloc as its default global allocator. It offers state-of-the-art performance with a simple integration path. The mimalloc crate can be added as a dependency, and the allocator can be set globally with a few lines of code in main.rs.18 While
jemalloc is a strong alternative, especially for its advanced profiling capabilities, mimalloc's raw performance and lower overhead make it the preferred choice for an immediate, system-wide boost. This change is the base of our optimization pyramid; it improves the efficiency of every allocation that still reaches the system, but it does not reduce the number of such allocations.

2.2 Object Pooling for Frequently Reused, Long-Lived Structures

The Uveddi analysis engine contains objects that are expensive to construct but are needed repeatedly across the analysis of many different files. Examples include compiled regular expressions for detectors, shared configuration objects, or complex data structures used for inter-procedural analysis. In the current system, these objects are likely created and destroyed repeatedly, contributing to allocation churn and "GC pressure."
Object pooling is the ideal pattern to solve this problem. A pool manages a set of pre-initialized, reusable objects. Instead of creating a new object, a client requests one from the pool. When the client is finished, it returns the object to the pool for another client to use, rather than destroying it.20 This pattern directly addresses the "GC Pressure Reduction" goal by transforming expensive
alloc/dealloc cycles into cheap pool.get()/pool.put() operations.
These objects have an inter-analysis lifetime: they persist across the analysis of multiple files but may not be needed for the entire duration of the application. The object pool is therefore the perfect manager for this intermediate lifetime scope. The design of our pool will be thread-safe to support Uveddi's concurrent analysis engine, likely using a sharded, lock-minimizing approach to ensure high performance under contention.

2.3 Arena Allocation for Transient Analysis Data

The most significant source of allocation overhead and memory fragmentation in Uveddi is the creation of transient data during the analysis of a single file. This includes AST nodes, lists of discovered issues (Vec<ArchitecturalIssue>), temporary strings, and other intermediate data structures. All of this data shares a common, short lifetime: it is needed only for the duration of one file's analysis and can be discarded immediately afterward.
Arena allocation is the perfect solution for this workload.2 An arena, or "bump allocator," works by pre-allocating a large, contiguous block of memory. Subsequent allocations are satisfied by simply "bumping" a pointer forward within this block, an operation that is orders of magnitude faster than a traditional
malloc call.2 When the file analysis is complete, the entire arena is deallocated in a single, constant-time operation, completely eliminating the cost of individual deallocations and preventing fragmentation.3
This strategy directly attacks the "Allocation Overhead" and "Memory Fragmentation" bottlenecks identified in the project mandate and is the same pattern used by high-performance compilers like rustc and Clang for their ASTs.1 The lifecycle is clear: a new arena is created for each analysis task, used for all temporary allocations within that task, and destroyed when the task completes.
The choice of a specific arena crate is critical. The following table compares the leading candidates based on criteria essential to Uveddi's success.

Table: Comparative Analysis of Rust Arena Crates


Crate
Allocation Model
Drop Handling
Concurrency
Collection Support
API Ergonomics
Best Fit for Uveddi
bumpalo 18
Mixed-Type
No Drop by default. Requires bumpalo::boxed::Box wrapper for dropping.
!Sync. Designed for thread-local use.
Yes (bumpalo::collections::Vec, String, etc.).
Excellent. Requires passing &'a Bump reference. Minimal lifetime annotation burden.
Excellent. The performance-first approach (no Drop by default) is ideal for transient analysis data. The provided collections are essential for safety. Its thread-local nature fits a per-thread task model perfectly.
typed-arena 18
Single-Type
Runs Drop on all allocated objects.
!Sync. Thread-local.
No. Requires use of standard collections, which is unsafe if they allocate.
Simple for single types but restrictive. Requires a separate arena for each type.
Poor. The single-type limitation is unworkable for the heterogeneous data created during analysis. Running Drop for all objects negates much of the performance benefit of an arena.
oxc_allocator 24
Mixed-Type
No Drop.
!Sync. Thread-local. Provides a AllocatorPool for thread-safe reuse of allocators.
Yes (Box, Vec, String, HashMap).
Good. Inspired by rustc's own allocators. API is clear and focused on compiler use cases.
Very Good. A strong alternative to bumpalo. Its design is explicitly for compiler/linter use cases (it powers the Oxc linter). The AllocatorPool is a compelling feature for managing arenas across threads.

Based on this analysis, bumpalo is the recommended choice. Its combination of high performance, excellent ergonomics, and built-in support for arena-aware collections makes it the best fit for Uveddi's primary use case of per-file analysis. The lack of automatic Drop is a feature, not a bug, for this workload, as it maximizes performance. The need to use its specialized collections to avoid memory leaks is a critical implementation detail that will be strictly enforced.

Part III: Detailed Design and API Specification

This section translates the high-level architecture into concrete, implementable designs for the core memory components. It provides detailed API specifications, implementation blueprints, and code prototypes to guide the development process, ensuring the final system is robust, ergonomic, and safe.

Section 3: The UveddiObjectPool<T>: A Concurrent, Reusable Allocator

The object pool is designed for managing expensive-to-create objects that are reused across multiple analysis tasks. Its primary goal is to reduce allocation churn by recycling object instances, satisfying the "GC Pressure Reduction" target.

3.1 API Design and Smart Pointer Wrappers (Pooled<T>)

The safety and ergonomics of the object pool hinge on a custom smart pointer, Pooled<T>, which manages the lifecycle of a borrowed object. This wrapper provides a familiar Box-like interface while ensuring the object is returned to the pool upon being dropped.

Rust


use std::ops::{Deref, DerefMut};
use std::sync::Arc;

// The central pool structure. It will be shared across threads using an Arc.
pub struct UveddiObjectPool<T> {
    // Internal implementation details (e.g., sharded slab)
    //...
}

// The smart pointer that clients of the pool will interact with.
pub struct Pooled<T> {
    obj: T,
    pool: Arc<UveddiObjectPool<T>>, // A reference back to the pool it came from
}

impl<T: Poolable> UveddiObjectPool<T> {
    /// Creates a new pool with a specified maximum capacity.
    pub fn new(capacity: usize) -> Self {
        //...
    }

    /// Retrieves an object from the pool. Returns None if the pool is empty.
    pub fn get(&self) -> Option<Pooled<T>> {
        //...
    }

    /// Internal method to return an object to the pool.
    fn put(&self, obj: T) {
        //...
    }
}

// Trait for objects that can be stored in the pool.
// Requires a `reset` method to ensure objects are in a clean state when reused.
pub trait Poolable {
    fn reset(&mut self);
}

// Deref implementations for ergonomic access.
impl<T> Deref for Pooled<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.obj
    }
}

impl<T> DerefMut for Pooled<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.obj
    }
}

// The crucial Drop implementation that enables recycling.
impl<T: Poolable> Drop for Pooled<T> {
    fn drop(&mut self) {
        // Instead of freeing memory, the object is returned to the pool.
        // We need to take ownership of the object to pass it back.
        // This requires `unsafe` or a method like `std::mem::take` if T is Default.
        let obj = std::mem::take(&mut self.obj); // Assuming T: Default for simplicity here
        self.pool.put(obj);
    }
}


This smart pointer design is the key to making the object pool safe. It encapsulates the complex logic of memory recycling within a standard Rust interface. The client gets an object that feels like it has unique ownership, but its Drop implementation is customized to return it to the pool, preventing both memory leaks and use-after-free errors.20

3.2 Implementation Blueprint: A Sharded, Lock-Free Slab

A naive implementation using a single Arc<Mutex<Vec<T>>> would create a global point of contention, harming scalability in Uveddi's multi-threaded analysis engine.26 A much more performant approach is to use a
sharded slab.
The internal storage will be a concurrent hash map, such as dashmap::DashMap, where the key is a thread ID and the value is a small, thread-local object pool (e.g., a Vec<T>).
get() operation:
The calling thread gets its ID.
It looks up its dedicated shard (a small Vec) in the DashMap.
It attempts to pop an object from its local Vec. This is a fast, often uncontended operation.
If its local shard is empty, it can attempt to "steal" an object from another thread's shard. This balances the load and improves utilization.
If all shards are empty, it can fall back to creating a new object, or return None.
put() operation (from Pooled<T>::drop):
The dropping thread gets its ID.
It looks up its dedicated shard.
It pushes the returned object into its local Vec.
This sharded design, inspired by crates like sharded-slab 18, drastically reduces lock contention because threads primarily interact with their own local pool, only contending with other threads during the less frequent "stealing" operations.

3.3 Managing Ownership, Lifetimes, and the Drop Trait

The ownership model is explicit and safe 20:
Acquisition: The pool gives ownership of an object to the client, wrapped in the Pooled<T> smart pointer. The client now has full control over the object for the lifetime of the Pooled<T> handle.
Return: When the Pooled<T> handle is dropped, its Drop implementation takes ownership of the object back from the client and returns it to the pool.
A critical consideration is the state of returned objects. An object might be in an arbitrary state when returned. The Poolable trait addresses this by requiring a reset method. Before an object is returned to the available queue in the pool, its reset method must be called to return it to a pristine, default state, ready for the next client.

Section 4: Per-Analysis Arenas: A bumpalo-Based Approach

Arenas are the primary tool for eliminating allocation overhead for the massive number of transient objects created during file analysis. Our strategy is based on the bumpalo crate due to its high performance and ergonomic design.

4.1 Lifecycle Management: Tying Arenas to Analysis Scopes

The lifecycle of an arena will be tightly bound to the scope of a single analysis task, such as processing one file.
Creation: At the beginning of AnalysisEngine::process_file, a new bumpalo::Bump instance is created on the stack.
Usage: A mutable reference to this arena, &'a Bump, is passed down through the entire call stack of the analysis. All functions involved in parsing, traversing the AST, and generating issues will perform their temporary allocations within this arena.
Deallocation: When process_file returns, the Bump instance goes out of scope. Its Drop implementation frees the entire block of memory it managed in a single operation. All objects allocated within it are dropped simultaneously without their individual Drop implementations being called.2
This model ensures that memory is reclaimed as quickly as possible, preventing the accumulation of transient garbage that currently leads to the 16GB+ memory pressure.

4.2 Prototype: Integrating Arenas into the Analysis Pipeline

The integration requires refactoring function signatures to accept the arena reference and switching from standard collections to their arena-aware counterparts.

Rust


// Current state in src/analysis/engine.rs
pub struct AnalysisEngine { /*... */ }

impl AnalysisEngine {
    pub fn process_file(&self, path: &PathBuf) -> Vec<ArchitecturalIssue> {
        // AST parsing and analysis logic...
        let mut issues = Vec::new(); // Standard heap allocation
        //... detectors push issues into the Vec, causing reallocations
        issues
    }
}

// Proposed refactored state
use bumpalo::Bump;
use bumpalo::collections::Vec as BumpVec;

pub struct AnalysisEngine { /*... */ }

impl AnalysisEngine {
    pub fn process_file_with_arena<'a>(
        &self,
        path: &PathBuf,
        arena: &'a Bump,
    ) -> BumpVec<'a, ArchitecturalIssue> {
        // AST parsing and analysis logic...
        // All transient data is now allocated in the provided arena.
        let mut issues = BumpVec::new_in(arena); // Arena allocation, no heap call
        
        // Example of a detector call
        let detector_results = self.run_detector_x(..., arena);
        issues.extend(detector_results);

        issues
    }
}

// The main analysis loop would look like this:
// for file in files_to_analyze {
//     let arena = Bump::new();
//     let issues = engine.process_file_with_arena(&file, &arena);
//     // Process and store the issues...
//     // `arena` is dropped here, reclaiming all memory instantly.
// }



4.3 Mitigating Memory Leaks with Arena-Aware Collections

A significant risk with high-performance arenas like bumpalo is their handling of the Drop trait. By default, to maximize speed, bumpalo does not call the Drop implementation of the objects it contains.28 If a developer allocates a standard
std::vec::Vec or String inside the arena, the Vec object itself (the pointer, capacity, and length) will be in the arena, but the heap-allocated buffer it points to will be leaked when the arena is destroyed, as the Vec's Drop implementation is never run.
This is a critical safety and correctness issue. The solution is twofold:
Mandatory Use of Arena-Aware Collections: Developers must exclusively use the collection types provided by bumpalo (e.g., bumpalo::collections::Vec, bumpalo::collections::String) for all data allocated within an arena.30 These collections are specifically designed to allocate their backing storage within the same arena, not on the global heap.
Static Enforcement: To prevent accidental misuse, a custom linting rule should be developed and integrated into Uveddi's CI pipeline. This lint would flag any use of std::vec::Vec, std::string::String, std::boxed::Box, etc., within any function that has a &'a Bump parameter in its signature. This programmatic enforcement turns a potential runtime memory leak into a compile-time error, upholding Rust's safety principles.
For the rare cases where an object allocated in the arena must have its Drop logic executed (e.g., it holds a file handle), bumpalo::boxed::Box<'a, T> can be used. This wrapper ensures that T's Drop implementation is called when the wrapper itself is dropped, at the cost of some additional tracking overhead within the arena.29 This should be treated as an exception, not the rule.

Part IV: Advanced Optimizations and System Integration

With the core memory architecture defined, this section details how it integrates with other Uveddi systems and how it can be extended with advanced techniques to maximize performance and observability. These strategies focus on optimizing I/O-bound operations and providing the necessary tools for runtime configuration and monitoring.

Section 5: High-Throughput Caching via Memory-Mapping and Zero-Copy Deserialization

The existing AST cache is a prime candidate for a transformative optimization. While it currently uses memory-mapped files, its reliance on bincode for serialization fundamentally limits its performance. The process of deserializing data from a cache file into new heap allocations negates the primary benefits of memory mapping. The proposed solution is to adopt a true zero-copy approach. This will not only reduce memory usage but also dramatically accelerate cache hits, making incremental analysis runs nearly instantaneous. This shift changes the cache's role from a simple memory-saving device to a core performance accelerator.

5.1 A Cross-Platform Memory-Mapping Strategy with memmap2

The memmap2 crate is the modern standard for memory-mapped I/O in Rust, offering a safe, cross-platform API.31 It will replace any direct or older
mmap implementations in the AST cache.
The key benefit of mmap is that it eliminates the boundary between kernel and user space for file I/O. Instead of issuing read() syscalls to copy data from the kernel's page cache into a user-space buffer, mmap directly maps the file's pages into the process's virtual address space.32 The operating system's virtual memory manager handles loading pages from disk into physical RAM on-demand via page faults. This provides two major advantages:
Reduced Syscall Overhead: No explicit read() calls are needed to access file data.
Elimination of Buffer Copies: Data is not copied from a kernel buffer to a user buffer, reducing CPU cycles and memory bandwidth consumption.33
While mmap performance can vary between platforms like Windows and Unix-like systems, for the primary use case of a read-only cache, the behavior is consistently excellent.33 The
memmap2 crate abstracts away these platform differences, providing a unified interface.31

5.2 Achieving Zero-Cost Access with rkyv

The true revolution in cache performance comes from pairing mmap with a zero-copy deserialization framework. The current bincode serializer requires a full deserialization step, which involves parsing the byte stream and allocating new objects on the heap. This is slow and memory-intensive.
Recommendation: The Uveddi AST cache must migrate from bincode to rkyv. rkyv is a framework designed specifically for zero-copy deserialization.36 It works by ensuring that the serialized, on-disk representation of a data structure is byte-for-byte identical to its in-memory representation.39
The workflow becomes:
Cache Write: An AST is serialized using rkyv and written to a file.
Cache Read: The file is memory-mapped using memmap2.
Access: The program performs a single, unsafe pointer cast to interpret the start of the memory-mapped region as a reference to the archived root type (e.g., &ArchivedAST).
From that point on, the entire AST can be traversed in-place, directly from the memory-mapped file buffer, with zero allocations and zero parsing overhead.39 A cache hit becomes as cheap as an
mmap syscall and a pointer cast.
Safety and Validation: Accessing rkyv data without validation is unsafe because a corrupted or malicious file could lead to invalid memory access. rkyv provides a validation layer via the bytecheck crate, which traverses the data and verifies all pointers and offsets are valid.41 For a local cache where the files are generated by Uveddi itself, these files can be considered trusted. Therefore, we can use the
unsafe access path for maximum performance. As a defense-in-depth measure, a checksum or magic number can be stored in the file header to guard against simple corruption.40

Section 6: A Framework for Configurable and Adaptive Allocation

To meet the qualitative goal of flexibility, the memory architecture must be tunable for different workloads and environments. We propose a two-pronged approach: simple, static configuration for predictable tuning, and a more advanced (but speculative) mechanism for runtime adaptation.

6.1 Static Configuration via Workload Profiles

The AllocationStrategy enum defined in the prompt will be implemented and exposed in Uveddi's main configuration file. This allows users to tune memory pool behavior based on their typical project size.

Rust


// In Uveddi's configuration module
pub enum AllocationStrategy {
    FixedSize(usize),
    GrowthBased { initial: usize, growth_factor: f64 },
    AdaptiveBased { target_memory: usize }, // Initially a placeholder for future work
}

// Example configuration in a TOML file
// [memory.pools.issue_pool]
// strategy = { type = "GrowthBased", initial = 1024, growth_factor = 2.0 }


We will define several top-level workload profiles (e.g., small_project, large_monorepo) that pre-configure these strategies to sensible defaults. This provides a simple knob for users while allowing power users to fine-tune individual pools.43

6.2 A Prototype for Runtime Memory Pressure Detection

Runtime adaptive memory management is a complex feature that should be approached with caution, as it can introduce its own performance overhead and unpredictable behavior. A simple feedback loop is plausible but should be considered a future enhancement rather than a core requirement for the initial rollout.
A potential implementation could leverage the memuse crate, which provides traits for querying the heap-allocated size of data structures at runtime.44
Prototype Feedback Loop:
A background thread periodically calls memuse::dynamic_usage() on the primary AST cache data structure.
If the reported memory usage exceeds a configured target_memory threshold, the system is considered under "memory pressure."
In response, the system could:
Trigger a more aggressive eviction policy in the LRU cache, reducing its target size.
Signal object pools to shrink their capacity, freeing memory back to the global allocator.
It is critical to note that Rust does not provide a standard API for detecting system-wide memory pressure.45 This approach would be a heuristic based on Uveddi's own internal state. The primary recommendation is to rely on static configuration and robust observability first, and only implement adaptive logic if profiling demonstrates a clear need that cannot be solved with static tuning.

Table: Proposed Memory Observability Metrics

To enable effective monitoring, debugging, and tuning, the new memory system must expose a rich set of metrics. These should be exported in a standard format (e.g., Prometheus) for integration with monitoring dashboards.
Metric Name
Metric Type
Target Component
Description
Implementation Source
uveddi_allocator_global_heap_used_bytes
Gauge
Global Allocator
Current heap memory used by the application post-GC.
jemalloc_stats or mimalloc stats API
uveddi_allocator_global_heap_active_bytes
Gauge
Global Allocator
Total bytes in active regions managed by the allocator.
jemalloc_stats or mimalloc stats API
uveddi_pool_objects_total{pool="<name>"}
Gauge
Object Pool
Total number of objects (both in-use and available) managed by the specified pool.
pool.capacity()
uveddi_pool_objects_in_use{pool="<name>"}
Gauge
Object Pool
Number of objects currently checked out from the specified pool.
pool.in_use()
uveddi_pool_gets_total{pool="<name>"}
Counter
Object Pool
Total number of successful get() operations from the pool.
Atomic counter inside get()
uveddi_pool_misses_total{pool="<name>"}
Counter
Object Pool
Total number of get() operations that returned None (pool was empty).
Atomic counter inside get()
uveddi_arena_allocations_total
Counter
Arena Allocator
Total number of allocations made across all arenas.
Atomic counter in arena alloc
uveddi_arena_bytes_allocated_total
Counter
Arena Allocator
Total number of bytes allocated across all arenas.
Atomic counter in arena alloc
uveddi_ast_cache_mmaps_bytes
Gauge
AST Cache
Total size of all memory-mapped cache files.
Tracked by cache manager
uveddi_ast_cache_hits_total
Counter
AST Cache
Total number of cache hits.
Atomic counter in cache get
uveddi_ast_cache_misses_total
Counter
AST Cache
Total number of cache misses.
Atomic counter in cache get

These metrics provide a comprehensive view of the memory system's health and performance, directly supporting the "Observability" goal.14

Section 7: Integration Architecture and Migration Plan

Integrating this new architecture requires careful refactoring of existing systems.

7.1 Coordinating with the AST Cache (UV-42)

The AST cache is a primary beneficiary and partner of the new memory system.
Producer/Consumer: The cache becomes the primary producer of rkyv-archived, memory-mapped ASTs.
Memory Accounting: Its memory_usage metric must be updated to track the size of the on-disk mmap files, not heap allocations.
Eviction Policy: The LRU eviction policy will now trigger munmap and file deletion instead of dropping heap-allocated objects.
Deserialization Context: If a component requires a fully-owned, mutable AST (rather than the read-only archived view), the deserialization from the rkyv format must occur into the per-file analysis arena. This correctly ties the lifetime of the deserialized AST to the analysis task that requested it.

7.2 Refactoring the Analysis Engine

This is the most significant integration effort. The migration must be systematic:
Prioritize with Profiling: Use heaptrack on the current codebase to identify the top 5-10 detectors responsible for the highest allocation counts.
Phased Refactoring: Begin by refactoring these hotspots. Modify their function signatures to accept the &'a Bump arena reference.
Collection Conversion: Systematically replace all instances of Vec, String, etc., within these functions with their bumpalo::collections equivalents.
Enforce with Lints: Immediately introduce the custom static analysis lint to prevent new code from accidentally using standard collections with arenas.

7.3 Host-Plugin Memory Coordination (WASM)

The WASM plugin system introduces a memory boundary. WASM modules operate in their own sandboxed linear memory and cannot directly access the host's memory pools or arenas.46 Direct memory sharing is complex and requires enabling experimental features like WASM threads and shared memory.47
This boundary creates a performance "impedance mismatch." Data managed efficiently on the host must be copied to be used by a plugin. The recommended integration strategy is therefore pragmatic:
Treat the Boundary as a Serialization Point: Data structures (like specific AST subtrees) needed by a plugin should be allocated in the host's per-file arena, serialized (e.g., using bincode or rkyv), and then the resulting byte buffer is copied into the WASM module's linear memory.
Maintain WASM Memory Limits: The plugin's internal memory usage remains constrained by its existing limits. This maintains security and resource isolation at the cost of a copy operation.
For future scalability, if certain plugins become performance-critical bottlenecks due to this copy overhead, a "native plugin" architecture (e.g., using dynamically loaded libraries via libloading 48) could be considered. This would allow trusted, high-performance plugins to participate directly in the host's memory ecosystem, but is outside the scope of this initial implementation.

Part V: Validation, Benchmarking, and Implementation Roadmap

A robust validation strategy is essential to prove that the new memory architecture meets its aggressive performance targets. This section outlines a comprehensive plan for benchmarking, profiling, and a phased implementation to ensure a successful rollout within the estimated 3-day effort.

Section 8: A Comprehensive Performance Validation Strategy

Our validation strategy combines micro-benchmarks to measure the performance of individual components, macro-benchmarks to measure end-to-end system performance, and detailed profiling to understand allocation patterns and memory usage.

8.1 Benchmarking Suite Design with criterion

We will use the criterion crate, the de facto standard for statistical benchmarking in Rust, to create a suite of performance tests.49 These benchmarks will be added to the Uveddi repository to prevent future performance regressions.
Micro-benchmarks: These will isolate and measure the performance of the new allocators against the baseline.
bench_pool_allocation_vs_heap: This benchmark will measure the latency of pool.get() and Pooled<T>::drop versus Box::new() and drop(Box<T>) for a representative analysis object. This will directly validate the "50%+ faster allocation" target for pooled objects.
Rust
// benches/pool_benchmark.rs
use criterion::{criterion_group, criterion_main, Criterion, Bencher};

fn bench_pool_allocation_vs_heap(c: &mut Criterion) {
    let pool = UveddiObjectPool::<MyObject>::new(100);

    c.bench_function("pool_get_put", |b: &mut Bencher| {
        b.iter(|| {
            // Get object and immediately drop it to return it to the pool
            let obj = pool.get().unwrap();
            criterion::black_box(obj);
        })
    });

    c.bench_function("heap_alloc_dealloc", |b: &mut Bencher| {
        b.iter(|| {
            let obj = Box::new(MyObject::default());
            criterion::black_box(obj);
        })
    });
}


bench_arena_allocation_vs_heap: This will compare the performance of creating and populating a bumpalo::collections::Vec versus a standard std::vec::Vec. This measures the cumulative effect of avoiding multiple small heap allocations.
Macro-benchmarks: These will measure the real-world impact on the entire application.
bench_large_codebase_analysis: This benchmark will run a full Uveddi analysis on a large, real-world codebase like tokio or rust-lang/rust. It will be run with a cold cache to measure raw analysis throughput. This is the primary benchmark for validating the "<8GB memory usage" target.
bench_cache_hit_performance: This benchmark will perform the same analysis as above but with a pre-warmed AST cache. This will specifically measure the performance improvement from the mmap+rkyv zero-copy caching strategy and is critical for evaluating the tool's performance in common incremental development workflows.
The design of the benchmark suite must distinguish between these first-run (cold cache) and incremental-run (warm cache) scenarios. The memory optimizations impact these two cases differently: arenas primarily benefit the first run, while the zero-copy cache dramatically accelerates incremental runs. Measuring both provides a complete picture of the performance gains.

8.2 Profiling Methodology with heaptrack and valgrind-massif

Benchmarking measures "how fast," while profiling explains "why." We will use two key Linux-based profilers to get a deep understanding of Uveddi's memory behavior.
heaptrack: This tool is exceptionally well-suited for our goals. It traces every memory allocation, allowing us to precisely measure allocation frequency, identify allocation hotspots (which code paths allocate the most), and find temporary allocations.51 We will use
heaptrack before and after the changes to quantitatively verify the "70% reduction in allocation frequency" target. It is also invaluable for guiding the refactoring effort: by running it on the current codebase, we can identify the most allocation-heavy detectors and prioritize their refactoring.52
valgrind --tool=massif: Massif is a heap profiler that produces detailed reports and graphs of heap usage over time.54 It is the ideal tool for visualizing the reduction in peak memory usage and for measuring memory fragmentation. We will generate Massif graphs for the large codebase analysis benchmark before and after the optimizations to provide clear, visual evidence of the improvements.
For effective profiling, release builds must be compiled with minimal debug information (debug = "line-tables-only" in Cargo.toml) and with frame pointers enabled (-C force-frame-pointers=yes) to ensure accurate stack traces.55

Section 9: Phased Implementation and Rollout Plan

The implementation is designed to fit within the 3-day effort estimate by prioritizing tasks and delivering value incrementally.

9.1 Phase 1 (Day 1): Foundational Allocators & Caching Backend

The first day focuses on foundational changes that unblock subsequent work and provide immediate system-wide benefits.
Integrate Global Allocator: Add mimalloc to Cargo.toml and configure it as the #[global_allocator]. This is a quick win that improves the performance of all remaining heap allocations.
Implement Object Pool: Build the core UveddiObjectPool<T> and Pooled<T> structures. The initial implementation can use a simpler Arc<Mutex<...>> backend, with the sharded slab as a later optimization if needed.
Upgrade AST Cache: Refactor the AST cache to use memmap2 and switch its serialization backend from bincode to rkyv. This is a critical dependency for later phases, as it changes how the analysis engine consumes cached ASTs.

9.2 Phase 2 (Day 2): Arena Integration and Core Refactoring

The second day focuses on the most impactful change: integrating arenas into the analysis engine.
Introduce Analysis Arenas: Modify the AnalysisEngine and its main processing loop to create and pass down a bumpalo::Bump arena for each file analysis task.
Data-Driven Refactoring: Using the profiling data gathered from heaptrack, refactor the top 2-3 most allocation-heavy detectors to use the arena and bumpalo's collections. This ensures that engineering effort is focused on the areas with the highest potential for improvement.
Integrate Object Pool: Identify one or two key data structures that fit the "expensive-to-create, reusable" profile and integrate them with the UveddiObjectPool.

9.3 Phase 3 (Day 3): Full Integration, Validation, and Tuning

The final day is dedicated to completing the integration, running the full validation suite, and finalizing the implementation.
Complete Refactoring: Roll out the arena-based allocation pattern to the remaining detectors and analysis pipeline components.
Implement Configuration: Implement the static AllocationStrategy framework, allowing pool sizes to be configured.
Full Validation Run: Execute the entire criterion benchmark suite and profiling scripts (heaptrack, massif) on the final code.
Analyze and Tune: Compare the results against the baseline and the quantitative targets. Tune pool capacities and other configuration parameters based on the benchmark results.
Documentation and Handoff: Document the new memory architecture, the APIs for the allocators, and the guidelines for their use.

Conclusions and Recommendations

The memory architecture outlined in this report represents a fundamental and necessary evolution for the Uveddi static analysis tool. The current reliance on a general-purpose allocator and standard heap-based collections is the primary cause of the performance and scalability issues plaguing the analysis of large codebases. By adopting a tiered, context-aware memory management strategy, Uveddi will align with the proven best practices of industry-leading tools like rustc and Clang, enabling it to meet its aggressive performance targets.
The key recommendations are as follows:
Adopt a Three-Tiered Allocation Strategy: Immediately replace the system allocator with mimalloc. Implement a concurrent object pool for reusable, long-lived objects. Critically, integrate per-analysis-task arena allocation using bumpalo for all transient data, which will provide the most significant performance gains by eliminating the bulk of allocation overhead and fragmentation.
Modernize the AST Cache with Zero-Copy Deserialization: Migrate the AST cache from bincode to rkyv, in conjunction with memmap2. This will transform cache hits from a slow deserialization process into an instantaneous, allocation-free operation, dramatically improving incremental analysis performance.
Enforce Safe Usage Programmatically: The power of arenas, particularly bumpalo's dropless approach, comes with the risk of memory leaks if used incorrectly. This risk must be mitigated by mandating the use of arena-aware collections and enforcing this rule with a custom static analysis lint integrated into the CI/CD pipeline.
Implement a Phased, Data-Driven Rollout: The provided three-day implementation plan prioritizes foundational work and focuses refactoring efforts on the most allocation-heavy components, as identified by profiling tools like heaptrack. This de-risks the implementation and ensures maximum impact within the allotted time.
Establish a Culture of Performance Measurement: The benchmarking and profiling suite defined in this report should not be a one-off validation effort. It must become a permanent part of Uveddi's development lifecycle to guard against future performance regressions and to inform ongoing optimization work.
By executing this plan, the Uveddi team will not only solve the immediate memory pressure issues but will also build a robust, scalable, and observable memory foundation. This will unlock new levels of performance, enabling Uveddi to confidently analyze codebases of any scale while maintaining the reliability and correctness expected of a premier static analysis tool.
Works cited
Memory management in rustc - Rust Compiler Development Guide, accessed July 14, 2025, https://rustc-dev-guide.rust-lang.org/memory.html
Arena-Based Allocation in Compilers - Inferara, accessed July 14, 2025, https://www.inferara.com/en/blog/arena-based-allocation-in-compilers/
Arena-Based Allocation in Compilers | by Inferara - Medium, accessed July 14, 2025, https://medium.com/@inferara/arena-based-allocation-in-compilers-b96cce4dc9ac
clang::ASTContext Class Reference, accessed July 14, 2025, https://clang.llvm.org/doxygen/classclang_1_1ASTContext.html
include/clang/AST/ASTContextAllocate.h File Reference, accessed July 14, 2025, https://clang.llvm.org/doxygen/ASTContextAllocate_8h.html
How to clone or create an AST Stmt node of clang? - Stack Overflow, accessed July 14, 2025, https://stackoverflow.com/questions/30451485/how-to-clone-or-create-an-ast-stmt-node-of-clang
clang::APNumericStorage Class Reference, accessed July 14, 2025, https://clang.llvm.org/doxygen/classclang_1_1APNumericStorage.html
Allocating AST nodes - Help - Ziggit, accessed July 14, 2025, https://ziggit.dev/t/allocating-ast-nodes/7800
Garbage Collection with LLVM — LLVM 21.0.0git documentation, accessed July 14, 2025, https://llvm.org/docs/GarbageCollection.html
Reconciling High-Level Optimizations and Low-Level Code in LLVM - Software Foundations Lab, accessed July 14, 2025, https://sf.snu.ac.kr/publications/llvmtwin.pdf
LLVM Language Reference Manual — LLVM 21.0.0git documentation, accessed July 14, 2025, https://llvm.org/docs/LangRef.html
Optimizing Performance | Bazel, accessed July 14, 2025, https://bazel.build/rules/performance
Optimize Memory | Bazel, accessed July 14, 2025, https://bazel.build/advanced/performance/memory
Breaking down build performance | Bazel, accessed July 14, 2025, https://bazel.build/advanced/performance/build-performance-breakdown
For a video game running at 60 frames per second each frame has 16 millisecond... | Hacker News, accessed July 14, 2025, https://news.ycombinator.com/item?id=21788823
Guide to using arenas in Rust - LogRocket Blog, accessed July 14, 2025, https://blog.logrocket.com/guide-using-arenas-rust/
Optimizing Rust Performance with jemalloc | by Leapcell - Medium, accessed July 14, 2025, https://leapcell.medium.com/optimizing-rust-performance-with-jemalloc-c18057532194
Memory management — list of Rust libraries/crates // Lib.rs, accessed July 14, 2025, https://lib.rs/memory-management
Picking a global allocator : r/rust - Reddit, accessed July 14, 2025, https://www.reddit.com/r/rust/comments/1ifjzvv/picking_a_global_allocator/
SoftwarePatternsLexicon.com/content/patterns-rust/6/9/index.md at main · MasteryEducation ... - GitHub, accessed July 14, 2025, https://github.com/MasteryEducation/SoftwarePatternsLexicon.com/blob/main/content/patterns-rust/6/9/index.md
Turns out, using custom allocators makes using Rust way easier : r/rust, accessed July 14, 2025, https://www.reddit.com/r/rust/comments/1jlopns/turns_out_using_custom_allocators_makes_using/
Rust Custom Allocators - Ian Bull, accessed July 14, 2025, https://ianbull.com/posts/rust-custom-allocators/
rust - How to implement a custom allocator? - Stack Overflow, accessed July 14, 2025, https://stackoverflow.com/questions/30326322/how-to-implement-a-custom-allocator
oxc_allocator - Rust - Docs.rs, accessed July 14, 2025, https://docs.rs/oxc_allocator
refpool - Rust - Docs.rs, accessed July 14, 2025, https://docs.rs/refpool
Implementing a pool of objects - help - The Rust Programming Language Forum, accessed July 14, 2025, https://users.rust-lang.org/t/implementing-a-pool-of-objects/27960
Effortless Resource Management: Easy Object Pooling in Rust, accessed July 14, 2025, https://www.hackingwithrust.net/2023/10/15/an-object-pool-in-rust-two-implementations/
Arenas and Rust - Josh Haberman, accessed July 14, 2025, https://blog.reverberate.org/2021/12/19/arenas-and-rust.html
Making Arenas Memory-Safe Via New Standard Trait? - Rust Users Forum, accessed July 14, 2025, https://users.rust-lang.org/t/making-arenas-memory-safe-via-new-standard-trait/121855
nesting allocators - Yoshua Wuyts, accessed July 14, 2025, https://blog.yoshuawuyts.com/nesting-allocators/
memmap2 - crates.io: Rust Package Registry, accessed July 14, 2025, https://crates.io/crates/memmap2
Memory-Mapped I/O vs Standard File I/O | Bernardo de Lemos, accessed July 14, 2025, http://bernardo.shippedbrain.com/mmap_vs_stdio/
Towards a more perfect RustIO - Page 2 - The Rust Programming Language Forum, accessed July 14, 2025, https://users.rust-lang.org/t/towards-a-more-perfect-rustio/18570?page=2
Rust program runs faster on Linux than Windows? - Reddit, accessed July 14, 2025, https://www.reddit.com/r/rust/comments/j2ilkn/rust_program_runs_faster_on_linux_than_windows/
StephanvanSchaik/mmap-rs: A cross-platform and safe Rust API to create and manage memory mappings in the virtual address space of the calling process. - GitHub, accessed July 14, 2025, https://github.com/StephanvanSchaik/mmap-rs
rkyv - Rust - Docs.rs, accessed July 14, 2025, https://docs.rs/rkyv
Crate rkyv - Rust, accessed July 14, 2025, https://doc.qu1x.dev/trackball/rkyv/index.html
rkyv - rkyv, accessed July 14, 2025, https://rkyv.org/
Zero-copy deserialization - rkyv, accessed July 14, 2025, https://rkyv.org/zero-copy-deserialization.html
Rust - vorpal.se, accessed July 14, 2025, https://vorpal.se/tag/rust/
rkyv - crates.io: Rust Package Registry, accessed July 14, 2025, https://crates.io/crates/rkyv/0.7.39
FAQ - rkyv, accessed July 14, 2025, https://rkyv.org/faq.html
Unofficial Guide to Rust Optimization Techniques | by Yong kang ..., accessed July 14, 2025, https://extremelysunnyyk.medium.com/unofficial-guide-to-rust-optimization-techniques-ec3bd54c5bc0
memuse - Rust - Docs.rs, accessed July 14, 2025, https://docs.rs/memuse/
Memory Safety in Web Rust System Zero Cost Secure（1751550293518900） - DEV Community, accessed July 14, 2025, https://dev.to/member_57439f86/memory-safety-in-web-rust-system-zero-cost-secure1751550293518900-24nd
SharedMemory in wasmtime - Rust, accessed July 14, 2025, https://docs.wasmtime.dev/api/wasmtime/struct.SharedMemory.html
Executing Wasm (Rust compiled to wasm) from multiple threads with shared memory using wasmtime. - Reddit, accessed July 14, 2025, https://www.reddit.com/r/rust/comments/1jm5tyo/executing_wasm_rust_compiled_to_wasm_from/
How to load dynamic libraries in Rust? - Gaurav Gahlot, accessed July 14, 2025, https://gauravgahlot.in/rust-dynamic-libraries/
Rust Benchmarking with Criterion.rs - Rustfinity, accessed July 14, 2025, https://www.rustfinity.com/blog/rust-benchmarking-with-criterion
How to benchmark Rust code with Criterion - Bencher, accessed July 14, 2025, https://bencher.dev/learn/benchmarking/rust/criterion/
KDE/heaptrack: A heap memory profiler for Linux - GitHub, accessed July 14, 2025, https://github.com/KDE/heaptrack
Profiling heap allocation in rust - FlakM blog, accessed July 14, 2025, https://flakm.github.io/posts/heap_allocation/
Profiling Rust Programs with valgrind, heaptrack, and hyperfine - YouTube, accessed July 14, 2025, https://www.youtube.com/watch?v=X6Xz4CRd6kw
Valgrind, accessed July 14, 2025, https://valgrind.org/docs/manual/ms-manual.html
Profiling - The Rust Performance Book, accessed July 14, 2025, https://nnethercote.github.io/perf-book/profiling.html
