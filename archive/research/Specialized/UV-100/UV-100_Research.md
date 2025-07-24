
Architectural Blueprint for Multi-Language Tight Coupling Analysis in the UV-100 Framework


Executive Summary

This report presents a comprehensive architectural blueprint for extending the UV-100 static analysis framework with a sophisticated, multi-language tight coupling detector. The primary objective is to address the current framework's limitation in language-specific dependency extraction, thereby enabling accurate calculation of established coupling metrics such as Coupling Between Objects (CBO), Response for Class (RFC), and fan-in/fan-out across Rust, Python, and JavaScript/TypeScript codebases. The research and recommendations herein provide the foundational knowledge required to guide the implementation of this critical new capability, ensuring the resulting tool is accurate, scalable, and extensible.
The proposed architecture is centered on the Tree-sitter parsing framework. Part I of this report provides a detailed analysis of Tree-sitter's query language and delivers language-specific "cookbooks" of queries designed to extract a wide array of dependency types. It details the unique challenges and opportunities presented by each language, from the explicit and strongly-typed nature of Rust, which allows for highly precise analysis, to the dynamic characteristics of Python and JavaScript, which necessitate more complex, heuristic-based approaches and the integration of symbol table analysis to achieve acceptable accuracy.
Part II translates the extracted dependency data into actionable architectural insights. It presents a thorough review of academic literature and industry best practices to establish evidence-based, configurable thresholds for coupling metrics. Recognizing that a single metric is insufficient for a nuanced assessment, this report proposes a multi-factorial severity model. This model combines various metrics and contextual factors—such as dependency type, architectural role, and code context (production vs. test)—into a weighted composite score, allowing for a more sophisticated and accurate classification of coupling issues into "Critical," "High," "Medium," and "Low" severity tiers.
Part III outlines the system architecture required to support this analysis at scale. It details the design of a scalable cross-file analysis engine built upon a dependency graph data structure. Key algorithmic components, including topological sorting for cycle detection and strategies for incremental and parallel analysis, are presented to ensure high performance on large, real-world projects. Furthermore, this section proposes a flexible, plugin-based architecture for the language analyzers and a robust YAML-based configuration system. This design ensures the UV-100 framework is not only powerful but also maintainable and extensible, allowing for the seamless addition of new languages in the future.
Finally, Part IV synthesizes these findings into a strategic implementation plan. It recommends a phased rollout, beginning with the Rust analyzer to establish a solid foundation, followed by the more complex Python and JavaScript/TypeScript implementations. The report concludes with a discussion of key architectural trade-offs and future-proofing strategies, such as the potential integration of semantic and logical coupling analysis, to ensure the UV-100 framework remains at the forefront of code quality analysis.

Part I: Language-Specific Dependency Extraction via Tree-sitter

The foundation of any accurate coupling metric calculation is the ability to precisely identify and categorize the dependencies between software components. This requires a deep, structural understanding of the source code. This section details the methodology for achieving this using the Tree-sitter parsing framework. It begins by establishing the fundamental principles of Tree-sitter querying before providing detailed, language-specific query cookbooks for Rust, Python, and JavaScript/TypeScript.

Section 1.1: Foundations of Tree-sitter Querying for Dependency Analysis

To effectively implement a multi-language coupling detector, the development team must first possess a solid theoretical and practical understanding of how Tree-sitter enables structural code analysis. This section outlines the core concepts that underpin all subsequent language-specific implementations.

The Tree-sitter Paradigm

Tree-sitter is a modern parser generator tool and incremental parsing library designed for use in developer tools.1 Its architecture is particularly well-suited for the UV-100 framework's requirements due to several key design goals: speed sufficient for real-time analysis on every keystroke, robustness in the presence of syntax errors, and a dependency-free C runtime that facilitates embedding in various applications.2
A critical distinction of Tree-sitter is that it produces a Concrete Syntax Tree (CST), not an Abstract Syntax Tree (AST).3 While an AST represents the abstract syntactic structure of the code, often omitting non-semantic elements like punctuation, comments, and whitespace, a CST is a more faithful representation that retains every piece of information from the source file.4 This fidelity is crucial for a comprehensive analysis tool, as it allows queries to match not only on semantic nodes but also on the exact textual representation of the code, which can be vital for certain rules and for providing precise issue locations.

Anatomy of a Tree-sitter Query

Tree-sitter provides a powerful, Lisp-like query language for pattern-matching against the CST.5 A query is an S-expression that describes a path or structure within the tree, allowing for the extraction of specific nodes without writing complex, manual traversal logic.6 Understanding the anatomy of these queries is the first step toward building the dependency extractor.
Node Matching: The fundamental unit of a query is the node pattern, written as (node_type...). This expression matches any node in the CST that has the specified node_type, which is defined in the language's grammar.7 For example,
(function_declaration) would match all function declaration nodes.
Child and Field Matching: To create more specific structural matches, patterns can be nested. A pattern like (binary_expression (number_literal) (number_literal)) matches a binary expression whose children are both number literals.7 Grammars can also assign names to specific children, known as "fields." Queries can match on these fields using the syntax
field_name: (pattern). For example, (call_expression function: (identifier)) matches a call expression where the specific child in the function field is an identifier.6 This is the primary mechanism for targeting specific parts of a larger construct.
Captures: The goal of querying is to extract information. This is achieved through captures, which are defined by prefixing a pattern or node name with an @ symbol (e.g., @capture-name).6 When a query matches, Tree-sitter returns the set of nodes that were "captured" by these names. This allows the UV-100 framework to extract the specific nodes that represent the source and target of a dependency. For instance, in
(function_declaration name: (identifier) @function.name), the @function.name capture extracts the identifier node containing the function's name.5
Predicates: To further refine matches, queries can include predicates. These are special functions that apply conditions to captures. The most common predicates are #eq? and #match?, which filter matches based on whether the text of a captured node is equal to a given string or matches a regular expression, respectively.6 For example,
(identifier) @name (#eq? @name "my_function") will only match identifiers whose text is exactly "my_function". This is invaluable for finding calls to specific functions or dependencies on specific modules.

A Generalized Dependency Model

To ensure a consistent architecture, it is essential to define a generalized, language-agnostic model for representing dependencies. The language-specific analyzers will be responsible for populating this model, which will then be consumed by the metric calculators and issue generators. A dependency can be modeled as a tuple: (Source Component, Target Component, Dependency Type, Location).
Source/Target Component: These are the fully qualified names of the components involved in the dependency (e.g., my_crate::my_module::MyStruct, com.example.MyClass). Resolving these names is a key task of the language analyzer.
Dependency Type: This is an enumeration that categorizes the nature of the coupling. This allows the severity model to weight different types of dependencies appropriately. Example types include: ModuleImport, Inheritance, InterfaceImplementation, FunctionCall, MethodInvocation, FieldAccess, GenericTypeUsage.
Location: This stores the source file path and the precise line/column range of the code that establishes the dependency. This is critical for reporting issues back to the user.
The primary function of the Tree-sitter queries is to identify the nodes corresponding to these dependency types and extract the necessary information to populate this generalized model. The quality and precision of the entire coupling analysis system are therefore directly contingent on the quality and completeness of these queries. An omission or error in a query will result in a silent failure—an incomplete dependency graph—which in turn leads to inaccurate and misleading metrics. This underscores the necessity of rigorous testing of the queries against a diverse corpus of real-world code for each supported language, using tools like the Tree-sitter playground and CLI for validation.6

Table 1: Generic Dependency Query Patterns

To provide a foundational mental model for developers implementing the language analyzers, the following table maps common dependency-creating constructs to their generic Tree-sitter query patterns. While the specific node_type names will vary by language, the underlying structure is often similar.
Dependency Type
Generic Query Pattern
Description
Function/Method Call
(call_expression function: @callable arguments: @args)
Matches a function or method invocation, capturing the entity being called and its arguments.
Import/Use
(import_statement path: @module)
Matches a module import statement, capturing the path of the imported module.
Inheritance
(class_declaration superclass: @parent)
Matches a class definition that inherits from a parent class, capturing the parent.
Field/Property Access
(field_expression object: @object field: @field)
Matches access to a field or property on an object, capturing the object and the field name.


Section 1.2: Advanced Dependency Queries for Rust

Rust's explicit syntax and strong, static type system make it an ideal candidate for high-precision dependency analysis using Tree-sitter. Its grammar provides distinct node types for different semantic constructs, which allows for the creation of unambiguous queries that require minimal heuristic post-processing. This section provides a comprehensive guide to extracting dependency information from Rust code.

Mapping the Rust AST for Coupling Analysis

A thorough analysis of the tree-sitter-rust grammar is the first step. The grammar.js file for the parser defines the syntax rules and node names 9, while example AST structures provide concrete illustrations of how these nodes are organized.10 For the purpose of coupling analysis, the following node types are of primary importance:
Module and Import Dependencies: use_declaration, use_wildcard, scoped_identifier.
Function and Method Calls: call_expression (for free functions) and method_call_expression (for .-notation calls).
Type Definitions: struct_item, enum_item, union_item.
Field and Member Access: field_expression (for struct fields), tuple_index_expression (for tuple struct fields).
Implementation and Trait Dependencies: impl_item, trait_item, trait_bounds.
Type Usage: type_identifier, generic_type, scoped_type_identifier, reference_type.
Lifetimes: lifetime nodes, while representing a unique Rust-specific dependency, can also be captured for advanced analysis.11

Rust Query Cookbook

The following is a collection of annotated Tree-sitter queries designed to capture the various forms of coupling present in Rust code.
Module and use Dependencies: This is the most direct form of file-level coupling.
Code snippet
; Captures the full path of a 'use' statement.
; e.g., in 'use std::collections::HashMap;', captures 'std::collections::HashMap'.
(use_declaration
  argument: [
    (scoped_identifier) @module.path
    (use_list (scoped_identifier) @module.path)
  ]
) @use.statement

This query correctly handles both single use paths and use lists (e.g., use std::{fmt, io};). The scoped_identifier node represents a ::-separated path.10
Function and Method Calls: Distinguishing between free function calls and method calls is crucial for RFC calculations.
Code snippet
; Captures a standard function call, including the function path and arguments.
; e.g., 'my_mod::my_func(a, b)'
(call_expression
  function: (_) @function.name
  arguments: (arguments) @function.arguments)

The function field can contain a simple identifier or a scoped_identifier, representing the full path to the function being called.6
Code snippet
; Captures a method call, identifying the object, method name, and arguments.
; e.g., 'my_object.do_something()'
(method_call_expression
  receiver: (_) @object
  method: (field_identifier) @method.name
  arguments: (arguments) @method.arguments)

This query specifically targets the .-notation for method calls, capturing the receiver (the object whose method is being called) and the method name separately.
Trait and Struct Implementations: This represents a strong form of coupling where a type conforms to a contract.
Code snippet
; Identifies a trait implementation for a specific type.
; e.g., 'impl MyTrait for MyStruct'
(impl_item
  trait: (type_identifier) @trait.name
  type: (type_identifier) @struct.name)

This query captures the explicit coupling between a concrete type (@struct.name) and the trait it implements (@trait.name). This is a fundamental dependency for CBO calculations.
Struct Field and Enum Variant Usage:
Code snippet
; Captures access to a struct's field.
; e.g., 'my_struct.field_name'
(field_expression
  value: (_) @object
  field: (field_identifier) @field.name)

This query identifies direct data coupling through field access. The value field represents the struct instance being accessed.
Code snippet
; Captures the instantiation of a struct.
(struct_expression
  name: (_) @struct.name)

Instantiating a struct creates a dependency on that struct's definition.
Code snippet
; Captures the usage of an enum variant.
; e.g., 'MyEnum::Variant'
(scoped_identifier
  path: (identifier) @enum.name
  name: (identifier) @variant.name)

Using an enum variant creates a dependency on the enum type itself.
Generic Type and Lifetime Dependencies: While not always included in traditional coupling metrics, these dependencies are important for a complete picture of a component's relationships.
Code snippet
; Captures the use of a generic type parameter in a definition.
; e.g., in 'struct MyWrapper<T>', captures 'T'.
(type_parameters
  (type_parameter
    name: (type_identifier) @generic.param))

; Captures the use of a concrete type in a generic context.
; e.g., in 'let x: MyWrapper<String>;', captures 'String'.
(generic_type
  type: (type_identifier) @generic.type
  type_arguments: (type_arguments
    (type_identifier) @concrete.type))

This captures both the definition and usage of generics, which represent a dependency on the constraints and properties of the types involved.12
Code snippet
; Captures a lifetime annotation in a function or struct definition.
; e.g., in 'fn foo<'a>(x: &'a str)', captures ''a'.
(lifetime (identifier) @lifetime.name)

Capturing lifetime dependencies provides data for more advanced, Rust-specific analyses beyond standard coupling metrics, as they represent a temporal coupling contract.11
The explicit nature of Rust's grammar provides a significant advantage. Unlike more dynamic languages, where a single call node might represent various types of invocations, Rust's grammar distinguishes between call_expression and method_call_expression. Similarly, impl blocks explicitly declare the relationship between a type and a trait. This high degree of precision in the CST means that the queries can be more direct and less reliant on heuristics. Consequently, the dependency graph generated for Rust will be of higher fidelity, leading to more accurate and trustworthy coupling metrics. This makes Rust the ideal candidate for the initial implementation phase of the UV-100 coupling detector, as it allows the team to validate the core framework mechanics against a reliable data source.

Section 1.3: Advanced Dependency Queries for Python

Analyzing Python for tight coupling presents a distinct set of challenges compared to Rust. The language's dynamic nature, duck typing, and idiomatic constructs like decorators require a more sophisticated analysis strategy that combines Tree-sitter's syntactic parsing with semantic, post-processing logic. This section details the queries and the necessary architectural considerations for building an effective Python coupling analyzer.

Mapping the Python AST for Coupling Analysis

An examination of the tree-sitter-python grammar is the prerequisite for writing effective queries.13 The Python bindings and query examples provide valuable context for how these grammar nodes are used in practice.15 Key nodes for dependency analysis include:
Module Dependencies: import_statement and import_from_statement.
Class Structure: class_definition, which includes an optional (argument_list) for superclasses.
Function/Method Structure: function_definition.
Invocations: call is the generic node for all function and method calls.
Member Access: attribute is used for both accessing fields (obj.field) and methods before a call (obj.method).
Decorators: decorated_definition wraps a function or class definition with one or more decorators.

Python Query Cookbook

The following queries are designed to extract syntactic evidence of coupling. They must be paired with the semantic analysis discussed later to achieve high accuracy.
Module Imports: Python has two primary import syntaxes, both of which must be captured.
Code snippet
; Captures a standard 'import X' or 'import X.Y as Z'.
(import_statement name: (dotted_name) @module.name)

Code snippet
; Captures 'from X import Y' or 'from.X import Y'.
(import_from_statement
  module_name: (_) @module.name
  name: (_) @imported.entity)

These queries form the basis of the module-level dependency graph, which is essential for calculating file-level fan-in and fan-out.
Method Calls and Attribute Access: In Python's AST, a method call is a call node whose function is an attribute node.
Code snippet
; Captures any call where the function is an attribute access (e.g., obj.method()).
(call
  function: (attribute
    object: (_) @object.name
    attribute: (identifier) @method.name)
  arguments: (_) @arguments)

This is the primary pattern for detecting method invocations.17 A separate query can be used to capture attribute accesses that are not immediately called, representing data coupling.
Class Inheritance: Superclasses are listed in the argument_list of a class_definition.
Code snippet
; Captures the superclasses of a class definition.
; e.g., in 'class MyChild(Parent1, Parent2):', captures 'Parent1' and 'Parent2'.
(class_definition
  name: (identifier) @class.name
  superclasses: (argument_list
    (_) @superclass.name
  )
)

This query directly captures inheritance coupling, which is one of the tightest forms of coupling.8
Decorator Usage: Decorators create a strong dependency, as they wrap and can fundamentally alter the behavior of the decorated function or class.
Code snippet
; Captures decorators applied to a function or class.
; Handles both simple decorators (@my_decorator) and decorators with arguments (@my_decorator(arg)).
(decorated_definition
  decorator: (call
    function: (identifier) @decorator.name)
)
(decorated_definition
  decorator: (identifier) @decorator.name
)

This pattern captures the name of the decorator function, establishing a dependency from the decorated entity to the decorator.19
Handling Dynamicism (A Heuristic Approach): Python's dynamic features, like getattr and __import__, pose a significant challenge for static analysis.
Code snippet
; Detects potential dynamic attribute access via getattr().
(call
  function: (identifier) @func
  (#eq? @func "getattr")
  arguments: (argument_list
    (identifier) @object.name
    (string) @attribute.name.str
  )
)

Code snippet
; Detects potential dynamic imports via __import__().
(call
  function: (identifier) @func
  (#eq? @func "__import__")
  arguments: (argument_list (string) @module.name.str)
)

It is generally impossible for a purely static analyzer to determine the exact dependency created by these calls. The recommended strategy is to use these queries to flag such dynamic usages. The UV-100 framework can then report these as "unresolved" or "dynamic" dependencies, alerting the user to a potential source of coupling that cannot be fully traced and may require manual inspection.

The Challenge of Ambiguity and the Need for a Symbol Table

The queries above can successfully extract the syntactic patterns of coupling. However, for metrics like CBO, which measures coupling between objects, this is insufficient. The core challenge in Python is ambiguity resolution.
Consider the query for method calls. It can identify that my_object.do_something() is a method call on an object named my_object. However, it cannot, by itself, determine the type of my_object. If two different variables, a and b, are instances of ClassA and ClassB respectively, the calls a.do_something() and b.do_something() will produce syntactically similar matches in Tree-sitter. A purely syntactic analysis would not know whether to increment the CBO count for ClassA or ClassB.
To resolve this, the UV-100 Python analyzer must implement a crucial post-processing step: symbol table construction. After the Tree-sitter queries have been executed, the analyzer must perform a scope-aware traversal of the AST to build a symbol table that maps variable names to their inferred types. This process would involve:
Tracking assignments (e.g., my_object = ClassA()) to associate a variable name with a class type within a given scope.
When a method call my_object.do_something() is encountered, the analyzer would look up my_object in the symbol table for the current scope.
If the type is found to be ClassA, the dependency can be resolved, and the CBO for the calling class can be correctly updated to reflect a coupling to ClassA.
This architectural requirement significantly increases the complexity of the Python analyzer compared to the Rust implementation. It moves the analyzer from a purely syntactic tool to one that performs rudimentary semantic analysis. The implementation will need to handle variable shadowing, scope resolution, and basic type inference. Without this additional layer, the CBO and RFC metrics for Python will be inherently imprecise and of limited value.

Section 1.4: Advanced Dependency Queries for JavaScript/TypeScript

The JavaScript ecosystem presents a unique blend of challenges for coupling analysis, stemming from its weak typing, multiple coexisting module systems, and dual inheritance models (class-based and prototype-based). TypeScript introduces a powerful advantage by adding a static type system, which can be leveraged for much more precise analysis. A robust analyzer for this ecosystem must therefore be a hybrid system, capable of handling the dynamism of JavaScript while fully exploiting the static information available in TypeScript.

Mapping the JS/TS AST for Coupling Analysis

The tree-sitter-javascript grammar is typically used to parse JavaScript, JSX, and TypeScript, as it is designed to handle the superset of features.21 A thorough examination of this grammar is necessary to identify the nodes critical for dependency analysis.22
Module Systems:
ES Modules (ESM): import_statement, export_statement.
CommonJS (CJS): call_expression with a function identifier named require.
Inheritance and Object Structure:
Classes: class_declaration, class_heritage (which contains extends and implements clauses).
Prototypes: assignment_expression where the left-hand side is a member_expression accessing a .prototype property.
Invocations and Access: call_expression (for function and method calls) and member_expression (for property and method access).
TypeScript-Specific Constructs: interface_declaration, type_alias_declaration, implements_clause, type_annotation.

JavaScript/TypeScript Query Cookbook

The following queries are designed to capture dependencies across the various paradigms present in the JS/TS ecosystem.
Module Dependencies: The analyzer must handle both ESM and CJS.
Code snippet
; Captures ES Module imports. e.g., import { a } from './module';
(import_statement source: (string) @module.path)

Code snippet
; Captures CommonJS imports. e.g., const a = require('./module');
(call_expression
  function: (identifier) @func
  arguments: (arguments (string) @module.path)
  (#eq? @func "require"))

Support for both module systems is non-negotiable for analyzing modern (and legacy) Node.js and frontend projects.24
Class-Based Inheritance (ES6/TypeScript):
Code snippet
; Captures 'class Child extends Parent'.
(class_declaration
  name: (identifier) @child.class
  (class_heritage
    (extends_clause
      (identifier) @parent.class)))

This query targets the extends keyword to directly identify class inheritance, a strong form of coupling.25
Prototype-Based Dependencies: This is a more heuristic pattern to detect the classical JavaScript inheritance mechanism.
Code snippet
; Captures 'MyClass.prototype.myMethod = function()...'
(assignment_expression
  left: (member_expression
    object: (member_expression
      object: (identifier) @class.name
      property: (property_identifier) @prop
      (#eq? @prop "prototype"))
    property: (property_identifier) @method.name))

This pattern identifies when a property (often a method) is being added to a constructor's prototype, which is a core pattern of coupling in prototype-based object-oriented programming.22
Asynchronous and Callback Dependencies: Asynchronous operations are a major source of coupling in JavaScript.
Code snippet
; Heuristic: Captures a function passed as an argument, likely a callback.
(call_expression
  arguments: (arguments
    [
      (arrow_function) @callback
      (function) @callback
    ]
  )
)

This query identifies a common pattern of event-driven or asynchronous coupling, such as in addEventListener('click', () => {... }) or fs.readFile('path', (err, data) => {... }). While it doesn't resolve the dependency to the event source itself without further analysis, it flags the presence of this important coupling type.28
TypeScript Interface Implementation: This captures a key feature of TypeScript that defines explicit contracts between components.
Code snippet
; Captures 'class MyClass implements MyInterface'.
(class_declaration
  name: (identifier) @class.name
  (class_heritage
    (implements_clause
      (type_identifier) @interface.name)))

This dependency is crucial as it represents a commitment by a class to adhere to the structure defined by an interface, creating a strong architectural link.

The TypeScript Advantage: A TypeScript-First Analysis Strategy

Much like Python, a purely syntactic analysis of JavaScript is insufficient for accurate object-level coupling metrics. A call to data.process() could refer to any object with a process method. However, TypeScript provides the exact information needed to resolve this ambiguity directly within the CST.
Consider this TypeScript code:
const myData: DataProcessor = new DataProcessor(); myData.process();
The tree-sitter-javascript grammar will produce a (lexical_declaration) node containing a (variable_declarator) with a (type_annotation) child. The query would be:

Code snippet


(lexical_declaration
  (variable_declarator
    name: (identifier) @variable.name
    (type_annotation (type_identifier) @variable.type)))


This query allows the analyzer to build a highly accurate symbol table that maps @variable.name (myData) to @variable.type (DataProcessor). When the analyzer later encounters a (method_call) on myData, it can confidently attribute the coupling to the DataProcessor class.
This leads to a critical architectural recommendation: the UV-100 JavaScript analyzer must be a TypeScript-first analyzer.
The analyzer should always attempt to parse files as TypeScript. The grammar is designed to handle plain JavaScript as a subset.
It should prioritize leveraging type_annotation nodes to build a precise symbol table.
For files that are pure JavaScript (i.e., contain no type annotations), the analyzer should fall back to the more heuristic-based approach used for Python, where type inference is based on assignments (e.g., const myData = new DataProcessor()).
The results from pure JavaScript files should be flagged as having potentially lower precision than those from TypeScript files.
This hybrid strategy allows the UV-100 framework to provide the highest possible accuracy for TypeScript projects, which are increasingly the standard for large-scale application development, while still offering valuable (if less precise) insights for legacy or pure JavaScript codebases. This is a significant architectural decision that acknowledges the realities of the JavaScript ecosystem.

Part II: From Metrics to Actionable Insights

Extracting a dependency graph is only the first step. To provide value, the UV-100 framework must translate this raw data into meaningful, prioritized, and actionable feedback for developers. This requires establishing evidence-based thresholds for coupling metrics and developing a sophisticated model for classifying the severity of detected issues. This section details the research and recommendations for building this crucial layer of the analysis engine.

Section 2.1: Establishing Evidence-Based Metric Thresholds

The practice of setting thresholds for software metrics is a balance between academic rigor and industry pragmatism. While research provides a strong theoretical basis for why high coupling is problematic, it is often hesitant to prescribe universal values. In contrast, commercial tools must provide concrete, out-of-the-box configurations. A successful implementation for UV-100 must navigate this landscape by providing sensible defaults backed by evidence, while making the system highly configurable.

A Survey of Industry-Standard and Academic Thresholds

A review of existing static analysis tools and academic literature reveals a range of proposed values for coupling metrics.
Static Analysis Tool Thresholds:
Microsoft Visual Studio /.NET: The documentation for their class coupling metric explicitly cites the S2010 academic paper, which suggests an optimal upper-limit value of 9 for CBO (Coupling Between Objects) for a single member.29 This is one of the most direct, evidence-based thresholds found in tool documentation.
SonarQube: SonarQube's approach is less direct. It does not appear to have a built-in metric named "CBO" or "RFC." However, it has a related rule for Java (java:S6539), "Classes should not depend on an excessive number of classes," which serves a similar purpose. The default threshold for this rule is a couplingThreshold of 20.30 This suggests a more pragmatic, experience-based value rather than one derived from a specific academic study. A deprecated PMD rule for "ExcessiveImports" further indicates a focus on the raw count of dependencies as a proxy for high coupling.31
CodeClimate: This tool focuses on a 10-point maintainability assessment that does not include direct coupling metrics like CBO or RFC. Instead, it measures related concepts like method-count (threshold: 20) and argument-count (threshold: 4), which are often correlated with complexity and coupling but are not direct measures of it.32
Checkstyle: The ClassFanOutComplexity check, which is equivalent to CBO or Fan-Out at the class level, does not have a default threshold. It requires the user to configure a max value explicitly, reinforcing the idea that the "correct" value is project-specific.34
Academic Literature Findings:
The foundational work by Chidamber and Kemerer (CK) established CBO and RFC as key object-oriented metrics.35 Subsequent research has consistently validated that high values for these metrics are statistically significant predictors of fault-proneness, increased maintenance effort, and reduced software quality.37
However, the academic community largely refrains from setting universal thresholds. Studies emphasize that ideal values are highly dependent on the context, including the system's domain, size, and the specific programming language used.40
Instead of "magic numbers," academic approaches often involve deriving thresholds empirically for a specific dataset. This can be done using statistical techniques like logistic regression to find values that best predict defects, or by using percentile-based benchmarks (e.g., flagging the top 10% most-coupled classes in a project as high-risk).37

Contextual Adaptation of Thresholds

The clear conclusion from this research is that a one-size-fits-all threshold is inadequate. The UV-100 framework must support context-aware thresholds that can be adapted to the specific characteristics of the project being analyzed.
By Project Size: As a project grows, the number of components naturally increases, and with it, the potential for higher fan-out values. A fan-out of 15 might be alarming in a small utility library but perfectly normal in a large monolithic application orchestrating many services. The documentation for Project Analyzer explicitly notes that SFOUT/file is likely to be higher in large systems.42 Therefore, the UV-100 configuration should allow for different threshold profiles (e.g.,
small, medium, large) based on project size, which can be measured in lines of code (LOC) or the total number of analyzed components.43
By Domain: The acceptable level and nature of coupling can vary significantly across different software domains.
Embedded Systems: These systems often have strict hardware dependencies, leading to necessary External Coupling.45 They may also employ patterns like
Temporal Cohesion, where functionalities are grouped by timing requirements rather than purely logical concerns, which can be an acceptable design choice in that context.46 The tooling and development cycles in embedded systems can also differ significantly from web development, influencing design trade-offs.47
Web Applications: Modern web frameworks often promote patterns like Dependency Injection and Inversion of Control, which are designed to minimize coupling. Therefore, stricter thresholds may be more appropriate for web applications to enforce these architectural principles.45
By Architectural Role: The significance of a dependency is heavily influenced by its position in the architecture.
Fan-in: A high fan-in is often desirable for stable, reusable components like utility classes or foundational framework components that are part of a public API. Many other parts of the system should depend on them.42 However, a high fan-in for a volatile, internal implementation detail is a major architectural smell, indicating that a piece of logic that should be encapsulated is instead entangled throughout the codebase.
Fan-out: A high fan-out consistently indicates a component with many responsibilities and dependencies, making it complex, difficult to test, and hard to reuse.42
UV-100 should allow rules to be configured based on component visibility (e.g., public vs. internal), allowing for stricter fan-out rules and more lenient fan-in rules for public API components.
The divergence between academic caution and industry pragmatism is clear. While academia proves the correlation between high coupling and negative outcomes, it avoids prescribing universal solutions. Tools, needing to be immediately useful, provide configurable defaults as a starting point. The implication for UV-100 is clear: the system must not enforce a single, rigid set of thresholds. Its primary strength will lie in a flexible configuration system that empowers teams to define their own standards based on their specific context. The values found in research should be offered as well-documented, recommended starting points.

Table 2: Comparative Analysis of Coupling Metric Thresholds

This table consolidates the findings to provide an evidence-based starting point for UV-100's default configurations.

Metric
Source
Recommended Threshold
Context/Notes
CBO / Class Fan-Out
Microsoft Docs (citing Shatnawi, 2010) 29
9
Optimal upper limit for a single class member's coupling.
CBO / Class Fan-Out
SonarQube (Java Rule java:S6539) 30
20
Default for "Classes should not depend on an excessive number of classes."
CBO / Class Fan-Out
Academic (General) 37
Percentile-based (e.g., 80th/90th)
Thresholds are context-dependent; often derived from statistical analysis of a specific project corpus.
Fan-in (Procedure/Method)
Project Analyzer Docs 42
>= 2
Indicates code reuse. High fan-in is generally desirable for utility functions.
Fan-out (Procedure/Method)
Academic (General) 48
Low (no specific value)
High fan-out indicates high complexity and too many responsibilities. Should be kept low.
Fan-out (File)
Project Analyzer Docs 42
"Reasonable" / Low
High fan-out indicates strong cross-file coupling. Tends to increase with project size.


Section 2.2: A Multi-Factorial Model for Severity Classification

To provide truly valuable feedback, a static analysis tool must differentiate between a minor transgression and a critical architectural flaw. A simple binary "good/bad" based on a single threshold is insufficient. This section proposes a more nuanced, multi-factorial model for classifying the severity of coupling issues, enabling UV-100 to prioritize findings effectively.

Defining Severity Tiers

A standard practice in software quality and incident management is to use a tiered severity model. This allows teams to focus their attention on the most impactful issues first. We propose a four-tier model for UV-100, with definitions tailored to the impact of coupling.49
Critical: Represents a severe architectural breakdown. The component is a "god object" or a "dependency magnet" that violates fundamental design principles. Changes to this component have a high probability of causing cascading failures across unrelated parts of the system. Its maintenance is exceptionally costly and risky.
High: A significant violation of architectural principles. The component is tightly coupled to many others, making it difficult to test, maintain, and reuse. Its CBO or RFC value is substantially above the configured threshold (e.g., >200% of the limit), indicating a clear and present maintenance burden.
Medium (Warning): The component's coupling is approaching a problematic level. While not an immediate crisis, it represents growing technical debt that should be addressed opportunistically to prevent it from becoming a high-severity issue. This could be triggered by a metric value that is just over the configured threshold.
Low (Info): A minor or stylistic coupling issue. This might include less severe forms of coupling like "stamp coupling" (passing a large data structure when only a few fields are needed) or a dependency that slightly violates a desired architectural layer but has minimal practical impact.53 These issues are worth noting but are not a priority to fix.

A Weighted Composite Score for Coupling Severity

The severity of a coupling issue is not determined by a single metric but by a combination of factors. A simple if CBO > threshold check is a blunt instrument. It would, for example, treat a class that depends on ten stable, standard library classes the same as a class that depends on ten volatile, internal business logic classes. An experienced architect knows the latter is far more dangerous. The UV-100 severity model must encode this expert intuition. We propose a weighted composite score to achieve this.
Weighting by Dependency Type: Different forms of coupling have different impacts on maintainability and fragility. The model should assign higher weights to tighter forms of coupling. A strong theoretical basis for this weighting can be derived from the classic coupling hierarchy 53:
Content/Common Coupling (Highest Weight): While harder to detect with static analysis, any indication of one module modifying the internals of another or relying on mutable global state should be weighted very heavily.
Inheritance Coupling: Subclass coupling creates a very tight, white-box dependency. A change in the parent class can easily break the child. This should have a high weight.
Control Coupling: One module passing a flag to control the logic of another. This should have a moderate-to-high weight.
Stamp Coupling: Passing complex data structures. This has a moderate weight.
Data Coupling (Lowest Weight): Passing simple data via parameters. This is the ideal, loosest form of coupling.
The UV-100 analyzer can implement this by assigning different weights to the dependency edges it creates based on the Tree-sitter query that discovered them (e.g., a dependency from an (implements_clause) gets a higher weight than one from a (call_expression)).
Combining Metrics: A component that is problematic across multiple dimensions is a clearer sign of a problem. A class with both a high CBO and a high RFC is more likely to be a "god object" than a class with only one elevated metric. A simple composite score could be a linear combination:
SeverityScore=∑i=1n​(wtype​×wmetric​×dependencyi​)
Here, the score for a component is the sum over all its outgoing dependencies, where each dependency is weighted by its type (e.g., inheritance vs. call) and the metric it contributes to (e.g., CBO, RFC). More advanced statistical methods, such as using Principal Component Analysis (PCA) to derive weights from a dataset of known "bad" components, could also be explored.54
Contextual Modifiers: The score should be adjusted based on the context of the dependency target.
Stability: Dependencies on stable, well-defined APIs (e.g., the standard library, mature external frameworks) are less risky than dependencies on volatile, rapidly changing internal application modules. The configuration system should allow users to define a list of "stable" packages or modules. Dependencies on these targets would have their contribution to the severity score down-weighted.
Architectural Boundary: Coupling across major architectural boundaries (e.g., from the data access layer directly to the UI layer) is more severe than coupling within the same layer. If the project structure defines these layers (e.g., via directory structure), this can be used as a modifier.

Differentiated Analysis for Production vs. Test Code

Production code and test code serve different purposes and operate under different constraints, which must be reflected in the analysis rules.55
Test Code Characteristics: Test code is written to verify the contract of production code.55 By its very nature, it is tightly coupled to the code it is testing. A unit test for
ClassA will directly instantiate ClassA, call its methods, and access its state. Enforcing the same low-coupling rules as for production code would be counterproductive, leading to a high volume of false-positive issues and "alert fatigue." Test code also often prioritizes readability and clarity over being DRY (Don't Repeat Yourself), leading to patterns that might be flagged as duplication in production code.55
Proposed Strategy: The UV-100 analysis engine must be able to differentiate between production and test code and apply different rule sets accordingly.
Identification: The analyzer will identify test files using configurable glob patterns (e.g., **/tests/, **/*_test.py, **/*.spec.js).
Configuration Profiles: The configuration system will support separate profiles for production and test code.
Relaxed Rules: For the test profile, coupling-related rules should be either disabled entirely or configured with significantly higher thresholds. For example, the CBO of a test class is expected to be high, and this should not trigger a warning.
By implementing this differentiated approach, UV-100 can provide meaningful insights for production architecture without creating unnecessary noise from the expected and necessary coupling found in test suites.

Part III: System Architecture and Performance

Building a powerful, multi-language coupling detector requires a robust and scalable architecture. This section details the design of the core analysis engine, focusing on the data structures, algorithms, and integration patterns necessary to handle large, real-world codebases efficiently and to integrate seamlessly with the existing UV-100 framework.

Section 3.1: Scalable Cross-File Dependency Analysis

Analyzing a project with thousands of files demands an architecture optimized for performance and scalability. A naive approach of re-analyzing all files on every run is not viable for developer tools that require rapid feedback.

Dependency Graph Construction and Representation

The central data structure for cross-file analysis is a project-wide dependency graph.
Data Structure: A directed graph is the natural representation for dependencies, where nodes represent software components (e.g., files, classes, functions) and edges represent a dependency from one component to another. For the typically sparse graphs found in software systems (where a component depends on a small fraction of all other components), an adjacency list is the most memory-efficient and performant representation. Each node in the graph would store a list of its outgoing dependencies (its dependents). The edges themselves should be annotated with metadata, including the dependency type (e.g., Call, Inheritance) and the source code location, as defined in the generalized dependency model.56
Graph Construction Algorithm: The process of building this graph can be broken down into a multi-stage pipeline:
File Discovery: Recursively scan the project directory to identify all relevant source files based on their extensions, respecting any include/exclude patterns from the configuration.
AST Parsing: For each discovered file, parse it using the appropriate Tree-sitter language grammar to produce a Concrete Syntax Tree (CST). This step is computationally intensive and a primary candidate for optimization.
Dependency Extraction: Run the language-specific set of Tree-sitter queries (from Part I) against the CST for each file. This yields a set of "raw" dependencies, which are essentially captured nodes.
Dependency Resolution and Graph Population: For each raw dependency, resolve the target's fully qualified name. This may involve consulting the file's import statements (use, import, etc.) to map a local name to a full module path. Once resolved, add a directed edge from the source component to the target component in the global dependency graph. The Scripted editor's dependency analysis engine follows a similar pattern of detection, resolution, and transitive traversal.57

Algorithms for Transitive Dependency Resolution and Cycle Detection

Once the direct dependency graph is built, more advanced analyses can be performed.
Topological Sort: A topological sort of the dependency graph is a linear ordering of its nodes such that for every directed edge from node A to node B, A comes before B in the ordering. This algorithm is fundamental for several reasons:
It can be used to determine the correct build order for components.
It can identify the architectural layers within a system.
It is the basis for detecting circular dependencies.
An efficient algorithm for topological sorting is Kahn's algorithm, which iteratively finds nodes with no incoming edges, adds them to the sorted list, and removes their outgoing edges from the graph.58
Circular Dependency Detection: A critical function of a coupling analyzer is to detect circular dependencies between modules or components, as these are a severe architectural anti-pattern. A cycle in the dependency graph makes the system harder to understand, test, and maintain. Cycles can be detected during the topological sort: if the algorithm terminates before all nodes have been added to the sorted list, a cycle exists.59 Alternatively, a Depth-First Search (DFS) can be used to explicitly find cycles by tracking the set of visited nodes in the current recursion stack.

Incremental and Parallel Analysis Strategies

For a tool like UV-100 to be integrated into a developer's workflow (e.g., in an IDE or a CI/CD pipeline), performance is paramount. A full analysis of a large project can take minutes, which is too slow for real-time feedback.
Caching: A multi-level caching strategy is essential to avoid redundant work:
AST Cache: The parsed CST for each file should be cached on disk. The cache key should be a combination of the file path and a hash of its content. On subsequent runs, if the file has not changed, the CST can be loaded directly from the cache, saving significant parsing time.
Dependency Cache: Similarly, the list of direct dependencies extracted from a file's CST can also be cached. This avoids re-running the Tree-sitter queries for unchanged files.
Incremental Analysis: When a file is modified, a full re-analysis of the entire project is wasteful. The system should support incremental updates.
Tree-sitter's edit function allows for extremely fast re-parsing of a tree after a small change has been made to the source text.15
After re-parsing the changed file and extracting its new set of direct dependencies, the analysis engine can update the global dependency graph. It only needs to re-calculate the coupling metrics (like fan-in and fan-out) for the changed component and the components it directly depends on. This localized update is orders of magnitude faster than a full scan.
Parallelization: The initial, "cold" analysis of a project can be significantly accelerated through parallelization. The file discovery, parsing, and dependency extraction stages are "embarrassingly parallel," as the analysis of one file does not depend on the analysis of another. A map-reduce architecture is a natural fit:
Map Phase: A pool of worker threads can process files in parallel. Each worker takes a file, parses it, extracts its direct dependencies, and emits a list of (source, target) dependency pairs.
Reduce Phase: A single process then aggregates all the dependency pairs from the workers to construct the final, global dependency graph.
This pipeline-based architecture, combining parallel processing for the initial scan with robust caching and incremental updates for subsequent analyses, provides the foundation for a highly scalable and performant coupling analysis engine.62

Section 3.2: Integration with the UV-100 Framework

The new coupling analysis capabilities must be integrated into the existing UV-100 architecture in a clean, maintainable, and forward-looking manner. This requires a flexible configuration system, an extensible plugin architecture for language support, and seamless interfacing with existing framework subsystems.

Designing a Flexible Configuration System

A powerful and user-friendly configuration system is critical for the tool's adoption and effectiveness. It allows teams to tailor the analysis to their specific standards and contexts.
Format: A human-readable and writable format is essential. YAML is recommended over JSON due to its support for comments, which allows for better in-file documentation of rules, and its cleaner syntax for nested structures. The configuration file, named .uv100.yml, should reside at the root of the user's project for discoverability and version control.
Structure and Features: The configuration schema must be designed for flexibility and power.63 It should support:
Global Defaults: A top-level section for defining baseline thresholds and settings that apply to the entire project.
Per-Language Overrides: A languages section where users can specify different thresholds or rules for Rust, Python, and JavaScript. This is crucial for accommodating the different characteristics of each language.
YAML
#.uv100.yml
coupling:
  thresholds:
    cbo: 15
    rfc: 50
  languages:
    python:
      thresholds:
        cbo: 20 # Higher default for Python


Path-Based Configuration: The ability to include and exclude files or directories using glob patterns. This is essential for ignoring test directories, third-party vendored code, or generated files.65 It can also be used to apply different rule sets to different parts of the application (e.g., stricter rules for
src/core/ than for src/legacy/).
Rule Customization: Advanced configuration for the severity model, allowing users to define weights for different dependency types (e.g., giving inheritance a higher severity weight than function_call) and to specify a list of "stable" modules (like standard libraries) whose dependencies should not contribute as heavily to coupling scores.
Validation: The UV-100 framework must perform validation of the .uv100.yml file upon loading. It should provide clear, actionable error messages for any malformed syntax, unknown configuration keys, or invalid value types. This prevents user confusion and ensures the analysis runs with the intended configuration.

A Plugin-Based Analyzer Architecture

To ensure the long-term maintainability and extensibility of the coupling detector, a plugin-based architecture is strongly recommended. This pattern is common in extensible developer tools like IDEs and other static analyzers.66
The LanguageAnalyzer Interface: A central LanguageAnalyzer trait (or interface) will define the contract that all language-specific analyzers must adhere to. This decouples the core dependency graph and metric calculation logic from the language-specific parsing details.
Rust
// A simplified representation of the plugin interface
trait LanguageAnalyzer {
    /// Returns the set of Tree-sitter queries needed to find all relevant
    /// dependency types for this language.
    fn get_dependency_queries(&self) -> Vec<String>;

    /// Takes the raw matches from the Tree-sitter queries and performs
    /// language-specific resolution to produce concrete dependency edges.
    /// This is where logic like symbol table lookups for Python/JS would reside.
    fn resolve_dependencies(&self, matches: Vec<QueryMatch>, ast: &Tree, symbol_table: &SymbolTable) -> Vec<DependencyEdge>;
}


Detector Registry Integration: The main coupling detector will not have hardcoded knowledge of Rust, Python, or JavaScript. Instead, it will use the existing UV-100 detector registry (or a similar service locator pattern) to discover and load all available implementations of the LanguageAnalyzer trait at runtime. When analyzing a given source file, it will query the registry for an analyzer that handles that file's language (determined by its extension).
Benefits: This design provides immense long-term benefits. Adding support for a new language, such as Go, becomes a matter of implementing a new, self-contained plugin that conforms to the LanguageAnalyzer interface. No modifications would be needed in the core coupling analysis engine. This modularity is a hallmark of a well-architected, extensible system.

Interfacing with Existing UV-100 Subsystems

The new analysis engine must be a good citizen within the larger UV-100 ecosystem.
AST Caching: The engine must integrate with the existing AST caching system. Before parsing a file, it should request the AST from the cache. If a cached version is available (and the file content is unchanged), it should be used. If not, the engine will parse the file and then provide the new AST to the cache for future use.
Error Handling: Errors originating from the language plugins (e.g., a parsing error in a malformed file, an invalid query) must be handled gracefully. The LanguageAnalyzer interface methods should return Result types, allowing structured errors to be propagated up to the main UV-100 error handling and reporting system. This ensures that users receive clear and consistent error messages, a point of friction noted in a discussion about a SonarQube plugin that failed silently.70
Reporting and Visualization: The final output of the coupling analysis—the dependency graph and the list of identified coupling issues with their severities—should be passed to the existing reporting and visualization pipelines in a standardized format. This allows the UI/reporting layer to render the results without needing any knowledge of the underlying language-specific analysis that produced them.
The long-term value of the UV-100 coupling detector will be defined by its architectural quality, particularly its extensibility. While the immediate goal is to support three languages, the most critical engineering effort is to design the LanguageAnalyzer plugin interface and the configuration system correctly. A well-designed abstraction will dramatically lower the cost of adding new languages and features in the future, ensuring the tool remains relevant and powerful as programming languages and development practices evolve.

Part IV: Strategic Recommendations and Implementation Roadmap

This final part of the report translates the preceding technical blueprint into a high-level strategic plan. It provides a recommended implementation roadmap, summarizes the key architectural decisions and their trade-offs, and offers recommendations for future-proofing the UV-100 coupling analysis framework.

Section 4.1: Phased Implementation Strategy

To manage complexity, mitigate risk, and deliver value incrementally, a phased implementation approach is strongly recommended. This allows the team to build and validate the core infrastructure before tackling the more complex language-specific challenges.
Phase 1: Core Framework and Rust Analyzer.
Objectives: Establish a solid, working foundation for the entire system.
Key Tasks:
Implement the core data structures: the dependency graph (using an adjacency list) and the generalized dependency model.
Implement the core metric calculators (CBO, RFC, Fan-in/Fan-out) that operate on the dependency graph.
Develop the Rust LanguageAnalyzer plugin using the queries from Section 1.2. Rust is the ideal first target because its explicit grammar allows for highly precise queries, providing a reliable "ground truth" to validate the core metric calculations against.
Implement the initial threshold-based issue generator and the flexible YAML configuration system with support for global and per-language thresholds.
Outcome: A fully functional coupling detector for Rust projects. This provides the first deliverable of value and validates the core architecture.
Phase 2: Python Analyzer and Symbol Table.
Objectives: Extend support to Python, tackling the challenges of a dynamic language.
Key Tasks:
Develop the Python LanguageAnalyzer plugin using the queries from Section 1.3.
Design and implement the supporting symbol table and basic type inference engine. This is the most complex part of this phase and is critical for achieving acceptable accuracy for CBO and RFC metrics in Python.
Integrate the symbol table lookup into the resolve_dependencies method of the Python analyzer.
Outcome: A coupling detector for Python with a reasonable degree of accuracy, capable of handling more than just syntactic patterns.
Phase 3: JavaScript/TypeScript Analyzer.
Objectives: Add support for the JavaScript/TypeScript ecosystem.
Key Tasks:
Develop the JS/TS LanguageAnalyzer plugin using the queries from Section 1.4.
Implement the "TypeScript-first" strategy, prioritizing the use of TypeScript's static type annotations to populate the symbol table for maximum accuracy.
Implement fallback heuristics for pure JavaScript files.
Ensure the analyzer correctly handles both ES Module and CommonJS syntax within the same project.
Outcome: A powerful coupling detector for modern web development, with the highest accuracy for TypeScript projects.
Phase 4: Performance and Scalability Enhancements.
Objectives: Ensure the tool is performant enough for large-scale projects and real-time IDE integration.
Key Tasks:
Implement the multi-level caching system for CSTs and extracted dependencies.
Implement the full incremental analysis engine that leverages Tree-sitter's edit functionality to only re-analyze changed files.
Implement the parallel analysis engine (map-reduce style) for fast "cold" scans of large projects.
Outcome: A highly performant and scalable coupling analysis framework suitable for both CI/CD and interactive developer workflows.

Section 4.2: Key Architectural Decisions and Trade-offs

The design presented in this report involves several critical architectural decisions. Understanding their trade-offs is essential for successful implementation.
Accuracy vs. Performance in Dynamic Languages: The decision to supplement syntactic analysis with a symbol table for Python and a type-aware analyzer for TypeScript is a significant one.
Trade-off: This approach increases implementation complexity and analysis time compared to a purely syntactic approach.
Justification: The alternative—a purely syntactic analysis—would render metrics like CBO fundamentally unreliable for these languages, as it cannot distinguish between method calls on different object types. This would lead to a high rate of false positives and negatives, eroding user trust.
Recommendation: Prioritize accuracy. A slightly slower tool that produces trustworthy results is far more valuable than a faster tool that produces noise.
Build-time vs. Real-time Analysis: The architecture must support two primary use cases.
Trade-off: A full, parallel analysis (Phase 4) is optimized for throughput and is ideal for a comprehensive scan in a CI/CD pipeline. A fully incremental, cached analysis is optimized for latency and is required for real-time feedback in an IDE.
Justification: Both use cases are critical for a modern developer tool.
Recommendation: The architecture should be designed from the start to support both modes. The core graph and metric logic is the same; only the triggering and data-gathering mechanisms differ.
Generality of the Dependency Model: The proposed abstract dependency model is intentionally simple (Source, Target, Type, Location).
Trade-off: A more complex model could capture more nuance (e.g., read vs. write dependencies, dependencies on specific function signatures).
Justification: A simpler model is easier to implement and is sufficient for the initial set of CBO/RFC/Fan-in/Fan-out metrics.
Recommendation: Start with the simple model. It can be extended in the future if more advanced metrics or analyses require additional information. The plugin architecture ensures that such an extension would be manageable.

Section 4.3: Future-Proofing the Coupling Analysis Framework

To ensure the UV-100 framework remains a leader in code quality analysis, the architecture should be designed with future extensions in mind.
Support for New Languages: The plugin architecture is the primary mechanism for future-proofing. It establishes a clear, low-friction path for adding support for new languages as they gain popularity.
Semantic and Logical Coupling Analysis: The current framework focuses on structural coupling (direct dependencies visible in the code). Future versions could be extended to detect more subtle, but equally important, forms of coupling.
Semantic Coupling: This refers to components that are conceptually related, even if they don't have a direct structural dependency. This can be detected by analyzing identifiers and comments using techniques like Latent Semantic Indexing (LSI) to find modules that "talk about" the same concepts.53
Logical/Change Coupling: This is a powerful technique that analyzes version control history (e.g., Git logs) to find files that are frequently modified in the same commit. If A.py and B.py are changed together 90% of the time, they are logically coupled, even if there is no direct import or call between them.53 This often reveals hidden dependencies or architectural flaws. Tools like CodeScene have pioneered this type of analysis.71
AI-Driven Insights: The dependency graph and the associated metric data constitute a rich dataset representing a project's architecture.
Future Opportunity: This dataset is an ideal input for machine learning models. Future research could focus on training models to identify complex architectural anti-patterns that go beyond simple metric thresholds. For example, a graph neural network could be trained to recognize the "shape" of a well-architected system versus one with significant technical debt, providing more sophisticated and context-aware insights than are possible with purely heuristic-based rules.
Works cited
tree-sitter/tree-sitter: An incremental parsing system for programming tools - GitHub, accessed July 13, 2025, https://github.com/tree-sitter/tree-sitter
Tree-sitter: Introduction, accessed July 13, 2025, https://tree-sitter.github.io/
Core Concepts in ast-grep's Pattern, accessed July 13, 2025, https://ast-grep.github.io/advanced/core-concepts.html
Modern Tree-sitter, part 3: syntax highlighting via queries | - Pulsar-Edit, accessed July 13, 2025, https://pulsar-edit.dev/blog/20231013-savetheclocktower-modern-tree-sitter-part-3.html
Unraveling Tree-Sitter Queries: Your Guide to Code Analysis Magic - DEV Community, accessed July 13, 2025, https://dev.to/shrsv/unraveling-tree-sitter-queries-your-guide-to-code-analysis-magic-41il
Knee Deep in tree-sitter Queries - Hackerman's Hacking Tutorials, accessed July 13, 2025, https://parsiya.net/blog/knee-deep-tree-sitter-queries/
Basic Syntax - Tree-sitter, accessed July 13, 2025, https://tree-sitter.github.io/tree-sitter/using-parsers/queries/1-syntax.html
Query — py-tree-sitter 0.24.0 documentation, accessed July 13, 2025, https://tree-sitter.github.io/py-tree-sitter/classes/tree_sitter.Query.html
accessed December 31, 1969, https://github.com/tree-sitter/tree-sitter-rust/blob/master/grammar.js
tree-sitter-rust/examples/ast.rs at master - GitHub, accessed July 13, 2025, https://github.com/tree-sitter/tree-sitter-rust/blob/master/examples/ast.rs
Validating References with Lifetimes - The Rust Programming Language, accessed July 13, 2025, https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html
similarity-generic — Rust application // Lib.rs, accessed July 13, 2025, https://lib.rs/crates/similarity-generic
Python grammar for tree-sitter - GitHub, accessed July 13, 2025, https://github.com/tree-sitter/tree-sitter-python
tree-sitter-python/src/grammar.json at master - GitHub, accessed July 13, 2025, https://github.com/tree-sitter/tree-sitter-python/blob/master/src/grammar.json
tree-sitter/py-tree-sitter: Python bindings to the Tree-sitter ... - GitHub, accessed July 13, 2025, https://github.com/tree-sitter/py-tree-sitter
py-tree-sitter/examples/usage.py at master - GitHub, accessed July 13, 2025, https://github.com/tree-sitter/py-tree-sitter/blob/master/examples/usage.py
Building Call Graphs for Code Exploration Using Tree-Sitter - DZone, accessed July 13, 2025, https://dzone.com/articles/call-graphs-code-exploration-tree-sitter
py-tree-sitter 0.24.0 documentation, accessed July 13, 2025, https://tree-sitter.github.io/py-tree-sitter/
Refactoring Python with Tree-sitter and Jedi | Hacker News, accessed July 13, 2025, https://news.ycombinator.com/item?id=41637286
Tree-sitter Starter Guide, accessed July 13, 2025, https://archive.casouri.cc/note/2023/tree-sitter-starter-guide/
tree-sitter-javascript - NPM, accessed July 13, 2025, https://www.npmjs.com/package/tree-sitter-javascript
Javascript grammar for tree-sitter - GitHub, accessed July 13, 2025, https://github.com/tree-sitter/tree-sitter-javascript
accessed December 31, 1969, https://github.com/tree-sitter/tree-sitter-javascript/blob/master/grammar.js
Node Tree-sitter - NPM, accessed July 13, 2025, https://www.npmjs.com/package/tree-sitter
tree_sitter_javascript - Rust - Docs.rs, accessed July 13, 2025, https://docs.rs/tree-sitter-javascript
Learn JavaScript INHERITANCE in 7 minutes! - YouTube, accessed July 13, 2025, https://www.youtube.com/watch?v=DqUPa0D2N78
Inheritance and the prototype chain - JavaScript - MDN Web Docs, accessed July 13, 2025, https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Inheritance_and_the_prototype_chain
tree-sitter/highlight/README.md at master - GitHub, accessed July 13, 2025, https://github.com/tree-sitter/tree-sitter/blob/master/highlight/README.md
Code metrics - Class coupling - Visual Studio (Windows) | Microsoft Learn, accessed July 13, 2025, https://learn.microsoft.com/en-us/visualstudio/code-quality/code-metrics-class-coupling?view=vs-2022
Java rule: Classes should not depend on an excessive number of ..., accessed July 13, 2025, https://next.sonarqube.com/sonarqube/coding_rules?open=java%3AS6539&rule_key=java%3AS6539
Coupling - excessive imports - Rules - SonarQube Community Build - SGS, accessed July 13, 2025, https://cloud-ci.sgs.com/sonar/coding_rules?open=pmd%3AExcessiveImports&rule_key=pmd%3AExcessiveImports
Default Analysis Configuration - Code Climate, accessed July 13, 2025, https://docs.codeclimate.com/docs/default-analysis-configuration
Maintainability - Code Climate, accessed July 13, 2025, https://docs.codeclimate.com/docs/maintainability
ClassFanOutComplexity - Checkstyle, accessed July 13, 2025, https://checkstyle.sourceforge.io/checks/metrics/classfanoutcomplexity.html
Predicting Maintainability of Object-oriented Software Using Metric Threshold - Science Alert, accessed July 13, 2025, https://scialert.net/fulltext/?doi=itj.2014.1540.1547
A METRICS SUITE FOR OBJECT ORIENTED DESIGN Shyam R. Chidamber Chris F. Kemerer M.I.T. Sloan School of Management E53-315 30 Wads - ESO.org, accessed July 13, 2025, https://www.eso.org/~tcsmgr/oowg-forum/TechMeetings/Articles/OOMetrics.pdf
(a) The distribution of the predicted bugs using RFC-CBO. (b) The ..., accessed July 13, 2025, https://www.researchgate.net/figure/a-The-distribution-of-the-predicted-bugs-using-RFC-CBO-b-The-distribution-of-the_fig3_220071232
Vovel metrics—novel coupling metrics for improved software fault prediction - PMC, accessed July 13, 2025, https://pmc.ncbi.nlm.nih.gov/articles/PMC8205299/
Software Coupling and Cohesion Model for Measuring the Quality of Software Components, accessed July 13, 2025, https://www.researchgate.net/publication/376860110_Software_Coupling_and_Cohesion_Model_for_Measuring_the_Quality_of_Software_Components
Techniques for Calculating Software Product Metrics Threshold Values: A Systematic Mapping Study - MDPI, accessed July 13, 2025, https://www.mdpi.com/2076-3417/11/23/11377
(PDF) Evaluating Thresholds for Object-Oriented Software Metrics - ResearchGate, accessed July 13, 2025, https://www.researchgate.net/publication/384568253_Evaluating_Thresholds_for_Object-Oriented_Software_Metrics
Structural Fan-In and Fan-Out metrics - Math-Unipd, accessed July 13, 2025, https://www.math.unipd.it/~tullio/IS-1/2004/Approfondimenti/Fan-in_Fan-out.html
Mastering Size Metrics in Software - Number Analytics, accessed July 13, 2025, https://www.numberanalytics.com/blog/ultimate-guide-size-metrics-software
Project Size Estimation Techniques - Software Engineering - GeeksforGeeks, accessed July 13, 2025, https://www.geeksforgeeks.org/software-engineering/software-engineering-project-size-estimation-techniques/
Mastering Coupling in Software Metrics - Number Analytics, accessed July 13, 2025, https://www.numberanalytics.com/blog/mastering-coupling-in-software-metrics
Coupling and Cohesion - Software Engineering - GeeksforGeeks, accessed July 13, 2025, https://www.geeksforgeeks.org/software-engineering/software-engineering-coupling-and-cohesion/
Is embedded much harder than your typical frontend/backend web development? I want to join this field but am anxious about stress and work life balance. - Reddit, accessed July 13, 2025, https://www.reddit.com/r/embedded/comments/13mbsd6/is_embedded_much_harder_than_your_typical/
Using Modularity Metrics to assist Move Method Refactoring of Large Systems - arXiv, accessed July 13, 2025, https://arxiv.org/pdf/1308.4011
Severity vs Priority: Bug Prioritization in Software Testing | BairesDev, accessed July 13, 2025, https://www.bairesdev.com/blog/severity-vs-priority/
Understanding incident severity levels | Atlassian, accessed July 13, 2025, https://www.atlassian.com/incident-management/kpis/severity-levels
Understanding Severity Levels in Software Development: A Comprehensive Guide, accessed July 13, 2025, https://teamhub.com/blog/understanding-severity-levels-in-software-development-a-comprehensive-guide/
How to Evaluate a Finding Severity | Cyfrin CodeHawks, accessed July 13, 2025, https://docs.codehawks.com/hawks-auditors/how-to-evaluate-a-finding-severity
Coupling (computer programming) - Wikipedia, accessed July 13, 2025, https://en.wikipedia.org/wiki/Coupling_(computer_programming)
A Weighted Composite Metric for Evaluating User Experience in Educational Chatbots: Balancing Usability, Engagement, and Effectiveness - MDPI, accessed July 13, 2025, https://www.mdpi.com/1999-5903/17/2/64
What are the differences between production code and test code? - DEV Community, accessed July 13, 2025, https://dev.to/sandordargo/what-are-the-differences-between-production-code-and-test-code-15pb
Understanding Software Dependency Graphs | Blog - VulnCheck, accessed July 13, 2025, https://vulncheck.com/blog/understanding-software-dependency-graphs
Dependency analysis in Scripted - Spring, accessed July 13, 2025, https://spring.io/blog/2012/11/20/dependency-analysis-in-scripted/
An example dependency resolution algorithm in Python - Breaking Code - WordPress.com, accessed July 13, 2025, https://breakingcode.wordpress.com/2013/03/11/an-example-dependency-resolution-algorithm-in-python/
Dependency graph resolution algorithm in Go - Marin Atanasov Nikolov, accessed July 13, 2025, https://dnaeon.github.io/dependency-graph-resolution-algorithm-in-go/
tree-sitter - crates.io: Rust Package Registry, accessed July 13, 2025, https://crates.io/crates/tree-sitter
tree-sitter - PyPI, accessed July 13, 2025, https://pypi.org/project/tree-sitter/
Why Code Dependencies Matters for Static Analysis (SAST) | Black Duck Blog, accessed July 13, 2025, https://www.blackduck.com/blog/code-dependencies-static-analysis-sast.html
Static Code Analysis Best Practices - Number Analytics, accessed July 13, 2025, https://www.numberanalytics.com/blog/static-code-analysis-best-practices-systems-engineering
Customizing Static Code Analysis Rules to Improve Code Quality - IN-COM Data Systems, accessed July 13, 2025, https://www.in-com.com/blog/customizing-static-code-analysis-rules-to-improve-code-quality/
Configuring Your Analysis - Code Climate, accessed July 13, 2025, https://docs.codeclimate.com/docs/configuring-your-analysis
The Ultimate Guide to Extensible Software Design - Number Analytics, accessed July 13, 2025, https://www.numberanalytics.com/blog/ultimate-guide-extensible-software-design
10 Best Static Code Analysis Tools | Clutch.co, accessed July 13, 2025, https://clutch.co/resources/10-best-static-code-analysis-tools
Building a plugin architecture with Managed Extensibility Framework - Part 3, accessed July 13, 2025, https://www.elementsofcomputerscience.com/posts/building-plugin-architecture-with-mef-03/
I am looking for resources on plugin architectures, can you help? : r/programming - Reddit, accessed July 13, 2025, https://www.reddit.com/r/programming/comments/b4dhz/i_am_looking_for_resources_on_plugin/
Plugin for a static analysis tool - Plugin Development - Sonar ..., accessed July 13, 2025, https://community.sonarsource.com/t/plugin-for-a-static-analysis-tool/133383
Change Coupling: Visualize Logical Dependencies — CodeScene 6.9.10 Documentation, accessed July 13, 2025, https://docs.enterprise.codescene.io/versions/6.9.10/guides/technical/change-coupling.html
