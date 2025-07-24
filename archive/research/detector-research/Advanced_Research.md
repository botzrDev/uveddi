
Advanced Detector Implementation for the Uveddi Platform


Introduction: Principles of Advanced Static Analysis for a Polyglot Platform

The modern software development lifecycle (SDLC) increasingly relies on automated tools to ensure code quality, security, and maintainability. Static code analysis, in particular, has emerged as an indispensable practice, serving as a proactive quality gate that identifies potential defects and vulnerabilities during development, long before they can escalate into costly production issues.1 By automatically scanning source code for common pitfalls such as memory leaks, security flaws, and architectural anti-patterns, these tools empower developers to remediate problems when the cost and effort are at their lowest.
A platform like Uveddi, with its cross-language capabilities, is uniquely positioned to offer a holistic view of code health across a diverse technology stack. While language-specific linters like Pylint for Python or ESLint for JavaScript are powerful, they operate in silos.3 Uveddi's mission extends beyond replicating these individual tools; it aims to provide a unified quality dashboard that enforces consistent standards across a polyglot codebase, offering insights that are greater than the sum of their parts. This is analogous to specialized tools like MLScent, which provides integrated analysis across machine learning frameworks, but applied to general-purpose software development.5
This report provides a comprehensive, actionable blueprint for implementing four critical static analysis detectors within the Uveddi platform: Magic Values, Tight Coupling, Cyclic Dependencies, and God Objects. These anti-patterns represent common but counterproductive responses to recurring design problems and are known to degrade software quality over time.6 The following sections detail the research, design, and implementation strategy for each detector, with a focus on robust, cross-language support for Rust, Python, and JavaScript.

A Unified Approach to Anti-Pattern Detection Across Rust, Python, and JavaScript

While the conceptual definitions of these anti-patterns are universal, their practical manifestations vary significantly across programming languages. These differences stem from the unique syntax, type systems, module resolution rules, and idiomatic practices of each language. A successful detection strategy must therefore be deeply aware of these nuances. For instance, what constitutes a cyclic dependency in Rust's crate-and-module system is fundamentally different from a circular import in Python, which is again different from a module cycle in JavaScript's CommonJS or ESM environments.8
The following table provides a high-level summary of these language-specific nuances, establishing a foundational context for the detailed detection strategies that follow. This comparison underscores why a one-size-fits-all algorithm is insufficient and why each detector must be carefully tailored for the language it targets.

Anti-Pattern
Rust Nuances
Python Nuances
JavaScript Nuances
Magic Values
Literals are strongly typed. The const keyword is the idiomatic solution. Hex literals are common for low-level programming and file signatures.11
Dynamically typed. Constants are a convention (e.g., UPPER_CASE names). ast.literal_eval can be used for safe evaluation of literal structures.12
const is the standard for constants. Magic strings are prevalent in object keys and event names. Enums are not native, leading to string constants as a common pattern.13
Tight Coupling
Coupling is managed through a strict module system (mod) and visibility rules (pub). The use keyword brings items into scope. Traits provide a powerful mechanism for abstraction and loose coupling.14
The import system is flexible, making it easy to create dependencies. Type hints (typing module) can formalize dependencies. Duck typing can lead to implicit, hard-to-trace coupling.16
Coupling occurs via require() (CommonJS) or import (ESM). The lack of native interfaces (prior to TypeScript) encourages coupling to concrete implementations. Prototypal inheritance creates different coupling patterns than classical inheritance.18
Cyclic Dependencies
Inter-crate cycles are forbidden by the compiler/Cargo.20 Intra-crate module cycles are allowed but are considered a code smell, as the crate is compiled as a single unit.8
Circular imports often lead to runtime ImportError because of the top-to-bottom module initialization order. Can sometimes be resolved by changing import style (e.g., import module vs. from module import name).9
Behavior differs between module systems. CommonJS may return a partially initialized object ({}), leading to undefined errors. ESM handles cycles by creating live, non-writable bindings, but values are undefined until the exporting module finishes evaluation.10
God Object
Structs with many fields and methods. Traits can be used to break up responsibilities, but a single impl block can still become bloated. The borrow checker can sometimes complicate refactoring large objects.23
A single class with an excessive number of methods and instance variables. Python's dynamic nature makes it easy to add responsibilities incrementally, leading to large classes.24
A single class or prototype-based object that handles numerous, unrelated tasks. Often seen in older codebases or where developers centralize state and logic in a single object.26


Overview of Core Techniques: AST Parsing and Dependency Graph Analysis

The foundation of modern static analysis rests on two core techniques: Abstract Syntax Tree (AST) parsing and dependency graph analysis.
First, source code, which is unstructured text, is parsed into an AST. An AST is a hierarchical, tree-like data structure that represents the code's syntactic structure in a way that is easy for a program to analyze.1 This process discards syntactically irrelevant information like whitespace and comments while preserving the code's logical structure. For Uveddi, the primary AST parsing libraries will be:
Rust: The syn crate, a powerful and widely used library for parsing Rust code into a rich syntax tree.29
Python: The built-in ast module, which provides a robust interface for parsing Python code and traversing its AST.28
JavaScript: Libraries like acorn or esprima, which are fast, ESTree-compliant parsers for JavaScript, providing a standardized AST format.32
Second, for detectors that analyze relationships between different parts of a codebase, such as Tight Coupling and Cyclic Dependencies, the ASTs are used to construct a dependency graph. In this graph, nodes represent software entities (e.g., modules, classes, functions), and directed edges represent dependencies between them (e.g., an import, a function call, a type reference).36 Once this graph is built, powerful algorithms from graph theory can be applied to detect architectural anti-patterns like cycles or to calculate metrics that quantify coupling.
The following chapters will detail how these core techniques are applied to build each of the four detectors, providing specific algorithms, configuration schemas, and actionable guidance for integration into the Uveddi platform.

Chapter 1: Implementing the Magic Value Detector


1.1. Defining Magic Values: Literals, Unexplained Constants, and Configuration Leakage

The term "magic value" refers to the anti-pattern of using unnamed numeric or string literals directly in source code.11 This practice obscures the developer's intent, increases the likelihood of subtle errors, and complicates maintenance by violating the Don't Repeat Yourself (DRY) principle.11 A literal like
1.05 in price * 1.05 or "valueKey" in cache["valueKey"] is considered "magic" because its meaning is not immediately apparent from the code itself.11 Without a descriptive name, a future developer (or even the original author) must infer its purpose from the surrounding context, a process that is both inefficient and error-prone.
The negative consequences of this anti-pattern are significant. A magic number duplicated across a codebase becomes a maintenance hazard; changing the value requires finding and updating every instance, a task ripe for human error. One might accidentally miss an instance or, worse, change a different number that happens to have the same value but a completely different semantic meaning.11 Similarly, magic strings used for configuration keys, event names, or database column names are a common source of bugs when the string is changed in one part of the system but not another.38 The preferred solution is to replace these literals with named constants, which provides a single source of truth and self-documenting code.13

1.2. Detection Strategy: AST Traversal and Literal Node Identification

The core detection strategy for magic values relies on parsing the source code into an Abstract Syntax Tree (AST) and then traversing this tree to identify all literal nodes. However, a naive approach that simply flags every literal found would be impractical, generating an overwhelming number of false positives. The true challenge and sophistication of a magic value detector lie not in finding literals, but in analyzing their context to determine if they are genuinely "magic."
The quality of a literal is not inherent to its value but is defined by its semantic role within the code. For example, the number 0 is not magic when used to initialize a loop counter (for i = 0;...), but it could be magic if it represents a specific status code (if status == 0). This contextual distinction is paramount. Therefore, the Uveddi detector must move beyond simple value-based whitelisting and analyze the parent and sibling nodes of each literal in the AST. This allows the detector to understand the literal's context—is it part of a constant declaration? An array index? A default parameter value?—and make an intelligent decision about whether to flag it. This approach mirrors the advanced configurations found in mature linters like ESLint, which provide options to ignore literals based on their usage context.39
The general algorithm is as follows:
Parse the source file into a language-specific AST.
Instantiate a visitor object that will traverse the AST.
The visitor will implement methods to handle literal nodes (numeric and string).
For each literal node encountered, the visitor will apply a set of heuristics by inspecting the node's parent and its position within the parent's structure.
If the literal is not exempted by any heuristic, it is flagged as a magic value.

1.3. Language-Specific Parsing and AST Node Identification

The implementation of the AST traversal will differ for each target language, leveraging its specific parsing tools and AST node structures.
Rust: The analysis will use the syn crate.29 A custom visitor implementing the
syn::visit::Visit trait will be created to traverse the parsed AST.29 The detector will specifically override the
visit_expr_lit method to inspect syn::ExprLit nodes. Inside this method, it will match on the lit field to handle different literal types, such as syn::Lit::Int, syn::Lit::Float, and syn::Lit::Str.
Python: The detector will utilize the built-in ast module.28 A visitor class inheriting from
ast.NodeVisitor will be implemented. The key method to override is visit_Constant, which is called for all constant literals (numbers, strings, booleans, None). The actual value can be accessed via node.value. For compatibility with older Python versions, it may also be necessary to handle deprecated nodes like ast.Num and ast.Str.28
JavaScript: The detector will use a standard ESTree-compliant parser like acorn 32 or
esprima.34 A visitor will traverse the resulting AST, looking for nodes with the type
Literal. The raw value of the literal is available in the node.value property.

1.4. Mitigating False Positives: Contextual Analysis and Idiom Recognition

To make the detector useful and prevent "warning fatigue," a robust set of heuristics must be applied to filter out legitimate uses of literals. These heuristics are based on analyzing the AST context of each literal node.
Constant and Variable Declarations: The most common way to avoid magic values is to assign them to a named constant. The detector should ignore literals that appear on the right-hand side of a constant declaration.
Rust: Ignore literals in a const item declaration (const MAX_RETRIES: u32 = 5;).
JavaScript: Ignore literals in a const variable declaration (const MAX_RETRIES = 5;).
Python: By convention, ignore literals assigned to a variable with an UPPER_CASE_WITH_UNDERSCORES name (MAX_RETRIES = 5).
Commonly Accepted Literals: A configurable whitelist of numeric values that are rarely considered "magic" should be maintained. This list should include values like 0, 1, -1, and 2, which are frequently used in loops, indexing, and arithmetic logic.11 Powers of 10 used for base conversions (e.g., 10, 100, 1000) are also common candidates for this list.
Array and Slice Indexing: Literals used as an index into an array, slice, or tuple are typically clear from context and should be ignored. This can be detected by checking if the literal's parent node is a subscript or indexing operation.39 For example,
my_array should not be flagged.
Default Values: Literals used as default values for function parameters, struct fields, or object properties are often acceptable. The detector can identify this context by checking if the literal is part of a function signature or a default property assignment.39 For example,
function set_timeout(duration = 500) should be permissible if configured.
Object and Dictionary Keys: While magic strings are often used as keys, flagging all of them can be noisy, especially in the context of JSON-like objects or data transfer objects (DTOs). A reasonable heuristic is to flag string literals used as keys if they are repeated multiple times within the same scope, suggesting they should be defined as a constant.38
Specialized Literals: Certain literal formats often have inherent meaning and should be ignored. For example, hexadecimal literals like 0xCAFEBABE are commonly used as file signatures or protocol identifiers and are not "magic" in the traditional sense.11

1.5. Configuration Schema: Whitelists, Ignored Files, and Thresholds in TOML

To provide users with fine-grained control, the detector must be highly configurable. The configuration should be defined in Uveddi's TOML file, following a clear and hierarchical structure similar to those used by Pylint and ESLint.39 This approach adheres to TOML best practices by grouping related settings under descriptive tables.42
Proposed uveddi.toml Schema for Magic Value Detector:

Ini, TOML


# Configuration for the Magic Value detector.
[detector.magic-values]
# Enable or disable the detector.
enabled = true

# Set the severity level for reported issues: "error", "warning", or "info".
severity = "warning"

# A list of file paths to ignore, supporting glob patterns.
# Useful for excluding test files, mocks, or generated code.
ignore_files = [
    "**/test_*.py",
    "**/*.spec.js",
    "src/generated/**",
]

# Configuration specific to magic numbers.
[detector.magic-values.numbers]
# A list of numeric literals that should always be allowed.
# Includes common, non-magic integers and floating-point numbers.
allowed_values = [0, 1, -1, 2, 10, 100, 1000, 3.14159, 2.71828]

# If true, ignores numbers used as array/slice/list indices (e.g., `data`).
ignore_array_indexes = true

# If true, ignores numbers used in default value assignments, such as
# function parameters or struct/class fields.
ignore_default_values = true

# Configuration specific to magic strings.
[detector.magic-values.strings]
# A list of string literals that should always be allowed.
# Useful for common strings like content types or encodings.
allowed_values = ["__main__", "utf-8", "application/json"]

# The minimum length for a string to be considered for magic value analysis.
# Shorter strings are less likely to be "magic".
min_length = 3



1.6. Error Handling & Reporting: Guiding Developers to Define Constants

Error messages are a critical part of a static analyzer's user experience. They must be clear, concise, non-blaming, and, most importantly, actionable.44 A good error message not only identifies a problem but also guides the user toward the correct solution.
Proposed Error Reporting Format:
Message: A human-readable description of the anti-pattern.
Location: The precise file, line, and column where the magic value was found.
Suggestion: A concrete code example demonstrating the recommended fix.
Example Error Report for a Magic Number:
Message: Magic number '0.07' detected. Consider defining it as a named constant to improve readability and maintainability.
Location: src/billing/invoice.py:25:15
Suggestion: For example: SALES_TAX_RATE = 0.07
Example Error Report for a Magic String:
Message: Magic string "user:session:id" detected. Using string literals for keys can lead to hard-to-find bugs. Define it as a constant.
Location: src/cache/manager.js:12:21
Suggestion: For example: const USER_SESSION_KEY = "user:session:id";

1.7. Practical Examples: Positive and Negative Cases

To validate the detector's logic and provide clear documentation, a comprehensive set of examples is necessary.

Rust Examples

Positive Detection (Flagged):
Rust
// src/finance.rs
fn calculate_interest(principal: f64) -> f64 {
    principal * 0.05 // Magic number: 0.05
}


Negative Detection (Ignored):
Rust
// src/finance.rs
const ANNUAL_INTEREST_RATE: f64 = 0.05;

fn calculate_interest(principal: f64) -> f64 {
    principal * ANNUAL_INTEREST_RATE // Correct: uses a named constant.
}

fn get_first_item(items: &[i32]) -> Option<i32> {
    items.get(0).copied() // Correct: 0 is an ignored array index.
}



Python Examples

Positive Detection (Flagged):
Python
# models/user.py
def check_password(self, password):
    if len(password) < 8: # Magic number: 8
        raise ValueError("Password is too short.")


Negative Detection (Ignored):
Python
# models/user.py
MIN_PASSWORD_LENGTH = 8

def check_password(self, password):
    if len(password) < MIN_PASSWORD_LENGTH: # Correct: uses a named constant.
        raise ValueError("Password is too short.")

def get_status_code(code=404): # Correct: default value (if configured).
    return code



JavaScript Examples

Positive Detection (Flagged):
JavaScript
// api/auth.js
function setAuthCookie(res, token) {
  res.cookie('auth_token', token, { maxAge: 3600000 }); // Magic number: 3600000
}


Negative Detection (Ignored):
JavaScript
// api/auth.js
const ONE_HOUR_IN_MS = 3600 * 1000;

function setAuthCookie(res, token) {
  res.cookie('auth_token', token, { maxAge: ONE_HOUR_IN_MS }); // Correct: uses a named constant.
}

const colors = ['red', 'green', 'blue'];
const firstColor = colors; // Correct: 0 is an ignored array index.



1.8. Integration Guidance: Registration, Testing, and Documentation

Registration: The Magic Value detector is a file-local analysis. It should be registered within Uveddi's core analysis engine to be invoked for each file being scanned. It will take a file's content (or pre-parsed AST) as input and produce a Vec<Detection> as output.
Testing: A multi-layered testing strategy is essential.
Unit Tests: Create isolated tests for the heuristic logic. For each language, feed hand-crafted AST snippets directly to the detection function to verify that it correctly identifies magic values and applies all contextual exclusion rules (e.g., constant declarations, array indexing).
Integration Tests: Create a suite of small, complete source files for each language. This suite should include files that are designed to trigger the detector (positive cases) and files that use literals correctly (negative cases). These tests will validate the entire flow, from file parsing and configuration loading to final error reporting.
Documentation: Comprehensive documentation is key for user adoption. A dedicated markdown file for the detector should be created, containing:
A clear explanation of the Magic Value anti-pattern and its negative impact.
A description of the detection strategy, including the heuristics used to reduce false positives.
A complete reference for all TOML configuration options, with clear examples for each.
Guidance on how to interpret the detector's findings and the recommended refactoring patterns.

Chapter 2: Implementing the Tight Coupling Detector


2.1. Defining and Quantifying Inter-Module Dependencies

In software engineering, coupling refers to the degree of interdependence between software modules.48
Tight coupling is an anti-pattern where modules are highly dependent on each other's concrete implementations. This creates a brittle architecture where a change in one module often triggers a cascade of required changes in other modules, a phenomenon known as the "ripple effect".48 Such systems are difficult to maintain, test, and reuse, as individual components cannot be easily isolated or replaced.18 In contrast,
loose coupling, where modules interact through stable, well-defined interfaces, is a hallmark of a well-structured and maintainable system.16
To move beyond a purely qualitative assessment, static analysis relies on quantitative metrics to measure coupling. Two of the most established and effective metrics, originating from the Chidamber and Kemerer metrics suite, are Coupling Between Objects (CBO) and Response For a Class (RFC).50
Coupling Between Objects (CBO): CBO for a class (or module) is a count of the number of other classes (or modules) to which it is coupled.51 A module
A is considered coupled to module B if A uses any members of B, such as its methods, fields, types, or constants. This includes dependencies introduced via parameters, local variables, return types, inheritance, or interface implementations.51 A high CBO value indicates that a module is entangled with many other parts of the system, reducing its independence and reusability. For Uveddi's detector, we will focus on
efferent (outgoing) coupling, which measures how many other modules a given module depends on.
Response For a Class (RFC): RFC is the size of the set of methods that can potentially be executed in response to a message received by an object of that class.50 It is calculated as the sum of the number of methods defined within the class and the number of methods called by the class that are defined in other classes. A high RFC suggests a class has complex control flow and a large potential impact on the system, making it more difficult to test and understand.50

2.2. Detection Strategy: Metric-Based Analysis with CBO and RFC

The detection strategy for tight coupling is a project-wide analysis that involves two primary steps:
Construct a Project-Wide Dependency Graph: The first step is to parse every source file in the project into its respective AST. From these ASTs, a directed graph is constructed where nodes represent the primary modular units (classes or modules, depending on the language) and directed edges represent dependencies between them. An edge from node A to node B exists if A depends on B.
Calculate Coupling Metrics for Each Node: Once the dependency graph is complete, the detector traverses the graph and calculates the CBO and RFC metrics for each node.
CBO Calculation: For a given node N, its efferent CBO is simply the out-degree of that node in the dependency graph—that is, the number of unique nodes it has an edge pointing to.
RFC Calculation: For a given class/struct node N, its RFC is calculated by:
a. Counting the number of methods defined within N.
b. Traversing the AST of each of those methods to identify all external method calls.
c. Summing the method count from (a) with the count of unique external methods called from (b).
Any node whose calculated CBO or RFC value exceeds a configurable threshold is flagged as being tightly coupled.

2.3. Language-Specific Coupling Nuances

The precise definition of a "dependency" and a "module" must be tailored to each language's semantics. A failure to account for these nuances will lead to inaccurate metric calculations.
Rust: The primary units of analysis are structs and modules (mod). A dependency is created by:
use statements that import types or functions.
Type references in function signatures (e.g., fn process(data: other_mod::Data)), struct fields, and type aliases.
Trait implementations for external types (impl MyTrait for other_mod::ExternalType).
Direct path qualification in expressions (e.g., let result = other_mod::compute();).
The syn crate provides the necessary AST nodes to identify these patterns, such as syn::ItemUse, syn::TypePath, and syn::ExprPath.29 A key distinction in Rust is its strict privacy and module system. A
use statement does not necessarily create tight coupling if it only accesses public, stable APIs. The detector should ideally differentiate between dependencies on public vs. private items, although a simpler first-pass implementation can treat all use statements as dependencies.
Python: The units of analysis are classes and files (modules). Dependencies are formed through:
import <module> and from <module> import <name> statements.
Type annotations that reference external types (e.g., def process(user: models.User)).
Inheritance (class Child(parent_module.Parent):).
Direct attribute access on imported modules or classes (e.g., db_connector.connect()).
Python's dynamic nature and weaker encapsulation mean that almost any import creates a strong potential for coupling. The ast module's visitors for ast.Import, ast.ImportFrom, ast.ClassDef, and ast.Attribute are essential for building the dependency graph.28
JavaScript: The units of analysis are classes and files (modules). The module system is fragmented, so the detector must handle both CommonJS (require()) and ES Modules (import). Dependencies are created by:
import and export statements in ESM.
require() calls in CommonJS.
Class inheritance (class Child extends Parent).
Instantiation of external classes (const instance = new OtherModule.Thing();).
An AST parser like acorn can identify ImportDeclaration nodes for ESM and CallExpression nodes where the callee is an Identifier named require for CommonJS.32

2.4. Configuration Schema: Defining Coupling Thresholds in TOML

Providing sensible, evidence-based default thresholds is crucial for the detector's usability. Users often struggle to determine what constitutes a "high" CBO or RFC value. By setting a default based on published research, the tool becomes immediately useful out of the box.

Metric
Recommended Threshold
Justification/Source
CBO
9
Recent studies have shown that an upper-limit CBO value of 9 is an efficient predictor of software failure and maintenance effort.51 Values between 1 and 4 are considered good.53
RFC
50
There is less consensus on a specific RFC threshold, but it is widely accepted that a large number of methods that can be invoked from a class increases its complexity and testing burden.50 A value of 50 is a common starting point in many analysis tools.

Proposed uveddi.toml Schema for Tight Coupling Detector:

Ini, TOML


# Configuration for the Tight Coupling detector.
[detector.tight-coupling]
# Enable or disable the detector.
enabled = true

# Set the severity level for reported issues.
severity = "warning"

# A list of file or module patterns to ignore during coupling analysis.
# Useful for excluding third-party libraries, generated code, or test utilities.
ignore = [
    "src/generated/**",
    "vendor/**",
    "tests/**",
]

# Coupling Between Objects (CBO) threshold. A module is flagged if its
# number of outgoing dependencies exceeds this value.
# Research suggests a value around 9 is a good starting point.[51]
max_cbo = 9

# Response For a Class (RFC) threshold. A class is flagged if the number of
# methods it defines plus the number of external methods it calls exceeds this value.
max_rfc = 50



2.5. Actionable Reporting: Highlighting Problematic Dependencies

A simple report like "CBO is 12" is unhelpful. The error message must be actionable by pinpointing exactly which dependencies are contributing to the high coupling score. This allows developers to immediately focus their refactoring efforts.
Proposed Error Reporting Format:
Message: A clear statement of which metric threshold was violated.
Location: The file and line number where the module or class is defined.
Details: A list of the specific modules or classes that it depends on.
Suggestion: A high-level recommendation for refactoring.
Example Error Report:
Message: Module 'ReportGenerator.js' is tightly coupled, with a CBO of 12 (threshold is 9). High coupling makes code difficult to maintain and test.
Location: src/services/ReportGenerator.js:5:1
Details: Dependencies:
Suggestion: Consider reducing dependencies by using interfaces/abstractions and dependency injection, or by communicating via an event bus.

2.6. Practical Examples: Illustrating Tightly and Loosely Coupled Modules


Python Example

Positive Detection (Tightly Coupled):
Python
# report_generator.py
from db.connector import DatabaseConnection
from writers.pdf_writer import PDFWriter
from services.email_sender import EmailSender

class ReportGenerator:
    def generate_and_send_report(self):
        db = DatabaseConnection() # Direct instantiation
        data = db.fetch_sales_data()

        writer = PDFWriter() # Direct instantiation
        pdf_path = writer.create_report(data)

        mailer = EmailSender() # Direct instantiation
        mailer.send("management@example.com", pdf_path)

This class has a high CBO because it directly depends on three concrete implementations.
Negative Detection (Loosely Coupled via Dependency Injection):
Python
# report_generator.py
from interfaces import IDatabase, IReportWriter, IEmailer

class ReportGenerator:
    def __init__(self, db: IDatabase, writer: IReportWriter, mailer: IEmailer):
        self._db = db
        self._writer = writer
        self._mailer = mailer

    def generate_and_send_report(self):
        data = self._db.fetch_sales_data()
        report_path = self._writer.create_report(data)
        self._mailer.send("management@example.com", report_path)

This refactored class has a much lower CBO. It depends only on abstractions (interfaces), not concrete classes, making it more flexible and testable.54

2.7. Integration Guidance: Registration, Testing, and Documentation

Registration: The Tight Coupling detector requires a project-wide analysis. It should be registered to run after all individual files have been parsed into ASTs. The Uveddi engine will need to provide the detector with a collection of all ASTs to enable the construction of the global dependency graph.
Testing:
Unit Tests: Test the CBO and RFC calculation logic in isolation. Create small, hand-crafted dependency graphs (e.g., as adjacency lists) and assert that the calculated metrics are correct.
Integration Tests: Create several small, multi-file projects for each language. These projects should be designed to exhibit both high and low coupling. Run the full detector on these projects and verify that the correct modules are flagged and that the reported metrics and dependency lists are accurate.
Documentation: The detector's documentation should clearly explain the concepts of coupling, CBO, and RFC. It must detail the configuration options and provide practical advice on how to interpret the results. This should include linking to articles or tutorials on common decoupling patterns like Dependency Inversion, the use of interfaces/traits, and event-driven architectures.

Chapter 3: Implementing the Cyclic Dependency Detector


3.1. The Architectural Impact of Circular References

A circular dependency, or cycle, occurs when two or more software modules depend on each other, either directly (A depends on B, and B depends on A) or indirectly (A depends on B, B depends on C, and C depends on A).36 This anti-pattern is a severe architectural flaw because it effectively merges the dependent modules into a single, tightly coupled component. Once a cycle is introduced, it becomes impossible to understand, test, or reuse any single module from the cycle in isolation.36
The consequences of circular dependencies are often immediate and disruptive. In compiled languages, they can lead to build failures. In interpreted languages like Python and JavaScript, they frequently cause runtime errors, such as ImportError or TypeError, because one module attempts to access a name from another module that has not yet been fully initialized.9 Even if a runtime error is avoided, cycles can lead to subtle bugs, unexpected behavior, and, in systems with reference-counting garbage collection, memory leaks.36 Architecturally, cycles violate the principle of a layered, directed acyclic graph (DAG) of dependencies, which is fundamental to building scalable and maintainable software.

3.2. Detection Strategy: Depth-First Search (DFS) for Cycle Detection

The most robust and efficient algorithm for detecting cycles in a directed graph is the Depth-First Search (DFS).56 The strategy involves modeling the project's inter-module dependencies as a directed graph and then using a specialized DFS traversal to check for "back edges"—edges that point from a node to one of its ancestors in the DFS tree, which indicates a cycle.
The algorithm proceeds as follows, operating on a graph where modules are vertices and imports are directed edges 56:
Initialization: Create two sets to track the state of each node:
visited: Stores all nodes that have been visited at any point during the entire analysis. This prevents redundant traversals of the same subgraphs.
recursion_stack: Stores the nodes that are currently in the path of the active DFS traversal. This is the key to detecting a cycle.
Graph Traversal: Iterate through every node (module) in the graph. If a node has not yet been added to the visited set, begin a recursive DFS traversal from that node.
Recursive DFS Step: For a given node u:
a. Mark u as visited by adding it to both the visited and recursion_stack sets.
b. For each neighbor v of u (i.e., for each module that u imports):
i. Cycle Detected: If v is already in the recursion_stack, a cycle has been found. The path from v back to itself within the stack constitutes the cycle.
ii. Continue Traversal: If v has not been visited yet, make a recursive call to the DFS function on v. If this recursive call returns true (indicating a cycle was found deeper in the graph), propagate true up the call stack.
c. Backtrack: After visiting all neighbors of u, remove u from the recursion_stack. This signifies that the traversal of the path through u is complete.
This algorithm has a time complexity of O(V+E), where V is the number of modules and E is the number of dependencies, making it highly efficient for large codebases.56

3.3. Language-Specific Import and Module Resolution Logic

The primary challenge in implementing this detector is not the graph traversal algorithm itself, but rather the construction of an accurate dependency graph that correctly models the specific import and name resolution rules of each language.
Rust: The concept of a "cycle" in Rust is nuanced.
Inter-Crate Cycles: At the package level, Cargo, Rust's build system, strictly forbids cyclic dependencies between crates. A package cannot depend on another package that, in turn, depends on the original. This is a hard build error.20 The Uveddi detector, when analyzing a Cargo workspace, should build a graph of
Cargo.toml dependencies and flag any cycles as high-severity errors. Tools like cargo-tree and workspacer-detect-circular-deps provide a precedent for this analysis.59
Intra-Crate Cycles: Within a single crate, Rust's module system (mod) allows for circular dependencies. Since the entire crate is treated as a single compilation unit, the compiler can resolve all paths before code generation.21 However, these cycles are still considered a code smell because they make the code harder to reason about and refactor.8 The detector should build a graph of
mod declarations and use statements within a crate and flag any cycles found as lower-severity warnings.
Python: The dependency graph is built from import and from... import statements found in the AST. Python's module execution model is top-to-bottom. When a circular import occurs, Python's import machinery can fail, raising an ImportError with a "partially initialized module" message because one module tries to access a name from another before the latter has finished executing.9 The detector must accurately resolve both absolute and relative imports to construct the graph. Tools like
pylint and pydeps are capable of detecting these cycles.62 An advanced feature for the Uveddi detector could be to suggest a common workaround: changing a
from module import name statement to import module, which can sometimes break a cycle by deferring the name lookup until runtime.22
JavaScript: The dependency graph must be built by parsing both CommonJS require() calls and ESM import/export statements. The behavior of cycles differs significantly between these two systems:
CommonJS: When a cycle occurs, the require() call may return an empty object ({}) if the target module has not yet finished its execution, leading to TypeError when trying to access properties of an undefined value.10
ESM: The specification defines a formal process for handling cycles. It creates live, immutable bindings for all exports before executing any code. If a module accesses an import that is part of a cycle, the value will be undefined until the exporting module's code has been evaluated.10

The detector must be able to parse both syntaxes. Precedent for this exists in tools like eslint-plugin-import and webpack's circular-dependency-plugin.64

3.4. Configuration Schema: Ignoring Known Cycles and Setting Scope

In some complex domain models, circular dependencies can be a deliberate, if not ideal, design choice. The detector must provide a mechanism to ignore known and accepted cycles to avoid generating persistent noise.
Proposed uveddi.toml Schema for Cyclic Dependency Detector:

Ini, TOML


# Configuration for the Cyclic Dependency detector.
[detector.cyclic-dependencies]
# Enable or disable the detector.
enabled = true

# Set the severity level for reported issues.
severity = "error"

# For Rust projects, specify the scope of cycle detection.
# "module": Check for intra-crate module cycles only (warning).
# "crate": Check for inter-crate dependency cycles only (error).
# "all": Check for both.
rust_scope = "all"

# A list of cycles to ignore. Each entry is an array of module paths
# representing a known dependency loop. The detector will ignore any
# cycle that is an exact match for one of these lists.
ignore_cycles = ["src/models/user.py", "src/models/order.py", "src/models/user.py"],
  ["src/api/a.js", "src/api/b.js", "src/api/c.js", "src/api/a.js"]



3.5. Actionable Reporting: Visualizing the Dependency Cycle

A simple "Cycle detected" message is insufficient. The most critical piece of information for a developer is the exact path of the cycle. The report must clearly trace the chain of imports that forms the loop.
Proposed Error Reporting Format:
Message: A concise statement indicating that a cycle was found.
Location: The error should be reported for each file involved in the cycle, providing the relevant line number of the import statement.
Details (Cycle Path): A clear, step-by-step visualization of the dependency loop.
Suggestion: A high-level recommendation for refactoring.
Example Error Report:
Message: Cyclic dependency detected involving 'user_service.py'.
Location: src/services/user_service.py:4:1
Details:
Dependency Cycle Path:
   src/services/user_service.py imports 'get_auth_context' from src/auth/context.py
   src/auth/context.py imports 'User' from src/models/user.py
   src/models/user.py imports 'log_user_event' from src/services/user_service.py (completes the cycle)


Suggestion: Break the cycle by extracting shared functionality (like 'log_user_event') into a separate utility module, or by using dependency injection to invert control.

3.6. Practical Examples: Demonstrating and Resolving Cycles


Python Example

Positive Detection (Cyclic):
Python
# a.py
from b import b_func
def a_func():
    print("Calling b_func")
    b_func()

# b.py
from a import a_func
def b_func():
    print("Calling a_func")
    a_func()


Negative Detection (Refactored):
Python
# common.py
def common_func_for_a():
    print("Common for a")

def common_func_for_b():
    print("Common for b")

# a.py
from common import common_func_for_b
def a_func():
    print("Calling common_func_for_b")
    common_func_for_b()

# b.py
from common import common_func_for_a
def b_func():
    print("Calling common_func_for_a")
    common_func_for_a()

This refactoring breaks the cycle by moving shared dependencies into a common.py module, creating a DAG where both a.py and b.py depend on common.py.8

3.7. Integration Guidance: Registration, Testing, and Documentation

Registration: This is a project-wide detector. Similar to the Tight Coupling detector, it must be invoked after all project files have been parsed, so it can operate on a complete, global dependency graph.
Testing:
Unit Tests: Thoroughly test the DFS cycle detection algorithm with a variety of hand-crafted graph structures. This should include graphs with no cycles, simple two-node cycles, complex multi-node cycles, and multiple disjoint cycles.
Integration Tests: Create small, multi-file projects for each language that are specifically designed to contain direct and indirect circular dependencies. Run the full detector on these projects to verify that the cycles are correctly identified and that the reported cycle paths are accurate.
Documentation: The documentation must explain the severe architectural problems caused by cyclic dependencies. It should describe the detection algorithm at a high level and provide a full reference for the configuration options. Crucially, it should offer detailed guidance on common refactoring strategies, such as the "Extract Module/Class" pattern and the use of Dependency Inversion.

Chapter 4: Implementing the God Object Detector


4.1. Defining the God Object: The Single Responsibility Principle Violation

A God Object, also known as a God Class or The Blob, is a design anti-pattern that describes a class or object that has accumulated an excessive number of responsibilities.23 Such an object centralizes a disproportionate amount of the system's intelligence, violating the fundamental
Single Responsibility Principle (SRP), which states that a class should have only one reason to change.26 God Objects typically exhibit a combination of tell-tale characteristics: they are large and complex, have low internal cohesion, and are tightly coupled to many other, often trivial, data classes that they manipulate.50
This concentration of functionality makes the system difficult to understand, maintain, and test. A change to one of the object's many responsibilities can have unforeseen and unintended ripple effects on its other, unrelated functions.23 The resulting code is brittle, hard to reuse, and a significant source of technical debt.70

4.2. Detection Strategy: A Hybrid Approach with LCOM, WMC, and Size Metrics

No single metric can reliably identify a God Object. A class might be large but highly cohesive (e.g., a complex parser), or small yet non-cohesive. Therefore, a robust detection strategy must employ a hybrid, heuristic-based approach that combines multiple metrics measuring different facets of the anti-pattern: size, complexity, and cohesion. This multi-faceted approach is a standard practice in advanced static analysis tools, which recognize that a confluence of symptoms is a much stronger indicator of a design flaw than any single metric in isolation.50
The proposed heuristic for the Uveddi detector is to flag a class or struct as a potential God Object if it exceeds a configurable number of thresholds across the following metrics:
High Complexity - Weighted Methods per Class (WMC): WMC is the sum of the cyclomatic complexities of all methods within a class.50 Cyclomatic complexity measures the number of linearly independent paths through a method's source code, effectively quantifying its decision logic (
if, while, for, case, etc.).71 A high WMC indicates that the class as a whole contains a large amount of complex logic and is doing too much work.
Low Cohesion - Lack of Cohesion in Methods (LCOM): LCOM measures the degree to which methods within a class are related to one another based on the instance variables they share.50 A high LCOM value suggests that the class is handling multiple, unrelated responsibilities, as its methods operate on different subsets of its data. This report recommends implementing the
LCOM4 variant, which is widely regarded as one of the most robust versions of the metric. LCOM4 calculates the number of "connected components" of methods within a class; a value greater than 1 signifies that the class can be split into multiple, more cohesive classes.73
High Access to Foreign Data (ATFD): A key behavior of a God Object is that it controls and manipulates many other simpler "data" objects.69 The ATFD metric quantifies this by counting the number of distinct external classes whose attributes are directly accessed by the methods of the class under analysis. This is a specialized form of coupling that specifically targets the data-centric control typical of God Objects.
Excessive Size - Number of Methods (NOM) and Number of Attributes (NAtt): While not sufficient on their own, raw size metrics are strong supplementary indicators. A class with a very high number of methods or attributes is a prime candidate for closer inspection.50

4.3. Language-Specific Metric Calculation for Structs and Classes

The calculation of these metrics requires a detailed analysis of each class or struct's AST.
WMC (Weighted Methods per Class):
For each class/struct, identify all its defined methods.
For each method, traverse its AST to build a Control Flow Graph (CFG).
Calculate the method's Cyclomatic Complexity using the formula M=E−N+2P, where E is the number of edges, N is the number of nodes in the CFG, and P is the number of connected components (typically 1 for a single method).71 A simpler approximation is
1+(number of decision points).
The WMC for the class is the sum of the complexities of all its methods.
LCOM4 (Lack of Cohesion in Methods v4):
For a given class, create a graph where each node represents a method defined within that class.
For each method, identify the set of instance variables (fields) it accesses and the set of other methods within the same class that it calls.
Add an edge between any two method nodes M1 and M2 if:
M1 and M2 access at least one common instance variable.
M1 calls M2, or M2 calls M1.
Calculate the number of connected components in this graph. This number is the LCOM4 value.73 An LCOM4 of 1 is ideal (the class is cohesive).
ATFD (Access to Foreign Data):
For a given class C, iterate through all of its methods.
Within each method, traverse the AST to find all attribute/field access expressions (e.g., some_object.some_field).
If the base of the access (some_object) is an instance of another class C', add C' to a set of foreign data classes.
The ATFD is the final size of this set.

4.4. Establishing Effective Heuristics and Configurable Thresholds

Providing sensible default thresholds is essential for making the detector immediately useful. These defaults should be based on empirical studies and common industry practices.

Metric
Recommended Threshold
Justification/Source
WMC
40
A cyclomatic complexity of 10 is often cited as an upper limit for a single function.74 A WMC of 40 suggests a class has, on average, more than four complex methods or a larger number of simpler ones, indicating high overall complexity.
LCOM4
1
The definition of LCOM4 states that a value of 1 indicates a cohesive class. Any value greater than 1 means the class has disjoint sets of methods and can be split.73
ATFD
5
This is a heuristic value. A class that directly manipulates the data of more than a handful of other classes is likely centralizing logic that should belong to those other classes.
NOM
20
A class with more than 20 methods is often considered large and may be taking on too many responsibilities.68
NAtt
20
A class with more than 20 attributes often indicates it is managing too much state, a common symptom of a God Object.68


4.5. Configuration Schema: Fine-Tuning Detection Sensitivity in TOML

The configuration should allow users to adjust the sensitivity of the detector by tuning the thresholds and specifying how many must be violated to trigger a report.
Proposed uveddi.toml Schema for God Object Detector:

Ini, TOML


# Configuration for the God Object detector.
[detector.god-object]
# Enable or disable the detector.
enabled = true

# Set the severity level for reported issues.
severity = "warning"

# A class/struct is flagged as a potential God Object if it violates at least
# this many of the defined metric thresholds. A value of 2 or 3 is a good starting point.
min_violations_to_report = 2

# Threshold for Weighted Methods per Class (WMC).
# This is the sum of the cyclomatic complexities of all methods in the class.
max_wmc = 40

# Threshold for Lack of Cohesion in Methods (LCOM4).
# This measures the number of disjoint sets of methods. A value > 1 is a strong indicator of low cohesion.
max_lcom4 = 1

# Threshold for Access to Foreign Data (ATFD).
# This counts the number of other classes whose data is directly accessed.
max_atfd = 5

# Threshold for the total number of methods in the class.
max_methods = 20

# Threshold for the total number of attributes/fields in the class.
max_attributes = 20



4.6. Actionable Reporting: Suggesting Refactoring Pathways

The error report must clearly state why the class was flagged by showing which metric thresholds were exceeded. It should also provide a concrete suggestion for refactoring.
Proposed Error Reporting Format:
Message: A summary explaining that the class may be a God Object and why this is problematic.
Location: The file and line where the class/struct is defined.
Details: A list of the specific metric violations that triggered the detection.
Suggestion: A recommendation to refactor the class, potentially using the LCOM4 value to guide the process.
Example Error Report:
Message: Class 'ApplicationManager' may be a God Object. It appears to have too many unrelated responsibilities, making it difficult to maintain and test.
Location: src/main/app_manager.py:25:1
Details (Violated Metrics):
LCOM4: 3 (Threshold: 1)
WMC: 52 (Threshold: 40)
Number of Methods: 28 (Threshold: 20)
Suggestion: Consider refactoring this class by extracting related methods and attributes into smaller, more cohesive classes. The LCOM4 value of 3 suggests this class could potentially be split into 3 separate classes based on its method groups. 69

4.7. Practical Examples: Contrasting Cohesive Classes with God Objects


JavaScript Example

Positive Detection (God Object):
JavaScript
// GodObject.js
class SystemManager {
  constructor() {
    this.users =;
    this.products =;
    this.orders =;
  }
  // User management methods
  addUser(user) { /*... */ }
  getUser(id) { /*... */ }
  // Product inventory methods
  addProduct(product) { /*... */ }
  getProduct(id) { /*... */ }
  // Order processing methods
  createOrder(order) { /*... */ }
  getOrder(id) { /*... */ }
}

This class violates SRP by handling users, products, and orders. Its LCOM4 would be 3, and its NOM would be high.26
Negative Detection (Refactored):
JavaScript
// UserManager.js
class UserManager {
  constructor() { this.users =; }
  addUser(user) { /*... */ }
  getUser(id) { /*... */ }
}

// ProductManager.js
class ProductManager {
  constructor() { this.products =; }
  addProduct(product) { /*... */ }
  getProduct(id) { /*... */ }
}

// OrderManager.js
class OrderManager {
  constructor() { this.orders =; }
  createOrder(order) { /*... */ }
  getOrder(id) { /*... */ }
}

The responsibilities have been split into three smaller, highly cohesive classes, each with a low WMC and an LCOM4 of 1.26

4.8. Integration Guidance: Registration, Testing, and Documentation

Registration: The God Object detector operates on a per-class/struct basis. It can be registered to run on the AST of each file as it is parsed, identifying all class/struct definitions and analyzing them individually.
Testing:
Unit Tests: Create isolated unit tests for each metric calculation function (WMC, LCOM4, ATFD, etc.). Provide hand-crafted class/struct ASTs to these functions and assert that the calculated metrics are correct.
Integration Tests: Develop a suite of sample classes for each language. This suite should include clear examples of God Objects that violate multiple thresholds, as well as examples of well-designed, cohesive classes that should not be flagged. These tests will validate the end-to-end heuristic logic and reporting.
Documentation: The documentation must thoroughly explain the Single Responsibility Principle and the concept of a God Object. It should detail each of the metrics used in the detection heuristic (WMC, LCOM4, ATFD, NOM, NAtt), explain all configuration options, and provide guidance on how to interpret the results. Crucially, it should link to resources on common refactoring techniques like "Extract Class" and "Extract Superclass/Subclass."

Chapter 5: Platform-Wide Performance & Usability


5.1. Ensuring Scalability: Caching and Incremental Analysis Strategies

The performance of static analysis tools on large, enterprise-scale codebases is a critical factor for their adoption and usability. A tool that takes many minutes to run will be abandoned by developers, especially in contexts like pre-commit hooks or interactive IDE feedback.76 The primary performance bottlenecks in static analysis are typically file I/O, the computational cost of parsing source code into ASTs, and the complexity of project-wide graph analysis.4 To ensure Uveddi remains fast and scalable, a multi-pronged optimization strategy involving caching and incremental analysis is essential.

Optimization 1: Caching Abstract Syntax Trees (ASTs)

Parsing is one of the most resource-intensive steps in static analysis. Re-parsing every file on every run is highly inefficient, especially when most files in a project remain unchanged between analyses. To mitigate this, Uveddi should implement a persistent AST cache.
The mechanism for this is straightforward and follows standard caching principles 77:
Hashing: For each source file to be analyzed, compute a cryptographic hash (e.g., SHA-256) of its content.
Cache Check: Before parsing, check if an entry exists in the cache (e.g., a file-based key-value store on disk) with a key matching the file's path and content hash.
Cache Hit: If a matching entry is found, it means the file has not changed since the last analysis. The detector can then load the pre-serialized AST directly from the cache, completely bypassing the expensive parsing step.
Cache Miss: If no matching entry exists, the file is parsed into an AST. This new AST is then serialized and stored in the cache, along with the file's path and content hash, for future runs.
This strategy effectively trades disk space for a significant reduction in computation time, dramatically improving performance for subsequent analyses of the same project.

Optimization 2: Incremental Analysis

Beyond caching ASTs, Uveddi should perform incremental analysis, focusing its work only on the parts of the codebase that have changed.78 This is particularly important for integration into CI/CD pipelines and developer workflows where feedback speed is paramount.76
However, incremental analysis is complicated by the different scopes of the detectors.
Local Detectors: The Magic Value and God Object detectors are local. Their analysis is confined to a single file or class definition. For these, an incremental approach is simple: run the detector only on the ASTs of files that have changed (i.e., those that resulted in a cache miss).
Global Detectors: The Tight Coupling and Cyclic Dependency detectors are global. They rely on a project-wide dependency graph. A change in a single file can alter the structure of this entire graph, potentially creating or resolving cycles and coupling issues far from the site of the change.
A naive incremental approach would re-run the entire global analysis if even one file changes, which negates much of the performance benefit. A more sophisticated, hybrid strategy is required:
Maintain a Cached Dependency Graph: In addition to caching ASTs, Uveddi should maintain a cached representation of the project's full dependency graph.
Targeted Graph Updates: When a file changes, the system should not rebuild the entire graph. Instead, it should:
a. Parse the new file to get its updated list of dependencies.
b. In the cached graph, remove all edges originating from the node representing the changed file.
c. Add new edges based on the updated dependencies.
Optimized Global Analysis:
For Tight Coupling, the analysis can be localized. Only the CBO/RFC metrics for the changed node and any nodes that directly depend on it need to be recalculated.
For Cyclic Dependencies, while a full graph traversal is often still necessary for correctness, the search can be optimized. For instance, the DFS can be prioritized to start from the changed node and its affected neighbors, as new cycles are most likely to involve these nodes.
This intelligent incremental approach ensures that Uveddi provides fast feedback for local issues while efficiently maintaining the integrity of its global analyses, making it a viable tool for even the largest and most complex codebases.

5.2. A Unified Framework for Testing and Documentation

Consistency in testing and documentation is crucial for the long-term maintainability of the Uveddi platform and for providing a clear, predictable experience for its users.

Standardized Testing Structure

A standardized directory structure and testing methodology should be adopted for all detectors. For each detector, the test suite should be organized as follows:



detectors/
└── <detector_name>/
    ├── tests/
    │   ├── rust/
    │   │   ├── positive/
    │   │   │   └── test_case_01.rs
    │   │   └── negative/
    │   │       └── test_case_01.rs
    │   ├── python/
    │   │   ├── positive/
    │   │   │   └── test_case_01.py
    │   │   └── negative/
    │   │       └── test_case_01.py
    │   └── javascript/
    │       ├── positive/
    │       │   └── test_case_01.js
    │       └── negative/
    │           └── test_case_01.js
    └── src/
        └── lib.rs


This structure ensures that each detector has comprehensive test coverage across all supported languages, with clear separation between cases that should trigger a detection (positive) and those that should not (negative).

Standardized Documentation Template

To ensure a consistent user experience, the documentation for every detector should follow a uniform template. Each detector's documentation, likely a markdown file, should contain the following sections:
The Anti-Pattern: A clear, concise explanation of the anti-pattern, its negative consequences, and why it should be avoided.
Detection Strategy: A high-level overview of how the detector identifies the anti-pattern, including the key metrics or algorithms used.
Configuration: A complete reference of all available TOML configuration options for the detector, including their purpose, accepted values, default values, and illustrative examples.
Interpreting Results: Guidance on how to understand the detector's output, with examples of error messages and suggestions for common refactoring patterns to resolve the detected issues.
Adopting these standardized frameworks for testing and documentation will streamline the development of future detectors and ensure that Uveddi remains a robust, reliable, and user-friendly platform.

Conclusion: Advancing Uveddi to a Production-Grade Analysis Platform

This report has laid out a comprehensive and actionable blueprint for transforming four foundational detector scaffolds within the Uveddi platform—Magic Values, Tight Coupling, Cyclic Dependencies, and God Objects—into production-grade static analysis tools. The successful implementation of these designs will provide developers with powerful, automated quality gates capable of operating across Rust, Python, and JavaScript codebases.
The core recommendations are grounded in established software engineering principles and state-of-the-art static analysis techniques. For each detector, this report has provided:
Language-Specific Detection Strategies: Algorithms and heuristics tailored to the unique syntax and semantics of Rust, Python, and JavaScript, leveraging core parsing libraries like syn, ast, and acorn.
Robust Configuration Schemas: Detailed TOML configurations that grant users fine-grained control over detection sensitivity, thresholds, and whitelisting, ensuring the tool can be adapted to any project's standards.
Actionable Error Reporting: A framework for generating clear, concise, and non-blaming error messages that not only identify a problem but also guide the developer toward a concrete solution.
Beyond the individual detectors, this report emphasizes the critical importance of platform-wide features that are the hallmark of a truly effective developer tool.80 The proposed strategies for
performance optimization, including AST caching and intelligent incremental analysis, are essential for ensuring Uveddi remains fast and scalable on large, real-world projects. Furthermore, the adoption of a unified framework for testing and documentation will guarantee consistency, reliability, and a superior user experience as the platform grows.
By implementing the detailed plans outlined herein, the Uveddi project can move beyond its initial scaffolding to become a powerful, multi-language static analysis platform. It will be equipped to identify not just superficial style issues but deep-seated architectural flaws, empowering development teams to build more maintainable, reliable, and secure software across their entire technology stack.

References

11 Magic number (programming). (n.d.). In
Wikipedia. Retrieved from https://en.wikipedia.org/wiki/Magic_number_(programming
38 Magic Strings. (n.d.).
deviq.com. Retrieved from https://deviq.com/antipatterns/magic-strings/
5 MLScent: A tool for Anti-pattern detection in ML projects. (2025).
arXiv. Retrieved from https://arxiv.org/html/2502.18466v1
6 Anti-Patterns vs Patterns: What's The Difference? (n.d.).
BMC Blogs. Retrieved from https://www.bmc.com/blogs/anti-patterns-vs-patterns/
7 Anti-Patterns in Software Architecture. (n.d.).
Number Analytics. Retrieved from https://www.numberanalytics.com/blog/anti-patterns-in-software-architecture
48 Coupling (computer programming). (n.d.). In
Wikipedia. Retrieved from https://en.wikipedia.org/wiki/Coupling_(computer_programming
82 What are Anti-patterns? (n.d.).
Embold. Retrieved from https://docs.embold.io/anti-patterns/
36 Circular dependency. (n.d.). In
Wikipedia. Retrieved from https://en.wikipedia.org/wiki/Circular_dependency
56 Sassi, W. (2025). Understanding Dependency Cycles: How SparkDI Uses DFS for Detection.
Medium. Retrieved from https://medium.com/@sassiwalid/understanding-dependency-cycles-how-sparkdi-uses-dfs-for-detection-4fa450436837
37 Han, Y. (n.d.). Detect Cycle When Analyzing Dependencies.
hanyoung.uk. Retrieved from https://www.hanyoung.uk/blog/detect-cycle-when-analyze-dependencies/
55 What's wrong with circular references? (2010).
Software Engineering Stack Exchange. Retrieved from https://softwareengineering.stackexchange.com/questions/11856/whats-wrong-with-circular-references
50 A Metric-Based Approach for Anti-pattern Detection in UML Designs. (2007).
ResearchGate. Retrieved from https://www.researchgate.net/publication/226460096_A_Metric-Based_Approach_for_Anti-pattern_Detection_in_UML_Designs
68 Palomba, F. (n.d.). The Blob.
dibt.unimol.it. Retrieved from https://dibt.unimol.it/staff/fpalomba/documents/B1.pdf
70 What Is a God Class? (n.d.).
LinearB. Retrieved from https://linearb.io/blog/what-is-a-god-class
Static Code Analysis. (n.d.). Sangfor. Retrieved from https://www.sangfor.com/glossary/cybersecurity/static-code-analysis
Static Code Analysis. (n.d.). accelq. Retrieved from https://www.accelq.com/blog/static-code-analysis/
80 Static Analysis vs Hidden Anti-Patterns: What It Sees and What It Misses. (n.d.).
IN-COM. Retrieved from https://www.in-com.com/blog/static-analysis-vs-hidden-anti-patterns-what-it-sees-and-what-it-misses/
51 Code metrics - class coupling. (2022).
Microsoft Learn. Retrieved from https://learn.microsoft.com/en-us/visualstudio/code-quality/code-metrics-class-coupling?view=vs-2022
52 Optimizing Software Design with CBO. (2025).
Number Analytics. Retrieved from https://www.numberanalytics.com/blog/optimizing-software-design-with-cbo
53 Coupling Between Object classes (CBO). (n.d.).
ObjectScript Quality. Retrieved from https://objectscriptquality.com/docs/metrics/coupling-between-object-classes-cbo
83 Lack of Cohesion in Methods. (n.d.).
ARISA. Retrieved from https://www.arisa.se/compendium/node116.html
72 Ultimate Guide to Lack of Cohesion in Methods (LCOM) in Software Metrics. (2024).
Number Analytics. Retrieved from https://www.numberanalytics.com/blog/ultimate-guide-lack-cohesion-methods-software-metrics
73 Lack of Cohesion in Methods (LCOM4). (n.d.).
ObjectScript Quality. Retrieved from https://objectscriptquality.com/docs/metrics/lack-cohesion-methods-lcom4
57 Cycles and Graph Connectivity: A Deep Dive. (n.d.).
Number Analytics. Retrieved from https://www.numberanalytics.com/blog/cycles-graph-connectivity-deep-dive
58 Detect cycle in an undirected graph. (n.d.).
GeeksforGeeks. Retrieved from https://www.geeksforgeeks.org/dsa/detect-cycle-undirected-graph/
12 What is ast.literal_eval(node_or_string) in Python? (n.d.).
Educative. Retrieved from https://www.educative.io/answers/what-is-astliteralevalnodeorstring-in-python
11 Magic number (programming). (n.d.). In
Wikipedia. Retrieved from https://en.wikipedia.org/wiki/Magic_number_(programming
38 Magic Strings. (n.d.).
deviq.com. Retrieved from https://deviq.com/antipatterns/magic-strings/
13 Alapont, R. (2023). Magic numbers and magic strings, it's time to talk about it.
DEV Community. Retrieved from https://dev.to/ruben_alapont/magic-numbers-and-magic-strings-its-time-to-talk-about-it-ci2
14 Discussion on CSL Next. (2023).
GitHub. Retrieved from https://github.com/bdarcus/csln/discussions/130
49 When is tight coupling essential or a good thing? (2015).
Stack Overflow. Retrieved from https://stackoverflow.com/questions/28777799/when-is-tight-coupling-essential-or-a-good-thing
16 What is the difference between loose coupling and tight coupling in the object oriented paradigm? (n.d.).
Stack Overflow. Retrieved from https://stackoverflow.com/questions/2832017/what-is-the-difference-between-loose-coupling-and-tight-coupling-in-the-object-o
17 Cohesion & Coupling in Python. (2022).
Python in Plain English. Retrieved from https://python.plainenglish.io/cohesion-coupling-in-python-f332da3745f8
18 Cohesion and Coupling in Javascript. (2023).
DEV Community. Retrieved from https://dev.to/m__mdy__m/cohesion-and-coupling-in-javascript-2efg
8 Lint for module dependency cycles. (2021).
GitHub. Retrieved from https://github.com/rust-lang/rust-clippy/issues/6541
21 How and why does rust allow cyclic imports in modules? (2024).
Reddit. Retrieved from https://www.reddit.com/r/rust/comments/1iho34s/how_and_why_does_rust_allow_cyclic_imports_in/
9 How to Fix a Circular Import in Python. (n.d.).
Rollbar. Retrieved from https://rollbar.com/blog/how-to-fix-circular-import-in-python/
22 Batchelder, N. (2024). One way to fix Python circular imports.
Ned Batchelder's blog. Retrieved from https://nedbatchelder.com/blog/202405/one_way_to_fix_python_circular_imports.html
10 Viktomas. (2022). Circular Dependencies in JavaScript Explained.
viktomas.com. Retrieved from https://blog.viktomas.com/graph/circular-dependencies-in-javascript-explained/
64 3 Ways to Detect Circular Dependencies in JavaScript Projects. (n.d.).
Bit Blog. Retrieved from https://blog.bitsrc.io/3-ways-to-detect-circular-dependencies-in-javascript-projects-f5a22310cb5a
23 God object. (n.d.). In
Wikipedia. Retrieved from https://en.wikipedia.org/wiki/God_object
24 When does a regular class become a God class? (n.d.).
Reddit. Retrieved from https://www.reddit.com/r/learnprogramming/comments/3ye2ng/when_does_a_regular_class_become_a_god_class/
25 Refactoring the God Class in Python. (2021).
Better Programming. Retrieved from https://betterprogramming.pub/refactoring-the-god-class-in-python-5c13942d0e75
26 Freitas, W. (2024). Understanding God Objects in Object-Oriented Programming.
DEV Community. Retrieved from https://dev.to/wallacefreitas/understanding-god-objects-in-object-oriented-programming-5636
27 Avoiding Software Bottlenecks: Understanding the 'God Object' Anti-Pattern. (n.d.).
HackerNoon. Retrieved from https://hackernoon.com/avoiding-software-bottlenecks-understanding-the-god-object-anti-pattern
29 syn - Docs.rs. (n.d.).
docs.rs. Retrieved from https://docs.rs/syn
30 syn - crates.io. (n.d.).
crates.io. Retrieved from https://crates.io/crates/syn
31 Python's "ast" module. (2019).
YouTube. Retrieved from https://www.youtube.com/watch?v=2tOr_0k8EYE
28 ast — Abstract Syntax Trees. (n.d.).
Python documentation. Retrieved from https://docs.python.org/3/library/ast.html
32 acorn vs esprima vs typescript. (n.d.).
npm compare. Retrieved from https://npm-compare.com/acorn,esprima,typescript
33 acorn vs esprima. (n.d.).
npm compare. Retrieved from https://npm-compare.com/acorn,esprima
41 Pylint configuration. (n.d.).
Codeac. Retrieved from https://www.codeac.io/documentation/pylint-configuration.html
44 How to Find Actionable Static Analysis Warnings. (2022).
ResearchGate. Retrieved from https://www.researchgate.net/publication/360804346_How_to_Find_Actionable_Static_Analysis_Warnings
40 Clippy Lints. (2021).
rust-lang.github.io. Retrieved from https://rust-lang.github.io/rust-clippy/rust-1.56.0/index.html
39 no-magic-numbers. (n.d.).
ESLint. Retrieved from https://archive.eslint.org/docs/rules/no-magic-numbers
54 Code examples for a noob to understand tight coupling vs dependency injection? (2023).
Reddit. Retrieved from https://www.reddit.com/r/csharp/comments/168nbv1/code_examples_for_a_noob_to_understand_tight/
15 Accidentally Coupled: The Worst Coupling by Loose Coupling. (n.d.).
hacklewayne.com. Retrieved from https://www.hacklewayne.com/accidentally-coupled-the-worst-coupling-by-loose-coupling
Top Python Code Analysis Tools to Improve Code Quality. (2024). Jit. Retrieved from https://www.jit.io/resources/appsec-tools/top-python-code-analysis-tools-to-improve-code-quality
59 workspacer-detect-circular-deps. (n.d.).
crates.io. Retrieved from https://crates.io/crates/workspacer-detect-circular-deps/0.1.1
20 Does cargo support cyclic dependencies? (2019).
Rust Programming Language Forum. Retrieved from https://users.rust-lang.org/t/does-cargo-support-cyclic-dependencies/35666
62 How to Resolve Circular Imports in Python. (n.d.).
LabEx. Retrieved from https://labex.io/tutorials/python-how-to-resolve-circular-imports-in-python-418812
63 How to Fix and Prevent Circular Imports in Python. (2024).
DataCamp. Retrieved from https://www.datacamp.com/tutorial/python-circular-import
65 How to analyze circular dependencies in ES6. (n.d.).
Railsware Blog. Retrieved from https://railsware.com/blog/how-to-analyze-circular-dependencies-in-es6/
64 3 Ways to Detect Circular Dependencies in JavaScript Projects. (n.d.).
Bit Blog. Retrieved from https://blog.bitsrc.io/3-ways-to-detect-circular-dependencies-in-javascript-projects-f5a22310cb5a
29 syn - Docs.rs. (n.d.).
docs.rs. Retrieved from https://docs.rs/syn
42 Python and TOML: New Best Friends. (n.d.).
Real Python. Retrieved from https://realpython.com/python-toml/
45 How to write effective error messages. (2017).
Opensource.com. Retrieved from https://opensource.com/article/17/8/write-effective-error-messages
76 Top Static Code Analysis Tools for 2025. (n.d.).
Uproot Security. Retrieved from https://www.uprootsecurity.com/blog/static-code-analysis-tools-guide
29 Module syn::visit. (n.d.).
docs.rs. Retrieved from https://docs.rs/syn/latest/syn/visit/index.html
34 Getting Started. (n.d.).
Esprima. Retrieved from https://esprima.readthedocs.io/en/latest/getting-started.html
35 Syntactic Analysis (Parsing). (n.d.).
Esprima. Retrieved from https://esprima.readthedocs.io/en/latest/syntactic-analysis.html
11 Magic number (programming). (n.d.). In
Wikipedia. Retrieved from https://en.wikipedia.org/wiki/Magic_number_(programming
38 Magic Strings. (n.d.).
deviq.com. Retrieved from https://deviq.com/antipatterns/magic-strings/
54 Code examples for a noob to understand tight coupling vs dependency injection? (2023).
Reddit. Retrieved from https://www.reddit.com/r/csharp/comments/168nbv1/code_examples_for_a_noob_to_understand_tight/
61 How bad is for usability to allow circular module dependencies? (2021).
Reddit. Retrieved from https://www.reddit.com/r/ProgrammingLanguages/comments/rf7mra/how_bad_is_for_usability_to_allow_circular_module/
23 God object. (n.d.). In
Wikipedia. Retrieved from https://en.wikipedia.org/wiki/God_object
69 How do you refactor a "god class"? (2010).
Stack Overflow. Retrieved from https://stackoverflow.com/questions/14870377/how-do-you-refactor-a-god-class
27 Avoiding Software Bottlenecks: Understanding the 'God Object' Anti-Pattern. (n.d.).
HackerNoon. Retrieved from https://hackernoon.com/avoiding-software-bottlenecks-understanding-the-god-object-anti-pattern
43 The Developer's Guide to TOML. (n.d.).
anbowell.com. Retrieved from https://www.anbowell.com/blog/the-developers-guide-to-toml
46 How to write & design user-friendly error messages. (2020).
Medium. Retrieved from https://medium.com/thinking-design/how-to-write-design-user-friendly-error-messages-87d0207bb902
47 Writing user-friendly error messages. (n.d.).
OneSchema. Retrieved from https://docs.oneschema.co/docs/error-message-best-practices
78 Grandma's Recipe for Mastering Regular Static Analysis. (2024).
Medium. Retrieved from https://unicorn-dev.medium.com/grandmas-recipe-for-mastering-regular-static-analysis-2c99c49930fb
77 What is Caching? (n.d.).
Amazon Web Services. Retrieved from https://aws.amazon.com/caching/
71 What is Cyclomatic Complexity? (n.d.).
SonarSource. Retrieved from https://www.sonarsource.com/learn/cyclomatic-complexity/
74 Code metrics - cyclomatic complexity. (2022).
Microsoft Learn. Retrieved from https://learn.microsoft.com/en-us/visualstudio/code-quality/code-metrics-cyclomatic-complexity?view=vs-2022
60 cargo-tree. (n.d.).
The Cargo Book. Retrieved from https://doc.rust-lang.org/cargo/commands/cargo-tree.html
66 CircularDependencyRspackPlugin. (n.d.).
Rspack. Retrieved from https://rspack.rs/plugins/rspack/circular-dependency-rspack-plugin
67 circular-dependency-plugin. (n.d.).
npm. Retrieved from https://www.npmjs.com/package/circular-dependency-plugin
69 How do you refactor a "god class"? (2010).
Stack Overflow. Retrieved from https://stackoverflow.com/questions/14870377/how-do-you-refactor-a-god-class
75 God Class: The Definitive Guide to Identifying and Avoiding It. (n.d.).
MetriDev. Retrieved from https://www.metridev.com/metrics/god-class-the-definitive-guide-to-identifying-and-avoiding-it/
81 Linting Best Practices. (n.d.).
webapp.io. Retrieved from https://webapp.io/blog/linting-best-practices/
4 Top 20 Python Static Analysis Tools in 2025. (n.d.).
IN-COM. Retrieved from https://www.in-com.com/blog/top-20-python-static-analysis-tools-in-2025-improve-code-quality-and-performance/
79 Incremental Static Regeneration (ISR): Keeping Your Pages Fresh and Fast. (2024).
Medium. Retrieved from https://medium.com/@ignatovich.dm/incremental-static-regeneration-isr-keeping-your-pages-fresh-and-fast-f28933c28e54
28 ast — Abstract Syntax Trees. (n.d.).
Python documentation. Retrieved from https://docs.python.org/3/library/ast.html
84 Pylint configuration. (n.d.).
Pylint documentation. Retrieved from https://pylint.readthedocs.io/en/latest/user_guide/configuration/index.html
85 Configuration Files. (n.d.).
ESLint. Retrieved from https://eslint.org/docs/latest/use/configure/configuration-files
86 Module syn::parse. (n.d.).
docs.rs. Retrieved from https://docs.rs/syn/latest/syn/parse/index.html
87 Module syn::visit. (n.d.).
docs.rs. Retrieved from https://docs.rs/syn/latest/syn/visit/index.html
88 How To Traverse an Abstract Syntax Tree with Acorn and Recast in JavaScript. (2022).
DigitalOcean. Retrieved from https://www.digitalocean.com/community/tutorials/js-traversing-ast
35 Syntactic Analysis (Parsing). (n.d.).
Esprima. Retrieved from https://esprima.readthedocs.io/en/latest/syntactic-analysis.html
45 How to write effective error messages. (2017).
Opensource.com. Retrieved from https://opensource.com/article/17/8/write-effective-error-messages
39 no-magic-numbers. (n.d.).
ESLint. Retrieved from https://archive.eslint.org/docs/rules/no-magic-numbers
Works cited
The Essential Guide to Static Code Analysis - Sangfor Technologies, accessed July 12, 2025, https://www.sangfor.com/glossary/cybersecurity/static-code-analysis
Static Code Analysis: Tools, Types & How It Works - ACCELQ, accessed July 12, 2025, https://www.accelq.com/blog/static-code-analysis/
Top 10 Python Code Analysis Tools in 2025 to Improve Code Quality - Jit.io, accessed July 12, 2025, https://www.jit.io/resources/appsec-tools/top-python-code-analysis-tools-to-improve-code-quality
Top 20 Python Static Analysis Tools in 2025: Improve Code Quality and Performance, accessed July 12, 2025, https://www.in-com.com/blog/top-20-python-static-analysis-tools-in-2025-improve-code-quality-and-performance/
MLScent: A tool for Anti-pattern detection in ML projects - arXiv, accessed July 12, 2025, https://arxiv.org/html/2502.18466v1
Anti-Patterns vs. Patterns: What Is the Difference? – BMC Software | Blogs, accessed July 12, 2025, https://www.bmc.com/blogs/anti-patterns-vs-patterns/
Anti-Patterns in Software Architecture - Number Analytics, accessed July 12, 2025, https://www.numberanalytics.com/blog/anti-patterns-in-software-architecture
Optional warning on module dependency cycles · Issue #6541 · rust-lang/rust-clippy, accessed July 12, 2025, https://github.com/rust-lang/rust-clippy/issues/6541
How to Fix a Circular Import in Python - Rollbar, accessed July 12, 2025, https://rollbar.com/blog/how-to-fix-circular-import-in-python/
Circular Dependencies in JavaScript Explained - Tomas Vik, accessed July 12, 2025, https://blog.viktomas.com/graph/circular-dependencies-in-javascript-explained/
Magic number (programming) - Wikipedia, accessed July 12, 2025, https://en.wikipedia.org/wiki/Magic_number_(programming)
What is ast.literal_eval(node_or_string) in Python? - Educative.io, accessed July 12, 2025, https://www.educative.io/answers/what-is-astliteralevalnodeorstring-in-python
Magic Numbers and Magic Strings: It's time to talk about it - DEV Community, accessed July 12, 2025, https://dev.to/ruben_alapont/magic-numbers-and-magic-strings-its-time-to-talk-about-it-ci2
Why Rust? · bdarcus csln · Discussion #130 - GitHub, accessed July 12, 2025, https://github.com/bdarcus/csln/discussions/130
Accidentally coupled! The worst coupling by loose coupling | Hackle's blog, accessed July 12, 2025, https://www.hacklewayne.com/accidentally-coupled-the-worst-coupling-by-loose-coupling
stackoverflow.com, accessed July 12, 2025, https://stackoverflow.com/questions/2832017/what-is-the-difference-between-loose-coupling-and-tight-coupling-in-the-object-o#:~:text=Tight%20Coupling%20means%20one%20class,runtime%20instead%20of%20hard%2Dcoded.
Cohesion & Coupling in Python, accessed July 12, 2025, https://python.plainenglish.io/cohesion-coupling-in-python-f332da3745f8
dev.to, accessed July 12, 2025, https://dev.to/m__mdy__m/cohesion-and-coupling-in-javascript-2efg#:~:text=Tight%20coupling%20means%20modules%20are,in%20other%20tightly%20coupled%20modules.
Cohesion and Coupling in Javascript - DEV Community, accessed July 12, 2025, https://dev.to/m__mdy__m/cohesion-and-coupling-in-javascript-2efg
Does Cargo support cyclic dependencies? - help - The Rust Programming Language Forum, accessed July 12, 2025, https://users.rust-lang.org/t/does-cargo-support-cyclic-dependencies/35666
How and why does rust allow cyclic imports in modules? - Reddit, accessed July 12, 2025, https://www.reddit.com/r/rust/comments/1iho34s/how_and_why_does_rust_allow_cyclic_imports_in/
One way to fix Python circular imports - Ned Batchelder, accessed July 12, 2025, https://nedbatchelder.com/blog/202405/one_way_to_fix_python_circular_imports.html
God object - Wikipedia, accessed July 12, 2025, https://en.wikipedia.org/wiki/God_object
When does a regular class become a God class? : r/learnprogramming - Reddit, accessed July 12, 2025, https://www.reddit.com/r/learnprogramming/comments/3ye2ng/when_does_a_regular_class_become_a_god_class/
Refactoring the God Class in Python | by Brian Redmond - Better Programming, accessed July 12, 2025, https://betterprogramming.pub/refactoring-the-god-class-in-python-5c13942d0e75
Understanding God Objects in Object-Oriented Programming - DEV Community, accessed July 12, 2025, https://dev.to/wallacefreitas/understanding-god-objects-in-object-oriented-programming-5636
Avoiding Software Bottlenecks: Understanding the 'God Object' Anti-Pattern - HackerNoon, accessed July 12, 2025, https://hackernoon.com/avoiding-software-bottlenecks-understanding-the-god-object-anti-pattern
ast — Abstract Syntax Trees — Python 3.13.5 documentation, accessed July 12, 2025, https://docs.python.org/3/library/ast.html
syn - Rust - Docs.rs, accessed July 12, 2025, https://docs.rs/syn
syn - Parser for Rust source code - Crates.io, accessed July 12, 2025, https://crates.io/crates/syn
Powerful Python source code processing with "ast" - YouTube, accessed July 12, 2025, https://www.youtube.com/watch?v=2tOr_0k8EYE
acorn vs typescript vs esprima: Which is Better JavaScript Parsing and Type Checking Libraries? - NPM Compare, accessed July 12, 2025, https://npm-compare.com/acorn,esprima,typescript
acorn vs esprima | JavaScript Parsing Libraries Comparison - NPM Compare, accessed July 12, 2025, https://npm-compare.com/acorn,esprima
Chapter 1. Getting Started — Esprima master documentation, accessed July 12, 2025, https://esprima.readthedocs.io/en/latest/getting-started.html
Chapter 2. Syntactic Analysis (Parsing) — Esprima master ..., accessed July 12, 2025, https://esprima.readthedocs.io/en/latest/syntactic-analysis.html
Circular dependency - Wikipedia, accessed July 12, 2025, https://en.wikipedia.org/wiki/Circular_dependency
Detect Cycle When Analyzing Dependencies - Han Young's blog, accessed July 12, 2025, https://www.hanyoung.uk/blog/detect-cycle-when-analyze-dependencies/
Magic Strings - DevIQ, accessed July 12, 2025, https://deviq.com/antipatterns/magic-strings/
no-magic-numbers - ESLint - Pluggable JavaScript linter, accessed July 12, 2025, https://archive.eslint.org/docs/rules/no-magic-numbers
Clippy Lints, accessed July 12, 2025, https://rust-lang.github.io/rust-clippy/rust-1.56.0/index.html
Pylint configuration - Codeac, accessed July 12, 2025, https://www.codeac.io/documentation/pylint-configuration.html
Python and TOML: New Best Friends – Real Python, accessed July 12, 2025, https://realpython.com/python-toml/
The Developer's Guide To TOML | AnBowell, accessed July 12, 2025, https://www.anbowell.com/blog/the-developers-guide-to-toml
(PDF) How to Find Actionable Static Analysis Warnings - ResearchGate, accessed July 12, 2025, https://www.researchgate.net/publication/360804346_How_to_Find_Actionable_Static_Analysis_Warnings
How to write better error messages | Opensource.com, accessed July 12, 2025, https://opensource.com/article/17/8/write-effective-error-messages
How to Write and Design User-Friendly Error Messages | by Nick Babich - Medium, accessed July 12, 2025, https://medium.com/thinking-design/how-to-write-design-user-friendly-error-messages-87d0207bb902
Writing user-friendly error messages - Getting Started, accessed July 12, 2025, https://docs.oneschema.co/docs/error-message-best-practices
Coupling (computer programming) - Wikipedia, accessed July 12, 2025, https://en.wikipedia.org/wiki/Coupling_(computer_programming)
When is tight coupling essential or a good thing? - Stack Overflow, accessed July 12, 2025, https://stackoverflow.com/questions/28777799/when-is-tight-coupling-essential-or-a-good-thing
A Metric-Based Approach for Anti-pattern Detection in UML Designs - ResearchGate, accessed July 12, 2025, https://www.researchgate.net/publication/226460096_A_Metric-Based_Approach_for_Anti-pattern_Detection_in_UML_Designs
Code metrics - Class coupling - Visual Studio (Windows) | Microsoft Learn, accessed July 12, 2025, https://learn.microsoft.com/en-us/visualstudio/code-quality/code-metrics-class-coupling?view=vs-2022
Optimizing Software Design with CBO - Number Analytics, accessed July 12, 2025, https://www.numberanalytics.com/blog/optimizing-software-design-with-cbo
Coupling Between Object classes (CBO) | objectscriptQuality, accessed July 12, 2025, https://objectscriptquality.com/docs/metrics/coupling-between-object-classes-cbo
Code examples for a noob to understand tight coupling? : r/csharp - Reddit, accessed July 12, 2025, https://www.reddit.com/r/csharp/comments/168nbv1/code_examples_for_a_noob_to_understand_tight/
What's wrong with circular references? - Software Engineering Stack Exchange, accessed July 12, 2025, https://softwareengineering.stackexchange.com/questions/11856/whats-wrong-with-circular-references
Understanding Dependency Cycles: How SparkDI Uses DFS for Detection | by Walid Sassi, accessed July 12, 2025, https://medium.com/@sassiwalid/understanding-dependency-cycles-how-sparkdi-uses-dfs-for-detection-4fa450436837
Cycles in Graph Connectivity: A Deep Dive - Number Analytics, accessed July 12, 2025, https://www.numberanalytics.com/blog/cycles-graph-connectivity-deep-dive
Detect cycle in an undirected graph - GeeksforGeeks, accessed July 12, 2025, https://www.geeksforgeeks.org/dsa/detect-cycle-undirected-graph/
workspacer-detect-circular-deps - crates.io: Rust Package Registry, accessed July 12, 2025, https://crates.io/crates/workspacer-detect-circular-deps/0.1.1
cargo tree - The Cargo Book - Rust Documentation, accessed July 12, 2025, https://doc.rust-lang.org/cargo/commands/cargo-tree.html
How bad is for usability to allow circular module dependencies? - Reddit, accessed July 12, 2025, https://www.reddit.com/r/ProgrammingLanguages/comments/rf7mra/how_bad_is_for_usability_to_allow_circular_module/
How to resolve circular imports in Python | LabEx, accessed July 12, 2025, https://labex.io/tutorials/python-how-to-resolve-circular-imports-in-python-418812
Python Circular Import: Causes, Fixes, and Best Practices | DataCamp, accessed July 12, 2025, https://www.datacamp.com/tutorial/python-circular-import
3 Ways To Detect Circular Dependencies In JavaScript Projects - Bits and Pieces, accessed July 12, 2025, https://blog.bitsrc.io/3-ways-to-detect-circular-dependencies-in-javascript-projects-f5a22310cb5a
Analyzing ES6 Circular Dependencies | Railsware Blog, accessed July 12, 2025, https://railsware.com/blog/how-to-analyze-circular-dependencies-in-es6/
CircularDependencyRspackPlugin - Rspack, accessed July 12, 2025, https://rspack.rs/plugins/rspack/circular-dependency-rspack-plugin
circular-dependency-plugin - NPM, accessed July 12, 2025, https://www.npmjs.com/package/circular-dependency-plugin
Anti-Pattern Detection: Methods, Challenges, and Open Issues, accessed July 12, 2025, https://dibt.unimol.it/staff/fpalomba/documents/B1.pdf
How do you refactor a God class? - Stack Overflow, accessed July 12, 2025, https://stackoverflow.com/questions/14870377/how-do-you-refactor-a-god-class
What Is a God Class and Why Should We Avoid It? | LinearB Blog, accessed July 12, 2025, https://linearb.io/blog/what-is-a-god-class
What is Cyclomatic Complexity? Definition Guide & Examples - Sonar, accessed July 12, 2025, https://www.sonarsource.com/learn/cyclomatic-complexity/
Understanding Lack of Cohesion in Methods - Number Analytics, accessed July 12, 2025, https://www.numberanalytics.com/blog/ultimate-guide-lack-cohesion-methods-software-metrics
Lack of Cohesion in Methods (LCOM4) | objectscriptQuality, accessed July 12, 2025, https://objectscriptquality.com/docs/metrics/lack-cohesion-methods-lcom4
Code metrics - Cyclomatic complexity - Visual Studio (Windows) | Microsoft Learn, accessed July 12, 2025, https://learn.microsoft.com/en-us/visualstudio/code-quality/code-metrics-cyclomatic-complexity?view=vs-2022
God Class: The Definitive Guide to Identifying and Avoiding It - Metridev, accessed July 12, 2025, https://www.metridev.com/metrics/god-class-the-definitive-guide-to-identifying-and-avoiding-it/
Static Code Analysis Tools: Strengths, Limitations, and Real-World Use - Uproot Security, accessed July 12, 2025, https://www.uprootsecurity.com/blog/static-code-analysis-tools-guide
What is Caching and How it Works | AWS, accessed July 12, 2025, https://aws.amazon.com/caching/
Grandma's recipe for mastering regular static analysis | by Unicorn Developer - Medium, accessed July 12, 2025, https://unicorn-dev.medium.com/grandmas-recipe-for-mastering-regular-static-analysis-2c99c49930fb
Incremental Static Regeneration (ISR): Keeping Your Pages Fresh and Fast - Medium, accessed July 12, 2025, https://medium.com/@ignatovich.dm/incremental-static-regeneration-isr-keeping-your-pages-fresh-and-fast-f28933c28e54
Static Analysis vs. Hidden Anti-Patterns: What It Sees and What It Misses, accessed July 12, 2025, https://www.in-com.com/blog/static-analysis-vs-hidden-anti-patterns-what-it-sees-and-what-it-misses/
Best practices for Linting - Webapp.io, accessed July 12, 2025, https://webapp.io/blog/linting-best-practices/
Anti-patterns - Code Quality Docs, accessed July 12, 2025, https://docs.embold.io/anti-patterns/
Lack of Cohesion in Methods ( ) - ARiSA, accessed July 12, 2025, https://www.arisa.se/compendium/node116.html
Configuration - Pylint 3.3.7 documentation, accessed July 12, 2025, https://pylint.readthedocs.io/en/latest/user_guide/configuration/index.html
Configuration Files - ESLint - Pluggable JavaScript Linter, accessed July 12, 2025, https://eslint.org/docs/latest/use/configure/configuration-files
syn::parse - Rust - Docs.rs, accessed July 12, 2025, https://docs.rs/syn/latest/syn/parse/index.html
syn::visit - Rust - Docs.rs, accessed July 12, 2025, https://docs.rs/syn/latest/syn/visit/index.html
Read JavaScript Source Code, Using an AST | DigitalOcean, accessed July 12, 2025, https://www.digitalocean.com/community/tutorials/js-traversing-ast
