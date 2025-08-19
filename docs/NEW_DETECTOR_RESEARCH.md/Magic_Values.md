
Detecting and Remediating Magic Values: A Comprehensive Guide to Static Analysis Techniques Across Modern Programming Languages


I. The Anatomy of Magic Values: From Anti-Pattern to Actionable Insight

The term "magic value" refers to a long-standing anti-pattern in software development, a practice that, while seemingly innocuous, introduces significant friction into the maintenance, readability, and reliability of a codebase. This section establishes a formal definition of magic values, provides a taxonomy of their various forms, explores the fundamental software engineering principles they violate, and meticulously details the crucial exceptions where literals are not only acceptable but preferable. The central thesis is that the "magic" quality of a literal is determined not by its value, but by its lack of self-documenting context.

1.1 Formal Definitions: A Taxonomy of Unexplained Literals

At its core, a magic value is a literal—a fixed value represented directly in source code—that lacks an explicit, self-documenting explanation of its meaning or purpose.1 This anti-pattern is language-agnostic and represents a violation of one of the oldest principles of programming, dating back to the manuals for COBOL and FORTRAN in the 1960s.1 The issue is not limited to numeric types; the term applies to any data type where replacing the literal with a named constant would render the code more flexible and communicative.1 A comprehensive understanding requires a taxonomy of these values.

1.1.1 Taxonomy of Magic Numbers

Magic numbers are the most frequently cited form of this anti-pattern. They can be categorized based on their intended purpose.
Unnamed Numerical Constants: This is the canonical form, where a numeric literal represents a physical constant, a business rule, a configuration parameter, or a threshold without any context. An expression like price_tax = 1.05 * price is a classic example where 1.05 is a magic number representing a 5% tax rate.1 These values obscure developer intent and make the code difficult to modify.
Format Indicators: These are constant numerical or text values used to identify a file format or protocol. Examples include 0xCAFEBABE for Java class files or 0x89504E47 for PNG files.1 While technically "magic" by name, these values are part of a public specification and are often handled as special cases by analysis tools, as their meaning is fixed and global.5
GUIDs and Unique Identifiers: Globally Unique Identifiers (GUIDs) or Universally Unique Identifiers (UUIDs) are distinctive values that are inherently opaque. Their purpose is uniqueness, not human readability, and thus they are not candidates for replacement with a named constant.1
Debug Values: During development and debugging, specific, non-standard values are often used to mark memory regions. For example, 0xDEADBEEF might be used to fill uninitialized memory, or 0xABADCAFE to mark freed memory.1 These values are chosen to be easily recognizable in a debugger and are not intended for production logic.

1.1.2 Taxonomy of Magic Strings

The concept extends seamlessly to string literals, which can introduce similar, if not more dangerous, issues.
Behavior-Controlling Strings: These are string literals that dictate program flow, often used as keys in dictionaries, event names, or states in a finite state machine.3 For instance, using
"ERR001" in a switch statement makes the code brittle. A simple typo, like "ERRO01", would not be caught at compile time in many languages but would lead to a silent failure at runtime.8
Hidden Functionality Triggers: In some cases, developers embed unlikely string inputs to activate hidden functionality, often for testing or debugging purposes. For example, entering "***" as a credit card number might bypass validation logic.9 If left in production code, these can be inadvertently triggered by users or exploited as backdoors.

1.2 The Rationale for Avoidance: A Deep Dive into Readability, Maintainability, and Reliability Costs

The persistent admonition against magic values stems from the significant technical debt they accrue, impacting three core pillars of software quality: readability, maintainability, and reliability.
Readability: Magic values force future readers of the code, including the original author, to deduce the literal's meaning from its surrounding context. This increases cognitive load and makes the code harder to understand.2 A number like
52 in a loop might represent the number of weeks in a year or the number of cards in a deck; without a named constant, the intent is ambiguous.1 This ambiguity is compounded when the same value is used for different semantic purposes within the same codebase. For example, the number
60 could mean seconds in a minute in one function and minutes in an hour in another, creating a high potential for confusion.11
Maintainability: The use of magic values is a direct violation of the Don't Repeat Yourself (DRY) principle.12 A single conceptual value, such as a timeout duration or a maximum retry count, may be scattered as a literal across multiple files and functions. When this value needs to be changed, a developer must hunt down every occurrence, a process that is both laborious and highly susceptible to error.2 A global search-and-replace is a perilous strategy, as the same numeric value might be used for entirely different reasons in different contexts.8 Replacing all instances of
10 could inadvertently break logic where 10 meant something other than the intended MAX_RETRY_COUNT. A named constant provides a single, unambiguous, authoritative representation of that piece of knowledge.12
Reliability: Magic values introduce opportunities for subtle, hard-to-detect bugs. A typo in a complex numeric constant like 3.14159265 is difficult to spot visually and will not be flagged by a compiler.1 In contrast, a typo in a named constant like
GRAVITATIONAL_CONSTANT would result in a compile-time error, preventing the bug from ever reaching production.10 This is particularly critical for magic strings in dynamically typed languages, where a misspelled key or state name leads to silent runtime failures instead of compile-time errors.8
The presence of magic values often signals a deeper architectural deficiency: a missing or poorly defined domain concept. For example, hardcoding user roles as integer literals (1 for admin, 2 for user) is not merely a magic number problem; it is a failure to model the concept of a "User Role" as a proper, type-safe entity. The initial fix might be to create constants like const ADMIN_ROLE = 1; and const USER_ROLE = 2;. However, this is an incomplete solution. A variable holding this role can still be assigned an invalid integer like 99. This reveals that the primitive type (int) is an insufficient representation of the domain concept. The more robust refactoring is to introduce a type-safe abstraction, such as an enum (UserRole.ADMIN), which encapsulates the concept and restricts the set of possible values at the type-system level. Therefore, a high density of related magic numbers serves as a strong indicator for areas where the application's domain model is underdeveloped and requires more formal abstraction.15

1.3 The Principle of Context: Commonly Accepted Exceptions and Their Justifications

The rule against magic values is not absolute. Its application requires nuance, as the "magic" quality is fundamentally about context. Certain literals are so universally understood in specific programming contexts that abstracting them would harm readability rather than improve it.
The Universal Exceptions: The values 0, 1, and -1 are almost universally exempt from magic number rules by static analysis tools.12 Their meaning is typically self-evident from their immediate syntactic context.
0: Used for initializing counters in loops (for (int i = 0;...)), representing the base index of an array, checking for emptiness (collection.size() == 0), or as the null pointer representation in languages like C and C++.1
1: Used for incrementing or decrementing counters (i += 1) or as a multiplicative/divisive identity.
-1: A widely accepted sentinel value indicating "not found" from search functions (e.g., String.indexOf) or a generic error state.16
Contextual Idioms: Other small integers are often considered acceptable when used in well-established programming idioms.
2: Commonly used in modulo operations to check for even or odd numbers (x % 2 == 0).1
Powers of 10 (10, 100, 1000): Frequently used for metric conversions or percentage calculations, where their meaning is clear from the operation.1
The Gray Area: The boundary of what constitutes a magic number becomes subjective for values that are obvious to a domain expert but not a novice. An expression like let secondsInWorkday = hours * 60 * 60 is arguably self-documenting.12 Introducing constants for
MINUTES_PER_HOUR and SECONDS_PER_MINUTE could make the expression more verbose without adding significant clarity, especially if those constants are not used elsewhere. This subjectivity underscores the need for configurable tooling that allows teams to define their own standards.12
Strings as Exceptions: Certain categories of string literals are also typically exempt.
Logging and Tracing: Strings intended for outputting log messages are not behavior-controlling and are almost always ignored.12
Punctuation and Delimiters: Simple, universal strings like ",", ";", or " " are considered acceptable as their purpose is purely syntactic and self-evident from context.18
Ultimately, the practical, working definition of a "magic number" is heavily influenced and enforced by the default configurations of the static analysis tools prevalent in a given language ecosystem. A Java developer using Checkstyle, for instance, learns that -1, 0, 1, and 2 are acceptable because the default MagicNumberCheck configuration explicitly ignores them.19 This tool-driven feedback loop establishes a community-wide convention that is more specific and rigid than the abstract principle of avoiding unexplained literals. The developer's mental model of what constitutes "good code" is thus shaped not just by theory but by the daily reinforcement provided by their tooling.

II. Algorithmic Detection via Abstract Syntax Tree Analysis

Modern static analysis tools detect magic values not by parsing raw text with regular expressions, but through a sophisticated analysis of the code's structural and semantic representation: the Abstract Syntax Tree (AST). This approach allows tools to understand the context in which a literal appears, enabling the nuanced distinction between a problematic magic value and a legitimate, self-explanatory constant.

2.1 The Role of the AST in Lexical and Syntactic Analysis

An AST is a hierarchical, tree-like representation of source code, generated by a parser during the compilation process.21 It captures the essential structure of the code, abstracting away syntactic details like whitespace, comments, and parentheses that are irrelevant to the program's meaning. Each node in the tree represents a construct in the code, such as a variable declaration, a function call, or a literal value. By operating on the AST, static analysis tools can reason about the code's structure and semantics in a way that is impossible with simple text-based methods.21 For example, an AST can distinguish between the number
10 used as an array index and the same number used as a function argument, because they will have different parent nodes in the tree.

2.2 The Core Algorithm: A Language-Agnostic Approach

Despite language-specific differences in AST structure, the fundamental algorithm for detecting magic values is remarkably consistent across tools and ecosystems. It is typically implemented using the Visitor design pattern to traverse the AST.21
Parse Source Code: The tool first parses the source code into a language-specific AST.
Instantiate a Visitor: A custom visitor class is created, designed to "visit" specific types of nodes in the tree.
Implement visit Methods: The visitor implements methods for the node types corresponding to literals (e.g., visit_NumericLiteral, visit_StringLiteral, visit_Constant).
Traverse the AST: The visitor is dispatched to traverse the entire AST, starting from the root node. As it encounters each node, it calls the appropriate visit method.
Apply Detection Logic: Inside the visit method for a literal node, the core detection logic is executed:
Value Check: The literal's value is first checked against a configurable allow-list of universally accepted values (e.g., 0, 1, -1). If it matches, the analysis for this node terminates.
Contextual Analysis: If the value is not on the allow-list, the tool performs contextual analysis by examining the literal's parent and ancestor nodes to determine its role in the code.

2.3 Contextual Analysis: Using Parent and Ancestor Nodes to Classify Literals

Contextual analysis is the most critical and sophisticated part of the detection algorithm. By inspecting the chain of parent nodes leading from a literal, a tool can build a "syntactic fingerprint" that reliably classifies its usage. This fingerprint is a language-independent pattern that allows tools to implement highly specific heuristics for excluding legitimate literals.
Common Exclusionary Contexts (Heuristics):
Constant Declaration: The most important exclusion is a literal used to initialize a constant. The tool checks if an ancestor node is a variable or field declaration marked with a const, final, or equivalent keyword.19
Array Index: A literal used as an array index (e.g., data) is often permissible. The tool identifies this by checking if the parent node is an array access expression.27
Enum Member: A literal used to define the value of an enum member is part of a type-safe constant definition and is always ignored.29 The parent node would be an
EnumMember or similar construct.
Annotation/Decorator Argument: Literals used within annotations (Java) or decorators (Python/JS) are often configuration values and can be ignored based on tool settings.19
The Java Checkstyle tool's MagicNumberCheck provides a powerful implementation of this concept with its constantWaiverParentToken property. This property allows users to define an explicit list of AST node types (such as ASSIGN, ARRAY_INIT, METHOD_CALL, PLUS, DIV) that are considered "safe" parents for a literal, provided that the literal is ultimately part of a final variable's definition. This demonstrates how the AST path provides a robust signature for classifying a literal's context.19

2.4 Language-Specific AST Patterns and Code Examples

While the core algorithm is similar, the specific AST node types and traversal patterns differ for each language.

2.4.1 Rust (syn Crate)

Linters like Clippy and procedural macros in the Rust ecosystem use the syn crate for parsing.
Key AST Node: syn::Lit. This is an enum with variants like Lit::Int, Lit::Float, and Lit::Str for different literal types. These are typically wrapped in an syn::Expr::Lit node.
Detection Pattern: A visitor traverses the AST, looking for Expr::Lit nodes. Upon finding one, it inspects the literal's value. For contextual analysis, the visitor must track the parent nodes to determine if the literal is part of a const item (syn::ItemConst) or a static item (syn::ItemStatic), which are considered valid constant declarations. A literal found elsewhere, such as in the body of a function initializing a mutable let binding, would be flagged.
Example:

Rust


const MAX_CONNECTIONS: u32 = 100; // OK: LitInt is within an ItemConst.
fn configure() {
    let timeout = 5; // Potential Magic Number: LitInt is within a Local binding.
}



2.4.2 Python (ast Module)

Python's standard library includes the ast module, which is the canonical way to process Python source code into an AST.21
Key AST Node: ast.Constant. In Python 3.8 and later, this single node represents all literal types (numbers, strings, booleans, None). It has a value attribute containing the literal's actual value. Older versions used separate nodes like ast.Num and ast.Str, which are now deprecated.33 This language evolution simplifies the AST, but tools must handle both forms to support older Python versions. This demonstrates a direct link between a language's grammar evolution and the architecture of its static analysis tooling.
Detection Pattern: A visitor class inheriting from ast.NodeVisitor implements a visit_Constant method. Inside this method, it checks the type of node.value. To determine context, the visitor must be augmented to track parent nodes (the ast module does not do this by default). The logic then checks if the parent is an ast.Assign node where the target is a variable name that follows the UPPER_SNAKE_CASE convention for constants.
Example:

Python


MAX_USERS = 1000  # OK: Constant parent is Assign, target name 'MAX_USERS' follows convention.
if user_count > 1000:  # Potential Magic Number: Constant is a child of a Compare node.



2.4.3 JavaScript (ESTree)

The JavaScript ecosystem has largely standardized on the ESTree specification, ensuring that ASTs produced by different parsers (like Esprima, Acorn, or Babel) are compatible.36
Key AST Node: Literal. This node has a value property holding the literal's value and a raw property with its source text representation. Its type property is the string 'Literal'.36
Detection Pattern: A visitor traverses the tree looking for nodes with type: 'Literal'. The contextual analysis involves checking the type of the parent node. If the parent is a VariableDeclarator whose kind property is 'const', the literal is part of a constant definition. If the parent is a CallExpression, it is a function argument and likely a magic value.
Example (viewable in AST Explorer 38):

JavaScript


const TIMEOUT_MS = 5000; // OK: Literal parent is VariableDeclarator with kind: 'const'.
setTimeout(callback, 5000); // Potential Magic Number: Literal is an argument in a CallExpression.



2.4.4 Java (Eclipse JDT)

The Eclipse Java Development Tools (JDT) provide a comprehensive API for parsing and analyzing Java code.25
Key AST Nodes: JDT uses specific classes for different literals, such as NumberLiteral, StringLiteral, and CharacterLiteral, all of which inherit from ASTNode.22
Detection Pattern: An ASTVisitor subclass overrides the visit methods for each literal type (e.g., public boolean visit(NumberLiteral node)). Inside the method, the logic traverses up the ancestor chain using node.getParent(). It checks if any ancestor is a FieldDeclaration or VariableDeclarationStatement that has the final modifier flag set. If not, the literal is flagged.
Example:

Java


private static final int MAX_RETRIES = 3; // OK: NumberLiteral's ancestor is a final FieldDeclaration.
if (retries < 3) { /*... */ } // Potential Magic Number: NumberLiteral is in an InfixExpression.



2.4.5 C++ (Clang AST Matchers)

Clang offers the most powerful and expressive system for AST analysis through its LibTooling library and AST Matchers, a C++-based DSL for declaring patterns in the AST.42
Key AST Matchers: integerLiteral() and floatLiteral() are the primary node matchers for numeric literals. These can be combined with narrowing and traversal matchers to build complex queries.
Detection Pattern: Instead of a procedural visitor, developers write declarative matchers. To find magic numbers, one would construct a matcher that finds all literals unless they satisfy certain exclusionary conditions. For example, a matcher could be integerLiteral(unless(hasAncestor(varDecl(anyOf(isStaticLocal(), hasGlobalStorage()))))). This finds any integer literal that does not have an ancestor that is a static or global variable declaration. The readability-magic-numbers check in Clang-Tidy uses this approach to implement its logic.45
Example:

C++


constexpr int TIMEOUT_SECONDS = 10; // OK: integerLiteral has an ancestor varDecl that is constexpr.
sleep(10); // Potential Magic Number: integerLiteral is an argument to a CallExpr.



III. Advanced Heuristics for Intelligent Filtering

While basic detection relies on an allow-list and simple syntactic context, truly effective static analysis requires more advanced heuristics to minimize false positives and provide genuinely useful feedback. These heuristics approximate a developer's semantic understanding by recognizing common programming idioms and value-based patterns, thereby distinguishing between a truly "magic" number and one whose meaning is implicitly clear from its usage.

3.1 Beyond Simple Allow-Lists: Developing Context-Aware Heuristics

The primary goal of advanced heuristics is to codify the unwritten rules and patterns that experienced developers use to interpret code.48 A naive linter that flags every literal not in a
const declaration will quickly be disabled due to excessive noise. A useful linter must therefore learn to recognize these idiomatic exceptions. The set of heuristics in a mature tool can be seen as a form of "lossy compression" of community-accepted programming practices, capturing the collective wisdom of when a number's meaning is self-evident. For example, when an experienced programmer sees x & 16, they immediately recognize it as a bitmask operation to check the fifth bit; 16 is not a magic number in this context. A linter option like Clang-Tidy's IgnorePowersOf2IntegerValues is the explicit implementation of this learned idiom, making the tool more practical.45

3.2 Value-Based Heuristics

These heuristics analyze the literal's value itself to infer its purpose.
Powers of Two: In bitwise operations (&, |, ^, <<, >>), literals that are powers of two (2, 4, 8, 16,..., 1024, etc.) are almost never magic numbers. They represent specific bits or bitmasks, and their meaning is derived from the binary representation. Many tools, such as Clang-Tidy, provide an option to automatically ignore all powers of two to avoid flagging these legitimate use cases.45
Hexadecimal Patterns: Certain hexadecimal values are idiomatic and self-describing to developers familiar with low-level programming. Values like 0xFF (a full byte), 0xFFFF (a 16-bit word), or color codes like 0xFF0000 (pure red) often do not require a named constant to be understood.52
Known Mathematical and Physical Constants: Advanced linters can be pre-programmed with knowledge of common constants. For example, Rust Clippy's approx_constant lint detects floating-point literals that are close approximations of constants like π or e and suggests replacing them with the more precise and explicit standard library versions (e.g., std::f64::consts::PI).53

3.3 Syntactic Heuristics: Identifying Idiomatic Usage

These heuristics examine the immediate syntactic structure surrounding a literal.
Loop Constructs: The use of 0 and 1 in the initialization and increment parts of a for loop is a universal idiom and is always excluded.1 The loop's boundary condition (e.g.,
i < 10) is a more classic magic number candidate, but even here, context matters.
Arithmetic Expressions in Constant Definitions: A powerful heuristic is to not flag literals that are part of a self-documenting calculation within a constant's definition. In the statement const SECONDS_PER_DAY = 24 * 60 * 60;, the literals 24, 60, and 60 should not be flagged individually. The expression itself explains their meaning and how the final value is derived.26 A sophisticated tool must analyze the entire right-hand side of the assignment, not just the leaf nodes of the AST.
Function and Method Call Context: The name of a function can provide significant context. The number 5000 in set_timeout_ms(5000) is less "magic" than in process_data(5000).4 However, this is a weaker heuristic and can be misleading. For example, in
set_height(6), the unit is still ambiguous (feet, meters, inches?), so the number remains magic despite the function's name.55
The design and effectiveness of these heuristics are fundamentally influenced by the language's typing system. In statically-typed languages like Rust, Java, and C++, the analyzer can leverage rich type information. If a function expects an argument of an enum type, any integer literal passed to it is a clear violation. In dynamically-typed languages like Python and JavaScript, the tool lacks this definitive type information at analysis time. It must rely on weaker heuristics, such as naming conventions (e.g., assuming a variable in ALL_CAPS is a constant) or structural inference. The development of TypeScript-specific extensions for ESLint, such as ignoreEnums and ignoreNumericLiteralTypes, perfectly illustrates this principle: the linter's precision and power increase dramatically when it can access the type system that TypeScript layers on top of JavaScript.27

3.4 Semantic Heuristics: The Final Frontier and Its Challenges

The ultimate goal of a magic number detector is to understand the developer's semantic intent—a task that pushes the boundaries of static analysis.
The Core Challenge: A tool cannot possess the domain-specific knowledge required to know that, in a particular financial application, the number 3 always signifies "maximum number of retries for a transaction".56 This semantic gap is the primary source of false positives and is where human judgment remains essential.
Frequency Analysis: A potential heuristic for prioritizing warnings is to analyze the frequency of a literal's use. A numeric value that appears multiple times across a codebase is a much stronger candidate for a magic value that should be a constant. A value used only once, in a localized context, might be less critical to refactor.8 A linter could use this frequency count to adjust the severity of the reported issue, flagging repeated literals as more critical than one-off values.
Linguistic and AI-Based Analysis: The frontier of this field involves applying natural language processing (NLP) and machine learning techniques to the source code.59 An advanced tool could analyze the names of variables, functions, and classes associated with a literal. For example, if the number
100 is consistently used in contexts involving variables named percentage, progress, or ratio, the tool could infer its meaning and even suggest a constant name like PERCENTAGE_COMPLETE. This approach moves from syntactic pattern matching to a form of semantic inference, representing the next evolution in static analysis tooling.

IV. A Comparative Analysis of Linter Implementations

The theoretical principles of magic value detection are put into practice by static analysis tools, or "linters," each with its own philosophy, feature set, and configuration options. This section provides a comparative analysis of the leading linters for Rust, Python, JavaScript/TypeScript, Java, and C++, highlighting their distinct approaches to solving this common problem.

4.1 Rust (Clippy)

Clippy, the official and canonical linter for Rust, embodies the language's focus on idiomatic correctness. Rather than a single, broad "magic number" rule, Clippy provides a collection of highly specific lints that target situations where a literal is likely a mistake or a less-idiomatic alternative to a language feature.
Philosophy: Clippy prioritizes high-confidence warnings and encourages the use of Rust's strong type system, particularly enum and const, to eliminate magic values by design. It avoids a general-purpose lint that might be overly noisy.
Key Lints:
approx_constant: This lint detects floating-point literals that are close approximations of mathematical constants defined in the standard library (e.g., 3.14159). It suggests using the more precise and self-documenting official constant, such as std::f64::consts::PI.53
unusual_byte_groupings: This highly specific lint warns about certain hexadecimal integer literals (e.g., 0xCAFEBABE, 0xDEADBEEF) that are often used as format identifiers or debug values. The goal is to catch potential typos in these well-known "magic" constants.5
Configuration: Lints are configured in a clippy.toml file. While individual lints can be configured, there is no central mechanism for defining a project-wide list of allowed magic numbers, reinforcing the philosophy of addressing these issues through better code structure rather than tool configuration.62

4.2 Python (Pylint & Ruff)

The Python ecosystem has traditionally focused on a more targeted application of magic value detection, primarily within comparison contexts where ambiguity is most likely to cause logical errors.
Philosophy: The focus is on a specific, high-impact code smell: using unexplained literals directly in comparison operations. This is considered a "refactoring" issue.
Key Rule: magic-value-comparison (Pylint rule R2004). This check is part of an optional extension in Pylint and must be explicitly enabled. It flags code like if score > 100: but would ignore timeout = 100.64
Configuration: Pylint allows users to specify an allow-list of numbers via the valid-magic-values option in the configuration file.66 Ruff, a modern, high-performance linter written in Rust, implements a compatible version of this rule (
PLR2004) and offers similar configuration options, making it a popular alternative.67

4.3 JavaScript/TypeScript (ESLint)

ESLint provides the most flexible and extensively configurable rule for magic number detection, reflecting the dynamic nature of JavaScript and the diverse coding styles within its community.
Philosophy: To provide a powerful, general-purpose rule that can be tailored to fit any project's specific standards, from lenient to extremely strict.
Key Rule: no-magic-numbers. This rule is comprehensive and flags almost any numeric literal that is not part of a variable declaration.27
Configuration: The rule's power lies in its extensive configuration object:
ignore: An array of numbers to be universally allowed (e.g., [0, 1, -1]).
ignoreArrayIndexes: A boolean to permit literals used as array indices.
ignoreDefaultValues: A boolean to allow literals in default function parameters and object destructuring.
enforceConst: A boolean that requires variables holding numbers to be declared with const.
detectObjects: A boolean to extend the check to literals used as object property values.
TypeScript Integration: The @typescript-eslint/no-magic-numbers extension rule enhances the base rule by leveraging TypeScript's type system. It adds options like ignoreEnums, ignoreNumericLiteralTypes, and ignoreReadonlyClassProperties, allowing for much more precise and context-aware analysis.29

4.4 Java (Checkstyle)

Checkstyle offers a robust, AST-aware implementation that provides deep, structural control over where literals are permitted.
Philosophy: To enable fine-grained control based on the syntactic context of a literal, leveraging a deep understanding of the Java AST.
Key Rule: MagicNumberCheck.19
Configuration:
ignoreNumbers: The primary allow-list, which defaults to -1, 0, 1, 2.
Contextual Booleans: Flags like ignoreHashCodeMethod, ignoreAnnotation, and ignoreFieldDeclaration allow developers to disable the check for entire categories of code constructs.
constantWaiverParentToken: This is Checkstyle's most advanced feature. It allows users to specify a list of AST parent node types (e.g., ASSIGN, PLUS, METHOD_CALL) that are considered "safe" contexts for a literal, provided it is part of a final variable's initialization expression. This offers unparalleled control over the detection logic.31

4.5 C++ (Clang-Tidy)

Clang-Tidy, part of the Clang/LLVM toolchain, provides a check that is well-aligned with modern C++ best practices and major coding standards.
Philosophy: To enforce rules from established guidelines like the C++ Core Guidelines (ES.45) and High Integrity C++ (5.1.1), promoting the use of symbolic constants over raw literals.45
Key Rule: readability-magic-numbers.45
Configuration: The check is highly configurable to accommodate various idiomatic uses of literals in C++:
IgnoredIntegerValues and IgnoredFloatingPointValues: Semicolon-separated allow-lists for integer and floating-point types.
IgnorePowersOf2IntegerValues: A boolean to exempt all powers of two, common in bitwise manipulation.
IgnoreBitFieldsWidths: A boolean to allow literals when defining the width of bit-fields in structs, a frequent pattern in systems and embedded programming.
IgnoreTypeAliases: A boolean to allow literals in typedef and using declarations.

4.6 Table IV.1: Comparative Analysis of Magic Value Detection Tools

The following table synthesizes the key characteristics of the linters discussed, providing a high-level overview for comparison.
Language/Ecosystem
Tool
Rule Name(s)
Default Ignored Values
Key Configuration Options
Primary Detection Context
AST-Awareness Level
Rust
Clippy
approx_constant, unusual_byte_groupings
N/A (lint-specific)
Lint-specific toggles
Highly specific idioms (e.g., math constants, hex identifiers)
High
Python
Pylint, Ruff
magic-value-comparison
None
valid-magic-values (allow-list)
Comparison operations (>, ==, etc.)
Medium
JavaScript/TS
ESLint
no-magic-numbers
None
ignore, ignoreArrayIndexes, enforceConst, ignoreEnums (TS)
All literal usage outside of const declarations
High
Java
Checkstyle
MagicNumberCheck
-1, 0, 1, 2
ignoreNumbers, ignoreHashCodeMethod, constantWaiverParentToken
All literal usage outside of final declarations
Very High
C++
Clang-Tidy
readability-magic-numbers
0, 1, 2, 3, 4, -1
IgnoredIntegerValues, IgnorePowersOf2IntegerValues, IgnoreBitFieldsWidths
All literal usage outside of constant declarations
Very High


V. Remediation Strategies and Best Practices

Identifying magic values is only the first step; the ultimate goal is to remediate them in a way that improves code quality. This requires a combination of disciplined refactoring techniques, the selection of appropriate language constructs, and the integration of these practices into a healthy development workflow.

5.1 The "Extract Constant" Refactoring Technique

The most fundamental remediation strategy is to replace the magic value with a symbolic constant. This simple refactoring immediately addresses the core problems of readability and maintainability.15
The Core Solution: A literal with an unclear meaning is extracted into a variable or constant with a descriptive, human-readable name. For example, return mass * height * 9.81; is refactored to const GRAVITATIONAL_CONSTANT = 9.81; return mass * height * GRAVITATIONAL_CONSTANT;.15
Naming Conventions: The effectiveness of this technique hinges on the quality of the chosen name. The name must convey the semantic meaning of the value. A name like const NUMBER_TEN = 10; is an anti-pattern itself, as it adds no semantic information.17 Constants should follow language-specific conventions, such as
UPPER_SNAKE_CASE in Python, Java, and C++, to signal their immutability to other developers.54
Scope and Locality: A critical decision is where to define the new constant. If the constant is only used within a single function or class, defining it locally improves code locality and makes it easier to understand the code block in isolation. However, if the constant represents a global concept used throughout the application, it should be defined in a central, accessible location. Care must be taken to avoid creating a generic "Constants" file that becomes a disorganized "junk drawer" of unrelated values; constants should be grouped logically by their domain.1
IDE Support: Modern Integrated Development Environments (IDEs) like IntelliJ IDEA, Visual Studio Code, and PhpStorm provide powerful, built-in "Extract Constant" refactoring tools. These tools automate the process of creating the constant, naming it, and finding and replacing all occurrences of the literal within a specified scope, significantly reducing the manual effort and risk of error.72

5.2 A Superior Alternative: Leveraging Enums for Type Safety and Grouping

While extracting a single magic number to a constant is a good first step, a more powerful and robust solution is required when dealing with a set of related magic values that represent states, types, or categories. In these scenarios, an enum (enumeration) is the superior language construct.71
Benefits over Constants:
Type Safety: An enum introduces a new, distinct type into the program. A function defined to accept an OrderStatus enum cannot be accidentally passed an arbitrary integer; the compiler will enforce this constraint, eliminating an entire class of potential bugs.76
Grouping and Namespacing: Enums logically group a set of related constants under a single, named type. This improves code organization and prevents the pollution of the global namespace with numerous individual constants.79
Self-Documentation and Readability: The code becomes significantly more expressive and readable. A conditional statement like if (order.status == OrderStatus.SHIPPED) is far clearer and less error-prone than its magic number equivalent, if (order.status == 3).77
Language Support: Enums are a first-class feature in statically-typed languages like Rust, Java, and C++. Python provides a robust enum module in its standard library. TypeScript also includes enums, which are a key tool for bringing type safety to JavaScript codebases.
The process of fixing a magic number often serves as a gateway to deeper code quality improvements. A developer, prompted by a linter to fix if (status == 2), might start by extracting a constant: const SHIPPED_STATUS = 2;. In doing so, they may notice other related constants like PENDING_STATUS = 1 and DELIVERED_STATUS = 3. This realization—that these are not disparate values but related states of a single concept—can trigger the more profound refactoring of creating an OrderStatus enum. The initial, tactical fix for the magic number thus acts as a catalyst for a more strategic improvement to the application's domain model.

5.3 The Future of Remediation: Automated Refactoring and AI

The remediation of magic values is becoming increasingly automated. Advanced tooling is moving beyond simple detection to actively suggesting and performing refactorings.
Automated Refactoring Tools: Tools like Sourcery for Python can analyze code and automatically suggest refactorings, including the extraction of magic numbers into constants, directly within the IDE.82
AI-Powered Code Assistants: The next generation of tools, powered by Large Language Models (LLMs), explicitly list "Separate hardcoded literals" as a core feature.83 These tools can not only identify a magic number but also infer its semantic meaning from the surrounding code and suggest a descriptive name for the new constant.84
This trend signals a fundamental shift in the developer's role from a manual "fixer" to a "reviewer." As AI becomes more adept at performing the mechanical aspects of refactoring, the developer's primary responsibility will be to validate the tool's suggestions. They must provide the critical domain context that the AI lacks, ensuring that the generated constant name is not just plausible but semantically correct. This elevates the importance of code review and critical thinking over rote, repetitive changes.

5.4 Effective Reporting: Integrating Findings into the Development Workflow

For magic value detection to be effective, its findings must be integrated seamlessly and constructively into the team's development lifecycle.
CI/CD Integration: Linters must be a standard part of the Continuous Integration (CI) pipeline. This ensures that code with magic value violations is flagged automatically before it can be merged into the main branch, enforcing quality standards proactively.85
Code Review Best Practices:
Constructive Feedback: Linter output should be treated as objective, non-personal feedback designed to improve code quality, fostering a positive review culture.87
Checklists: Teams should use code review checklists that explicitly include an item for "Check for magic values." This ensures consistency and prevents such issues from being overlooked.88
Author Annotations: Authors should annotate their code changes before a review, explaining the purpose of any necessary literals to preemptively address questions about potential magic numbers.88
Clear and Actionable Messages: A high-quality linter report provides more than just an error code. It should offer a clear explanation of why the literal is considered a magic value and suggest a concrete path for remediation. For example, a message like, "The number '3.1415' is a magic number. Consider replacing it with a named constant or using the standard library constant 'Math.PI'," is far more helpful than a generic warning.89

VI. Conclusion: A Unified Framework for Managing Literals in Modern Software Development

The analysis of magic values, from their fundamental definition to the sophisticated algorithms used for their detection, reveals a set of universal principles that transcend specific programming languages. The avoidance of unexplained literals is not a dogmatic rule but a cornerstone of writing readable, maintainable, and reliable software. This report has demonstrated that the "magic" of a value lies not in the number or string itself, but in the absence of self-documenting context.
The most effective detection strategies are rooted in a deep, structural understanding of the source code, made possible by Abstract Syntax Tree (AST) analysis. By examining the syntactic "fingerprint"—the path of nodes from a literal to its enclosing declaration—static analysis tools can apply nuanced, context-aware heuristics that intelligently filter out legitimate uses of literals, thereby minimizing false positives and increasing developer trust. The evolution of these tools, particularly their configuration options, reflects a codified history of community-accepted programming idioms.
A comparative analysis of leading linters across Rust, Python, JavaScript, Java, and C++ highlights a spectrum of philosophical approaches. At one end, Rust's Clippy favors a set of highly specific, idiomatic lints, while at the other, JavaScript's ESLint provides a single, universally applicable rule with extensive configuration to accommodate the language's dynamic nature. Tools for statically-typed languages like Java and C++ leverage type information to achieve a high degree of precision and contextual awareness.
Remediation of magic values, primarily through the "Extract Constant" refactoring, is a critical step in improving code quality. However, a more profound improvement is often achieved by recognizing when a set of related magic values signals a missing domain abstraction, for which an enum is the superior, type-safe solution. The simple act of addressing a magic number can thus serve as a catalyst for deeper, more meaningful improvements to the software's architecture.
Based on this comprehensive analysis, a unified framework for managing literals in modern software development emerges:
Prioritize Context and Intent: Establish a team-wide understanding that the goal is not to eliminate all numbers from the code, but to ensure that every literal's purpose is immediately obvious, either through its value (e.g., 0, 1), its idiomatic usage (e.g., x % 2), or its association with a descriptively named constant or enum.
Leverage AST-Based Tooling: Integrate a powerful, configurable, AST-based linter into the CI/CD pipeline for every project. Relying on simple text-based searches is insufficient and counterproductive.
Configure Intelligently: Invest time in configuring the chosen linter. Start with the community-accepted defaults and incrementally tailor the rules to the project's specific domain and coding standards. Use advanced features, like Checkstyle's constantWaiverParentToken or Clang-Tidy's IgnorePowersOf2IntegerValues, to codify project-specific idioms and reduce noise.
Refactor with Discipline: When a magic value is flagged, do not just apply the simplest fix. First, consider if the value is part of a larger set of related concepts that would be better represented by an enum. Use IDE refactoring tools to ensure all instances are updated correctly and consistently.
Embrace Automation, Guided by Human Review: As AI-powered refactoring tools become more prevalent, shift the team's focus from performing mechanical fixes to critically reviewing automated suggestions. The developer's role is to provide the essential domain knowledge and semantic oversight that ensures the generated code is not just syntactically correct, but also meaningful and maintainable.
By adopting this framework, engineering organizations can move beyond a simplistic view of magic values and implement a mature, systematic approach to managing literals that enhances code quality, reduces technical debt, and ultimately improves long-term developer productivity.
Works cited
Magic number (programming) - Wikipedia, accessed August 19, 2025, https://en.wikipedia.org/wiki/Magic_number_(programming)
Definition of constants and magic numbers, accessed August 19, 2025, https://www.inf.unibz.it/~calvanese/teaching/04-05-ip/lecture-notes/uni04/node15.html
Magic Numbers and Magic Strings: It's time to talk about it - DEV Community, accessed August 19, 2025, https://dev.to/ruben_alapont/magic-numbers-and-magic-strings-its-time-to-talk-about-it-ci2
What Are Magic Numbers And Why Are They Bad - Web Dev Simplified Blog, accessed August 19, 2025, https://blog.webdevsimplified.com/2020-02/magic-numbers/
Rust will yell at you if you use specific magic numbers like these edit: yes, 0x... | Hacker News, accessed August 19, 2025, https://news.ycombinator.com/item?id=41231447
Magic Strings - DevIQ, accessed August 19, 2025, https://deviq.com/antipatterns/magic-strings/
DON'T use magic-strings, use Enum/Constants instead! - Developer Forum | Roblox, accessed August 19, 2025, https://devforum.roblox.com/t/dont-use-magic-strings-use-enumconstants-instead/2916563
What is wrong with magic strings? - Software Engineering Stack Exchange, accessed August 19, 2025, https://softwareengineering.stackexchange.com/questions/365339/what-is-wrong-with-magic-strings
Magic string - Wikipedia, accessed August 19, 2025, https://en.wikipedia.org/wiki/Magic_string
Avoid Magic Numbers. Use Explanatory Constants Instead | by Thijmen Dam | ITNEXT, accessed August 19, 2025, https://itnext.io/magic-numbers-are-problematic-5b12cfe5f31a
Magic Strings & Magic Numbers - DEV Community, accessed August 19, 2025, https://dev.to/thibaultchatelain/magic-strings-magic-numbers-4fa4
Pluralsight Tech Blog | Avoiding Magic Numbers, accessed August 19, 2025, https://www.pluralsight.com/tech-blog/avoiding-magic-numbers/
Why are magic numbers bad practice? - Software Engineering Stack Exchange, accessed August 19, 2025, https://softwareengineering.stackexchange.com/questions/411206/why-are-magic-numbers-bad-practice
Code Smells: The Impact of Magic Numbers | by CodeChuckle - Medium, accessed August 19, 2025, https://medium.com/@codechuckle/code-smells-the-impact-of-magic-numbers-37a90b04cfbe
Replace Magic Number with Symbolic Constant - Refactoring.Guru, accessed August 19, 2025, https://refactoring.guru/replace-magic-number-with-symbolic-constant
Are 0.0 and 1.0 considered magic numbers? - Stack Overflow, accessed August 19, 2025, https://stackoverflow.com/questions/19521582/are-0-0-and-1-0-considered-magic-numbers
Usage of magic strings/numbers [closed] - Software Engineering Stack Exchange, accessed August 19, 2025, https://softwareengineering.stackexchange.com/questions/221034/usage-of-magic-strings-numbers
Should I avoid magic strings as possible? - Stack Overflow, accessed August 19, 2025, https://stackoverflow.com/questions/10662299/should-i-avoid-magic-strings-as-possible
MagicNumber – checkstyle, accessed August 19, 2025, https://checkstyle.sourceforge.io/checks/coding/magicnumber.html
checkstyle/src/main/java/com/puppycrawl/tools/checkstyle/checks/coding/MagicNumberCheck.java at master - GitHub, accessed August 19, 2025, https://github.com/checkstyle/checkstyle/blob/master/src/main/java/com/puppycrawl/tools/checkstyle/checks/coding/MagicNumberCheck.java
Metaprogramming Beyond Decency: Part 1 - Hackflow, accessed August 19, 2025, http://suor.github.io/blog/2015/03/29/metaprogramming-beyond-decency/
Basic understanding of Abstract Syntax Tree (AST) | by Jessica López Espejel | Medium, accessed August 19, 2025, https://medium.com/@jessica_lopez/basic-understanding-of-abstract-syntax-tree-ast-d40ff911c3bf
AST Structure - peter-can-write/clang-notes - GitHub, accessed August 19, 2025, https://github.com/peter-can-write/clang-notes/blob/master/ast-structure.md
regex - check for magic numbers - Stack Overflow, accessed August 19, 2025, https://stackoverflow.com/questions/19542931/check-for-magic-numbers
Eclipse JDT - Abstract Syntax Tree (AST) and the Java Model - Vogella, accessed August 19, 2025, https://www.vogella.com/tutorials/EclipseJDT/article.html
MagicNumberCheck (checkstyle 8.8 API), accessed August 19, 2025, https://daniilyar.github.io/checkstyle.github.io/apidocs/com/puppycrawl/tools/checkstyle/checks/coding/MagicNumberCheck.html
no-magic-numbers - ESLint - Pluggable JavaScript Linter, accessed August 19, 2025, https://eslint.org/docs/latest/rules/no-magic-numbers
no-magic-numbers - ESLint - Pluggable JavaScript linter, accessed August 19, 2025, https://archive.eslint.org/docs/rules/no-magic-numbers
no-magic-numbers - typescript-eslint, accessed August 19, 2025, https://typescript-eslint.io/rules/no-magic-numbers/
Implement `noMagicNumbers` - `eslint/no-magic-numbers`, `typescript-eslint/no-magic-numbers` · Issue #4333 · biomejs/biome - GitHub, accessed August 19, 2025, https://github.com/biomejs/biome/issues/4333
MagicNumberCheck (checkstyle 11.0.0 API), accessed August 19, 2025, https://checkstyle.sourceforge.io/apidocs/com/puppycrawl/tools/checkstyle/checks/coding/MagicNumberCheck.html
How to avoid magic number warning when initialize static field (for example BigDecimal)?, accessed August 19, 2025, https://stackoverflow.com/questions/54015246/how-to-avoid-magic-number-warning-when-initialize-static-field-for-example-bigd
Pending Removal in Python 3.14 — Python 3.13.6 documentation, accessed August 19, 2025, https://docs.python.org/3/deprecations/pending-removal-in-3.14.html
Deprecations — Python 3.13.7 documentation, accessed August 19, 2025, https://docs.python.org/3/deprecations/index.html
ast — Abstract Syntax Trees — Python 3.13.7 documentation, accessed August 19, 2025, https://docs.python.org/3/library/ast.html
Appendix A. Syntax Tree Format — Esprima master documentation - Read the Docs, accessed August 19, 2025, https://esprima.readthedocs.io/en/latest/syntax-tree-format.html
Acornima is a standard-compliant JavaScript parser for .NET. It is a fork of Esprima.NET combined with the .NET port of the acornjs parser. - GitHub, accessed August 19, 2025, https://github.com/adams85/acornima
AST explorer, accessed August 19, 2025, https://astexplorer.net/
Package org.eclipse.jdt.core.dom - IBM, accessed August 19, 2025, https://www.ibm.com/docs/en/developer-for-zos/17.0?topic=reference-orgeclipsejdtcoredom
ASTNumericLiteral (PMD Java 7.0.0 API), accessed August 19, 2025, https://docs.pmd-code.org/apidocs/pmd-java/7.0.0/net/sourceforge/pmd/lang/java/ast/ASTNumericLiteral.html
Package org.eclipse.jdt.core.dom - IBM, accessed August 19, 2025, https://www.ibm.com/docs/sk/SS8PJ7_9.6.1/org.eclipse.jdt.doc.isv/reference/api/org/eclipse/jdt/core/dom/package-summary.html
AST Matcher Reference - Clang, accessed August 19, 2025, https://clang.llvm.org/docs/LibASTMatchersReference.html
Matching the Clang AST — Clang 22.0.0git documentation, accessed August 19, 2025, https://clang.llvm.org/docs/LibASTMatchers.html
Writing AST matchers for libclang - @Manu343726 - C++, accessed August 19, 2025, https://manu343726.github.io/2017-02-11-writing-ast-matchers-for-libclang/
readability-magic-numbers — Extra Clang Tools 22.0.0git documentation, accessed August 19, 2025, https://clang.llvm.org/extra/clang-tidy/checks/readability/magic-numbers.html
clang::tidy::readability::MagicNumbersCheck Class Reference, accessed August 19, 2025, https://clang.llvm.org/extra/doxygen/classclang_1_1tidy_1_1readability_1_1MagicNumbersCheck.html
D49114 [clang-tidy] Add a check for "magic numbers" - LLVM Phabricator archive, accessed August 19, 2025, https://reviews.llvm.org/D49114
Six Heuristics To Make You A Better Magic Player - Star City Games, accessed August 19, 2025, https://articles.starcitygames.com/magic-the-gathering/premium/six-heuristics-to-make-you-a-better-magic-player/
How to come up with heuristic values - Stack Overflow, accessed August 19, 2025, https://stackoverflow.com/questions/28191575/how-to-come-up-with-heuristic-values
Heuristic (psychology) - Wikipedia, accessed August 19, 2025, https://en.wikipedia.org/wiki/Heuristic_(psychology)
readability-magic-numbers - clang-tidy - ROCm Documentation, accessed August 19, 2025, https://rocm.docs.amd.com/projects/llvm-project/en/latest/LLVM/clang-tools/html/clang-tidy/checks/readability/magic-numbers.html
Magic Numbers Are Problematic (Use Explanatory Constants Instead) : r/ProgrammerTIL, accessed August 19, 2025, https://www.reddit.com/r/ProgrammerTIL/comments/1018pm6/magic_numbers_are_problematic_use_explanatory/
Clippy Lints, accessed August 19, 2025, https://rust-lang.github.io/rust-clippy/rust-1.56.0/index.html
Refactoring Magic Numbers - JavaScript Code Readability, accessed August 19, 2025, https://www.codereadability.com/magic-numbers/
Disable clang-tidy for ALL (specific) function invocations - Stack Overflow, accessed August 19, 2025, https://stackoverflow.com/questions/75060469/disable-clang-tidy-for-all-specific-function-invocations
Are there exceptions to magic numbers? : r/learnprogramming - Reddit, accessed August 19, 2025, https://www.reddit.com/r/learnprogramming/comments/1mkgep7/are_there_exceptions_to_magic_numbers/
When is a number a magic number? - Software Engineering Stack Exchange, accessed August 19, 2025, https://softwareengineering.stackexchange.com/questions/251540/when-is-a-number-a-magic-number
Is it good practice to get rid of magic numbers no matter the language? - Reddit, accessed August 19, 2025, https://www.reddit.com/r/learnprogramming/comments/1blkk14/is_it_good_practice_to_get_rid_of_magic_numbers/
MAGIC: Detecting Advanced Persistent Threats via Masked Graph Representation Learning - USENIX, accessed August 19, 2025, https://www.usenix.org/system/files/sec23winter-prepub-490-jia.pdf
The Impact of Key Ideas on Automatic Deception Detection in Text - SciELO México, accessed August 19, 2025, https://www.scielo.org.mx/scielo.php?script=sci_arttext&pid=S1405-55462020000301229
Unraveling the Language of Fraud: Linguistics in Anomaly Detection, accessed August 19, 2025, https://nrao-prashanthi.medium.com/unraveling-the-language-of-fraud-linguistics-in-anomaly-detection-9a20d77a64de
Clippy Lints - GitHub Pages, accessed August 19, 2025, https://rust-lang.github.io/rust-clippy/master/index.html
Lint Configuration - Clippy Documentation, accessed August 19, 2025, https://doc.rust-lang.org/clippy/lint_configuration.html
magic-value-comparison (PLR2004) | Ruff - Astral Docs, accessed August 19, 2025, https://docs.astral.sh/ruff/rules/magic-value-comparison/
magic-value-comparison / R2004 - Pylint 3.3.8 documentation, accessed August 19, 2025, https://pylint.readthedocs.io/en/stable/user_guide/messages/refactor/magic-value-comparison.html
File magic_value.py - CodeSync, accessed August 19, 2025, https://www.codesync.com/files/1151908
Pylint 3.3.8 documentation, accessed August 19, 2025, https://pylint.readthedocs.io/
eslint/no-magic-numbers | The JavaScript Oxidation Compiler - Oxc, accessed August 19, 2025, https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-magic-numbers
Replace Magic Literal - Refactoring, accessed August 19, 2025, https://refactoring.com/catalog/replaceMagicNumberWithSymbolicConstant.html
Python Refactoring: Turn Magic Numbers into Constants for Readability - YouTube, accessed August 19, 2025, https://www.youtube.com/watch?v=IljQqLnPC14
2 Rookie Java Constants and Enums Pitfalls - Ted Vinke's Blog - WordPress.com, accessed August 19, 2025, https://tedvinke.wordpress.com/2016/04/14/2-rookie-java-constants-and-enums-pitfalls/
XRefactoring Plugin for JetBrains IDEs, accessed August 19, 2025, https://plugins.jetbrains.com/plugin/23125-xrefactoring
Extract constant | PhpStorm Documentation - JetBrains, accessed August 19, 2025, https://www.jetbrains.com/help/phpstorm/extract-constant.html
Extract constant | IntelliJ IDEA Documentation - JetBrains, accessed August 19, 2025, https://www.jetbrains.com/help/idea/extract-constant.html
Refactoring - Visual Studio Code, accessed August 19, 2025, https://code.visualstudio.com/docs/editing/refactoring
Java Enums with Examples, Enums Vs Constants | by Priya Salvi - Medium, accessed August 19, 2025, https://medium.com/@salvipriya97/java-enums-with-examples-enums-vs-constants-b12685b4c4ec
Enums vs Constants in Android: Best Practices with Simple Examples | by Arun - Medium, accessed August 19, 2025, https://arundevofficial.medium.com/enums-vs-constants-in-android-best-practices-with-simple-examples-a065e0dc4297
Lesson 32: Including Magic Numbers and Enums - Job Ready Java [Book] - O'Reilly Media, accessed August 19, 2025, https://www.oreilly.com/library/view/job-ready-java/9781119775645/c32.xhtml
Should I use #define, enum or const? - c++ - Stack Overflow, accessed August 19, 2025, https://stackoverflow.com/questions/112433/should-i-use-define-enum-or-const
Benefits of using enums over directly using integral types? - Stack Overflow, accessed August 19, 2025, https://stackoverflow.com/questions/3711916/benefits-of-using-enums-over-directly-using-integral-types
Giving Meaning to Magic Numbers with Python Enums, accessed August 19, 2025, https://www.assertnotmagic.com/2019/03/13/enums-magic-numbers-python/
#266 - Refactoring Your Code like Magic with Sourcery - YouTube, accessed August 19, 2025, https://www.youtube.com/watch?v=NazqbX90Fb4
Learn, improve and generate code with AI | Refraction, accessed August 19, 2025, http://refraction.dev/
Code Smell 02 - Constants and Magic Numbers : r/refactoring - Reddit, accessed August 19, 2025, https://www.reddit.com/r/refactoring/comments/1moo34w/code_smell_02_constants_and_magic_numbers/
4 tips to improve code quality - Work Life by Atlassian, accessed August 19, 2025, https://www.atlassian.com/blog/add-ons/4-tips-to-improve-code-quality
Code quality and best practices : r/SalesforceDeveloper - Reddit, accessed August 19, 2025, https://www.reddit.com/r/SalesforceDeveloper/comments/1g9k6r5/code_quality_and_best_practices/
Code reviews: Best practices for maintaining code quality - Statsig, accessed August 19, 2025, https://www.statsig.com/perspectives/code-reviews-best-practices-for-maintaining-code-quality
Best Practices for Peer Code Review - SmartBear, accessed August 19, 2025, https://smartbear.com/learn/code-review/best-practices-for-peer-code-review/
Rules | SonarQube Server Documentation, accessed August 19, 2025, https://docs.sonarsource.com/sonarqube/latest/user-guide/rules/
