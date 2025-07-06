
Technical Specification for Architectural Anti-Pattern Detectors in Rust


Part I: Foundational Infrastructure for Code Analysis

The effective detection of architectural anti-patterns such as leaky abstractions and tight coupling necessitates a robust and performant foundational infrastructure. This infrastructure is responsible for transforming a project's source code into a structured, queryable representation—a dependency graph. This section details the design and implementation of this core component, focusing on multi-language support, efficient data structures, and a high-performance analysis engine optimized for large-scale codebases.

Section 1: Constructing a Multi-Language Dependency Graph

The cornerstone of the analysis is a comprehensive dependency graph that represents all entities (functions, classes, structs, modules, etc.) and their interrelationships across the entire codebase. Building this graph for multiple languages requires a sophisticated, multi-stage process.

1.1 Cross-File Symbol Resolution and Graph Construction

The construction of the dependency graph is a two-stage process designed for accuracy and parallelism. This approach decouples the parsing of individual files from the global linking of dependencies, which is essential for scalable analysis.
Stage 1: File-Level Parsing and Symbol Extraction
In this initial stage, every source file in the target project is parsed independently. This task is inherently parallelizable. The process for each file is as follows:
Parsing: A tree_sitter::Parser instance, configured with the appropriate language grammar (e.g., tree-sitter-rust, tree-sitter-python), is used to generate a concrete syntax tree (CST) for the file.1
Symbol Identification: Language-specific Tree-sitter queries are executed against the CST to identify all defined symbols (e.g., structs, functions, classes) and all used symbols (e.g., imports, function calls, type annotations).
Scope Generation: The results are stored in a temporary FileScope structure. This struct contains two primary collections: defined_symbols and used_symbols. Each entry includes the symbol's name, its type (function, class, etc.), and its source location (file path, line, and column).
Stage 2: Graph Linking and Population
After all files have been processed in Stage 1, the second stage commences. This stage is responsible for resolving the dependencies identified in the FileScope objects and constructing the global petgraph graph.
Symbol Table Population: A central, thread-safe symbol table, implemented using dashmap::DashMap<String, NodeIndex>, maps fully qualified symbol names to their corresponding NodeIndex in the graph. The defined_symbols from every FileScope are used to populate this map. The use of DashMap is critical as it allows for concurrent reads and writes, facilitating a parallel-first architecture.
Dependency Resolution: The engine iterates through the used_symbols list of each FileScope. For each used symbol, it queries the SymbolTable to find the NodeIndex of the corresponding definition.
Edge Creation: Once the source node (the entity using the symbol) and the target node (the defined symbol being used) are identified, a directed edge is created between them in the petgraph graph. The edge can be weighted with data representing the type of dependency (e.g., function call, trait implementation, type usage).
Tree-sitter Queries for Symbol Extraction
The accuracy of this process hinges on precise Tree-sitter queries tailored for each language.
Rust: use_declaration nodes are queried to identify module-level dependencies. struct_item, trait_item, and function_item nodes are used to identify definitions.3
Python: Dependencies are primarily identified by querying for import_statement and from_import_statement nodes. Definitions are found via class_definition and function_definition nodes.5
JavaScript: For ES Modules (ESM), import_statement queries are sufficient. For CommonJS, the query must identify call_expression nodes where the callee is the require identifier.8
This two-stage process ensures that all symbol definitions are known before attempting to resolve usages, correctly handling the arbitrary file orderings common in modern software projects.

1.2 Selecting the Optimal petgraph Representation

The choice of data structure for the dependency graph is a foundational architectural decision that profoundly impacts performance, memory usage, and future extensibility. The petgraph crate offers several implementations, each with distinct trade-offs.10
petgraph::Graph: An adjacency list representation. It is the most versatile and widely supported type within the petgraph ecosystem, making it an excellent default choice. Its primary drawback is that removing nodes can invalidate existing NodeIndex and EdgeIndex values by shifting internal arrays.10 For a non-interactive CLI tool like
Uveddi that builds the graph once per run, this is not a significant issue.
petgraph::StableGraph: Similar to Graph but guarantees that indices remain stable across node or edge removals. This stability comes at the cost of slightly higher memory usage and potential performance overhead.10 It is the ideal choice if
Uveddi is expected to support incremental analysis or interactive graph modification in the future.
petgraph::GraphMap: An adjacency list backed by a HashMap. Node identifiers are the hash keys themselves, which can simplify lookups (e.g., using a function's name directly as its identifier). However, this requires node identifiers to implement Copy, Eq, and Hash, which is often too restrictive for the complex data needed to represent a code entity.12
petgraph::Csr (Compressed Sparse Row): A highly compact and memory-efficient representation optimized for sparse graphs.10 Since source code dependency graphs are typically sparse (most entities do not depend on most other entities),
Csr offers significant memory savings for very large codebases. Its API is slightly more restrictive than Graph's.
Recommendation:
The recommended approach for Uveddi is to start with petgraph::Graph. Its flexibility and comprehensive algorithm support provide the fastest path to a functional implementation. The node weight (N) should be a custom struct (CodeEntity) containing the symbol's fully qualified name, its kind (struct, function, etc.), file path, and source location. The edge weight (E) should be an enum (DependencyKind) representing the type of dependency (e.g., Call, Implementation, TypeUsage).
Crucially, the analysis engine should be designed around a trait-based abstraction for graph operations. This will allow the underlying graph implementation to be swapped from Graph to Csr in the future as a performance optimization, without requiring a major rewrite of the analysis logic.

1.3 Memory-Efficient Graph Traversal Techniques

For large codebases, the memory footprint of graph traversal algorithms can become a bottleneck. Several techniques can mitigate this.
Index-Based Graph Representation: The fundamental design of petgraph, which uses integer indices (NodeIndex, EdgeIndex) to refer to nodes and edges, is inherently more memory-efficient than pointer-based approaches (e.g., Rc<RefCell<Node>>). This design avoids heap fragmentation, pointer overhead, and the complexities of managing reference cycles in Rust's ownership system.14
Efficient "Visited" Sets: During graph traversals like Depth-First Search (DFS) or Breadth-First Search (BFS), a mechanism is needed to track visited nodes to prevent infinite loops. While a HashSet<NodeIndex> is functionally correct, using a fixedbitset::FixedBitSet can be significantly more memory-efficient, especially for graphs with a large number of nodes. A FixedBitSet uses a single bit per node, whereas a HashSet has higher overhead per entry. The fixedbitset crate is already a dependency of petgraph, making it readily available.15
Bitwise Operations for Specialized Algorithms: For certain dense sub-problems within the graph, such as finding all nodes reachable within two steps (a key operation for some coupling metrics), advanced bitwise techniques can offer dramatic performance improvements. By representing the adjacency matrix (or relevant parts of it) as a series of bitmasks, entire rows of connections can be processed in a single CPU instruction using bitwise OR operations. This can lead to performance gains of over 40x for specific algorithms.16 While this is an advanced optimization, awareness of its potential is key for future performance tuning.

1.4 Handling Partial Parses and Syntax Errors Gracefully

Real-world codebases are rarely free of syntax errors, especially during active development. A robust static analysis tool must not fail on the first malformed file.
Tree-sitter's Error Recovery: Tree-sitter is designed for resilience. When it encounters a syntax error, it does not stop. Instead, it inserts an ERROR node into the syntax tree, attempts to recover, and continues parsing the rest of the file.17 This produces a partial but usable tree.
Analysis Strategy: The Uveddi engine should be designed to handle these partial trees.
Continue on Parse Failure: The main file processing loop should treat a parse failure as a non-fatal event.
Log and Report Errors: The tool should collect the locations of all ERROR nodes and report them to the user as warnings, indicating that the analysis for those files may be incomplete. A simple Tree-sitter query (ERROR) @error can find all such nodes.
Skip Invalid Subtrees: During dependency extraction, the analysis logic should be programmed to skip traversing any ERROR node and its children. This prevents malformed code from introducing spurious or incorrect dependencies into the graph.
Detecting Missing Nodes: In addition to ERROR nodes, Tree-sitter can insert (MISSING) nodes where it expects a token but finds none. These can also be queried (e.g., (MISSING identifier) @missing_id) to provide more precise error diagnostics to the user.18
By adopting this strategy, Uveddi can provide maximum value even on imperfect codebases, a critical feature for a tool intended for use in continuous integration and development workflows.

Section 2: High-Performance Analysis Engine

To be practical, a static analysis tool must execute quickly. For Uveddi, this means leveraging modern hardware through parallelism and optimizing the core analysis algorithms.

2.1 Parallelizing Graph Construction and Traversal with Rayon

The analysis workflow is well-suited for parallel execution, which can dramatically reduce wall-clock time on multi-core processors. The rayon crate provides a simple yet powerful model for data parallelism in Rust.19
Parallel Parsing and Symbol Extraction: The first stage of graph construction—parsing files and extracting symbols—is an "embarrassingly parallel" problem. Given a list of file paths, rayon's parallel iterators (par_iter()) can be used to distribute the workload across a thread pool. Each thread will independently parse a file and produce a FileScope object. This approach scales linearly with the number of available CPU cores and is the single most effective optimization for the entire process.
Rust
use rayon::prelude::*;
use std::path::PathBuf;

struct FileScope { /*... */ }

fn parse_file(path: &PathBuf) -> Result<FileScope, std::io::Error> {
    //... implementation using tree-sitter...
    // Ok(file_scope)
    unimplemented!();
}

fn run_parallel_parsing(file_paths: Vec<PathBuf>) -> Vec<Result<FileScope, std::io::Error>> {
    let results: Vec<_> = file_paths
       .par_iter()
       .map(|path| parse_file(path))
       .collect();
    results
}


Concurrent Graph Population: While parsing is parallel, writing to the shared petgraph::Graph requires synchronization. The most straightforward approach is to wrap the graph and the SymbolTable in an Arc<Mutex<T>>. A more performant pattern involves using DashMap for the symbol table, which allows for lock-free concurrent writes, and collecting all node/edge creation commands from the worker threads into a Vec. A final, single-threaded step can then apply these commands to the petgraph::Graph, minimizing the duration of contention.
Parallel Algorithms with petgraph: The petgraph crate includes a rayon feature flag. When enabled, this feature provides parallel implementations for certain iterators and algorithms.10 This is particularly useful for calculating metrics across all nodes in the graph. For instance, computing Fan-in/Fan-out for every node can be done in parallel, where each thread calculates the metric for a subset of the nodes. This design choice is informed by the challenges faced by projects like
rustc in retrofitting parallelism; by designing for it from the start, Uveddi can avoid these legacy constraints.20

2.2 Advanced Tree-sitter Query Optimization

The efficiency of dependency extraction is directly tied to the performance of the Tree-sitter queries. Writing optimized queries is crucial.
Maximize Specificity: Queries should be as specific as possible to reduce the search space. Avoid generic wildcards like (_) when a specific node type is known. Anchoring a pattern within a larger, more specific structure significantly improves performance. For example, instead of a broad query for (identifier), a more targeted query like (call_expression function: (identifier) @call) is much more efficient because it constrains the context of the match.21
Utilize Named Fields: Tree-sitter grammars define named fields for children of a node (e.g., a function_declaration node has name and body fields). Using these fields in queries (name: (identifier) @func_name) is more robust and performant than relying on the order of anonymous child nodes.18
Leverage Predicates for In-Engine Filtering: Tree-sitter's query language includes predicates that allow for filtering directly within the query engine. Predicates like #eq? (for equality checks) and #match? (for regex matching) should be used to filter nodes based on their text content.22 This is far more efficient than fetching all potential matches into Rust and then filtering them, as it minimizes the data that needs to cross the boundary from the C library to the Rust runtime.
Scheme
; Inefficient: Get all function calls and filter in Rust
(call_expression function: (identifier) @name)

; Efficient: Filter within the query itself
(call_expression
  function: (identifier) @name
  (#match? @name "^get_"))


Understand Locality: Be mindful of "non-local" patterns, which are patterns that can match within repeating sequences of nodes. These patterns can disable certain internal optimizations. The query.is_pattern_non_local(index) method can be used to check for this property.24 For performance-critical paths, structuring queries to be "rooted" (starting from a single, unambiguous node type) is a best practice.
By implementing this high-performance foundational infrastructure, Uveddi will be well-equipped to support the complex analyses required by the Leaky Abstraction and Tight Coupling detectors.

Part II: Leaky Abstraction Detector: Implementation Strategies

A leaky abstraction exposes implementation details that its users should not need to be concerned with. These leaks manifest in various forms, from architectural layering violations to subtle performance degradation and framework-specific coupling. This section provides detailed implementation blueprints for detecting these anti-patterns.

Section 3: Architectural Layer Conformance Validation

A clean architecture enforces strict rules about how different layers of an application can interact. For example, the domain layer should be independent of the presentation layer. This detector validates the implemented architecture against a user-defined plan.

3.1 Defining and Configuring Architectural Layers

The tool cannot infer architectural intent; it must be provided by the user. This is best achieved through a clear and explicit configuration file, such as uveddi.toml.
Configuration Schema: The user will define their architectural layers by mapping directory glob patterns to layer names. They will also specify the allowed dependency flow between these layers.
Ini, TOML
# Example uveddi.toml configuration

[layers]
# Maps file paths to logical layer names.
# The first matching glob determines the layer.
presentation = ["src/presentation/**", "src/api/**", "src/controllers/**"]
application = ["src/application/**", "src/services/**"]
domain = ["src/domain/**"]
infrastructure = ["src/infrastructure/**", "src/db/**"]

[layer_rules]
# Defines the matrix of allowed dependencies.
# Any dependency not explicitly listed here is considered a violation.
allow = ["presentation", "application"],
    ["application", "domain"],
    ["application", "infrastructure"],
    ["infrastructure", "domain"]


Internal Representation: During the graph construction phase, as each file is parsed, its path is checked against the [layers] configuration. The corresponding layer name is then stored as metadata on the node(s) representing that file or its contents in the petgraph graph. This tagging process enriches the dependency graph with architectural context.

3.2 A Rule-Based Engine for Detecting Layering Violations (Divergences)

With the graph nodes tagged by layer, detecting violations becomes a straightforward, rule-based process. This approach directly implements the concept of "divergence" from academic work on reflexion models, where a divergence is a dependency found in the source code that is forbidden by the high-level architectural model.25
Algorithm:
After the full, layer-tagged dependency graph is built, iterate through every edge in the graph.
For each edge from a source node U to a destination node V, retrieve their respective layers, Layer(U) and Layer(V).
If Layer(U) and Layer(V) are different, consult the allow list from the [layer_rules] configuration.
If the tuple [Layer(U), Layer(V)] is not present in the allow list, a layering violation has been detected.
The violation should be reported to the user with the source file and line number of the dependency, the target of the dependency, and a clear message explaining the rule that was broken (e.g., "Violation: Domain layer should not depend on Presentation layer. Found dependency from src/domain/user.rs to src/presentation/http.rs.").
Rule-Based vs. Machine Learning: A rule-based approach is unequivocally superior for this task. Architectural rules are deterministic and intentional. A rule-based system provides transparency and direct control, allowing developers to enforce their exact architectural vision.27 An ML-based approach would attempt to
infer rules, which is counterproductive when the goal is to conform to an explicitly planned architecture. ML models are better suited for problems where patterns are complex and not easily describable with rules, which is not the case here.27

3.3 Heuristics for Monolithic vs. Microservice Architectures

The layer validation strategy applies differently depending on the architectural style of the project.
Layered Monoliths: The described approach is perfectly suited for analyzing traditional layered monoliths, n-tier applications, and modular monoliths. The entire codebase resides in a single repository, allowing the tool to build a complete dependency graph and validate it against the defined layers.
Microservices: Static analysis of a single microservice repository cannot, by itself, detect architectural violations between services (e.g., one service directly calling another's database). However, Uveddi can provide significant value in a microservices context in two ways:
Internal Service Architecture: Each individual microservice should have its own clean internal architecture (e.g., following hexagonal or layered patterns). The detector can be run on a single service's repository to enforce these internal boundaries, ensuring the service itself is well-designed and maintainable.
Identifying Coupling for Decomposition: When analyzing a legacy monolith that is a candidate for decomposition into microservices, the Tight Coupling detector (discussed in Part III) becomes invaluable. It can identify modules with high CBO and RFC metrics, which represent areas of the code that will be difficult to separate and may require significant refactoring before they can be extracted into independent services.
Detecting cross-service violations would require a more advanced form of analysis that extends beyond source code to infrastructure-as-code configurations (e.g., network policies, database access rules) or dynamic analysis of network traffic, which represents a potential future direction for the tool.

Section 4: Detecting Performance-Related Abstraction Leaks

Some of the most insidious abstraction leaks are those that manifest as performance problems. An Object-Relational Mapper (ORM), for instance, provides a powerful abstraction over SQL, but when used incorrectly, it can leak performance details by generating highly inefficient queries. This section focuses on statically identifying the patterns of such leaks.

4.1 Statically Identifying N+1 Query Anti-Patterns in ORM Usage

The N+1 query problem is a classic performance anti-pattern where an application performs one initial query to retrieve a list of N items, and then executes N additional queries inside a loop to fetch related data for each item.30 This results in
N+1 database round-trips instead of an optimal one or two.
The Static Pattern: This anti-pattern has a recognizable structure in code: a loop that iterates over a collection returned by an ORM query, with another ORM query executed inside the loop's body.31
Detection via Tree-sitter: This structure can be precisely targeted with Tree-sitter queries. The key is to identify a loop construct whose iterator is an ORM query result, and whose body contains another ORM query call.
Table: N+1 Anti-Pattern Query Mapping
Anti-Pattern
Language/Framework
Code Example
Tree-sitter Query
Key Captures
ORM Query in Loop
Python / Django
python\n# N+1 Problem\nauthors = Author.objects.all()\nfor author in authors:\n # This triggers one query per author\n print(author.books.all())\n
(for_statement right: (call function: (attribute attribute: (identifier) @_orm_method_outer (#any-of? @_orm_method_outer "all" "filter"))) @query_collection body: (block. (call function: (attribute object: (attribute) attribute: (identifier) @_orm_method_inner (#any-of? @_orm_method_inner "all" "filter" "get"))) @n_plus_one_call)) @n1_loop
@n1_loop: The entire for loop.
@query_collection: The initial "1" query.
@n_plus_one_call: The "N" queries inside the loop.
ORM Query in Loop
JavaScript / Prisma
javascript\n// N+1 Problem\nconst users = await prisma.user.findMany();\nfor (const user of users) {\n // This triggers one query per user\n const posts = await prisma.post.findMany({\n where: { authorId: user.id },\n });\n}\n
(for_in_statement right: (await_expression (call_expression)) @query_collection body: (statement_block. (await_expression (call_expression function: (member_expression property: (property_identifier) @_orm_method (#eq? @_orm_method "findMany")))) @n_plus_one_call)) @n1_loop
@n1_loop: The entire for...of loop.
@query_collection: The initial "1" query.
@n_plus_one_call: The "N" queries inside the loop.

Reporting and Remediation: When a match is found, the tool should report the location of the loop (@n1_loop) and suggest the appropriate fix for the framework, which is typically eager loading. For Django, this would be using select_related (for foreign keys) or prefetch_related (for many-to-many or reverse foreign keys).31 For Prisma, this involves using the
include option in the initial query.34

4.2 Detecting Unawaited Tasks and Potential Runtime Leaks in Async Rust

In Rust's async ecosystem, a common source of bugs and resource leaks is spawning a task and failing to await its completion. When tokio::spawn or a similar function is called, it returns a JoinHandle. If this handle is immediately dropped, the spawned task becomes "detached." The main program will not wait for it to complete, and if the program exits, the task may be silently terminated, leaving resources (like open files or network connections) in an inconsistent state.35
The Static Pattern: The anti-pattern is a call to a task-spawning function (like tokio::spawn) that is not part of an assignment, not immediately awaited, and not stored in a collection. The most direct indicator is when the call is the entirety of an expression_statement.
Tree-sitter Query (Rust):
Scheme
(expression_statement
  (call_expression
    function: (scoped_identifier
      path: (identifier) @mod
      name: (identifier) @func
      (#any-of? @mod "tokio" "task")
      (#eq? @func "spawn")
    )
  ) @spawn_call
) @unawaited_spawn


Detection Logic: This query is highly specific. It looks for a call to tokio::spawn or task::spawn that constitutes a standalone statement. This syntactic structure implies its return value, the JoinHandle, is being discarded. A correct usage pattern, such as let handle = tokio::spawn(...), would be a let_declaration node and would not be matched. Similarly, tokio::spawn(...).await? would be part of an await_expression.
Important Caveat: Static analysis cannot definitively prove a memory leak.38 It can, however, identify code patterns that are strongly correlated with bugs. The tool's output should be phrased carefully, such as: "Potential resource leak: Task spawned on line X is not awaited or tracked. The program will not wait for this task to complete upon exit." This guides the developer to use structured concurrency patterns, like
tokio::task::JoinSet, to manage spawned tasks correctly.37

4.3 Identifying Event Loop Blocking Patterns in JavaScript

Node.js achieves high concurrency through a non-blocking, single-threaded event loop. Any long-running, synchronous, CPU-bound code will block this loop, preventing the application from handling any other concurrent requests.40 While perfectly identifying all such code is impossible statically, common culprits can be detected.
The Static Pattern: The most common offenders are synchronous I/O operations and complex regular expression evaluations. For example, using fs.readFileSync() instead of its asynchronous counterpart fs.readFile() in a request handler is a classic blocking anti-pattern.
Tree-sitter Query for Synchronous I/O (JavaScript):
Scheme
(call_expression
  function: (member_expression
    object: (identifier) @fs (#eq? @fs "fs")
    property: (property_identifier) @sync_call (#match? @sync_call "Sync$")
  )
) @blocking_io_call


Detection Logic: This query identifies any method call on an object named fs where the method name ends in Sync (e.g., readFileSync, writeFileSync, statSync). This is a strong heuristic for detecting potential event loop blocking. The analysis could be further refined by checking if this call occurs within a function that appears to be a request handler (e.g., in an Express or Koa application).
Regex DoS: Another pattern to detect is "ReDoS" (Regular Expression Denial of Service), which involves "vulnerable" regular expressions with nested quantifiers or overlapping clauses that can lead to catastrophic backtracking on certain inputs.40 A query can search for
new RegExp() calls or regex literals containing these problematic patterns.

Section 5: Language-Specific Leakage Rules for Python and JavaScript

Beyond general performance issues, abstractions can leak when domain logic becomes coupled to framework-specific implementation details.

5.1 Detecting Framework Coupling in Django and Flask Applications

A core principle of clean architecture is that the domain or business logic should be independent of the delivery mechanism (e.g., the web framework). A leak occurs when a domain module imports and uses concepts from the web layer, such as HttpRequest objects.41
The Pattern: A Python file designated as part of the "domain" or "application" layer (via uveddi.toml configuration) contains an import statement that pulls in a web-specific class from the django or flask packages.
Detection Logic:
First, identify the layer of the file being analyzed based on its path.
If the file belongs to a core layer (e.g., domain), execute a query to find forbidden imports.
Flag any import from django.http, django.views, flask.request, etc., as a layering violation.
Tree-sitter Query (Python):
Scheme
; Detects importing web-layer classes into non-web layers
(import_from_statement
  module_name: (dotted_name
    (identifier) @mod_name
    (#any-of? @mod_name "django" "flask")
  )
  name: (dotted_name
    (identifier) @imported_class
    (#any-of? @imported_class "HttpRequest" "Response" "request")
  )
) @framework_leak


This query, combined with the layer information, allows Uveddi to enforce a strict separation of concerns, preventing the business logic from becoming entangled with the web framework's implementation details.43

5.2 Identifying Business Logic Leakage into React and Vue UI Components

In modern frontend development, it is a best practice to separate UI rendering logic from business logic. Components should be responsible for displaying state, while complex state management and business operations should be handled by dedicated services, custom hooks, or state management libraries (e.g., Redux, Pinia).44 A common form of leakage is when components fail to clean up resources, leading to memory leaks.
The Pattern (Memory Leaks): A React component uses the useEffect hook to set up a subscription, timer, or event listener but fails to provide a cleanup function. This means the resource will persist in memory even after the component has unmounted.45
Detection Logic for Uncleaned Side Effects:
Use a Tree-sitter query to find all useEffect calls.
Analyze the AST of the callback function passed to useEffect.
Identify calls that create persistent resources, such as window.addEventListener, setInterval, or new WebSocket(...).
Check if the useEffect callback has a return statement that returns a function. This returned function is the cleanup function.
If a resource-creating call is found and no cleanup function is returned, flag a potential memory leak.
Tree-sitter Query for Missing useEffect Cleanup (JavaScript/React):
Scheme
; Find useEffect calls that add an event listener but do not return a cleanup function
(call_expression
  function: (identifier) @hook_name (#eq? @hook_name "useEffect")
  arguments: (arguments
    (arrow_function
      body: (statement_block
        ; Find a resource-creating call inside
        (expression_statement
          (call_expression
            function: (member_expression
              property: (property_identifier) @method_name
              (#any-of? @method_name "addEventListener" "setInterval")
            )
          )
        )
        ; And crucially, ensure there is no return statement in the block
       !((return_statement))
      )
    )
  )
) @leak_in_use_effect


By flagging these patterns, Uveddi can help frontend developers adhere to best practices, preventing subtle memory leaks and improving the separation between UI and logic.

Part III: Tight Coupling Detector: Implementation Strategies

Tight coupling is an architectural anti-pattern where components are highly interdependent, making the system difficult to maintain, test, and evolve. A change in one component is likely to cause ripple effects in many others. This detector quantifies coupling using established software metrics and identifies problematic areas such as cyclic dependencies.

Section 6: Core Coupling Metrics and Algorithms

The foundation of the tight coupling detector is a set of well-defined metrics that can be calculated from the dependency graph.

6.1 Calculating Fan-in and Fan-out

Fan-in and Fan-out are the simplest and most direct measures of a component's connectivity.
Definitions 48:
Fan-in: For a given module or class M, Fan-in is the count of other modules that depend on M. It measures responsibility and reuse. A high Fan-in indicates that M is a central, heavily relied-upon component.
Fan-out: For a given module or class M, Fan-out is the count of other modules that M depends on. It measures dependency and complexity. A high Fan-out suggests that M has many external dependencies and may be complex to test and maintain.
Implementation with petgraph: These metrics are trivial to compute from the dependency graph. For a given NodeIndex n:
Fan-in: graph.neighbors_directed(n, petgraph::Direction::Incoming).count()
Fan-out: graph.neighbors_directed(n, petgraph::Direction::Outgoing).count()
Performance: The calculation for all nodes can be efficiently parallelized using rayon. The list of all NodeIndex values can be processed by a parallel iterator, with each thread computing the metrics for a subset of the nodes and storing the results in a concurrent map or a final collection.

6.2 Implementing Coupling Between Objects (CBO) and Response For a Class (RFC)

CBO and RFC are two of the six original Chidamber and Kemerer object-oriented metrics, providing deeper insight into class-level coupling.51
Coupling Between Objects (CBO):
Definition: For a class C, CBO is a count of the number of other classes to which C is coupled. A class A is coupled to class B if A uses methods or instance variables of B, or if B uses methods or instance variables of A. Inheritance also establishes coupling.51 Essentially, it measures the number of unique classes a given class directly interacts with.
Implementation:
For each node n representing a class in the dependency graph:
Initialize an empty HashSet<NodeIndex> to store unique coupled classes.
Iterate through all incoming neighbors (dependencies on n) and add their indices to the set.
Iterate through all outgoing neighbors (dependencies of n) and add their indices to the set.
The CBO for class n is the final size of the HashSet. Using a set automatically handles the requirement that each coupled class is counted only once, even if there are multiple dependency edges.55
Response For a Class (RFC):
Definition: For a class C, RFC is the total number of methods that can be executed in response to a message sent to an object of that class. This is the sum of the number of methods defined within C itself (local methods) and the number of distinct methods in other classes that are called by C's local methods (remote methods).56 A high RFC indicates high complexity, making the class difficult to test and debug.
Implementation: Calculating RFC requires a more detailed call graph, where nodes represent individual methods and edges represent method calls.
Construct this method-level call graph using Tree-sitter queries to identify method definitions and call sites.
For each class C:
Identify the set of local methods, M_local, defined within C.
Initialize an empty HashSet for remote methods, M_remote.
For each method m in M_local, traverse its outgoing edges in the call graph. For each called method m_called that belongs to a different class, add m_called to the M_remote set.
The RFC for class C is calculated as RFC = |M_local| + |M_remote|.

6.3 Efficient Cyclic Dependency Detection using Depth-First Search

A cyclic dependency, where module A depends on B, and B depends back on A (either directly or through a longer chain), is a severe architectural flaw. It makes the system impossible to reason about, test in isolation, and often leads to compilation or runtime errors.60
Algorithm: The canonical algorithm for cycle detection in a directed graph is a Depth-First Search (DFS) traversal that maintains the state of each node using a three-color system.61
White: The node has not been visited yet.
Gray: The node is currently being visited (i.e., it is in the current recursion stack of the DFS).
Black: The node and all its descendants have been fully visited.
A cycle is detected if the DFS traversal encounters a gray node. This indicates a "back edge" in the DFS tree, which points from a node to one of its ancestors in the current traversal path.
Rust Implementation:
While petgraph provides a convenient petgraph::algo::is_cyclic_directed function to check for the existence of a cycle, Uveddi must also report the nodes that form the cycle. This requires a custom DFS implementation.
Rust
use petgraph::graph::{Graph, NodeIndex};
use petgraph::Direction;
use std::collections::{HashMap, HashSet};

enum Color { White, Gray, Black }

/// Detects all elementary cycles in a directed graph.
pub fn find_cycles(graph: &Graph<String, ()>) -> Vec<Vec<String>> {
    let mut colors: HashMap<NodeIndex, Color> = graph.node_indices().map(|i| (i, Color::White)).collect();
    let mut path: Vec<NodeIndex> = Vec::new();
    let mut cycles = Vec::new();

    for node in graph.node_indices() {
        if let Some(Color::White) = colors.get(&node) {
            dfs_visit(graph, node, &mut colors, &mut path, &mut cycles);
        }
    }
    // Convert NodeIndex cycles to String cycles for reporting
    cycles.into_iter().map(|cycle_indices| {
        cycle_indices.iter().map(|&idx| graph[idx].clone()).collect()
    }).collect()
}

fn dfs_visit(
    graph: &Graph<String, ()>,
    u: NodeIndex,
    colors: &mut HashMap<NodeIndex, Color>,
    path: &mut Vec<NodeIndex>,
    cycles: &mut Vec<Vec<NodeIndex>>,
) {
    colors.insert(u, Color::Gray);
    path.push(u);

    for v in graph.neighbors_directed(u, Direction::Outgoing) {
        match colors.get(&v) {
            Some(Color::Gray) => {
                // Cycle detected
                if let Some(pos) = path.iter().position(|&x| x == v) {
                    cycles.push(path[pos..].to_vec());
                }
            }
            Some(Color::White) => {
                dfs_visit(graph, v, colors, path, cycles);
            }
            _ => {} // Node is Black, already visited
        }
    }

    path.pop();
    colors.insert(u, Color::Black);
}


This implementation will find and collect the specific nodes involved in each cycle, allowing Uveddi to provide actionable reports to the user.

Section 7: Language-Specific Coupling Analysis

The abstract concepts of coupling must be translated into concrete analysis rules for each target language. This requires understanding how dependencies are expressed in Rust, Python, and JavaScript at a syntactic level.

7.1 Rust: Analyzing Trait, Struct, and Module Dependencies

Rust's rich type system offers several mechanisms for coupling between components.
Modeling Dependencies:
Structs and Enums: A struct or enum definition creates dependencies on all the types used in its fields. For example, struct Foo { bar: Bar } creates a dependency from Foo to Bar.
Trait Implementations: An impl MyTrait for MyStruct block creates a dependency from MyStruct to MyTrait. This is a fundamental form of coupling in idiomatic Rust.3
Function Signatures: Parameters and return types in functions create dependencies on those types.
Trait Bounds: Generic functions with trait bounds, such as fn process<T: Serialize>(item: T), create a dependency from the function process to the trait Serialize.64
Module System: use statements create explicit dependencies between modules.4
Tree-sitter Queries for Rust:
Struct Field Types: (field_declaration type: (type_identifier) @dependency)
Trait Implementations: (impl_item trait: (type_identifier) @trait_dependency)
Function Parameters: (parameter type: (type_identifier) @param_dependency)
Generic Trait Bounds: (where_clause (where_predicate type: (_) bounded_type: (trait_bounds (type_identifier) @bound_dependency)))
These queries allow the engine to build a detailed graph that accurately reflects the coupling inherent in Rust's design patterns.

7.2 Python: Measuring Module Coupling in Hierarchical Packages

In Python, the primary unit of code organization and coupling is the module (.py file) and the package (a directory of modules).65
Modeling Dependencies: Dependencies are almost exclusively created via import and from... import... statements. The analysis must correctly resolve these imports relative to the project's root directory and any configured source paths. For example, an import from my_app.services import user_service in my_app/api/views.py creates a dependency from the views module to the user_service module.
Hierarchical Packages: A directory containing an __init__.py file is treated as a package, which can contain sub-packages and modules. The analysis engine must parse this structure to build a complete map of resolvable module paths.65
Tree-sitter Query for Python Imports:
Scheme
; Captures 'import my_module' and 'import my_module as alias'
(import_statement name: (dotted_name) @import)

; Captures 'from my_package import my_module'
(from_import_statement
  module_name: (dotted_name) @from_import_module
  name: (dotted_name) @from_import_name
)


By processing the captures from these queries, the engine can build the module-level dependency graph for the entire Python project.

7.3 JavaScript: Building and Analyzing ES Module Dependency Graphs

JavaScript has two primary module systems: ES Modules (ESM) and CommonJS. Both must be supported for comprehensive analysis.
Modeling Dependencies:
ES Modules (ESM): Dependencies are declared statically using import and export statements. Their static nature makes them particularly easy and reliable to analyze.9
CommonJS: Dependencies are loaded dynamically using the require() function. Analysis requires identifying call_expression nodes where the callee is require.
Tooling Reference: The madge CLI tool is an excellent reference for JavaScript dependency analysis. It can generate dependency graphs and detect circular dependencies for both ESM and CommonJS.8
Uveddi aims to provide similar functionality within its Rust-based engine.
Tree-sitter Queries for JavaScript:
ESM Imports:
Scheme
; Captures 'import defaultExport from "module-name";'
; and 'import { namedExport } from "module-name";'
(import_statement
  source: (string (string_fragment) @source)
)


CommonJS Requires:
Scheme
(call_expression
  function: (identifier) @require_func (#eq? @require_func "require")
  arguments: (arguments (string (string_fragment) @source))
)


These queries provide the necessary information to construct a complete dependency graph for modern JavaScript projects, forming the basis for the coupling metric calculations.

Section 8: Establishing and Applying Coupling Thresholds

Metrics are meaningless without context. Thresholds provide that context, turning raw numbers into actionable insights by flagging components that are statistical outliers or violate established best practices.

8.1 A Catalogue of Industry-Standard Thresholds

While no threshold is universally perfect, research and industry practice have converged on several widely accepted heuristic values. Uveddi should provide these as sensible defaults to make the tool immediately useful.68
Table: Recommended Default Coupling Thresholds

Metric
Low Risk (Good)
Moderate Risk (Warning)
High Risk (Critical)
Rationale / Source
CBO (Coupling Between Objects)
< 5
5 - 9
> 9
A high CBO indicates a class is overly dependent on others, harming maintainability and reuse. A limit of 9 has been shown to be an efficient predictor of software failure.70 Other sources suggest a higher limit of 14 for a critical threshold.52
RFC (Response for Class)
< 20
20 - 50
> 50
A high RFC indicates significant complexity, making a class difficult to understand, test, and debug. A threshold of 50 is a common recommendation.52
Fan-out (Module/Class)
< 7
7 - 15
> 15
A high Fan-out indicates a component has too many outgoing dependencies, making it fragile and complex. This is a general heuristic, as acceptable values are highly context-dependent.49
Cyclic Dependencies
0
N/A
> 0
Cyclic dependencies are considered a severe architectural flaw and should always be treated as a high-risk issue.60

These defaults provide a strong, evidence-based starting point for analysis.

8.2 Designing a System for User-Configurable Sensitivity

Teams must have the ability to tune these thresholds to match their project's specific context, domain complexity, and coding standards. This is achieved by allowing users to override the defaults in the uveddi.toml configuration file.
Implementation:
Ini, TOML
[thresholds]
# Users can set their own critical limits.
# Any value below this is considered acceptable.
cbo = 12
rfc = 60
fan_out = 10


This configurability is essential for the tool's practical adoption, as it allows teams to progressively tighten their quality gates as a codebase matures.

8.3 Advanced Strategy: Implementing Adaptive Thresholds

A "one-size-fits-all" threshold can be problematic. A CBO of 10 might be normal for a complex framework integration class but unacceptably high for a simple data object. Adaptive thresholds solve this by deriving limits from the statistical properties of the project being analyzed.75
Concept: Instead of comparing a metric value to a fixed number, compare it to the distribution of values from the entire project. A component is flagged if it is a statistical outlier (e.g., in the top 10% of most coupled components).
Implementation Strategy:
First, calculate the desired metric (e.g., CBO) for every class in the project.
Collect these values into a list and compute descriptive statistics: mean, standard deviation, and percentiles (e.g., 75th, 80th, 90th).
The threshold is then set based on this distribution. For example, the warning threshold could be the 75th percentile and the critical threshold could be the 90th percentile.
This can be exposed to the user via a special configuration syntax:
Ini, TOML
[thresholds]
# Use a fixed value
cbo = 12

# Or use an adaptive percentile-based threshold
rfc = "p90" # Flag anything in the top 10% as critical


Benefits: This approach makes the detector sensitive to the specific context of the codebase. It automatically adjusts for project size and complexity, reducing false positives on complex projects and successfully identifying outliers on simpler ones. This method is supported by academic research aiming to derive more reliable, data-driven thresholds.69

Part IV: Visualization and Reporting

The final component of the analysis tool is its output. Raw data and lists of violations are useful, but visual representations of the dependency structure provide a much richer and more intuitive understanding of the system's architecture.

Section 9: Integrating Dependency Visualization with Mermaid.js

Mermaid.js is a lightweight, JavaScript-based diagramming tool that uses a Markdown-inspired syntax. Its simplicity and excellent integration with web-based documentation and platforms like GitHub make it a superior choice over older formats like DOT/Graphviz for modern development workflows.78

9.1 Generating Mermaid.js Syntax from petgraph Graphs

The core of the integration is a Rust function that traverses the petgraph graph and emits a valid Mermaid.js flowchart string.
Process: A function fn to_mermaid(graph: &Graph<...>, cycles: &[Vec<NodeIndex>]) -> String will perform the conversion.
Implementation Steps:
Initialize a String buffer, starting with the graph orientation, e.g., graph TD;\n for a top-down layout.81
Create a HashSet containing all NodeIndex values that are part of a detected cycle for efficient styling lookups.
Node Definitions: Iterate through all nodes in the petgraph::Graph. For each NodeIndex and its associated data (e.g., the CodeEntity struct), format a Mermaid node definition. The node ID can be derived from the NodeIndex (e.g., n0, n1), and the label should be the human-readable name of the entity. Append this to the buffer (e.g., n0["src/domain/user.rs::User"];\n).
Edge Definitions: Iterate through all edges in the petgraph::Graph. For each edge from u to v, append a line defining the link: n{u_index} --> n{v_index};\n.
Cycle Highlighting: Iterate through the set of cyclic nodes. For each cyclic node c, append a style directive to color it red, making it visually prominent. This is a key feature for drawing attention to architectural problems.82 For example:
style n{c_index} fill:#ffcccc,stroke:#cc0000,stroke-width:2px;\n.
Metric-Based Highlighting: Similarly, apply different styles for nodes that exceed warning or critical thresholds for metrics like CBO or RFC, using colors like orange or yellow.
This direct generation approach provides complete control over the final diagram's appearance, which is more flexible than relying on intermediate crates that may have limited styling options.83

9.2 Techniques for Visualizing High-Density Graphs

Visualizing the entire dependency graph of a large project will produce an unreadable "hairball" diagram. The tool must provide mechanisms to manage this complexity.
Sub-graphing and Focus: The CLI should allow the user to specify a "focus" entity (e.g., a class or module name). The generated diagram would then only include that entity, its direct dependencies (incoming), and its direct dependents (outgoing). An optional --depth flag could control how many levels of transitive dependencies to show.
Module/Package Collapsing: For a higher-level view, nodes can be grouped into subgraph blocks in Mermaid syntax. All entities within the same module or package would be rendered inside a bounding box, and dependencies between these groups would be drawn as edges between the subgraphs. This allows users to understand the high-level component architecture before drilling into specific classes.
Code snippet
graph TD
    subgraph "Domain Layer"
        n1["User"]
        n2["Order"]
    end
    subgraph "Application Layer"
        n3
    end
    n3 --> n1



9.3 Highlighting Cyclic Dependencies and Problematic Nodes

Visual feedback is the fastest way to communicate the results of the analysis. As described previously, using Mermaid's styling capabilities is essential for creating an intuitive and actionable report.
Color-Coding Scheme: A consistent color code should be used across all visualizations.
Red: Nodes involved in a cyclic dependency. This is the highest severity issue and should be immediately obvious.82
Orange: Nodes that exceed a "critical" threshold for a coupling metric (e.g., CBO > 14).
Yellow: Nodes that exceed a "warning" threshold (e.g., 9 < CBO <= 14).
Blue/Default: Healthy nodes that are within acceptable limits.
This visual language transforms the dependency graph from a simple structural diagram into a rich architectural health report, enabling developers to quickly identify and prioritize areas for refactoring.74

Conclusions and Recommendations

This report has provided a comprehensive technical blueprint for implementing two critical architectural anti-pattern detectors—Leaky Abstraction and Tight Coupling—within the Uveddi static analysis tool. The successful implementation of these features will significantly enhance Uveddi's ability to provide deep, actionable insights into the quality and maintainability of Rust, Python, and JavaScript codebases.
Key Recommendations for Implementation:
Prioritize the Foundational Graph Infrastructure: The success of both detectors hinges on a robust, multi-language dependency graph. The two-stage process of parallel parsing followed by synchronized linking is the recommended architecture. petgraph::Graph should be used initially for its flexibility, with a clear path to migrate to petgraph::Csr for memory optimization on very large projects.
Embrace a Parallel-First Design: Leverage rayon aggressively for file parsing and metric calculations. Designing for parallelism from the outset will ensure the tool remains performant and scalable as its feature set grows.
Use Rule-Based Engines for Intentional Architectures: For architectural layer validation, a user-configurable, rule-based engine is the correct approach. It provides the determinism and transparency required to enforce a planned architecture, a task for which machine learning is ill-suited.
Focus on Actionable, Statically-Detectable Patterns: For performance leaks like N+1 queries and unawaited async tasks, the detectors should focus on precise Tree-sitter queries that identify well-known anti-patterns. The reporting should be clear about what is being detected—a risky pattern, not necessarily a proven runtime bug.
Implement a Full Suite of Coupling Metrics: The Tight Coupling detector should compute Fan-in/Fan-out, CBO, and RFC. Crucially, it must include a robust, DFS-based cycle detection algorithm that not only identifies the presence of cycles but also reports the specific components involved.
Provide Both Static and Adaptive Thresholds: Offer sensible, research-backed default thresholds to make the tool immediately useful. However, the inclusion of user-configurable and, most importantly, adaptive (percentile-based) thresholds will be a key differentiator, allowing the tool to intelligently adapt to the unique characteristics of each project.
Leverage Visualization for Impact: A text-based list of violations is necessary, but a visual dependency graph generated with Mermaid.js is invaluable for intuitive understanding. The visualization must use color-coding to highlight cycles and high-coupling nodes, and provide features like sub-graphing to manage complexity.
By following these specifications, the Uveddi team can build a powerful, state-of-the-art static analysis tool that helps developers write cleaner, more maintainable, and more robust software. The focus on performance, language-specific nuance, and actionable reporting will position Uveddi as a critical component in any modern software development lifecycle.
Works cited
rust-code-analysis - crates.io: Rust Package Registry, accessed July 6, 2025, https://crates.io/crates/rust-code-analysis/0.0.21/dependencies
tree_sitter - Rust - Docs.rs, accessed July 6, 2025, https://docs.rs/tree-sitter
Confusion : Struct, impl, self, trait - help - The Rust Programming Language Forum, accessed July 6, 2025, https://users.rust-lang.org/t/confusion-struct-impl-self-trait/3941
Idiomatic project structure in Rust (cry for help from a messy directory) - Reddit, accessed July 6, 2025, https://www.reddit.com/r/rust/comments/mjm3he/idiomatic_project_structure_in_rust_cry_for_help/
Parse and analyze source codes with Tree-sitter - LINCS, accessed July 6, 2025, https://www.lincs.fr/events/parse-and-analyze-source-codes-with-tree-sitter/
Using tree-sitter with Python - Simon Willison: TIL, accessed July 6, 2025, https://til.simonwillison.net/python/tree-sitter
nvim-treesitter - how to highlight imports/modules text : r/neovim - Reddit, accessed July 6, 2025, https://www.reddit.com/r/neovim/comments/18pgv70/nvimtreesitter_how_to_highlight_importsmodules/
pahen/madge: Create graphs from your CommonJS, AMD or ES6 module dependencies - GitHub, accessed July 6, 2025, https://github.com/pahen/madge
How ES Modules and CommonJS Handle Dependencies Differently — And Why It Matters, accessed July 6, 2025, https://itamar-toledano.medium.com/how-es-modules-and-commonjs-handle-dependencies-differently-and-why-it-matters-244ef29630ce
petgraph - Rust - Docs.rs, accessed July 6, 2025, https://docs.rs/petgraph/
petgraph - Rust - Shadow, accessed July 6, 2025, https://shadow.github.io/docs/rust/petgraph/index.html
Graphs in Rust: An Introduction to Petgraph | Depth-First, accessed July 6, 2025, https://depth-first.com/articles/2020/02/03/graphs-in-rust-an-introduction-to-petgraph/
petgraph/petgraph: Graph data structure library for Rust. - GitHub, accessed July 6, 2025, https://github.com/petgraph/petgraph
Graphs in Memory - The Rust Programming Language Forum, accessed July 6, 2025, https://users.rust-lang.org/t/graphs-in-memory/18414
petgraph - Rust Package Registry - Crates.io, accessed July 6, 2025, https://crates.io/crates/petgraph/dependencies
Graph Algorithms: From Theory to Optimization (Examples in Rust) - Medium, accessed July 6, 2025, https://medium.com/@jordangrilly/graph-algorithms-from-theory-to-optimization-examples-in-rust-aa4ad2734255
Tree-sitter: Introduction, accessed July 6, 2025, https://tree-sitter.github.io/
Basic Syntax - Tree-sitter, accessed July 6, 2025, https://tree-sitter.github.io/tree-sitter/using-parsers/queries/1-syntax.html
Speeding up data analysis with Rayon and Rust - The Data Quarry, accessed July 6, 2025, https://thedataquarry.com/blog/intro-to-rayon/
Parallelizing rustc using Rayon - compiler - Rust Internals, accessed July 6, 2025, https://internals.rust-lang.org/t/parallelizing-rustc-using-rayon/6606
Knee Deep in tree-sitter Queries - Hackerman's Hacking Tutorials, accessed July 6, 2025, https://parsiya.net/blog/knee-deep-tree-sitter-queries/
Unraveling Tree-Sitter Queries: Your Guide to Code Analysis Magic - DEV Community, accessed July 6, 2025, https://dev.to/shrsv/unraveling-tree-sitter-queries-your-guide-to-code-analysis-magic-41il
Learn How to Navigate Code Structures and Extract Details Using Tree-sitter - Medium, accessed July 6, 2025, https://medium.com/@rijulrajtkeey2/learn-how-to-navigate-code-structures-and-extract-details-using-tree-sitter-510d6d2801f1
Query — py-tree-sitter 0.24.0 documentation, accessed July 6, 2025, https://tree-sitter.github.io/py-tree-sitter/classes/tree_sitter.Query.html
Heuristics for Discovering Architectural Violations - RMOD Files - Inria, accessed July 6, 2025, https://rmod-files.lille.inria.fr/Team/Texts/Papers/Maff13b-HeuristicsArchitecturalViolations-WCRE13.pdf
Warnings: Violation Symptoms Indicating Architecture Erosion - arXiv, accessed July 6, 2025, https://arxiv.org/pdf/2212.12168
Rule-Based Vs. Machine Learning AI: Which Produces Better Results? | Pecan AI, accessed July 6, 2025, https://www.pecan.ai/blog/rule-based-vs-machine-learning-ai-which-produces-better-results/
Rule-Based vs. LLM-Based AI Agents: A Side-by-Side Comparison - TeckNexus, accessed July 6, 2025, https://tecknexus.com/rule-based-vs-llm-based-ai-agents-a-side-by-side-comparison/
Can a machine learning model completely replace a rules based system? - Quora, accessed July 6, 2025, https://www.quora.com/Can-a-machine-learning-model-completely-replace-a-rules-based-system
Solving N+1 Query Problem with Distributed Tracing - Edge Delta, accessed July 6, 2025, https://edgedelta.com/company/blog/how-can-distributed-tracing-solve-n1-query-problem
How to Detect n+1 Queries in PHP - Laravel News, accessed July 6, 2025, https://laravel-news.com/how-to-detect-n1-queries-in-php
Understanding N+1 Database Queries - Scout APM, accessed July 6, 2025, https://www.scoutapm.com/blog/understanding-n1-database-queries
Finding and optimizing N+1 queries on a relational database | by Gonzalo Lopez | Mixpanel Engineering | Medium, accessed July 6, 2025, https://medium.com/mixpaneleng/finding-and-optimizing-n-1-queries-on-a-relational-database-1ce02f1f2ffb
Solving N+1 Query Problems in Go Applications - Go Prisma, accessed July 6, 2025, https://goprisma.org/blog/solving-n1-query-problems-in-go-applications
The State of Async Rust: Runtimes, accessed July 6, 2025, https://corrode.dev/blog/async/
How do you collect/track spawned tasks to make sure they are all complete before termination? : r/rust - Reddit, accessed July 6, 2025, https://www.reddit.com/r/rust/comments/177qrmj/how_do_you_collecttrack_spawned_tasks_to_make/
Tokio TaskTracker not awaiting for spawned futures to complete - Rust Users Forum, accessed July 6, 2025, https://users.rust-lang.org/t/tokio-tasktracker-not-awaiting-for-spawned-futures-to-complete/110139
Can static analysis detect memory leaks? - Stack Overflow, accessed July 6, 2025, https://stackoverflow.com/questions/37119336/can-static-analysis-detect-memory-leaks
How to make Rust leak memory (also: how to make it stop) · The Fly ..., accessed July 6, 2025, https://fly.io/blog/rust-memory-leak/
Don't Block the Event Loop (or the Worker Pool) - Node.js, accessed July 6, 2025, https://nodejs.org/en/learn/asynchronous-work/dont-block-the-event-loop
Django vs Flask: Which Python Framework to Choose in 2025? - MindInventory, accessed July 6, 2025, https://www.mindinventory.com/blog/djnago-vs-flask/
Django vs. Flask in 2024: Which Framework to Choose | TestDriven.io, accessed July 6, 2025, https://testdriven.io/blog/django-vs-flask/
python - How to build a pep8 like static analysis tool for Django ..., accessed July 6, 2025, https://stackoverflow.com/questions/10175011/how-to-build-a-pep8-like-static-analysis-tool-for-django
Path To A Clean(er) React Architecture (Part 6) - Business Logic Separation - Reddit, accessed July 6, 2025, https://www.reddit.com/r/reactjs/comments/1dl2scn/path_to_a_cleaner_react_architecture_part_6/
How to identify and fix memory leaks in react - DEV Community, accessed July 6, 2025, https://dev.to/emmanuelo/how-to-identify-and-fix-memory-leaks-in-react-3bbh
Decoding Memory Leaks in React JS: Strategies for Efficient Development, accessed July 6, 2025, https://suvra1.medium.com/decoding-memory-leaks-in-react-js-strategies-for-efficient-development-049aa5d2fe46
Understanding Memory Leaks in React: How to Find and Fix Them - Medium, accessed July 6, 2025, https://medium.com/@ignatovich.dm/understanding-memory-leaks-in-react-how-to-find-and-fix-them-fc782cf182be
How do I measure FAN-OUT/FAN-IN? - ResearchGate, accessed July 6, 2025, https://www.researchgate.net/post/How-do-I-measure-FAN-OUT-FAN-IN
Software Process & Project Management, accessed July 6, 2025, https://www.se.rit.edu/~swen-256/slides/SWEN256-13-MeasurementMetrics.pdf
Project Metrics Help - Structural Fan-In and Fan-Out metrics - Aivosto, accessed July 6, 2025, https://www.aivosto.com/project/help/pm-sf.html
CMS Assessment Model - Information - CAST Enforce Object Oriented Metrics - Chidamber and Kemerer Metrics Suite - CAST Documentation, accessed July 6, 2025, https://doc.castsoftware.com/export/TG/CMS+Assessment+Model+-+Information+-+CAST+Enforce+Object+Oriented+Metrics+-+Chidamber+and+Kemerer+Metrics+Suite
Project Metrics Help - Chidamber & Kemerer object-oriented metrics suite - Aivosto, accessed July 6, 2025, https://www.aivosto.com/project/help/pm-oo-ck.html
Mastering Coupling Between Objects - Number Analytics, accessed July 6, 2025, https://www.numberanalytics.com/blog/mastering-coupling-between-objects
Coupling Metrics for Object-Oriented Design -
CBO coupling between object - Stack Overflow, accessed July 6, 2025, https://stackoverflow.com/questions/27515541/cbo-coupling-between-object
The Response for Class metric - Sonar Code Quality Testing Essentials [Book], accessed July 6, 2025, https://www.oreilly.com/library/view/sonar-code-quality/9781849517867/ch09s04.html
Response for Class (RFC) - Code Quality Docs, accessed July 6, 2025, https://docs.embold.io/response-for-class/
Response for Class | objectscriptQuality, accessed July 6, 2025, https://objectscriptquality.com/docs/metrics/response-for-class
Analizo::Metric::ResponseForClass - Response for Class (RFC) metric - Ubuntu Manpage, accessed July 6, 2025, https://manpages.ubuntu.com/manpages//noble/man3/Analizo::Metric::ResponseForClass.3pm.html
Dependency graph - Wikipedia, accessed July 6, 2025, https://en.wikipedia.org/wiki/Dependency_graph
Detecting Cycles and Ordering Dependencies: Graph Algorithms in Kotlin - Medium, accessed July 6, 2025, https://medium.com/@chetanshingare2991/detecting-cycles-and-ordering-dependencies-graph-algorithms-in-kotlin-a3807cf8a57c
Detecting Cycles in a Graph - Algorithms and Data Structures - Profound Academy, accessed July 6, 2025, https://profound.academy/algorithms-data-structures/detecting-cycles-in-a-graph-Dm0rFXWv4Z1skpv90djA
Detect Cycle in a Directed Graph - GeeksforGeeks, accessed July 6, 2025, https://www.geeksforgeeks.org/dsa/detect-cycle-in-a-graph/
Rust Traits: Defining Behavior | Matt Oswalt, accessed July 6, 2025, https://oswalt.dev/2020/07/rust-traits-defining-behavior/
How to structure Python module hierarchy - LabEx, accessed July 6, 2025, https://labex.io/tutorials/python-how-to-structure-python-module-hierarchy-425421
Python Code Hierarchy - Swetha Ganapathi Raman - Medium, accessed July 6, 2025, https://swethag04.medium.com/softwarepython-code-hierarchy-0a80093344be
Structuring Python code: Modules and Packages - IN3110/4110: Higher Level Programming, accessed July 6, 2025, https://uio-in3110.github.io/lectures/python/packages_and_testing.html
Techniques for Calculating Software Product Metrics Threshold Values: A Systematic Mapping Study - MDPI, accessed July 6, 2025, https://www.mdpi.com/2076-3417/11/23/11377
(PDF) Techniques for Calculating Software Product Metrics Threshold Values: A Systematic Mapping Study - ResearchGate, accessed July 6, 2025, https://www.researchgate.net/publication/356760165_Techniques_for_Calculating_Software_Product_Metrics_Threshold_Values_A_Systematic_Mapping_Study
Code metrics - Class coupling - Visual Studio (Windows) | Microsoft Learn, accessed July 6, 2025, https://learn.microsoft.com/en-us/visualstudio/code-quality/code-metrics-class-coupling?view=vs-2022
Chidamber & Kemerer object-oriented metrics suite, accessed July 6, 2025, https://people.scs.carleton.ca/~jeanpier/sharedF14/T1/extra%20stuff/about%20metrics/Chidamber%20&%20Kemerer%20object-oriented%20metrics%20suite.pdf
Optimizing Class Response - Number Analytics, accessed July 6, 2025, https://www.numberanalytics.com/blog/optimizing-class-response-software-metrics
Vovel metrics—novel coupling metrics for improved software fault prediction - PMC, accessed July 6, 2025, https://pmc.ncbi.nlm.nih.gov/articles/PMC8205299/
Dependency Diagrams: Leverage and Understand Graphs - Devzery, accessed July 6, 2025, https://www.devzery.com/post/dependency-diagrams
What Is Adaptive Thresholding? - Splunk, accessed July 6, 2025, https://www.splunk.com/en_us/blog/learn/adaptive-thresholding.html
Evaluating Thresholds for Object-Oriented Software Metrics - DCC::LLP, accessed July 6, 2025, https://llp.dcc.ufmg.br/Publications/Journal2024/Filo-et-al-2024-JBCS-paper.pdf
Predicting Relative Thresholds for Object Oriented Metrics - arXiv, accessed July 6, 2025, https://arxiv.org/pdf/2103.11442
Mermaid | Diagramming and charting tool, accessed July 6, 2025, https://mermaid.js.org/
mermaid-js/mermaid: Generation of diagrams like flowcharts or sequence diagrams from text in a similar manner as markdown - GitHub, accessed July 6, 2025, https://github.com/mermaid-js/mermaid
Mermaid User Guide, accessed July 6, 2025, https://mermaid.js.org/intro/getting-started.html
Flowcharts – Basic Syntax - Mermaid Chart, accessed July 6, 2025, https://docs.mermaidchart.com/mermaid-oss/syntax/flowchart.html
How to Detect Circular Dependencies with Degraph, accessed July 6, 2025, https://stevenschwenke.de/howToDetectCircularDependenciesWithDegraph
simple_mermaid - Rust - Docs.rs, accessed July 6, 2025, https://docs.rs/simple-mermaid
mermaid-rs - Rust Package Registry - Crates.io, accessed July 6, 2025, https://crates.io/crates/mermaid-rs
Dependency Graph Template: Visualize Complex Relationships - MyMap.AI, accessed July 6, 2025, https://www.mymap.ai/template/dependency-graph
