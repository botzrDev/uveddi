
Cross-File Semantic Analysis for the Detection of Leaky Abstractions: A Technical Blueprint


Foundations of Code Representation: The Semantic Dependency Graph

The detection of subtle software defects, particularly those that span architectural boundaries like leaky abstractions, necessitates a departure from traditional static analysis techniques that rely on purely syntactic representations of source code. An effective analysis must be grounded in a comprehensive understanding of the program's semantics—its meaning, its behavior, and the intricate web of dependencies that connect its components. This section establishes the theoretical and practical foundations for such an analysis by proposing a rich, graph-based code representation: the Semantic Dependency Graph. We will explore the limitations of simpler models, survey the landscape of advanced code graphs, and define a detailed schema that is capable of capturing the nuanced relationships required for sophisticated, cross-file semantic analysis.

Beyond Syntax: The Need for Semantic Representation

Static analysis is broadly defined as the process of analyzing source code without executing it.1 This process can be bifurcated into two distinct categories: syntactic analysis and semantic analysis. Syntactic analysis is concerned with the structure and form of the code, verifying that it adheres to the grammatical rules of the programming language. For example, a compiler performing syntactic analysis might flag an error for a "missing semicolon".1 While essential for ensuring a program is well-formed, this level of analysis is fundamentally incapable of understanding the program's intent or logic.
Semantic analysis, in contrast, aims to compute or estimate the meaning of the source code.1 An analysis that reports an "unused variable" is performing a semantic check; it has inferred that despite the variable being syntactically correct, it serves no logical purpose in the program's data flow. The problem of detecting leaky abstractions falls squarely within the domain of semantic analysis. A leaky abstraction is not a syntactic error but a violation of design intent, where a high-level component is improperly exposed to the implementation details of a lower-level component it depends on.3
To detect such violations, the analysis must operate on a whole-program view. The meaning of a function call in one file is incomplete without knowledge of the function's definition, which may reside in an entirely different file or even a pre-compiled library. An analysis confined to a single file operates with a profound lack of context, rendering it blind to the cross-boundary dependencies that define a program's architecture. Therefore, the cornerstone of any system designed to find leaky abstractions is a data structure that can represent the entire codebase as a single, interconnected semantic entity.

A Comparative Overview of Code Graphs: From Call Graphs to Code Property Graphs (CPGs)

The representation of software as a graph is a foundational concept in program analysis.5 However, not all graph models are created equal. The expressive power of the analysis is directly constrained by the richness of the underlying graph representation. The journey from simple structural graphs to comprehensive semantic graphs illustrates a trade-off between ease of construction and analytical capability.
Call Graphs (CGs) and Class Collaboration Networks (CCNs): These are among the simplest and most common graph representations. A Call Graph represents functions or methods as nodes and function calls as directed edges.7 A Class Collaboration Network operates at a higher level of abstraction, with nodes representing classes and edges representing any form of interaction (e.g., method calls, field access) between them. While useful for high-level visualization and understanding which components interact, these models are semantically poor. They capture the "who calls whom" relationship but omit the crucial context of
how and why—for instance, what data is passed, under what conditions the call is made, or how data flows back to the caller.8
Program Dependence Graphs (PDGs): A significant advancement over Call Graphs, the Program Dependence Graph introduces a finer granularity. In a PDG, nodes represent individual program statements and expressions, and edges represent fundamental dependencies between them.7 There are two primary types of edges:
Data Dependency Edges: These connect a statement that defines a variable to a statement that uses that variable.
Control Dependency Edges: These connect a control predicate (e.g., the condition of an if statement) to the statements whose execution is governed by that predicate.
By capturing these dependencies, PDGs enable more sophisticated analyses like program slicing and data flow tracking, which are prerequisites for detecting many classes of bugs and vulnerabilities.9
Semantic Code Graph (SCG): Recent academic work has proposed the Semantic Code Graph as a model specifically designed to bridge the gap between abstract dependency models and practical software comprehension tools.7 The SCG is a multi-level graph that can represent dependencies between entities at various granularities, from classes and methods down to individual value declarations, while maintaining a direct link back to the source code location of each node and edge.7 Empirical studies have shown that the SCG is a more comprehensive model than both CGs and CCNs, enabling analyses that are not possible with the simpler models.7
Code Property Graph (CPG): The Code Property Graph is a powerful and widely adopted concept that unifies multiple program representations into a single, queryable data structure.10 A CPG integrates the Abstract Syntax Tree (AST), the Control Flow Graph (CFG), and the Program Dependence Graph (PDG). By overlaying these different graph types, a CPG allows for queries that fluidly traverse syntactic structure, control flow paths, and data dependencies simultaneously. For example, one could query for "all variable usages (
Data Flow) that occur within a specific if branch (Control Flow) of a particular function defined by a certain syntax (AST)." This holistic view is exceptionally well-suited for complex security and architectural analyses.11
Knowledge Graphs: The most recent evolution of this concept is the Code Knowledge Graph.5 This approach extends the CPG by enriching it with additional metadata and external knowledge. Nodes and edges can be augmented with information from documentation, version control history, API specifications, and even insights generated by Large Language Models (LLMs).13 The goal is to create a complete, context-aware digital twin of the codebase, enabling highly sophisticated, semantic queries about its structure, behavior, and history.12
For the purpose of detecting leaky abstractions, a model with the richness of a CPG or SCG is not merely an enhancement; it is a fundamental requirement. The analysis is only as powerful as the information encoded in its underlying graph. While the implementation complexity increases with the richness of the model, the analytical payoff is a prerequisite for success.

Designing a Rich Semantic Graph: A Detailed Schema

To move from theory to implementation, a concrete and comprehensive schema for the Semantic Dependency Graph is required. A well-defined schema serves as a formal contract, dictating the output of the parsing and symbol resolution phases and defining the "language" available to the analysis queries. This separation of concerns—defining the data model independently of the logic that creates or consumes it—is a hallmark of a robust and maintainable analysis tool. The following schema, inspired by the detailed models presented in academic research and industry tools 5, provides a robust blueprint.
Node Schema: Nodes represent the entities within the codebase. Every node should possess a set of common attributes: a unique identifier, a kind or type label to denote the entity it represents, a collection of specific properties, and precise source location information to link it back to the code.7
Core Entities:
File: Represents a single source file.
Module or Package: A higher-level grouping of code, such as a Python module or a Java package.
Class / Interface / Struct: Represents user-defined composite types.
Function / Method: Represents a callable block of code.
Argument: Represents a formal parameter of a function or method.
Variable: Represents a named storage location (e.g., local variable, global variable, class field).
Statement: Represents an individual executable statement (e.g., assignment, if_statement).
Edge Schema: Edges represent the manifold relationships between these entities. Every edge connects a source node to a target node and is labeled with a type that defines the nature of the relationship. Edges can also have properties, such as the source code location where the relationship is established.7
Relationship Types:
Structural Relationships: CONTAINS (e.g., (:Module)-->(:Class)), DEFINED_IN (e.g., (:Function)-->(:File)).
Object-Oriented Relationships: INHERITS_FROM, IMPLEMENTS.
Behavioral Relationships: CALLS (from one function to another), HAS_ARGUMENT (from a function to its arguments).
Data-Centric Relationships: DECLARES (from a scope like a function to a variable), USES (from a statement to the variable it reads), ASSIGNS (from a statement to the variable it writes to).
Module-Level Relationships: DEPENDS_ON or IMPORTS (from one module to another).
Flow Relationships: REACHES (data-flow edge from a definition to a use), CONTROLS (control-flow edge from a predicate to a controlled statement).
The following table consolidates this schema into a practical reference for implementation.
Table 1: Semantic Graph Node and Edge Schema
Category
Type
Name/Label
Attributes / Properties
Example
Node
Entity
File
name: string, path: string, size: int
A node for utils.js.


Entity
Module
name: string, path: string
A node for the java.util package.


Entity
Class
name: string, access_modifier: string, is_abstract: bool
A node for public class MyService.


Entity
Function
name: string, return_type: string, complexity: int
A node for int calculate(int x).


Entity
Variable
name: string, type: string, scope: string
A node for a local variable int count = 0;.


Entity
Argument
name: string, type: string, default_value: string
A node for the int x parameter.
Edge
Structural
CONTAINS
-
(:Module)-->(:Class)


Structural
DEFINED_IN
location: {uri, start, end}
(:Function)-->(:File)


OO
INHERITS_FROM
-
(:ClassA)-->(:ClassB)


Behavioral
CALLS
location: {uri, start, end}
(:FunctionA)-->(:FunctionB)


Data
USES
location: {uri, start, end}
(:Statement)-->(:Variable)


Data
ASSIGNS
location: {uri, start, end}
(:Statement)-->(:Variable)


Module
DEPENDS_ON
-
(:ModuleA)-->(:ModuleB)

This schema provides a powerful and extensible foundation. It is detailed enough to capture the semantic nuances needed for leak detection while being structured enough to be systematically constructed and queried.

Modeling Dependencies: Data, Control, and Type

The edges in the Semantic Dependency Graph are not arbitrary; they represent fundamental dependency types rooted in compiler theory.16 Accurately modeling these dependencies is the key to capturing the program's true behavior.
Data Dependencies: These describe how data flows through a program and are critical for understanding how the value of one statement can affect another.16 There are three canonical types:
Flow Dependency (Read-after-Write, RAW): This is the most intuitive dependency. A statement S2 has a flow dependency on S1 if S1 writes to a variable that S2 subsequently reads. This establishes a direct data-flow path.
Anti-dependence (Write-after-Read, WAR): This occurs when a statement S2 writes to a variable that a preceding statement S1 reads. Reordering these statements would cause S1 to read the wrong value. This is crucial for preserving program semantics, especially in parallel execution contexts.16
Output Dependency (Write-after-Write, WAW): This exists when two statements, S1 and S2, both write to the same variable. The order of their execution determines the final value of that variable.16
Control Dependencies: These arise from the conditional control flow of a program.16 A statement
S2 is control-dependent on a predicate S1 (e.g., an if condition) if the outcome of S1 determines whether S2 will be executed at all.7 For example, all statements within an
if block are control-dependent on the if condition. This is essential for understanding under what circumstances a piece of code is reachable.
Type Dependencies: These are implicit in statically typed languages. A variable declaration like MyClass x; creates a dependency from the variable x to the definition of the type MyClass. Similarly, a function signature public MyClass doSomething() creates a dependency from the function to the MyClass type. Tracing these dependencies is fundamental to type checking and symbol resolution across files.
By meticulously modeling these dependency types as distinct edge labels in the graph, the analysis tool gains the ability to ask precise questions about the program's structure and behavior, laying the groundwork for the detection of complex architectural flaws.

Building the Graph: From Source Code to a Global Semantic View

Constructing a comprehensive, cross-file Semantic Dependency Graph is a multi-phase process that transforms disconnected source files into a single, coherent model of the entire program. This process moves from the purely syntactic realm of parsing individual files to the deeply semantic challenge of resolving symbols and linking dependencies across the entire codebase. This section details a robust pipeline for this transformation, emphasizing the distinct roles of parsing tools like Tree-sitter and the custom logic required for true semantic analysis.

Phase 1: Parsing with Tree-sitter

The initial stage of the pipeline involves converting the raw text of each source file into a structured representation. For this task, a modern parser generator like Tree-sitter is an ideal choice due to its speed, robustness, and incremental nature.18
Role of Tree-sitter: Tree-sitter's primary function is to generate a parser for a given language grammar. This parser takes a source file as input and produces a Concrete Syntax Tree (CST), which is a detailed, lossless representation of the source text, including all tokens like parentheses and semicolons. For analysis purposes, this is often simplified into an Abstract Syntax Tree (AST), which captures the essential structural elements.20 Tree-sitter excels at producing these trees for
individual source files quickly and reliably, even in the presence of syntax errors.19
Generating Per-File ASTs: The first concrete step in the build pipeline is to identify all relevant source files in a project (e.g., all .java or .py files) and apply the corresponding Tree-sitter parser to each one. This results in a collection of disconnected ASTs, one for every file in the project.21
Extracting Syntactic Constructs with Queries: Tree-sitter includes a powerful query engine that uses an S-expression-based syntax to find patterns within an AST.21 This mechanism is perfect for extracting the raw materials for our graph. For a given AST, we can execute queries to find all nodes corresponding to
function_definition, class_declaration, import_statement, call_expression, and other syntactic constructs. These captured nodes are the candidates for becoming nodes and edges in our final semantic graph.
The Critical Limitation: It is imperative to understand that Tree-sitter's scope is fundamentally limited to the syntax of a single file.24 When a Tree-sitter query identifies a
call_expression node for a function foo(), it knows only that a syntactic construct representing a call to an identifier named "foo" exists at a certain location. It has no intrinsic knowledge of what foo refers to. It does not know where foo is defined, what its parameter types are, what it returns, or even if it is a function, a variable, or a class. This semantic information is non-local and cannot be derived from the AST of a single file. This is where the next phase of the pipeline becomes essential.

Phase 2: The Crux of Cross-File Analysis: Symbol Resolution

This phase addresses the core challenge of cross-file analysis: transforming the collection of isolated ASTs into a semantically linked program representation. This is achieved through symbol resolution, a process borrowed directly from compiler and linker design.26 The goal is to connect every
use of an identifier (a name) to its unique declaration.
The Global Symbol Table: The central data structure for this phase is the symbol table. In its simplest form, this can be implemented as a hash map that maps a unique, fully qualified name of a symbol (e.g., com.myproject.MyClass.myMethod) to a record containing detailed information about that symbol, such as its kind (function, class, variable), type, scope, and the file and line number of its declaration.29
A Two-Pass Resolution Strategy: A robust and widely used approach to populate and use the symbol table is to make two distinct passes over the project's ASTs:
Pass 1: Declaration Gathering. In the first pass, the system iterates through every AST in the project. Using Tree-sitter queries, it identifies all declarations of symbols—classes, functions, global variables, etc. For each declaration found, it constructs a fully qualified name and creates an entry in the global symbol table. This entry stores all relevant metadata, including a direct reference to the declaration's AST node and its source location.29 After this pass, the symbol table contains a complete map of every symbol defined within the project.
Pass 2: Usage Resolution. The system then performs a second pass, again iterating through every AST. This time, it uses queries to find every use of a symbol—a function call, a variable reference, a type annotation. For each usage, it attempts to resolve the identifier by looking it up in the now-complete global symbol table. A successful lookup links the usage AST node to the declaration AST node, effectively creating a semantic link that may cross file boundaries.
Handling Language-Specific Semantics: The logic for resolving symbols is highly language-specific. The resolver must understand the language's scoping rules (e.g., lexical scope, block scope, global scope) to correctly determine which declaration a given usage refers to.30 It must also process
import, require, or using statements, as these explicitly alter the set of symbols visible within a file's scope.
Case Study: The PMD Approach and External Dependencies: Real-world codebases are not self-contained; they rely heavily on third-party libraries. An analysis that only considers the project's own source code will fail to resolve any symbols defined in these external dependencies, leading to a massively incomplete graph and inaccurate results. The architecture of the popular static analysis tool PMD provides a practical solution to this problem. PMD requires the user to provide an "auxiliary classpath," which is a list of paths to the compiled artifacts (e.g., .jar files) that the project depends on.32 PMD then uses a bytecode inspection library like ASM to read these compiled files, extract their symbol information, and add it to its symbol table, just as it does for the source code.32 This allows PMD to resolve references to library functions and classes, creating a much more complete and accurate semantic model. Any serious analysis tool must adopt a similar strategy to handle external dependencies.
This architecture, which decouples the language-agnostic parsing from the language-specific semantic resolution, is a robust and scalable pattern. It allows the parser component to be a swappable, commodity piece (one per language), while the symbol resolver contains the custom, high-value logic of the analysis engine. This is validated by the architecture of PMD, which follows a clear pipeline: parse source code, build symbol tables, and then run further analyses like Data Flow Analysis (DFA) on the resulting enriched structure.33

Phase 3: Weaving the Graph

With the symbol resolution phase complete, the system now has all the information needed to construct the final, interconnected Semantic Dependency Graph. This phase involves instantiating the nodes and edges defined in our schema (Section 1.3).
Instantiating Nodes: The process begins by iterating through the global symbol table. For each entry, a corresponding node is created in the graph data structure (whether an in-memory adjacency list or a graph database). For example, a symbol table entry for a function results in the creation of a Function node in the graph. All the metadata stored in the symbol table record—such as name, return type, access modifiers, and source location—is added as attributes to this new graph node.5
Creating Edges: The semantic links established during the resolution pass are now materialized as edges in the graph.
When the resolver linked a call_expression node in fileA.js to a function_definition node in fileB.js, a directed CALLS edge is created from the graph node representing the calling function to the graph node for the callee.
When the resolver identifies that ClassA inherits from ClassB, an INHERITS_FROM edge is created between their respective Class nodes.
When a statement is found to read the value of a variable, a USES edge is created from the node representing that statement to the Variable node corresponding to the variable's declaration.
Enriching with Deeper Semantic Information: The graph built so far primarily represents the program's static structure and call relationships. It can be further enriched with more detailed semantic information through additional analysis passes.
Control Flow Analysis: For each Function node in the graph, a local Control Flow Graph (CFG) can be computed by analyzing its body. This adds FLOWS_TO edges between statement nodes, representing the possible paths of execution.
Data Flow Analysis: More advanced techniques like taint analysis or value-flow analysis can be applied to trace the flow of data between variables and across function calls.35 This can result in fine-grained
REACHES edges, indicating that data from a particular definition can reach a specific use, which is a critical capability for detecting information leaks and many security vulnerabilities.36
At the conclusion of this phase, the system has transformed a directory of text files into a rich, multi-layered graph that represents the deep semantic structure of the entire application, ready for the analysis algorithms that will detect the targeted architectural flaws.

Algorithms for Leaky Abstraction Detection

Once the Semantic Dependency Graph is constructed, it becomes the substrate for analysis. The abstract software engineering concept of a "leaky abstraction" must be translated into concrete, queryable patterns within this graph. This section details the algorithms and analytical techniques required to traverse the graph and identify these architectural flaws. The approach is not a simple boolean check but rather a heuristic search for violations of design intent, guided by architectural principles and data-flow analysis.

Defining Leaky Abstractions as Graph Patterns

A leaky abstraction, at its core, is an unintended dependency where a client of an abstraction is forced to be aware of the abstraction's internal implementation details.3 This concept can be effectively modeled as the presence of "illegal" or "suspicious" paths within the Semantic Dependency Graph.
Architectural Layer Violations: Many software systems are designed with explicit architectural layers, such as a classic three-tier architecture: Presentation -> Business Logic -> Data Access. A well-defined architecture forbids direct communication that bypasses a layer. A leaky abstraction in this context would manifest as a CALLS or USES edge that originates from a node within a Presentation layer module and terminates at a node within a low-level DataAccess module (or worse, a third-party database driver), completely bypassing the Business Logic layer. This is a clear violation of the intended architectural boundaries and can be detected by querying for paths that cross these predefined layers in a prohibited manner.
Implementation Detail Exposure: Consider a function processData(data) that is intended to operate on data through a public interface. If the implementation of this function directly accesses a private or internal field, for example data.internal_state.value, it creates a USES dependency from the processData function to an implementation detail of the data object. This dependency "leaks" knowledge of the internal structure of data into its client. If the internal structure of data changes, processData will break, even though the public interface of data may be unchanged. This can be detected by identifying paths from a function to nodes representing private or non-public members of its parameters or collaborators.
Architectural Smells as Indicators: Leaky abstractions are a specific type of architectural smell, and their presence often correlates with other detectable structural problems.37 Graph metrics can be used to identify components that are "hotspots" for such smells. For instance, a "God Component"—a module or class with an excessively high number of incoming and outgoing dependencies—is a prime suspect for both containing and causing leaky abstractions, as its tangled responsibilities often lead to blurred architectural lines.38 Analyzing the graph for such smells can help prioritize where to search for leaks.

Core Graph Traversal and Analysis Algorithms

A set of fundamental graph algorithms provides the toolkit for navigating the graph and uncovering its structural properties. These algorithms are essential prerequisites for the more advanced detection techniques.
Topological Sorting: For any set of dependencies that should not be circular (e.g., module dependencies, build steps), a topological sort provides a linear ordering of the nodes such that for every directed edge from node u to v, u comes before v in the ordering.40 This is fundamental for many dependency analyses and build systems.17 A topological sort is only possible on a Directed Acyclic Graph (DAG). The inability to produce a topological sort is a definitive proof that the graph contains at least one cycle.42 Common implementations include Kahn's algorithm, which is based on Breadth-First Search (BFS), and a recursive algorithm based on Depth-First Search (DFS).41
Strongly Connected Components (SCCs): The primary algorithm for detecting and isolating circular dependencies in a directed graph is SCC analysis.17 A strongly connected component is a subgraph where every vertex is reachable from every other vertex. In a dependency graph, any non-trivial SCC (one with more than one node) represents a cycle.43 For example, if module A depends on module B, and module B depends on module A, they will form an SCC. Such cycles are severe architectural flaws that tightly couple components and are a breeding ground for leaky abstractions, as the components must have intimate knowledge of each other to function.44 Several linear-time (
O(V+E)) algorithms exist for finding SCCs, with Kosaraju's and Tarjan's being the most well-known.43
The choice between Kosaraju's and Tarjan's algorithm involves a trade-off. While both have the same optimal time complexity, their implementation details and practical characteristics differ.
Table 2: Comparison of SCC Algorithms (Kosaraju's vs. Tarjan's)
Feature
Kosaraju's Algorithm
Tarjan's Algorithm
Core Idea
Perform a DFS on the original graph to get finishing times, then a second DFS on the transposed graph in order of finishing times.
Perform a single DFS, maintaining a stack and "low-link" values for each node to identify SCC roots.
Number of Passes
Two DFS passes.
One DFS pass.
Data Structures
Requires an explicit representation of the transposed graph. Uses one stack for the first pass.
Uses one stack for nodes being visited and requires extra storage per node for discovery/low-link times.
Implementation Complexity
Conceptually simpler to understand and implement.
More complex due to the management of low-link values and the single-pass logic.
Performance
May have slightly higher constant factors due to the second pass and graph transposition.
Generally considered slightly faster in practice due to being a single-pass algorithm.


Advanced Analysis for Leak Detection

With the foundational algorithms in place, more specialized techniques can be applied to directly target the patterns of leaky abstractions.
Taint Analysis / Data-Flow Tracking: This is arguably the most powerful and direct method for this problem. Taint analysis is a form of data-flow analysis that tracks the propagation of "tainted" data through a program.36 To detect leaky abstractions, we can adapt this technique:
Define Sources: Taint sources are the origins of sensitive information. In this context, a source is an implementation detail that should not be leaked, such as a private field of a class, a variable internal to a low-level module, or data from an untrusted user input that should be sanitized.
Define Sinks: Sinks are locations where the sensitive information should not arrive. For leaky abstractions, a sink could be the public API boundary of a high-level module, a return value from a public function, or a user interface component.
Define Sanitizers: Sanitizers are the intended, legitimate pathways for data. In this case, the sanitizer is the abstraction layer itself—the public methods of the low-level module that are designed to mediate access to its internal state.
Detect Leaks: A leak is reported if the analysis finds a data-flow path in the graph from a taint source to a taint sink that does not pass through a valid sanitizer function. This technique directly models the flow of information across improper boundaries and is used by advanced tools like SonarQube to find complex injection vulnerabilities 36, a principle that maps perfectly to this problem.9
Path Analysis: This technique involves using graph traversal algorithms (like BFS or DFS) to find paths between specific nodes.48 To detect a leaky abstraction, an analyst can formulate queries such as: "Is there a
CALLS path from any function in the UI module to any function in the DatabaseDriver module that does not contain any nodes from the DataAccess module?" The existence of such a path is a strong indicator of a layer violation and a leaky abstraction.
Graph Metrics for Smell Detection: As previously mentioned, architectural smells can be identified by computing metrics over the dependency graph, which helps to focus the search for leaks.38
Coupling Metrics: A component with an unusually high out-degree (many outgoing DEPENDS_ON or CALLS edges) is highly coupled and warrants investigation.
Instability: A component's instability can be defined as the ratio of its outgoing dependencies to its total dependencies (incoming + outgoing). A component that depends on many other components but has few dependents itself is unstable. An architecture where stable components depend on unstable ones can lead to maintenance nightmares and is a sign of poor design.38
Cycle Detection: The presence, number, and size of cycles (SCCs) are powerful indicators of architectural decay and are often the root cause of leaky abstractions.46
By combining these algorithmic approaches, an analysis tool can move beyond simple rule-checking to perform a deep, structural, and data-flow-aware investigation of a program's architecture, providing developers with high-fidelity signals about where their abstractions are failing.

Engineering for Scale and Responsiveness

A sophisticated static analysis is of little practical value if it is too slow to integrate into a developer's daily workflow or if it consumes prohibitive amounts of memory. For an analysis tool to be adopted, especially in the context of large, enterprise-scale codebases, it must be engineered for performance, scalability, and responsiveness. This section addresses the critical engineering decisions surrounding data structures, incremental analysis architecture, and optimizations for large-scale projects.

Data Structures and Performance Trade-offs

The choice of the underlying data structure to store the Semantic Dependency Graph has profound and lasting implications for the tool's performance profile, memory footprint, and scalability. The primary choice is between in-memory representations and persistent graph databases.
In-Memory Representations:
Adjacency Matrix: This representation uses a V×V matrix (where V is the number of vertices) to store the graph, with matrix[i][j] = 1 indicating an edge from vertex i to j.49 Its primary advantage is the
O(1) time complexity for checking the existence of an edge. However, its O(V2) space complexity makes it completely impractical for the sparse graphs that typically represent software systems, where the number of edges E is far less than V2.50 Iterating over a node's neighbors also requires a slow
O(V) scan of an entire row.
Adjacency List: This is the standard in-memory representation for sparse graphs. It uses an array or hash map where each vertex maps to a list of its adjacent vertices.49 The space complexity is an optimal
O(V+E). Iterating over a node's neighbors is efficient, proportional to its degree. The main trade-off is that checking for the existence of a specific edge requires scanning a node's adjacency list, taking O(degree) time on average.50 For most static analysis tasks, where traversal is more common than random edge lookups, the adjacency list is the superior in-memory choice.
Graph Databases: Systems like Neo4j, FalkorDB, or JanusGraph offer a different set of trade-offs.
Advantages: Their key benefit is persistence and scalability beyond main memory. They provide powerful, high-level query languages (e.g., Cypher, Gremlin) that can simplify the implementation of complex analysis queries.6 They are optimized for "localized" queries, where the traversal starts from a small set of seed nodes and explores their local neighborhood. Performance for such queries tends to remain relatively constant even as the overall graph size grows.10
Disadvantages: Graph databases introduce operational overhead for setup and maintenance. Ingestion of the graph can be slower than building an in-memory structure. For global analyses that require traversing the entire graph (e.g., computing SCCs on the whole program), they can be slower than a highly optimized in-memory algorithm due to disk I/O and query interpretation overhead.54 Performance is also highly sensitive to the data model and the specific query patterns used.56
The choice of representation is dictated by the primary use case. For an interactive tool in an IDE that needs to provide rapid feedback on a developer's current changes, a fast, in-memory adjacency list is likely the best fit. For an offline, whole-program analysis of a massive, multi-million-line codebase where results are persisted for ad-hoc exploration by multiple users, a graph database becomes a compelling option.
Table 3: Performance Characteristics of Graph Representations
Operation
Adjacency Matrix
Adjacency List
Graph Database
Space Complexity
O(V2)
O(V+E)
O(V+E) + index/storage overhead
Add Vertex
O(V2) (resize)
O(1)
Amortized O(1) or O(logV)
Add Edge
O(1)
O(1) (amortized)
Amortized O(1) or O(logV)
Check Edge (u, v)
O(1)
O(degree(u))
O(1) to O(logV) with indexing
Iterate Neighbors(u)
O(V)
O(degree(u))
Proportional to degree (fast local traversal)
Global Traversal (e.g., BFS)
O(V2)
O(V+E)
Slower than in-memory due to I/O and overhead
Best For
Dense graphs (rare in code)
Sparse graphs, in-memory analysis
Very large graphs, persistence, complex queries


Architectural Patterns for Incremental Analysis

Analyzing an entire multi-million-line codebase from scratch for every minor change is not feasible. An incremental architecture is essential for providing timely feedback in a CI/CD or IDE context. The core principle is that the analysis time should be proportional to the size of the change and its semantic impact, not the size of the entire project.57
The Dependency Graph as a Guide: The Semantic Dependency Graph is the key to achieving incrementality. When a file is modified, it is not enough to re-analyze just that file. The change may affect other parts of the system that depend on it. The "impact set"—the collection of all files that need to be re-analyzed—can be determined by performing a backward traversal from the changed node in the dependency graph to find all nodes that depend on it, and then a forward traversal from those nodes to find what they, in turn, affect.57
Architectural Models for Incrementality:
Build System Analogy: A powerful mental model is to treat the analysis process like a build system such as Make or Bazel.17 Each analysis result (e.g., the symbol table for a file, the taint analysis for a function) is an "artifact." Each artifact has dependencies on source files or other artifacts. When a source file changes, all downstream artifacts that depend on it are marked "dirty" and must be recomputed. This creates a dependency graph of
analysis tasks that mirrors the code's dependency graph.
Reactive Architecture: The system can be designed to be event-driven. It listens for file change events from the version control system or file system. A change event triggers a targeted re-analysis of the affected files and their dependents, as determined by the dependency graph.60
Incremental Static Regeneration (ISR) Pattern: This pattern, borrowed from modern web development, prioritizes user responsiveness.61 When a developer requests an analysis, the system can immediately return the last known "stale" result from a cache, while simultaneously launching a background process to recompute the analysis based on the latest changes. Once the new result is ready, it updates the cache and can be pushed to the user.
The Central Role of Caching and Invalidation: Incrementality is impossible without caching. The system must cache intermediate and final analysis results, such as parsed ASTs, resolved symbol tables, and control-flow graphs. The primary challenge is not caching itself, but cache invalidation. The dependency graph is the oracle for this: a change to a node invalidates the cached results for itself and all nodes that transitively depend on it.62
Lazy vs. Eager Computation:
Eager: The traditional approach is to analyze the entire project upfront. This has a high initial time and memory cost, but all results are immediately available for querying.
Lazy: A more responsive approach, especially for IDEs, is lazy evaluation.65 An analysis is only performed when its result is explicitly requested.66 For example, the system might only resolve the symbols and run data-flow analysis for a function when the developer actually opens that function in their editor. This minimizes startup time and resource usage, at the cost of a slight delay when the information is first requested.
The architecture of an effective incremental analysis system is therefore more akin to a database query planner or a modern build system than a simple script. It requires careful management of dependencies, cached state, and computation triggers to achieve the necessary performance for real-world integration.

Optimizing for Large Codebases: Memory Management and Parallelism

As codebases grow, even incremental analysis can become a bottleneck. Further optimizations are needed to handle projects at the scale of millions of lines of code.
On-Demand and Partial Loading: It may be impossible to load the entire semantic graph for a very large project into memory.13 The architecture should support loading the graph in partitions. For example, when analyzing a specific module, the system could load only the subgraph corresponding to that module and its direct dependencies.
Parallelization: Many parts of the analysis pipeline are amenable to parallel execution.69
Graph Construction: The initial parsing of individual files (Phase 1) is an embarrassingly parallel task, as each file can be processed independently on a separate CPU core.70 The symbol resolution and graph linking phases are more complex to parallelize due to potential race conditions when writing to the global symbol table and graph, requiring careful synchronization.
Analysis Queries: Some graph algorithms and analysis queries can be parallelized, but this often requires specialized algorithms and adds significant implementation complexity.
Performance Case Studies: Real-world experience demonstrates both the promise and the challenges of these techniques. A case study on incrementalizing CodeQL showed that it was possible to reduce analysis update times for pull requests to a few seconds on average. However, this came at the cost of a very high initial analysis time (up to an hour) and significant memory usage (tens of gigabytes), as the system builds and maintains complex data structures to accelerate subsequent updates.71 Another study using a technique called path abstraction for incremental analysis reported performance gains of up to 32% on codebases as large as 87,000 lines of code.72 These results highlight a fundamental trade-off: there is often an inverse relationship between the initial, from-scratch analysis cost and the speed of subsequent incremental updates. The optimal balance depends on the specific application scenario.

Implementation Blueprint: A Practical Guide to Using Tree-sitter

This section provides a focused, practical guide to implementing the core components of the analysis pipeline using the Tree-sitter ecosystem. It connects the theoretical concepts discussed previously to the specific tools and libraries available, offering a concrete starting point for development and highlighting the architectural decisions that must be made.

The Tree-sitter Ecosystem: Parsers, Queries, and Bindings

Tree-sitter is not a monolithic application but a collection of tools and libraries designed to work together.19 Understanding these components is the first step to leveraging its power.
The Parser Generator (CLI): The tree-sitter command-line tool is used to generate a parser from a grammar definition file (typically grammar.js). This process produces C source code for the parser, which is then compiled into a shared library (.so, .dll) or a WebAssembly module (.wasm).18
The Runtime Library: At the core of Tree-sitter is a dependency-free runtime library written in pure C. This library is responsible for using the generated parser tables to parse source code and manage the syntax tree.19
Language Bindings: To use a Tree-sitter parser in a higher-level language like Python, Rust, or JavaScript, one uses a language-specific binding. These bindings provide an idiomatic API to load the compiled parser library, feed it source code, and interact with the resulting syntax tree.19
Pre-built Grammars: A significant advantage of the Tree-sitter ecosystem is the large number of high-quality, community-maintained grammars available for popular programming languages. This means that for many use cases, it is not necessary to write a grammar from scratch; one can simply use the existing parser for Java, Python, Rust, etc..18

A Practical Guide to the tree-sitter-graph DSL

The tree-sitter-graph library offers a declarative Domain-Specific Language (DSL) for defining how to build a graph from a Tree-sitter AST.73 It allows you to specify patterns in the syntax tree and the corresponding graph nodes and edges to create when those patterns are matched.
Core Concepts: The DSL is defined in files with a .tsg extension. A .tsg file consists of a series of stanzas. Each stanza begins with a Tree-sitter query that identifies specific nodes in the AST. Following the query is a block that defines the graph construction rules for any matches.74 These rules can create nodes, set attributes on them, and create edges between them.
The Architectural Gap and A Conceptual Example: It is critical to recognize that tree-sitter-graph is designed to operate on a single AST at a time.25 It has no built-in mechanism for resolving symbols or creating edges that span across different files. This implies an architecture where
tree-sitter-graph is used to generate "graph fragments" on a per-file basis. A separate, higher-level "graph linker" process is then responsible for stitching these fragments together using a global symbol table.
While the official documentation is sparse on complex examples 73, we can construct a conceptual
.tsg file to illustrate the principle. The following example aims to extract a partial call graph from a single Python file.

Code snippet


; File: python_calls.tsg

; Rule 1: Create a graph node for each function definition.
; The query matches a function_definition node and captures its name.
(function_definition
  name: (identifier) @function.name) @function.definition
{
  ; Create a node in the output graph, uniquely identified by the
  ; underlying AST node of the function definition.
  node @function.definition
  {
    ; Set attributes on the newly created node.
    kind: "function",
    name: (source-text @function.name),
    is_exported: true
  }
}

; Rule 2: Create a node for each function call and an edge to its definition.
(call
  function: (identifier) @call.name) @call.expression
{
  ; This is the conceptual part that highlights the architectural gap.
  ; The DSL itself cannot resolve @call.name across files.
  ; We assume an external mechanism provides a `resolve-function` function
  ; that looks up the name in a global symbol table and returns the
  ; graph node for the corresponding function definition.
  let resolved_callee = (resolve-function (source-text @call.name))

  ; Only create an edge if the function was successfully resolved.
  if (is-not-null resolved_callee) {
    ; Assume the calling function's node is available in a variable,
    ; set by a parent rule.
    edge (variable calling_function_node) -> resolved_callee
    {
      label: "calls",
      location: (source-span @call.expression)
    }
  }
}


In a real implementation, the resolve-function call would be handled by the application code that invokes the tree-sitter-graph library, which would have access to the global symbol table built in the preceding analysis phase. This demonstrates the necessary separation of concerns: tree-sitter-graph handles the syntactic pattern matching within a file, while the main application handles the semantic, cross-file logic.

Building the Symbol Resolution Layer on Top of Tree-sitter

This subsection provides a concrete, step-by-step walkthrough of how to build the crucial symbol resolution and graph linking logic that sits on top of Tree-sitter's parsing capabilities. This process synthesizes the principles from Section 2 into an actionable implementation plan.
Step 1: Parse All Project Files.
The process begins by traversing the project directory and identifying all source files. For each file, use the appropriate language binding (e.g., tree-sitter-python) to invoke the Tree-sitter parser, generating an in-memory AST. Store these ASTs, perhaps in a map from file paths to tree objects, for subsequent passes.
Step 2: First Pass — Populate the Global Symbol Table.
Create a global data structure, such as a HashMap<String, SymbolInfo>, to serve as the symbol table. Iterate through every AST generated in Step 1. For each AST, execute a series of Tree-sitter queries designed to find all top-level declarations: (function_definition), (class_definition), (global_statement), etc.
For each declaration match, extract the symbol's name, determine its fully qualified name based on its module and enclosing scopes, and create a SymbolInfo struct. This struct should contain the symbol's kind (function, class), its type information (if available from type hints), its file path, and a direct reference or pointer to its AST node. Insert this record into the global symbol table.
Step 3: Second Pass — Resolve Usages.
After the first pass, the symbol table contains a complete inventory of every symbol defined within the project's source code. Now, perform a second iteration over all the ASTs. This time, execute queries to find all usages of symbols, such as (call_expression), (identifier) in an expression context, or (type_identifier).
For each usage, extract the identifier's name. Then, perform a lookup in the global symbol table. This lookup logic must be language-specific, respecting the language's scoping and import rules. For example, for an identifier foo, the resolver would first check the local scope, then enclosing scopes, then the module's global scope, and finally check the file's import statements to see if foo is an alias for a symbol from another module.
Step 4: Stitch the Graph.
This step runs concurrently with Step 3. When a symbol usage is successfully resolved to a declaration in the symbol table, it establishes a semantic link. This link is then materialized as an edge in the final Semantic Dependency Graph.
If a call_expression for bar() in fileA is resolved to the function_definition for bar in fileB, create a CALLS edge in the final graph between the nodes representing the calling function and the bar function.
If a variable usage is resolved, create a USES edge from the statement node to the resolved Variable declaration node.
If a symbol cannot be resolved within the project's source code, the system should then consult its representation of external dependencies (the "auxiliary classpath"). If it's found there, the link is made to a node representing the library symbol. If it's still not found, it is marked as an unresolved symbol, which could represent an error in the user's code.
This methodical process, separating parsing from symbol resolution and graph construction, forms a robust foundation for a cross-file static analysis tool. The logic for symbol resolution is undeniably the most complex, language-specific part of the system and requires careful implementation of the target language's semantic rules, a task that goes well beyond what a purely syntactic tool like Tree-sitter can provide on its own.77

Conclusion and Recommendations

The endeavor to implement a cross-file semantic analysis system for detecting leaky abstractions is a significant but achievable engineering challenge. It requires a synthesis of concepts from compiler theory, software architecture, and graph analysis. The success of such a tool hinges not on a single clever algorithm, but on a well-designed, multi-phase architecture that systematically builds a rich semantic representation of the entire codebase before querying it for architectural flaws. A naive approach that fails to account for cross-file dependencies or the complexities of large-scale software is destined to produce inaccurate and impractical results.

Synthesizing the Approach: A High-Level Implementation Roadmap

Based on the detailed analysis of algorithms, data structures, and architectural patterns, the following high-level roadmap provides a recommended path for implementation:
Foundation: Per-File Parsing. The foundation of the system should be a robust parsing layer. Leverage the Tree-sitter ecosystem to parse individual source files into Abstract Syntax Trees. This provides a fast, reliable, and fault-tolerant way to handle the syntax of multiple programming languages without needing to write parsers from scratch.
Core Engine: Cross-File Symbol Resolution. The heart of the system is the semantic engine that links the individual ASTs. Implement a two-pass, language-specific symbol resolver. The first pass should scan all ASTs to populate a global symbol table with every declaration. The second pass should then resolve every symbol usage against this table. This engine must correctly handle language-specific scoping, inheritance, and import/module resolution rules. Crucially, it must also support an "auxiliary classpath" to resolve symbols from compiled third-party dependencies.
Representation: The Semantic Graph. The output of the core engine should be a rich Semantic Dependency Graph, akin to a Code Property Graph. Choose the graph representation based on the primary use case. For interactive, IDE-based tools where speed is paramount, an in-memory adjacency list is the optimal choice. For offline analysis of massive codebases or for enabling complex ad-hoc queries, a persistent graph database like Neo4j or FalkorDB offers superior scalability and querying power.
Analysis: Incremental by Design. Performance and responsiveness are non-negotiable for developer tools. Architect the system for incrementality from the outset. Use the dependency graph itself to drive the analysis. A change to a file should trigger a re-analysis of only that file and its semantic dependents (the "impact set"). Employ caching for all intermediate artifacts (ASTs, symbol tables, analysis results) and use the dependency graph to manage cache invalidation.
Detection: Heuristic Graph Queries. The final step is to query the graph to find leaky abstractions. Model leaks as heuristic patterns of architectural violation. The most powerful and direct technique is taint analysis, tracking the flow of implementation details (sources) across abstraction boundaries (sinks) without passing through intended public interfaces (sanitizers). Supplement this with path analysis to detect architectural layer violations and compute graph metrics to identify structural "smells" like cycles and God components, which serve as indicators of potential problem areas.

Key Challenges and Future Directions

While the roadmap is clear, implementation entails navigating several key challenges. The primary difficulties lie not in the parsing stage, but in:
Correctly implementing the language-specific semantics for symbol resolution, which can be complex and nuanced.
Managing the performance and memory footprint of the graph and analysis algorithms, especially for projects with millions of lines of code.
Designing a truly robust and correct incremental update engine, particularly the logic for cache invalidation in the face of complex, transitive dependencies.
Looking forward, the field of static analysis is continually evolving. The integration of Large Language Models (LLMs) presents a compelling future direction. LLMs could be used to enrich the semantic graph by generating natural-language summaries for functions and modules, or to infer semantic relationships that are not explicit in the code.12 It may even become possible to generate complex analysis queries from natural language prompts, making these powerful tools more accessible.14 Furthermore, as software development increasingly relies on open-source components, the scope of analysis is expanding to encompass the entire software supply chain, analyzing not just first-party code but the entire tree of transitive dependencies for vulnerabilities and architectural incompatibilities.78 Building a flexible and extensible semantic graph architecture today will position a tool to incorporate these powerful new capabilities in the future.
Works cited
What is the difference between static analysis and semantic analysis? - Stack Overflow, accessed July 3, 2025, https://stackoverflow.com/questions/20498566/what-is-the-difference-between-static-analysis-and-semantic-analysis
Static vs. dynamic code analysis: A comprehensive guide - vFunction, accessed July 3, 2025, https://vfunction.com/blog/static-vs-dynamic-code-analysis/
Blocking code is a leaky abstraction : r/rust - Reddit, accessed July 3, 2025, https://www.reddit.com/r/rust/comments/1g86eoq/blocking_code_is_a_leaky_abstraction/
How To Hide A Leaky Abstraction In Plain Sight | by Fayner Brack | Medium, accessed July 3, 2025, https://fagnerbrack.com/repairing-the-leaky-abstraction-e726baab91b5
Code Graph: From Visualization to Integration - FalkorDB, accessed July 3, 2025, https://www.falkordb.com/blog/code-graph/
Enhancing Code Analysis With Code Graphs - DZone, accessed July 3, 2025, https://dzone.com/articles/enhancing-code-analysis-with-code-graphs
arxiv.org, accessed July 3, 2025, https://arxiv.org/html/2310.02128v2
Semantic Code Graph—An Information Model to Facilitate Software Comprehension, accessed July 3, 2025, https://www.researchgate.net/publication/377287545_Semantic_Code_Graph_-_an_information_model_to_facilitate_software_comprehension/download
Confidentiality Leakage Analysis of Database-Driven Applications - ResearchGate, accessed July 3, 2025, https://www.researchgate.net/publication/355720947_Confidentiality_Leakage_Analysis_of_Database-Driven_Applications
Use of graph databases for static code analysis, accessed July 3, 2025, https://richardg.users.greyc.fr/publis/Dauprat-All_2022.pdf
Learning Graph-based Code Representations for Source-level Functional Similarity Detection - Jun ZENG, accessed July 3, 2025, https://jun-zeng.github.io/file/tailor_paper.pdf
Building a Knowledge Graph of Your Codebase - Daytona, accessed July 3, 2025, https://www.daytona.io/dotfiles/building-a-knowledge-graph-of-your-codebase
Hierarchical Graph-Based Code Summarization for Enhanced Context Retrieval - arXiv, accessed July 3, 2025, https://arxiv.org/html/2504.08975v1
Building Knowledge Graph over a Codebase for LLM | by Zimin Chen | Medium, accessed July 3, 2025, https://medium.com/@ziche94/building-knowledge-graph-over-a-codebase-for-llm-245686917f96
Constructing knowledge graphs from text using OpenAI functions - Tomaz Bratanic - Medium, accessed July 3, 2025, https://bratanic-tomaz.medium.com/constructing-knowledge-graphs-from-text-using-openai-functions-096a6d010c17
Dependency Graph in Compiler Design - GeeksforGeeks, accessed July 3, 2025, https://www.geeksforgeeks.org/compiler-design/dependency-graph-in-compiler-design/
Dependency graph - Wikipedia, accessed July 3, 2025, https://en.wikipedia.org/wiki/Dependency_graph
Incremental Parsing Using Tree-sitter - Strumenta - Federico Tomassetti, accessed July 3, 2025, https://tomassetti.me/incremental-parsing-using-tree-sitter/
Tree-sitter: Introduction, accessed July 3, 2025, https://tree-sitter.github.io/
Abstract syntax tree - Wikipedia, accessed July 3, 2025, https://en.wikipedia.org/wiki/Abstract_syntax_tree
Unraveling Tree-Sitter Queries: Your Guide to Code Analysis Magic - DEV Community, accessed July 3, 2025, https://dev.to/shrsv/unraveling-tree-sitter-queries-your-guide-to-code-analysis-magic-41il
Building Call Graphs for Code Exploration Using Tree-Sitter - DZone, accessed July 3, 2025, https://dzone.com/articles/call-graphs-code-exploration-tree-sitter
5 Powerful Ways to Use Tree-sitter in Your Next Project | by Istiaq Ahmed Fahad - Medium, accessed July 3, 2025, https://medium.com/@ahmedfahad04/5-powerful-ways-to-use-tree-sitter-in-your-next-project-50e17c1f7055
tree-sitter explained - YouTube, accessed July 3, 2025, https://www.youtube.com/watch?v=09-9LltqWLY
Tree-sitter: Introduction, accessed July 3, 2025, https://tree-sitter.github.io/tree-sitter/
Symbol resolution in static vs dynamic libraries - c++ - Stack Overflow, accessed July 3, 2025, https://stackoverflow.com/questions/9299634/symbol-resolution-in-static-vs-dynamic-libraries
How to solve linker symbol problems - LabEx, accessed July 3, 2025, https://labex.io/tutorials/cpp-how-to-solve-linker-symbol-problems-493612
Symbol Processing - Linker and Libraries Guide, accessed July 3, 2025, https://docs.oracle.com/cd/E26505_01/html/E26506/chapter2-90421.html
Symbol table - Wikipedia, accessed July 3, 2025, https://en.wikipedia.org/wiki/Symbol_table
Symbol Tables and Static Checks - cs.wisc.edu, accessed July 3, 2025, https://pages.cs.wisc.edu/~fischer/cs536.s08/course.hold/html/NOTES/6.SYMBOL-TABLES.html
14.2. RTEMS Symbols, accessed July 3, 2025, https://docs.rtems.org/docs/main/user/tools/symbols.html
Java support | PMD Source Code Analyzer, accessed July 3, 2025, https://pmd.github.io/pmd/pmd_languages_java.html
How PMD Works | PMD Source Code Analyzer, accessed July 3, 2025, https://pmd.github.io/pmd/pmd_devdocs_how_pmd_works.html
How PMD Works | PMD Source Code Analyzer, accessed July 3, 2025, https://docs.pmd-code.org/latest/pmd_devdocs_how_pmd_works.html
Static Code Analysis: Everything You Need To Know - Codacy | Blog, accessed July 3, 2025, https://blog.codacy.com/static-code-analysis
Advanced security with SonarQube | Sonar, accessed July 3, 2025, https://www.sonarsource.com/solutions/security/
Architectural smell analysis - Arcan, accessed July 3, 2025, https://www.arcan.tech/architectural-smell-analysis/
Does your architecture smell? - Revisited - Designite, accessed July 3, 2025, https://www.designite-tools.com/blog/does-your-architecture-smell-revisited
Anti-Patterns in Software Architecture - Number Analytics, accessed July 3, 2025, https://www.numberanalytics.com/blog/anti-patterns-in-software-architecture
Topological Sorting Simplified - Number Analytics, accessed July 3, 2025, https://www.numberanalytics.com/blog/topological-sorting-simplified-beginner-guide
Topological Sorting Explained: A Step-by-Step Guide for Dependency Resolution - Medium, accessed July 3, 2025, https://medium.com/@amit.anjani89/topological-sorting-explained-a-step-by-step-guide-for-dependency-resolution-1a6af382b065
Topological Sort Algorithm - Interview Cake, accessed July 3, 2025, https://www.interviewcake.com/concept/java/topological-sort
Strongly connected component - Wikipedia, accessed July 3, 2025, https://en.wikipedia.org/wiki/Strongly_connected_component
Strongly Connected Components of Graph (Properties, Uses, Algorithms) - WsCube Tech, accessed July 3, 2025, https://www.wscubetech.com/resources/dsa/strongly-connected-components
Strongly Connected Components - GeeksforGeeks, accessed July 3, 2025, https://www.geeksforgeeks.org/dsa/strongly-connected-components/
Graph-Based Analysis and Prediction for Software Evolution, accessed July 3, 2025, https://www.cs.ucr.edu/~neamtiu/pubs/icse12bhattacharya.pdf
(PDF) A survey of static analysis methods for identifying security ..., accessed July 3, 2025, https://www.researchgate.net/publication/224101611_A_survey_of_static_analysis_methods_for_identifying_security_vulnerabilities_in_software_systems
Understanding Software Dependency Graphs | Blog - VulnCheck, accessed July 3, 2025, https://vulncheck.com/blog/understanding-software-dependency-graphs
Graph and its representations - GeeksforGeeks, accessed July 3, 2025, https://www.geeksforgeeks.org/dsa/graph-and-its-representations/
What's the difference between an adjacency matrix and an adjacency list in graph representation? | TutorChase, accessed July 3, 2025, https://www.tutorchase.com/answers/a-level/computer-science/what-s-the-difference-between-an-adjacency-matrix-and-an-adjacency-list-in-graph-representation
Big O Complexity for Graphs: Adjacency Matrix vs Adjacency List - DEV Community, accessed July 3, 2025, https://dev.to/akashdeep/big-o-complexity-for-graphs-adjacency-matrix-vs-adjacency-list-3akf
When is it better to use adjacency matrix vs. adjacency list representation of a graph? : r/csMajors - Reddit, accessed July 3, 2025, https://www.reddit.com/r/csMajors/comments/h8y8nv/when_is_it_better_to_use_adjacency_matrix_vs/
Graph database performance - neo4j - Stack Overflow, accessed July 3, 2025, https://stackoverflow.com/questions/29629903/graph-database-performance
A Performance Evaluation of Open Source Graph Databases - David A. Bader, accessed July 3, 2025, https://davidbader.net/publication/2014-mep/2014-mep.pdf
A Review of Graph Databases - NebulaGraph, accessed July 3, 2025, https://www.nebula-graph.io/posts/review-on-graph-databases
Bullshit Graph Database Performance Benchmarks - Max De Marzi, accessed July 3, 2025, https://maxdemarzi.com/2023/01/11/bullshit-graph-database-performance-benchmarks/
Incremental Static Analysis of Large Source Code ... - ftsrg - BME, accessed July 3, 2025, https://ftsrg.mit.bme.hu/thesis-works/pdfs/stein-daniel-bsc.pdf
A static analysis method of incremental codes using value ..., accessed July 3, 2025, https://www.spiedigitallibrary.org/conference-proceedings-of-spie/12290/122900M/A-static-analysis-method-of-incremental-codes-using-value-dependency/10.1117/12.2640792.full
Efficient Pattern-based Static Analysis Approach via Regular-Expression Rules, accessed July 3, 2025, https://www.shinhwei.com/saner_modified.pdf
Evolutionary Architecture in Practice: Implementing Incremental Complexity Patterns for High-Velocity Full-Stack Systems | by Romoaldo Doliz - Medium, accessed July 3, 2025, https://medium.com/@romoaldodoliz_57699/evolutionary-architecture-in-practice-implementing-incremental-complexity-patterns-for-904f3311cbc0
Incremental Static Regeneration (ISR): Keeping Your Pages Fresh and Fast - Medium, accessed July 3, 2025, https://medium.com/@ignatovich.dm/incremental-static-regeneration-isr-keeping-your-pages-fresh-and-fast-f28933c28e54
What is Semantic Caching For LLMs? | GigaSpaces AI, accessed July 3, 2025, https://www.gigaspaces.com/data-terms/semanticaching-for-llms
Adaptive Semantic Prompt Caching with VectorQ - arXiv, accessed July 3, 2025, https://arxiv.org/html/2502.03771v1
Get started with semantic caching policies | Apigee - Google Cloud, accessed July 3, 2025, https://cloud.google.com/apigee/docs/api-platform/tutorials/using-semantic-caching-policies
Lazy evaluation - Wikipedia, accessed July 3, 2025, https://en.wikipedia.org/wiki/Lazy_evaluation
Lazy loading - Performance - MDN Web Docs - Mozilla, accessed July 3, 2025, https://developer.mozilla.org/en-US/docs/Web/Performance/Guides/Lazy_loading
Mastering Lazy Loading Techniques - Number Analytics, accessed July 3, 2025, https://www.numberanalytics.com/blog/mastering-lazy-loading-techniques
Code Graphs: A New Approach to Visualizing and Optimizing Software Architecture, accessed July 3, 2025, https://cloud-computing.tmcnet.com/columns/articles/460517-code-graphs-new-approach-visualizing-optimizing-software-architecture.htm
How Does Static Code Analysis Handle Multi-Threaded or Concurrent Code?, accessed July 3, 2025, https://www.in-com.com/blog/how-does-static-code-analysis-handle-multi-threaded-or-concurrent-code/
Static Code Analysis - AutoRABIT Knowledge Base, accessed July 3, 2025, https://knowledgebase.autorabit.com/product-guides/arm/arm-features/reports/static-code-analysis
Incremental CodeQL - GitHub Next, accessed July 3, 2025, https://githubnext.com/projects/incremental-codeql/
Efficient Incremental Static Analysis Using Path Abstraction - ResearchGate, accessed July 3, 2025, https://www.researchgate.net/publication/300858196_Efficient_Incremental_Static_Analysis_Using_Path_Abstraction
tree-sitter-graph - crates.io: Rust Package Registry, accessed July 3, 2025, https://crates.io/crates/tree-sitter-graph/0.7.0
tree_sitter_graph - Rust - Docs.rs, accessed July 3, 2025, https://docs.rs/tree-sitter-graph/*/tree_sitter_graph/
IBM/tree-sitter-codeviews: Extract and combine multiple source code views using tree-sitter - GitHub, accessed July 3, 2025, https://github.com/IBM/tree-sitter-codeviews
tree-sitter/tree-sitter-graph: Construct graphs from parsed ... - GitHub, accessed July 3, 2025, https://github.com/tree-sitter/tree-sitter-graph
Dossier: A tree-sitter based multi-language source code and docstring parser : r/rust - Reddit, accessed July 3, 2025, https://www.reddit.com/r/rust/comments/1980y0j/dossier_a_treesitter_based_multilanguage_source/
Less effort, more insight: Introducing Dependency Graph for Supply Chain - Semgrep, accessed July 3, 2025, https://semgrep.dev/blog/2024/less-effort-more-insight-introducing-dependency-graph-for-supply-chain/
