
Automated Detection of Leaky Abstractions via Cross-Module Type Flow Analysis


Introduction

In the discipline of software engineering, the principle of abstraction stands as a cornerstone for managing complexity. By hiding implementation details behind well-defined interfaces, developers can build large, maintainable, and evolvable systems. However, these abstractions are often imperfect. A "leaky abstraction" is one that fails to adequately conceal its underlying mechanics, forcing the consumer of the abstraction to be aware of details that were meant to be hidden.1 This phenomenon is not a rare anomaly but a pervasive challenge in software development, leading to tightly coupled modules, reduced maintainability, and architectural decay.3
The "Law of Leaky Abstractions" famously posits that all non-trivial abstractions, to some degree, are leaky.4 For instance, a network library that presents a remote file as if it were local cannot fully hide the realities of network latency and unreliability.6 Similarly, an Object-Relational Mapper (ORM) that abstracts away SQL may require developers to understand the underlying database's query execution plan to resolve performance bottlenecks.2 While some leaks are inherent to the technology stack, a more insidious category arises from unintentional design flaws: the exposure of internal, implementation-specific types through public Application Programming Interfaces (APIs). This type of leak creates a brittle contract between modules, where a change in an internal data structure can cause a cascade of breaking changes throughout the system.
To combat this architectural degradation proactively, this report outlines a methodology for the automated detection of leaky abstractions using static analysis. The core premise is to analyze the flow of types across module boundaries, identifying where internal types improperly cross into public interfaces. By performing this analysis at the source code level, either during compilation or as part of a Continuous Integration/Continuous Deployment (CI/CD) pipeline, these architectural defects can be caught and remediated before they become entrenched in the codebase.8
This report provides a comprehensive blueprint for designing and implementing such a detector. It begins by establishing the foundations of type flow analysis using Abstract Syntax Trees (ASTs), with a specific focus on the Tree-sitter parsing framework. It then details algorithms for constructing a scalable type dependency graph, the central data structure for this analysis. Subsequently, the report addresses the unique and nuanced challenges presented by the type systems of Rust, Python, and JavaScript/TypeScript. Finally, it catalogs common leaky abstraction patterns and provides a practical guide, including concrete Tree-sitter queries, for their automated detection.

Section 1: Foundations of Type Flow Analysis with Abstract Syntax Trees

This section establishes the theoretical and practical groundwork for analyzing source code to understand how types are defined, propagated, and utilized across a codebase. The methodology is rooted in parsing source code into an Abstract Syntax Tree (AST) and then performing semantic analysis to track the flow of types.

1.1 The Static Analysis Pipeline: From Source Code to Actionable Insights

The process of transforming raw source code into a high-level understanding of type dependencies involves a multi-stage pipeline.
First, the source code is parsed into an Abstract Syntax Tree (AST). An AST is a hierarchical, tree-based representation of the code's syntactic structure, abstracting away non-essential details like punctuation, whitespace, and comments.9 Each node in the tree represents a construct in the language, such as a function definition, an expression, or a variable declaration.9 This structured format is far more amenable to programmatic analysis than raw text and serves as the foundational data structure for compilers, linters, and other code analysis tools.9
Second, the AST undergoes semantic analysis. While the AST captures the code's syntax, it does not capture its meaning. For example, the AST can identify an identifier named User, but it does not know whether User refers to a class defined in the current file or a type imported from another module. To resolve this, the analyzer traverses the AST to build a Symbol Table. This is a critical data structure that maps identifiers to their definitions, scopes, and, most importantly for our purposes, their resolved types.10 This process, also known as contextual analysis, enriches the syntactic information from the AST with semantic meaning.
Finally, with a populated symbol table, the system can perform type flow analysis. This is a specific form of data dependency analysis that tracks the types of values as they are assigned to variables, passed as arguments to functions, and returned from functions.12 By following this flow, the analyzer can determine the type of any expression at any point in the program, forming the basis for detecting when a type from an "internal" module leaks into a "public" one.

1.2 Tree-sitter as an Analysis Backend: A Practical Choice

For the parsing stage of the pipeline, this report advocates for the use of Tree-sitter. Tree-sitter is a modern parser generator tool and library with several features that make it exceptionally well-suited for building robust, multi-language analysis tools.13
Its core strengths include:
Speed and Incrementality: Tree-sitter is designed to be fast enough to parse code on every keystroke in a text editor. Its incremental parsing capability allows it to efficiently update the syntax tree when the source file is edited, rather than re-parsing the entire file. This is crucial for responsive IDE integrations.13
Robustness: It features a powerful error-recovery system, enabling it to produce a useful and largely correct syntax tree even in the presence of syntax errors. This is vital for analyzing code that is actively being written and is often in an incomplete state.13
Language Agnosticism: Tree-sitter provides a consistent API across all supported languages. With available grammars for Rust, Python, JavaScript, TypeScript, and many others, it serves as a unified backend for a multi-language leaky abstraction detector.13
Powerful Query Engine: Perhaps its most compelling feature for this task is its integrated query engine. Queries are written in a Lisp-like S-expression syntax and allow for declarative pattern matching against the AST.17 This enables the extraction of all function definitions, import statements, or type annotations with a concise query, eliminating the need to write complex and brittle imperative AST traversal code.18
While Tree-sitter provides the syntactic foundation, it is crucial to recognize that it is not a complete solution. It can identify that a function returns a type_identifier node with the text User, but it cannot resolve what User semantically refers to. The heavy lifting of scope resolution, import tracking, and type linking must be implemented in a semantic analysis layer built on top of the AST provided by Tree-sitter.20

1.3 Core Analysis Primitives: Building Symbol Tables and Resolving Scopes

Building the semantic layer on top of Tree-sitter requires a structured, multi-pass approach to handle forward references and inter-module dependencies. A single pass is insufficient because a type or function may be used before it is defined.
Pass 1: Declaration Discovery: The first pass scans the entire codebase to identify all top-level declarations. Using Tree-sitter queries, the analyzer finds all struct, class, enum, type_alias, and function_definition nodes. For each declaration, it creates an entry in a global symbol table, mapping the declared name to its AST node, its type, and the file path where it is defined. This pass builds a complete inventory of all named entities in the project.
Pass 2: Import Resolution: The second pass focuses on module boundaries. The analyzer processes all import (Python, JS/TS) and use (Rust) statements. For each imported symbol, it must resolve the module path to a file on disk. It then parses the target module (if not already processed in Pass 1) and looks up the imported symbol in that module's symbol table. This step establishes the links in the module dependency graph, which is essential for cross-module type flow analysis.22
Pass 3: Local Scope and Variable Analysis: The final pass performs a detailed, intra-procedural analysis. The analyzer performs a depth-first traversal of each function body. As it enters new scopes (e.g., function bodies, loops, conditional blocks), it creates nested symbol tables. When it encounters a variable declaration, it determines the variable's type from its type annotation or by inferring it from the initializer expression. When an identifier is used, the analyzer resolves it by searching for its definition, starting in the current scope and moving up the scope chain until a match is found in a local, module-level, or global symbol table.21

1.4 An Algorithmic Framework for Tracking Type Flow

With the semantic infrastructure of symbol tables and resolved scopes in place, the analyzer can begin to track the flow of types.
Intra-procedural Analysis: Within a single function, type flow is relatively straightforward.
In an assignment like let y = x;, the type of y is resolved to be the same as the type of x.
In an assignment like let z = f();, the type of z is the resolved return type of the function f.
For an expression like a + b, the resulting type depends on the types of a and b and the language's rules for the + operator.
Inter-procedural and Inter-module Analysis: Type flow across function calls is the primary mechanism by which types propagate through a program.
When a function g(x) is called, the type of the argument x flows into the corresponding parameter of the function g.
The resolved return type of g flows out and becomes the type of the g(x) call expression.
To perform this analysis systematically, the analyzer implicitly or explicitly constructs a call graph, which maps every function call site to the function(s) it could potentially invoke.18 For cross-module calls, this resolution depends on the import graph built during Pass 2.

1.5 Resolving Advanced Type Constructs

Modern type systems include features like generics and aliases that require special handling during analysis.

1.5.1 Handling Generic Types and Type Parameters

Generic types allow a function or data structure to be defined over one or more placeholder types. For example, a generic class Queue in Python can hold elements of any type T.23
The analysis must treat the type parameter T as a variable. When a concrete instance of the generic type is created, such as q: Queue[int], the analyzer must perform a substitution. For this specific instance q, every occurrence of T in the Queue class's method signatures is replaced with int. For example, the method pop() -> T would be resolved to pop() -> int for this instance. This requires the analyzer to maintain a mapping from each generic definition to all of its concrete instantiations found in the code.24

1.5.2 Unwrapping Type Aliases and Wrapper Types

Type aliases and wrapper types are two distinct mechanisms for creating new names for types, and they must be handled differently.
Type Aliases: A type alias is purely a syntactic convenience that gives a new name to an existing type, such as Url = str in Python or using Point = (int, int); in C#.24 They do not create a new, distinct type. During analysis, these aliases must be fully resolved to their underlying concrete types. The analyzer should maintain a table of all type aliases. When an alias is encountered, it is substituted with its definition. This substitution must be recursive, as an alias may refer to another alias.26 The static analysis tool should effectively see through the alias as if it were never there.24
Wrapper Types (The Newtype Pattern): In contrast, the Newtype pattern, common in languages like Rust, uses a struct or tuple struct to wrap an existing type, for example, pub struct UserId(i64);.27 This is fundamentally different from a type alias.
UserId is a new, distinct type that is incompatible with i64. This pattern is a powerful abstraction mechanism used to enforce type safety (preventing a ProductId(i64) from being used where a UserId(i64) is expected) and to encapsulate implementation details by exposing a limited public API on the wrapper type.28 At runtime, this is often a zero-cost abstraction, meaning the wrapper has no memory or performance overhead.31 For the leaky abstraction detector, however, the semantic distinction is paramount. A leak occurs if a public API exposes the inner
i64 instead of the abstract UserId type.

Section 2: Constructing and Analyzing the Type Dependency Graph

The culmination of the parsing and semantic analysis phases is the construction of a Type Dependency Graph. This graph is the central data structure for our analysis, providing a holistic, architectural view of how types relate to one another across the entire codebase. By analyzing the structure of this graph, we can identify modules with high coupling and detect the cross-boundary flows that signify leaky abstractions.

2.1 Graph Representation: Modeling Types as Nodes and Dependencies as Edges

The dependency graph is a directed graph where nodes represent types and edges represent dependencies between them.32
Nodes: Each unique type definition discovered during the analysis—including classes, structs, enums, interfaces, traits, and type aliases—is represented as a node in the graph. Primitive types (e.g., int, str, bool) can also be included as fundamental nodes. Each node should store metadata, such as the type's fully qualified name, the file path where it is defined, and its assigned architectural role (e.g., persistence, api_contract).
Edges: A directed edge from TypeA to TypeB signifies that the definition or usage of TypeA depends on TypeB. Such dependencies arise from several common language constructs 12:
Composition/Aggregation: A type contains another type as a field or member.
Example: struct TypeA { field: TypeB } creates an edge TypeA -> TypeB.
Inheritance/Subtyping: A type extends or implements another type.
Example: class TypeA extends TypeB creates an edge TypeA -> TypeB.
Function Signatures: A function's return type depends on its parameter types, and the function itself depends on both.
Example: fn f(p: TypeB) -> TypeA creates edges from the function f to TypeA and TypeB. It also implies a dependency of the return type on the parameter types.
Generic Instantiation: A generic type is instantiated with a concrete type argument.
Example: let x: List<User> creates a dependency from the concrete type List<User> to the User type.

2.2 A General Algorithm for Graph Construction from ASTs

The construction of the type dependency graph is integrated into the multi-pass analysis described in the previous section.
Initialize Graph: After parsing all source files, create an empty graph data structure.
Populate Nodes: Following the declaration discovery pass (Pass 1), iterate through the global symbol table. For each type definition found, add a corresponding node to the graph, enriching it with metadata like its name, file path, and architectural role (derived from configuration or convention).
Add Dependency Edges: After all nodes have been added, perform another pass over all type definitions to establish their relationships. For each type T in the graph:
Analyze Fields/Members: If T is a composite type (like a struct or class), iterate through its fields. For each field, resolve its type F using the symbol tables. Add a directed edge from the node for T to the node for F.
Analyze Inheritance: If T inherits from or implements other types, resolve these parent types (P1, P2,...). Add directed edges from T to each parent P.
Analyze Function Signatures: If T is a function type or a type with methods, analyze the signatures of those functions. For each function, resolve its parameter types and return type. Add edges representing these dependencies.
Analyze Aliases and Generics: For a type alias A = B, add an edge A -> B. For a generic instantiation G<T>, add an edge from the specific instantiation to the type argument T.
This process systematically builds a comprehensive map of all structural type dependencies within the codebase, providing a bird's-eye view of its architecture.32

2.3 Data Structures for Scalable Dependency Graphs in Large Codebases

The choice of the underlying data structure to represent the graph is a critical engineering decision that directly impacts the performance and scalability of the analysis tool, especially for large, enterprise-scale codebases. A naive implementation can lead to excessive memory consumption and slow traversal times.35
A simple Adjacency List, which maps each node to a list of its neighbors, is flexible and easy to implement. However, for very large graphs, more specialized structures offer significant performance benefits. Compressed Sparse Row (CSR) is a format optimized for static, read-only graphs. It uses three arrays to represent the graph structure, which is extremely cache-friendly and allows for very fast traversals. Its main drawback is that updates (adding or removing edges) are expensive, often requiring a complete rebuild of the structure.36
Given that source code is constantly evolving, a purely static representation may be insufficient for tools that need to provide incremental analysis, such as those integrated into an IDE. This has led to the development of dynamic graph data structures that balance read performance with update efficiency:
CSR++: This hybrid data structure combines the high-performance, array-based layout of CSR for read-only analytics with the flexibility of adjacency lists to handle mutations. It aims to provide the "best of both worlds," enabling fast graph traversals while also supporting efficient, in-place updates with low memory overhead.37
DHB (Dynamic Hierarchical Block): Another advanced data structure designed for dynamic graphs, DHB uses a block-based approach to store edges. It has been shown to outperform traditional adjacency arrays for dynamic operations while introducing only a small overhead compared to static CSR for traversals.36
MDG (Multiversion Dependency Graph): This is a more semantically rich data structure that explicitly models the state evolution of objects over time by creating new "versions" of nodes upon updates. While exceptionally powerful for deep analyses like security vulnerability detection (e.g., tracking tainted data flow through object mutations), its complexity may be unnecessary for the more direct task of detecting type leakage across module boundaries.38
The choice between these structures represents a fundamental architectural trade-off. For a CI tool that performs a full analysis on each run, a static CSR format may be optimal. For an IDE plugin that must respond to every keystroke, a dynamic structure like CSR++ or DHB is essential to support incremental updates without re-analyzing the entire project.13
Data Structure
Read/Traversal Performance
Update Performance
Memory Usage (Static)
Memory Usage (Under Updates)
Primary Use Case
Adjacency List
Moderate
Good
Moderate
Moderate
General purpose, small to medium graphs
CSR
Excellent
Poor (requires rebuild)
Low
High (on rebuild)
Batch static analysis, read-only workloads
CSR++
Excellent
Good
Low
Low (proportional to mutations)
High-performance incremental analysis
DHB
Very Good
Excellent
Low
Low
General-purpose dynamic graph processing
MDG
Good
Good
High
High
Deep semantic/vulnerability analysis

Table 2.1: A comparative analysis of graph data structures suitable for static code analysis, highlighting the trade-offs between performance, memory, and dynamism.

2.4 Cycle Detection and Resolution Strategies

A cycle in the type dependency graph indicates a recursive type definition, for example, struct A { b: B } and struct B { a: A }. In many statically typed languages, such direct recursion is illegal because it would imply an infinitely large data structure. Such definitions are only valid if an indirection, like a pointer or a Box, is used to break the cycle.
Cycles can be detected algorithmically during graph analysis. One common method is to perform a topological sort of the graph. A topological sort produces a linear ordering of nodes such that for every directed edge from node u to node v, u comes before v in the ordering. This is only possible if the graph is a Directed Acyclic Graph (DAG).32 An iterative algorithm for topological sorting, which repeatedly removes nodes with an in-degree of zero, will terminate with nodes still left in the graph if and only if a cycle exists.33 Another approach is to use a
Depth-First Search (DFS) traversal. If the DFS encounters a node that is already in the current recursion stack (a "back edge"), a cycle has been detected.32
For a leaky abstraction detector, simply identifying and reporting the types involved in a cycle is often sufficient. These constructs are inherently complex and can be a source of bugs, so flagging them provides value to the developer. The analysis of the cycle can then be terminated, preventing infinite loops in the analyzer itself. The presence of cycles between modules (not just types) is a particularly strong indicator of poor architectural design and high coupling, a condition the tool should flag as a high-priority architectural smell.3

Section 3: Language-Specific Implementation Challenges and Strategies

While the general principles of AST parsing and dependency graph construction are universal, their practical application must be tailored to the unique characteristics of each programming language's type system. A "one-size-fits-all" analyzer is not feasible; robust detection requires language-specific modules that can handle the nuances of Rust, Python, and JavaScript/TypeScript.

3.1 Rust: Navigating Ownership, Borrows, and the Trait System

Rust's type system is distinguished by its strict compile-time enforcement of memory safety through ownership, borrowing, and lifetimes. An effective analyzer for Rust must model these core concepts.

3.1.1 Modeling Ownership Semantics in the Dependency Graph

In Rust, every value has a single, unique owner.40 How a function interacts with a value—whether it takes ownership, borrows it immutably, or borrows it mutably—is a fundamental part of its contract. For example, a function signature
fn process(data: String) is semantically different from fn inspect(data: &String). The former consumes the String, while the latter only borrows it temporarily.
This distinction is too important to ignore. A simple type dependency edge is insufficient. The dependency graph for Rust must be enriched with metadata on its edges to capture these ownership semantics. For a dependency from a function f to a type T, the edge should be annotated as one of the following:
Owns: The function takes ownership of a value of type T.
BorrowsShared (&): The function takes an immutable reference to T.
BorrowsMutable (&mut): The function takes a mutable reference to T.
Capturing this information is critical for understanding a function's true impact and for accurately identifying leaks. For instance, returning a raw pointer or an index into an owned data structure can be a form of leaky abstraction, as it bypasses the borrow checker's safety guarantees.42

3.1.2 Differentiating Trait Objects from Concrete Implementations

Rust's traits are a mechanism for defining shared behavior, analogous to interfaces in other languages.44 A function can operate on a concrete type that implements a trait (e.g.,
fn pet(animal: Dog)) or on an abstract trait object (e.g., fn pet(animal: &dyn Animal)).
Using a trait object is a deliberate act of abstraction. It decouples the function from any specific implementation, allowing it to work with any type that satisfies the Animal trait contract. A leaky abstraction occurs when a function that should operate on the abstract dyn Animal instead exposes a concrete implementation type like Dog in its public signature. This couples the caller to a specific implementation detail, defeating the purpose of the trait-based abstraction.
The analyzer must be ableto distinguish these cases. It needs to parse the dyn Trait syntax and represent trait objects as distinct nodes in the dependency graph. The tool's heuristics should then flag public functions that return a concrete type when a more abstract trait object would be more appropriate for maintaining a clean boundary.

3.2 Python: Taming Dynamicism with Type Hints

Python's dynamic nature presents a different set of challenges. Types are determined at runtime, and type annotations are optional hints rather than compile-time guarantees.45

3.2.1 A "Best-Effort" Analysis Model for Duck Typing and Type Hints

Given Python's dynamism, a perfect, provably correct type analysis is impossible. The analyzer must therefore adopt a "best-effort" or "optimistic" strategy that leverages type hints wherever they are available.47
When a function or variable is annotated (e.g., name: str), the analyzer should trust that hint as the ground truth for that part of the analysis.24 For code without annotations, the analyzer can perform limited type inference. For example, from the statement
x = 10, it can infer that the type of x is int within the current scope. However, it must be aware that a subsequent assignment could change the type of x.
The typing.Any type is an explicit escape hatch from the type system. When a value is annotated as Any, it is compatible with all types, and static analysis on it effectively ceases.24 The analyzer should recognize
Any and understand that it introduces a point of uncertainty in the type flow graph.

3.2.2 Resolving Types from Third-Party Stubs and Modules

A significant portion of any real-world Python project's dependencies are on third-party libraries (e.g., Django, NumPy, pandas). The types for these libraries are often not available in the source code itself but are provided in separate stub files with a .pyi extension.
To resolve types like pandas.DataFrame, the analyzer must be configured to search for and parse these .pyi files. This involves looking in standard locations, such as the project's virtual environment and the typeshed repository, which contains type stubs for the standard library and many popular third-party packages.48 Without this capability, the analyzer would be blind to a large portion of the types flowing through the application.

3.3 JavaScript & TypeScript: The Duality of Prototypes and Static Types

JavaScript and TypeScript present a unique duality. TypeScript provides a powerful, static type system, but it is ultimately a layer on top of JavaScript's dynamic, prototype-based runtime.

3.3.1 Analyzing the Prototype Chain for Type Information

In JavaScript, inheritance is achieved through the prototype chain. When a property is accessed on an object, the JavaScript engine first looks for it on the object itself. If not found, it looks on the object's internal prototype, and so on, up the chain until it reaches null.49 The
class syntax introduced in modern JavaScript and used extensively in TypeScript is merely syntactic sugar over this underlying prototypal inheritance mechanism.50
For a pure JavaScript analysis, a tool would need to simulate this prototype lookup behavior. For a TypeScript project, the static type information provided by the language is usually sufficient. However, an understanding of the prototype model is crucial for analyzing the interaction between TypeScript code and plain JavaScript libraries, where static type information may be incomplete or absent.

3.3.2 Leveraging the TypeScript Compiler API and Declaration Files

The TypeScript type system is exceptionally complex, featuring advanced concepts like conditional types, mapped types, and powerful inference. Re-implementing this entire system from scratch using only Tree-sitter would be a monumental and error-prone undertaking.20
A more pragmatic and robust strategy is a hybrid approach. Tree-sitter can be used for the initial, fast parsing of .ts and .tsx files to build the AST and perform high-level pattern matching.8 However, for the definitive resolution of complex types, the analyzer should integrate with the official
TypeScript Compiler API. This API exposes the compiler's internal data structures, providing access to a fully resolved and type-checked representation of the program. This allows the tool to query the compiler for the precise type of any expression, effectively leveraging the work of the TypeScript team instead of reinventing it. This approach also naturally handles types from external libraries, as the compiler API resolves them from their corresponding declaration files (.d.ts).
Language
Core Challenge
Primary Analysis Strategy
Key Data Sources (beyond source)
Rust
Ownership, borrowing, lifetimes, and the trait system are integral to the type system.
Enrich dependency graph edges with ownership metadata (Owns, BorrowsShared, BorrowsMutable). Distinguish between concrete types and abstract trait objects.
Cargo.toml for dependency information.
Python
Dynamic typing, optional type hints, and the Any type introduce ambiguity.
Adopt a "best-effort" model that trusts type hints when present and performs limited inference otherwise.
.pyi stub files (from typeshed and installed packages) for third-party library types.
TypeScript
Extremely complex type system layered on top of JavaScript's dynamic, prototype-based runtime.
Use a hybrid approach: Tree-sitter for initial parsing and pattern matching, but leverage the official TypeScript Compiler API for authoritative type resolution.
.d.ts declaration files for library types, tsconfig.json for compiler configuration.

Table 3.1: A summary of the primary challenges and recommended analysis strategies for each target language.

Section 4: Automated Detection of Leaky Abstraction Patterns

With a fully constructed and resolved type dependency graph, the analyzer can proceed to its primary goal: detecting leaky abstractions. This requires translating the abstract concept of a "leak" into concrete, machine-verifiable heuristics. These heuristics are based on identifying when types associated with a module's internal implementation details are exposed in the public interface of another module that serves a different architectural purpose.

4.1 Defining Leakage Heuristics for Automated Tooling

The central heuristic for detecting leaky abstractions is as follows: A type defined in a module with an "internal" architectural role should not appear in the public interface (e.g., function parameters or return values) of a module with an "external" or "API" architectural role.
To apply this heuristic, the tool must first be able to classify modules according to their architectural role. This is information that cannot be inferred from the source code alone; it reflects developer intent. Therefore, the analyzer must be configured with this architectural model. This can be achieved in two ways:
Explicit Configuration: A configuration file (e.g., architecture.yaml) can map directories or file patterns to roles. For example:
YAML
roles:
  persistence:
    - src/db/
    - src/models/entities/
  business_logic:
    - src/services/
    - src/domain/
  api_controller:
    - src/controllers/
    - src/api/routes/
  api_contract:
    - src/api/schemas/
    - src/dtos/


Convention-Based Heuristics: The tool can rely on common directory naming conventions (e.g., any directory named db, entities, or persistence is assigned the persistence role).
This classification is fundamental. A leak is not an intrinsic property of a type but rather a property of its inappropriate use across architectural boundaries.3

4.2 Pattern 1: Persistence Models in Public APIs

This is arguably the most common and damaging form of leaky abstraction in multi-layered applications. It occurs when a type that directly maps to a database table (an ORM entity, a database record) is used in the signature of a public API endpoint. This tightly couples the API's data contract to the internal database schema, making it impossible to evolve the database without potentially breaking external clients.2
Leaky Example (Python with FastAPI and SQLAlchemy):
Python
# Location: src/db/models.py
# Role: persistence
class UserInDB(Base):  # SQLAlchemy ORM model
    __tablename__ = "users"
    id: Mapped[int] = mapped_column(primary_key=True)
    username: Mapped[str]
    hashed_password: Mapped[str]  # Internal detail

# Location: src/api/routes.py
# Role: api_controller
@app.get("/users/{user_id}")
def get_user(user_id: int) -> UserInDB:  # <-- LEAK! Exposes internal model with hashed_password
    #... logic to fetch user from database...
    return db_user


Proper Example (Using a Data Transfer Object):
The correct approach is to introduce a Data Transfer Object (DTO). A DTO is a simple data structure whose sole purpose is to act as the data contract for a specific layer, such as an API. It is decoupled from the persistence model and contains only the data that should be publicly exposed.52
Python
# Location: src/api/schemas.py
# Role: api_contract
class UserPublic(BaseModel):  # Pydantic DTO model
    id: int
    username: str
    # No hashed_password field

# Location: src/api/routes.py
# Role: api_controller
@app.get("/users/{user_id}")
def get_user(user_id: int) -> UserPublic:  # <-- CORRECT! Returns the public DTO.
    db_user = get_user_from_db(user_id)
    # Manually or automatically map the entity to the DTO
    return UserPublic.from_orm(db_user)


Detection Algorithm:
Using the architectural model, classify the module src/db/models.py as persistence and src/api/routes.py as api_controller.
In the first pass, identify the class UserInDB and record its role as persistence.
In the analysis pass, resolve the return type of the function get_user to the canonical UserInDB type.
Apply the heuristic: A function in an api_controller module returns a type from a persistence module. This is a violation.
Report the leak, specifying the function, the problematic type, and their respective architectural roles.

4.3 Pattern 2: Framework-Specific Types in Business Logic

This leak occurs when core business logic becomes dependent on types defined by a specific framework (e.g., a web framework, a message queue library). This couples the business logic to that framework, making it difficult to test in isolation, reuse in a different context (e.g., a command-line tool), or migrate to a new framework in the future.3
Leaky Example (Rust with Actix-Web):
Rust
// Location: src/services/user_service.rs
// Role: business_logic
use actix_web::{HttpRequest, Error}; // Framework-specific import

// Leaky: Core business logic depends directly on the web request object.
pub fn is_user_premium(req: &HttpRequest) -> Result<bool, Error> { // <-- LEAK!
    // This logic is now untestable without creating a mock HttpRequest.
    let api_key = req.headers().get("X-API-KEY");
    //... logic to validate key and check subscription status...
}


Proper Example (Applying Dependency Inversion):
The solution is to apply the Dependency Inversion Principle. The boundary layer (the API controller) should extract the necessary primitive data from the framework-specific object and pass that data to the business logic layer.
Rust
// Location: src/services/user_service.rs
// Role: business_logic
// Correct: The function now depends only on primitive types, not the framework.
pub fn is_user_premium(api_key: Option<&str>) -> bool { // <-- CORRECT!
    // This logic is pure and easily testable.
    //... logic to validate key and check subscription status...
}

// Location: src/controllers/user_controller.rs
// Role: api_controller
pub async fn check_premium_status(req: HttpRequest) -> impl Responder {
    // The controller is responsible for interacting with the framework.
    let key = req.headers().get("X-API-KEY").and_then(|h| h.to_str().ok());

    // It calls the business logic with primitive data.
    if user_service::is_user_premium(key) {
        HttpResponse::Ok().finish()
    } else {
        HttpResponse::Forbidden().finish()
    }
}


Detection Algorithm:
Maintain a list of known framework namespaces (e.g., actix_web, django.http, express). Classify any type from these namespaces as having the framework role.
Identify the module src/services/user_service.rs as having the business_logic role.
Analyze the function is_user_premium and resolve the type of its parameter req to actix_web::HttpRequest.
Apply the heuristic: A function in a business_logic module has a parameter whose type has the framework role. This is a violation.
Report the leak.

4.4 Pattern 3: Implementation Details in Public Signatures

This pattern involves exposing a raw, concrete collection type (like a HashMap or a List) or other low-level implementation detail in a public API. This practice makes the implementation rigid. If the internal storage mechanism needs to change (e.g., from a HashMap to a BTreeMap for ordering), it becomes a breaking change for all consumers of the API.
Leaky Example (Rust):
Rust
// In a public library crate
use std::collections::HashMap;

// Leaky: Exposes the specific choice of HashMap as the implementation.
// Callers might start relying on HashMap-specific properties.
pub fn get_feature_flags() -> HashMap<String, bool> { // <-- LEAK!
    //...
}


Proper Example (Using the Newtype Pattern for Abstraction):
A better approach is to wrap the concrete collection in a newtype. This newtype becomes the public interface, hiding the implementation detail. It provides a stable API, and the internal collection type can be changed without breaking downstream code.27
Rust
use std::collections::HashMap;

// A dedicated public type that encapsulates the collection.
pub struct FeatureFlags(HashMap<String, bool>); // The inner field is private.

impl FeatureFlags {
    // The public API provides methods to interact with the data.
    pub fn is_enabled(&self, flag_name: &str) -> bool {
        self.0.get(flag_name).copied().unwrap_or(false)
    }
}

// Correct: The function returns the abstract FeatureFlags type.
pub fn get_feature_flags() -> FeatureFlags { // <-- CORRECT!
    //...
}


Detection Algorithm:
Maintain a configurable list of "low-level implementation types" (e.g., HashMap, Vec, BTreeMap, dict, list, Array).
Identify all functions that are part of the module's public API (e.g., marked with pub in Rust).
Flag any public function whose return type is one of these low-level types, especially if the type is generic (e.g., HashMap<K, V>).
This heuristic is less precise than the role-based ones and may generate false positives, so it should be presented to the user with a lower confidence level or allow for configuration to ignore specific cases. Nonetheless, it is a powerful indicator of a potential failure to abstract.1

Section 5: A Practical Guide to Tree-sitter Queries for Type Analysis

This section provides concrete Tree-sitter query patterns for implementing the discovery phase of the leaky abstraction detector. These queries form the foundation of the analysis, enabling the tool to extract the necessary syntactic information from the source code across different languages. The application logic then uses the nodes captured by these queries to build the semantic model and type dependency graph.

5.1 A Primer on Tree-sitter Query Syntax

Tree-sitter queries use an S-expression syntax to define patterns that match nodes in an AST. The key components of the syntax are:
Node Patterns: A node is matched by its type in parentheses, e.g., (function_declaration). Anonymous nodes (like operators) are matched by their string literal in quotes, e.g., ("+").54
Captures: A matched node can be "captured" by assigning it a name with an @ prefix, e.g., (identifier) @function.name. This makes the captured node available to the calling application.17
Fields: To make patterns more specific, you can match on named fields of a node. The syntax is (node_type field_name: (child_node_type)). For example, (function_declaration name: (identifier)) matches a function declaration and specifically targets its name field, which must be an identifier node.54
Predicates: Predicates are functions that add conditional logic to a match. They are enclosed in (#...). A common predicate is #eq?, which checks if the text of a captured node equals a given string, e.g., ((identifier) @var.name (#eq? @var.name "self")).17

5.2 Queries for Function Signatures

These queries are used to find function definitions and extract their names, parameters, and return types.
Rust:
Scheme
; Captures the name and return type of a function item.
(function_item
  name: (identifier) @function.name
  return_type: (_) @function.return_type) @function.definition

This query targets function_item nodes. It captures the identifier in the name field as @function.name and captures whatever node is in the return_type field as @function.return_type. The wildcard (_) matches any node.19
Python:
Scheme
; Captures the name and return type annotation of a function.
(function_definition
  name: (identifier) @function.name
  return_type: (type) @function.return_type) @function.definition

This query is similar but uses the node types from the Python grammar. It finds function_definition nodes and captures the identifier for the name and the type node for the return type annotation.55
TypeScript:
Scheme
; Captures the name and return type annotation of a function.
(function_declaration
  name: (identifier) @function.name
  return_type: (type_annotation
    ":"
    (_) @function.return_type
  )
) @function.definition

The TypeScript grammar is slightly more nested. This query finds a function_declaration, captures its name, and then looks inside the return_type field for a type_annotation node, capturing the type itself.16

5.3 Queries for Type Definitions and Aliases

These queries are used to populate the symbol table with all the user-defined types in the codebase.
Rust struct definition:
Scheme
(struct_item
  name: (type_identifier) @type.name) @type.definition


Python class definition:
Scheme
(class_definition
  name: (identifier) @type.name) @type.definition


TypeScript type alias:
Scheme
; Captures a type alias declaration, its name, and its underlying value.
(type_alias_declaration
  name: (type_identifier) @alias.name
  value: (_) @alias.value) @alias.definition

This query is essential for resolving aliases. It captures the name of the alias and the entire node representing the type it is an alias for, allowing for recursive resolution.60

5.4 Walkthrough: Implementing a Simple Leak Detector for a Python API Endpoint

This step-by-step example demonstrates how to combine the queries and analysis logic to detect the "Persistence Model in Public API" pattern from Section 4.2.
Configuration: The analyzer starts by loading an architectural model:
YAML
# module_roles.yaml
roles:
  persistence: ['db/']
  api_controller: ['api/']


Pass 1: Type Discovery: The analyzer runs the class_definition query on all files.
When parsing db/models.py, it finds class UserInDB....
It creates an entry in the global symbol table: {'UserInDB': {path: 'db/models.py', role: 'persistence', node: <Node object>}}.
Pass 2: Function Analysis: The analyzer runs the function_definition query on api/routes.py.
It finds the get_user function and gets two captures: @function.name (node with text "get_user") and @function.return_type (node with text "UserInDB").
Resolution: The application logic takes the text of the @function.return_type capture, which is "UserInDB". It then performs a lookup in the global symbol table for "UserInDB". The lookup succeeds, returning the entry created in Step 2.
Heuristic Check: The analyzer now has all the necessary information:
The function get_user is located in a file (api/routes.py) that maps to the api_controller role.
Its return type resolves to UserInDB, which is defined in a file (db/models.py) that maps to the persistence role.
The heuristic (api_controller should not expose persistence types) is violated.
Report Leak: The tool generates a user-facing error message:
Leaky Abstraction Detected in api/routes.py: Function 'get_user' in API controller returns persistence type 'UserInDB' defined in db/models.py.
This walkthrough illustrates the critical interplay between Tree-sitter's syntactic querying and the semantic resolution logic of the application. The queries extract the raw materials, but the application must use its symbol tables and architectural model to connect the dots and identify the leak.
Analysis Task
Rust Query
Python Query
TypeScript Query
Find Function Return Type
(function_item return_type: (_) @return)
(function_definition return_type: (type) @return)
(function_declaration return_type: (type_annotation (_) @return))
Find Function Parameters
(function_item parameters: (parameters) @params)
(function_definition parameters: (parameters) @params)
(function_declaration parameters: (formal_parameters) @params)
Find Struct/Class Definition
(struct_item name: (type_identifier) @name)
(class_definition name: (identifier) @name)
(class_declaration name: (type_identifier) @name)
Find Type Alias
(type_item name: (type_identifier) @name type: (_) @value)
(type_alias_statement name: (type) @name value: (type) @value)
(type_alias_declaration name: (type_identifier) @name value: (_) @value)
Find Imports
(use_declaration) @import
(import_statement) @import (import_from_statement) @import
(import_statement) @import

Table 5.1: A reference of core Tree-sitter query patterns for extracting type-related information from Rust, Python, and TypeScript source code.

Conclusion

The integrity of architectural boundaries is a critical factor in the long-term health and maintainability of a software system. Leaky abstractions, particularly the unintentional exposure of internal types through public interfaces, represent a significant threat to this integrity. They create tight coupling between modules, hinder refactoring, and make systems more brittle. This report has presented a comprehensive methodology for the automated detection of such leaks through static type flow analysis.
The proposed analysis pipeline forms a robust foundation for building a multi-language detector. It begins with syntactic analysis using the Tree-sitter framework to parse source code into an AST. This is followed by a multi-pass semantic analysis phase that builds scope-aware symbol tables, resolves imports, and constructs a global type dependency graph. This graph serves as a semantic model of the codebase's architecture. Finally, the detector applies a set of configurable, role-based heuristics to this graph to identify when a type from an internal module (e.g., persistence) improperly flows across a boundary into a public-facing module (e.g., api_controller).
The primary strength of this approach is its ability to automate the enforcement of architectural principles that are typically left to manual code review. By integrating such a tool into a CI/CD pipeline, teams can receive immediate feedback on architectural defects, preventing them from becoming permanent fixtures of the codebase. The use of Tree-sitter provides a fast, robust, and language-agnostic parsing backend, while the language-specific analysis modules handle the unique complexities of Rust's ownership, Python's dynamicism, and TypeScript's advanced type system.
However, the approach has limitations. Its effectiveness is contingent on the accuracy of the architectural model provided through configuration. The heuristics, while powerful, cannot perfectly capture developer intent and may produce false positives or negatives in edge cases.1 A function returning a
HashMap might be a deliberate design choice, not a leak. The tool must therefore be seen as a guide that highlights potential issues, not as an infallible arbiter of correctness.
This work opens several avenues for future research and development:
Taint Analysis: The core heuristic can be formalized and extended into a more powerful taint analysis framework. Types from "internal" modules can be treated as "taint sources," and public function signatures as "sinks." This would allow for the detection of more subtle, indirect data flows, mirroring techniques used in security analysis.38
Automated Refactoring: A highly advanced version of this tool could go beyond detection and offer automated remediation. Upon detecting a persistence model leaking from an API, it could automatically generate the corresponding DTO class and the necessary mapping code, significantly accelerating the refactoring process.
Machine Learning for Architectural Discovery: The current reliance on manual configuration for defining architectural roles could be reduced. A machine learning model could be trained to analyze a project's dependency graph and code patterns to automatically classify modules into roles like persistence, business_logic, and controller, making the tool easier to adopt and apply to new codebases.
Ultimately, by systematically tracking how types flow across a system, developers can gain unprecedented insight into their software's architecture, enabling them to build more modular, resilient, and maintainable applications.
Works cited
How to identify that code is over abstracted? [closed] - Stack Overflow, accessed July 3, 2025, https://stackoverflow.com/questions/3715625/how-to-identify-that-code-is-over-abstracted
Leaky Abstraction — What Is It?. Spot leakness in your code and see how… | by Bartosz Salwiczek | Better Programming, accessed July 3, 2025, https://betterprogramming.pub/leaky-abstraction-what-is-it-ed0bc84000fd
Common modularization patterns | App architecture | Android ..., accessed July 3, 2025, https://developer.android.com/topic/modularization/patterns
The Law of Leaky Abstractions : r/programming - Reddit, accessed July 3, 2025, https://www.reddit.com/r/programming/comments/5qzc79/the_law_of_leaky_abstractions/
The Law of Leaky Abstractions - Joel on Software, accessed July 3, 2025, https://www.joelonsoftware.com/2002/11/11/the-law-of-leaky-abstractions/
programming languages - Meaning of Leaky Abstraction? - Stack Overflow, accessed July 3, 2025, https://stackoverflow.com/questions/3883006/meaning-of-leaky-abstraction
The Law of Leaky Abstractions - Hacker News, accessed July 3, 2025, https://news.ycombinator.com/item?id=19906796
A guide to static analysis in JavaScript and TypeScript - Mattermost, accessed July 3, 2025, https://mattermost.com/blog/a-guide-to-static-analysis-in-javascript-and-typescript/
AST: Hierarchical Code Structure for Analysis | Lenovo US, accessed July 3, 2025, https://www.lenovo.com/us/en/glossary/ast/
Abstract syntax tree - Wikipedia, accessed July 3, 2025, https://en.wikipedia.org/wiki/Abstract_syntax_tree
Python's Abstract Syntax Trees (AST): Manipulating Code at Its Core - Codedamn, accessed July 3, 2025, https://codedamn.com/news/python/python-abstract-syntax-trees-ast-manipulating-code-core
Dependency Graph in Compiler Design - GeeksforGeeks, accessed July 3, 2025, https://www.geeksforgeeks.org/compiler-design/dependency-graph-in-compiler-design/
Tree-sitter: Introduction, accessed July 3, 2025, https://tree-sitter.github.io/
I don't get why treesitter is a big deal, and at this point I'm afraid to ask : r/neovim - Reddit, accessed July 3, 2025, https://www.reddit.com/r/neovim/comments/15jxqgn/i_dont_get_why_treesitter_is_a_big_deal_and_at/
tree-sitter explained - YouTube, accessed July 3, 2025, https://www.youtube.com/watch?v=09-9LltqWLY
TypeScript grammar for tree-sitter - GitHub, accessed July 3, 2025, https://github.com/tree-sitter/tree-sitter-typescript
Unraveling Tree-Sitter Queries: Your Guide to Code Analysis Magic - DEV Community, accessed July 3, 2025, https://dev.to/shrsv/unraveling-tree-sitter-queries-your-guide-to-code-analysis-magic-41il
5 Powerful Ways to Use Tree-sitter in Your Next Project | by Istiaq Ahmed Fahad - Medium, accessed July 3, 2025, https://medium.com/@ahmedfahad04/5-powerful-ways-to-use-tree-sitter-in-your-next-project-50e17c1f7055
Digging Deeper into Code with Tree-Sitter: How to Query Your Syntax Tree, accessed July 3, 2025, https://dev.to/shailendra53/digging-deeper-into-code-with-tree-sitter-how-to-query-your-syntax-tree-3i1
Dossier: A tree-sitter based multi-language source code and docstring parser : r/rust - Reddit, accessed July 3, 2025, https://www.reddit.com/r/rust/comments/1980y0j/dossier_a_treesitter_based_multilanguage_source/
How to resolve types in Python code with Tree-sitter? - Stack Overflow, accessed July 3, 2025, https://stackoverflow.com/questions/78176948/how-to-resolve-types-in-python-code-with-tree-sitter
Dependency analysis in Scripted - Spring, accessed July 3, 2025, https://spring.io/blog/2012/11/20/dependency-analysis-in-scripted/
Python 3.12 Preview: Static Typing Improvements, accessed July 3, 2025, https://realpython.com/python312-typing/
PEP 484 – Type Hints | peps.python.org, accessed July 3, 2025, https://peps.python.org/pep-0484/
Alias any type - C# feature specifications | Microsoft Learn, accessed July 3, 2025, https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/proposals/csharp-12.0/using-alias-types
Generic Tips Part 3: Avoid Repeating Type Expressions - Effective TypeScript, accessed July 3, 2025, https://effectivetypescript.com/2021/01/20/gentips-3-aliases/
The Newtype Pattern in Rust, accessed July 3, 2025, https://www.worthe-it.co.za/blog/2020-10-31-newtype-pattern-in-rust.html
Advanced Types - The Rust Programming Language - MIT, accessed July 3, 2025, https://web.mit.edu/rust-lang_v1.25/arch/amd64_ubuntu1404/share/doc/rust/html/book/second-edition/ch19-04-advanced-types.html
Refactoring in Rust: Abstraction with the Newtype Pattern, accessed July 3, 2025, https://oida.dev/refactoring-rust-abstraction-newtype/
Master Hexagonal Architecture in Rust - How To Code It, accessed July 3, 2025, https://www.howtocodeit.com/articles/master-hexagonal-architecture-rust
How does Rust implement Zero-cost abstraction for NewTypes Pattern - Stack Overflow, accessed July 3, 2025, https://stackoverflow.com/questions/75614715/how-does-rust-implement-zero-cost-abstraction-for-newtypes-pattern
Dependency graph - Wikipedia, accessed July 3, 2025, https://en.wikipedia.org/wiki/Dependency_graph
Dependency graph resolution algorithm in Go - Marin Atanasov Nikolov, accessed July 3, 2025, https://dnaeon.github.io/dependency-graph-resolution-algorithm-in-go/
Safely restructure your codebase with Dependency Graphs - Understand Legacy Code, accessed July 3, 2025, https://understandlegacycode.com/blog/safely-restructure-codebase-with-dependency-graphs/
Efficient Program Analyses that Scale to Large Codebases - eScholarship.org, accessed July 3, 2025, https://www.escholarship.org/content/qt68f0q810/qt68f0q810.pdf
A Fast Data Structure for Dynamic Graphs Based on Hash-Indexed Adjacency Blocks - DROPS, accessed July 3, 2025, https://drops.dagstuhl.de/storage/00lipics/lipics-vol233-sea2022/LIPIcs.SEA.2022.11/LIPIcs.SEA.2022.11.pdf
A Scalable Data Structure for Efficient Graph Analytics and In-Place ..., accessed July 3, 2025, https://www.mdpi.com/2306-5729/8/11/166
Efficient Static Vulnerability Analysis for JavaScript ... - andrew.cmu.ed, accessed July 3, 2025, https://www.andrew.cmu.edu/user/liminjia/research/papers/graphjs-pldi24.pdf
How to resolve dependencies between modules within multi-module project?, accessed July 3, 2025, https://stackoverflow.com/questions/14694139/how-to-resolve-dependencies-between-modules-within-multi-module-project
Ownership & Borrowing in Rust - Brandon Wofford - Medium, accessed July 3, 2025, https://bwoff.medium.com/ownership-borrowing-in-rust-40ab69521247
Understanding Ownership - The Rust Programming Language, accessed July 3, 2025, https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html
For Beginners: An interesting article about Ownership and Borrowing - Rust Users Forum, accessed July 3, 2025, https://users.rust-lang.org/t/for-beginners-an-interesting-article-about-ownership-and-borrowing/108718
Understanding Rust: ownership, borrowing, lifetimes | by Sergey Bugaev - Medium, accessed July 3, 2025, https://medium.com/@bugaevc/understanding-rust-ownership-borrowing-lifetimes-ff9ee9f79a9c
Abstraction in Rust and Python. Simple examples - DEV Community, accessed July 3, 2025, https://dev.to/antonov_mike/abstraction-in-rust-and-python-simple-examples-5dd8
A Comprehensive Guide to Python Type Hints - Codefinity, accessed July 3, 2025, https://codefinity.com/blog/A-Comprehensive-Guide-to-Python-Type-Hints
Dynamic Typing - Python - GeeksforGeeks, accessed July 3, 2025, https://www.geeksforgeeks.org/python/dynamic-typing-python/
Type Hinting in Python: Bridging the Gap Between Dynamic and Static Typing - Medium, accessed July 3, 2025, https://medium.com/@admnfree01/type-hinting-in-python-bridging-the-gap-between-dynamic-and-static-typing-1bc09f56e7c3
Python Type Hints and why you should use them. - Reddit, accessed July 3, 2025, https://www.reddit.com/r/Python/comments/1iqytkf/python_type_hints_and_why_you_should_use_them/
Inheritance and the prototype chain - JavaScript - MDN Web Docs, accessed July 3, 2025, https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Inheritance_and_the_prototype_chain
Typescript Prototypes: The Secret Behind Inheritance (Explained Simply) | by Bekalu Sisay Iticha | Jun, 2025 | Medium, accessed July 3, 2025, https://medium.com/@bekalusisay2010/javascript-prototypes-the-secret-behind-inheritance-explained-simply-746fa662707d
Typescript generates javascript code for simple class inheritance - Stack Overflow, accessed July 3, 2025, https://stackoverflow.com/questions/22901249/typescript-generates-javascript-code-for-simple-class-inheritance
Difference between Entity and DTO - DEV Community, accessed July 3, 2025, https://dev.to/wagnernegrao/difference-between-entity-and-dto-2pkk
The DTO Pattern: Simplifying Data Transfer | by Liberatoreanita | Medium, accessed July 3, 2025, https://medium.com/@liberatoreanita/the-dto-pattern-simplifying-data-transfer-f99ab4f25c34
Basic Syntax - Tree-sitter, accessed July 3, 2025, https://tree-sitter.github.io/tree-sitter/using-parsers/queries/1-syntax.html
Node — py-tree-sitter 0.24.0 documentation, accessed July 3, 2025, https://tree-sitter.github.io/py-tree-sitter/classes/tree_sitter.Node.html
Treesitter - Neovim docs, accessed July 3, 2025, https://neovim.io/doc/user/treesitter.html
Knee Deep in tree-sitter Queries - Hackerman's Hacking Tutorials, accessed July 3, 2025, https://parsiya.net/blog/knee-deep-tree-sitter-queries/
Diving into Tree-Sitter: Parsing Code with Python Like a Pro - DEV Community, accessed July 3, 2025, https://dev.to/shrsv/diving-into-tree-sitter-parsing-code-with-python-like-a-pro-17h8
tree-sitter-typescript/queries/highlights.scm at master - GitHub, accessed July 3, 2025, https://github.com/tree-sitter/tree-sitter-typescript/blob/master/queries/highlights.scm
ast-grep's Journey to Type Safety in Node API, accessed July 3, 2025, https://ast-grep.github.io/blog/typed-napi.html
tree-sitter-typescript/tsx/src/node-types.json at master - GitHub, accessed July 3, 2025, https://github.com/tree-sitter/tree-sitter-typescript/blob/master/tsx/src/node-types.json
Static Dependence Analysis for Java - Yannis Smaragdakis, accessed July 3, 2025, https://yanniss.github.io/theses/filippakis.pdf
