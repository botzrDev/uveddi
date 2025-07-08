
A Unified Strategy for Multi-Language Dead Code Detection


Section 1: Architectural Foundation: From Source Text to a Unified Code Graph

The cornerstone of any effective, multi-language static analysis system is a robust architectural foundation capable of transforming disparate source code files into a unified, queryable representation. This section outlines a strategy to build this foundation, starting from initial parsing with Tree-sitter, moving to structured symbol management, and culminating in the construction of a comprehensive Code Property Graph (CPG) stored within a high-performance graph database. This architecture is designed not only to support dead code detection but also to serve as an extensible platform for a wide range of future code intelligence tools.

1.1. Parsing with Tree-sitter: Leveraging Incremental Concrete Syntax Trees

The initial and most fundamental step in analyzing source code is parsing: converting a sequence of characters into a structured representation. For a multi-language system targeting Rust, Python, and JavaScript, the choice of parser is critical. The proposed system will leverage Tree-sitter as its universal parsing engine, a decision driven by its performance, robustness, and architectural elegance.1
Tree-sitter is a parser generator tool that creates fast, incremental parsers.3 While the generator itself is written in Rust and its grammar definitions use a JavaScript DSL, it produces highly optimized C parsers that can be compiled to native libraries or WebAssembly.3 This allows consumption from a multitude of host languages through official and third-party bindings, including Rust and Python, which are ideal for the backend implementation of this analysis system.1 A rich ecosystem of mature, pre-existing grammars for Rust, Python, and JavaScript is available, significantly reducing the initial development effort required to support these target languages.6
A key differentiator of Tree-sitter is its generation of a Concrete Syntax Tree (CST) rather than a more traditional Abstract Syntax Tree (AST).9 A CST is a "lossless" representation of the source code, meaning it retains every piece of information from the original file, including syntax trivia such as comments, whitespace, and parentheses.10 While dead code analysis primarily relies on semantic structures, preserving the full CST provides significant advantages for future tooling. For instance, an automated refactoring tool that removes dead code requires this precise syntactic information to rewrite the source file correctly without disrupting formatting.
The most significant architectural advantage of Tree-sitter is its proficiency in incremental parsing.3 When a source file is modified, Tree-sitter can efficiently update the existing syntax tree by re-parsing only the changed regions, reusing the unchanged portions of the previous tree.10 This capability is the bedrock of the system's performance strategy, particularly for interactive use cases within an IDE or rapid feedback cycles in a CI/CD pipeline, as it obviates the need for a full re-parse of every file on minor changes.
The adoption of Tree-sitter is therefore more than a tactical choice for parsing; it is a strategic architectural decision. It provides a unified, high-performance parsing pipeline that is language-agnostic at its core. The system can be designed around a single engine that loads a language-specific Tree-sitter grammar and executes a set of queries against the resulting CST to extract syntactic information.2 This approach offers remarkable extensibility. Supporting a new language in the future would primarily involve providing its Tree-sitter grammar and a corresponding set of symbol-extraction queries, rather than building an entirely new analysis frontend from the ground up. This modularity ensures the system can evolve efficiently to meet new requirements.

1.2. A Multi-Language Symbol Table: Design and Schema

Once the source code is parsed into a CST, the next step is to build a symbol table. The symbol table is a critical data structure that acts as a bridge between the syntactic and semantic analysis phases of the system.13 It associates identifiers (symbols) found in the code—such as variable names, function names, and classes—with their essential attributes, including type, scope, and source location.15
To accurately model the scoping rules of modern programming languages, the system will implement a hierarchical symbol table. Programming languages like Rust, Python, and JavaScript utilize lexical scoping, where the visibility of a name is determined by its location within nested blocks of code. A proven and effective method for representing this is a stack of hash tables.18 Each time the analysis enters a new scope (e.g., a module, a function body, or a control-flow block), a new hash table is pushed onto the stack. When resolving a name, the stack is searched from top to bottom (from the innermost scope to the outermost). This structure correctly handles name resolution, variable shadowing, and ensures that symbols are only accessible within their defined lexical boundaries.15 For implementation, the hash table is the preferred data structure due to its average-case O(1) time complexity for insertion and lookup operations, making it highly efficient for managing the large number of symbols in a typical codebase.14
To support a multi-language analysis, a standardized schema for symbol table entries is required. This ensures that the subsequent analysis stages can operate on a consistent data model, regardless of the source language. Each symbol entry will contain the following attributes:
symbol_id: A unique identifier for the symbol instance across the entire project.
symbol_name: The textual name of the identifier (e.g., my_function).
symbol_kind: An enumeration specifying the nature of the symbol (e.g., Function, Class, Variable, Import, Module, Parameter).
location: A reference to the source location, including file path, and start and end coordinates (line and column).
scope_id: A unique identifier for the lexical scope in which the symbol is defined.
type_info: Information about the symbol's data type. For Rust, this will be a precise type derived from its static type system. For Python and JavaScript, this may be an inferred type, a set of possible types, or simply any where the type cannot be determined statically.
visibility: The visibility modifier of the symbol (e.g., public, private, pub(crate) for Rust).
is_entry_point: A boolean flag, initialized to false, to be set during the entry point detection phase.
is_live: A boolean flag, initialized to false (except for entry points), which will be updated during the final reachability analysis.

1.3. The Code Property Graph: Storing Cross-File Definitions and Usages

While a symbol table is essential for managing scoped declarations, it is not optimized for representing and querying the complex, non-local relationships that span an entire codebase. To address this, the system will model the entire project as a Code Property Graph (CPG).19 A CPG is a powerful, unified data structure that integrates multiple representations of code—including the AST, control-flow information, and data-flow dependencies—into a single, queryable graph.20 This holistic representation is exceptionally well-suited for the kind of whole-program analysis required for dead code detection.
The CPG will be constructed based on a well-defined schema of nodes and edges:
Nodes: Nodes represent the fundamental entities within the code. Each node will be typed and will store attributes derived from the symbol table schema. Core node types include:
File: Represents a source code file.
Module: Represents a language-specific module or namespace.
Class: Represents a class definition.
Function: Represents a function or method definition.
Variable: Represents a variable, constant, or parameter declaration.
Import: Represents an import statement.
Edges: Edges represent the relationships between these entities. Key edge types include:
CONTAINS: A structural relationship (e.g., a File node CONTAINS a Function node).
IMPORTS: Links a File or Module to the symbols or modules it imports.
DEFINES: Connects a scope (like a Function or Module) to the Variable or Function nodes it defines.
REFERENCES: A syntactic-level edge indicating that one code element refers to another by name. This is a precursor to a semantic edge.
CALLS: A semantic edge representing a resolved function or method call from one Function to another.
USES_VARIABLE: A semantic edge linking a Function to a Variable it reads from or writes to.
INHERITS_FROM: Connects a Class node to its parent class(es).
IMPLEMENTS_TRAIT: Connects a Class (or struct in Rust) to a Trait it implements.
AST_PARENT/AST_CHILD: To preserve the fine-grained CST structure within a function body, allowing for detailed intra-procedural analysis if needed in the future.
The construction of the CPG will proceed in two main phases. First, in a Parsing and Symbol Extraction phase, the system will iterate through every source file, using Tree-sitter and a set of language-specific queries to identify all definitions and references.2 This phase populates the CPG with nodes and initial, unresolved
REFERENCES edges. Second, a Linking and Resolution phase will process these REFERENCES. For each reference, it will query the global symbol table to find the corresponding definition, transforming the syntactic REFERENCES edge into a more meaningful semantic edge like CALLS or USES_VARIABLE. This linking phase is where the language-specific module and name resolution logic is applied.

1.4. Database Strategy: Selecting an Efficient Graph Database

Storing and querying a CPG for a large-scale codebase with millions of nodes and edges is computationally intensive and requires a specialized data storage solution. A graph database is the natural and most effective choice for this task.19
While an in-memory graph library could suffice for analyzing small projects in a single run, this approach lacks persistence and does not scale to the size of modern enterprise codebases. A persistent graph database, such as Neo4j or FalkorDB, offers a far more robust and scalable solution.20 These databases are purpose-built for storing property graph models, aligning perfectly with our CPG schema. They provide transactional guarantees, indexing for fast lookups, and, most importantly, powerful graph query languages like Cypher.21 A query to find all functions transitively reachable from a set of entry points, which is the core of dead code analysis, can be expressed concisely and executed efficiently with a Cypher query like
MATCH (entry)-->(live_function).21
The adoption of a persistent graph database fundamentally elevates the system from a simple, one-shot linter into a "living model" of the codebase. This persistent CPG can be incrementally updated as the code evolves, with only the affected subgraphs needing to be re-processed.25 This capability is not just a performance optimization; it unlocks a host of new possibilities. The same CPG built for dead code detection can be queried for other complex analyses without re-parsing the entire codebase. Potential future applications include:
Dependency Visualization: Generating interactive maps of module and component dependencies.
Security Analysis: Performing taint tracking to identify vulnerabilities like SQL injection or cross-site scripting.
Architectural Conformance: Enforcing rules such as "module A is not allowed to depend on module B."
Impact Analysis: Answering developer questions like, "If I change this API, what downstream code will be affected?"
Therefore, the investment in a CPG and a graph database provides a strategic foundation for an entire suite of advanced static analysis tools, making the dead code detector a powerful first step with a significant potential return for the broader Uveddi architecture.

Section 2: Whole-Program Call Graph Construction

The call graph is the central data structure for reachability analysis; it represents all the potential runtime calls between functions and methods in a program.26 An edge from function
F to function G in this graph signifies that F may call G. The accuracy of dead code detection is directly contingent on the precision of this call graph. Constructing a sound (not missing any real calls) and precise (not including many impossible calls) call graph is a classic problem in static analysis, especially when dealing with the diverse features of both statically and dynamically typed languages. This section details the algorithms and strategies for building a high-fidelity, whole-program call graph for Rust, Python, and JavaScript.

2.1. A Comparative Analysis of Call Graph Algorithms

The primary challenge in static call graph construction is the resolution of indirect calls—cases where the target of a call is not fixed at compile time. This includes dynamic dispatch (virtual methods), function pointers, and higher-order functions.26 Various algorithms exist, each offering a different trade-off between analytical precision and computational cost.
Class Hierarchy Analysis (CHA): This is a fast but imprecise algorithm primarily used for object-oriented languages. For a method call like obj.method(), CHA conservatively assumes that the call could resolve to any method named method within the declared class of the variable obj or any of its subclasses in the entire class hierarchy.28 While CHA is guaranteed to be sound (it over-approximates and thus won't miss a potential call), its imprecision leads to many spurious edges in the call graph. These spurious edges can make reachable code appear to be called by other reachable code, resulting in false negatives where dead code is incorrectly classified as live.31
Rapid Type Analysis (RTA): RTA is a practical and efficient improvement over CHA.29 It operates in two stages. First, it performs a quick pass over the entire program to identify the set of all classes that are actually instantiated (e.g., via
new MyClass() or MyClass()). In the second stage, it refines the results of CHA by pruning any potential call targets belonging to classes that were never instantiated.28 This simple heuristic eliminates a significant number of impossible calls, especially in projects that use large libraries but only instantiate a small subset of their classes.
Points-to Analysis: This is a family of more powerful and precise data-flow analysis techniques. Instead of relying on the declared type of a variable, points-to analysis attempts to compute the set of actual objects or memory locations that a variable could "point to" at runtime.27 A call
obj.method() is then resolved by examining the methods of only the specific object types in obj's points-to set, rather than all possible subtypes.
0-CFA (Context-Insensitive): This is a common variant that computes one points-to set for each variable across the entire program. It is more precise than RTA but does not distinguish between different call contexts of a function.29
k-CFA (Context-Sensitive): This is a more precise (and much more expensive) variant that maintains separate points-to sets for a function based on its call stack history (up to a depth of k).
Given the diverse nature of the target languages, a one-size-fits-all approach is suboptimal. The system will therefore implement a hybrid, configurable strategy:
For Rust: The language's strong static type system allows for highly precise analysis without resorting to expensive, general-purpose algorithms. The strategy will focus on resolving Rust's specific dispatch mechanisms directly.
For Python and JavaScript: The dynamic nature of these languages necessitates a more flexible approach. The default analysis will use a combination of RTA for class-based method calls and simple name resolution for standalone functions. This provides a fast baseline. For users requiring higher accuracy, an optional, more expensive points-to analysis (0-CFA style) can be enabled. This allows for a direct trade-off between performance and precision, which can be configured per-project.
Algorithm
Core Principle
Precision
Performance Cost
Suitability for Rust
Suitability for Python/JS
Class Hierarchy Analysis (CHA)
Resolves calls based on the declared type's entire subclass hierarchy.
Low
Very Low
Limited use; Rust's trait system is more relevant.
Baseline for imprecise, fast analysis.
Rapid Type Analysis (RTA)
Like CHA, but restricted to classes that are actually instantiated.
Medium
Low
Useful for resolving dyn Trait calls.
Excellent as a fast, default algorithm.
0-CFA (Points-to Analysis)
Tracks the set of possible runtime objects a variable can point to.
High
Medium-High
Overkill for most static dispatch; useful for function pointers.
The gold standard for achieving high precision.
k-CFA (Points-to Analysis)
Context-sensitive version of 0-CFA; tracks objects per call site.
Very High
Very High
Generally impractical and unnecessary due to Rust's type system.
Impractical for a general-purpose tool due to cost.

Table 2.1: Comparison of Static Call Graph Construction Algorithms
The circular dependency between call graph construction and alias (or points-to) analysis, where each requires the other for full precision, necessitates an iterative approach.26 The analysis cannot be a single pass. The engine must start with an initial, approximate graph (e.g., built with RTA) and then iteratively refine both the call graph and the points-to information. New call edges enable more data to flow between functions, which in turn updates the points-to sets; this new points-to information can then resolve more call sites, adding new edges to the graph. This process continues until a fixed point is reached, where an iteration produces no changes to either the call graph or the points-to sets.27 The core of the analysis engine must therefore be implemented as a worklist-based, fixed-point iteration algorithm, a standard and correct approach for this class of data-flow problem.

2.2. Strategy for Rust: Resolving Static Dispatch, Traits, and Function Pointers

Rust's design emphasizes compile-time safety and performance, which means most function calls are resolved statically. This makes call graph construction significantly more tractable than in dynamic languages.
Static Dispatch: The vast majority of calls in Rust are direct function calls or static method calls. These can be identified and resolved with high precision using Tree-sitter queries to find call expressions and the powerful name resolution capabilities of the Rust compiler's analysis APIs (or a re-implementation of them).
Trait Objects (dyn Trait): Trait objects are Rust's mechanism for dynamic dispatch. A call on a trait object (e.g., my_trait.method()) could invoke any implementation of that method for any type that fulfills the trait contract.37 This is analogous to virtual method calls in other languages. An RTA-style approach is highly effective here: the analysis first identifies all structs and enums that implement the given trait. Then, it identifies all locations where these types are coerced into a
dyn Trait object. The set of potential callees for a method call on the trait object is the intersection of all method implementations for the types that are actually used as that trait object.
Function Pointers (fn()): Resolving calls via function pointers is a classic hard problem in static analysis.37 A naive analysis, knowing only the type
fn(i32) -> i32, would have to assume a call could target any function with a matching signature in the entire program, which is massively imprecise. Rust's strong typing, however, allows for a more refined strategy 37:
Signature-Based Filtering: The first step is to identify the set of all functions in the codebase whose signatures are compatible with the function pointer's type.
Points-to Analysis: A targeted points-to analysis is then used to track the flow of function items. The analysis identifies all functions that are explicitly assigned to or coerced into a variable of the function pointer type. Only these functions are considered potential callees. This drastically prunes the set of possibilities.
Leveraging Compiler Metadata: A forward-looking strategy involves preparing the system to consume enhanced compiler-emitted metadata. An experimental proposal for rustc suggests embedding metadata in the generated LLVM-IR that explicitly links function pointer coercions to their call sites and trait object usage to its implementations.38 If this feature becomes stable, it would allow the analysis to resolve these indirect calls with near-perfect precision by simply reading the metadata, bypassing the need for complex and expensive points-to analysis. The system's architecture should be designed to optionally consume this metadata, providing a pathway to future accuracy improvements.

2.3. Strategy for Python & JavaScript: A Hybrid Approach for Dynamic Dispatch

In dynamically typed languages like Python and JavaScript, the absence of mandatory static type information makes call graph construction inherently more challenging. A method call like obj.method() can only be definitively resolved at runtime, as the type of obj can change, and methods can be dynamically added or replaced (monkey-patching).43 Static analysis must therefore rely on approximations.
The proposed strategy is a multi-tiered approach that balances performance and precision:
Tier 1: Fast, Type-Heuristic Analysis (RTA-based): This will be the default mode, optimized for speed.
The analyzer first scans the entire codebase to identify all class definitions (e.g., class MyClass:...) and all instantiation sites (e.g., x = MyClass()). This builds a set of "live" or instantiated types, as in Rapid Type Analysis.28
For a call site obj.method(), the analyzer performs local data-flow analysis to infer the type of obj. If it can confidently trace obj back to an instantiation of MyClass, it creates a call graph edge to MyClass.method.
If the type of obj cannot be determined locally, the analyzer falls back to a CHA-like over-approximation: it creates call graph edges to every method named method on every known class in the program. While imprecise, this ensures soundness.
Tier 2: Precise, Points-to Analysis (0-CFA based): For users who need higher accuracy and are willing to accept a performance trade-off, a more advanced analysis mode can be enabled. This mode implements a flow-insensitive, context-insensitive (0-CFA) points-to analysis.29 This analysis constructs a global graph of all assignments (
a = b, x = MyClass(), return x) to compute a more accurate set of possible runtime types for each variable. This technique is used by state-of-the-art tools like PyCG for Python and js-callgraph for JavaScript, and it dramatically reduces the number of spurious call edges generated by the more naive heuristics.45
This tiered approach provides essential flexibility. A developer running the analysis interactively in an IDE can use the fast Tier 1 for quick feedback, while a comprehensive CI job can enable the more thorough Tier 2 to find more dead code before a merge.

2.4. Handling Higher-Order Functions and Callbacks

Higher-order functions—functions that take other functions as arguments or return them—are a common pattern in Python and JavaScript, especially in the context of event handling, asynchronous programming, and functional-style code. A call to a function like register_handler(my_callback) creates an indirect call relationship that must be captured.
Resolving these calls is a natural application of the points-to analysis framework described above. The analysis treats functions as first-class objects whose references can be tracked as they flow through the program.
Tracking Function Flow: When my_callback is passed as an argument to register_handler, the points-to analysis records that the corresponding parameter of register_handler now "points to" the my_callback function object.
Resolving Indirect Invocations: Later, inside the body of register_handler (or some other function to which it passes the callback), when the parameter is invoked (e.g., handler()), the analyzer resolves this call site. It queries the points-to set for the handler variable and creates call graph edges to all function objects it might contain, including my_callback.
This mechanism is crucial for accurately modeling modern frameworks. For example, in a React application, an event handler like <button onClick={handleClick}> is effectively registering handleClick as a callback. The analysis must create a call graph edge from the React framework's event dispatch system to the handleClick function. The framework-specific entry point detectors (detailed in Section 3) will be responsible for identifying these callback registration sites and modeling them as indirect calls.
The inherent dynamism of Python and JavaScript means that any purely static analysis will have limitations. A codebase that makes heavy use of features like Python's getattr or JavaScript's reflective property access will force the analyzer to make wide, imprecise assumptions. This reality suggests that the tool should not present its findings as absolute truth. Instead, it should communicate its limitations to the user by flagging "analysis hazards"—regions of code where precision is necessarily low due to dynamic features. This helps manage user expectations and can even guide developers toward writing code that is more amenable to static analysis and, often, more maintainable.

Section 3: Reachability Analysis: Identifying Live and Dead Code

Once a comprehensive, whole-program call graph has been constructed, the final step is to perform a reachability analysis to distinguish live code from dead code. This process starts from a set of known "entry points"—code that is guaranteed to be executed—and traverses the call graph to find all transitively reachable symbols. Any symbol not reached during this traversal is considered dead.47

3.1. The Mark-and-Sweep Approach: Traversing the Call Graph

The most direct and well-established algorithm for this task is analogous to the mark-and-sweep algorithm used in garbage collection.49 The process operates on the Code Property Graph (CPG) and consists of three stages:
Initialization: The analysis begins by identifying all valid program entry points (as detailed in Section 3.2 and 3.3). A worklist is created and populated with the CPG nodes corresponding to these entry points. Each of these nodes is immediately marked as "live" by setting its is_live attribute to true.
Mark Phase: The core of the algorithm is a graph traversal that iteratively discovers all reachable code. The process is as follows:
While the worklist is not empty, a node N is removed from it.
The analyzer examines all outgoing semantic edges from N in the CPG.
For every outgoing CALLS edge from a function N to another function M, if M is not already marked as live, its is_live flag is set to true, and M is added to the worklist.
Similarly, for every USES_VARIABLE, INHERITS_FROM, or IMPLEMENTS_TRAIT edge from a live node N to another symbol S (e.g., a global variable, a base class), if S is not yet live, it is marked as such and added to the worklist (if it can lead to further calls, like a class instantiation).
Sweep Phase: After the worklist is empty, the mark phase is complete. All nodes in the CPG that are transitively reachable from the entry points have been marked as live. The final "sweep" phase simply involves iterating through all symbol-defining nodes (functions, classes, variables, etc.) in the CPG. Any node whose is_live attribute remains false is identified and reported as dead code.
This approach correctly handles transitive reachability. A function h that is only called by another function g will be correctly identified as dead if g itself is not reachable from any primary entry point.50

3.2. Automatic Entry Point Discovery Strategies

The correctness and completeness of the dead code analysis depend entirely on the initial set of entry points. An incomplete set will lead to false positives (live code being reported as dead), while an overly broad set will lead to false negatives. The system must therefore employ a sophisticated, multi-faceted strategy for automatically discovering these entry points, tailored to the conventions of each language and popular frameworks.
The process of entry point detection is not a one-time implementation but rather a continuous effort of pattern matching against an evolving ecosystem of tools and frameworks. A hardcoded detector for today's frameworks will quickly become obsolete. Therefore, the detection system should be architected in a pluggable manner. A core analysis engine will consume a list of entry points, which will be provided by a set of independent "detector plugins." Each plugin will be responsible for recognizing the conventions of a specific language, framework, or tool (e.g., a FlaskDetector, a ReactDetector, a JestDetector). This modular design, inspired by successful tools like knip 51, ensures the system is extensible and can easily adapt to new technologies.
Furthermore, the very concept of an "entry point" is context-dependent. A library, when analyzed in isolation, must treat its entire public API as live. However, when that same library is analyzed as a dependency of a specific application, only the subset of the public API that is actually used by the application is live.50 This necessitates that the tool support different "analysis modes":
Application Mode: Performs a whole-program analysis starting from application-specific entry points (e.g., a main function). This mode provides the most precise results for end-user projects.
Library Mode: Assumes all publicly exported symbols are entry points. This mode is appropriate for library authors who want to find dead code within their internal implementation, without making assumptions about how the library will be consumed.
The following table outlines the specific heuristics that the automatic detector plugins will use.

Language
Project Archetype
Pattern/Heuristic
Example
Rust
Binary Crate
fn main() or function with #[main] attribute.
fn main() {... } 52


Library Crate
All pub items (functions, structs, enums, traits) in the crate's root.
pub fn my_api_function() {... } 50


Test Code
Functions annotated with #[test].
#[test] fn it_works() {... }
Python
Script/Application
Top-level code in the main executed file; entry points defined in pyproject.toml or setup.py.
[project.scripts] my-cli = "my_package.main:run" 55


Flask Application
Functions decorated with @app.route(...) or @blueprint.route(...).
@app.route('/') def index():... 56


Django Project
View functions/classes referenced in urls.py; management/commands; middleware classes.
path('home/', views.home_view, name='home') 58
JavaScript
Generic/Node.js
Top-level code in the main file specified in package.json ("main": "index.js").
-


Webpack Project
The entry point(s) defined in webpack.config.js.
module.exports = { entry: './src/index.js' }; 60


React Application
The root component passed to ReactDOM.render() or createRoot().render().
ReactDOM.render(<App />, document.getElementById('root')); 61


Next.js Application
Files within the pages/ or app/ directories that define page components or route handlers.
export default function HomePage() {... } in pages/index.js


Express.js App
Route handler functions passed to app.get(), app.post(), etc., and middleware passed to app.use().
app.get('/users/:id', (req, res) => {... }); 63


Test Files (Jest/Vitest)
Test suites defined with describe(), and test cases with it() or test().
test('should add two numbers', () => {... });

Table 3.1: Entry Point Detection Heuristics

3.3. A Framework for Configuration-Based Entry Point Definition

Despite sophisticated heuristics, automatic entry point detection can never be infallible. Code may be invoked in ways that are invisible to static analysis, such as through foreign function interfaces (FFI), dynamic plugin loading, or reflection-based framework magic. To handle these cases and provide users with full control, the system must support configuration-based overrides. This is a standard and essential feature for production-grade analysis tools.65
A configuration file, such as .uveddi.yml, will be supported at the root of the project. This file will allow users to supplement or override the automatically discovered entry points. The configuration schema will provide the following key options:
entry_points: An explicit list of fully qualified symbol names (e.g., my_package.my_module.my_function) that should be treated as roots for the reachability analysis. This is the primary mechanism for defining custom entry points.
ignore_patterns: A list of glob patterns (e.g., "tests/**", "**/vendor/*.js") specifying files and directories that should be completely excluded from the analysis. This is useful for ignoring test suites, third-party libraries, or auto-generated code that may contain intentionally "unused" symbols from the perspective of the main application.
keep_symbols: A list of symbol names or regular expression patterns for symbols that should be marked as live, regardless of whether they are found to be reachable. This is the escape hatch for code that is known to be used through mechanisms invisible to the static analyzer, such as being called from native code or being referenced by name in a configuration file.
This combination of automated detection and manual configuration provides a pragmatic and powerful approach, ensuring that the analysis can be tailored to the unique characteristics of any given project, thereby maximizing accuracy and minimizing false positives.

Section 4: Advanced Challenges in Dynamic and Metaprogramming Contexts

The most formidable challenges for any static analysis tool arise from language features that defer structural and behavioral decisions to runtime. For Rust, this primarily involves its powerful macro system. For Python and JavaScript, the challenges are more pervasive, stemming from their dynamic typing, runtime code evaluation, and metaprogramming capabilities. A robust dead code detector cannot simply ignore these features; it must employ specific strategies to handle them, even if it means making principled approximations. This section details the proposed strategies for these advanced cases and introduces a confidence scoring mechanism to communicate the inherent uncertainty of the analysis to the user.

4.1. Taming eval and Dynamic Imports: A Strategy of Over-Approximation and Taint Analysis

Runtime code evaluation and dynamic module loading represent the theoretical limit of static analysis, as their behavior can depend on data that is unknowable before execution.
Runtime Evaluation (eval): Functions like eval() in Python and JavaScript can execute arbitrary code from a string constructed at runtime.44 Statically determining the content of this string is, in the general case, an undecidable problem.
Strategy: The system will adopt a conservative, multi-level approach.
Constant Propagation: In the simple case where the argument to eval is a string literal, the analyzer can parse the literal's content and analyze it as if it were regular code.
Taint Analysis: A more sophisticated (and costly) approach involves performing taint analysis. If the string can be traced back exclusively to a set of known, constant sources, it can be analyzed. However, if the string is "tainted" by runtime inputs (e.g., user input, network responses), it is considered fully dynamic.
Pragmatic Over-Approximation: When the content of the eval string is determined to be dynamic, the analyzer must make a sound over-approximation. The most practical strategy is to assume that the eval call may access or call any symbol visible in its current scope. This means all such symbols are marked as live, preventing the tool from incorrectly reporting them as dead. This call site will be flagged as a low-confidence "analysis hazard."
Dynamic Imports: JavaScript's import(expression) and Python's __import__() or importlib.import_module() present a similar challenge when the module specifier is not a static string literal.66
Strategy:
Static Resolution: If the argument is a string literal (e.g., import('./utils.js')), the dependency can be resolved directly and added to the CPG. The analyzer will also perform simple constant propagation to resolve cases like const moduleName = 'utils'; import(./${moduleName}.js);.
Dynamic Path Handling: If the module path is fully dynamic and cannot be resolved to a finite set of possible files, the analysis must approximate. A highly conservative approach would be to assume it could import any file in the project, marking all exported symbols from all files as live. However, this would render the tool nearly useless. A more pragmatic strategy is to flag the dynamic import as an analysis hazard and not resolve it. This acknowledges the limitation and accepts the risk of potential false positives (i.e., reporting code as dead that is only used via this import). The finding would be accompanied by a message indicating the source of uncertainty.

4.2. Resolving Metaprogramming: Analyzing Python Decorators/Metaclasses and Rust Macros

Metaprogramming allows code to modify or generate other code, creating structures that are not present in the literal source text.
Python Decorators: A decorator is syntactic sugar that wraps a function or class.69 The declaration
@my_decorator def f():... is equivalent to def f():...; f = my_decorator(f).70
Strategy: The analyzer must model this transformation. When a decorator is encountered:
The decorator function (my_decorator) is marked as live and a CALLS edge is created to it.
The original, undecorated function (f) is considered "used" as it is passed as an argument to the decorator.
The analysis must then attempt to determine what the decorator returns. If it returns a new wrapper function that, in its body, calls the original function, a CALLS edge must be created from the wrapper to the original function. Analyzing the body of arbitrary decorators can be complex.
A sound and practical approximation is to treat any decorated function as inherently live. The act of decorating it implies its behavior is being used and modified, even if the original function object is not called directly.
Python Metaclasses: Metaclasses are an even more powerful form of metaprogramming, allowing control over the creation of class objects themselves, often by dynamically injecting methods or modifying attributes via the __new__ method.71
Strategy: Statically analyzing the behavior of a metaclass is exceptionally difficult, as it can involve arbitrary Python code execution during class creation time. Attempting to "interpret" the metaclass logic is beyond the scope of this tool. The only robust and sound strategy is to detect the use of a custom metaclass (any metaclass other than the default type) and conservatively treat the resulting class and all of its methods as opaque and, therefore, live.
Rust Macros: Rust macros are a form of compile-time metaprogramming that operate on token streams to generate new Rust code.75 There are two main types: declarative macros (
macro_rules!) and procedural macros.
Strategy: It is not feasible to analyze the macro definition itself. The analysis must operate on the macro's expansion.
The system will integrate with a tool like cargo-expand or leverage rustc's own expansion capabilities to obtain the post-macro-expansion source code for a given crate.
This expanded code, which is plain Rust code, is then parsed by Tree-sitter, and the resulting symbols and call graph edges are added to the CPG.
This ensures that any functions, variables, or calls generated by a macro are correctly included in the analysis. The macro definition itself is considered live if it is invoked anywhere in the codebase.
Practical Limitations: This approach faces challenges. Procedural macros, in particular, can be complex and may rely on compiler-internal details, making them difficult to expand correctly outside of a full rustc compilation. This is a known source of false positives in existing tools.52 As a fallback, the system may need to conservatively mark code within complex or unexpandable macro invocations as live and flag it as an analysis hazard.

4.3. Confidence Scoring: Quantifying Uncertainty in Dynamic Analysis

Given that the analysis of dynamic languages involves necessary approximations, presenting results as a binary "dead" or "alive" classification would be misleading and erode user trust.78 To address this, the system will associate a
confidence score with each piece of code it identifies as dead. This score quantifies the analyzer's certainty in its finding.
Implementation: Each reported instance of dead code will be assigned a confidence level, for example, from 0.0 (lowest) to 1.0 (highest). The score will be determined by the nature of the evidence:
1.0 (High Confidence): An unused, non-exported symbol in a statically-typed language (Rust) or a private/internal function in Python/JS with no identifiable incoming calls from any live code.
0.8 (Medium-High Confidence): An unused, exported function in an application (not a library) that is not an identified entry point and has no incoming calls. It is probably dead but could theoretically be accessed via reflection.
0.5 (Medium Confidence): An unused symbol in a file that is part of a project using known metaprogramming patterns (e.g., complex decorators) that the analyzer handles with approximation. The report would state, "This function appears unused, but analysis of decorated functions is limited."
0.2 (Low Confidence): A symbol that is only "dead" because the analysis was blocked by a highly dynamic feature. For example: "This module appears unused, but it is in a directory that could be targeted by a dynamic import() whose path could not be resolved."
This scoring system transforms the tool from a blunt instrument into a nuanced assistant. It allows users to filter results based on confidence, focusing first on high-confidence findings and treating low-confidence ones as suggestions for further investigation.
The following table summarizes the proposed strategies for handling these advanced features.
Feature
Language(s)
Recommended Strategy
Impact on Soundness
Confidence Level
eval()
Python, JS
Constant propagation; fall back to marking all symbols in scope as live.
Sound (over-approximates).
Low
Dynamic import()
Python, JS
Constant propagation; fall back to flagging as an analysis hazard without resolving.
Unsound (may miss dependencies).
Low
Python Decorators
Python
Model the f = decorator(f) transformation; conservatively mark decorated functions as live.
Sound (over-approximates).
Medium
Python Metaclasses
Python
Detect custom metaclass usage; mark the entire class and its methods as live.
Sound (over-approximates).
Medium
Rust Macros
Rust
Analyze the expanded source code from a tool like cargo-expand.
Sound if expansion is perfect.
High (if expandable)

Table 4.1: Strategies for Handling Dynamic and Metaprogramming Features
This strategic approach—combining direct analysis where possible, principled approximation where necessary, and transparent communication of uncertainty—is essential for building a tool that is both powerful and trustworthy for developers working with modern, multi-paradigm languages.

Section 5: Performance Engineering for Large-Scale Codebases

For a static analysis tool to be adopted in a professional software development environment, it must be performant. Analyses that take hours to run on large codebases are relegated to infrequent, overnight jobs and fail to integrate into the fast-paced, iterative workflow of modern development. This section details a three-pronged performance strategy—incrementality, caching, and parallelism—designed to ensure the dead code detector is fast and scalable enough for both interactive IDE usage and rapid CI/CD feedback on enterprise-scale monorepos.

5.1. Designing for Incrementality: Reusing Analysis Results on Code Changes

The most significant performance gain comes from avoiding redundant work. A full, from-scratch analysis of a multi-million-line codebase on every minor change is computationally prohibitive. The system must therefore be designed for incremental analysis, reusing the maximum amount of work from previous runs.80
The CPG-based architecture is naturally suited for this. The analysis can be conceptualized as an update to the persistent graph model rather than a one-off computation.
Change Detection and Parsing: The process begins by identifying the set of modified files (e.g., via Git diff). For each changed file, the system leverages Tree-sitter's incremental parsing capabilities. By providing the old syntax tree and the details of the edit, Tree-sitter can produce an updated CST far more quickly than parsing from scratch.3
Subgraph Invalidation and Update: The analyzer identifies the subgraph within the CPG that corresponds to the changed file. This subgraph is invalidated and updated based on the new CST, modifying the relevant nodes (definitions) and local edges (references).
Dependency-Guided Re-analysis: The crucial step is to intelligently limit the scope of the re-analysis. Instead of re-running the entire global analysis, the system identifies the "boundary" of the change. This boundary consists of all nodes in the CPG that had an incoming or outgoing dependency edge to or from the modified subgraph. The expensive semantic analysis phases—such as call graph resolution and reachability traversal—are then re-initiated only from this boundary, propagating the effects of the change outward until a new fixed point is reached.81 This targeted approach, which re-analyzes only the code affected by a change and its dependencies, can reduce analysis time by orders of magnitude compared to a full run.

5.2. Multi-Layered Caching Strategies

Complementary to incremental analysis, a robust caching strategy is essential for eliminating re-computation of unchanged artifacts, both within a single analysis run and across multiple runs (e.g., in a CI environment). The system will implement a multi-layered cache:
Layer 1: Parsed Concrete Syntax Trees (CSTs): The raw CST for each source file, as produced by Tree-sitter, can be cached on disk. The cache key will be a hash of the file's content. If a file has not changed between analysis runs, its CST can be loaded directly from the cache, skipping the parsing step entirely.
Layer 2: File-Level Subgraphs: The result of the initial symbol extraction phase for each file—an unresolved CPG subgraph containing all definitions and local references—can also be cached. The cache key would be the hash of the source file's CST. This avoids re-running the Tree-sitter queries for unchanged files.
Layer 3: Expensive Analysis Results: The results of computationally intensive analyses can be cached at a more granular level. For example, the set of potential call targets computed by a points-to analysis for a specific function can be cached. If that function's body and its direct dependencies have not changed, this result can be reused without re-running the data-flow analysis.
Cache Invalidation is critical for correctness. The cache key for any artifact must be derived from a cryptographic hash of all its inputs. For example, the key for a cached file-level subgraph would depend on the source file's content hash and the version of the analysis tool itself (to invalidate caches when the analysis logic changes). This ensures that any relevant change correctly invalidates the corresponding cache entries, guaranteeing the soundness of the analysis.

5.3. Parallelizing the Analysis Pipeline: A File- and Component-Based Approach

Modern multi-core processors offer a significant opportunity for performance improvement through parallelism. Many stages of the static analysis pipeline are "embarrassingly parallel" and can be executed concurrently to reduce wall-clock time.
Parallel Parsing and Symbol Extraction: The initial phase of the analysis, which involves parsing each source file and extracting its symbols into a file-level subgraph, is highly parallelizable. Since files are syntactically independent, they can be processed concurrently by a pool of worker threads. This has been demonstrated to yield major performance improvements, with one case study showing a reduction from 2.3 seconds to 200-300 milliseconds for a large number of files by using the rayon crate in Rust.83
Parallel Component Analysis: After the initial parsing and linking phase, the global CPG may not be a single, monolithic graph. Large codebases often consist of several loosely-coupled or entirely disconnected components. A preliminary analysis can identify these components. The more intensive analysis phases, such as fixed-point iteration for call graph construction, can then be run in parallel on each of these independent components.
This parallelization strategy introduces a potential tension between performance and the complexity of managing dependencies. The initial parsing phase is simple to parallelize, but the later linking and global analysis phases require careful management of access to the shared CPG to avoid race conditions. A well-structured, multi-phase architecture, similar to a MapReduce model, is an effective way to manage this complexity:
Map Phase (Parallel): Worker threads concurrently parse all source files into independent, in-memory, file-level subgraphs.
Shuffle/Link Phase (Serial or Controlled Concurrency): A central coordinator ingests all the file-level subgraphs and links them together, resolving the cross-file dependencies to build the final, global CPG. This phase requires careful synchronization to safely write to the shared graph structure.
Reduce Phase (Parallel): The final reachability analysis is performed. This can itself be parallelized, for example, by having different threads trace paths from different entry points or explore independent subgraphs of the call graph.
This architecture balances the significant performance gains of parallelism with the need to correctly and safely manage the inter-dependencies inherent in whole-program analysis. These performance engineering strategies are not merely optimizations; they are enabling features that determine the practical utility of the tool. Incremental analysis makes low-latency IDE integration feasible, while caching and parallelism make fast CI/CD feedback a reality.

Section 6: Integration and Recommendations for the Uveddi Architecture

The ultimate success of the dead code detection system depends not only on its analytical capabilities but also on its seamless integration into the existing development workflow and its ability to serve as a foundation for future tools. This section provides a high-level implementation plan, an API design proposal, and a summary of key recommendations for integrating the system into the Uveddi architecture.

6.1. API Design for the Dead Code Detector

To ensure the system is a valuable component within the broader Uveddi ecosystem, it should expose a clean, well-documented, and programmatic API. This API will serve as the primary interface for CI/CD integrations, IDE plugins, and other internal tools.
Core Analysis API: The main functionality will be exposed through a simple, high-level API.
analyze(project_path: Path, config: Config) -> AnalysisResult: This function triggers a full, from-scratch analysis of the specified project. It takes the project's root path and a configuration object (containing user-defined entry points, ignore patterns, etc.) as input.
update(project_path: Path, config: Config, changed_files: List[Path]) -> AnalysisResult: This function triggers an incremental analysis. It takes the list of changed files as an additional argument, allowing the engine to perform a targeted update instead of a full run.
AnalysisResult: This will be a structured data object returned by the analysis functions. It will contain a list of all identified dead symbols, with each entry including the symbol's name, kind, source location, the confidence score of the finding, and a human-readable reason for the finding (e.g., "Unused private function" or "Potentially unused; analysis blocked by dynamic import").
CPG Query API: A key strategic recommendation is to expose the underlying Code Property Graph for more advanced use cases. If a persistent graph database like Neo4j is used, this API could be a thin wrapper that allows other services in the Uveddi architecture to execute queries directly against the CPG using a standard query language like Cypher. This would unlock the ability to build other powerful tools—such as for security taint analysis or dependency visualization—on top of the same underlying code model, maximizing the return on investment for this project.

6.2. Phased Implementation Roadmap

A project of this complexity should be implemented in phases to manage risk, deliver value incrementally, and allow for iterative refinement of the core architecture.
Phase 1: Foundational Architecture and Rust Analysis
Objective: Build and validate the core pipeline in the most predictable environment.
Tasks:
Implement the core Tree-sitter parsing pipeline.
Set up the graph database and define the final CPG schema.
Implement the graph construction engine for Rust, focusing on direct calls, pub API and main function entry points, and basic resolution for trait objects. Rust's static nature makes it the ideal candidate for validating the end-to-end flow from parsing to reachability analysis.
Develop the initial mark-and-sweep reachability algorithm.
Outcome: A functional dead code detector for Rust binary and library crates.
Phase 2: Python and JavaScript Core Analysis
Objective: Extend support to the dynamic languages, focusing on the most common use cases.
Tasks:
Integrate the Tree-sitter parsers for Python and JavaScript.
Extend the graph construction engine with language-specific name and module resolution rules for Python and JS.
Implement the fast, RTA-based call graph algorithm for dynamic method dispatch.
Develop the pluggable entry point detectors for major frameworks, starting with React, Express.js, Flask, and Django.
Outcome: A functional dead code detector for common Python and JavaScript applications, with known limitations in highly dynamic code.
Phase 3: Advanced Analysis, Performance, and Precision
Objective: Address the more difficult analysis challenges and optimize the system for scale.
Tasks:
Implement the optional, higher-precision points-to analysis (0-CFA style) for Python and JavaScript.
Develop and integrate the strategies for handling advanced features: Rust macros (via expansion), Python decorators, and flagging of eval and dynamic imports.
Implement the confidence scoring system to communicate analysis uncertainty.
Build the full incremental analysis and multi-layered caching systems to ensure performance on large codebases.
Implement the parallel processing pipeline.
Outcome: A production-ready, performant, and precise dead code detector for all three languages, capable of integrating into demanding CI/CD and IDE environments.

6.3. Summary of Recommendations and Future Work

This report proposes a comprehensive and robust strategy for developing a multi-language dead code detection system. The core recommendations are as follows:
Adopt a Code Property Graph (CPG) as the Central Architecture: This unified model, stored in a persistent graph database, is the key to handling multi-language complexity and provides a foundation for future analysis tools.
Build a Unified, Language-Agnostic Engine: Leverage Tree-sitter's consistent API to create a single analysis engine driven by language-specific grammars and queries. This maximizes code reuse and extensibility.
Implement a Hybrid, Configurable Call Graph Strategy: Use high-precision techniques tailored to Rust's static guarantees while offering a flexible, tiered approach (RTA and optional Points-to Analysis) for the dynamic nature of Python and JavaScript.
Design a Pluggable Entry Point Detection System: Acknowledge that framework conventions evolve by creating a modular system of detectors that can be easily updated and extended.
Embrace Uncertainty with Confidence Scoring: For dynamic languages where perfect static analysis is impossible, use a confidence scoring system to provide nuanced, trustworthy results to developers.
Looking beyond the immediate objective, the architecture laid out in this report opens the door to several powerful future enhancements:
Hybrid Static/Dynamic Analysis: The system could be extended to ingest runtime data, such as code coverage reports from a test suite. By correlating static analysis findings with dynamic execution data, the tool could eliminate false positives and confirm with near-certainty that the remaining reported code is truly dead.
Automated Refactoring: With a high-confidence finding and a lossless CST from Tree-sitter, the system could offer automated "quick fixes" to delete the identified dead code, streamlining the code maintenance process.
An Expanded Suite of Analysis Tools: The CPG is a valuable asset. The same graph that powers dead code detection can be leveraged to build a wide array of other static analysis tools, transforming this initial project into a central code intelligence platform for the entire Uveddi architecture.
Works cited
Tree-sitter: Introduction, accessed July 6, 2025, https://tree-sitter.github.io/
Unraveling Tree-Sitter Queries: Your Guide to Code Analysis Magic ..., accessed July 6, 2025, https://dev.to/shrsv/unraveling-tree-sitter-queries-your-guide-to-code-analysis-magic-41il
Incremental Parsing Using Tree-sitter - Strumenta - Federico Tomassetti, accessed July 6, 2025, https://tomassetti.me/incremental-parsing-using-tree-sitter/
Tree-sitter (parser generator) - Wikipedia, accessed July 6, 2025, https://en.wikipedia.org/wiki/Tree-sitter_(parser_generator)
Using Parsers - Tree-sitter, accessed July 6, 2025, https://tree-sitter.github.io/tree-sitter/using-parsers/
Nvim Treesitter configurations and abstraction layer - GitHub, accessed July 6, 2025, https://github.com/nvim-treesitter/nvim-treesitter
tree-sitter-language-pack 0.3.0: A Comprehensive Collection of Pre-built Tree-sitter Languages : r/Python - Reddit, accessed July 6, 2025, https://www.reddit.com/r/Python/comments/1iivt9i/treesitterlanguagepack_030_a_comprehensive/
tree-sitter-languages - PyPI, accessed July 6, 2025, https://pypi.org/project/tree-sitter-languages/
Hack grammar for tree-sitter - GitHub, accessed July 6, 2025, https://github.com/slackhq/tree-sitter-hack
Lightweight linting with tree-sitter - DeepSource, accessed July 6, 2025, https://deepsource.com/blog/lightweight-linting
Diving into Tree-Sitter: Parsing Code with Python Like a Pro - DEV ..., accessed July 6, 2025, https://dev.to/shrsv/diving-into-tree-sitter-parsing-code-with-python-like-a-pro-17h8
tree-sitter explained - YouTube, accessed July 6, 2025, https://www.youtube.com/watch?v=09-9LltqWLY
Symbol Table Management - Compiler Design - Meegle, accessed July 6, 2025, https://www.meegle.com/en_us/topics/compiler-design/symbol-table-management
Symbol Table in Compiler - GeeksforGeeks, accessed July 6, 2025, https://www.geeksforgeeks.org/compiler-design/symbol-table-compiler/
Mastering Symbol Tables in Programming - Number Analytics, accessed July 6, 2025, https://www.numberanalytics.com/blog/ultimate-guide-symbol-table-programming-languages
Symbol table - Wikipedia, accessed July 6, 2025, https://en.wikipedia.org/wiki/Symbol_table
Compiler Design - Symbol Table - Tutorialspoint, accessed July 6, 2025, https://www.tutorialspoint.com/compiler_design/compiler_design_symbol_table.htm
Symbol Tables and Static Checks - cs.wisc.edu, accessed July 6, 2025, https://pages.cs.wisc.edu/~fischer/cs536.s08/course.hold/html/NOTES/6.SYMBOL-TABLES.html
Use of graph databases for static code analysis, accessed July 6, 2025, https://richardg.users.greyc.fr/publis/Dauprat-All_2022.pdf
Enhancing Code Analysis With Code Graphs - DZone, accessed July 6, 2025, https://dzone.com/articles/enhancing-code-analysis-with-code-graphs
Why Your Code Is A Graph. Graph structures and how they are used… - ShiftLeft Blog, accessed July 6, 2025, https://blog.shiftleft.io/why-your-code-is-a-graph-f7b980eab740
Knee Deep in tree-sitter Queries - Hackerman's Hacking Tutorials, accessed July 6, 2025, https://parsiya.net/blog/knee-deep-tree-sitter-queries/
Application of Graph Databases for Static Code ... - CEUR-WS, accessed July 6, 2025, https://ceur-ws.org/Vol-2590/short30.pdf
Code Graph: From Visualization to Integration - FalkorDB, accessed July 6, 2025, https://www.falkordb.com/blog/code-graph/
Graph-Based Source Code Analysis of JavaScript Repositories - Critical Systems Research Group, accessed July 6, 2025, https://ftsrg.mit.bme.hu/thesis-works/pdfs/stein-daniel-msc.pdf
Call graph - Wikipedia, accessed July 6, 2025, https://en.wikipedia.org/wiki/Call_graph
Application-only Call Graph Construction - Gordon V. Cormack - University of Waterloo, accessed July 6, 2025, https://cormack.uwaterloo.ca/~olhotak/pubs/ecoop12.pdf
Call Graph Construction, accessed July 6, 2025, https://soot-oss.github.io/SootUp/v2_0_0/callgraphs/
Lecture Notes: Object-Oriented Call Graph Construction, accessed July 6, 2025, https://www.cs.cmu.edu/~aldrich/courses/17-355-17sp/notes/notes09-callgraph.pdf
Class Analyses Reference Analysis - People, accessed July 6, 2025, https://people.cs.vt.edu/ryder/516/sp03/lectures/ClassAnal-4-304.pdf
Program Analysis Call Graphs - Software Lab, accessed July 6, 2025, https://software-lab.org/teaching/winter2021/pa/lecture_call_graphs.pdf
Lecture 4 – Class Hierarchy Analysis - People, accessed July 6, 2025, https://people.cs.vt.edu/~ryder/6304/lectures/ClassHierarchyAnalysis-week3.pdf
Program Analysis Call Graphs (Part 2) - Software Lab, accessed July 6, 2025, https://software-lab.org/teaching/winter2020/pa/slides_call_graph_analysis_cha_rta.pdf
rta package - golang.org/x/tools/go/callgraph/rta - Go Packages, accessed July 6, 2025, https://pkg.go.dev/golang.org/x/tools/go/callgraph/rta
scispace.com, accessed July 6, 2025, https://scispace.com/pdf/efficient-points-to-analysis-for-partial-call-graph-15xnbrfecr.pdf
Points-to analysis for partial call graph construction - Academax, accessed July 6, 2025, https://academax.com/doi/10.3785/j.issn.1008-973X.2015.06.005
japaric/cargo-call-stack: Whole program static stack analysis - GitHub, accessed July 6, 2025, https://github.com/japaric/cargo-call-stack
[eRFC] Include call graph information in LLVM IR · Issue #59412 ..., accessed July 6, 2025, https://github.com/rust-lang/rust/issues/59412
C dead code detection while using ARM compiler - Stack Overflow, accessed July 6, 2025, https://stackoverflow.com/questions/17335756/c-dead-code-detection-while-using-arm-compiler
Resolving function pointers with static analysis - Embedded Computing Design, accessed July 6, 2025, https://embeddedcomputing.com/technology/software-and-os/os-filesystems-libraries/resolving-function-pointers-with-static-analysis
Function pointer types - The Rust Reference, accessed July 6, 2025, https://doc.rust-lang.org/reference/types/function-pointer.html
cargo-call-stack, part 2: getting call graph information from rustc | Embedded in Rust, accessed July 6, 2025, https://blog.japaric.io/stack-analysis-2/
Call Graph Construction Algorithms Explained - Ben Holland, accessed July 6, 2025, https://ben-holland.com/call-graph-construction-algorithms-explained/
Build a Call graph in python including modules and functions? - Stack Overflow, accessed July 6, 2025, https://stackoverflow.com/questions/13963321/build-a-call-graph-in-python-including-modules-and-functions
Scalable and Precise Application-Centered Call Graph Construction for Python - arXiv, accessed July 6, 2025, https://arxiv.org/html/2305.05949v3
Persper/js-callgraph: Construct approximate static call graph for JavaScript & Typescript, accessed July 6, 2025, https://github.com/Persper/js-callgraph
List of tools for static code analysis - Wikipedia, accessed July 6, 2025, https://en.wikipedia.org/wiki/List_of_tools_for_static_code_analysis
CWE-561: Dead Code (4.17) - Mitre, accessed July 6, 2025, https://cwe.mitre.org/data/definitions/561.html
How can I know which parts in the code are never used? - Stack Overflow, accessed July 6, 2025, https://stackoverflow.com/questions/4813947/how-can-i-know-which-parts-in-the-code-are-never-used
On dead_code warnings - The Rust Programming Language Forum, accessed July 6, 2025, https://users.rust-lang.org/t/on-dead-code-warnings/31279
Knip: Declutter your JavaScript & TypeScript projects, accessed July 6, 2025, https://knip.dev/
est31/warnalyzer: Show unused code from multi-crate Rust projects - GitHub, accessed July 6, 2025, https://github.com/est31/warnalyzer
Overview of the compiler - Rust Compiler Development Guide - Rustc Dev Guide, accessed July 6, 2025, https://rustc-dev-guide.rust-lang.org/overview.html
Visibility and privacy - The Rust Reference, accessed July 6, 2025, https://doc.rust-lang.org/reference/visibility-and-privacy.html
Entry Points - setuptools 80.9.0 documentation, accessed July 6, 2025, https://setuptools.pypa.io/en/latest/userguide/entry_point.html
Flask Tutorial - GeeksforGeeks, accessed July 6, 2025, https://www.geeksforgeeks.org/python/flask-tutorial/
Flask Application Structure - Meritshot, accessed July 6, 2025, https://www.meritshot.com/flask-application-structure/
The staticfiles app | Django documentation, accessed July 6, 2025, https://docs.djangoproject.com/en/5.2/ref/contrib/staticfiles/
Entry point hook for Django projects - Eldarion, accessed July 6, 2025, https://eldarion.com/blog/2013/02/14/entry-point-hook-django-projects/
Modern React Tools for Building Production-Ready React.js Applications - DhiWise, accessed July 6, 2025, https://www.dhiwise.com/post/react-tools-for-building-production-ready-app
3. Understanding Entry Point and First Component in React: Beginner's Gu... - Reddit, accessed July 6, 2025, https://www.reddit.com/r/reactjs/comments/14xdzee/3_understanding_entry_point_and_first_component/
React - Best practice for application with different entry points/sections - Stack Overflow, accessed July 6, 2025, https://stackoverflow.com/questions/58670010/react-best-practice-for-application-with-different-entry-points-sections
Express/Node introduction - Learn web development | MDN, accessed July 6, 2025, https://developer.mozilla.org/en-US/docs/Learn_web_development/Extensions/Server-side/Express_Nodejs/Introduction
Reading Code - Express | Alex Kondov - Software Engineer, accessed July 6, 2025, https://alexkondov.com/express-architecture-review/
peripheryapp/periphery: A tool to identify unused code in Swift projects. - GitHub, accessed July 6, 2025, https://github.com/peripheryapp/periphery
import() - JavaScript - MDN Web Docs - Mozilla, accessed July 6, 2025, https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/import
How to use the __import__ function to import a name from a submodule? - Stack Overflow, accessed July 6, 2025, https://stackoverflow.com/questions/9806963/how-to-use-the-import-function-to-import-a-name-from-a-submodule
Dynamic Imports | JavaScript Frontend Phone Interview Answers - HelloJavaScript.info, accessed July 6, 2025, https://www.hellojavascript.info/docs/general-javascript-questions/javascript-modules/dynamic-imports
Primer on Python Decorators, accessed July 6, 2025, https://realpython.com/primer-on-python-decorators/
Decorators in Python - GeeksforGeeks, accessed July 6, 2025, https://www.geeksforgeeks.org/python/decorators-in-python/
Python Meta Class Tutorial with Examples - DataCamp, accessed July 6, 2025, https://www.datacamp.com/tutorial/python-metaclasses
Mastering Python Metaclasses for Effective Object-Oriented Design - Netguru, accessed July 6, 2025, https://www.netguru.com/blog/python-metaclasses
Metaprogramming with Metaclasses in Python - GeeksforGeeks, accessed July 6, 2025, https://www.geeksforgeeks.org/metaprogramming-metaclasses-python/
Static Checking via Metaclasses | by Andrei Lapets | Python Supply - Medium, accessed July 6, 2025, https://medium.com/python-supply/static-checking-via-metaclasses-cec368765b58
Procedural Macros - The Rust Reference, accessed July 6, 2025, https://doc.rust-lang.org/reference/procedural-macros.html
Macros - The Rust Programming Language, accessed July 6, 2025, https://doc.rust-lang.org/book/ch19-06-macros.html
Item 28: Use macros judiciously - Effective Rust, accessed July 6, 2025, https://effective-rust.com/macros.html
use vulture to automatically remove dead code? · Issue #25 - GitHub, accessed July 6, 2025, https://github.com/jendrikseipp/vulture/issues/25
How can you find unused functions in Python code? - Stack Overflow, accessed July 6, 2025, https://stackoverflow.com/questions/693070/how-can-you-find-unused-functions-in-python-code
Reusing Caches and Invariants for Efficient and Sound Incremental Static Analysis - DROPS, accessed July 6, 2025, https://drops.dagstuhl.de/storage/00lipics/lipics-vol333-ecoop2025/LIPIcs.ECOOP.2025.28/LIPIcs.ECOOP.2025.28.pdf
Reviser – Incremental Analysis | Secure Software Engineering - Blogs – Uni Paderborn, accessed July 6, 2025, https://blogs.uni-paderborn.de/sse/tools/reviser/
A static analysis method of incremental codes using value dependency graph, accessed July 6, 2025, https://www.spiedigitallibrary.org/conference-proceedings-of-spie/12290/122900M/A-static-analysis-method-of-incremental-codes-using-value-dependency/10.1117/12.2640792.full
General Recommendations: Should I Use Tree-sitter as the AST for the LSP I am developing? : r/neovim - Reddit, accessed July 6, 2025, https://www.reddit.com/r/neovim/comments/1306suu/general_recommendations_should_i_use_treesitter/
