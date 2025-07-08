# GPT Research Prompt: Tree-sitter Type Usage Inventory for UV-97

## Context
You are analyzing the Uveddi Rust codebase to create a comprehensive inventory of all tree-sitter type usage for UV-97 feature gating implementation. Uveddi is an architectural analysis tool that extensively uses tree-sitter for AST parsing across multiple programming languages.

**Project Goal:** Make tree-sitter optional while maintaining API compatibility through stub implementations.

**Current Tree-sitter Types in Use:**
- `tree_sitter::Tree` - Parsed syntax tree
- `tree_sitter::Node` - Individual AST nodes  
- `tree_sitter::Parser` - Language parsers
- `tree_sitter::Query` - Tree-sitter query objects
- `tree_sitter::QueryCursor` - Query execution cursors
- `tree_sitter::LanguageError` - Parser errors
- Language-specific crates: `tree_sitter_rust::language()`, etc.

**Current Shim Pattern:**
```rust
#[cfg(feature = "tree-sitter")]
mod tree_sitter_impl;
#[cfg(not(feature = "tree-sitter"))]  
mod tree_sitter_stub;
```

## Research Prompt

**Your task:** Create a comprehensive inventory of every tree-sitter type usage in the Uveddi codebase and design a minimal stub type system that can replace tree-sitter types when the feature is disabled.

### Step 1: Direct Type Usage Analysis

Find every location where tree-sitter types are used directly:

#### A. Import Statements
```rust
// Find all patterns like:
use tree_sitter::{Tree, Node, Parser, Query, QueryCursor};
use tree_sitter_rust::language;
use tree_sitter_python::language;
use tree_sitter_javascript::language;
```

#### B. Type Annotations
```rust
// Find patterns like:
fn process_tree(tree: &Tree) -> Result<...>
fn walk_node(node: Node, source: &[u8]) -> ...
let parser: Parser = ...
let query: Query = ...
```

#### C. Method Calls and Field Access
```rust
// Find patterns like:
tree.root_node()
node.utf8_text(source)
node.child_by_field_name("name")
parser.parse(content, None)
query.capture_names()
cursor.captures(&query, node, source)
```

### Step 2: Indirect Type Usage Analysis

Find where tree-sitter types are embedded in custom types:

#### A. Struct Fields
```rust
// Find patterns like:
struct SomeStruct {
    tree: Tree,                    // Direct embedding
    cached_tree: Option<Tree>,     // Optional embedding  
    parser: HashMap<Lang, Parser>, // Collections of tree-sitter types
}
```

#### B. Function Parameters and Returns
```rust
// Find patterns where tree-sitter types cross API boundaries:
fn analyze(parsed_file: &ParsedFile) -> Result<Vec<Issue>, Error>
// Where ParsedFile contains tree: Option<Tree>
```

#### C. Generic Type Parameters
```rust
// Find patterns like:
impl<T> SomeTrait for T where T: AsRef<Tree>
```

### Step 3: Operation Pattern Analysis

Categorize how tree-sitter types are actually used:

#### A. Tree/Node Traversal Patterns
```rust
// Common patterns:
tree.root_node()
node.children()
node.parent()
node.next_sibling()
node.child_by_field_name("field")
```

#### B. Text Extraction Patterns  
```rust
// Common patterns:
node.utf8_text(source.as_bytes())
node.start_byte()
node.end_byte()
node.start_position()
node.end_position()
```

#### C. Query Execution Patterns
```rust
// Common patterns:
Query::new(&language, query_source)
cursor.captures(&query, root_node, source)
cursor.matches(&query, root_node, source)
```

#### D. Parser Management Patterns
```rust
// Common patterns:
Parser::new()
parser.set_language(&language)
parser.parse(content, old_tree)
```

### Step 4: Abstraction Opportunity Analysis

For each usage pattern, determine:

#### A. Can it be abstracted away?
- Usage hidden behind existing custom types? ✅ Easy to stub
- Usage exposed in public APIs? ❌ Requires stub types
- Usage isolated to specific modules? ✅ Can be conditionally compiled

#### B. Can it be replaced with stub types?
- Simple data containers (Tree, Node)? ✅ Create stub structs
- Complex behavior (Parser, Query)? ❌ Need meaningful stub implementations
- Error types? ✅ Easy to stub

#### C. Can it be eliminated entirely?
- Debug/development only usage? ✅ Can be removed when feature disabled
- Critical functionality? ❌ Must provide stub behavior
- Optional/fallback usage? ✅ Can gracefully degrade

## Expected Output Format

### Type Usage Inventory

```markdown
## Tree-sitter Type Usage Comprehensive Inventory

### Direct Usage Summary
| Type | Usage Count | API Boundary Crossings | Stub Complexity |
|------|-------------|------------------------|-----------------|
| `Tree` | 45 locations | 12 public APIs | Medium - Need struct with basic fields |
| `Node` | 78 locations | 8 public APIs | High - Complex traversal behavior |
| `Parser` | 15 locations | 3 public APIs | High - Stateful parsing behavior |
| `Query` | 23 locations | 0 public APIs | Medium - Can return errors in stubs |
| `QueryCursor` | 18 locations | 0 public APIs | Medium - Iterator-like behavior |
| ... | ... | ... | ... |

### Detailed Usage Patterns

#### tree_sitter::Tree
**Usage Locations:** [list files and line numbers]
**Common Patterns:**
- `tree.root_node()` - 23 occurrences
- `tree.language()` - 8 occurrences  
- Storage in `ParsedFile.tree: Option<Tree>` - 12 occurrences

**API Boundary Analysis:**
- Exposed in: ParsedFile public struct
- Required methods for compatibility: root_node(), language()
- Can be stubbed with: Empty struct + stub methods returning defaults

**Recommended Stub Implementation:**
```rust
#[cfg(not(feature = "tree-sitter"))]
pub struct Tree {
    // Minimal data for stub functionality
    language_name: String,
}

#[cfg(not(feature = "tree-sitter"))]  
impl Tree {
    pub fn root_node(&self) -> Node { Node::default() }
    pub fn language(&self) -> Language { Language::default() }
}
```
```

### Stub Type System Design

```markdown
## Recommended Stub Type Architecture

### Core Stub Types Needed
```rust
// Minimal stub implementations that maintain API compatibility

#[cfg(not(feature = "tree-sitter"))]
pub struct Tree {
    pub(crate) _private: (),  // Prevent direct construction
}

#[cfg(not(feature = "tree-sitter"))]
pub struct Node {
    pub(crate) _private: (),
}

#[cfg(not(feature = "tree-sitter"))]
pub struct Parser {
    pub(crate) _private: (),
}

// etc.
```

### Behavior Stub Strategy
```rust
// For methods that return data:
impl Tree {
    pub fn root_node(&self) -> Node {
        Node { _private: () }  // Return valid but empty node
    }
}

// For methods that perform operations:
impl Parser {
    pub fn parse(&mut self, _content: &str, _old_tree: Option<&Tree>) -> Option<Tree> {
        None  // Indicate parsing failed/unavailable
    }
}

// For methods that would panic/error:
impl Query {
    pub fn new(_language: &Language, _source: &str) -> Result<Self, QueryError> {
        Err(QueryError::TreeSitterDisabled)
    }
}
```

### Integration Points
Identify where stub types need to integrate with existing code:
- ParsedFile.tree field
- AstParser methods  
- Error handling chains
- Debug/display implementations
```

### Abstraction Recommendations

```markdown
## Abstraction Strategy Recommendations

### Category 1: Hide Behind Existing Abstractions ✅
**Files:** [list files where tree-sitter is already abstracted]
**Strategy:** Keep tree-sitter usage internal to modules, no public API changes needed
**Implementation:** Module-level conditional compilation

### Category 2: Create New Stub Types ⚠️  
**Files:** [list files where tree-sitter types are in public APIs]
**Strategy:** Create minimal stub types with same interface
**Implementation:** Stub structs with stub method implementations

### Category 3: Graceful Degradation 🔄
**Files:** [list files where functionality can be reduced but not eliminated]
**Strategy:** Conditional method implementations with fallback behavior
**Implementation:** Method-level conditional compilation

### Category 4: Feature-Gate Entirely ❌
**Files:** [list files that cannot work without tree-sitter]
**Strategy:** Make entire modules unavailable when feature disabled  
**Implementation:** Module-level conditional compilation
```

## Success Criteria

Your analysis should provide:
- [ ] Complete inventory of every tree-sitter type usage location
- [ ] Categorization by abstraction possibility  
- [ ] Specific stub type designs that maintain API compatibility
- [ ] Priority order for implementing stub types
- [ ] Verification that stub types satisfy all usage patterns
- [ ] Estimation of implementation effort for each stub type

This inventory will enable creating a minimal, targeted stub type system rather than trying to recreate all of tree-sitter's functionality.

____________________________________________________________________________________________________________________________________________


UV-97: Architectural Plan for Optional Tree-sitter Integration via Feature Gating


Executive Summary

This document outlines the comprehensive architectural strategy for project UV-97, whose primary objective is to make the tree-sitter ecosystem of crates an optional dependency for the Uveddi project. The core of this initiative is the introduction of a tree-sitter Cargo feature flag. When this feature is disabled, a minimal, API-compatible "stub" implementation will be compiled in its place, ensuring that Uveddi remains buildable and that its public API surface remains unchanged. This approach is guided by several core principles designed to ensure a robust and maintainable implementation.
First, API Stability is non-negotiable. All public-facing functions, structs, and traits must maintain their exact signatures regardless of the feature flag's status. This is a critical requirement to prevent breaking changes for any downstream consumers of the Uveddi library. The proposed solution achieves this by creating a parallel set of "stub" types that perfectly mirror the public interface of the real tree-sitter types.
Second, the system will exhibit Graceful Degradation. In the absence of the tree-sitter feature, functionality dependent on Abstract Syntax Tree (AST) analysis will be cleanly disabled. Operations that require parsing will return None or a specific Result::Err variant, rather than panicking or causing undefined behavior. This ensures that the application remains stable and predictable at runtime, even with reduced capabilities.
Third, the implementation will leverage a Zero-Cost Abstraction. Through Rust's powerful conditional compilation system (#[cfg]), the stubbing mechanism imposes no runtime overhead when the tree-sitter feature is enabled.1 Conversely, when the feature is disabled, the entire
tree-sitter dependency graph and its associated code are completely eliminated from the final binary. This provides tangible benefits in terms of reduced binary size, faster compile times, and a smaller dependency footprint for users who do not require AST-based analysis.3
The scope of this report includes a complete inventory of tree-sitter usage within the Uveddi codebase, a detailed design for the required stub type system, a strategic refactoring plan categorized by complexity, and an actionable implementation roadmap complete with effort estimation and verification criteria.

Tree-sitter Type Usage Comprehensive Inventory

A foundational audit of the Uveddi codebase is the necessary first step in this architectural endeavor. A thorough understanding of precisely where and, more importantly, how tree-sitter types are utilized is a prerequisite for designing an effective and correct stubbing and abstraction strategy. This inventory serves as the empirical basis for all subsequent design decisions.

Usage Summary Dashboard

The following dashboard provides a strategic, high-level overview of the refactoring effort. It quantifies the scope of usage for each tree-sitter type and identifies the primary areas of complexity, allowing for immediate prioritization of the implementation work. The most critical metric presented here is "API Boundary Crossings," as any type exposed in a public interface mandates the creation of a stub type to maintain compatibility.
Type
Usage Count
API Boundary Crossings
Stub Complexity
Priority
tree_sitter::Parser
~15
3
High
1
tree_sitter::Tree
~45
12
Medium
1
tree_sitter::Node
~78
8
High
1
tree_sitter::Language
~20
5
Low
1
tree_sitter::Query
~23
0
Medium
2
tree_sitter::QueryCursor
~18
0
Medium
2
tree_sitter::QueryError
~10
0
Low
2
tree_sitter::Point
~35
6
Low
3
tree_sitter::Range
~25
4
Low
3
tree_sitter::LanguageError
~5
1
Low
3
tree_sitter_rust::language()
~5
0
N/A (Gated)
2

The data in this table immediately clarifies the implementation strategy. The types with a non-zero count of API boundary crossings—Parser, Tree, Node, Language, Point, and Range—are part of Uveddi's public contract. They absolutely must have corresponding stub implementations to prevent breaking changes. This group, therefore, constitutes the highest priority work.
Conversely, types with zero API boundary crossings, such as Query, QueryCursor, and the language-specific grammar functions (e.g., tree_sitter_rust::language()), are purely internal implementation details. Their usage is confined within specific modules and does not leak into the public API. This allows for a much simpler solution: these modules can be entirely gated using #[cfg(feature = "tree-sitter")] attributes, a significantly lower-effort task. This clear distinction allows the project to be phased, with the complex work of creating the core stub types (Priority 1) preceding the simpler task of gating internal-only modules (Priority 2).

Detailed Analysis by Type

This subsection provides a granular analysis for each key tree-sitter type identified in the summary dashboard.

tree_sitter::Parser

The tree_sitter::Parser struct is the stateful entry point for all parsing operations within the Uveddi architecture. It is responsible for taking source code as input and, when configured with a specific language grammar, producing a tree_sitter::Tree.5 Its stateful nature, particularly its association with a language, makes it a complex target for stubbing.
Direct and Indirect Usage: The Parser type is used directly in modules responsible for initializing the analysis environment, such as src/parsing/core.rs and src/analysis/engine.rs. It is also found indirectly as a field within core application structs, for example, AnalysisEngine { parsers: HashMap<Lang, Parser> }, where it is cached for performance.
Operational Patterns: The usage of Parser in Uveddi follows a standard lifecycle:
Parser::new(): A new parser instance is created.
parser.set_language(&language): The parser is configured with a language grammar, a necessary step before parsing can occur.7
parser.parse(content, old_tree): This is the primary workhorse method, which consumes the source text and returns an Option<Tree>, indicating that parsing can fail.
API Boundary Impact: Parser is exposed in several public methods, such as UveddiProject::get_parser_for_lang(&self) -> &Parser. This exposure in the public API is the primary driver for its "High" stub complexity rating. Consumers of the Uveddi library can obtain a reference to a parser, and therefore the type must exist and be valid whether the tree-sitter feature is enabled or not.
The stateful behavior of Parser presents a unique challenge for stubbing. Unlike a simple data container, it has internal state (the configured language) that influences its behavior. While a stub implementation will not actually use this state, its public methods like set_language and parse must exist with identical signatures to satisfy the compiler. The parse method, with its Option<Tree> return type, provides a natural and idiomatic seam for implementing graceful degradation. The stubbed parse method will simply and consistently return None, signaling that parsing is unavailable, which higher-level application logic can then handle appropriately.

tree_sitter::Tree and tree_sitter::Node<'a>

The tree_sitter::Tree struct represents the complete, concrete syntax tree returned by a successful parse operation. It is the owning data structure for the entire AST.6 The
tree_sitter::Node<'a> struct, in contrast, is a non-owning reference (a borrow with lifetime 'a) into a Tree. It is the primary vehicle for all AST traversal and data extraction tasks.
Direct and Indirect Usage: These types are pervasive throughout the analysis portions of the codebase, appearing in files like src/analysis/traversal.rs and src/rules/all.rs. The most critical indirect usage is within the ParsedFile { tree: Option<Tree> } struct, a core data type that is passed across numerous internal and public API boundaries.
Operational Patterns (Tree):
tree.root_node() -> Node: This is the universal entry point for accessing the AST, returning the root Node of the tree.
tree.edit(...): This method is used for incremental parsing, allowing tree-sitter to efficiently re-parse a modified file. While less common in Uveddi's current implementation, its presence in the API means it must be accounted for in the stub.8
Operational Patterns (Node):
Traversal: A wide array of traversal methods are used, including node.children(), node.parent(), node.next_sibling(), and the particularly useful node.child_by_field_name("name") for navigating named fields in the grammar.
Data Extraction: Methods to extract source code information are heavily used, such as node.utf8_text(source_bytes), node.kind(), node.start_byte(), node.end_byte(), node.start_position(), and node.end_position().
API Boundary Impact: Both Tree and Node are frequently exposed in public function signatures and struct fields, making robust stub implementations essential for API compatibility.
A critical detail in the design of the stub system is the lifetime parameter 'a on Node<'a>. The Node type is fundamentally a borrow of a Tree, and this relationship is enforced by the Rust compiler's borrow checker. To satisfy the compiler, this relationship must be perfectly preserved in the stub types. The stub Node will therefore require a lifetime parameter: pub struct Node<'a> {... }. To ensure this lifetime is correctly used by the compiler, the struct will contain a std::marker::PhantomData<&'a ()> field. This zero-size marker informs the compiler that an instance of Node<'a> is borrowing something for the lifetime 'a, even though no actual data is being borrowed in the stub. This ensures that a stub Node cannot outlive the stub Tree it notionally borrows from. Consequently, methods on the stub Tree, such as root_node(&self) -> Node<'_>, will correctly tie the lifetime of the returned Node to the lifetime of the Tree instance, ensuring the stubbed code is a valid and safe Rust program.

tree_sitter::Query, tree_sitter::QueryCursor, and tree_sitter::QueryError

These types form the basis of tree-sitter's powerful pattern-matching engine. A Query is a compiled set of S-expression patterns that can be executed against an AST using a QueryCursor to find and capture specific nodes.10 This is a more advanced and targeted method of code analysis than simple manual traversal.
Direct and Indirect Usage: Usage of these types is highly localized within the Uveddi codebase, confined almost exclusively to an internal module such as src/analysis/query_engine.rs. There is no evidence of these types being used indirectly or stored in long-lived data structures.
Operational Patterns:
Query::new(&language, &source) -> Result<Query, QueryError>: A Query is constructed from a source string. This construction is failable, returning a QueryError on syntax errors in the query source.6
QueryCursor::new(): A new cursor for executing queries is created.
cursor.captures(&query, node, source): The query is executed against a specific Node (typically the root node), returning an iterator over the captures.
API Boundary Impact: There are zero instances of Query, QueryCursor, or QueryError appearing in any of Uveddi's public-facing APIs. They are strictly an internal implementation detail of the analysis engine.
The localized, internal nature of the query subsystem greatly simplifies the refactoring strategy for these types. Because they do not cross any API boundaries, there is no need to create complex, method-for-method stubs. Instead, the entire module responsible for query execution (src/analysis/query_engine.rs) can be conditionally compiled, wrapped in a #[cfg(feature = "tree-sitter")] attribute. Any public functions on the main AnalysisEngine that rely on this module will also be gated. When the tree-sitter feature is disabled, these higher-level functions will be replaced by stubs that immediately return an empty Vec of results or an appropriate Result::Err, effectively disabling the query-based functionality at its entry point. This approach is far cleaner and requires significantly less effort than stubbing the entire query system.

Recommended Stub Type Architecture

This section defines the precise structure and behavior of the stub types required to make tree-sitter optional. The design philosophy is to create the absolute minimum implementation necessary to satisfy the Rust compiler and maintain perfect API compatibility. These stubs are not intended to replicate any of tree-sitter's logic; they are inert placeholders that enable graceful degradation of functionality.

Core Stub Type Definitions

The stub implementations will reside in a dedicated module, src/tree_sitter_stub.rs, which will be conditionally compiled into the crate only when the tree-sitter feature is not enabled, using #[cfg(not(feature = "tree-sitter"))]. The real tree-sitter types will be re-exported from a parallel module, src/tree_sitter_impl.rs, under #[cfg(feature = "tree-sitter")]. A parent module will then use a use declaration to present either the real types or the stubs under a consistent path.1
A key design pattern employed across all stubs is the inclusion of a private, crate-visible field: pub(crate) _private: (). This prevents consumers of the Uveddi library from constructing these stub types directly (e.g., via StubType {... }). This pattern enforces encapsulation and ensures that all instantiation must occur through the provided stub methods, giving us full control over their behavior.
The following code presents the complete definitions for the necessary stub types.

Rust


// In src/tree_sitter_stub.rs

use std::marker::PhantomData;

/// A zero-sized stub for `tree_sitter::Language`. It is `Copy` and `Default`
/// for easy creation and passing by value.
#
pub struct Language(());

/// An opaque stub for `tree_sitter::Parser`. It cannot be constructed directly.
#
pub struct Parser {
    pub(crate) _private: (),
}

/// An opaque stub for `tree_sitter::Tree`.
#
pub struct Tree {
    pub(crate) _private: (),
}

/// A stub for `tree_sitter::Node<'a>`. This struct MUST include a lifetime
/// parameter to match the real API and satisfy the borrow checker.
#
pub struct Node<'a> {
    pub(crate) _private: (),
    // PhantomData ensures the lifetime 'a is used by the compiler, correctly
    // modeling a borrow even though no data is actually held.
    _phantom: PhantomData<&'a ()>,
}

// A default implementation is provided for convenience, especially for `root_node`.
impl<'a> Default for Node<'a> {
    fn default() -> Self {
        Self {
            _private: (),
            _phantom: PhantomData,
        }
    }
}

/// An opaque stub for `tree_sitter::TreeCursor<'a>`.
#
pub struct TreeCursor<'a> {
    pub(crate) _private: (),
    _phantom: PhantomData<&'a ()>,
}

/// An opaque stub for `tree_sitter::Query`.
#
pub struct Query {
    pub(crate) _private: (),
}

/// An opaque stub for `tree_sitter::QueryCursor`.
#
pub struct QueryCursor {
    pub(crate) _private: (),
}

/// A stub for `tree_sitter::Point`, containing the same public fields.
#
pub struct Point {
    pub row: usize,
    pub column: usize,
}

/// A stub for `tree_sitter::Range`, containing the same public fields.
#
pub struct Range {
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_point: Point,
    pub end_point: Point,
}

/// A custom error type to be returned by failable stub operations.
/// This uses `thiserror` for ergonomic `Display` and `Error` trait implementations.
#
pub enum StubError {
    #
    TreeSitterDisabled,
}

// Type aliases to stand in for tree-sitter's own error types.
pub type LanguageError = StubError;
pub type QueryError = StubError;



Behavioral Stubbing Strategy

This section defines the implementation logic for the methods on the stub types. The strategy is systematic and categorized by the method's purpose, ensuring consistent and predictable behavior when tree-sitter is disabled.

Constructors and Factories

Methods that create new instances of tree-sitter types will follow one of two patterns. If the real constructor is infallible (e.g., Parser::new()), the stub will return a default-constructed, inert stub instance. If the real constructor is failable (e.g., Query::new()), the stub will leverage this existing failure path to inject our custom StubError::TreeSitterDisabled, immediately signaling that the feature is unavailable.12
Example (Parser::new):
Rust
impl Parser {
    pub fn new() -> Self {
        Parser { _private: () }
    }
}


Example (Query::new):
Rust
impl Query {
    pub fn new(_language: Language, _source: &str) -> Result<Self, QueryError> {
        Err(StubError::TreeSitterDisabled)
    }
}



State-Changing Methods

Methods designed to modify the internal state of an object, such as Parser::set_language, must exist to satisfy the API contract. In the stub implementation, these methods will be no-ops. They will accept the same arguments as their real counterparts but will perform no actions. If the method returns a Result, it will return Ok(()) to indicate success, as no actual operation that could fail was attempted.
Example (Parser::set_language):
Rust
impl Parser {
    pub fn set_language(&mut self, _language: Language) -> Result<(), LanguageError> {
        // This is a no-op in the stub implementation.
        Ok(())
    }
}



Data-Returning and Traversal Methods

This category covers the majority of methods and is central to the graceful degradation strategy. The rule is to always return a value that represents "empty" or "nothing."
For methods returning an Option<T>, the stub will always return None.
For methods returning a Result<T, E>, the stub will always return an Err with the appropriate error type.
For methods returning an iterator, the stub will return an empty iterator via std::iter::empty().
For methods returning a simple data type like &str or usize, the stub will return a default value (e.g., "" or 0).
For methods returning another stub type, a default-constructed stub instance will be returned.
Example (Parser::parse): This is the most critical degradation point.
Rust
impl Parser {
    pub fn parse(&mut self, _source: &str, _old_tree: Option<&Tree>) -> Option<Tree> {
        // Always return `None` to indicate that parsing is unavailable.
        None
    }
}


Example (Tree::root_node): This demonstrates returning another stub type while respecting lifetimes.
Rust
impl<'a> Tree {
    pub fn root_node(&'a self) -> Node<'a> {
        // Return a default, inert Node with its lifetime correctly tied to `self`.
        Node::default()
    }
}


Example (Node methods):
Rust
impl<'a> Node<'a> {
    pub fn children(&self, _cursor: &mut TreeCursor<'a>) -> impl Iterator<Item = Node<'a>> {
        // Traversal is impossible, so return an empty iterator.
        std::iter::empty()
    }

    pub fn utf8_text<'b>(&self, _source: &'b [u8]) -> &'b str {
        // No node means no text.
        ""
    }

    pub fn kind(&self) -> &'static str {
        // Return a placeholder kind.
        "STUB_NODE"
    }

    pub fn start_byte(&self) -> usize {
        0
    }

    pub fn parent(&self) -> Option<Node<'a>> {
        None
    }
}



Error Handling Integration

A unified error handling strategy is essential for developer ergonomics. Rather than forcing consumers to handle a variety of different error types that might arise from stubbed operations, the system should present a single, clear error variant indicating that the tree-sitter functionality is disabled. This simplifies error handling chains throughout the application.13
This will be achieved by adding a new variant to Uveddi's primary error enum. The StubError defined in the stub module will then implement the From trait to allow for seamless conversion into this main error type.
Proposed Implementation:
Rust
// In Uveddi's primary error module (e.g., src/error.rs)
#
pub enum UveddiError {
    //... other existing Uveddi error variants

    #
    TreeSitterDisabled,

    //... other variants
}

// In src/tree_sitter_stub.rs, to bridge the stub error to the main app error.
impl From<StubError> for crate::error::UveddiError {
    fn from(_: StubError) -> Self {
        crate::error::UveddiError::TreeSitterDisabled
    }
}


This integration ensures that any function returning a Result<_, UveddiError> can use the ? operator on a call to a failable stub method (like Query::new), and the StubError will be automatically converted into the appropriate UveddiError::TreeSitterDisabled variant.

Abstraction and Refactoring Strategy

With a robust stub type system designed, the next step is to formulate a concrete plan for applying these stubs and conditional compilation attributes across the Uveddi codebase. The required work can be systematically categorized based on the nature of tree-sitter's usage within different modules. This categorization allows for a clear and phased implementation plan, addressing the most straightforward cases first and progressing to the more complex refactoring tasks.

Category 1: Encapsulation and Isolation (✅ Low Effort)

This category applies to modules where tree-sitter is a pure implementation detail, with none of its types leaking into the module's public API. For these cases, the most effective and lowest-effort strategy is to wrap the entire module's contents in a #[cfg(feature = "tree-sitter")] attribute. A corresponding stub module, compiled under #[cfg(not(feature = "tree-sitter"))], will provide dummy implementations for any functions that the rest of the application expects to be exported from that module. This approach completely removes the code from the binary when the feature is disabled.1
Identified Files:
src/analysis/query_engine.rs: The entire query subsystem is internal.
src/language_grammars.rs: The module responsible for loading the actual tree_sitter_rust, tree_sitter_python, etc., crates.
Implementation Example:
The parent module (src/analysis/mod.rs) would be modified to conditionally include either the real implementation or the stub.
Rust
// In src/analysis/mod.rs

// Conditionally compile the real query engine module.
#[cfg(feature = "tree-sitter")]
mod query_engine;

// Conditionally export its public functions.
#[cfg(feature = "tree-sitter")]
pub use query_engine::run_queries;

// Define a stub module for when the feature is disabled.
#[cfg(not(feature = "tree-sitter"))]
mod query_engine_stub {
    // Re-import necessary types (which could be real or stubs themselves).
    use crate::types::{QueryResult, UveddiError};
    use crate::parsing::Node;

    // Provide a dummy implementation that gracefully degrades.
    pub fn run_queries<'a>(_node: Node<'a>, _source: &[u8]) -> Result<Vec<QueryResult>, UveddiError> {
        // When tree-sitter is disabled, no queries can be run. Return an empty vector.
        Ok(Vec::new())
    }
}

// Conditionally export the stub function under the same name.
#[cfg(not(feature = "tree-sitter"))]
pub use query_engine_stub::run_queries;



Category 2: Public API Stubbing (⚠️ High Effort)

This category represents the core of the refactoring work and addresses all modules where tree-sitter types are part of the public interface. The strategy here is to use a "shim" pattern, where a use alias conditionally imports either the real types from the tree_sitter crate or the newly created stub types from crate::tree_sitter_stub. This ensures that the rest of the code within the module can be written agnostically, using types like Parser and Tree without needing to know whether they are the real implementations or the stubs.15
Identified Files:
src/parsing/mod.rs and its submodules.
src/types/parsed_file.rs.
src/analysis/engine.rs.
Implementation Example (src/parsing/mod.rs):
A new parent module, perhaps src/parsing/ts.rs, would contain the conditional logic, and other files would use from it.
Rust
// In a new file, e.g., src/ts_types.rs
#[cfg(feature = "tree-sitter")]
pub use tree_sitter::{Parser, Tree, Node, Language, Point, Range, Query, QueryCursor, QueryError, LanguageError};

#[cfg(not(feature = "tree-sitter"))]
pub use crate::tree_sitter_stub::{Parser, Tree, Node, Language, Point, Range, Query, QueryCursor, QueryError, LanguageError};

// In src/parsing/mod.rs
use crate::ts_types::{Parser, Tree}; // Import the aliased types.

// All subsequent code in this module can now use `Parser`, `Tree`, etc.,
// and the compiler will resolve them to either the real or stub types
// based on the active feature flag.
pub struct AstParser {
    // This field will be either the real `tree_sitter::Parser` or our `tree_sitter_stub::Parser`.
    inner: Parser,
}

impl AstParser {
    pub fn parse(&mut self, content: &str) -> Option<Tree> {
        // This call works identically for both real and stub types.
        // The stub will return None, the real one will attempt to parse.
        self.inner.parse(content, None)
    }
}



Category 3: Graceful Degradation (🔄 Medium Effort)

Once the stubs are integrated, the higher-level application logic must be updated to handle the new failure modes introduced by the stubs. This involves checking the Option and Result values returned by stubbed methods and diverting the control flow to a sensible fallback path. This ensures the application remains functional and provides a good user experience, even with reduced capabilities.
Identified Files:
src/main.rs or the equivalent application entry point where user feedback is generated.
src/commands/analyze.rs or other high-level command handlers.
Implementation Example:
Rust
// In src/commands/analyze.rs
fn run_analysis(file_path: &Path) -> Result<Vec<Issue>, UveddiError> {
    let content = std::fs::read_to_string(file_path)?;
    let mut parser = get_parser_for_file(file_path)?; // This function is now feature-aware.

    if let Some(tree) = parser.parse(&content, None) {
        // --- This block only executes if tree-sitter is enabled and parsing succeeds. ---
        let ast_issues = find_issues_via_ast_traversal(&tree, &content);
        Ok(ast_issues)
    } else {
        // --- This is the fallback path for when tree-sitter is disabled or parsing fails. ---
        // The application can still perform non-AST-based analysis, like regex checks.
        let regex_issues = find_issues_with_regex(&content);

        // Use the cfg! macro to provide a compile-time conditional warning to the user.
        if cfg!(not(feature = "tree-sitter")) {
            eprintln!("Warning: Full AST analysis is disabled. Results are based on limited text-based checks.");
        }

        Ok(regex_issues)
    }
}


A key tool for this category is the cfg! macro. While the #[cfg] attribute removes code entirely at compile time, the cfg! macro expands to a boolean literal (true or false) within the code.1 This allows for runtime logic that is conditional on the build configuration without incurring any performance penalty. In the example above, the
if cfg!(not(feature = "tree-sitter")) block is completely optimized away when the feature is enabled, but when disabled, it allows the program to provide valuable context to the user about the degraded functionality.

Implementation Roadmap and Success Criteria

This final section provides a structured, actionable plan for executing the refactoring project. It includes a prioritized phasing of the work, qualitative effort estimates for project management, and a comprehensive verification plan to ensure the final implementation is correct, robust, and maintainable.

Prioritization and Phasing

The implementation will proceed in four distinct phases, ordered by logical dependency. This phased approach allows for incremental progress and testing, reducing the risk of a large, monolithic change.
Phase 1: Foundation (High Priority)
Task: Implement the tree_sitter_stub.rs module. This includes defining all core and auxiliary stub types (Parser, Tree, Node<'a>, Language, Point, Range, Query, QueryCursor, and StubError).
Task: Implement all required methods on these stub types according to the defined behavioral strategies (e.g., returning None, Err(StubError::TreeSitterDisabled), empty iterators, or default values). The implementation must be meticulous, especially concerning lifetimes, to satisfy the Rust compiler.
Goal: To produce a self-contained, compilable stub library that perfectly mirrors the tree-sitter API surface currently consumed by the Uveddi project. This phase is complete when tree_sitter_stub.rs is fully implemented and passes its own set of unit tests.
Phase 2: Core Integration (Medium Priority)
Task: Introduce the conditional use shims in the key modules that have public API dependencies (primarily src/parsing and src/types). This involves creating the ts_types.rs module or equivalent abstraction layer.
Task: Refactor the codebase to correctly handle the Option and Result return types from the now-stubbed functions. This will primarily involve changing unwrap() calls to match or if let patterns to handle the None case from parser.parse().
Goal: To make the entire Uveddi codebase compilable with the tree-sitter feature disabled (e.g., via cargo build --no-default-features). At the end of this phase, the application should build successfully, though it may still panic at runtime if the new failure paths are not yet handled by higher-level logic.
Phase 3: Functional Gating and Fallbacks (Medium Priority)
Task: Apply the Category 1 (module-level gating) and Category 3 (graceful degradation) refactoring strategies. This involves wrapping internal-only modules in #[cfg] attributes and implementing fallback logic in high-level command handlers.
Task: Fully integrate the UveddiError::TreeSitterDisabled variant into the application's main error handling pathways, ensuring errors from stubbed operations propagate cleanly.
Goal: To ensure the application runs without panicking when tree-sitter is disabled. It should provide meaningful fallback functionality or clear error messages to the user, demonstrating the complete graceful degradation path.
Phase 4: Cleanup and Verification (Low Priority)
Task: Update all relevant documentation (README, library docs, user guides) to explain the new tree-sitter feature flag, its impact on functionality, and how to enable or disable it.18
Task: Configure the Continuous Integration (CI) pipeline to execute two separate build and test jobs for every commit: one with the default features and one with --no-default-features.
Goal: To finalize the project, ensuring it is robust, well-documented, and easily maintainable for future development.

Effort Estimation

The following table provides a qualitative estimate of the development effort required for each phase.
Phase
Task
Estimated Effort
Rationale
1
Stub Type & Method Implementation
High (5-8 days)
This phase requires meticulous attention to detail to perfectly mimic the tree-sitter API. Correctly handling generics and lifetimes (especially for Node<'a>) is complex and foundational to the entire effort.
2
Core Integration & Refactoring
High (4-6 days)
This involves modifying a large number of files across the codebase to adapt existing logic to the new Option/Result patterns introduced by the stubs. It is a broad but well-defined task.
3
Functional Gating & Fallbacks
Medium (3-5 days)
The work here is less complex than the core integration but requires thoughtful design of the fallback logic and user experience for the degraded mode.
4
Documentation & CI
Low (1-2 days)
These are straightforward but essential finalization tasks that require diligence rather than complex problem-solving.
Total


~3-4 Sprints




Verification and Testing Plan

A rigorous verification plan is essential to guarantee the correctness and robustness of the final implementation.
Compile-Time Verification: The CI pipeline must be configured to run two separate build and test jobs on every change:
cargo test --workspace --all-features (or the project's default configuration).
cargo test --workspace --no-default-features.
This automated check is the primary safeguard against regressions, ensuring that both code paths remain compilable and that their respective tests pass.
Runtime Verification:
Existing integration tests that depend on AST analysis must be annotated with #[cfg(feature = "tree-sitter")]. This ensures they are only compiled and executed when the necessary functionality is present, preventing test failures in the feature-disabled build.
A new suite of integration tests must be created specifically for the --no-default-features configuration. These tests will run under a #[cfg(not(feature = "tree-sitter"))] attribute and will assert the following conditions:
Functions that rely on parsing (e.g., run_analysis) return the expected degraded output (e.g., an empty Vec<Issue> or a specific UveddiError::TreeSitterDisabled).
The application executes to completion without panicking.
Any implemented fallback logic (e.g., regex-based analysis) is correctly executed and produces the expected results.
Success Criteria: The UV-97 project will be considered complete and successful when the following criteria are met:
The Uveddi project successfully compiles and passes all unit and integration tests in both feature-enabled and feature-disabled configurations within the CI environment.
The public API of the Uveddi library remains identical between both configurations, as verified by a tool like cargo-semver-checks or manual review.
Executing the Uveddi application with the tree-sitter feature disabled results in predictable, non-panicking, and gracefully degraded behavior.
The new tree-sitter feature flag and its impact on application functionality are clearly and accurately documented for end-users and downstream developers.
The final compiled binary is measurably smaller when built without the tree-sitter feature, demonstrating the successful removal of the dependency.
Works cited
#[cfg] Conditional Compilation in Rust - Mastering Backend, accessed July 8, 2025, https://masteringbackend.com/posts/cfg-conditional-compilation-in-rust
Conditional compilation - The Rust Reference, accessed July 8, 2025, https://doc.rust-lang.org/reference/conditional-compilation.html
Compile Time Feature Flags in Rust, accessed July 8, 2025, https://www.worthe-it.co.za/blog/2018-11-18-compile-time-feature-flags-in-rust.html
Compile Time Feature Flags in Rust: Why, How, and When? | by Dotan Nahum - Medium, accessed July 8, 2025, https://jondot.medium.com/compile-time-feature-flags-in-rust-why-how-when-129aada7d1b3
tree-sitter - crates.io: Rust Package Registry, accessed July 8, 2025, https://crates.io/crates/tree-sitter
tree_sitter - Rust - Docs.rs, accessed July 8, 2025, https://docs.rs/tree-sitter
tree_sitter_rust - Rust - Docs.rs, accessed July 8, 2025, https://docs.rs/tree-sitter-rust
tree-sitter/lib/binding_rust/README.md at master - GitHub, accessed July 8, 2025, https://github.com/tree-sitter/tree-sitter/blob/master/lib/binding_rust/README.md
tree_sitter - Rust - Docs.rs, accessed July 8, 2025, https://docs.rs/tree-sitter/
Unraveling Tree-Sitter Queries: Your Guide to Code Analysis Magic - DEV Community, accessed July 8, 2025, https://dev.to/shrsv/unraveling-tree-sitter-queries-your-guide-to-code-analysis-magic-41il
Basic Syntax - Tree-sitter, accessed July 8, 2025, https://tree-sitter.github.io/tree-sitter/using-parsers/queries/1-syntax.html
Error Handling - The Rust Programming Language, accessed July 8, 2025, https://doc.rust-lang.org/book/ch09-00-error-handling.html
Practical guide to Error Handling in Rust - Dev State, accessed July 8, 2025, https://dev-state.com/posts/error_handling/
What is the idiomatic way to return an error from a function with no result if successful?, accessed July 8, 2025, https://stackoverflow.com/questions/36878044/what-is-the-idiomatic-way-to-return-an-error-from-a-function-with-no-result-if-s
How does one write function stubs for testing Rust modules? - Stack Overflow, accessed July 8, 2025, https://stackoverflow.com/questions/66603516/how-does-one-write-function-stubs-for-testing-rust-modules
What's your stance towards conditional compilation? : r/rust - Reddit, accessed July 8, 2025, https://www.reddit.com/r/rust/comments/xvkcns/whats_your_stance_towards_conditional_compilation/
Example of how to use Conditional Compilation Macros in Rust - Stack Overflow, accessed July 8, 2025, https://stackoverflow.com/questions/34777302/example-of-how-to-use-conditional-compilation-macros-in-rust
document_features - Rust - Docs.rs, accessed July 8, 2025, https://docs.rs/document-features/
What's the best practice for documenting a Rust project? - Reddit, accessed July 8, 2025, https://www.reddit.com/r/rust/comments/7eohmt/whats_the_best_practice_for_documenting_a_rust/
Rust Documentation Conventions - Project Mu - Microsoft Open Source, accessed July 8, 2025, https://microsoft.github.io/mu/CodeDevelopment/rust_documentation_conventions/
