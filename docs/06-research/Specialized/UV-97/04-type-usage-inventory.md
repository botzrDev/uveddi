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
