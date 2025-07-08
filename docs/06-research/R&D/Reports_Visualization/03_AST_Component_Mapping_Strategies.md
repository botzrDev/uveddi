
A Multi-Language Framework for Statically-Derived Architectural Component Modeling


1. Principles of Automated Architecture Recovery


1.1. From Source Text to Architectural Insight: A Survey of Static Analysis Techniques

The fundamental objective of this framework is to transform raw source code into a structured, queryable model of its architecture. This process is a specialized application of static program analysis, a discipline dedicated to reasoning about the behavior of computer programs without executing them.1 Static analysis is the foundational technology behind a wide array of developer tools, from simple style linters to sophisticated security scanners that detect complex vulnerabilities.3 The general pipeline for these tools involves parsing a codebase into an intermediate representation, most commonly an Abstract Syntax Tree (AST), which is then subjected to various analytical phases to extract meaningful properties.5
Within the broad field of static analysis, our work aligns specifically with the sub-discipline of Software Architecture Recovery (SAR). SAR refers to the process of reassembling a software system's architecture from its implementation-level artifacts, such as source code.7 The explicit goal is to reverse-engineer the high-level design and component interactions that may be undocumented or have diverged from the original design over time.8 Numerous automated and semi-automated techniques have been proposed to tackle this challenge.8
Adopting the formalisms of SAR provides a rigorous foundation for this project. It allows for the definition of success against a "ground-truth architecture"—the conceptual model of the system as understood by its developers.8 Furthermore, it provides established metrics, such as MoJoSim, which measures the similarity between a recovered architecture and a ground-truth version by calculating the minimum number of "Move" and "Join" operations required to transform one into the other.10 This academic context reframes the task from simple code parsing to a research-informed engineering challenge, acknowledging known difficulties such as the generally low accuracy of purely automated recovery techniques and the critical need for high-fidelity data inputs.8

1.2. The Fidelity-Performance Trade-off in Dependency Extraction

A central challenge in all forms of static analysis is the inherent trade-off between the depth and precision of the analysis—its fidelity—and its computational cost in terms of time and memory.11 Deeper analytical techniques, such as inter-procedural, cross-file, and path-sensitive analysis, yield significantly more accurate results but are computationally intensive.12 For a tool intended to provide rapid feedback to developers, for instance by generating diagrams on demand, managing this trade-off is paramount.
A consistent finding in SAR research is that the accuracy of the input dependencies is a dominant factor in the quality of the recovered architecture.8 Studies comparing different recovery techniques have shown that the type of dependency information used as input often has a greater impact on the final accuracy than the choice of clustering or recovery algorithm itself.9 Specifically, analyses based on fine-grained "symbol dependencies"—which capture concrete relationships like function calls, variable accesses, and type instantiations—produce architectures of substantially higher quality than those based on coarse-grained "include dependencies" (e.g., C/C++
#include directives or Java import statements), which can both over-approximate and miss crucial relationships.9
Therefore, the design of this framework will explicitly prioritize the extraction of high-fidelity, symbol-level dependencies. The potential performance implications of this choice will not be ignored but will be systematically addressed through a suite of optimization strategies, including incremental analysis, intelligent caching, and memory-efficient data structures, as detailed in Section 6. This approach positions the system to achieve the highest possible accuracy, which is a primary success criterion for generating useful architectural diagrams.

1.3. Foundational Approaches: Acknowledging the Landscape

The field of SAR has explored various methodologies for identifying architectural components. Many established techniques are based on clustering algorithms, which group related source code elements (e.g., files, classes) into cohesive components. These algorithms typically rely on similarity metrics derived from dependency intensity, structural coupling, and textual analysis.7
More recent, state-of-the-art approaches have demonstrated that fusing information from multiple sources leads to more accurate and robust architecture recovery.14 The SARIF technique, for example, achieves superior results by combining dependency information, textual content from source code, and the project's directory structure.14 The directory structure, in particular, is a valuable heuristic, as it often reflects the developer's mental model of the system's organization.10
This framework will employ a sophisticated hybrid approach that builds upon these principles. Instead of relying on probabilistic clustering algorithms to discover components, the primary architectural structure will be derived deterministically from the explicit, syntactical constructs of the programming language itself—modules, classes, functions, traits, and so on. This provides a stable, predictable, and more easily understandable baseline architecture. This syntactically-derived structure will then be enriched with a detailed graph of inter-component dependencies. This methodology effectively fuses two powerful sources of information: the explicit structure defined by the language's syntax and the implicit structure revealed by dependency analysis, aligning with the core principle of information fusion for improved accuracy. The directory structure will be used as a supplementary heuristic, primarily for component naming and for resolving ambiguities in module organization.10

2. A Unified Data Model for Architectural Components

The cornerstone of this framework is a rich, canonical data model capable of representing architectural elements from disparate programming languages in a unified manner. This model is designed to be comprehensive, moving far beyond a simplistic enumeration of node types to capture the semantic nuances and idiomatic structures of each supported language.

2.1. The ArchitecturalComponent Canonical Model: Definition and Rationale

The central entity in our model is the ArchitecturalComponent. This structure is designed to be the atomic unit of architectural representation, containing all necessary information for analysis, visualization, and metrication. It replaces the limited ComponentNode enum with a flexible and extensible structure.
The core fields of the ArchitecturalComponent are defined as follows:
component_id: Uuid: A universally unique identifier assigned to each component upon its discovery. This ID is critical for its role as a stable key in hash maps, a unique node identifier in graph structures, and a handle for caching mechanisms.
name: String: The primary, human-readable name of the component, such as the name of a function, class, or module. This is extracted directly from the source code.
file_path: PathBuf: The absolute path to the source file where the component is defined. This provides essential context and is a key piece of information for cross-file analysis.
component_type: ComponentType: A detailed, language-aware enumeration that specifies the precise nature of the component. This is the most critical field for capturing semantic richness and is detailed further in Section 2.2.
dependencies: Vec<Dependency>: A vector that will be populated with all resolved outgoing dependencies from this component to others. The process for identifying and resolving these dependencies is the subject of Section 4.
metrics: ComponentMetrics: A nested structure containing quantitative metrics about the component, such as size and complexity. This enables advanced analysis and richer visualizations. This is detailed in Section 2.3.
source_location: SourceLocation: A struct containing the precise start and end points (line and column numbers) of the component's definition in its source file. This allows tools to link directly from a diagram back to the code.
visibility: Visibility: An enum representing the visibility of the component (e.g., Public, Private, Crate-level). This is essential for understanding a component's public API versus its internal implementation details.

2.2. Language-Specific Extensions: Capturing the Idioms of Rust, Python, and JavaScript

A generic, one-size-fits-all component model is inadequate for capturing the distinct architectural paradigms of modern programming languages. The power of the ArchitecturalComponent model lies in its ComponentType enum, which is a tagged union with variants tailored to the unique constructs of Rust, Python, and JavaScript. This design ensures that the resulting architectural model is not a lossy abstraction but a faithful representation of the source.
Rust-Specific Types: The model is designed to capture the features that define Rust's architecture, particularly its emphasis on safety and composition through the trait system.
Module: Represents a Rust module (mod), capturing its public API status (is_public).
Struct: Represents a struct definition, including a vector of its FieldInfo.
Enum: Represents an enum definition, including a vector of its VariantInfo.
Trait: Represents a trait definition, capturing its public interface through a list of MethodSignatures.
Impl: Represents an impl block, critically capturing both the target type being implemented and the optional trait_impl that is being satisfied. This distinction between a trait definition and its implementation is fundamental to Rust.
Function: Represents a free-standing function with its full FunctionSignature.
Python-Specific Types: The model focuses on Python's object-oriented and dynamic nature.
Class: Represents a class definition, capturing its list of bases (parent classes) to model inheritance. It also includes a list of its MethodInfo.
Method: A specialized function type for methods within a class, capturing boolean flags for is_static and is_class_method, which are essential for understanding how the method is bound and called.
JavaScript-Specific Types: The model addresses the features of modern ECMAScript.
EsModule: Represents an ES module, capturing its list of ExportInfo to define its public interface. This is the primary mechanism for static analysis of module boundaries in modern JS.15
Class: Represents a JS class, capturing the optional extends clause for inheritance.
Function: Represents a function, capturing boolean flags for is_async and is_generator, which have significant implications for the program's control flow and execution model.
The design of the ComponentType enum directly dictates the granularity and semantic richness of the final architectural representation. The proposed structure is a robust foundation because its variants map directly to the high-level node types defined in the respective tree-sitter grammars, ensuring that the extraction process detailed in Section 3 is both feasible and direct.16
Table 1 provides a comparative overview of the architectural concepts and their corresponding representations in the ComponentType model, justifying the chosen level of granularity.
Table 1: Comparison of Architectural Component Granularity
Architectural Concept
Rust ComponentType
Python ComponentType
JavaScript ComponentType
Rationale / Key Details Captured
Code Organization Unit
Module
(Implicit via file/package)
EsModule
Encapsulation, visibility, and namespacing boundaries. Captures the primary mechanism for code organization and reuse in each language's ecosystem.
Data Structure
Struct, Enum
Class
Class
State aggregation and the definition of user-defined types. The model captures the core building blocks for representing data.
Behavior Contract
Trait
(Implicit via Duck Typing)
(Informal via interfaces/prototypes)
Defines a set of behaviors or a shared API. Rust's explicit Trait system is a first-class architectural construct.
Contract Implementation
Impl
Class (method definitions)
Class (method definitions)
Provides the concrete logic for a behavior contract, linking it to a specific data structure. The separation of Trait and Impl is unique to Rust.
Subroutine
Function
Function, Method
Function
The fundamental unit of execution. The model differentiates between standalone functions and class-bound methods where applicable.
Inheritance
(Trait-based composition)
Class { bases: Vec<String> }
Class { extends: Option<String> }
Represents "is-a" relationships, a cornerstone of object-oriented design. The model captures multiple inheritance in Python and single inheritance in JS.


2.3. Quantifying Complexity: The ComponentMetrics Sub-Model

To enable more sophisticated analysis and visualization, each ArchitecturalComponent will be annotated with a ComponentMetrics structure. These objective, quantitative measures allow tools to, for example, size nodes in a diagram according to their complexity or identify potential maintenance hotspots.
The proposed metrics include:
lines_of_code: usize: A straightforward measure of the component's physical size, calculated from its source_location.
cyclomatic_complexity: u32: A well-established measure of a program's logical complexity, calculated by counting the number of linearly independent paths through its source code. For functions and methods, this can be computed by analyzing control flow statements (e.g., if, while, for, match) within the component's AST.
dependency_count: usize: A simple count of the number of outgoing dependencies, equivalent to dependencies.len(). This is a direct measure of a component's reliance on other parts of the system.
afferent_coupling (Ca): u32: A measure of incoming dependencies, representing the number of other components that depend on this component. A high afferent coupling indicates that the component is a core, highly-relied-upon part of the system. This metric will be calculated globally once the full dependency graph is constructed.
efferent_coupling (Ce): u32: A measure of outgoing dependencies, equivalent to dependency_count. A high efferent coupling indicates that a component has many responsibilities or is highly coupled to external components.
These metrics will be calculated during and after the component extraction and dependency resolution phases, providing a rich dataset for subsequent architectural analysis.

3. High-Fidelity Component Extraction Using Tree-sitter

This section provides the implementation blueprint for the core extraction engine. It details the algorithms and patterns required to transform source code into the ArchitecturalComponent data model defined in Section 2. The strategy hinges on the tree-sitter parsing engine and its powerful query language.

3.1. The Tree-sitter Engine: Incremental Parsing and Concrete Syntax Trees (CSTs)

The choice of tree-sitter as the foundational parsing technology is deliberate and based on its unique combination of features, which are ideally suited for this application. tree-sitter is an incremental parsing library designed for high performance and robustness, making it suitable for real-time use cases like code editors and continuous analysis tools.19
A key advantage of tree-sitter is its generation of a Concrete Syntax Tree (CST). Unlike a traditional Abstract Syntax Tree (AST), a CST is a lossless representation of the source code, preserving all syntactic elements, including comments, whitespace, and parentheses.21 This high-fidelity representation is invaluable for tools that require precise source location mapping and detailed structural analysis. Furthermore,
tree-sitter is designed to be resilient to syntax errors, producing a partial CST even for incomplete or incorrect code, which is essential for analyzing code as it is being written.20
The primary mechanism for interacting with the CST is tree-sitter's query language. This is a declarative, S-expression-based language for matching patterns within the syntax tree.23 By writing queries, we can extract specific nodes—such as function declarations or class definitions—without implementing complex, imperative tree-traversal logic for each language.25 This approach is both more maintainable and more powerful.
The overall extraction process will be as follows: for each supported language, a dedicated set of tree-sitter queries will be developed. The AstParser will invoke the appropriate tree-sitter language grammar, parse a source file into a CST, execute the set of queries against that CST, and iterate through the resulting matches. For each match, it will extract the captured nodes and use their content to instantiate and populate an ArchitecturalComponent struct.

3.2. Extraction Patterns for Rust (tree-sitter-rust)

The extraction logic for Rust will leverage the official tree-sitter-rust grammar, which provides a detailed and accurate representation of the Rust language.17 The
queries/ directory within this grammar's repository, particularly the highlights.scm and tags.scm files, serves as an excellent reference for constructing effective queries.17
Structs and Enums: The provided RUST_STRUCT_QUERY is a strong starting point. It correctly identifies a struct_item and captures its name via (type_identifier) @struct_name. To fully populate the ComponentType::Struct variant, the query must iterate over all field_declaration nodes within the field_declaration_list, capturing the field_identifier (@field_name) and the field's type (@field_type). A parallel query will be designed for enum_item nodes, iterating over enum_variant children to populate the variants vector.
Traits: A query targeting trait_item nodes will be created. It will capture the trait's name (type_identifier) and then traverse the declaration_list within its body to extract the signatures of its associated functions and types. This will populate the methods field of the ComponentType::Trait variant.
Impl Blocks: The RUST_IMPL_QUERY successfully captures the target type of the implementation (@impl_target). This query must be extended to also capture the optional trait being implemented. The impl_item node in the grammar has an optional trait field, which contains a type_identifier. The query will be enhanced to capture this node as @trait_name. This allows for the complete population of the ComponentType::Impl { target: String, trait_impl: Option<String> } variant, which is critical for linking concrete implementations to their abstract interfaces.
Functions: A query targeting function_item nodes will extract the function's name (identifier), its parameters, return_type, and any generic parameters defined in type_parameters.
Visibility: A crucial aspect of Rust's architecture is its visibility and module system. Nearly all item-level nodes in the tree-sitter-rust grammar (e.g., struct_item, function_item, trait_item) contain an optional visibility_modifier child. A recurring pattern in all our queries will be to capture this node ((visibility_modifier)? @visibility). The presence and content of this capture (e.g., pub, pub(crate)) will be used to populate the visibility field of the ArchitecturalComponent, enabling the analysis of public APIs versus internal implementation details.

3.3. Extraction Patterns for Python (tree-sitter-python)

Extraction for Python will be based on the tree-sitter-python grammar, which defines the AST nodes for Python code.16
Classes: The provided PYTHON_CLASS_QUERY serves as an effective base. It correctly matches a class_definition node, captures its name (@class_name), and iterates through the argument_list in the superclasses field to capture each @base_class. This directly maps to the ComponentType::Class { bases:..., methods:... } model.
Methods and Decorators: A key feature of Python is its use of decorators to modify function and method behavior. A simple query for a function_definition is insufficient. The tree-sitter-python grammar defines a decorated_definition node which wraps one or more decorator nodes and a function_definition or class_definition. The query must target decorated_definition, capture the identifiers within each decorator (e.g., @staticmethod, @classmethod), and then analyze the nested function_definition. The presence of specific decorator names will be used to set the is_static and is_class_method flags in the ComponentType::Method variant. The highlights.scm file in the grammar repository provides a reference for this pattern.29
Imports: To build the dependency graph later, it is essential to capture all module-level dependencies. Separate queries will be designed to match import_statement (e.g., import os, sys) and import_from_statement (e.g., from my_module import MyClass). These queries will extract the module names and the specific symbols being imported, which will be used to generate DependencyRelationship::Import edges in Section 4.

3.4. Extraction Patterns for JavaScript (tree-sitter-javascript)

JavaScript analysis will use the tree-sitter-javascript grammar, which supports modern ECMAScript features.18
Classes: The JS_CLASS_QUERY correctly identifies a class_declaration, its name, and its optional superclass. This provides the necessary information for the ComponentType::Class { extends:... } variant.
Functions: A comprehensive query will be needed to capture all forms of function definitions, including function_declaration, arrow_function, and method_definition. For each of these, the query must check for the presence of an async keyword and, for generator functions, the * token. The grammar defines these as part of the function node itself, making them straightforward to query. The results will populate the is_async and is_generator booleans in the ComponentType::Function model.
ES Modules: The static analysis of modern JavaScript projects relies heavily on the ES module system (import/export).15 This is the primary mechanism for defining dependencies between files.
Exports: A query will target export_statement nodes. It must be ableto handle various export forms, such as named exports (export { MyClass }), default exports (export default MyClass), and direct exports (export class MyClass {...}). The query will capture the exported item, which will populate the exports field of the ComponentType::EsModule variant.
Imports: A corresponding query will target import_statement nodes, capturing the module source (the path string) and the imported symbols (whether default, named, or namespace imports). This information is fundamental for the dependency analysis in Section 4 and will generate DependencyRelationship::Import edges.
Table 2 provides a detailed, actionable mapping from tree-sitter queries to the ComponentType variants they populate, serving as a clear specification for the implementation team.
Table 2: Tree-sitter Query-to-Component Mapping
Language
ComponentType
tree-sitter Node
Key Captures
Sample Query Snippet
Rust
Struct
struct_item
@vis, @name, @field_name, @field_type
(struct_item visibility: (visibility_modifier)? @vis name: (type_identifier) @name body: (field_declaration_list) @body)
Rust
Impl
impl_item
@trait_name, @impl_target
(impl_item trait: (type_identifier)? @trait_name type: (type_identifier) @impl_target...)
Rust
Function
function_item
@vis, @name, @params, @return
(function_item visibility: (visibility_modifier)? @vis name: (identifier) @name parameters: (parameters) @params return_type: (_)? @return)
Python
Class
class_definition
@name, @base_class
(class_definition name: (identifier) @name superclasses: (argument_list (identifier) @base_class)*)
Python
Method
decorated_definition
@decorator, @name, @params
(decorated_definition decorator: (decorator (identifier) @decorator)* definition: (function_definition name: (identifier) @name parameters: (parameters) @params))
JavaScript
Class
class_declaration
@name, @extends
(class_declaration name: (identifier) @name superclass: (identifier)? @extends)
JavaScript
EsModule Export
export_statement
@exported_item, @source
(export_statement (declaration) @exported_item) (export_statement source: (string) @source)
JavaScript
Function
function_declaration
@async, @name, @generator
(function_declaration "async"? @async name: (identifier) @name)


4. Global Dependency Resolution and Graph Construction

Extracting individual components is only the first step. To create a meaningful architectural diagram, these components must be connected by their dependencies. This section addresses the complex challenge of moving from a collection of components isolated within files to a fully connected, project-wide dependency graph. This requires a semantic analysis layer that operates on top of the syntactic information provided by tree-sitter.

4.1. A Formal Taxonomy of Inter-Component Dependencies

To ensure the resulting graph is precise and semantically rich, we must first establish a clear taxonomy of the dependency types to be detected. The DependencyRelationship enum will serve as the set of possible edge types in our EnhancedDependencyGraph.
Static Structural Dependencies: These relationships are declared explicitly in the code's structure and can be identified with high confidence from the AST.
Import { module, items } / Use { path, alias }: Represents a file-level dependency declared via import (JS, Python) or use (Rust) statements. This is the most fundamental link between files.
Extends { parent }: Represents class inheritance in object-oriented languages (Python, JS). This defines an "is-a" relationship and is critical for understanding type hierarchies.
Implements { trait_name }: Represents a Rust type providing a concrete implementation for a trait. This is the cornerstone of Rust's interface and polymorphism model.
Usage-Based Dependencies: These relationships represent interactions and data flow between components. While we detect them statically, they model behavior that occurs at runtime. Their accurate detection requires symbol resolution.
MethodCall { target, method }: A call to a method on an object or type (e.g., user.get_profile() or String::new()). Resolving the target's type is essential to link the call to the correct method definition.
FieldAccess { target, field }: Accessing a field or property of an object (e.g., user.id). This also requires resolving the target's type.
Instantiation { class }: The creation of a new object instance (e.g., new User() in JS or User{...} in Rust). This links the code that creates the object to the object's class/struct definition.
Compositional Dependencies: These dependencies represent structural ownership or "has-a" relationships, indicating that one component is part of another's definition.
HasField { field_type }: A struct or class contains a field of another component's type (e.g., struct Order { user: User }). This indicates a strong coupling.
Contains { element_type }: A component holds a collection of another component type (e.g., orders: Vec<Order>). This represents a one-to-many relationship.
Aggregates { component }: A more general form of composition where one component logically groups others, often inferred from function parameters or return types.

4.2. The Cross-File Resolution Challenge: Limitations of Purely Syntactic Analysis

A critical realization in designing this system is that a purely syntactic, single-file analysis is insufficient for accurate dependency detection. tree-sitter is a parser; it operates on the syntax of one file at a time and has no semantic, cross-file context.31 For example,
tree-sitter can parse a method call user.get_profile(), identifying it as a call_expression. However, it cannot determine the type of the user variable. The definition of user and its type User might be in the same file, or it might be imported from another file entirely. Without knowing the type of user, it is impossible to link the method call to the specific get_profile method defined on the User class.
This limitation is the primary motivation for developing a higher-level semantic analysis engine, embodied by the SymbolResolver. This engine's purpose is to build a global understanding of the entire codebase, enabling it to resolve these cross-file references accurately. The ability to perform "cross-function/cross file analysis" is a key feature of advanced commercial static analysis tools like SonarQube, highlighting both its importance and its complexity.12 Our framework must implement this capability to meet its accuracy goals.

4.3. Algorithm Specification: A Two-Phase Approach to Building a Global Symbol Table

To resolve symbols across an entire codebase, we must first build a comprehensive map of every symbol (class, function, variable, etc.) to its definition. This map is commonly known as a global symbol table.33 For large codebases, constructing this table efficiently and accurately requires a structured, multi-pass approach.
Phase 1: Declaration and Indexing Pass
This initial pass scans the entire codebase to build an index of all declared symbols and their relationships. It is a "wide but shallow" analysis.
File Iteration: The process begins by traversing the file system to identify all relevant source files for the project.
Component & Symbol Extraction: For each file, the AstParser uses a set of high-level tree-sitter queries to find all top-level declarations (e.g., class_definition, function_item, struct_item, export_statement).
Global Symbol Table Population: For each declaration found, a SymbolInfo record is created. This record contains the symbol's name, its kind (function, class, etc.), its file path, its source location, and its visibility (e.g., pub in Rust, or whether it's part of an export statement in JS). These records are added to the GlobalSymbolTable, which is implemented as a HashMap<String, Vec<SymbolInfo>>. Using a Vec allows the system to handle cases where the same symbol name is declared in multiple files.
Import/Export Graph Construction: Concurrently, the parser executes queries to identify all import and export statements (use in Rust). This information is used to build an ImportGraph, which explicitly maps which files import which symbols from other files. This graph is crucial for the resolution phase.
Phase 2: Resolution and Dependency Linking Pass
This second pass performs a deeper analysis of each file, using the global indexes built in Phase 1 to resolve dependencies.
Symbol Resolver Initialization: An instance of the SymbolResolver is created, equipped with the GlobalSymbolTable and ImportGraph from the first pass.
Deep File Analysis: The system iterates through each source file again. For each file, a FileContext is established, which contains information about the symbols declared locally and the symbols explicitly imported into that file's scope.
Dependency Identification: The parser traverses the AST of the file, this time looking for usage-based dependency patterns, such as method calls, property accesses, and type instantiations.
Symbol Resolution: When an unresolved symbol is encountered (e.g., the type of a variable in a MethodCall target), the SymbolResolver is queried. The resolver executes the cascade algorithm (detailed in 4.4) to find the definitive declaration of that symbol anywhere in the codebase.
Graph Edge Creation: Once a dependency is successfully resolved—for example, a method call in file_A.rs is linked to a method on a struct defined in file_B.rs—a corresponding DependencyRelationship edge is created. This edge connects the two ArchitecturalComponent nodes in the EnhancedDependencyGraph.

4.4. The Symbol Resolution Cascade: An Algorithm for Linking Symbols Across Scopes

The SymbolResolver::resolve_symbol method is the brain of the dependency analysis engine. Its logic mimics the process a compiler or interpreter uses to resolve a name to its declaration, following a specific order of precedence.33
Given a symbol name S being used within a specific FileContext F (which represents a file and a location within it), the resolution algorithm proceeds as follows:
Local Scope First: The resolver first searches for a declaration of S within the most immediate local scope (e.g., the current function or block). This includes function parameters, local variables, and loop variables.
File/Module Scope: If not found locally, the search expands to the top level of the current file F. This includes functions, classes, and other components defined within the same file.
Import Resolution (The Crucial Cross-File Step): If the symbol is still unresolved, the resolver consults the ImportGraph for file F. It checks if S matches a named import (e.g., from './utils' import S). If a match is found, the resolver follows the import path to the source module and looks up the exported symbol there using the GlobalSymbolTable. This is the primary mechanism for traversing file boundaries.
Language-Specific Namespace Traversal: For languages with explicit namespacing, like Rust, the resolver handles qualified paths (e.g., my_crate::components::MyStruct). It traverses the module hierarchy defined by the project structure to locate the symbol.
Global Fallback: As a final step, the resolver can search the entire GlobalSymbolTable for any public/exported symbols named S that might be implicitly available in the global scope.
Heuristics for Dynamic Languages: For highly dynamic languages like Python and JavaScript, where static resolution can be ambiguous (due to duck typing or prototype manipulation), the system may need to employ heuristics. If a definitive static link cannot be established, it can create a lower-confidence dependency based on name matching, which can be flagged in the UI as a "potential" link.

4.5. Instantiating the EnhancedDependencyGraph

The final output of this two-phase process is a fully instantiated EnhancedDependencyGraph. The nodes of this graph are the ArchitecturalComponent instances identified during the declaration pass and enriched with metrics and other details during the resolution pass. The edges are the DependencyRelationship instances created each time the SymbolResolver successfully links a usage of a symbol to its definition. This graph represents a comprehensive, project-wide model of the software's static architecture.
Table 3 provides formal definitions for each dependency type, including code examples and their architectural significance.
Table 3: Dependency Relationship Semantics
DependencyRelationship
Description
Example (Language)
How to Detect
Architectural Significance
Import
A file-level import of a module or specific items from it.
from pathlib import Path (Python)
Parse import_statement or import_from_statement nodes.
Establishes a foundational static link between files, forming the basis of the module dependency graph.
Extends
A class inherits from a parent class, establishing an "is-a" relationship.
class Dog extends Animal {} (JS)
Parse class_declaration and capture the superclass or bases node.
Models the inheritance hierarchy, which is fundamental to object-oriented design and polymorphism.
Implements
A struct provides a concrete implementation for a trait's interface.
impl Display for MyStruct {} (Rust)
Parse impl_item and capture both the trait and type identifiers.
Represents adherence to a contract, a core concept in Rust's architecture for achieving polymorphism and code reuse.
MethodCall
An instance or static method is called on an object or type.
my_obj.do_something() (Python)
Identify a call_expression, resolve the type of my_obj, and find the do_something method on that type's definition.
Represents a direct runtime interaction and creates functional coupling between the caller and the callee.
Instantiation
A new instance of a class or struct is created.
let user = User::new(); (Rust)
Identify instantiation syntax (e.g., new keyword, struct literal), resolve the type being instantiated.
Indicates a "creates" relationship, where one component is responsible for the lifecycle of another.
HasField
A struct or class contains another resolved component type as a data member.
struct A { b: B } (Rust)
Parse field_declaration within a struct/class, resolve the type of the field.
Represents a strong "has-a" or composition relationship, indicating tight structural coupling.


5. Data Structures for Hierarchical Representation and Analysis

With components and dependencies identified, the next critical step is to organize this information into data structures that are optimized for the types of queries and analyses required for architectural visualization. This involves representing both the containment hierarchy of components and the network of their dependencies.

5.1. The ComponentHierarchy Model for Navigational Queries

Source code is inherently hierarchical: files exist within modules, methods within classes, and classes within files. The ComponentHierarchy structure is designed to represent this physical and logical nesting explicitly. This enables efficient querying for contextual relationships, such as finding all components within the same module or all methods belonging to a class.
The proposed structure is defined as:

Rust


pub struct ComponentHierarchy {
    pub root: ComponentNode, // A conceptual root, e.g., the project root
    pub children: HashMap<Uuid, Vec<ComponentNode>>, // Maps a parent component's ID to its direct children
    pub parent_map: HashMap<Uuid, Uuid>, // Maps a child component's ID to its parent's ID
}


This design allows for fast execution of several key navigational queries:
get_siblings(component_id): This can be implemented by first using the parent_map to find the parent's Uuid, and then using the children map to retrieve the list of all children for that parent, filtering out the original component_id.
get_descendants(component_id): This can be implemented with a recursive or iterative traversal. Starting with the given component_id, the children map is used to find the direct children. The function then calls itself on each child, accumulating the results. This is useful for analyses that need to aggregate information up the hierarchy, such as calculating the total complexity of a module by summing the complexity of all its functions.
find_path(from, to): This is essential for visualizing the relationship between two components in a hierarchical context (e.g., showing the module path between them). This can be implemented by traversing up from both from and to using the parent_map until a common ancestor is found, then combining the paths.

5.2. Implementing the EnhancedDependencyGraph in Rust: A Comparative Analysis of Graph Libraries

The EnhancedDependencyGraph is the central data structure that models the entire system's architecture as a network. It is a directed graph where nodes are ArchitecturalComponents and edges are DependencyRelationships.

Rust


pub struct EnhancedDependencyGraph {
    pub graph: DiGraph<ArchitecturalComponent, DependencyRelationship>,
    pub component_index: HashMap<Uuid, NodeIndex>,
    //... other fields
}


The choice of a graph library in Rust is critical for performance and functionality. Several mature options exist, with petgraph being one of the most prominent and widely used.36
petgraph: This library is a strong candidate. It provides multiple graph representations, including Graph (an adjacency list, suitable for dynamic graphs) and StableGraph (which ensures node indices remain valid after removals).36 Its
DiGraph type is a natural fit for our directed dependency graph. It offers a comprehensive suite of graph algorithms, including traversals, cycle detection, and shortest path, which are directly applicable to our analysis needs.37 The library also supports exporting to DOT format for debugging and visualization.36
graph (from graph_builder): This is another high-performance option, specifically designed for large-scale graphs. It uses a Compressed Sparse Row (CSR) data structure, which is highly efficient for memory usage and concurrent access, making it suitable for analyzing massive codebases.38 While potentially more performant for static, read-heavy analysis, its API might be less flexible for the dynamic graph construction we require during the resolution phase.
Recommendation: For this project, petgraph is the recommended choice. Its balance of performance, rich feature set, and ergonomic API makes it well-suited for building the EnhancedDependencyGraph. The DiGraph structure provides the necessary directed graph model, and its built-in algorithms will accelerate the implementation of advanced analytical features. The component_index (HashMap<Uuid, NodeIndex>) will serve as a crucial bridge, allowing for fast lookups of graph nodes by their stable ArchitecturalComponent ID.

5.3. Advanced Graph Algorithms: Cycle Detection and Coupling Metrics (Afferent vs. Efferent)

Once the EnhancedDependencyGraph is constructed, it becomes a powerful tool for architectural analysis. Several advanced algorithms can be applied to extract critical insights.
Cycle Detection: Circular dependencies between components are a significant architectural anti-pattern, often leading to tightly coupled, brittle systems that are difficult to maintain and test. The EnhancedDependencyGraph enables the detection of these cycles. petgraph provides an implementation of Tarjan's algorithm for finding strongly connected components, which can be used to identify all cycles in the graph. The find_cycles method will return a list of paths, where each path represents a dependency cycle. This provides developers with actionable information to refactor their code and break these undesirable dependencies.
Coupling Metrics Calculation: Coupling is a measure of the degree of interdependence between software modules. The EnhancedDependencyGraph contains all the information needed to calculate standard coupling metrics.
Efferent Coupling (Ce): This measures the outgoing dependencies of a component. It is a count of how many other components a given component depends on. It can be calculated for a component with NodeIndex ix by simply counting the number of outgoing edges: graph.edges_directed(ix, Outgoing).count(). This value will be stored in the ComponentMetrics struct.
Afferent Coupling (Ca): This measures the incoming dependencies of a component. It is a count of how many other components depend on a given component. It can be calculated by counting the number of incoming edges: graph.edges_directed(ix, Incoming).count(). This value will also be stored in ComponentMetrics.
These metrics are invaluable for identifying architectural hotspots. A component with high efferent coupling may have too many responsibilities (violating the Single Responsibility Principle), while a component with high afferent coupling is a critical, high-impact part of the system that must be treated with care during modifications.

6. Strategies for Performance and Scalability in Large-Scale Codebases

Analyzing codebases with over 1000 files presents significant performance and memory challenges. A full, from-scratch analysis on every change is not viable for providing the rapid feedback developers expect. Therefore, a robust strategy for incremental analysis and memory optimization is not an optional feature but a core requirement of the system architecture. This section outlines a multi-pronged approach to ensure the framework is both fast and scalable.

6.1. The Incremental Extraction Pipeline: Caching, Fingerprinting, and Partial Re-analysis

The key to performance is to avoid redundant work. Since developers typically only modify a small subset of files at a time, the analysis should be able to reuse results for unchanged files. This principle is central to modern incremental static analysis frameworks.39 The
IncrementalComponentExtractor will orchestrate this process.
File Fingerprinting: Before parsing any file, the system will compute a "fingerprint" or hash (e.g., SHA-256) of its content. This fingerprint is stored in the file_cache alongside the results of its last analysis. When an analysis is triggered, the system re-computes the fingerprint for each file and compares it to the cached version. Only files with a changed fingerprint need to be re-analyzed.42
Cached Component Extraction: For files that have not changed, the previously extracted Vec<ArchitecturalComponent> can be retrieved directly from the component_cache. This avoids the cost of parsing and querying the AST for a vast majority of the codebase during a typical edit-compile cycle. The effectiveness of this strategy relies on a high cache hit ratio, which is expected in common development workflows.43
Incremental Dependency Graph Update: The most complex part of the incremental process is updating the dependency graph. A change in one file (file_A) can have cascading effects on other files that depend on it.
Step 1: Re-analyze Changed Files: For each modified file, run the full component and dependency extraction process (Sections 3 and 4). This will produce a new set of components and local dependency relationships for that file.
Step 2: Identify API Changes: Compare the new components from the changed file with the cached version. Did any public-facing signatures change? Was a public function removed? Was a struct's field layout altered?
Step 3: Propagate Changes: If a public API has changed, the system must identify all files that depend on that API. This is where the ImportGraph (or the reverse edges of the EnhancedDependencyGraph) becomes critical. All dependent files must be added to a "to-be-re-resolved" worklist.
Step 4: Partial Re-resolution: For files in the worklist, the system does not need to re-parse them (as their content hasn't changed), but it does need to re-run the dependency resolution pass (Phase 2 from Section 4.3). This will update their outgoing dependency edges to reflect the changes in the APIs they consume.
This approach, which combines file-level caching with dependency-driven partial re-analysis, is designed to make the update time proportional to the size of the change and its impact, rather than the size of the entire codebase.44

6.2. Memory Optimization via Data Compaction and String Interning

For very large codebases, the memory footprint of storing thousands of ArchitecturalComponent structs and a dense dependency graph can become a significant issue. Several techniques can be employed to create a more memory-efficient representation, as embodied by the CompactComponent struct.
String Interning: Throughout a codebase, many strings are repeated thousands of times (e.g., common variable names, type names like "String" or "Vec"). A StringInterner is a data structure that stores each unique string only once. It consists of a Vec<String> and a HashMap<String, u32>. When a string needs to be stored, the interner is consulted. If the string is new, it's added to the Vec and a new ID is assigned. If it already exists, its existing ID is returned. Instead of storing full String objects in every component, we store a compact u32 ID. This can dramatically reduce memory usage, especially for fields like name, file_path, and type names within dependencies.
Data Compaction: The ArchitecturalComponent struct, while ergonomic, may not be memory-optimal. The CompactComponent struct demonstrates several compaction strategies:
id: u32: Using a simple incrementing integer ID managed by a central registry is more compact than a 128-bit Uuid.
name_id: u32, file_id: u32: These fields store the u32 IDs from the string interner instead of full String or PathBuf objects.
component_type: u8: The ComponentType enum can be represented as a single byte, assuming there are fewer than 256 variants.
Metrics Packing: ComponentMetrics can also be packed into smaller integer types or a bitfield if their range of values allows it.
This compact representation would be used for the primary in-memory storage of the graph. The full ArchitecturalComponent struct could then be reconstructed on-demand when an API needs to expose a more developer-friendly representation.

6.3. A Proposed Benchmarking Suite for Validation Against Success Criteria

To validate that these performance strategies are effective and that the system meets its goals, a comprehensive benchmarking suite must be developed. This suite will measure performance against the key success criteria outlined in the research request.
Extraction Time vs. File Size: Measure the time taken to perform a full, from-scratch analysis on a variety of real-world open-source projects of different sizes (e.g., 10 files, 100 files, 1000+ files). This will establish a baseline and test the <100ms per file target.
Memory Usage Patterns: Profile the application's memory usage during the analysis of a large (1000+ file) codebase. This will measure heap allocations and total memory footprint, validating the <50MB target. The effectiveness of string interning and data compaction can be quantified here.
Cache Hit Rates: Instrument the IncrementalComponentExtractor to log cache hits and misses for both the file fingerprint cache and the component cache. Measure the hit rate during simulated common development scenarios (e.g., changing one file, changing five related files).
Incremental Parsing Benefits: Measure the time taken for an incremental update after a small change versus the time for a full re-analysis. The benchmark should demonstrate that the incremental update is significantly faster, ideally an order of magnitude or more for localized changes.
Accuracy Measurement: To validate the 95%+ accuracy in dependency detection goal, the system will be run on a set of projects with a manually-verified "ground-truth" dependency graph. The system's output will be compared against this ground truth, and precision/recall metrics will be calculated.
Table 4 summarizes the proposed optimization techniques and their expected impact.
Table 4: Performance and Memory Optimization Techniques

Strategy
Description
Expected Impact
Implementation Complexity
Relevant Research
Incremental Parsing
Re-parsing only files that have changed, using tree-sitter's ability to reuse unchanged parts of the tree.
Dramatically reduces analysis time for subsequent runs after small code changes.
Medium (Requires managing old and new trees).
19
Component Caching
Storing the extracted ArchitecturalComponent vectors for each file, keyed by a file content hash.
Avoids re-running extraction queries on unchanged files, significantly speeding up the "Declaration Pass".
Low (Requires a persistent key-value store or in-memory cache).
41
Dependency-Aware Re-resolution
When a file's public API changes, only re-run the dependency resolution pass on files that directly or indirectly depend on it.
Prevents a full, project-wide re-resolution for local changes, making updates proportional to impact.
High (Requires a robust import/dependency graph and change propagation logic).
39
String Interning
Storing each unique string (names, paths, types) only once and referencing it by a compact ID.
Substantially reduces memory footprint, especially in large codebases with many repeated identifiers.
Medium (Requires a thread-safe interning data structure).
N/A (Standard compiler optimization)
Data Compaction
Using smaller data types (e.g., u32 for IDs, u8 for enums) in the core graph data structures.
Reduces the overall memory size of the dependency graph, improving cache locality and performance.
Medium (Requires a mapping layer between compact and ergonomic structs).
46


7. Integration and Migration Plan

Successfully integrating this advanced analysis framework into an existing system requires a clear, phased plan that addresses API design, data model migration, and persistence layer updates.

7.1. API Specification: The ComponentExtractor Trait

To ensure a clean separation of concerns and to support future extensibility (e.g., adding new languages), the core functionality will be exposed through a set of traits. The primary entry point will be the ComponentExtractor trait.

Rust


pub trait ComponentExtractor {
    // Performs the full, multi-pass analysis on a set of files
    fn extract_graph(
        &self,
        files: &,
        // Optional previous state for incremental analysis
        previous_state: Option<AnalysisCache>,
    ) -> Result<(EnhancedDependencyGraph, AnalysisCache), ExtractionError>;

    // A simpler method for single-file, non-dependency-aware extraction
    fn extract_components_from_file(
        &self,
        file_path: &PathBuf,
    ) -> Result<Vec<ArchitecturalComponent>, ExtractionError>;

    fn supported_languages(&self) -> Vec<SourceLanguage>;
}


A concrete implementation, MultiLanguageExtractor, will encapsulate the language-specific logic. It will contain instances of RustComponentExtractor, PythonComponentExtractor, and JavaScriptComponentExtractor, each responsible for handling the tree-sitter queries and language-specific rules for its target language. The MultiLanguageExtractor will orchestrate the multi-pass analysis, delegating file-level parsing to the appropriate sub-extractor based on file extension.

7.2. A Phased Migration from ComponentNode to ArchitecturalComponent

Migrating from the existing, simple ComponentNode enum to the rich ArchitecturalComponent struct must be managed to minimize disruption. A phased approach is recommended.
Phase 1: Coexistence and Backward Compatibility: Initially, the new extraction pipeline will run in parallel to the old one. A translation layer or adapter function will be created: fn from_architectural_component(ac: &ArchitecturalComponent) -> ComponentNode. This function will map the new, detailed structure back to the old, simpler enum. For example, ComponentType::Struct and ComponentType::Class would both map to ComponentNode::Class. This allows the rest of the application to continue functioning with the old data model while the new pipeline is being validated.
Phase 2: Feature Flagging: The new features powered by the EnhancedDependencyGraph (e.g., detailed dependency views, cycle detection) will be introduced behind feature flags. This allows for gradual rollout and testing with a subset of users or in specific environments.
Phase 3: Deprecation and Removal: Once the new ArchitecturalComponent model and its associated features are stable and have proven their value, the old ComponentNode enum and the legacy extraction logic can be deprecated. A migration script may be necessary to update any persisted data (see Section 7.3) to the new format. Finally, the old code and the translation layer can be removed.

7.3. Persistence Layer: Schema Design for Component and Dependency Data

To support the caching and incremental analysis strategies, the results of the analysis must be persisted. This requires a database schema capable of storing the components, their dependencies, and the cache metadata. A graph database (like Neo4j) or a relational database with foreign key relationships could be used.
A relational schema might look like this:
components Table
id (UUID, Primary Key)
name (TEXT)
file_path (TEXT)
component_type (VARCHAR)
start_line, start_col, end_line, end_col (INTEGER)
visibility (VARCHAR)
metrics_json (JSONB) - Storing metrics as a flexible JSON object.
type_specific_data_json (JSONB) - Storing language-specific details (e.g., struct fields, class bases).
dependencies Table
id (SERIAL, Primary Key)
source_component_id (UUID, Foreign Key to components.id)
target_component_id (UUID, Foreign Key to components.id)
relationship_type (VARCHAR)
relationship_metadata_json (JSONB) - For details like the name of the method called.
file_cache Table
file_path (TEXT, Primary Key)
content_hash (TEXT)
last_analyzed_timestamp (TIMESTAMP)
This schema allows for the efficient storage and retrieval of the architectural model, providing the foundation for the incremental pipeline and enabling complex queries against the persisted dependency graph.

8. Advanced Topics and Future Research Directions

While the core framework provides a robust foundation for static architectural analysis, several areas present opportunities for future enhancement and research. Addressing these topics can further increase the accuracy and utility of the generated diagrams, particularly for complex, real-world codebases.

8.1. Heuristics for Analyzing Dynamic Language Features

Statically analyzing dynamic languages like Python and JavaScript presents inherent challenges. Features such as Python's duck typing and monkey-patching, or JavaScript's prototype-based inheritance and dynamic property access, make it impossible to resolve all dependencies with 100% certainty through static analysis alone.
Challenge: For example, in Python, if a function receives an object x and calls x.process(), the actual method being called depends on the type of x at runtime. Multiple classes could have a process method, and static analysis may not be able to definitively determine which one is the target.
Proposed Heuristic: When definitive static resolution fails, the system can fall back to name-based heuristics. It could identify all components named process that are methods and create low-confidence, "potential" dependency links to all of them. These could be visualized differently in the UI (e.g., with a dashed line) to indicate the uncertainty to the user.
Future Work: More advanced techniques, such as limited data-flow analysis or type inference (leveraging type hints in Python or TypeScript for JS), could be integrated to improve the accuracy of dependency resolution in these dynamic contexts.48

8.2. Exploring the Accuracy-Performance Frontier

The framework is designed with a specific point on the accuracy-performance spectrum in mind, prioritizing high-fidelity dependency information. However, this is not a fixed point.
Configurable Analysis Depth: A future enhancement could be to make the analysis depth configurable. A "quick scan" mode might only analyze file-level imports and explicit inheritance, providing a very fast but coarse-grained architectural overview. An "in-depth scan" mode would enable the full cross-file symbol resolution and data-flow analysis, providing maximum accuracy at a higher computational cost.
Adaptive Analysis: The system could be made adaptive. For instance, it could perform a quick scan initially and then run deeper analyses on specific, user-selected parts of the codebase in the background. This would provide immediate value while progressively enriching the architectural model over time.

8.3. Potential for Machine Learning-Assisted Architecture Recovery

The field of software engineering is increasingly exploring the use of machine learning to augment traditional program analysis.49
Component Naming and Grouping: While our approach uses directory structure for naming heuristics, a model trained on large codebases could learn common naming conventions and suggest more semantically meaningful names for discovered components.10
Dependency Inference: For the ambiguous dependencies in dynamic languages mentioned in 8.1, an ML model could be trained to predict the most likely target of a dynamic dispatch call based on contextual information, such as variable names and the call site's location.
Anti-Pattern Detection: Models can be trained to recognize common architectural anti-patterns (e.g., God Objects, cyclic dependencies) directly from the graph structure and component metrics, moving beyond simple rule-based detection.49
These future directions represent a path from a highly accurate static analysis tool to an intelligent architectural co-pilot, capable of providing deeper insights and more proactive guidance to developers. The robust and detailed EnhancedDependencyGraph produced by this framework is the ideal data structure to serve as the input for such advanced analytical techniques.
Works cited
Static Program Analysis - Department of Computer Science, accessed July 7, 2025, https://cs.au.dk/~amoeller/spa/spa.pdf
Scaling Symbolic Execution to Large Software Systems - arXiv, accessed July 7, 2025, https://arxiv.org/html/2408.01909v1
Source Code Analysis Tools - OWASP Foundation, accessed July 7, 2025, https://owasp.org/www-community/Source_Code_Analysis_Tools
Static Code Analysis: Everything You Need To Know - Codacy | Blog, accessed July 7, 2025, https://blog.codacy.com/static-code-analysis
Static Source Code Analysis: Getting Started | Xygeni, accessed July 7, 2025, https://xygeni.io/blog/static-source-code-analysis-getting-started/
A hands-on introduction to static code analysis - DeepSource, accessed July 7, 2025, https://deepsource.com/blog/introduction-static-code-analysis
a systematic analysis on software architecture recovery techniques - ResearchGate, accessed July 7, 2025, https://www.researchgate.net/publication/384241240_A_SYSTEMATIC_ANALYSIS_ON_SOFTWARE_ARCHITECTURE_RECOVERY_TECHNIQUES
Comparing Software Architecture Recovery Techniques Using ..., accessed July 7, 2025, https://www.cs.purdue.edu/homes/lintan/publications/archrec-icse15.pdf
Measuring the Impact of Code Dependencies on Software Architecture Recovery Techniques - Computer Science Purdue, accessed July 7, 2025, https://www.cs.purdue.edu/homes/lintan/publications/archrec-tse17.pdf
Recover and Optimize Software Architecture Based ... - KSI Research, accessed July 7, 2025, https://ksiresearch.org/seke/seke19paper/seke19paper_45.pdf
Efficient Static Vulnerability Analysis for JavaScript ... - andrew.cmu.ed, accessed July 7, 2025, https://www.andrew.cmu.edu/user/liminjia/research/papers/graphjs-pldi24.pdf
C Static Code Analysis Tool & Clean Code Programming Language - Sonar, accessed July 7, 2025, https://www.sonarsource.com/knowledge/languages/c/
Scala Static Code Analysis Tool & Clean Code Programming Language - SonarSource, accessed July 7, 2025, https://www.sonarsource.com/knowledge/languages/scala/
Software Architecture Recovery with Information Fusion (ESEC/FSE 2023 - Research Papers), accessed July 7, 2025, https://2023.esec-fse.org/details/fse-2023-research-papers/44/Software-Architecture-Recovery-with-Information-Fusion
A Guide to ES6 Import and Export Usage in Node.js | by Oluwaseun - Stackademic, accessed July 7, 2025, https://blog.stackademic.com/a-guide-to-es6-import-and-export-usage-in-node-js-b32a707fa103
accessed December 31, 1969, https://github.com/tree-sitter/tree-sitter-python/blob/master/grammar.js
Rust grammar for tree-sitter - GitHub, accessed July 7, 2025, https://github.com/tree-sitter/tree-sitter-rust
tree-sitter/tree-sitter-javascript: Javascript grammar for tree ... - GitHub, accessed July 7, 2025, https://github.com/tree-sitter/tree-sitter-javascript
Incremental Parsing Using Tree-sitter - Strumenta - Federico Tomassetti, accessed July 7, 2025, https://tomassetti.me/incremental-parsing-using-tree-sitter/
Tree-sitter: Introduction, accessed July 7, 2025, https://tree-sitter.github.io/
Lightweight linting with tree-sitter - DeepSource, accessed July 7, 2025, https://deepsource.com/blog/lightweight-linting
tree-sitter/tree-sitter: An incremental parsing system for programming tools - GitHub, accessed July 7, 2025, https://github.com/tree-sitter/tree-sitter
Unraveling Tree-Sitter Queries: Your Guide to Code Analysis Magic - DEV Community, accessed July 7, 2025, https://dev.to/shrsv/unraveling-tree-sitter-queries-your-guide-to-code-analysis-magic-41il
Knee Deep in tree-sitter Queries - Hackerman's Hacking Tutorials, accessed July 7, 2025, https://parsiya.net/blog/knee-deep-tree-sitter-queries/
Refactoring Python with Tree-sitter and Jedi - Hacker News, accessed July 7, 2025, https://news.ycombinator.com/item?id=41637286
tree-sitter explained - YouTube, accessed July 7, 2025, https://www.youtube.com/watch?v=09-9LltqWLY
nvim-treesitter/queries/rust/highlights.scm at master - GitHub, accessed July 7, 2025, https://github.com/nvim-treesitter/nvim-treesitter/blob/master/queries/rust/highlights.scm
Python grammar for tree-sitter - GitHub, accessed July 7, 2025, https://github.com/tree-sitter/tree-sitter-python
tree-sitter-python/queries/highlights.scm at master - GitHub, accessed July 7, 2025, https://github.com/tree-sitter/tree-sitter-python/blob/master/queries/highlights.scm
accessed December 31, 1969, https://github.com/tree-sitter/tree-sitter-javascript/blob/master/grammar.js
Using tree sitter to render class/struct specific information : r/emacs - Reddit, accessed July 7, 2025, https://www.reddit.com/r/emacs/comments/zd9rmp/using_tree_sitter_to_render_classstruct_specific/
Tree-sitter: are we there yet? : r/vim - Reddit, accessed July 7, 2025, https://www.reddit.com/r/vim/comments/1b5wpzl/treesitter_are_we_there_yet/
Symbol Tables and Static Checks - cs.wisc.edu, accessed July 7, 2025, https://pages.cs.wisc.edu/~fischer/cs536.s08/course.hold/html/NOTES/6.SYMBOL-TABLES.html
Symbol Table in Compiler - GeeksforGeeks, accessed July 7, 2025, https://www.geeksforgeeks.org/compiler-design/symbol-table-compiler/
Compiler Design - Symbol Table - Tutorialspoint, accessed July 7, 2025, https://www.tutorialspoint.com/compiler_design/compiler_design_symbol_table.htm
petgraph/petgraph: Graph data structure library for Rust. - GitHub, accessed July 7, 2025, https://github.com/petgraph/petgraph
Graphs in Rust: An Introduction to Petgraph | Depth-First, accessed July 7, 2025, https://depth-first.com/articles/2020/02/03/graphs-in-rust-an-introduction-to-petgraph/
graph - Rust - Docs.rs, accessed July 7, 2025, https://docs.rs/graph
Change Pattern Detection for Optimising Incremental Static Analysis - Software Languages Lab - Vrije Universiteit Brussel, accessed July 7, 2025, http://soft.vub.ac.be/Publications/2023/vub-tr-soft-23-13.pdf
Fixpoint Reuse for Incremental JavaScript Analysis - Ben Hardekopf, accessed July 7, 2025, https://hardekbc.github.io/files/nichols19fixpoint.pdf
Reusing Caches and Invariants for Efficient and Sound ... - DROPS, accessed July 7, 2025, https://drops.dagstuhl.de/storage/00lipics/lipics-vol333-ecoop2025/LIPIcs.ECOOP.2025.28/LIPIcs.ECOOP.2025.28.pdf
CodeFuse-Query: A Data-Centric Static Code Analysis System for Large-Scale Organizations - arXiv, accessed July 7, 2025, https://arxiv.org/html/2401.01571v1
How does caching work and what are common caching strategies in system design?, accessed July 7, 2025, https://www.designgurus.io/answers/detail/how-does-caching-work-and-what-are-common-caching-strategies-in-system-design
Incremental CodeQL - GitHub Next, accessed July 7, 2025, https://githubnext.com/projects/incremental-codeql/
Efficient and Flexible Incremental Parsing | Harmonia, accessed July 7, 2025, https://harmonia.cs.berkeley.edu/papers/twagner-parsing.pdf
Understanding Data Structures by Extracting Memory Access Graphs - OSTI.GOV, accessed July 7, 2025, https://www.osti.gov/servlets/purl/1813903
Static Memory Leak Detection Using Full-Sparse Value-Flow Analysis - Yulei Sui, accessed July 7, 2025, https://yuleisui.github.io/publications/issta12.pdf
saltudelft/ml4se: A curated list of papers, theses, datasets, and tools related to the application of Machine Learning for Software Engineering - GitHub, accessed July 7, 2025, https://github.com/saltudelft/ml4se
Machine Learning in Static Analysis of Program Source Code - Medium, accessed July 7, 2025, https://medium.com/pvs-studio/machine-learning-in-static-analysis-of-program-source-code-21bdc7e8ce6d
