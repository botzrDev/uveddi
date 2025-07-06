
Implementing a Multi-Language Large Class Anti-Pattern Detector: A Comprehensive Technical Report


Section 1: Foundational Principles of the Large Class Anti-Pattern

The implementation of a robust static analysis detector requires a deep, principled understanding of the problem it aims to solve. The "Large Class" anti-pattern is not merely a matter of code size; it is a symptom of fundamental design erosion that compromises maintainability, readability, and reliability. This section establishes the theoretical groundwork for the detector, defining the anti-pattern and its variants, identifying its root cause in the violation of core object-oriented principles, and proposing a multi-faceted metric-based framework for its detection.

1.1 Defining the Large Class, God Object, and The Blob

While often used interchangeably, the terms "Large Class," "God Object," and "The Blob" represent points on a spectrum of design decay. Recognizing their distinctions is key to building a nuanced detector.
A Large Class is the most general of these terms, identified as a class that has accumulated an excessive number of responsibilities, methods, fields, or lines of code.1 This growth is often gradual, as developers find it easier to add new functionality to an existing class rather than creating a new one.3 The result is a class that becomes complex and difficult to manage, a "code smell" indicating potential underlying issues.4
A God Object, also known as an omniscient or all-knowing object, represents a more severe and architecturally significant anti-pattern.5 It is a class that not only is large but also centralizes a disproportionate amount of the system's intelligence.6 It knows too much about other parts of the system and controls too many other objects, effectively becoming a central hub for information and interaction.7 This tight coupling across the system makes maintenance perilous, as a change in the God Object can have unpredictable ripple effects on unrelated functionalities.5 It is the object-oriented analog of procedural programs that rely heavily on global variables to store state information.5
The Blob is a synonym for the God Object, a term that vividly describes a class that has "consumed" the responsibilities of other potential classes, growing larger over time.9 The term particularly highlights the procedural-style design often found within these classes, where one complex controller class monopolizes all processing logic, while surrounding classes are reduced to simple data containers.11
The common thread and fundamental root cause of these anti-patterns is the violation of the Single Responsibility Principle (SRP).4 The SRP states that a module or class should have one, and only one, reason to change.8 A Large Class, by definition, handles multiple unrelated concerns, inherently violating this principle.13 This violation leads to a cascade of negative consequences, including reduced maintainability, poor readability, and significant challenges in testing and reuse.4
It is crucial, however, to interpret SRP correctly for automated analysis. The principle is not violated simply by a class having multiple methods or pieces of logic.16 If all operations within a class serve a single, cohesive purpose, it adheres to SRP.17 The abstract nature of "responsibility" makes it impossible to measure directly. Therefore, a static analysis tool must rely on measurable proxies, which will be the focus of this report.

1.2 A Multi-Metric Approach to Detection

A naive approach to detecting a Large Class, such as merely counting lines of code (LOC), is insufficient and prone to false positives.18 A class can be large for legitimate reasons, such as being a simple but comprehensive data container. The true anti-pattern emerges when size is combined with other negative characteristics.
Therefore, a sophisticated detector must adopt a multi-metric approach, analyzing a class from several perspectives to build a holistic profile of its health. This report proposes a detection framework based on three categories of metrics:
Size Metrics: These are quantitative measures that assess the physical or logical size of the class. They answer the question, "Is it big?" Key metrics include Lines of Code (LOC), Number of Methods (NOM), and Number of Fields (NOF).
Complexity Metrics: These metrics evaluate the logical intricacy of the code within the class. They answer the question, "Is it complicated?" Key metrics include Cyclomatic Complexity (CC) and Cognitive Complexity.
Structural Metrics: These metrics analyze the design integrity of the class and its role within the broader system architecture. They answer the questions, "Is it cohesive?" and "Is it overly entangled?" Key metrics include Lack of Cohesion in Methods (LCOM) and Coupling Between Objects (CBO).
By combining these three pillars of analysis, the detector can move beyond simplistic heuristics. It can differentiate between a benign, large-but-simple class and a truly problematic God Object that is large, complex, non-cohesive, and highly coupled. This multi-faceted view is essential for providing accurate, actionable findings and minimizing the noise of false positives that often leads developers to distrust and disable static analysis tools.19

Section 2: Core Measurement Metrics and Heuristics

To implement the proposed multi-metric framework, it is necessary to define each metric precisely. This section provides a detailed examination of the core metrics for size, complexity, and structure. These metrics form a language-agnostic foundation that can be applied across Rust, Python, and JavaScript, with language-specific adaptations discussed in the subsequent section. For each metric, we define its purpose, calculation method, and a recommended default threshold justified by industry precedents and academic research.

2.1 Size-Based Metrics

Size metrics provide the most straightforward, first-pass indication of a potential Large Class. While insufficient on their own, they are an indispensable part of the overall heuristic.

2.1.1 Lines of Code (LOC)

Lines of Code is a fundamental software metric used to measure the size of a program by counting the number of lines in its source code.21 For the purpose of anti-pattern detection, it is critical to distinguish between physical and logical LOC.
Physical LOC (LINES) counts all lines, including comments and blank lines. This is highly sensitive to formatting and coding style and is therefore a poor choice for a consistent metric.21
Logical LOC (LLOC) or Source LOC (SLOC) attempts to measure the number of executable statements, typically excluding comments and blank lines.18 This provides a more accurate measure of the code's functional substance.
For our detector, we will exclusively use Logical Lines of Code (LLOC). This aligns with the evolution of industry tools like SonarQube, which updated its file-length rule (S104) to count only lines of code, not comments or blank lines, to reduce noise.24

2.1.2 Number of Methods (NOM) / Weighted Methods per Class (WMC)

The Number of Methods (NOM) is a simple count of the methods defined within a class. It directly measures the breadth of the class's public interface and its set of behaviors.25 A class with an excessive number of methods is a primary indicator of violating the Single Responsibility Principle.4
The Weighted Methods per Class (WMC) metric is a more general form of NOM. It is defined as the sum of the complexities of all methods within a class.26 If each method is assigned a complexity of 1, then
WMC=NOM.25 For our size-based heuristic, we will use this simplified definition where WMC is equivalent to the method count. The complexity aspect of WMC will be handled separately by our complexity metrics.

2.1.3 Number of Fields (NOF) / Attributes (NOA)

The Number of Fields (NOF), also called Number of Attributes (NOA), is a count of the instance variables or fields declared in a class. A large number of fields indicates that the class is responsible for managing a large amount of state, which is a key characteristic of a Large Class or God Object.4 Pylint, a widely used linter for Python, includes a check for
max-attributes (R0902), demonstrating the utility of this metric in practice.28

2.2 Complexity Metrics

Complexity metrics move beyond size to quantify the difficulty of understanding and testing the code. A large class filled with simple, linear code is less problematic than a smaller class filled with convoluted logic.

2.2.1 Cyclomatic Complexity (CC)

Developed by Thomas McCabe, Cyclomatic Complexity measures the number of linearly independent paths through a program's source code.29 It is calculated by counting the number of decision points in the code and adding one.30 Decision points include
if, while, for, case, and logical operators like &&, ||, and the ternary operator ?.29
The formula based on a control-flow graph is:

C=E−N+2P

where E is the number of edges, N is the number of nodes, and P is the number of connected components (for a single function, P=1).32
A high CC value indicates that code is complex, difficult to test thoroughly, and more prone to errors.34 For our detector, we will calculate the CC for each method and then compute the
Average Cyclomatic Complexity (AvgCC) for the class. A high AvgCC is a strong signal that the class's logic is overly convoluted.

2.2.2 Cognitive Complexity

Cognitive Complexity, a metric developed and championed by SonarSource, measures how difficult code is for a human to read and understand.36 Unlike Cyclomatic Complexity, which treats all decision points equally, Cognitive Complexity applies penalties for constructs that break the linear flow of code and for nesting these constructs.36
Key principles of Cognitive Complexity include:
Increments for breaks in flow: if, for, while, switch, catch, goto, break, and continue statements increment the complexity.
Increments for logical operators: Sequences of logical operators like && and or increase complexity.
Increments for nesting: Nesting flow-breaking structures inside one another adds a penalty, reflecting the increased mental effort required to keep track of the context.36
No penalty for method calls: Well-named methods are seen as simplifying code, so calls to them do not increase complexity (with the exception of recursive calls).36
Because it aligns more closely with human perception of complexity, Cognitive Complexity is often a superior indicator of maintainability issues.37 A class with methods that have a high aggregate or average Cognitive Complexity is a prime candidate for refactoring.

2.3 Structural Metrics (Cohesion and Coupling)

Structural metrics provide the deepest level of insight, analyzing a class's design quality and its relationship with the rest of the system. They are the most effective proxies for detecting violations of the Single Responsibility Principle.

2.3.1 Lack of Cohesion in Methods (LCOM)

Cohesion refers to the degree to which the elements inside a module or class belong together.15 A class is cohesive if its methods and variables are co-dependent and form a logical whole.40 Lack of Cohesion in Methods (LCOM) is a metric designed to quantify this. A high LCOM value signifies low cohesion, indicating that a class is likely performing multiple unrelated functions and should be split.41
There are several variants of the LCOM metric (LCOM1-5, TCC, LCC), each with different calculation methods and nuances.15 LCOM1 and LCOM2, early versions, have known flaws and can produce misleading results.45
This report recommends using LCOM4 (Hitz & Montazeri), which is widely regarded as one of the most robust and intuitive variants.41 LCOM4 is defined as the number of "connected components" in a class.41
The calculation for LCOM4 is as follows 41:
Create a graph where the nodes represent all methods and instance fields within the class.
Add an edge between a method node and a field node if the method accesses that field.
Add an edge between two method nodes if one method calls the other.
The LCOM4 value is the total number of connected components in the resulting graph.
The interpretation is straightforward:
LCOM4 = 1: The class is cohesive. All methods and fields are connected, directly or indirectly, forming a single component. This is the ideal state.42
LCOM4 > 1: The class is not cohesive. It contains two or more independent groups of methods/fields. This is a strong indication that the class has multiple responsibilities and should be split into LCOM4 separate classes.41
LCOM4 = 0: This occurs if a class has no methods, which is also considered a design flaw.41

2.3.2 Coupling Between Objects (CBO)

While cohesion measures internal integrity, coupling measures external dependencies. The Coupling Between Objects (CBO) metric for a class is a count of the number of other classes to which it is coupled.48 This coupling can occur through various mechanisms, including method calls, field accesses, inheritance, use in method arguments or return types, and generic instantiations.48
A God Object, by its nature as a central controller, will exhibit a very high CBO.5 High coupling is undesirable because it makes a system rigid and fragile; changes in one class can trigger a cascade of required changes in all the classes coupled to it, increasing maintenance effort and risk.50

2.4 Proposed Metric Suite and Default Thresholds

Based on the analysis of these metrics and a review of industry-standard tools, the following suite of metrics and default thresholds is proposed as a starting point for the Large Class detector. These values are intended to be configurable by the end-user but provide a sensible and defensible baseline.
Table 1: Core Metrics and Default Thresholds for Large Class Detection

Metric
Description
Default Threshold
Rationale & Sources
Logical Lines of Code (LLOC)
Total number of executable lines in the class.
500
A common starting point. SonarQube's S104 uses 1000 for a file, but classes should be smaller. Recommendations usually range from 100 to 500.40 500 is a reasonable "warning" level.
Number of Methods (NOM)
Total number of methods in the class.
20
Pylint's max-public-methods rule defaults to 20, providing a strong industry precedent.28
Number of Fields (NOF)
Total number of instance fields/attributes.
15
Pylint's max-attributes is a strict 7.28 A more lenient starting point of 15 is less likely to generate noise for data-intensive classes while still flagging excessive state.
Average Cyclomatic Complexity (AvgCC)
The average CC across all methods in the class.
5
A NIST study suggests a limit of 10 per function is a good starting point.34 An average of 5 for the whole class indicates that while some methods may be simple, others are likely becoming complex.
Lack of Cohesion in Methods (LCOM4)
Number of disconnected groups of methods/fields.
> 1
An LCOM4 value greater than 1 is the theoretical definition of a non-cohesive class that violates SRP and should be split.41
Coupling Between Objects (CBO)
Number of other classes this class is coupled to.
10
Microsoft's documentation on code metrics suggests an optimal limit of 9.49 While Embold suggests a very high 30 48, a value of 10 is a more common and reasonable threshold for identifying high coupling.

The power of this framework lies not in any single metric but in their interplay. A class may legitimately exceed one threshold without being an anti-pattern. For example, a Data Transfer Object (DTO) will naturally have a high Number of Fields (NOF) and a high LCOM4, as each getter/setter pair can form its own isolated component. A naive detector would flag this as a severe issue. However, a sophisticated detector would recognize that this same class has very low Average Cyclomatic Complexity (AvgCC ≈ 1) and consists almost entirely of simple accessor methods. By combining metrics, the detector can build heuristics to identify and suppress such common false positives. A rule might state: "Flag if LCOM4 > 1, unless the class matches the DTO pattern (high NOF, low AvgCC, simple methods)." This contextual awareness is what elevates a detector from a simple counter to an intelligent analysis tool.

Section 3: Language-Specific Implementation Details

While the core metrics provide a universal framework, their practical implementation and interpretation must be tailored to the unique characteristics of each target language. The concept of a "class" itself manifests differently across Python, JavaScript, and Rust. A robust multi-language detector must first abstract this concept into a common "analyzable entity" and then apply language-specific parsing and analysis to populate the metrics for that entity. This section details the nuances, tooling precedents, and specific Tree-sitter queries required for each language.

3.1 Python Implementation

Python's dynamic nature and strong conventions over strict enforcement shape how the Large Class anti-pattern appears and how it should be detected.

3.1.1 Language Nuances

In Python, class boundaries are flexible. Attributes can be added dynamically at runtime, though the common convention is to declare all instance attributes within the __init__ method.54 Duck typing is prevalent, meaning the focus is on an object's behavior rather than its explicit type. This makes structural metrics like LCOM and CBO particularly important, as they analyze the
actual usage patterns of methods and attributes rather than just static declarations.

3.1.2 Tooling Precedents

Pylint is the de facto standard static analysis tool for Python, and its design checker provides excellent, empirically validated precedents for our detector's thresholds.28 Key Pylint rules include:
too-many-instance-attributes (R0902): Defaults to a maximum of 7 attributes.
too-many-public-methods (R0904): Defaults to a maximum of 20 public methods.
too-many-statements: Defaults to a maximum of 50 statements in a function or method.
too-many-branches: Defaults to a maximum of 12 branches in a function or method.
The McCabe max-complexity check defaults to a Cyclomatic Complexity of 10.
These defaults, particularly for method and attribute counts, serve as a strong baseline for our detector's configuration.

3.1.3 Tree-sitter Queries for Python Class Analysis

To implement the metric calculations, we will use the tree-sitter-python grammar.56 The analysis process involves parsing the source code into an Abstract Syntax Tree (AST) and then executing queries to extract the necessary nodes for counting and analysis.
Table 2: Key Tree-sitter Node Types for Python
Element
Tree-sitter Node Type
Class Definition
class_definition
Method
function_definition (as a child of a class_definition's block)
Field (in __init__)
assignment or typed_assignment where the left operand is an attribute (e.g., self.foo)
Field (class-level)
assignment or typed_assignment in the class body
Method Call
call
Attribute Access
attribute

The following S-expression queries can be used to capture the relevant parts of a Python class.
Query to identify a class and its body:
This query captures the name of the class and the block node containing its members, which serves as the scope for further analysis.

Code snippet


(class_definition
  name: (identifier) @class.name
  body: (block) @class.body)


Query to count methods within a class body:
This query is executed within the scope of a captured @class.body to find all top-level function definitions.

Code snippet


(block
  (function_definition
    name: (identifier) @method.name))


Query to count field assignments within an __init__ method:
This query specifically targets the __init__ method and counts assignments to instance attributes (e.g., self.x =...). This is the most common pattern for defining fields in Python.

Code snippet


(function_definition
  name: (identifier) @name
  (#eq? @name "__init__")
  body: (block
    (expression_statement
      (assignment
        left: (attribute object: (identifier) @self (#eq? @self "self")) @field.name))))



3.2 JavaScript/TypeScript Implementation

The JavaScript ecosystem presents a unique challenge due to its evolution from prototype-based objects to ES6 classes, and the widespread adoption of component-based frameworks like React.

3.2.1 Language Nuances

Modern JavaScript utilizes ES6 class syntax, which provides a clearer structure for object-oriented programming.57 However, the language's dynamic heritage remains. TypeScript adds a layer of static typing, which can be leveraged by a static analyzer for more accurate CBO and LCOM calculations by resolving types and dependencies more reliably.58
A critical consideration is the prevalence of large React components. A complex functional component with numerous useState hooks, useEffect calls, and sprawling JSX can be considered a modern incarnation of the Large Class anti-pattern, violating SRP by mixing state management, data fetching, and rendering logic.60 An effective detector must be able to analyze these components as "class-like" entities.

3.2.2 Tooling Precedents

ESLint is the dominant linting tool for JavaScript and TypeScript.
Core ESLint Rules: Provide basic size and complexity checks, such as max-lines (default 300 per file), max-lines-per-function, and complexity (default Cyclomatic Complexity of 20).53
eslint-plugin-sonarjs: This official SonarSource plugin is a crucial precedent, offering a cognitive-complexity rule with an empirically determined default of 15.38 It also includes many other rules for detecting bugs and code smells, making it a valuable reference for quality standards.66
Community Plugins: Tools like eslint-plugin-complexity and eslintcc focus specifically on complexity metrics, introducing concepts like risk ranking (e.g., A-F grades), which can inform the design of a severity scoring system.67

3.2.3 Tree-sitter Queries for JavaScript/TypeScript Analysis

Analysis will be performed using the tree-sitter-javascript and tree-sitter-typescript grammars. The queries are similar for both, with TypeScript providing richer type information nodes.
Table 3: Key Tree-sitter Node Types for JavaScript/TypeScript
Element
Tree-sitter Node Type
Class Definition
class_declaration, class (as part of export_statement or expression)
Method
method_definition
Field
field_definition (TS), public_field_definition (JS)
React Component (Func)
function_declaration or arrow_function returning a jsx_element
React Props
jsx_attribute within a jsx_opening_element
React State Hook
call_expression with function name useState
React Effect Hook
call_expression with function name useEffect

Query to identify a class and its body:

Code snippet


(class_declaration
  name: (identifier) @class.name
  body: (class_body) @class.body)


Query to count methods within a class body:

Code snippet


(class_body
  (method_definition
    name: (property_identifier) @method.name))


Query to identify a React functional component:
This query looks for an exported function or arrow function that contains a return statement with a JSX element. This heuristic is effective for identifying component definitions.

Code snippet


(export_statement
  value: [
    (function_declaration
      body: (statement_block
        (return_statement (jsx_element) @component.body)
      ) @component.scope
    )
    (variable_declaration
      (variable_declarator
        value: (arrow_function
          body: [
            (statement_block (return_statement (jsx_element) @component.body))
            (jsx_element) @component.body
          ]
        )
      ) @component.scope
    )
  ]
)



3.3 Rust Implementation

Rust's ownership model, trait-based polymorphism, and lack of traditional classes require a different perspective on the Large Class anti-pattern. Here, the focus shifts from a single class keyword to the combination of data (struct) and behavior (impl).

3.3.1 Language Nuances

In Rust, data and behavior are decoupled. A struct or enum defines the data layout, while one or more impl blocks associate methods with that type.68 A "Large Class" in Rust can therefore manifest in several ways:
A struct with an excessive number of fields.
A single impl block containing too many methods.
A type with multiple impl blocks that, when combined, represent an overwhelming amount of functionality.
A type whose size in bytes is excessively large, risking stack overflow.69
The "analyzable entity" for Rust is therefore the combination of a struct definition and all impl blocks associated with it.

3.3.2 Tooling Precedents

The official Rust compiler (rustc) and its linter, Clippy, are the primary sources for tooling precedents. Clippy's lints are particularly insightful as they often focus on performance and memory safety, which are core tenets of Rust.
too_many_arguments: While a function-level lint, it can indicate that a struct is missing and that these arguments should be fields.
cyclomatic_complexity: Clippy sets a notably higher default threshold of 25.70 This reflects a community tolerance for more complex functions, possibly due to the expressiveness of constructs like
match statements. This higher baseline should be considered for the Rust-specific configuration of the detector.
Size-based Lints: Clippy includes several lints that focus on the byte size of types, not just their line count. These include large_enum_variant (default 200 bytes), large_stack_frames (default 512KB), and large_types_passed_by_value (default 256 bytes).69 This suggests that for Rust, a comprehensive detector could optionally incorporate
std::mem::size_of as a metric.

3.3.3 Tree-sitter Queries for Rust Struct and Impl Analysis

Analysis will use the tree-sitter-rust grammar.71 The process requires two stages: first, identify all
structs, and second, find all impl blocks that target those structs to aggregate their methods.
Table 4: Key Tree-sitter Node Types for Rust
Element
Tree-sitter Node Type
Struct Definition
struct_item
Struct Fields
field_declaration within a field_declaration_list
Implementation Block
impl_item
Methods
function_item within an impl_item's declaration_list
Field Access
field_expression
Method Call
call_expression with a field_expression as the function

Query to identify structs and their field lists:

Code snippet


(struct_item
  name: (type_identifier) @struct.name
  body: (field_declaration_list) @struct.fields)


Query to count fields within a struct:
This query is run within the scope of a captured @struct.fields.

Code snippet


(field_declaration_list
  (field_declaration
    name: (field_identifier) @field.name))


Query to find impl blocks and their associated methods:
This query identifies all impl blocks and captures the type they implement, allowing the analyzer to associate them with the correct struct.

Code snippet


(impl_item
  type: (type_identifier) @impl.for_type
  body: (declaration_list
    (function_item
      name: (identifier) @method.name)))


The implementation logic must first run the struct_item query to get a list of all structs. Then, it must run the impl_item query and correlate the captured @impl.for_type with the struct names. Finally, for each struct, it aggregates all fields and all methods from all its associated impl blocks to form the complete "analyzable entity" to which the core metrics (LLOC, NOM, NOF, LCOM4, etc.) are applied.

Section 4: Advanced Detection and System Integration

A truly effective anti-pattern detector must go beyond simple metric calculations. It needs to be robust, intelligent, and seamlessly integrated into the developer's workflow. This involves designing a nuanced severity scoring system, proactively addressing common false positives, establishing a rigorous testing strategy, and ensuring the tool can be easily deployed in modern CI/CD and IDE environments.

4.1 Designing a Flexible Severity Scoring System

Not all large classes are created equal. A class that slightly exceeds a line count threshold is far less problematic than a true God Object that violates multiple design principles. A binary "is large / is not large" finding is unhelpful and likely to be ignored. The detector must therefore implement a configurable, multi-tiered severity scoring system (e.g., Info, Low, Medium, High, Critical).
The severity score should be a function of both the number of metrics that exceed their thresholds and the magnitude by which they do so. This allows for a more granular and context-aware assessment. A potential heuristic model could be:
Info: A single, minor size-based metric is slightly exceeded. For example, a class with 22 methods (where the threshold is 20) but is otherwise well-structured. This serves as a gentle nudge for the developer to be mindful of future growth.
Low: Multiple size-based metrics are exceeded, or a complexity metric is moderately high. For example, a class with over 600 LLOC and 25 methods.
Medium: A structural metric indicates a design weakness. For example, a class with an LCOM4 score of 3, indicating it has three distinct responsibilities, even if its size is moderate.
High: A combination of high size/complexity and poor structure. For example, a class with over 800 LLOC, an AvgCC of 8, and an LCOM4 of 4.
Critical: The class exhibits all the hallmarks of a God Object: extreme size, high complexity, very low cohesion, and high coupling. For example, a class with >1500 LLOC, LCOM4 > 5, and CBO > 20. This finding should be treated as a high-priority technical debt item.
Crucially, this scoring system must be configurable. The definition of "too large" is highly context-dependent; a limit that is appropriate for a new microservice may be entirely impractical for a 20-year-old legacy monolith.17 The variance in default thresholds across established tools like Pylint (7 attributes), Microsoft's analyzer (9 CBO), and Embold (30 CBO) is not a contradiction but evidence of different philosophies and target contexts.28 Therefore, the detector's long-term adoption hinges on its tunability. Users must be able to adjust thresholds and the weighting of each metric in the final severity calculation to align the tool with their project's specific domain, framework, and tolerance for technical debt.72

4.2 Addressing Detection Challenges and Edge Cases

The value of a static analysis tool is determined as much by the non-problems it doesn't report as by the real problems it finds. Minimizing false positives is paramount to building developer trust and preventing the tool from being disabled.19 The following edge cases are the most common sources of noise and must be handled gracefully.
Generated Code: Code generated by ORMs, protocol compilers (like Protobuf), or UI designers is a notorious source of large, complex, but unmodifiable classes. The detector must provide multiple strategies to exclude this code from analysis.73
Heuristics:
File Header Comments: Scan the top of each file for common markers like // <auto-generated>, // DO NOT EDIT, or comments indicating the code-generation tool.
File Naming Conventions: Allow users to configure glob patterns to ignore files matching common conventions, such as *.designer.cs, *.generated.rs, or *.pb.go.
Attributes/Annotations: In languages that support them, look for specific attributes like C#'s [GeneratedCode] or Java's @Generated annotation.73
Framework-Specific Classes: Many application frameworks require developers to inherit from large base classes (e.g., UIViewController in iOS, Android.app.Activity, legacy System.Web.UI.Page). These subclasses often appear large and non-cohesive by necessity.
Heuristic: The detector must allow users to configure a list of base classes, implemented interfaces, or traits to ignore. For instance, a configuration could specify: "do not flag any class that inherits from MyFramework.BaseController." This prevents penalizing developers for adhering to the framework's required architecture.74
Data Transfer Objects (DTOs) and Plain Old Data (PODs): As discussed previously, these classes are designed to be simple data containers. They will naturally have a high number of fields and a high LCOM4 score, as each property's accessor methods form a distinct component.
Heuristic: A class can be identified as a likely DTO or POD if it meets a specific profile: a high NOF, a high LCOM4, but a very low number of complex methods. Specifically, its Average Cyclomatic Complexity should be close to 1, and its methods should consist almost exclusively of simple getters and setters. When this pattern is detected, the LCOM4-based finding should be suppressed or assigned a minimal severity.
Fluent Interfaces and Builder Patterns: Classes implementing these patterns often have a large number of methods. However, these methods are typically simple, cohesive, and designed to be chained.
Heuristic: A strong indicator of a fluent interface or builder is that a majority of its methods have a return type of self or this. When this pattern is detected, the NOM metric should be weighted less heavily in the severity calculation.

4.3 A Strategy for Testing the Detector

A comprehensive testing strategy is essential to ensure the detector's accuracy, stability, and performance.
Unit Testing: Each individual metric calculation (LLOC, NOM, CC, LCOM4, CBO) must be isolated and tested with a suite of unit tests. These tests should use simple, hand-crafted code snippets with predictable outputs to verify the correctness of the core algorithms.
Integration and Golden File Testing: This is the most critical testing phase for the detector as a whole.
Create a Code Corpus: Assemble a diverse collection of source code files that represent a wide range of scenarios, including:
Clear True Positives: Textbook examples of God Objects and bloated classes.
Clear True Negatives: Examples of well-structured, cohesive classes of varying sizes.
Known Edge Cases: Specific examples of generated code, framework subclasses, DTOs, and builder patterns that should be ignored or handled with special logic.
Establish a Golden File: Run the detector on this corpus and save the resulting structured output (e.g., in JSON or SARIF format) as the "golden file." This file represents the expected, correct output for the entire test suite.
Regression Testing: In the CI pipeline, every code change to the detector should trigger a new run against the corpus. The new output is then compared to the golden file. Any deviation indicates a regression—a fix for one scenario may have inadvertently broken another. This approach provides a robust safety net for maintaining the detector's complex heuristics.
Performance Testing: Static analysis tools must not become a bottleneck in the development cycle. The detector should be benchmarked against very large source files (e.g., 10,000+ LLOC) to ensure its analysis time remains within acceptable limits. Tree-sitter's incremental parsing capabilities, which allow re-parsing only the changed portions of a file, are essential for achieving the performance required for real-time IDE integration.56

4.4 Integration into a Static Analysis Pipeline

To provide maximum value, the detector must be integrated into the environments where developers work.
CI/CD Integration: The detector must be a command-line tool that can be easily invoked within any CI/CD platform (e.g., Jenkins, GitHub Actions, GitLab CI). It should produce a standard, machine-readable output format like SARIF (Static Analysis Results Interchange Format) or JSON, which can be consumed by these platforms for reporting, quality gates, and pull request annotations.20
IDE Integration (Real-time Linting): For immediate feedback, the detector's logic should be exposed through a Language Server Protocol (LSP) implementation or a dedicated IDE extension (e.g., for VS Code, JetBrains IDEs).75 This requires the analysis to be extremely fast. Leveraging Tree-sitter's incremental parsing is non-negotiable for this use case, as it allows the analyzer to update its findings in milliseconds by only re-evaluating the code that has changed.56
Configuration Management: A user-friendly configuration system is vital for adoption. The tool should look for a configuration file (e.g., .largeclass_config.yml, pyproject.toml) in the root of the user's repository. This file should allow users to easily override default thresholds, specify file paths to ignore, and configure framework-specific exclusions.55

Section 5: Actionable Recommendations and Refactoring Guidance

A static analysis tool that only identifies problems without offering solutions provides limited value. The final and most crucial step is to guide the user toward remediation. This section outlines common refactoring patterns for addressing the Large Class anti-pattern and demonstrates how the detector can provide prescriptive, context-aware recommendations.

5.1 Common Refactoring Patterns for Large Classes

The choice of refactoring technique depends on the specific symptoms exhibited by the large class. The multi-metric approach allows the detector to infer the underlying problem and suggest the most appropriate solution.
Extract Class: This is the primary and most common refactoring for a Large Class.3 It is applied when a subset of the class's fields and methods serves a distinct responsibility that can be spun off into a separate component. The original class then creates an instance of the new class and delegates work to it.77 This pattern is the direct remedy for a
high LCOM4 score, as it physically separates the disconnected components identified by the metric.78
Extract Subclass (or Strategy Pattern): This pattern is used when a class has behavior that varies based on some internal state or type code, often manifesting as large switch statements or long chains of if-elif-else blocks.3 Instead of this complex conditional logic, the varying behavior is encapsulated into separate subclasses or interchangeable strategy objects.78 This is the recommended refactoring when the detector finds a class with
high Cyclomatic or Cognitive Complexity concentrated in methods that perform type or state checks.
Extract Interface (or Rust Trait): This pattern is applicable when different clients of a large class use distinct and separate subsets of its public methods.3 By defining smaller, role-based interfaces, clients can depend on a more focused and stable abstraction rather than the entire bloated class. This reduces coupling and clarifies the intended use cases for different parts of the class's functionality. This is a good recommendation when the
CBO metric is high, and analysis shows that different client classes call different sets of methods.
Replace Data Value with Object: This refactoring is used when a class becomes bloated with logic that rightfully belongs to one of the primitive data types it contains. For example, a Customer class might have numerous methods for parsing, validating, and formatting a phoneNumber string. The solution is to create a new PhoneNumber class (a Value Object) that encapsulates this data and its associated behavior. The Customer class then simply holds an instance of PhoneNumber, simplifying its own implementation.

5.2 Language-Specific Refactoring Examples

To make these abstract patterns concrete, the following examples illustrate their application.

5.2.1 Python: Extract Class

A common scenario is an Order class that also manages customer details. This leads to low cohesion, as order logic and customer logic are unrelated.
Before Refactoring (High LCOM4):

Python


class Order:
    def __init__(self, order_id, customer_name, customer_address):
        self.order_id = order_id
        self.items =
        self.total_price = 0.0
        # Customer details mixed in
        self.customer_name = customer_name
        self.customer_address = customer_address

    def add_item(self, item, price):
        self.items.append(item)
        self.total_price += price

    def get_customer_shipping_label(self):
        return f"{self.customer_name}\n{self.customer_address}"


After Refactoring (LCOM4 = 1 for both classes):
The detector would identify a high LCOM4 and recommend Extract Class. The developer creates a Customer class.

Python


class Customer:
    def __init__(self, name, address):
        self.name = name
        self.address = address

    def get_shipping_label(self):
        return f"{self.name}\n{self.address}"

class Order:
    def __init__(self, order_id, customer):
        self.order_id = order_id
        self.customer = customer  # Composition
        self.items =
        self.total_price = 0.0

    def add_item(self, item, price):
        self.items.append(item)
        self.total_price += price



5.2.2 JavaScript: Extract Class/Module

In a monolithic JavaScript file for a game, logic for rendering, event handling, and utility functions can become entangled.79
Before Refactoring (High LLOC, High NOM):

JavaScript


// index.js
class Game {
    constructor(canvas) {
        this.canvas = canvas;
        this.ctx = canvas.getContext('2d');
        this.setupEventListeners();
        //... more setup
    }

    setupEventListeners() {
        this.canvas.addEventListener('click', () => this.handleClick());
    }

    handleClick() { /*... */ }

    // Utility function mixed in
    getRandomNumber(min, max) {
        return Math.random() * (max - min) + min;
    }

    draw() { /*... drawing logic... */ }
}


After Refactoring (Improved Modularity):
The detector would flag the high line count and method count and could suggest extracting unrelated responsibilities.

JavaScript


// utils.js
export function getRandomNumber(min, max) {
    return Math.random() * (max - min) + min;
}

// event-listeners.js
export function setupEventListeners(gameInstance) {
    gameInstance.canvas.addEventListener('click', () => gameInstance.handleClick());
}

// game.js
import { getRandomNumber } from './utils.js';
import { setupEventListeners } from './event-listeners.js';

class Game {
    constructor(canvas) {
        this.canvas = canvas;
        this.ctx = canvas.getContext('2d');
        setupEventListeners(this);
        //...
    }
    handleClick() { /*... */ }
    draw() { /*... */ }
}



5.2.3 Rust: Extract Struct (Composition)

A large Player struct in a game might handle state, physics, and rendering logic simultaneously, violating SRP.
Before Refactoring (High NOF, High LCOM4 on its impl):

Rust


struct Player {
    // State
    health: u32,
    mana: u32,
    name: String,
    // Physics
    position: (f32, f32),
    velocity: (f32, f32),
    // Rendering
    sprite_id: u32,
}

impl Player {
    fn take_damage(&mut self, amount: u32) {
        self.health = self.health.saturating_sub(amount);
    }

    fn update_physics(&mut self, delta_time: f32) {
        self.position.0 += self.velocity.0 * delta_time;
        self.position.1 += self.velocity.1 * delta_time;
    }

    fn draw(&self, renderer: &mut Renderer) {
        renderer.draw_sprite(self.sprite_id, self.position);
    }
}


After Refactoring (Cohesive Components):
The detector would identify the low cohesion and suggest splitting the struct. The developer uses composition to create focused components.80

Rust


struct PlayerState {
    health: u32,
    mana: u32,
    name: String,
}
impl PlayerState {
    fn take_damage(&mut self, amount: u32) {
        self.health = self.health.saturating_sub(amount);
    }
}

struct PhysicsBody {
    position: (f32, f32),
    velocity: (f32, f32),
}
impl PhysicsBody {
    fn update(&mut self, delta_time: f32) {
        self.position.0 += self.velocity.0 * delta_time;
        self.position.1 += self.velocity.1 * delta_time;
    }
}

struct PlayerView {
    sprite_id: u32,
}
impl PlayerView {
    fn draw(&self, position: (f32, f32), renderer: &mut Renderer) {
        renderer.draw_sprite(self.sprite_id, position);
    }
}

// The main Player struct is now a coordinator of components
struct Player {
    state: PlayerState,
    body: PhysicsBody,
    view: PlayerView,
}

impl Player {
    // Methods now delegate to the appropriate component
    fn take_damage(&mut self, amount: u32) {
        self.state.take_damage(amount);
    }
    fn update_physics(&mut self, delta_time: f32) {
        self.body.update(delta_time);
    }
    fn draw(&self, renderer: &mut Renderer) {
        self.view.draw(self.body.position, renderer);
    }
}


By providing such prescriptive guidance, the detector transforms from a passive critic into an active partner in the code quality improvement process. It should not just state, "This class is too large." Instead, it should offer a diagnosis tied to the specific metrics that were violated: "This class has a high LCOM4 score of 4, suggesting it contains 4 separate responsibilities. Consider using the 'Extract Class' refactoring to improve cohesion." This closes the loop between detection and remediation, empowering developers to undertake the often-daunting task of refactoring large classes in an incremental, guided, and data-driven manner.80

Conclusions

The "Large Class" anti-pattern is a pervasive and detrimental code smell that signals deep-seated issues in software design, primarily the violation of the Single Responsibility Principle. Implementing an effective, multi-language detector for this anti-pattern is a complex but achievable engineering task that requires moving beyond simplistic heuristics like line counting.
This report has established that a state-of-the-art detector must be built upon a multi-metric foundation, synthesizing insights from three distinct categories of analysis: size (LLOC, NOM, NOF), complexity (Cyclomatic, Cognitive), and structure (LCOM4, CBO). The true power of the detector emerges not from any single metric but from the interplay between them. This allows the system to build a nuanced profile of a class, distinguishing a genuinely problematic God Object from a benign but large data structure, thereby minimizing the false positives that erode developer trust.
For a multi-language tool, the concept of a "class" must be abstracted. The implementation must first identify language-specific constructs—Python class, JavaScript class and functional components, Rust struct/impl pairs—and aggregate them into a common "analyzable entity" before applying the universal metric suite. This architectural separation of language-specific parsing from language-agnostic analysis is key to creating a scalable and maintainable tool.
The success and adoption of such a detector hinge on two critical factors: configurability and intelligence in handling edge cases. The definition of "too large" is inherently context-dependent, varying by project age, domain, and team standards. Therefore, the detector must be highly tunable, allowing users to adjust thresholds and severity calculations to fit their specific needs. Furthermore, it must proactively identify and suppress common sources of false positives, such as generated code, framework-specific base classes, and common design patterns like DTOs and Builders. Investing engineering effort in this suppression logic is paramount for user acceptance.
Finally, the detector's ultimate value lies in its ability to provide actionable guidance. By linking specific metric violations to corresponding refactoring patterns—such as recommending "Extract Class" for high LCOM4 or "Extract Subclass" for high complexity—the tool transforms from a mere problem-finder into an active participant in improving code quality. It provides a data-driven, incremental path for developers to tackle technical debt, making the daunting task of refactoring large classes more manageable and guided. By adhering to these principles, it is possible to build a Large Class detector that is not only technically sophisticated but also a genuinely valuable asset in the modern software development lifecycle.
Works cited
codesignal.com, accessed July 6, 2025, https://codesignal.com/learn/courses/refactoring-by-leveraging-your-tests-with-csharp-xunit/lessons/large-class-extract-class#:~:text=The%20%22Large%20Class%22%20smell%20is,becoming%20excessively%20large%20and%20complex.
Large Class - Code Smells, accessed July 6, 2025, https://code-smells.com/bloaters/large-class
Large Class - Refactoring.Guru, accessed July 6, 2025, https://refactoring.guru/smells/large-class
Large Class: Extract Class | CodeSignal Learn, accessed July 6, 2025, https://codesignal.com/learn/courses/refactoring-by-leveraging-your-tests-with-csharp-xunit/lessons/large-class-extract-class
God object - Wikipedia, accessed July 6, 2025, https://en.wikipedia.org/wiki/God_object
What Is a God Class and Why Should We Avoid It? | LinearB Blog, accessed July 6, 2025, https://linearb.io/blog/what-is-a-god-class
The “God Object” Anti-Pattern in Software Architecture. | by Dilanka Muthukumarana, accessed July 6, 2025, https://dilankam.medium.com/the-god-object-anti-pattern-in-software-architecture-b2b7782d6997
Avoiding Software Bottlenecks: Understanding the 'God Object' Anti-Pattern - HackerNoon, accessed July 6, 2025, https://hackernoon.com/avoiding-software-bottlenecks-understanding-the-god-object-anti-pattern
The Blob | DevIQ, accessed July 6, 2025, https://deviq.com/antipatterns/blob/
The Blob, accessed July 6, 2025, https://home.cs.colorado.edu/~ralex/courses/csci4830/documents/theBlob.pdf
A Dev Anti Pattern. The Blob represents a situation where a… | by Sangeevan Siventhirarajah, accessed July 6, 2025, https://sangeevan.medium.com/the-blob-a-dev-anti-pattern-5f72ca0ccad4
BLOB antipattern - Blog of Vincent VAUBAN, accessed July 6, 2025, https://blog.vvauban.com/blog/blob-antipattern
Code Smell - Large Classes - San Tuon, accessed July 6, 2025, https://www.santuon.com/code-smell-large-classes/
Part 2: Single Responsibility Principle (SRP) | by Bhanu Kumar | Code and Concepts, accessed July 6, 2025, https://medium.com/code-and-concepts/part-2-single-responsibility-principle-srp-1cc326e7ed10
Understanding Lack of Cohesion in Methods - Number Analytics, accessed July 6, 2025, https://www.numberanalytics.com/blog/ultimate-guide-lack-cohesion-methods-software-metrics
Does this class design violate the single responsibility principle?, accessed July 6, 2025, https://softwareengineering.stackexchange.com/questions/306801/does-this-class-design-violate-the-single-responsibility-principle
Does Single Responsibility Principle mean that each class should only have one method?, accessed July 6, 2025, https://www.reddit.com/r/learnprogramming/comments/15opwvp/does_single_responsibility_principle_mean_that/
Analyzing Software Code — Lines of Code | by Vineet Sharma - Medium, accessed July 6, 2025, https://mvineetsharma.medium.com/analyzing-software-code-lines-of-code-9a49742b5a6c
Advances in SonarQube's Bug Detection | Sonar, accessed July 6, 2025, https://www.sonarsource.com/blog/sonarqube-bug-detection-advances/
Static Code Analysis Approaches for Handling Code Quality - Launchable, accessed July 6, 2025, https://www.launchableinc.com/blog/static-code-analysis-approaches-for-handling-code-quality/
Source lines of code - Wikipedia, accessed July 6, 2025, https://en.wikipedia.org/wiki/Source_lines_of_code
Why lines of code are a bad measure of developer productivity - DX, accessed July 6, 2025, https://getdx.com/blog/lines-of-code/
Project Metrics Help - Lines of code metrics (LOC) - Aivosto, accessed July 6, 2025, https://www.aivosto.com/project/help/pm-loc.html
Update S104: "Too many lines in a file" should only count lines of code · Issue #396 · SonarSource/sonar-dotnet - GitHub, accessed July 6, 2025, https://github.com/SonarSource/sonar-dotnet/issues/396
Metric Descriptions, accessed July 6, 2025, https://gromit.iiar.pwr.wroc.pl/p_inf/ckjm/metric.html
Metric Descriptions - Diomidis Spinellis home page, accessed July 6, 2025, https://www.spinellis.gr/sw/ckjm/doc/metric.html
Metrics for Software Components in Object Oriented Environments - International Journal of Scientific Research in Computer Science and Engineering, accessed July 6, 2025, https://ijsrcse.isroset.org/index.php/j/article/download/16/16/32
Standard Checkers - Pylint 3.3.7 documentation, accessed July 6, 2025, https://pylint.readthedocs.io/en/stable/user_guide/configuration/all-options.html
Cyclomatic complexity - IBM, accessed July 6, 2025, https://www.ibm.com/docs/en/raa/6.1.0?topic=metrics-cyclomatic-complexity
jellyfish.co, accessed July 6, 2025, https://jellyfish.co/library/cyclomatic-complexity/#:~:text=Basic%20cyclomatic%20complexity%20formula%3A%20Cyclomatic,of%20all%20predicate%20nodes%20%2B%201.
Understanding measures and metrics | SonarQube Server Documentation, accessed July 6, 2025, https://docs.sonarsource.com/sonarqube-server/10.8/user-guide/code-metrics/metrics-definition/
What is Cyclomatic Complexity? Definition Guide & Examples - Sonar, accessed July 6, 2025, https://www.sonarsource.com/learn/cyclomatic-complexity/
Cyclomatic complexity - Wikipedia, accessed July 6, 2025, https://en.wikipedia.org/wiki/Cyclomatic_complexity
Code metrics - Cyclomatic complexity - Visual Studio (Windows) | Microsoft Learn, accessed July 6, 2025, https://learn.microsoft.com/en-us/visualstudio/code-quality/code-metrics-cyclomatic-complexity?view=vs-2022
Cyclomatic complexity: Definition and limits in understanding code quality - GetDX, accessed July 6, 2025, https://getdx.com/blog/cyclomatic-complexity/
sonar-python/python-checks/src/main/resources/org/sonar/l10n/py/rules/python/S3776.html at master · SonarSource/sonar-python - GitHub, accessed July 6, 2025, https://github.com/SonarSource/sonar-python/blob/master/python-checks/src/main/resources/org/sonar/l10n/py/rules/python/S3776.html
How to Identify and Reduce Cognitive Complexity in Your Codebase - Axify, accessed July 6, 2025, https://axify.io/blog/cognitive-complexity
SonarQube Cognitive Complexity. Sometimes there are too many conditions… | by Aleksei Jegorov | Dev Genius, accessed July 6, 2025, https://blog.devgenius.io/sonarqube-cognitive-complexity-265640dbad3e
Lack of Cohesion of Methods: What Is This And Why Should You Care? - NDepend Blog, accessed July 6, 2025, https://blog.ndepend.com/lack-of-cohesion-methods/
java - What is the recommended size of a class? - Stack Overflow, accessed July 6, 2025, https://stackoverflow.com/questions/16351566/what-is-the-recommended-size-of-a-class
Lack of Cohesion in Methods (LCOM4) | objectscriptQuality, accessed July 6, 2025, https://objectscriptquality.com/docs/metrics/lack-cohesion-methods-lcom4
Project Metrics Help - Cohesion metrics - Aivosto, accessed July 6, 2025, https://www.aivosto.com/project/help/pm-oo-cohesion.html
Comparison of Various Lacks of Cohesion Metrics, accessed July 6, 2025, https://www.ijeat.org/wp-content/uploads/papers/v2i3/C1085022313.pdf
Predicting Software Cohesion Metrics with Machine Learning Techniques - MDPI, accessed July 6, 2025, https://www.mdpi.com/2076-3417/13/6/3722
A Practical Look at the Lack of Cohesion in Methods Metric - UAH, accessed July 6, 2025, https://www.cs.uah.edu/~letzkorn/joop.pdf
A Pedagogical Evaluation and Discussion about the Lack of Cohesion in Method (LCOM) Metric Using Field Experiment. - arXiv, accessed July 6, 2025, https://arxiv.org/pdf/1004.3277
Lack of Cohesion in Methods and the LCOM4 metric - Sonar Code Quality Testing Essentials [Book] - O'Reilly Media, accessed July 6, 2025, https://www.oreilly.com/library/view/sonar-code-quality/9781849517867/ch09s05.html
Coupling Between Objects (CBO) – Code Quality Docs, accessed July 6, 2025, https://docs.embold.io/coupling-between-objects/
Code metrics - Class coupling - Visual Studio (Windows) | Microsoft Learn, accessed July 6, 2025, https://learn.microsoft.com/en-us/visualstudio/code-quality/code-metrics-class-coupling?view=vs-2022
Mastering Coupling Between Objects - Number Analytics, accessed July 6, 2025, https://www.numberanalytics.com/blog/mastering-coupling-between-objects
Coupling Between Object classes (CBO) | objectscriptQuality, accessed July 6, 2025, https://objectscriptquality.com/docs/metrics/coupling-between-object-classes-cbo
Sidebar 3 - WMC, CBO, RFC, LCOM, DIT, NOC - 'The Chidamber and Kemerer Metrics' - Virtual Machinery, accessed July 6, 2025, http://www.virtualmachinery.com/sidebar3.htm
max-lines - ESLint - Pluggable JavaScript Linter, accessed July 6, 2025, https://eslint.org/docs/latest/rules/max-lines
python, how to detect attributes or functions that defined in class but never called by the instance of class? - Stack Overflow, accessed July 6, 2025, https://stackoverflow.com/questions/35592102/python-how-to-detect-attributes-or-functions-that-defined-in-class-but-never-ca
Pylint configuration - Codeac, accessed July 6, 2025, https://www.codeac.io/documentation/pylint-configuration.html
Diving into Tree-Sitter: Parsing Code with Python Like a Pro - DEV Community, accessed July 6, 2025, https://dev.to/shrsv/diving-into-tree-sitter-parsing-code-with-python-like-a-pro-17h8
Refactoring Legacy JavaScript Code to Use Classes | Hacker News, accessed July 6, 2025, https://news.ycombinator.com/item?id=13808134
Clean Up Your TypeScript Classes: Finding and Removing Unused Static Members with ESLint | by Mohamed Said Ibrahim | Medium, accessed July 6, 2025, https://medium.com/@mohamedsaidibrahim/clean-up-your-typescript-classes-finding-and-removing-unused-static-members-with-eslint-8c1a8c74f3e5
eslint - Detecting class instance undefined method name in javascript - Stack Overflow, accessed July 6, 2025, https://stackoverflow.com/questions/75343051/detecting-class-instance-undefined-method-name-in-javascript
5 React Component Best Practices You Should Know | by Caelin Sutch, accessed July 6, 2025, https://betterprogramming.pub/best-practices-i-wish-all-react-developers-knew-part-1-ff6cdee0666a
Structuring React Components: Best Practices For Code Organization - Nile Bits, accessed July 6, 2025, https://www.nilebits.com/blog/2024/04/structuring-react-components/
complexity - ESLint - Pluggable JavaScript linter, accessed July 6, 2025, https://archive.eslint.org/docs/rules/complexity
complexity - ESLint - Pluggable JavaScript Linter, accessed July 6, 2025, https://eslint.org/docs/latest/rules/complexity
eslint-plugin-sonarjs - NPM, accessed July 6, 2025, https://www.npmjs.com/package/eslint-plugin-sonarjs
S3776 - Reason for the current default value of 15 - SonarQube for IDE - Sonar Community, accessed July 6, 2025, https://community.sonarsource.com/t/s3776-reason-for-the-current-default-value-of-15/127103
SonarSource/eslint-plugin-sonarjs: SonarJS rules for ESLint - GitHub, accessed July 6, 2025, https://github.com/SonarSource/eslint-plugin-sonarjs
ESLint Complexity of Code | eslintcc, accessed July 6, 2025, https://eslintcc.github.io/
Automated Refactoring of Rust Programs - Alex Potanin, accessed July 6, 2025, https://potanin.github.io/files/SamCameronPotaninACSC2017.pdf
Clippy Lints - GitHub Pages, accessed July 6, 2025, https://rust-lang.github.io/rust-clippy/master/
Clippy, accessed July 6, 2025, https://rust-lang.github.io/rust-clippy/v0.0.212/
hydro-project/rust-sitter: Use Tree Sitter to parse your own languages in Rust - GitHub, accessed July 6, 2025, https://github.com/hydro-project/rust-sitter
How to Use Static Code Analysis for Better Code - PixelFreeStudio Blog, accessed July 6, 2025, https://blog.pixelfreestudio.com/how-to-use-static-code-analysis-for-better-code/
Code Analysis on a Code Generator Generated File - How to Suppress Warnings?, accessed July 6, 2025, https://stackoverflow.com/questions/2221881/code-analysis-on-a-code-generator-generated-file-how-to-suppress-warnings
Advanced Class Cycle Detection for Java - Sonar Community, accessed July 6, 2025, https://community.sonarsource.com/t/advanced-class-cycle-detection-for-java/127500
ESLint - Visual Studio Marketplace, accessed July 6, 2025, https://marketplace.visualstudio.com/items?itemName=dbaeumer.vscode-eslint
Custom Rules - ESLint - Pluggable JavaScript Linter, accessed July 6, 2025, https://eslint.org/docs/latest/extend/custom-rules
Extract Class - Refactoring.Guru, accessed July 6, 2025, https://sourcemaking.com/refactoring/extract-class
Study with me for Code Refactoring ( Story 4— Large Class of Bloaters) | by Thaw Zin Toe, accessed July 6, 2025, https://thawzintoe.medium.com/study-with-me-for-code-refactoring-story4-large-class-of-bloaters-24e4aa472020?source=rss-------1
How to Refactor Messy JavaScript Projects - Chris Courses, accessed July 6, 2025, https://chriscourses.com/blog/how-to-refactor-messy-javascript-projects
Better refactoring large classes. Practicing Ugly Trivia Game Kata —… | by Carlo Maffi, accessed July 6, 2025, https://medium.com/@carlocarlen/better-refactoring-large-classes-7c672fda66de
How do you refactor a God class? - Stack Overflow, accessed July 6, 2025, https://stackoverflow.com/questions/14870377/how-do-you-refactor-a-god-class
