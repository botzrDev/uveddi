
Deterministic Detection of Complex Architectural Anti-Patterns: A Framework for Advanced Static Analysis


Introduction

The evolution of software systems is invariably accompanied by the degradation of their architectural integrity. This decay, often manifesting as architectural anti-patterns, introduces significant technical debt, impedes maintainability, and increases the propensity for defects.1 While static analysis tools have long sought to identify these issues, they have historically relied on simplistic heuristics and metrics that suffer from high false-positive rates and an inability to detect nuanced, structural flaws. The emergence of Large Language Models (LLMs) offers a new paradigm for code understanding, yet their non-deterministic nature and computational cost present challenges for integration into high-precision, automated development workflows.3
This report presents a comprehensive framework for the advanced, deterministic detection of complex architectural anti-patterns, designed to inform the development of next-generation static analysis tooling. The central thesis is that high-precision, language-agnostic, and architecturally-aware detection can be achieved through a synthesis of three core components: (1) the formalization of anti-patterns into a machine-readable specification, (2) the representation of source code in a language-agnostic intermediate model and a corresponding architectural graph, and (3) the application of sophisticated static analysis and graph algorithms to these representations.
The research detailed herein provides a complete blueprint for moving beyond basic heuristics. It delivers formal specifications for several complex anti-patterns, including Leaky Abstraction, Insufficient Access Control, and a novel, static approach to Modularity Violation detection that does not rely on version history. It further provides concrete algorithmic implementations, a robust validation framework for benchmarking precision and recall, a multi-factor model for quantifying detection confidence, and a calibrated strategy for integrating these deterministic methods with optional LLM-based verification. This work is intended to serve as a foundational technical document for engineering teams building sophisticated code analysis platforms, with specific implementation guidelines tailored for a Rust-based architecture.

Part I: Foundational Models for Architectural Representation

This initial part of the report establishes the essential theoretical and structural groundwork for advanced, deterministic anti-pattern detection. It addresses the fundamental challenges of defining architectural flaws in a machine-parsable manner and representing heterogeneous source code in a unified model suitable for complex analysis.

Section 1: Formalizing Architectural Anti-Patterns for Machine Analysis

The efficacy of any automated detection tool is fundamentally constrained by the precision of its target definitions. This section outlines a methodology for transforming ambiguous, human-centric descriptions of anti-patterns into formal, machine-readable specifications that can serve as the basis for high-precision deterministic algorithms.

1.1 The Problem with Ambiguity: Moving Beyond Heuristics

The discourse surrounding architectural anti-patterns is replete with qualitative, ambiguous descriptions. Terms such as "overly complex designs," "tight coupling," or "inadequate separation of concerns" are common but lack the formal rigor required for automated detection.5 An anti-pattern is often defined as a "bad solution to a problem," but without a formal definition of what constitutes "bad," detection tools are forced to rely on simplistic heuristics (e.g., raw line-of-code counts) or surface-level metrics.7 This ambiguity is a primary source of the high false-positive rates that plague many existing static analysis tools, eroding user trust and leading to alert fatigue.9
To achieve the goal of high-precision, deterministic detection, it is imperative to move beyond these informal descriptions. The process of formalization, analogous to its use in verifying the consistency of design pattern applications, allows for the creation of rigorous, precise, and unambiguous descriptions that are directly translatable into machine logic.10 By defining anti-patterns in terms of verifiable structural properties and relationships, we can construct detectors that operate with predictable accuracy and provide clear, justifiable results.

1.2 A Theoretical Basis for Formalization: Design Rule Theory

A robust formalization requires a sound theoretical foundation. Baldwin and Clark's Design Rule Theory provides such a foundation by positing that well-structured software systems are decomposed into independent modules that are decoupled by stable design rules, which typically manifest as interfaces or abstract classes.11 This theory provides a powerful lens through which to view and define architectural anti-patterns as concrete, detectable violations of these fundamental principles.
Under this framework, an anti-pattern is not merely a "bad practice" but a measurable deviation from a sound architectural principle:
An Unstable Interface is a direct violation of the principle that a design rule—an influential interface with many dependents—should remain stable to prevent change propagation.11
A Modularity Violation occurs when modules that are structurally independent according to the design rules exhibit implicit dependencies (e.g., by sharing unstated assumptions), violating the principle of module independence.11
An Unhealthy Inheritance Hierarchy violates the Liskov Substitution Principle and undermines the parent class's role as a decoupling design rule.11
By grounding our definitions in Design Rule Theory, we move from subjective assessments to objective, principle-based analysis. This approach provides a clear rationale for why a given structure is considered an anti-pattern and directly informs the design of algorithms that detect these violations.

1.3 A Domain-Specific Language (DSL) for Anti-Pattern Specification

To make these formal definitions machine-readable and executable, this report proposes the development of a Domain-Specific Language (DSL) tailored for the specification of architectural anti-patterns. A DSL is a language created to solve problems within a particular domain, offering greater expressivity and clarity than a general-purpose language for that specific context.14 In this case, the DSL will allow architects and tool developers to define anti-patterns declaratively, specifying the entities, relationships, properties, and constraints that constitute a violation. This approach transforms architectural rules from informal diagrams into version-controlled, testable artifacts.15
The proposed DSL would feature a formal grammar (e.g., defined in EBNF) and include the following core constructs:
Entities: Primitives representing code elements (Module, Package, Class, Interface, Function, Method, Parameter).
Relationships: Verbs defining the connections between entities (dependsOn, inheritsFrom, implements, calls, accesses, contains).
Properties: Attributes of entities that can be queried (isPublic, isAbstract, isStatic, annotation, LOC > n, fanIn > n, fanOut > n, cyclomaticComplexity > n).
Constraints: Rules that must be satisfied or violated (mustNotDependOn, mustOnlyBeAccessedBy, formsCycleWith).
Scope: Modifiers to limit the context of a rule (within(Package), across(Layer)).
Such a DSL provides a clear, unambiguous, and powerful mechanism for defining anti-patterns that can be directly parsed and executed by an analysis engine.

1.4 Formal Specifications of Target Anti-Patterns using the DSL

This section provides formal specifications for three complex anti-patterns using the proposed DSL. These definitions are designed to be directly translatable into the detection logic detailed in Part II.

1.4.1 Leaky Abstraction

A Leaky Abstraction occurs when an implementation detail of a lower-level module is exposed through the public interface of a higher-level module, forcing consumers of the abstraction to be aware of its internal workings.17 This violates the principle of information hiding and creates tight coupling between layers.
Formal DSL Specification:

Code snippet


DEFINE ANTI_PATTERN LeakyAbstraction {
  // Define a set of known low-level implementation namespaces.
  // This list must be configurable and extensible.
  LET ImplementationDetail = Type(source IN);

  // An abstraction is a public component in a layer intended to hide details of lower layers.
  // Here, we define it as any class within a 'service' or 'business' package.
  LET Abstraction = Class(package IN ["..service..", "..business.."]);

  // Iterate over all public methods within the defined abstractions.
  FOR EACH method IN Abstraction.methods WHERE (method.isPublic) {
    // A violation occurs if a public method's signature (return type or parameters)
    // exposes a type from the ImplementationDetail list.
    VIOLATION IF (method.returnType IS ImplementationDetail) OR
                 (ANY(method.parameters.type) IS ImplementationDetail)
    MESSAGE "Method '{method.name}' in abstraction '{Abstraction.name}' leaks implementation detail type '{leaked.type}'.";
  }
}


This specification formalizes the leak as a public method in a designated abstraction layer exposing a type from a known implementation framework. For example, a method in a UserService class that returns a java.sql.ResultSet is a direct violation of this rule.17

1.4.2 Insufficient Access Control (Unprotected Sensitive Operation)

This anti-pattern occurs when a code path allows data from an untrusted, external source to influence a security-sensitive operation without passing through an explicit authorization check.20 This is a common source of vulnerabilities like Insecure Direct Object References (IDOR) and privilege escalation.20
Formal DSL Specification:

Code snippet


DEFINE ANTI_PATTERN InsufficientAccessControl {
  // Define sources of untrusted data, typically from web request bindings.
  LET UntrustedSource = Parameter(annotation IN);

  // Define sinks, which are security-sensitive operations.
  LET SensitiveOperation = MethodCall(target IN);

  // Define sanitizers, which are explicit authorization checks.
  LET AuthorizationCheck = MethodCall(target.annotation IS "@PreAuthorize");

  // Use a data flow query to trace paths from sources to sinks.
  FOR EACH path FROM UntrustedSource TO SensitiveOperation {
    // A violation occurs if a data flow path exists that does not
    // pass through an authorization check.
    VIOLATION IF path.doesNotPassThrough(AuthorizationCheck)
    MESSAGE "Untrusted data from '{UntrustedSource.name}' reaches sensitive operation '{SensitiveOperation.name}' without authorization.";
  }
}


This specification uses a data flow analysis (or "taint tracking") model. Data is "tainted" at the UntrustedSource, and if this tainted data reaches a SensitiveOperation sink without being "sanitized" by an AuthorizationCheck, a violation is flagged.

1.4.3 Modularity Violation (Static Structural Anomaly)

A Modularity Violation traditionally refers to structurally independent components that frequently change together, implying a hidden dependency.1 As historical analysis is precluded by the research prompt, this report proposes a novel, purely static definition based on structural community detection and anomalous coupling. The principle is that components within a well-defined module should be more cohesive with each other than with components outside the module.
Formal DSL Specification:

Code snippet


DEFINE ANTI_PATTERN ModularityViolation {
  // Partition the system's dependency graph into structural communities
  // using an algorithm like Louvain. These represent de-facto modules.
  LET {C1, C2,...} = findCommunities(graph, algorithm: 'Louvain');

  // Iterate over every pair of distinct communities.
  FOR EACH (CommunityA, CommunityB) IN {C1, C2,...} WHERE (CommunityA!= CommunityB) {
    LET ModuleA = Node(in_community: CommunityA);
    LET ModuleB = Node(in_community: CommunityB);

    // Calculate the average internal cohesion (edge weight) for each community.
    LET internalCohesionA = avgEdgeWeight(within: CommunityA);
    LET internalCohesionB = avgEdgeWeight(within: CommunityB);

    // Find all edges that cross between the two communities.
    FOR EACH edge FROM ModuleA TO ModuleB {
      // A violation is an inter-community dependency that is significantly
      // stronger than the internal cohesion of the modules it connects.
      // The threshold (e.g., 0.75) should be configurable.
      VIOLATION IF (edge.weight > 0.75 * internalCohesionA) AND
                   (edge.weight > 0.75 * internalCohesionB)
      MESSAGE "Anomalous coupling of weight {edge.weight} found between module '{ModuleA.name}' in community {CommunityA} and '{ModuleB.name}' in community {CommunityB}.";
    }
  }
}


This specification leverages graph theory to identify structural anomalies. It first uses community detection to infer the system's modular structure and then flags dependencies between these modules that are disproportionately strong, suggesting a violation of modular boundaries.23
The act of creating such formal specifications provides a profound shift in how architecture is managed. It moves architectural principles from passive documents and diagrams into an active, verifiable, and automatable part of the development lifecycle. This opens the door to integrating architectural validation directly into CI/CD pipelines, IDEs, and other development tools, creating a continuous feedback loop that prevents architectural drift before it takes root.

Section 2: Language-Agnostic Code Representation via Intermediate Models

To analyze code from multiple programming languages with a single, consistent set of detectors, it is necessary to transform the source code into a unified, language-agnostic representation. Directly building analyzers for each language's specific Abstract Syntax Tree (AST) is inefficient, difficult to maintain, and leads to inconsistent detection capabilities across languages.25 A unified intermediate representation (IR) allows for the development of a single analysis engine that operates universally, regardless of the source language.26

2.1 Comparative Analysis of Intermediate Representations (IRs)

The choice of IR is a critical architectural decision for any multi-language analysis tool. The ideal IR must abstract away language-specific syntax while preserving the high-level structural and semantic information necessary for architectural analysis. The following table compares several common IRs based on their suitability for this task.
IR Type
Description
Preservation of High-Level Constructs
Language Support Effort
Suitability for Anti-Pattern Detection
Language-Specific AST
A direct tree representation of the source code's syntax for a single language.
High. Preserves all information, including annotations, comments, and exact syntax.
High. Requires a separate analysis engine for each supported language.
High (for one language), Poor (for multi-language). Excellent for deep, language-specific analysis but not scalable for a polyglot tool.
Compiler IR (e.g., LLVM IR, CIL)
A low-level, language-independent representation used by compilers for optimization and code generation.
Low. High-level constructs like classes, methods, and annotations are compiled away, losing semantic context crucial for architectural analysis.25
Low. Compilers for many languages already target these IRs.
Very Low. Unsuitable for detecting architectural anti-patterns, which rely on high-level abstractions, not machine-level instructions.
UML (Unified Modeling Language)
A standardized graphical modeling language for specifying, visualizing, and documenting software systems.
Variable. Can represent high-level components and relationships, but is often disconnected from the source code and lacks constructs for "blocks of code" or implementation details.25
High. Requires a robust reverse-engineering tool to generate accurate models from code, which can be complex and lossy.
Medium. Good for visualizing intended architecture but poor for detecting violations in the actual implementation, as it lacks code-level fidelity.
Generic/Language-Agnostic AST (LAAST)
A unified AST that abstracts common programming constructs (classes, functions, loops, etc.) into a single schema.
High. Designed to retain essential high-level constructs (types, annotations, member visibility) while abstracting away syntactic sugar.25
Medium. Requires a language-specific front-end parser to transform the source into the LAAST, but the core analysis engine is built only once.25
Excellent. Provides the optimal balance of language independence and high-level semantic detail required for robust, deterministic anti-pattern detection.


2.2 Recommended Approach: A Feature-Rich Language-Agnostic Abstract Syntax Tree (LAAST)

Based on the comparative analysis, the Language-Agnostic Abstract Syntax Tree (LAAST) is the recommended intermediate representation. This approach provides the necessary abstraction for cross-language analysis while preserving the high-level semantic information that low-level compiler IRs discard.25
The implementation of a LAAST-based pipeline involves a clear, multi-stage process:
Parsing: For each supported language, a dedicated parser consumes the source code and generates a language-specific Concrete Syntax Tree (CST) or AST. Modern parser-generator toolkits like tree-sitter are well-suited for this task as they support a wide variety of languages.
Transformation: A language-specific transformer module traverses the initial AST/CST and maps its nodes to the corresponding constructs in the unified LAAST schema. For example, a Java class declaration and a Python class definition would both be transformed into a single ClassDeclaration node in the LAAST.
Analysis: The core analysis engine operates exclusively on the LAAST, executing the anti-pattern detection algorithms defined in Part II.
A crucial aspect of this approach is the design of the LAAST schema itself. It must be rich enough to capture not only basic structural elements but also the metadata essential for architectural analysis, including:
Visibility Modifiers: public, private, protected.
Annotations/Decorators: Critical for identifying framework-specific roles (e.g., @RestController, @Entity).
Type Information: Fully qualified type names for variables, parameters, and return values.
Inheritance and Implementation Clauses: Explicit representation of extends and implements relationships.
By investing in a rich LAAST, the analysis platform gains more than just language-agnostic detection. This unified representation of a polyglot codebase becomes a foundational asset. It creates a platform for a wide range of future innovations, such as cross-language code clone detection, the mining of emergent architectural patterns, or even the training of language-agnostic machine learning models for advanced code intelligence.27 This strategic choice transforms the immediate engineering task into a long-term investment in the platform's core capabilities.

Section 3: Graph-Based Architecture Modeling for Structural Analysis

While the LAAST provides a detailed representation of individual code files, understanding system-wide architectural properties requires a higher-level view of the relationships between these files. A graph data structure is the natural and most powerful way to model these dependencies. This section details the construction and implementation of an Architectural Dependency Graph (ADG), the primary data structure for detecting modularity and structural anti-patterns.

3.1 From Code to Graph: Constructing the Architectural Dependency Graph (ADG)

The ADG is a directed, weighted graph constructed by traversing the LAAST and representing its elements and their relationships as nodes and edges.
Nodes: Nodes in the ADG represent architectural components at varying levels of granularity. The analysis can be configured to create nodes for Packages, Modules (source files), Classes, or Functions. This flexibility allows the same detection algorithms to be applied at different architectural scopes.
Edges: Directed edges represent dependencies between nodes. The types of dependencies captured are critical for the accuracy of the analysis and include:
Call: A function or method in node A calls a function or method in node B.
Inheritance: A class in node A extends a class in node B.
Implementation: A class in node A implements an interface in node B.
Instantiation: Code in node A creates an instance of a class from node B.
Access: Code in node A reads or writes a field in node B.
Edge Weighting: A simple unweighted graph is insufficient for nuanced analysis. A weighted graph, where the edge weight quantifies the "strength" of the dependency, is essential. The weighting scheme must be configurable but a sound default can be based on the type and frequency of interactions:
Inheritance and implementation dependencies represent very strong, structural coupling and should receive a high base weight.
Method call dependencies can be weighted by their frequency. Multiple calls between two modules indicate a stronger dependency than a single call.
The type of data being passed can also influence weight. A dependency that passes a complex domain object might be weighted more heavily than one passing a primitive type.
This process transforms the static code representation into a dynamic model of the system's architectural forces, upon which graph algorithms can be applied.

3.2 Data Structures for Efficient Graph Representation in Rust

For the practical implementation of the ADG within a Rust-based tool, the choice of data structure and library is critical for performance and scalability. Software dependency graphs are typically sparse (i.e., the number of edges is much smaller than the number of possible edges), which makes an Adjacency List representation far more memory-efficient than an Adjacency Matrix.
The unequivocal recommendation for a Rust implementation is the petgraph library.29 It is the de-facto standard for graph analysis in the Rust ecosystem and provides all the necessary features for this project:
Flexibility: It offers multiple graph types, including Graph (an adjacency list) and StableGraph (which maintains stable node indices upon removal), allowing the implementation to choose the best trade-off.
Generic Weights: petgraph is generic over node and edge weights, allowing for the use of custom structs to store rich information about each component and dependency.
Rich Algorithm Ecosystem: It includes built-in implementations of many standard graph algorithms required for anti-pattern detection, such as traversals (DFS, BFS), shortest path (Dijkstra), and, critically, algorithms for finding Strongly Connected Components (e.g., tarjan_scc) needed for cycle detection.
The ADG is not a static, one-size-fits-all model. A key to its power lies in its ability to adapt to different architectural styles. For example, in a microservices architecture, a single network call between two services represents a far more significant and costly dependency than an in-process method call within a monolith. This can be modeled by introducing Architectural Style Profiles. These profiles, loaded at the start of an analysis, would adjust the edge weighting algorithm. A "microservice" profile would assign a massively inflated weight to dependencies identified as remote API calls (e.g., through analysis of framework annotations like @FeignClient or HTTP client library usage), while a "monolith" profile would place a higher emphasis on inheritance and tight coupling. This mechanism allows the same underlying graph model and detection algorithms to be tuned to the specific context of the system under analysis, dramatically improving the relevance and accuracy of the findings.

Part II: Deterministic Detection Algorithms and Implementation

This part translates the formal specifications and foundational models from Part I into concrete, deterministic algorithms. Each algorithm is designed for high-precision detection and is described in a manner that facilitates direct implementation within the target analysis engine.

Section 4: AST-Based Detection of Encapsulation and Abstraction Flaws

These anti-patterns are primarily concerned with violations of information hiding and proper layering within the code. They are best detected by traversing the LAAST and analyzing the signatures and contents of classes and methods.

4.1 Detecting Leaky Abstractions

The Leaky Abstraction anti-pattern, as formally defined in Section 1.4.1, can be detected using a targeted AST traversal algorithm. The algorithm identifies when a high-level module's public interface exposes types that are implementation details of a lower-level module.
Algorithm:
Configuration Loading: Begin by loading two configurable lists:
Layer Definitions: A mapping of package patterns to architectural layers (e.g., com.mycompany.api.** -> presentation, com.mycompany.services.** -> business, com.mycompany.persistence.** -> data_access).
Implementation Detail Namespaces: A list of namespaces that are considered low-level implementation details to be hidden (e.g., java.sql.*, org.hibernate.*, boto3.*).17
LAAST Traversal: Traverse the LAAST, visiting every ClassDeclaration node.
Layer Identification: For each class, determine its architectural layer based on its package and the loaded layer definitions. If a class does not belong to a defined layer, it is skipped for this analysis.
Public Interface Inspection: For each class, iterate through its MethodDeclaration nodes that have a public visibility modifier.
Signature Analysis: For each public method, perform the following checks:
Return Type: Resolve the fully qualified type of the method's ReturnType. Check if this type's namespace matches any pattern in the ImplementationDetail list or belongs to a layer lower than the method's own layer.
Parameter Types: Iterate through each Parameter of the method. Resolve its fully qualified type and perform the same check as for the return type.
Violation Flagging: If any type in the public signature is identified as a "leak" (i.e., it is an implementation detail or from a lower layer), a LeakyAbstraction violation is flagged. The report should include the name of the leaking class and method, the specific type that is being leaked, and the layers involved.
The core of this algorithm relies on identifying a specific structural pattern in the LAAST: a MethodDeclaration node with the isPublic property set to true, whose child ReturnType or Parameter nodes have a Type that resolves to a forbidden namespace. The effectiveness of this detector is directly proportional to the comprehensiveness of the configured layer definitions and implementation detail lists.

4.2 Detecting Insufficient Access Control

The Insufficient Access Control anti-pattern, as specified in Section 1.4.2, represents a potential information flow vulnerability. Its detection requires a more sophisticated form of static analysis known as taint analysis or taint tracking, which follows the flow of data from untrusted sources to sensitive sinks.33
Algorithm:
Define Taint Analysis Configuration: Load a framework-specific configuration that defines three key sets of nodes within the LAAST:
Sources: These are points where untrusted external data enters the application. This is typically identified by searching for Parameter nodes with specific annotations from web frameworks (e.g., @RequestBody in Spring, request.form in Flask) or calls to functions that read from I/O streams.20
Sinks: These are security-sensitive operations that should not be influenced by untrusted data without validation. These are identified by MethodCall nodes targeting specific functions (e.g., java.lang.Runtime.exec, javax.persistence.EntityManager.createQuery).33
Sanitizers/Validators: These are nodes that represent an explicit authorization or validation check. A data flow passing through a sanitizer is considered "clean." These can be identified as MethodCall nodes to a security API (e.g., SecurityContextHolder.getContext().getAuthentication()) or methods annotated with security constraints (e.g., @PreAuthorize).20
Construct Data Flow Graph (DFG): Build an intra- and inter-procedural Data Flow Graph. This graph's nodes are program variables and expressions, and its edges represent the flow of data (e.g., assignments, parameter passing, return values). This can be derived from the LAAST and the ADG.
Taint Propagation: Perform a graph traversal starting from every Source node in the DFG.
Mark the data originating at a Source as "tainted."
Propagate this taint status along the DFG edges. If variable a is tainted and the code contains b = a, then b also becomes tainted.
If a tainted flow passes through a Sanitizer node, the taint is removed for that specific path.
Check for Tainted Sinks: During the traversal, if a Sink node is reached by a tainted data flow, it signifies a potential vulnerability.
Flag Violation: When a tainted sink is found, flag an InsufficientAccessControl violation. The report should include the complete data flow path from the source to the sink, highlighting the lack of an intermediate sanitizer. This path information is crucial for developers to understand and remediate the vulnerability.36
The power of these AST-based detectors is not in the traversal algorithms themselves, which are relatively standard, but in the semantic knowledge encoded in their configurations. A successful tool must ship with a rich, extensible set of "Framework Profiles" that pre-define the sources, sinks, sanitizers, and layer boundaries for popular frameworks like Spring, Django, ASP.NET, and others. This provides high-quality detection out-of-the-box while allowing expert users to define custom profiles for their own internal libraries and frameworks, making the analysis both powerful and adaptable.

Section 5: Graph-Based Detection of Modularity and Dependency Flaws

These anti-patterns relate to the macro-structure of the system and are best identified by applying graph theory algorithms to the Architectural Dependency Graph (ADG).

5.1 Detecting Modularity Violations (Static Approach)

Addressing the user's constraint to avoid historical analysis requires a novel approach to detecting Modularity Violations. Traditional methods almost exclusively rely on mining version control history to find components that change together despite lacking a structural dependency.1 The proposed static algorithm reconceptualizes the violation as an
anomalous structural coupling between otherwise distinct modules.
Algorithm:
Construct Weighted ADG: Generate the weighted Architectural Dependency Graph (ADG) as described in Section 3.1, where nodes are modules (e.g., packages or source files) and edge weights represent the strength of coupling.
Apply Community Detection: Execute a community detection algorithm on the ADG to partition it into a set of de-facto modules or "communities." The Louvain Modularity algorithm is a suitable choice due to its efficiency and widespread use in network science.23 The goal of this step is not to find a single, "perfect" architecture but to establish a data-driven baseline of the system's current modular structure based on its coupling patterns.
Analyze Inter-Community Coupling: For every pair of distinct communities, Ci​ and Cj​, identify the set of all edges that connect a node in Ci​ to a node in Cj​.
Identify Anomalous Edges: A Modularity Violation is flagged if the coupling between two components in different communities is significantly stronger than the internal cohesion of those communities. The specific condition is:
For an edge e between node ni​∈Ci​ and nj​∈Cj​, a violation exists if:
$weight(e) > \alpha \cdot \text{avg_internal_weight}(C_i)$ AND $weight(e) > \alpha \cdot \text{avg_internal_weight}(C_j)$
Where avg_internal_weight(C) is the average weight of all edges connecting nodes within community C, and α is a configurable sensitivity threshold (e.g., 0.75).
Flag Violation: Report each anomalous edge as a Modularity Violation. The report should detail the two components involved, their inferred communities, the weight of their anomalous dependency, and the average internal cohesion of their respective communities. This provides developers with clear evidence of a structural tension that needs investigation.
It is important to acknowledge that modularity maximization algorithms are known to have limitations and rarely produce a globally optimal partition.23 However, for this use case, this is an acceptable trade-off. The community detection algorithm is used as a powerful heuristic to find a
plausible modular structure. The anti-pattern is then defined not as a failure to meet some ideal architecture, but as a significant and measurable structural deviation from that plausible, data-driven baseline.

5.2 Detecting Cyclic Dependencies

Cyclic dependencies are a classic and highly detrimental structural anti-pattern where two or more components depend on each other, either directly or indirectly. This creates tight coupling, hinders testability, and makes the system difficult to understand and maintain.1 Their detection is a standard graph analysis problem.
Algorithm:
Construct Unweighted ADG: Generate an unweighted, directed ADG where nodes represent the components of interest (e.g., packages or classes) and edges represent the existence of one or more dependencies.
Find Strongly Connected Components (SCCs): Apply a standard algorithm for finding SCCs to the ADG. Tarjan's algorithm or Kosaraju's algorithm are both efficient and well-established choices for this task.
Flag Violations: An SCC is a subgraph where every node is reachable from every other node in that subgraph. Therefore, any SCC containing more than one node represents a dependency cycle.
Report Cycles: For each SCC with a size greater than one, flag a CyclicDependency violation. The report should list all the components (nodes) that are part of the cycle, allowing developers to visualize the entire circular dependency chain.
This algorithm can be applied at different levels of granularity by changing the node type in the ADG. Running it on a graph of packages will detect package-level cycles, which are often considered severe architectural flaws.11 Running it on a graph of classes or files will detect finer-grained cycles within a module.
The modularity violation detector can be further refined by exposing the "resolution" parameter often found in community detection algorithms. A low resolution setting will tend to find large, coarse-grained communities, corresponding to major architectural layers or subsystems. Violations detected at this level represent significant architectural breakdowns. Conversely, a high resolution setting will produce smaller, more numerous communities. Violations at this level might correspond to finer-grained design smells like Feature Envy or misplaced class responsibilities. By allowing users to select a sensitivity level (e.g., "Architectural" vs. "Fine-Grained"), the tool can be adapted to the needs of different roles, from architects examining the system's macro-structure to developers focusing on local code health.

Section 6: Implementation Guidelines for the Uveddi Rust Architecture

This section provides concrete recommendations and examples for implementing the proposed detection framework within a Rust-based environment, addressing specific questions from the research prompt.

6.1 Recommended Rust Libraries for Parsing and AST Generation

The choice of a parsing library is foundational to the entire analysis pipeline. The library must be performant, robust, and capable of handling the complexities of multiple programming languages. For a tool like Uveddi, which must analyze code that may be incomplete or contain syntax errors during development, error recovery is a paramount concern.
Library
Paradigm
Key Features
Performance
Error Recovery
Ecosystem/Maturity
rustpython-parser 39
Parser Generator
CPython-compatible AST for Python.
Good for Python.
Language-specific.
Mature for Python. Not a general solution.
pest 40
PEG Parser Generator
Expressive grammar definition in separate files. Good error messages.
Good.
Basic. Can report first error well.
Mature and widely used. Good for custom languages.
nom 40
Parser Combinator
Zero-copy, byte-oriented. High performance.
Excellent.
Manual. Requires significant developer effort to implement.
Very mature, foundational crate. Steep learning curve.
chumsky 40
Parser Combinator
Expressive combinators, context-sensitive parsing, excellent performance.
Excellent.
Best-in-class. Designed for flexible, powerful error recovery, can report multiple errors.
Modern and rapidly maturing. Ideal for compilers/analyzers.

Recommendation: For building a high-performance, multi-language static analysis tool, chumsky is the recommended library.40 Its combination of top-tier performance and, most importantly, sophisticated error recovery capabilities makes it uniquely suited for analyzing real-world code in various states of completion. The ability to recover from syntax errors and still produce a partial AST is a critical feature for a tool intended for use within a development workflow.
pest remains a strong alternative if the team prefers a grammar-based definition approach over a combinator-based one.40

6.2 Recommended Rust Library for Graph Analysis

As established in Section 3.2, the petgraph library is the industry standard for graph manipulation and analysis in Rust. It is the definitive recommendation for implementing the ADG.
Recommendation: petgraph.29
Implementation Example (Conceptual):
The following Rust snippets illustrate how petgraph could be used to model the ADG and run a cycle check.

Rust


// 1. Define custom structs for node and edge data.
#
struct ModuleNode {
    id: String,
    path: String,
    layer: String,
}

#
struct DependencyEdge {
    dep_type: DependencyType,
    weight: f32,
}

enum DependencyType {
    Call,
    Inheritance,
    //... other types
}

// 2. Create the Architectural Dependency Graph (ADG) using petgraph.
use petgraph::graph::{Graph, NodeIndex};
use petgraph::algo::tarjan_scc;
use std::collections::HashMap;

let mut adg = Graph::<ModuleNode, DependencyEdge>::new();
let mut node_map: HashMap<String, NodeIndex> = HashMap::new();

// 3. Populate the graph from the LAAST (pseudo-code).
// for each module_data in laast_modules {
//     let id = module_data.id.clone();
//     let node_idx = adg.add_node(ModuleNode { id: id.clone(),... });
//     node_map.insert(id, node_idx);
// }
// for each dependency in laast_dependencies {
//     let source_idx = node_map.get(&dependency.source_id).unwrap();
//     let target_idx = node_map.get(&dependency.target_id).unwrap();
//     adg.add_edge(*source_idx, *target_idx, DependencyEdge {... });
// }

// 4. Run a cycle detection algorithm.
let sccs = tarjan_scc(&adg);
for scc in sccs {
    if scc.len() > 1 {
        println!("Cyclic dependency found involving modules:");
        for node_idx in scc {
            println!("- {}", adg[node_idx].id);
        }
    }
}


This example demonstrates the ease with which petgraph can be used to model the system and apply powerful, pre-built algorithms like Tarjan's SCC for cycle detection.

6.3 Adapting Algorithms for Different Architectural Styles

To make the analysis engine adaptable to different architectural styles, as discussed in Section 3, the concept of "Architectural Style Profiles" should be implemented. This allows the weighting of dependencies in the ADG to be tuned based on the context of the system being analyzed.
A profile can be implemented as a simple configuration file (e.g., in TOML format) that is loaded by the analysis engine.
Example microservices.toml profile:

Ini, TOML


# Weights for dependencies in a microservices architecture.
# Network calls are architecturally significant and costly.
[weights]
network_call = 10.0
database_access = 8.0
async_message = 7.0
inheritance = 0.5      # Should be rare between services
in_process_call = 1.0  # Less significant than cross-service calls


Example monolith.toml profile:

Ini, TOML


# Weights for dependencies in a monolithic architecture.
# Tight coupling via inheritance and direct calls are key concerns.
[weights]
network_call = 1.0     # Should not be a primary dependency type
database_access = 5.0
async_message = 2.0
inheritance = 8.0
in_process_call = 5.0


The analysis engine would parse this file and use the specified values when calculating edge weights for the ADG. The type of dependency (e.g., network_call) would be identified during the LAAST traversal by looking for usage of specific frameworks or libraries (e.g., Spring Feign, gRPC clients, message queue libraries). This simple yet powerful mechanism allows the same set of detection algorithms to yield contextually relevant results for vastly different system architectures.

Part III: Validation, Confidence, and Hybrid Augmentation

The final part of this framework addresses the crucial aspects of ensuring the developed detectors are accurate, their findings are trustworthy and actionable, and the system is designed to integrate with future technologies like LLMs.

Section 7: A Framework for Validation and Performance Benchmarking

A high-precision detection tool requires a rigorous validation framework. The primary challenge in this domain is the scarcity of publicly available, manually annotated datasets for architectural anti-patterns.43 While datasets for code-level smells exist, they are not suitable for validating architectural detectors.48 Therefore, the first step is to create a reliable ground truth.

7.1 Creating a Benchmark Dataset

A robust benchmark dataset is the cornerstone of validation. The following methodology is proposed for its creation:
Project Selection: Select 5-10 large, well-known open-source projects that span multiple languages (e.g., Java, Python, C#) and exhibit diverse architectural styles. Candidates could include systems like Apache Cassandra, Django, Guava, or Ansible, which are complex enough to contain real architectural challenges.50
Candidate Generation: Run the newly developed deterministic detectors on the selected projects using lenient thresholds. This will produce a large set of candidate anti-pattern instances, maximizing recall at the expense of precision.
Expert Manual Annotation: Assemble a panel of at least three human experts with significant software architecture experience. Each expert independently reviews every candidate instance and classifies it as either a True Positive (TP) (a genuine architectural issue) or a False Positive (FP) (an incorrect finding).
Adjudication and Ground Truth: The final label for each instance is determined by a majority vote from the expert panel. Instances with no clear consensus can be discarded or marked for further discussion. This process results in a high-quality, manually validated ground truth dataset.43
Publication: It is strongly recommended that this annotated dataset be published as an open-source artifact. This would represent a significant contribution to the software engineering research community by filling a well-known and critical gap, enabling other researchers to benchmark their tools.

7.2 Metrics for Evaluation: Precision, Recall, and F1-Score

With a ground truth dataset in place, the performance of the detection algorithms can be quantitatively measured using standard information retrieval metrics.52
True Positives (TP): The number of anti-patterns correctly identified by the tool that are also marked as positives in the ground truth.
False Positives (FP): The number of anti-patterns reported by the tool that are marked as negatives in the ground truth (i.e., "false alarms").
False Negatives (FN): The number of anti-patterns present in the ground truth that the tool failed to detect.
From these counts, the key performance indicators are calculated:
Precision: Measures the accuracy of the reported findings. It answers the question: "Of all the anti-patterns we reported, what fraction were actual problems?" High precision is critical for building user trust and minimizing alert fatigue.
Precision=TP+FPTP​
Recall (Sensitivity): Measures the completeness of the detection. It answers the question: "Of all the actual anti-patterns in the code, what fraction did we find?" High recall is important for ensuring comprehensive coverage.
Recall=TP+FNTP​
F1-Score: The harmonic mean of precision and recall. It provides a single, balanced metric for assessing the overall effectiveness of a detector, which is particularly useful when there is a trade-off between precision and recall.
F1​=2⋅Precision+RecallPrecision⋅Recall​
The primary success criterion is to tune the algorithms to achieve a precision of over 85% on the benchmark dataset, as specified in the research prompt, while maintaining an acceptable level of recall.

7.3 Documenting and Mitigating False Positives/Negatives

The validation process will inevitably uncover systematic sources of error. It is essential to catalog these to guide future improvements. A False Positive/Negative Catalog should be maintained, where each entry contains:
Anti-Pattern: The rule that was triggered incorrectly.
Code Example: A minimal, reproducible code snippet that causes the error.
Root Cause Analysis: A detailed explanation of why the detector failed (e.g., "The algorithm misinterprets fluent API chaining as a deep dependency," or "The taint analysis does not recognize a custom sanitization function.").
Mitigation Strategy: A proposed change to the algorithm or its configuration to correct the behavior (e.g., "Add common fluent API method names to an ignore list," "Allow users to annotate custom sanitizer functions.").

Section 8: A Probabilistic Model for Detection Confidence Scoring

Binary "found/not-found" alerts are insufficient for complex architectural issues. Developers and architects are often inundated with warnings and need a mechanism to prioritize their efforts.9 A confidence score, which quantifies the tool's certainty in each finding, provides this crucial prioritization capability.55

8.1 A Multi-Factor Confidence Model

A robust confidence score for static analysis should be derived from multiple factors that reflect the quality and completeness of the evidence for a given finding. This report proposes a heuristic-based probabilistic model inspired by scoring systems used in threat intelligence, which combine multiple signals to assess confidence.58
The confidence score, C, for a given anti-pattern instance is calculated as a weighted sum of several factors:
C=w1​⋅Pcompleteness​+w2​⋅Estrength​+w3​⋅Sseverity​
Where:
Pcompleteness​ (Pattern Completeness): This factor measures how completely the detected code structure matches the formal DSL specification of the anti-pattern. A finding that matches all clauses of a rule receives a score of 1.0, while a partial match (e.g., a Leaky Abstraction that only leaks through a parameter but not the return type) would receive a lower score.
Estrength​ (Evidence Strength): This factor quantifies the strength of the underlying structural evidence. It is specific to each anti-pattern:
For a Modularity Violation, this could be the normalized weight of the anomalous cross-community edge. A very high-weight edge is stronger evidence than a marginally anomalous one.
For a Cyclic Dependency, this could be inversely proportional to the length of the cycle. Shorter cycles (e.g., A -> B -> A) represent tighter, more problematic coupling and thus receive a higher score than longer, more convoluted cycles.
For Insufficient Access Control, this could be the length of the data flow path. A direct flow from source to sink is stronger evidence than a long, indirect one.
Sseverity​ (Heuristic Severity): This is a configurable factor based on the semantic context of the components involved. For example, an InsufficientAccessControl violation where the sink is Runtime.exec is more severe than one where the sink is a simple database write. A LeakyAbstraction involving a type named Password or Credential is more severe than one involving a generic List. This allows domain-specific knowledge to influence the final score.
w1​,w2​,w3​ are tunable weights that sum to 1, allowing the model to be calibrated based on empirical results from the validation framework.

8.2 Calibrating Thresholds for Actionability

The calculated confidence scores can be used to establish clear thresholds for action. By plotting a Precision-Recall curve based on the validation dataset, the relationship between the confidence score and the detector's performance can be visualized.52 This allows for the selection of meaningful thresholds:
High Confidence / Triage-Free (e.g., C≥0.9): Findings above this threshold have very high precision. They can be reported directly to users as confirmed issues or even be used to fail a CI/CD build automatically.
Medium Confidence / Review-Required (e.g., 0.6≤C<0.9): Findings in this range are plausible but may contain false positives. They are ideal candidates for manual review or for the hybrid LLM triage process described in the next section.
Low Confidence / Investigative (e.g., C<0.6): These findings have a high probability of being false positives or minor issues. They should be suppressed from the default view to avoid noise but made available in an "investigative" or "verbose" mode for architects performing deep dives.
This confidence score becomes more than just a sorting key; it evolves into the central nervous system of the entire analysis and reporting pipeline. It enables intelligent, context-aware automation, allowing the tool to distinguish between critical, actionable alerts and low-priority, informational findings, thereby directly addressing the problem of alert fatigue.

Section 9: Integration Strategy for Hybrid Analysis (Deterministic + LLM)

While the primary goal is deterministic detection, a hybrid approach that leverages LLMs for verification can enhance accuracy and reduce the manual triage burden. The key is to use the LLM not as a primary detector, but as an intelligent assistant to triage the findings of the deterministic engine.

9.1 Review of Hybrid Models

Research into combining deterministic systems and LLMs has identified several primary integration patterns 3:
Rule-Based Preprocessing: Static analysis is used to identify interesting or potentially problematic code snippets, which are then fed to an LLM for deeper, more contextual analysis.
LLM as a Rule Generator: An LLM analyzes a large corpus of code to suggest new detection rules, which are then implemented in a deterministic engine.
Confidence-Based Fallback/Triage: The deterministic tool runs first. If its confidence in a finding is low, or for verification purposes, an LLM is invoked to provide a "second opinion."

9.2 Recommended Strategy: Confidence-Driven LLM Triage

Given the project's requirement for a deterministic-first approach with optional LLM validation, the Confidence-Driven LLM Triage model is the optimal integration strategy. This approach leverages the strengths of both paradigms: the speed and predictability of deterministic analysis, and the nuanced contextual understanding of LLMs.
Proposed Workflow:
Deterministic Analysis: The core static and graph-based analysis engine runs first, producing a set of findings with associated confidence scores, as calculated in Section 8.
Automated Triage: The findings are automatically triaged based on their confidence score:
High-Confidence Findings (C≥0.9): These are reported directly to the user as high-priority issues. No LLM intervention is needed.
Medium-Confidence Findings (0.6≤C<0.9): These findings are automatically flagged for "AI-assisted review." The system invokes an LLM to analyze the finding and provide a recommendation.
Low-Confidence Findings (C<0.6): These are suppressed by default.
LLM Verification: For medium-confidence findings, a detailed prompt is constructed and sent to an LLM (e.g., GPT-4, Claude 3.5 Sonnet). The LLM's response is then attached to the original finding as a review comment, helping the developer decide whether to address the issue.

9.3 Prompt Engineering for Vulnerability Confirmation

The success of the LLM triage step is critically dependent on the quality of the prompt. The prompt must provide the LLM with all necessary context to make an informed judgment, effectively simulating how a human expert would review the finding.63
Proposed Prompt Template:



You are an expert software architect and security reviewer. Your task is to adjudicate a potential architectural anti-pattern identified by a static analysis tool. Analyze the provided information and source code to determine if this is a true positive or a false positive.

**Static Analysis Finding:**
- **Anti-Pattern Type:** {anti_pattern_name}
- **Tool Confidence Score:** {confidence_score}
- **Location:** {file_path}:{line_number}
- **Tool's Explanation:** {generated_explanation_from_detector}

**Source Code Context:**
The following code snippet contains the potential issue. The relevant line is marked with `// VIOLATION HERE`.

```{language}
{relevant_code_snippet_with_surrounding_context_and_line_numbers}


Your Task:
Analyze the finding. Based on the anti-pattern definition and the provided code, evaluate whether this is a genuine architectural issue.
Provide your reasoning. Clearly explain why you believe this is a true positive or a false positive. If it is a true positive, describe the negative consequences. If it is a false positive, explain why the tool's finding is incorrect in this context.
Provide a final verdict. Conclude your response with a single JSON object containing your determination.
Example Response Format:
This is a because... [your detailed reasoning here].
{"verdict": ""}



This structured approach ensures the LLM receives the deterministic tool's preliminary analysis, the precise code location, and the surrounding context, enabling it to function as a highly effective and reliable triage assistant.

This hybrid model also creates a powerful, long-term feedback loop. The LLM-generated verdicts ("True Positive" or "False Positive") on medium-confidence findings can be collected and aggregated (with user consent). By analyzing this data, it becomes possible to identify systematic weaknesses in the deterministic detectors. For instance, if the LLM consistently marks a specific type of `ModularityViolation` as a false positive when it involves a known design pattern like the Visitor pattern, this insight can be used to refine the deterministic rule itself. The LLM thus evolves from a simple validation tool into a key component of an automated, self-improving system that enhances the accuracy of the core deterministic engine over time.

## Conclusion

This report has laid out a comprehensive and actionable framework for advancing the detection of architectural anti-patterns from simplistic heuristics to a system of high-precision, deterministic analysis. The proposed methodology addresses the core requirements of the research prompt by providing a clear path toward a more sophisticated and reliable static analysis engine.

The key deliverables and strategies outlined in this document include:

1.  **Formalization via DSL:** The introduction of a Domain-Specific Language for defining anti-patterns transforms ambiguous concepts into machine-readable, verifiable specifications. This is the bedrock of deterministic detection.
2.  **Unified Code Representation:** The recommendation to adopt a Language-Agnostic Abstract Syntax Tree (LAAST) and an Architectural Dependency Graph (ADG) provides a robust and scalable foundation for multi-language and system-wide structural analysis.
3.  **Advanced Detection Algorithms:** Concrete, deterministic algorithms for detecting complex anti-patterns like **Leaky Abstraction**, **Insufficient Access Control**, and a novel static approach for **Modularity Violation** have been specified, moving beyond a reliance on historical data.
4.  **Robust Validation Framework:** A clear methodology for creating a benchmark dataset, evaluating performance using Precision and Recall, and cataloging errors ensures that the developed tools can be rigorously tested and continuously improved toward the goal of >85% precision.
5.  **Actionable Confidence Scoring:** The multi-factor probabilistic confidence model moves beyond binary alerts, enabling intelligent prioritization and context-aware workflows that reduce alert fatigue and focus developer attention where it is most needed.
6.  **Calibrated Hybrid Integration:** The proposed Confidence-Driven LLM Triage model provides a pragmatic and powerful strategy for integrating deterministic analysis with LLM-based verification, leveraging the strengths of both paradigms while creating a feedback loop for continuous improvement.

By implementing the recommendations within this report, the Uveddi team can develop a next-generation analysis engine capable of identifying nuanced architectural flaws with high precision across a variety of languages and architectural styles. This framework not only provides a solution to the immediate technical challenges but also establishes a strategic foundation for future innovation in the field of automated software quality and architecture assurance.


Works cited
Detecting software modularity violations | Request PDF, accessed June 28, 2025, https://www.researchgate.net/publication/221555767_Detecting_software_modularity_violations
Empirical Evidence of Code Decay: A Systematic Mapping Study - Northwest Missouri State University, accessed June 28, 2025, https://www.nwmissouri.edu/csis/pdf/vitae/bandi/Empirical%20Evidence%20of%20Code%20Decay.pdf
LLMs vs. Rule-Based Systems: Bridging AI with Deterministic Logic ..., accessed June 28, 2025, https://blog.gopenai.com/llms-vs-deterministic-logic-overcoming-rule-based-evaluation-challenges-8c5fb7e8fe46
LLM-Assisted Static Analysis for Detecting Security Vulnerabilities | PromptLayer, accessed June 28, 2025, https://www.promptlayer.com/research-papers/llm-assisted-static-analysis-for-detecting-security-vulnerabilities
Systems Design Anti-Patterns - Number Analytics, accessed June 28, 2025, https://www.numberanalytics.com/blog/systems-design-anti-patterns
Anti-Patterns in Software Architecture - Number Analytics, accessed June 28, 2025, https://www.numberanalytics.com/blog/anti-patterns-in-software-architecture
Anti Pattern - C2 wiki, accessed June 28, 2025, https://wiki.c2.com/?AntiPattern
Anti patterns in software architecture | by Christoph Nißle - Medium, accessed June 28, 2025, https://medium.com/@christophnissle/anti-patterns-in-software-architecture-3c8970c9c4f5
Using Automation to Prioritize Alerts from Static Analysis Tools - SEI Blog, accessed June 28, 2025, https://insights.sei.cmu.edu/projects/using-automation-to-prioritize-alerts-from-static-analysis-tools/
Formal Specification and Verification of Design Patterns | Request PDF - ResearchGate, accessed June 28, 2025, https://www.researchgate.net/publication/290930300_Formal_Specification_and_Verification_of_Design_Patterns
Architecture Anti-patterns: Automatically Detectable Violations of Design Principles - Department of Computer Science, accessed June 28, 2025, https://www.cs.drexel.edu/~yfcai/papers/2019/tse2019.pdf
An Anomaly-Based Approach for Detecting Modularity Violations on ..., accessed June 28, 2025, https://www.researchgate.net/publication/369390255_An_Anomaly-Based_Approach_for_Detecting_Modularity_Violations_on_Method_Placement
A Case Study on Modularity Violations in Cyber Physical Systems - National Science Foundation, accessed June 28, 2025, https://par.nsf.gov/servlets/purl/10186200
Domain-specific language - Wikipedia, accessed June 28, 2025, https://en.wikipedia.org/wiki/Domain-specific_language
Design Spaces of Domain-Specific Languages: Comparing and Contrasting Approaches in PL and HCI - MIT Visualization Group, accessed June 28, 2025, https://vis.csail.mit.edu/pubs/dsl-design-spaces.pdf
The complete guide to (external) Domain Specific Languages - Strumenta - Federico Tomassetti, accessed June 28, 2025, https://tomassetti.me/domain-specific-languages/
How to use Abstraction and the Repository Pattern Effectively in your Flutter apps, accessed June 28, 2025, https://codewithandrea.com/articles/abstraction-repository-pattern-flutter/
Leaky Abstraction - Khalil Stemmler, accessed June 28, 2025, https://khalilstemmler.com/wiki/leaky-abstraction/
Leaky Abstraction — What Is It?. Spot leakness in your code and see how… | by Bartosz Salwiczek | Better Programming - Medium, accessed June 28, 2025, https://medium.com/better-programming/leaky-abstraction-what-is-it-ed0bc84000fd
‍A Guide to Identifying IDOR Vulnerabilities - Aptori, accessed June 28, 2025, https://www.aptori.com/blog/a-guide-to-identifying-idor-vulnerabilities
A Deep Dive Understanding of Smart Contract Vulnerabilities - Part 1 - TUTORIALBOY, accessed June 28, 2025, https://tutorialboy24.hashnode.dev/a-deep-dive-understanding-of-smart-contract-vulnerabilities-part-1
Mitigating Access Control Vulnerabilities through Interactive Static Analysis - ResearchGate, accessed June 28, 2025, https://www.researchgate.net/publication/300579789_Mitigating_Access_Control_Vulnerabilities_through_Interactive_Static_Analysis
Analyzing Modularity Maximization in Approximation, Heuristic, and ..., accessed June 28, 2025, https://arxiv.org/abs/2310.10898
Graph Analysis and Modularity of Brain Functional Connectivity Networks: Searching for the Optimal Threshold - Frontiers, accessed June 28, 2025, https://www.frontiersin.org/journals/neuroscience/articles/10.3389/fnins.2017.00441/full
Advancing Static Code Analysis With Language-Agnostic Component Identification - SciSpace, accessed June 28, 2025, https://scispace.com/pdf/advancing-static-code-analysis-with-language-agnostic-etlsny1x.pdf
GAST: A Generic AST Representation for Language-Independent Source Code Analysis - Visor Redalyc, accessed June 28, 2025, https://www.redalyc.org/journal/5722/572276219003/
Language Agnostic Code Embeddings - ACL Anthology, accessed June 28, 2025, https://aclanthology.org/2024.naacl-long.38.pdf
A language-agnostic framework for mining static analysis rules from code changes - Amazon Science, accessed June 28, 2025, https://www.amazon.science/publications/a-language-agnostic-framework-for-mining-static-analysis-rules-from-code-changes
petgraph - Rust - Shadow, accessed June 28, 2025, https://shadow.github.io/docs/rust/petgraph/index.html
graph - Keywords - crates.io: Rust Package Registry, accessed June 28, 2025, https://crates.io/keywords/graph
petgraph - Rust - Docs.rs, accessed June 28, 2025, https://docs.rs/petgraph/
petgraph/petgraph: Graph data structure library for Rust. - GitHub, accessed June 28, 2025, https://github.com/petgraph/petgraph
Static Code Analysis - OWASP Foundation, accessed June 28, 2025, https://owasp.org/www-community/controls/Static_Code_Analysis
Abstract Syntax Extraction from Context Free Grammar - ResearchGate, accessed June 28, 2025, https://www.researchgate.net/figure/Abstract-Syntax-Extraction-from-Context-Free-Grammar_fig1_327290143
All You Ever Wanted to Know About Dynamic Taint Analysis and Forward Symbolic Execution (but might have been afraid to ask) - Electrical and Computer Engineering, accessed June 28, 2025, https://users.ece.cmu.edu/~aavgerin/papers/Oakland10.pdf
Language-based Security: Access Control and Static Analysis - CiteSeerX, accessed June 28, 2025, https://citeseerx.ist.psu.edu/document?repid=rep1&type=pdf&doi=2e1f97289f198456b3beeca38b6c97b81b78f176
A Replication Case Study to Measure the Architectural Quality of a Commercial System - Department of Computer Science, accessed June 28, 2025, https://www.cs.drexel.edu/~yfcai/papers/2014/esem2014.pdf
A Tool to Address Cybersecurity Vulnerabilities Through Design - SEI Blog, accessed June 28, 2025, https://insights.sei.cmu.edu/blog/a-tool-to-address-cybersecurity-vulnerabilities-through-design/
rustpython_parser - Rust - Docs.rs, accessed June 28, 2025, https://docs.rs/rustpython-parser
Parser tooling — list of Rust libraries/crates // Lib.rs, accessed June 28, 2025, https://lib.rs/parsing
pest. The Elegant Parser, accessed June 28, 2025, https://pest.rs/
zesterer/chumsky: Write expressive, high-performance ... - GitHub, accessed June 28, 2025, https://github.com/zesterer/chumsky
DACOS-A Manually Annotated Dataset of Code Smells - ResearchGate, accessed June 28, 2025, https://www.researchgate.net/publication/369265995_DACOS-A_Manually_Annotated_Dataset_of_Code_Smells
MLScent: A tool for Anti-pattern detection in ML projects - arXiv, accessed June 28, 2025, https://arxiv.org/html/2502.18466v1
Deep Learning Anti-patterns from Code Metrics History - arXiv, accessed June 28, 2025, https://arxiv.org/pdf/1910.07658
Architecture Anti-Patterns: Automatically Detectable Violations of ..., accessed June 28, 2025, https://www.researchgate.net/publication/332646821_Architecture_Anti-Patterns_Automatically_Detectable_Violations_of_Design_Principles
A systematic mapping study on architectural smells detection ..., accessed June 28, 2025, https://www.researchgate.net/publication/347419232_A_systematic_mapping_study_on_architectural_smells_detection
AI Models and Data Evaluation Track. Benchmarking LLM for Code Smells Detection: OpenAI GPT-4.0 vs DeepSeek-V3 - arXiv, accessed June 28, 2025, https://arxiv.org/html/2504.16027v1
DACOS—A Manually Annotated Dataset of Code Smells - Tushar Sharma, accessed June 28, 2025, https://www.tusharma.in/preprints/DacosMSR2023.pdf
Does your architecture smell? - Revisited - Designite, accessed June 28, 2025, https://www.designite-tools.com/blog/does-your-architecture-smell-revisited
Design Smell Detection and Analysis for Open Source Java Software - IIT, DU, accessed June 28, 2025, https://iit.du.ac.bd/about_iit/download/369
Precision and recall - Wikipedia, accessed June 28, 2025, https://en.wikipedia.org/wiki/Precision_and_recall
Understanding Precision and Recall in Model Performance Evaluation - Lyzr AI, accessed June 28, 2025, https://www.lyzr.ai/glossaries/precision-and-recall/
Recall and Precision - Verifysoft, accessed June 28, 2025, https://www.verifysoft.com/en_recall_and_precision.html
Verification, validation, and uncertainty quantification (VVUQ) in structural analysis of concrete dams - Frontiers, accessed June 28, 2025, https://www.frontiersin.org/journals/built-environment/articles/10.3389/fbuil.2024.1452415/full
Alert Severity and Confidence - Taegis Documentation, accessed June 28, 2025, https://docs.taegis.secureworks.com/alerts/alert_severity_confidence/
Confidence score - Custom question answering - Azure AI services - Learn Microsoft, accessed June 28, 2025, https://learn.microsoft.com/en-us/azure/ai-services/language-service/question-answering/concepts/confidence-score
Confidence Scoring in Threat Intelligence | Cyware, accessed June 28, 2025, https://www.cyware.com/resources/security-guides/cyber-threat-intelligence/what-is-confidence-scoring-in-threat-intelligence
How the Confidence Score is Calculated - Data - PurpleAir Community, accessed June 28, 2025, https://community.purpleair.com/t/how-the-confidence-score-is-calculated/10641
Understanding Confidence Scores in Machine Learning: A Practical Guide - Mindee, accessed June 28, 2025, https://www.mindee.com/blog/how-use-confidence-scores-ml-models
Hybrid AI Reasoning: Integrating Rule- Based Logic with Transformer Inference - Preprints.org, accessed June 28, 2025, https://www.preprints.org/manuscript/202504.1453/v1/download
Combining Large Language Models with Static Analyzers for Code Review Generation The replication package is available at https://github.com/ImenJaoua/Hybrid-Code-Review and the data is available at https://zenodo.org/records/14061110. - arXiv, accessed June 28, 2025, https://arxiv.org/html/2502.06633v1
IRIS: LLM-Assisted Static Analysis for Detecting Security ..., accessed June 28, 2025, https://openreview.net/forum?id=9LdJDU7E91
Evaluating Static Analysis Alerts with LLMs - SEI Blog, accessed June 28, 2025, https://insights.sei.cmu.edu/blog/evaluating-static-analysis-alerts-with-llms/
Large Language Models (LLMs) for Software Security Analysis, accessed June 28, 2025, https://www.scirp.org/journal/paperinformation?paperid=142334
