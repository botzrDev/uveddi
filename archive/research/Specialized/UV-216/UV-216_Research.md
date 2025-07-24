
Enhancing the Long Methods Detector: A Technical Deep Dive and Implementation Guide for UV-216


1.0 Executive Summary


1.1 Project Mandate (UV-216)

This report provides a comprehensive technical analysis and a set of actionable recommendations for the implementation of the Long Methods Detector within the JIRA_PLATFORM project. The primary objective is to develop a reliable, scalable, and multi-language detector that identifies and flags excessively long methods, a well-known anti-pattern that degrades code quality, increases complexity, and hinders maintainability.

1.2 Core Problem

The current implementation of the Long Methods Detector is incomplete and non-functional. It lacks the fundamental capability to perform Abstract Syntax Tree (AST) analysis, which is essential for accurately identifying method boundaries and calculating their length. Consequently, it fails to support key languages used within the project—namely Rust, Python, and JavaScript—and lacks a mechanism for user-configurable thresholds. This results in failing tests and renders the tool ineffective at enforcing code quality standards across diverse codebases.

1.3 Proposed Solution Architecture

The recommended solution is to build the detector upon the robust and high-performance Tree-sitter parsing framework, a dependency enabled by the completion of task UV-212. This architecture leverages Tree-sitter's powerful query engine to declaratively identify method and function constructs across different languages. By parsing source code into a concrete syntax tree, the detector can precisely analyze the code's structure, independent of formatting or stylistic variations. The core of the solution involves developing language-specific queries for Rust, Python, and JavaScript. The detector will be engineered with a flexible, hierarchical configuration system, allowing development teams to customize detection thresholds on a per-project or per-language basis to align with their specific standards.

1.4 Key Recommendations Synopsis

This report outlines a detailed implementation strategy with the following key recommendations:
Adopt Tree-sitter Queries: Utilize Tree-sitter's declarative query language as the primary mechanism for all method and function identification. This approach is more robust, maintainable, and scalable than manual AST traversal.
Implement Multi-faceted Length Calculation: To provide a comprehensive view of method size, the detector should calculate length using three distinct strategies: physical line count, logical lines of code (excluding comments and blank lines), and total statement count.
Incorporate Advanced Complexity Metrics: Move beyond simple length metrics by implementing both Cyclomatic Complexity and Cognitive Complexity. These metrics provide deeper, more nuanced insights into the testability and understandability of code, respectively.
Design a Hierarchical Configuration System: Implement a project-level configuration file (e.g., .jira_platform_linter.toml) that allows global and language-specific settings for all thresholds and detector options, drawing inspiration from industry-standard tools like ESLint and Clippy.
Establish a Comprehensive Test Suite: Create a dedicated suite of test files for each supported language, covering a wide range of cases including short methods, long methods, complex methods, and language-specific edge cases to ensure detector accuracy and prevent future regressions.

1.5 Expected Business Impact

A fully implemented and robust Long Methods Detector will yield significant improvements in software quality and development efficiency. By systematically identifying and encouraging the refactoring of overly complex methods, the tool will directly contribute to reducing technical debt. This leads to a more maintainable codebase, which in turn improves developer productivity by making code easier to comprehend, modify, and debug. Furthermore, by flagging high-complexity areas that are statistically more prone to defects, the detector serves as a proactive risk mitigation tool, ultimately lowering the cost of maintenance and improving the stability of the final product.

2.0 Architectural Foundation: Leveraging Tree-sitter for AST-Based Analysis


2.1 Introduction to Tree-sitter

The foundational technology for the Long Methods Detector will be Tree-sitter, a powerful parser generator and incremental parsing library.1 The selection of Tree-sitter is a strategic architectural decision driven by its core design principles, which are exceptionally well-suited for building scalable and performant static analysis tools. Its key advantages include:
Generality: Tree-sitter is designed to parse any programming language, with a vast ecosystem of community-maintained grammars available for immediate use.2 This inherent flexibility is crucial for the JIRA_PLATFORM project, which encompasses multiple languages.
Speed and Efficiency: Written in pure C, Tree-sitter is fast enough to parse code on every keystroke, a critical feature for potential real-time analysis within an IDE.3 Its incremental parsing capability allows it to efficiently update the syntax tree when source code is edited, re-parsing only the changed portions of the file.6
Robustness: It is resilient to syntax errors, capable of producing a useful syntax tree even for incomplete or erroneous code.1 This is essential for a static analysis tool that must operate on code that is actively being developed.
Dependency-Free Runtime: The core parsing library has no external dependencies, making it straightforward to embed within larger applications like the JIRA_PLATFORM's analysis framework.1
The adoption of Tree-sitter is not merely a solution for the immediate task of supporting three languages; it establishes a unified and extensible framework for all future AST-based analysis. The core logic for parsing, querying, and analyzing nodes can be implemented in a language-agnostic manner. Consequently, extending the detector to support a new language in the future (e.g., Go, C++, Java) becomes a low-effort task, primarily involving the addition of a new grammar file and the development of a language-specific query string. This approach ensures the detector is not only reliable but also highly scalable, significantly reducing the cost and complexity of future enhancements.

2.2 Concrete vs. Abstract Syntax Trees (CST vs. AST)

To effectively utilize Tree-sitter, it is important to understand the distinction between a Concrete Syntax Tree (CST) and an Abstract Syntax Tree (AST). A CST, also known as a parse tree, is an exact and unambiguous representation of the source code, including all syntactic details like parentheses, commas, and whitespace.2 Tree-sitter initially generates a CST.
An AST, in contrast, is an abstraction of the CST that removes syntactically irrelevant details to represent the code's fundamental structure and semantic meaning.2 For example, the expressions
(a + b) + c and a + (b + c) would have different CSTs due to the parentheses but could be represented by the same AST because they are semantically equivalent. Our detector operates on the principles of an AST, as it is concerned with structural elements like function definitions and their bodies, not with superficial syntax.

2.3 The Power of the Tree-sitter Query Engine

A cornerstone of the proposed architecture is Tree-sitter's powerful query engine. This engine provides a declarative, Lisp-like language for matching patterns within a syntax tree.10 Queries are composed of S-expressions that describe a path through the tree, targeting specific node types and their relationships.11 This mechanism is vastly superior to manual, imperative AST traversal for several reasons:
Maintainability: Queries are concise and readable, making the detection logic self-documenting and easier to maintain compared to complex, nested loops and conditional statements required for manual traversal.13
Accuracy: Queries allow for precise targeting of nodes, including specifying named fields (e.g., the name or body of a function), which reduces the likelihood of incorrect node selection.10
Robustness: The query engine abstracts away the complexities of tree traversal, minimizing the risk of common programming errors such as off-by-one errors or incorrect handling of tree depth.
By using the query engine, we can define precise patterns to capture all function and method definitions for each target language, forming the basis of our detection logic.

2.4 Integration with the JIRA_PLATFORM Project

The implementation of the Long Methods Detector is contingent upon the successful completion of task UV-212, which enables the core Tree-sitter feature within the platform. The detector's logic will be housed in the specified module, src/analysis/detectors/anti_patterns/long_methods.rs. This Rust module will use the official tree-sitter crate, which provides the necessary bindings to load language grammars, parse source code into a Tree object, compile query strings into Query objects, and execute those queries against the tree to capture matching nodes.6

3.0 Core Implementation: Multi-Language Method Detection and Length Calculation


3.1 A Unified Approach to Method Length Calculation

The core of the detector relies on accurately identifying method boundaries and calculating their length. Tree-sitter provides a robust foundation for this by exposing detailed positional information for every node in the syntax tree. Each node contains start_point and end_point properties, which are (row, column) tuples representing the node's position in the source file.7 This allows for a precise, syntax-aware calculation of length that is resilient to variations in code formatting, a significant advantage over fragile text-based or regex approaches.17
To provide a comprehensive and configurable analysis, the detector will implement three distinct length calculation strategies:
Physical Lines: This is the most straightforward metric, calculated directly from the node's positional data: length = node.end_point().row - node.start_point().row + 1. It represents the total number of lines spanned by the method body, including comments and blank lines.
Logical Lines of Code (LLOC): A more refined metric that aims to measure only the executable code. This is calculated by first finding the physical line count and then subtracting the number of lines that contain only comments or whitespace. This requires a secondary query or traversal within the method body to identify and count comment nodes and blank lines.13
Statement Count: This metric measures the number of distinct statements within a method's body. It is calculated by traversing the direct children of a method's body node and counting nodes that represent executable statements (e.g., expression_statement, if_statement, return_statement). This metric is the most abstract and is largely independent of code formatting styles.

3.2 Rust Method Identification and Analysis

Grammar Analysis: The official tree-sitter-rust grammar is robust and well-defined.18 Analysis of the grammar and its S-expression output reveals that the primary node type for both standalone functions and methods within an
impl block is function_item.6 This provides a single, consistent target for our query.
Tree-sitter Query for Rust: A simple yet powerful query can reliably capture all function and method definitions. The name field captures the function's identifier, and the body field captures the block of code to be analyzed.

Scheme


(function_item
  name: (identifier) @function.name
  body: (block) @function.body)


Implementation in long_methods.rs: The detector will load the tree_sitter_rust::LANGUAGE grammar 6, compile the query above, and execute it against the parsed AST of a Rust source file. For each match found, the code will extract the node captured by
@function.body and apply the length calculation strategies defined in section 3.1.

3.3 Python Method Identification and Analysis

Grammar Analysis: The tree-sitter-python grammar uses the function_definition node type for all function definitions, including methods within classes.21 This consistency simplifies the detection logic. Numerous examples confirm that querying for
function_definition is the standard approach for extracting functions from Python code.7 The grammar provides convenient named fields for
name and body (which is a block node).
Tree-sitter Query for Python: The query for Python is structurally similar to that for Rust, reflecting the consistency of the respective grammars.

Scheme


(function_definition
  name: (identifier) @function.name
  body: (block) @function.body)


Implementation Notes: The implementation will mirror the Rust logic, substituting the Python grammar (tree_sitter_python::language()).22 A key advantage of using Tree-sitter is that its external scanner for Python correctly handles indentation-based scoping, meaning our detector does not require any special logic to interpret Python's block structure.23

3.4 JavaScript Method Identification and Analysis

Grammar Analysis: JavaScript's syntactic flexibility presents a greater challenge, as functions can be defined in multiple ways. The tree-sitter-javascript grammar reflects this diversity with several distinct node types that must be targeted to ensure comprehensive coverage.24 These include:
function_declaration: For standard named functions (function foo() {}).10
method_definition: For methods inside a class body (class C { foo() {} }).26
arrow_function: For arrow functions, typically assigned to a variable (const foo = () => {}).
function_expression: For anonymous functions assigned to variables (const foo = function() {}).26
Tree-sitter Query for JavaScript: To be robust, the detector must use a multi-pattern query that captures all of these forms. Each pattern targets a specific function syntax and uses captures to consistently identify the function's name and body.

Scheme


; Matches: function foo() {}
(function_declaration
  name: (identifier) @function.name
  body: (statement_block) @function.body)

; Matches: class MyClass { foo() {} }
(method_definition
  name: (property_identifier) @function.name
  body: (statement_block) @function.body)

; Matches: const foo = () => {}
(variable_declarator
  name: (identifier) @function.name
  value: (arrow_function
    body: [(statement_block) (expression)] @function.body))

; Matches: const foo = function() {}
(variable_declarator
  name: (identifier) @function.name
  value: (function_expression
    body: (statement_block) @function.body))


Implementation Notes: This unified query ensures all common function definitions are detected. A special consideration for JavaScript is that the body of an arrow_function can be either a statement_block (for multi-line functions) or a single expression (for concise, single-line functions). The length calculation logic must correctly handle both cases, treating a single expression body as a one-line method.

3.5 Summary of Language-Specific AST Nodes

The analysis of each language grammar reveals distinct node types and query strategies required for accurate method identification. The following table summarizes these findings, providing a clear reference for implementation and future maintenance.
Table 1: Function/Method AST Node Types by Language
Language
Primary Node Types
Query Strategy
Rust
function_item
Single pattern targeting function_item.
Python
function_definition
Single pattern targeting function_definition.
JavaScript
function_declaration, method_definition, arrow_function, function_expression
Multi-pattern query to capture all function definition forms.

This consolidation highlights the relative simplicity of parsing Rust and Python compared to the syntactic variety of JavaScript. It underscores the necessity of the multi-pattern query for JavaScript to meet the acceptance criteria of robust detection.

4.0 Advanced Metrics for Deeper Code Insight


4.1 Rationale for Advanced Metrics

While method length, measured in lines or statements, is a valuable first-level heuristic for identifying potential code smells, it is an incomplete measure of complexity.9 A long method consisting of simple, sequential operations may be far more maintainable than a short but convoluted method with deep nesting and complex conditional logic. To fulfill the requirement for "robust metrics calculation," the detector must incorporate more sophisticated analysis that quantifies the intrinsic complexity of a method's control flow and cognitive load. This report recommends the implementation of two industry-standard metrics: Cyclomatic Complexity and Cognitive Complexity.

4.2 Calculating Cyclomatic Complexity from the AST

Definition: Cyclomatic Complexity is a quantitative measure of the number of linearly independent paths through a program's source code.28 It is calculated using the formula
M=D+1, where D is the number of decision points in the code.29 A higher cyclomatic complexity score indicates more complex branching logic, which directly correlates with increased difficulty in testing and a higher probability of defects.28
Implementation Strategy: The calculation can be performed efficiently using the AST. After isolating a method's body with the queries from Section 3.0, a second query or a targeted traversal is executed to count all nodes that represent a decision point. The total count of these nodes, plus one, yields the cyclomatic complexity score. This approach is demonstrated by existing tools that use Tree-sitter for this purpose.32 The specific decision-point node types for each language are as follows:
Rust: if_expression, while_expression, for_expression, match_arm (each arm after the first contributes +1), && and || operators within conditions.
Python: if_statement, for_statement, while_statement, except_clause, and and or operators.
JavaScript: if_statement, for_statement, while_statement, switch_case, catch_clause, ternary expressions (?), logical operators (&&, ||), and the nullish coalescing operator (??).

4.3 Implementing Cognitive Complexity Analysis

Definition: Cognitive Complexity, a metric developed by Sonar, measures the cognitive effort required for a human to understand a piece of code.33 Unlike Cyclomatic Complexity, which is rooted in graph theory for test path analysis, Cognitive Complexity specifically penalizes code structures that break the linear, top-to-bottom flow of reading and comprehension.35
Core Rules for Calculation: The score is calculated based on a set of simple, additive rules 33:
Breaks in Linear Flow (+1): An increment of +1 is applied for each structure that diverts the control flow. This includes if, else, switch, for, while, catch blocks, goto-like statements (break, continue), and each binary logical operator in a sequence (&&, ||).
Nesting Penalty (+N): An additional increment is applied for each level of nesting. For example, a for loop inside an if statement incurs a nesting penalty of +1, in addition to the base penalty for the for loop itself.
Structural Benefits (No Penalty): Structures that enhance readability do not increase complexity. For instance, an else if is not penalized beyond the initial if, and a switch statement is penalized only once, regardless of the number of case statements.
Implementation Strategy: Calculating Cognitive Complexity requires a stateful traversal of the AST within each method body. The algorithm proceeds as follows:
Initialize a complexity score and a nesting_level to 0.
Begin a recursive traversal of the nodes within the method body.
When a node that breaks linear flow is encountered, increment complexity by 1+nesting_level.
If that same node also introduces a new level of nesting (e.g., an if or for statement), increment nesting_level before traversing its child nodes.
After traversing the children of a nesting node, decrement nesting_level.
This approach has been successfully implemented in other Tree-sitter-based analysis tools and can be adapted for our detector.36
By implementing both Cyclomatic and Cognitive Complexity, the detector provides a far more powerful and nuanced analysis. It moves beyond a simple "length checker" to become a sophisticated refactoring tool. It can distinguish between code that is difficult to test (high cyclomatic complexity) and code that is difficult to understand (high cognitive complexity). These are related but distinct issues. A long switch statement, for example, has high cyclomatic complexity but low cognitive complexity; it is tedious to test all paths but easy to understand. Conversely, a short function with deeply nested ternary operators may have low cyclomatic complexity but be very difficult to comprehend. Providing both metrics allows the detector to offer a precise diagnosis, guiding developers toward the most appropriate refactoring strategy—whether it's simplifying branching logic or reducing nesting depth—and thereby delivering a significantly greater impact on code maintainability.

5.0 Designing a Flexible and Configurable Detector


5.1 The Need for Configurability

Static analysis tools are most effective when their rules can be tailored to the specific context, standards, and goals of a project.37 A one-size-fits-all, hardcoded threshold for what constitutes a "long method" is a brittle approach that fails to account for language-specific idioms or project-specific requirements. For example, a 100-line method might be acceptable in a generated Rust module but unacceptable in a core Python business logic file. To ensure high adoption and long-term utility, the detector must provide a robust and intuitive configuration system.39

5.2 Design Patterns from Industry-Standard Linters

The design of the configuration system will be modeled on the proven patterns established by widely-used static analysis tools such as ESLint 40, Pylint 42, and Clippy.44 These tools demonstrate a set of best practices that should be adopted:
External Configuration File: A dedicated, version-controllable file at the project root (e.g., .jira_platform_linter.toml) should house all settings. This makes configuration explicit and consistent for all team members.43
Rule-Specific Options: The configuration should allow users to enable or disable the detector entirely and to set specific options, such as the thresholds for each metric.40
Hierarchical Overrides: The system must support a hierarchy of settings. Global defaults should be established, with the ability to provide language-specific overrides. This allows for fine-grained control, accommodating the different norms and styles of each programming language.47

5.3 Proposed Configuration Schema

A TOML file is recommended for its human-readable syntax. The proposed schema is designed to be extensible, supporting the Long Methods Detector while providing a clear structure for future detectors.
Table 2: Proposed Configuration Schema (.jira_platform_linter.toml)
Section/Key
Type
Description
Example
[anti_patterns.long_method]
Table
Global settings for the long method detector.


enabled
Boolean
Toggles the entire detector on or off.
enabled = true
thresholds.lines
Integer
Default line count threshold.
thresholds.lines = 50
thresholds.statements
Integer
Default statement count threshold.
thresholds.statements = 25
thresholds.cyclomatic
Integer
Default cyclomatic complexity threshold.
thresholds.cyclomatic = 10
thresholds.cognitive
Integer
Default cognitive complexity threshold.
thresholds.cognitive = 15
options.count_comments
Boolean
Whether to include comment lines in the line count.
options.count_comments = false
options.count_blank_lines
Boolean
Whether to include blank lines in the line count.
options.count_blank_lines = false
[anti_patterns.long_method.rust]
Table
Overrides for Rust files.


thresholds.lines
Integer
Rust-specific line count threshold.
thresholds.lines = 100
[anti_patterns.long_method.python]
Table
Overrides for Python files.


thresholds.cognitive
Integer
Python-specific cognitive complexity threshold.
thresholds.cognitive = 12

This schema provides a clear blueprint for implementation. The use of nested tables (e.g., [anti_patterns.long_method.rust]) creates an intuitive hierarchy for language-specific overrides, while the options table allows for fine-tuning the behavior of the metrics themselves.

5.4 Establishing Sensible Defaults

Providing sensible default values is crucial for out-of-the-box utility. The following defaults are recommended, based on established industry standards and best practices:
Line Count: 50. This aligns with the default for ESLint's max-lines-per-function rule and is a common convention.48
Statement Count: 25. Pylint's max-statements default is 50, but a more conservative value of 25 is recommended as a starting point to encourage smaller functions.49 ESLint's default is 10, which may be too restrictive for many projects.50
Cyclomatic Complexity: 10. This is a widely accepted threshold in the software engineering community for identifying code that is becoming overly complex. While Clippy uses a more lenient default of 25, a stricter default of 10 encourages better testability from the outset.51
Cognitive Complexity: 15. This is the default threshold used by Sonar, the creators of the metric, and represents a well-researched balance point for code understandability.33

6.0 Validation, Performance, and Integration


6.1 A Strategy for Comprehensive Testing

To meet the acceptance criteria and ensure the detector is reliable, a comprehensive and automated validation suite is essential. The testing strategy should be built around a dedicated set of source code files designed to exercise every aspect of the detector's logic.
Test Data Structure: A testdata directory should be created within the detector's module, containing subdirectories for rust, python, and javascript.
Test Case Design: Each language-specific directory will contain a collection of source files covering various scenarios:
Valid Short Methods: Files containing functions that are well below all configured thresholds to ensure no false positives are generated.
Long Methods (by Lines): Files with methods that are simple and linear but exceed the line count threshold. This validates the basic length calculation.52
Complex Methods: Files with short methods that exceed the Cyclomatic or Cognitive Complexity thresholds due to deep nesting, complex conditional logic, or multiple return paths.54
Edge Cases: Files containing empty functions, functions with only comments and whitespace, and, particularly for JavaScript, files that mix multiple function definition syntaxes (e.g., standard functions, arrow functions, class methods) in a single file.55
Test Automation Logic: The unit tests will programmatically parse each file in the testdata directory, execute the detector with a known configuration, and assert that the results are correct. Assertions will verify the exact number of violations found, the line numbers reported for each violation, and the calculated metric values (e.g., line count, complexity scores).

6.2 Performance Considerations for Real-Time Analysis

A key design goal for any static analysis tool is to provide rapid feedback without disrupting the developer's workflow. Tree-sitter is architected for this exact use case, with its high-speed parsing and, most importantly, its incremental parsing capabilities.1 When a file is modified, Tree-sitter can reuse the unchanged portions of the existing syntax tree, making re-parsing extremely fast.6
The detector must be designed to leverage this feature. The core analysis function should be capable of accepting an optional existing Tree object. When integrated into an IDE or a file-watching service, the tool can maintain the AST in memory and pass the previous version to the parser upon each file change. This will ensure that analysis remains performant enough for a "lint-as-you-type" experience, providing immediate feedback to developers.

6.3 Strategies for Visualizing Long Method Reports

The utility of a static analysis tool is directly tied to how clearly and effectively it presents its findings. The primary consumer of this information is the developer, so feedback should be integrated as closely as possible into their daily workflow.
IDE Integration: The most effective form of visualization is direct integration with the developer's Integrated Development Environment (IDE).56 The detector's output should be a structured format (e.g., JSON) that can be easily consumed by IDE plugins for platforms like VS Code and JetBrains IDEs. This allows violations to be displayed as inline "squiggles" directly in the code editor, with detailed information available on hover. The violation message should be clear and actionable, for example: "Method
calculate_results is too long (85 lines > 50 threshold)" or "Method process_data has high cognitive complexity (22 > 15 threshold)."
CI/CD Integration and Reporting: For enforcing standards at the team level, the detector must be executable from the command line and integrated into the Continuous Integration/Continuous Deployment (CI/CD) pipeline.37 The command-line interface should support various output formats, including human-readable text for direct feedback in build logs and a machine-readable format like JSON or SARIF for ingestion by other tools. This structured output can be used to populate code quality dashboards in tools like CodeScene or Embold, enabling teams to track complexity metrics and technical debt over time.58

7.0 Actionable Recommendations and Implementation Roadmap

To ensure a structured and incremental delivery of the Long Methods Detector, the following phased implementation roadmap is proposed. Each phase represents a logical grouping of tasks that delivers tangible value and can be completed within a standard sprint cycle.

7.1 Phase 1: Core Framework and Rust Implementation (Sprints 1-2)

This initial phase focuses on establishing the foundational architecture of the detector and delivering a functional implementation for the first target language, Rust.
Task: Implement the core detector logic in long_methods.rs. This includes creating the main analysis struct, integrating the Tree-sitter parser, and developing the parser for the .jira_platform_linter.toml configuration file.
Task: Develop, test, and validate the Tree-sitter query for Rust to accurately identify function_item nodes.
Task: Implement the three primary method length calculation strategies: physical lines, logical lines of code, and statement count.
Task: Create the initial set of Rust test cases in the testdata directory, covering valid short methods and methods that are long by line count.
Acceptance Criteria: All Rust-related unit tests pass, and the detector correctly identifies long methods in Rust files based on configurable line count thresholds.

7.2 Phase 2: Python & JavaScript Support (Sprint 3)

This phase expands the detector's capabilities to cover the remaining target languages, leveraging the framework built in Phase 1.
Task: Develop, test, and validate the Tree-sitter queries for Python (function_definition) and JavaScript (the multi-pattern query for all function types).
Task: Integrate the tree-sitter-python and tree-sitter-javascript grammars into the detector's loading mechanism.
Task: Create comprehensive test cases for both Python and JavaScript, paying special attention to the various function definition syntaxes in JavaScript.
Acceptance Criteria: All unit tests for Python and JavaScript pass. The detector correctly identifies long methods across all three supported languages.

7.3 Phase 3: Advanced Metrics and Finalization (Sprint 4)

The final phase focuses on implementing the advanced complexity metrics and preparing the detector for deployment.
Task: Implement the stateful AST traversal logic required to calculate Cyclomatic and Cognitive Complexity for Rust, Python, and JavaScript.
Task: Extend the configuration schema and parsing logic to support thresholds for these new complexity metrics.
Task: Augment the existing test suite to include assertions for correct complexity scores on all relevant test cases.
Task: Finalize user-facing documentation, including a detailed guide on all configuration options and examples for each language.
Acceptance Criteria: All acceptance criteria outlined in the original user query (UV-216) are met. The detector is fully functional, comprehensively tested, and ready for integration into the JIRA_PLATFORM's CI/CD pipeline and developer workflows.
Works cited
tree-sitter/tree-sitter: An incremental parsing system for programming tools - GitHub, accessed July 20, 2025, https://github.com/tree-sitter/tree-sitter
TreeSitter - the holy grail of parsing source code - Symflower, accessed July 20, 2025, https://symflower.com/en/company/blog/2023/parsing-code-with-tree-sitter/
Tree-sitter: Introduction, accessed July 20, 2025, https://tree-sitter.github.io/
tree-sitter - GitHub, accessed July 20, 2025, https://github.com/tree-sitter
"Parse and analyze source codes with Tree-sitter" by Maxime Mouchet - YouTube, accessed July 20, 2025, https://www.youtube.com/watch?v=zz3A3Rv2PHk
tree-sitter - crates.io: Rust Package Registry, accessed July 20, 2025, https://crates.io/crates/tree-sitter
Diving into Tree-Sitter: Parsing Code with Python Like a Pro - DEV Community, accessed July 20, 2025, https://dev.to/shrsv/diving-into-tree-sitter-parsing-code-with-python-like-a-pro-17h8
Tree-sitter (parser generator) - Wikipedia, accessed July 20, 2025, https://en.wikipedia.org/wiki/Tree-sitter_(parser_generator)
A Beginner's Guide to Tree-sitter - DEV Community, accessed July 20, 2025, https://dev.to/shreshthgoyal/understanding-code-structure-a-beginners-guide-to-tree-sitter-3bbc
Unraveling Tree-Sitter Queries: Your Guide to Code Analysis Magic - DEV Community, accessed July 20, 2025, https://dev.to/shrsv/unraveling-tree-sitter-queries-your-guide-to-code-analysis-magic-41il
Understanding Tree-sitter Query Syntax | by Lince Mathew | Jun, 2025 | Medium, accessed July 20, 2025, https://medium.com/@linz07m/understanding-tree-sitter-query-syntax-def33e33a9d2
Basic Syntax - Tree-sitter, accessed July 20, 2025, https://tree-sitter.github.io/tree-sitter/using-parsers/queries/1-syntax.html
5 Powerful Ways to Use Tree-sitter in Your Next Project | by Istiaq Ahmed Fahad - Medium, accessed July 20, 2025, https://medium.com/@ahmedfahad04/5-powerful-ways-to-use-tree-sitter-in-your-next-project-50e17c1f7055
tree_sitter - Rust - Docs.rs, accessed July 20, 2025, https://docs.rs/tree-sitter
py-tree-sitter 0.25.0 documentation, accessed July 20, 2025, https://tree-sitter.github.io/py-tree-sitter/
How to Use Tree Sitter Queries in Python - YouTube, accessed July 20, 2025, https://www.youtube.com/watch?v=bP0zl4K_LY8
Using Tree Sitter to extract insights from your code and drive your development metrics, accessed July 20, 2025, https://colinwren.medium.com/using-tree-sitter-to-extract-insights-from-your-code-and-drive-your-development-metrics-8f52f95749d0
tree_sitter_rust - Rust - Docs.rs, accessed July 20, 2025, https://docs.rs/tree-sitter-rust
Rust grammar for tree-sitter - GitHub, accessed July 20, 2025, https://github.com/tree-sitter/tree-sitter-rust
tree-sitter/lib/binding_rust/README.md at master - GitHub, accessed July 20, 2025, https://github.com/tree-sitter/tree-sitter/blob/master/lib/binding_rust/README.md
Python grammar for tree-sitter - GitHub, accessed July 20, 2025, https://github.com/tree-sitter/tree-sitter-python
Python bindings to the Tree-sitter parsing library - GitHub, accessed July 20, 2025, https://github.com/tree-sitter/py-tree-sitter
Writing a Tree-sitter grammar, I found the UX is great! - Jacopo Farina's blog, accessed July 20, 2025, https://jacopofarina.eu/posts/writing-a-tree-sitter-grammar/
tree_sitter_javascript_sg - Rust - Docs.rs, accessed July 20, 2025, https://docs.rs/tree-sitter-javascript-sg
Javascript grammar for tree-sitter - GitHub, accessed July 20, 2025, https://github.com/tree-sitter/tree-sitter-javascript
tree-sitter-javascript/queries/locals.scm at master - GitHub, accessed July 20, 2025, https://github.com/tree-sitter/tree-sitter-javascript/blob/master/queries/locals.scm
How to get the tree structure data of class/property/method in tree-sitter? - Stack Overflow, accessed July 20, 2025, https://stackoverflow.com/questions/78861740/how-to-get-the-tree-structure-data-of-class-property-method-in-tree-sitter
What is Cyclomatic Complexity? Definition Guide & Examples - Sonar, accessed July 20, 2025, https://www.sonarsource.com/learn/cyclomatic-complexity/
furoxr/cyclomatic: A tool to calculate cyclomatic complexity - GitHub, accessed July 20, 2025, https://github.com/furoxr/cyclomatic
github.com, accessed July 20, 2025, https://github.com/furoxr/cyclomatic#:~:text=The%20cyclomatic%20complexity%20is%20equal,the%20num%20of%20decesion%20points
Cyclomatic Complexity Definition, Calculation & Examples - Jellyfish.co, accessed July 20, 2025, https://jellyfish.co/library/cyclomatic-complexity/
frite/mccabe: Calculate the cyclomatic complexity of the source code - GitHub, accessed July 20, 2025, https://github.com/frite/mccabe
Cognitive Complexity and Its Effect on the Code | Baeldung, accessed July 20, 2025, https://www.baeldung.com/java-cognitive-complexity
5 Clean Code Tips for Reducing Cognitive Complexity - Sonar, accessed July 20, 2025, https://www.sonarsource.com/blog/5-clean-code-tips-for-reducing-cognitive-complexity/
Cognitive Complexity Metric Plugin - SciTools Blog, accessed July 20, 2025, https://blog.scitools.com/cognitive-complexity-metric-plugin/
Show cognitive complexity of code in Emacs 29+ (treesit-based) - GitHub, accessed July 20, 2025, https://github.com/emacs-vs/cognitive-complexity
Static Code Analysis Best Practices for Developers - ACCELQ, accessed July 20, 2025, https://www.accelq.com/blog/static-code-analysis-best-practices/
How to Use Static Code Analysis for Better Code - PixelFreeStudio Blog, accessed July 20, 2025, https://blog.pixelfreestudio.com/how-to-use-static-code-analysis-for-better-code/
Best Practices for Using Static Analysis Tools - Parasoft, accessed July 20, 2025, https://www.parasoft.com/blog/best-practices-for-using-static-analysis-tools/
Configure Rules - ESLint - Pluggable JavaScript Linter, accessed July 20, 2025, https://eslint.org/docs/latest/use/configure/rules
Setting up a linter to change your coding life for the better | by Amy Loftus | Medium, accessed July 20, 2025, https://medium.com/@aploftus/setting-up-a-linter-to-change-your-coding-life-for-the-better-da5925ce22c5
Pylint 3.3.7 documentation, accessed July 20, 2025, https://pylint.readthedocs.io/
How to Use Linters for Enforcing Code Standards - PixelFreeStudio Blog, accessed July 20, 2025, https://blog.pixelfreestudio.com/how-to-use-linters-for-enforcing-code-standards/
Usage - Clippy Documentation, accessed July 20, 2025, https://doc.rust-lang.org/clippy/usage.html
Configuring Clippy - Rust Documentation, accessed July 20, 2025, https://doc.rust-lang.org/clippy/configuration.html
Configuration Files - ESLint - Pluggable JavaScript Linter, accessed July 20, 2025, https://eslint.org/docs/latest/use/configure/configuration-files
Configure Linters | docs - Trunk.io, accessed July 20, 2025, https://docs.trunk.io/code-quality/linters/configure-linters
max-lines-per-function - ESLint - Pluggable JavaScript Linter, accessed July 20, 2025, https://eslint.org/docs/latest/rules/max-lines-per-function
too-many-statements / R0915 - Pylint 4.0.0-dev0 documentation, accessed July 20, 2025, https://pylint.readthedocs.io/en/latest/messages/refactor/too-many-statements.html
max-statements - ESLint - Pluggable JavaScript Linter, accessed July 20, 2025, https://eslint.org/docs/latest/rules/max-statements
Clippy, accessed July 20, 2025, https://rust-lang.github.io/rust-clippy/v0.0.212/
Python Code Quality: Best Practices and Tools, accessed July 20, 2025, https://realpython.com/python-code-quality/
Optimizing Long Tasks in JavaScript | by Max Hoang, accessed July 20, 2025, https://javascript.plainenglish.io/optimizing-long-tasks-in-javascript-7541aca9d997
How to Write Tests - The Rust Programming Language - Rust Documentation, accessed July 20, 2025, https://doc.rust-lang.org/book/ch11-01-writing-tests.html
Top 10 Examples of long code in Javascript - CloudDefense.AI, accessed July 20, 2025, https://www.clouddefense.ai/code/javascript/example/long
What Is Static Code Analysis? A Comprehensive Overview - Parasoft, accessed July 20, 2025, https://www.parasoft.com/learning-center/static-code-analysis-guide/
Source Code Analysis Tools - OWASP Foundation, accessed July 20, 2025, https://owasp.org/www-community/Source_Code_Analysis_Tools
5 Best Code Visualization Tools I'd Use If I Were Managing a Codebase - DesignRush, accessed July 20, 2025, https://www.designrush.com/agency/software-development/trends/code-visualization-tools
