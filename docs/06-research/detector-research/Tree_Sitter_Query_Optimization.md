
A Cross-Language Guide to Effective Tree-sitter Queries: Detecting Calls and References in Rust, Python, and JavaScript

Tree-sitter has emerged as a foundational technology for a new generation of developer tooling, enabling applications to move beyond fragile, text-based regular expressions toward a deep, structural understanding of source code. As a parser generator, it produces fast, robust, and incremental parsers that are capable of handling syntactically incomplete or erroneous code, making it ideal for real-time applications like text editors and IDEs. The primary objective of this report is to provide a comprehensive, expert-level guide for developers to craft precise, performant, and robust query patterns for identifying function calls and references. This analysis will navigate the distinct grammars of Rust, Python, and JavaScript, offering not just a list of patterns, but a deep dive into the rationale behind their construction, grounded in the structure of each language's concrete syntax tree (CST).

Foundations of Tree-sitter Querying: From Source Text to Actionable Insights

Before tackling language-specific patterns, it is essential to establish a solid mental model of Tree-sitter's architecture and its powerful query language. This foundational understanding is crucial for writing queries that are both effective and maintainable.

The Tree-sitter Pipeline: Grammar, Parser, and Tree

The Tree-sitter ecosystem operates on a clear, multi-stage pipeline. It begins with a grammar definition, typically written in a JavaScript-based domain-specific language (DSL) within a grammar.js file. This file describes the syntactic rules of a programming language. The Tree-sitter command-line interface (CLI) then processes this grammar to generate a highly optimized parser written in C. This generated C code can be compiled into a dynamic library or a WebAssembly module, allowing it to be consumed by applications written in a variety of host languages, such as Rust, Python, or Node.js, via official language bindings. When this parser is fed source code, it produces a syntax tree that can be traversed and queried.
A critical aspect of Tree-sitter's output is that it generates a Concrete Syntax Tree (CST), not a more traditional Abstract Syntax Tree (AST). A CST is a complete and faithful representation of the source text, retaining every piece of information, including punctuation (e.g., parentheses, commas), keywords, comments, and whitespace. This is a deliberate design choice that makes Tree-sitter exceptionally well-suited for tasks like syntax highlighting, code formatting, and automated refactoring, which require access to the full fidelity of the original source. This distinction directly influences query construction; because the CST contains nodes for constructs like operators and keywords, queries must sometimes match these "anonymous" nodes to be sufficiently precise. For example, a query to find a specific binary operation might need to match the operator token itself, such as (binary_expression operator: "!="...). Understanding that the CST preserves this level of detail is fundamental to mastering the query language.

Anatomy of a Syntax Tree: Nodes, Fields, and Children

The CST produced by a Tree-sitter parser is a hierarchical structure composed of nodes. Each node in the tree represents a segment of the source code and possesses several key properties accessible through the API bindings in languages like Rust and Python.
Named vs. Anonymous Nodes: Nodes are categorized as either "named" or "anonymous." Named nodes correspond directly to rules defined in the language's grammar (e.g., function_definition, if_statement). Anonymous nodes, by contrast, typically represent literal strings or keywords from the grammar (e.g., the keyword "def" or the operator "+"). This distinction is vital for writing accurate queries, as patterns can target both types.
Fields: To enhance precision and create more readable, robust queries, grammars can assign "field names" to specific child nodes within a rule. For instance, in a function definition, the node for the function's name might be assigned the field name. This allows developers to access children by a meaningful name using methods like child_by_field_name or, more importantly, to specify these fields directly within a query pattern, such as (function_definition name: (identifier)). This practice is strongly preferred over relying on numerical child indices, as it makes queries more resilient to grammar updates. It is worth noting that the maturity of Tree-sitter grammars varies; older or less-maintained grammars may lack comprehensive field names, forcing a reliance on more brittle, index-based traversal.

The Query Language: An S-expression DSL for Code

Tree-sitter's query engine uses a declarative, Lisp-like language based on S-expressions to find patterns within a CST. This DSL is expressive and allows for the precise targeting of syntactic structures.
Node Patterns: The most basic pattern is a node type enclosed in parentheses, e.g., (call_expression). This will match every node of that type.
Captures: To extract a specific node from a match, the @ symbol is used to create a capture. For example, in (function_definition name: (identifier) @function.name), the identifier node is "captured" with the name @function.name, making it retrievable from the match results.
Fields in Queries: As mentioned, specifying fields makes patterns more robust. The syntax (node_type field: (child_node_type)) constrains the match to nodes where the specified field contains a child of the given type.
Wildcards: The underscore character _ serves as a wildcard. A bare _ matches any node (named or anonymous), while (_) matches any named node.
Alternations: Square brackets `` are used to define a set of alternative patterns. The query will match if any one of the patterns inside the brackets matches. For example, [ (function_declaration) (arrow_function) ] would match both standard function declarations and arrow functions.
Quantifiers: The quantifiers ? (zero or one), * (zero or more), and + (one or more) can be applied to a pattern to match repeated or optional structures.
Predicates: Predicates add conditional logic to queries, filtering matches based on node properties. The most common predicates, #eq? and #match?, filter captures based on their textual content. For instance, (#eq? @function.name "my_func") would ensure that a captured function name is exactly "my_func". Predicates are indispensable for distinguishing between different function calls or variables.

Querying JavaScript and TypeScript Codebases

Modern JavaScript and its typed superset, TypeScript, present a rich and varied syntax, including multiple function declaration styles, module systems, and asynchronous constructs. Crafting effective Tree-sitter queries requires a firm grasp of the corresponding grammar nodes.

The JavaScript/TypeScript Grammar Landscape

The tree-sitter-javascript and tree-sitter-typescript grammars provide the necessary node types for analysis. For modern codebases, it is often advantageous to use the TypeScript parser even for plain JavaScript, as it correctly handles syntax like decorators and type annotations that might otherwise cause parsing errors.
Key node types for this analysis include:
call_expression: Represents a function or method call.
member_expression: Represents property access, e.g., object.property.
new_expression: Represents object instantiation with the new keyword.
function_declaration: A standard function foo() {} declaration.
arrow_function: An arrow function, e.g., () => {}.
method_definition: A method within a class.
import_statement: An ES module import statement.
await_expression: An await expression.

Core Patterns for Function and Method Calls

The foundation of call detection lies in targeting the call_expression node and inspecting its function field.
Simple Function Calls: For a direct call like myFunc(), the function field will be an identifier.
Pattern: (call_expression function: (identifier) @function.name)
Method Calls: For a method call like obj.method(), the function field is a member_expression. The method's name is the property of that expression.
Pattern: (call_expression function: (member_expression property: (property_identifier) @method.name))
Constructor Calls: Instantiation with new is captured via the new_expression node.
Pattern: (new_expression constructor: (identifier) @class.name)
Optional Chaining Calls: Calls using ?., such as obj?.method(), involve an optional_chain node that wraps the call_expression. The core call pattern remains the same, but it will be a child of the optional_chain node.
Pattern: (optional_chain (call_expression function: (member_expression property: (property_identifier) @method.name)))

Distinguishing Definitions from References

A common and critical task is distinguishing where an identifier is defined versus where it is referenced. The queries/locals.scm file, found in many official grammar repositories, provides a robust, battle-tested framework for this exact purpose. It operates on three core capture types: @local.scope, @local.definition, and @local.reference. Any analysis that requires differentiating definitions from uses should adopt this conceptual model.
A node is a definition if it is captured by a @local.definition pattern. A node is a reference if it is an identifier that is not a definition.
Definition Patterns:
Standard function: (function_declaration name: (identifier) @function.def)
Arrow function assigned to a variable: (variable_declarator name: (identifier) @variable.def value: (arrow_function))
Class method: (method_definition name: (property_identifier) @method.def)
Reference Patterns: The most common reference is an identifier being used as the function in a call.
Pattern: (call_expression function: (identifier) @function.ref)

Handling Modern JavaScript Idioms

Arrow Functions: As seen above, arrow functions are typically defined within a variable_declarator. Detecting their definition requires matching this structure.
Async/Await: An awaited call is a call_expression wrapped in an await_expression.
Pattern: (await_expression (call_expression) @awaited.call)
Dynamic Imports: The import() syntax is parsed as a call_expression where the function being called is a special import node.
Pattern: (call_expression function: (import)) @dynamic.import
CommonJS require: Node.js-style require calls are also call_expression nodes. A predicate is needed to ensure the function being called is named require.
Pattern: (call_expression function: (identifier) @func arguments: (arguments (string) @module.path)) (#eq? @func "require")

Table: JavaScript/TypeScript Function Call & Reference Query Patterns

The following table summarizes the most effective query patterns for common analysis tasks in JavaScript and TypeScript.
Analysis Task
Core Node(s)
Effective Query Pattern
Notes/Caveats
Simple Function Call
call_expression, identifier
(call_expression function: (identifier) @function.name)
Captures direct calls like myFunc().
Method Call
call_expression, member_expression
(call_expression function: (member_expression property: (property_identifier) @method.name))
Captures object.method() calls.
Constructor Call
new_expression
(new_expression constructor: (identifier) @class.name)
Captures new MyClass() instantiation.
Function Definition
function_declaration
(function_declaration name: (identifier) @function.def)
Captures function myFunc() {}.
Arrow Function Definition
variable_declarator, arrow_function
(variable_declarator name: (identifier) @function.def value: (arrow_function))
Captures const myFunc = () => {}.
Variable Reference
identifier
(identifier) @reference
General-purpose. Must be filtered to exclude definition sites.
Dynamic import()
call_expression, import
(call_expression function: (import) arguments: (arguments (string) @path))
Captures import('./module.js').
CommonJS require()
call_expression, identifier
(call_expression function: (identifier) @func) (#eq? @func "require")
Captures require('module').


Querying Python Codebases

Python's grammar, while generally more regular than JavaScript's, has unique syntactic features like decorators that require specific query patterns for accurate analysis.

The Python Grammar Landscape

Analysis of Python code relies on the node types defined in the tree-sitter-python grammar. Key nodes for this discussion are:
call: Represents a function or method call.
attribute: Represents property access, e.g., object.attribute.
identifier: A name for a variable, function, or class.
function_definition: A def block.
class_definition: A class block.
decorated_definition: A definition (class or function) that is preceded by one or more decorators.
decorator: The @decorator syntax itself.

Core Patterns for Function Calls and Class Instantiation

Simple Function/Class Calls: A direct call to a function or a class constructor (e.g., my_func() or MyClass()) is represented by a call node whose function field is an identifier.
Pattern: (call function: (identifier) @function.call)
Method Calls: A method call (e.g., my_obj.method()) is a call node where the function field is an attribute node. The method name is the attribute field of that node.
Pattern: (call function: (attribute attribute: (identifier) @method.call))
Distinguishing Constructors: Python does not have a new keyword. By convention (PEP 8), class names are capitalized. This convention can be leveraged in a query using a predicate to specifically find likely constructor calls.
Pattern: ((call function: (identifier) @constructor.call) (#match? @constructor.call "^[A-Z]"))

Navigating Decorated Functions

A frequent point of failure in naive Python analysis is incorrectly handling decorators. In the Python CST, a decorator is not a simple property of a function. Instead, a decorated_definition node acts as a wrapper around the function_definition (or class_definition). The decorators themselves are children of this wrapper node.
This structure means that a simple query for (function_definition) will fail to find any function that has a decorator. To robustly find all function definitions, one must account for both plain and decorated definitions.
Effective Pattern for Decorated Functions: This pattern captures the decorator(s) and the name of the function they apply to.
Scheme
(decorated_definition
  (decorator) @decorator
  definition: (function_definition
    name: (identifier) @function.name
  )
)


Finding All Functions: To capture all functions, regardless of decoration, an alternation is the most effective approach.
Pattern: [ (function_definition) (decorated_definition) ] @any.function

Distinguishing Definitions from References

Definition Patterns: The primary node for a function definition is (function_definition name: (identifier) @function.def). This can be inside a decorated_definition or stand alone. Variable assignments ((assignment left: (identifier) @var.def)) are another key definition site.
Reference Patterns: As with other languages, an (identifier) @reference that appears in a context of use, such as the function field of a (call) node, constitutes a reference.

Table: Python Function Call & Reference Query Patterns

This table provides a quick reference for the most common query tasks in Python.
Analysis Task
Core Node(s)
Effective Query Pattern
Notes/Caveats
Simple Function Call
call, identifier
(call function: (identifier) @function.call)
Also matches class instantiations.
Method Call
call, attribute
(call function: (attribute attribute: (identifier) @method.call))
Captures object.method().
Constructor Call
call, identifier
((call function: (identifier) @constructor.call) (#match? @constructor.call "^[A-Z]"))
Relies on the PEP 8 naming convention.
Function Definition
function_definition
(function_definition name: (identifier) @function.def)
Captures undecorated functions.
Decorated Function
decorated_definition
(decorated_definition definition: (function_definition name: (identifier) @function.def))
Captures the function part of a decorated definition.
Decorator
decorator
(decorator) @decorator
Captures the decorator itself, e.g., @my_decorator.
Variable Reference
identifier
(identifier) @reference
General-purpose. Must be filtered to exclude definition sites.


Querying Rust Codebases

Rust's grammar is known for its complexity and power, driven by features like an advanced type system, lifetimes, and a pervasive macro system. Querying Rust code effectively requires careful attention to these unique language features.

The Rust Grammar Landscape

The tree-sitter-rust grammar defines the node types for parsing Rust code. Key nodes for detecting calls and references include:
call_expression: A function or method call.
field_expression: Field or method access on a struct, e.g., my_struct.field.
scoped_identifier: A path-qualified identifier, e.g., std::vec::Vec.
function_item: A top-level or impl block function definition.
macro_invocation: The use of a function-like or attribute macro.
use_declaration: An use statement for importing items into scope.

Core Patterns for Function and Method Calls

Rust has several distinct syntaxes for invoking callable code, each with its own CST structure.
Simple Function Calls: A call to a function in the local scope.
Pattern: (call_expression function: (identifier) @function.call)
Method Calls: A call to a method on an instance, using dot-notation. The function being called is a field_expression.
Pattern: (call_expression function: (field_expression field: (field_identifier) @method.call))
Associated Functions (Static Methods): A call to a function associated with a type, using the turbofish :: syntax (e.g., String::new()). The function is a scoped_identifier.
Pattern: (call_expression function: (scoped_identifier name: (identifier) @function.call))

The Challenge of Metaprogramming: Querying Macros

One of the most significant challenges in syntactically analyzing Rust is its powerful macro system. It is crucial to understand that Tree-sitter parses the source code before macro expansion. It sees the macro invocation itself (e.g., println!();) but is completely unaware of the code that the macro will generate. This is a fundamental boundary of purely syntactic analysis.
Consequently, any attempt to find a "call" to println! using a call_expression query will fail. Instead, one must explicitly query for the macro_invocation node.
Function-like Macro Invocation: This pattern captures calls like vec!.
Pattern: (macro_invocation macro: (identifier) @macro.name "!" (token_tree) @macro.args)
Attribute Macros: These are attached to items like functions or structs.
Pattern: (attribute_item (attribute (identifier) @attribute.macro))
This distinction is paramount: for tools that need to analyze the code after macro expansion, Tree-sitter is only the first step. True semantic analysis requires integration with a tool like rust-analyzer, which performs macro expansion and type resolution.

Syntactic Underpinnings of Definitions and References

Definition Patterns: The primary node for a function definition is (function_item name: (identifier) @function.def).
Reference Patterns: An (identifier) @reference appearing within a call_expression or as a variable in an expression is a reference. Again, the locals.scm framework is the canonical way to distinguish these from definitions within their respective scopes.

Table: Rust Function Call & Reference Query Patterns

This table summarizes the essential patterns for Rust, emphasizing the critical distinctions between its various call syntaxes.
Analysis Task
Core Node(s)
Effective Query Pattern
Notes/Caveats
Simple Function Call
call_expression, identifier
(call_expression function: (identifier) @function.call)
For functions in the current scope.
Method Call
call_expression, field_expression
(call_expression function: (field_expression field: (field_identifier) @method.call))
For instance.method() calls.
Associated Function Call
call_expression, scoped_identifier
(call_expression function: (scoped_identifier name: (identifier) @function.call))
For Type::function() calls.
Function-like Macro
macro_invocation
(macro_invocation macro: (identifier) @macro.name)
Captures my_macro!(...). Does not see expanded code.
Function Definition
function_item
(function_item name: (identifier) @function.def)
Captures fn my_func() {}.
Variable Reference
identifier
(identifier) @reference
General-purpose. Must be filtered to exclude definition sites.
use Declaration
use_declaration
(use_declaration) @import
Captures an entire use statement.


Advanced Techniques and Cross-Language Best Practices

While each language has its grammatical idiosyncrasies, several high-level principles and techniques apply universally when writing effective and performant Tree-sitter queries.

A Comparative Analysis of Call-Site Patterns

A direct comparison of the primary node structures for accessing properties or methods reveals subtle but important differences in grammar design.
JavaScript: (member_expression object: _ property: (property_identifier))
Python: (attribute object: _ attribute: (identifier))
Rust: (field_expression value: _ field: (field_identifier))
These variations, though seemingly minor, underscore a critical point: a "one-size-fits-all" query for a concept like "method call" is impossible. The node names (member_expression, attribute, field_expression) and field names (property, attribute, field) are specific to each language's grammar. This illustrates that effective querying is fundamentally tied to a solid understanding of the target language's specific CST structure.

Crafting Precise and Robust Queries

Prioritize Fields over Indices: Always use named fields in queries when available. A query like (function_definition name: (identifier)) is far more robust and readable than a query that relies on child indices, e.g., (function_definition (identifier)), which could break if the grammar is updated to add a new node before the name.
Use Anchors for Positional Precision: The . anchor operator provides powerful control over a pattern's position relative to its siblings. For example, (block. (_)) ensures a node is the first child of a block, while (block (_).) ensures it is the last. This can dramatically reduce ambiguity and prevent false positives.
Leverage locals.scm: For any project that needs to resolve definitions and references, the locals.scm file provides the canonical framework. Implementing queries for @local.scope, @local.definition, and @local.reference is the most reliable way to build features like "go to definition" or "find all references" on a syntactic level.

Performance Considerations and Anti-Patterns

While Tree-sitter is remarkably fast, poorly constructed queries can lead to significant performance degradation, especially on large files. Community discussions and the library's own documentation reveal that query performance is directly tied to how "local" a pattern is.
The py-tree-sitter documentation explicitly notes that "non-local" patterns—those with multiple root nodes that can match across repeating sequences—disable certain optimizations. When a query cursor is executed, it must still traverse any large node that intersects with its target range, even if the final match is small. Therefore, broad, unspecific queries force the engine to do more work.
Anti-Patterns to Avoid:
Overuse of Wildcards: A top-level query like ((_) @anything) forces the engine to match every single named node in the tree. Wildcards should be used sparingly and within more specific parent patterns.
Deeply Nested, Un-Fielded Patterns: Patterns like (a (b (c (d @capture)))) without field specifiers can lead to excessive backtracking and slow performance.
Non-Local Patterns: Queries with multiple, un-anchored top-level patterns can be slow as they prevent the query engine from making assumptions about the structure of the tree.
Optimization Strategies:
Be Specific: The more constrained a query is, the faster it will run. Use fields, anchors, and specific node types.
Narrow the Scope: Whenever possible, execute queries on a specific subtree (e.g., a single function body) rather than the entire file's root node.
Understand Host Integration: Be aware of how the host application (e.g., Neovim, VS Code) executes queries. Some integrations may re-run queries on every keystroke, making performance even more critical.

Conclusion and Strategic Recommendations

Tree-sitter provides an exceptionally powerful and performant system for the structural analysis of source code. By understanding the specific node types and idiomatic structures of each language's grammar, developers can craft precise queries to detect function calls, definitions, and references in Rust, Python, and JavaScript. The canonical patterns for these tasks involve targeting nodes like call_expression (or call), member_expression (or attribute/field_expression), and function_declaration (or function_item/function_definition), while leveraging fields and predicates for disambiguation.
Two strategic recommendations stand out for any developer embarking on a project with Tree-sitter:
Consult Existing Query Files: Before writing a query from scratch, developers should thoroughly examine the queries directory of mature, widely-used projects. The highlights.scm, locals.scm, folds.scm, and indents.scm files within repositories like nvim-treesitter or the official Tree-sitter grammars are a goldmine of battle-tested, community-vetted patterns that solve common problems effectively.
Respect the Syntactic/Semantic Divide: It is crucial to recognize the boundaries of Tree-sitter's capabilities. It is a tool for syntactic analysis. It excels at understanding the structure of code as it is written in a single file. However, for tasks that require semantic understanding—such as type resolution, cross-file definition tracking, or analyzing the result of Rust macro expansion—Tree-sitter is the necessary first step, not the final solution. These more advanced analyses require integrating the CST from Tree-sitter with semantic engines, most notably those provided by Language Server Protocol (LSP) implementations like rust-analyzer.
By embracing these principles and leveraging the detailed patterns provided in this report, developers can unlock the full potential of Tree-sitter to build the next generation of intelligent, responsive, and syntax-aware developer tools.
