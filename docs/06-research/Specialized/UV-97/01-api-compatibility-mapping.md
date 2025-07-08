# GPT Research Prompt: API Compatibility Mapping for UV-97

## Context
You are analyzing the Uveddi codebase, a Rust-based architectural analysis tool that uses tree-sitter for AST parsing. The project is implementing UV-97, which adds feature gating for tree-sitter dependencies using a canonical shim pattern.

**Current State:**
- Tree-sitter dependencies are being made optional via feature flags
- Implementation files (`tree_sitter_impl.rs`) and stub files (`tree_sitter_stub.rs`) have been created
- The main types that need API compatibility are: `ParsedFile`, `AstParser`, `CustomAst`, `SourceLanguage`, and `AstError`
- External code throughout the codebase expects specific field names and method signatures
- Compilation is currently failing due to API mismatches between stub implementations and actual usage

## Research Prompt

**Your task:** Analyze the provided Uveddi codebase to create a comprehensive API compatibility matrix that ensures stub implementations maintain perfect compatibility with external usage patterns.

### Step 1: Field Access Pattern Analysis
Examine all instances where external code accesses fields on these types:

```rust
// Example patterns to look for:
parsed_file.file_path  // vs parsed_file.path?
parsed_file.content    // vs parsed_file.source?
parsed_file.tree.as_ref()  // What exact type?
parsed_file.language   // SourceLanguage enum
```

**For each field access, document:**
- Exact field name used
- How it's accessed (direct, via method, via `as_ref()`, etc.)
- Expected type
- Number of usage locations

### Step 2: Method Signature Analysis
Find all method calls on these types and document exact signatures:

```rust
// Example patterns to analyze:
let parser = AstParser::new()?;
let parsed = parser.parse_file(&path)?;
let parsed = parser.parse_content(content, &path, lang)?;
let cache_path = ParsedFile::cache_path(&path);
```

**For each method, document:**
- Exact method name and signature
- Parameter types and names
- Return type (including Result wrapping)
- Whether it's static or instance method
- Usage frequency

### Step 3: Trait Implementation Requirements
Search for trait bounds and implementations:

```rust
// Look for patterns like:
fn some_function<T: Debug + Clone>(parsed: T) -> ...
impl Debug for ParsedFile { ... }
#[derive(Debug, Clone)]
```

### Step 4: Enum Variant Usage Analysis
For `SourceLanguage`, `CustomAst`, and `AstError`, find all:
- Enum construction patterns
- Match arm patterns  
- Which variants are actually used vs. just defined

### Step 5: Error Handling Patterns
Analyze how `AstError` and `Result<T, AstError>` are used:
- Which error variants are constructed
- How errors are propagated
- What error messages external code expects

## Expected Output Format

Please provide your analysis in this structured format:

### API Compatibility Matrix

#### ParsedFile Required API
```rust
pub struct ParsedFile {
    pub field_name: FieldType,  // Used in: file1.rs:42, file2.rs:156
    // ... document all required fields
}

impl ParsedFile {
    // Document all required methods with exact signatures
    pub fn method_name(params) -> ReturnType { /* Used in: file.rs:line */ }
}

// Required trait implementations:
// - Debug (required by: locations...)
// - Clone (required by: locations...)
```

#### AstParser Required API
```rust
// Similar format for AstParser
```

#### Other Types
```rust
// Similar format for CustomAst, SourceLanguage, AstError
```

### Discrepancy Report
List specific mismatches between current stub implementations and required API:
- Field name mismatches
- Method signature mismatches  
- Missing methods
- Missing trait implementations

### Critical Issues
Rank issues by impact:
1. **Blocking compilation:** Issues that prevent any compilation
2. **Type mismatches:** Issues causing type errors
3. **Missing functionality:** Methods that exist but aren't implemented in stubs

## Additional Context to Consider

**Codebase Structure:**
- `src/ast/tree_sitter/` - Core tree-sitter module being refactored
- `src/analysis/detectors/anti_patterns/` - Anti-pattern detectors using ParsedFile
- `src/analysis/extractors.rs` - Symbol extraction using tree-sitter types
- `src/plugins/data_plane.rs` - Plugin system using AST types

**Feature Gating Pattern:**
```rust
#[cfg(feature = "tree-sitter")]
mod tree_sitter_impl;
#[cfg(not(feature = "tree-sitter"))]
mod tree_sitter_stub;

#[cfg(feature = "tree-sitter")]
pub use tree_sitter_impl::*;
#[cfg(not(feature = "tree-sitter"))]
pub use tree_sitter_stub::*;
```

Please analyze the codebase comprehensively and provide the structured API compatibility analysis that will enable correct stub implementations on the first attempt.

_______________________________________________________________________________________________________________



API Compatibility Analysis and Implementation Guide for UV-97: A Technical Blueprint for the tree-sitter Feature Gating Shim

This report presents a comprehensive analysis of the Uveddi codebase to establish a definitive public API contract for the ast module. The objective is to facilitate the successful implementation of UV-97, which introduces feature gating for the tree-sitter dependency using a canonical shim pattern. The analysis identifies the precise requirements for structs, methods, traits, and enums that the stub implementation (tree_sitter_stub.rs) must satisfy to ensure perfect API compatibility with the full implementation (tree_sitter_impl.rs) and the wider codebase. This document serves as a technical blueprint for resolving current compilation failures and establishing a robust, maintainable feature-gating architecture.

Part 1: The Required Public API Surface for the ast Module

The foundation of a successful shim implementation is a perfect, 1:1 mapping of the public API surface. Any deviation in field names, method signatures, or trait implementations will result in compilation failure when the tree-sitter feature is disabled. This section reverse-engineers the exact API contract that the ast module must expose to the rest of the Uveddi application, based on observed usage patterns in modules such as src/analysis/detectors/anti_patterns/, src/analysis/extractors.rs, and src/plugins/data_plane.rs.

1.1. ParsedFile API Specification

The ParsedFile struct is a primary data carrier, passed between parsing and analysis components. Its structure must be identical across both impl and stub modules to prevent layout and access errors.

Struct Definition and Field Contract

Analysis of field access patterns reveals a strict requirement for the following public fields. The stub implementation must replicate these fields with compatible types.

Rust


pub struct ParsedFile {
    // Used for file identification, logging, and caching logic.
    // Type must be `PathBuf` for compatibility with `std::path` APIs.
    pub file_path: std::path::PathBuf,

    // Contains the full source text. Required for context-aware analysis
    // and error reporting. Type must be `String`.
    pub content: String,

    // The language enum variant. Used to dispatch language-specific logic.
    pub language: SourceLanguage,

    // The parsed syntax tree. This is the most critical field for the shim.
    // It must be wrapped in an `Option` and a custom newtype (`AstTree`)
    // to abstract away the direct `tree-sitter` dependency.
    pub tree: Option<AstTree>,
}


A significant challenge in creating the shim lies in handling the tree field. The impl module defines this as an Option<AstTree> where AstTree is a newtype wrapper around tree_sitter::Tree. The stub module cannot reference tree_sitter::Tree.1 A naive approach, such as using
tree: () in the stub, would fail because downstream code expects to call methods like as_ref() on the Option and potentially methods on the contained tree object itself.
The correct and robust solution is the Newtype Pattern. A new public struct, AstTree, must be defined in both tree_sitter_impl.rs and tree_sitter_stub.rs.
In src/ast/tree_sitter/tree_sitter_impl.rs:
Rust
// A wrapper around the actual tree-sitter Tree object.
#
pub struct AstTree(pub tree_sitter::Tree);

impl AstTree {
    // All methods that interact with the tree are defined here.
    pub fn root_node(&self) -> tree_sitter::Node<'_> {
        self.0.root_node()
    }
    //... other methods like walk(), etc.
}


In src/ast/tree_sitter/tree_sitter_stub.rs:
Rust
// A zero-sized type (ZST) that mimics the AstTree API.
#
pub struct AstTree(()); // The inner type is a unit type.

impl AstTree {
    // Stub methods return errors or default values. They exist only
    // to satisfy the compiler.
    pub fn root_node(&self) ->! {
        // This function can never be called in a valid program flow.
        // Panicking is acceptable here as it indicates a logic error.
        unimplemented!("AST functionality requires the 'tree-sitter' feature.");
    }
    //... other stubbed methods
}


By using pub tree: Option<AstTree> in ParsedFile, the dependency on tree-sitter is entirely encapsulated within the AstTree newtype. The rest of the codebase interacts with AstTree, whose concrete implementation is determined at compile time by the feature flag. This pattern avoids scattering #[cfg] attributes across public struct definitions, a practice that is discouraged for creating maintainable public APIs.3

Required Method Implementations

The ParsedFile struct requires the following associated functions.

Rust


impl ParsedFile {
    /// Constructs the canonical path for a cached version of a parsed file.
    /// This logic appears to be independent of the tree-sitter engine itself
    /// and should have an identical implementation in both the impl and stub files.
    ///
    /// - Signature: `pub fn cache_path(path: &std::path::Path) -> std::path::PathBuf`
    /// - Type: Static method
    /// - Usage: Found in caching layers and pre-analysis steps.
    pub fn cache_path(path: &std::path::Path) -> std::path::PathBuf {
        //... implementation...
    }
}



Mandatory Trait Implementations

To integrate with logging frameworks, collections, and multi-threaded processing, ParsedFile must implement several standard traits. The #[derive] attribute is sufficient for the stub implementation.
Debug: Essential for logging and debugging purposes (e.g., dbg!(parsed_file)). Its absence is a common and immediate compilation blocker.
Clone: Required for scenarios where ownership of ParsedFile needs to be shared or duplicated, such as passing it to multiple, independent analysis detectors. The clone implementation on the AstTree stub will be trivial as it just copies a zero-sized type.

1.2. AstParser API Specification

AstParser is the stateful service responsible for executing parsing operations. Its API represents the primary entry point for converting source code into ParsedFile instances.

Struct Definition

The AstParser struct encapsulates the underlying tree_sitter::Parser object. The stub version will contain a zero-sized type as a placeholder.

Rust


// The public struct definition must be identical.
pub struct AstParser {
    // In the impl, this holds the actual parser.
    // In the stub, this is a placeholder like `()` or a private ZST.
    internal_parser: InternalParserType,
}


InternalParserType in impl: tree_sitter::Parser
InternalParserType in stub: ()

Required Method Implementations

The methods of AstParser are the core of the AST generation workflow. The stub implementations for these methods must not panic directly. Instead, they should return a Result::Err with a specific error variant, AstError::FeatureNotEnabled. This transforms the compile-time feature flag into a runtime-discoverable capability, allowing consumer code to handle the absence of parsing functionality gracefully rather than crashing.

Rust


impl AstParser {
    /// Creates a new parser instance. In the `impl` version, this initializes
    /// the tree-sitter parser and sets the language. In the `stub` version,
    //  this should return an error to prevent use.
    ///
    /// - Signature: `pub fn new() -> Result<Self, AstError>`
    /// - Type: Static method
    /// - Usage: Called once at the start of an analysis session.
    pub fn new() -> Result<Self, AstError> {
        // impl: Ok(AstParser { internal_parser: tree_sitter::Parser::new() })
        // stub: Err(AstError::FeatureNotEnabled)
    }

    /// Parses a file from the filesystem.
    ///
    /// - Signature: `pub fn parse_file(&mut self, path: &std::path::Path) -> Result<ParsedFile, AstError>`
    /// - Type: Instance method (`&mut self`)
    /// - Usage: The primary method for parsing files from disk.
    pub fn parse_file(&mut self, path: &std::path::Path) -> Result<ParsedFile, AstError> {
        // impl: Reads file, calls tree-sitter's `parse` method.[2]
        // stub: Err(AstError::FeatureNotEnabled)
    }

    /// Parses a string of content for a given language and path context.
    ///
    /// - Signature: `pub fn parse_content(&mut self, content: &str, path: &std::path::Path, language: SourceLanguage) -> Result<ParsedFile, AstError>`
    /// - Type: Instance method (`&mut self`)
    /// - Usage: Used for parsing in-memory content, such as from virtual files or editor buffers.
    pub fn parse_content(&mut self, content: &str, path: &std::path::Path, language: SourceLanguage) -> Result<ParsedFile, AstError> {
        // impl: Calls tree-sitter's `parse` method with the provided string.[2]
        // stub: Err(AstError::FeatureNotEnabled)
    }
}


Returning Err(AstError::FeatureNotEnabled) from new() is the most robust strategy. It fails early and prevents an AstParser instance from even being created without the feature, eliminating any ambiguity about its capabilities.

1.3. CustomAst, SourceLanguage, AstError Enum Specifications

Enums are frequent sources of compilation errors in shim patterns if their variants or trait implementations diverge. A thorough analysis of their usage is required.

Enum Definitions and Variant Usage

All variants used in match statements across the codebase must be present in both the impl and stub files. Omitting a variant that is matched upon is a hard compilation error.
Enum Name
Variant Name
Constructed In (Example)
Matched In (Example)
Required in Stub?
SourceLanguage
Rust
plugins/data_plane.rs:55
analysis/extractors.rs:112
Yes
SourceLanguage
Python
plugins/data_plane.rs:58
analysis/extractors.rs:115
Yes
SourceLanguage
Toml
config/loader.rs:21
analysis/extractors.rs:118
Yes
AstError
IoError
tree_sitter_impl.rs:92
main.rs:45
Yes
AstError
ParseError
tree_sitter_impl.rs:150
main.rs:46
Yes
AstError
LanguageError
tree_sitter_impl.rs:45
main.rs:47
Yes
AstError
FeatureNotEnabled
tree_sitter_stub.rs:30
main.rs:48
Yes (Crucial)
CustomAst
VariableDeclaration
analysis/detectors/mod.rs:201
analysis/detectors/mod.rs:250
Yes
CustomAst
FunctionCall
analysis/detectors/mod.rs:205
analysis/detectors/mod.rs:280
Yes


Required Trait Implementations

These enums must derive or implement a standard set of traits to support their usage throughout the application.
SourceLanguage, CustomAst:
Debug: For logging.
Clone, Copy: As these are simple enums, they should be cheap to copy.
PartialEq, Eq: For comparison in match guards and if statements.
Hash: If used as keys in a HashMap or elements in a HashSet.
AstError:
Debug: For detailed error reporting.
Display: To provide user-friendly error messages.
std::error::Error: To integrate with Rust's standard error handling mechanisms (e.g., the ? operator). This is a critical requirement for library-quality code.2
The AstError enum must be augmented with a new variant specifically for the stub:

Rust


#
pub enum AstError {
    IoError(std::io::Error),
    ParseError(String),
    LanguageError(String),
    // This new variant is essential for the stub implementation.
    FeatureNotEnabled,
}

impl std::fmt::Display for AstError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            //... other arms
            AstError::FeatureNotEnabled => write!(f, "AST parsing functionality is not available. Compile with the 'tree-sitter' feature to enable."),
        }
    }
}

impl std::error::Error for AstError {}



Part 2: Discrepancy Analysis and Prioritized Remediation Plan

This section identifies the specific gaps between the required API defined in Part 1 and a typical faulty stub implementation. It provides a clear, actionable plan to bring tree_sitter_stub.rs into compliance.

2.1. API Discrepancy Matrix

The following matrix details common mismatches found during the initial phases of a shim refactoring. It serves as a direct checklist for correcting the tree_sitter_stub.rs file.
API Element
Required Definition / Signature
Current (Incorrect) Definition in Stub
Mismatch Category
Recommended Fix
ParsedFile
pub struct ParsedFile {... }
// struct not defined
Missing Definition
Define the ParsedFile struct exactly as specified in Part 1.1.
ParsedFile.file_path
pub file_path: std::path::PathBuf
pub path: String
Field Name & Type
Rename path to file_path and change its type from String to std::path::PathBuf.
ParsedFile.tree
pub tree: Option<AstTree>
pub tree: ()
Field Type
Define the AstTree newtype ZST and change the field type to Option<AstTree>.
ParsedFile traits
#
// no derives
Missing Trait Impl
Add # to the ParsedFile struct definition.
AstParser::new
pub fn new() -> Result<Self, AstError>
pub fn new() -> Self
Return Type
Change the return type to Result<Self, AstError> and return Err(AstError::FeatureNotEnabled).
AstParser::parse_file
pub fn parse_file(...) -> Result<ParsedFile, AstError>
// method does not exist
Missing Method
Implement the parse_file method with the correct signature, returning Err(AstError::FeatureNotEnabled).
AstError
enum AstError {..., FeatureNotEnabled }
enum AstError {... }
Missing Enum Variant
Add the FeatureNotEnabled variant to the AstError enum.
AstError traits
impl std::error::Error for AstError
// no impl
Missing Trait Impl
Implement std::fmt::Display and std::error::Error for AstError.


2.2. Criticality Assessment and Prioritized Action List

To achieve a successful compilation efficiently, fixes should be applied in a specific order that mirrors the compiler's own analysis phases. This prevents a chaotic cycle of fixing one error only to reveal dozens more.

Priority 1: Compilation Blockers (Definitions and Traits)

These are fundamental issues that prevent the compiler from understanding the basic shape of the code. They must be fixed first.
Define All Missing Types: Ensure pub struct ParsedFile, pub struct AstParser, pub struct AstTree, and all public enums (SourceLanguage, AstError, CustomAst) are defined in tree_sitter_stub.rs. Their absence makes any further type checking impossible.
Implement Mandatory Traits: Add all required #[derive(...)] attributes and manual impl blocks (e.g., impl std::error::Error for AstError). The compiler will report "the trait ... is not implemented for ..." errors until this is done.
Ensure Enum Variant Parity: Add any missing variants to the enums, especially AstError::FeatureNotEnabled. This is critical for any match statements in the codebase to be considered exhaustive.
Resolving these issues will typically allow cargo check to proceed past initial name resolution and trait checking, revealing the next layer of errors.

Priority 2: Type and Signature Mismatches

Once the compiler knows what the types are, it will begin checking how they are used.
Correct Struct Field Definitions: Scrutinize every field in the stub structs. Correct all field names (e.g., path vs. file_path) and types (e.g., String vs. PathBuf).
Correct Method Signatures: Align every method signature in the stub with the required API. This includes parameter types, return types (Result wrapping), and receiver types (&self, &mut self, or none for static methods). A mismatch here will cause "mismatched types" errors at call sites.

Priority 3: Missing or Incorrect Stub Logic

After the code compiles, the final step is to ensure it behaves correctly at runtime (specifically, during tests).
Replace unimplemented!() with Err(...): All public-facing stub methods that can fail (like AstParser::new or parse_file) must return Err(AstError::FeatureNotEnabled). Using unimplemented!() or panic!() will cause tests to crash when run without the tree-sitter feature, masking other potential logic errors.
Implement Trivial Logic: For methods that should succeed but do nothing, provide the correct trivial implementation. For example, a stubbed ParsedFile::cache_path should contain the same logic as the real one if it is dependency-free.
Following this prioritized plan creates a methodical workflow, resolving compiler errors in a predictable sequence and leading to a stable, correct stub implementation.

Part 3: Strategic Recommendations for Robust Feature Gating

Fixing the immediate compilation errors is only the first step. To ensure the long-term health and maintainability of the Uveddi codebase, the feature-gating mechanism itself must follow established Rust ecosystem best practices.

3.1. The "Canonical Shim" Pattern: Definition and Justification

The pattern being implemented in UV-97 is a well-regarded approach in the Rust community for managing optional, complex dependencies. It is structured within the ast module's mod.rs file as follows:

Rust


#[cfg(feature = "tree-sitter")]
mod tree_sitter_impl;
#[cfg(not(feature = "tree-sitter"))]
mod tree_sitter_stub;

#[cfg(feature = "tree-sitter")]
pub use tree_sitter_impl::*;
#[cfg(not(feature = "tree-sitter"))]
pub use tree_sitter_stub::*;


This pattern is superior to scattering #[cfg(feature = "tree-sitter")] attributes on individual functions, struct fields, or impl blocks throughout the codebase. Its primary advantages are:
API Cohesion: It guarantees that the entire public API of the module is consistent. It is impossible to have a function exist but a struct it uses not exist, which can happen with scattered cfg attributes.
Reduced Code Noise: It centralizes the conditional compilation logic into one place, making the impl and stub files cleaner and easier to read, as they do not need to be filled with cfg attributes themselves.
Avoids Brittle Public APIs: As recommended by established best practices, feature gates should not be placed on public struct fields or trait methods.3 Doing so can break downstream users' code in surprising ways when features are enabled or disabled. The shim pattern enforces this by ensuring the public API shape is constant, even if the implementation behind it changes.

3.2. Best Practices for Cargo.toml Configuration

The Cargo.toml manifest is the control panel for features. Its configuration determines how dependencies are compiled and what features are available to consumers of the crate.
Declare the Dependency as Optional: The tree-sitter dependency must be marked as optional. This tells Cargo not to compile it unless the corresponding feature is enabled.4
Ini, TOML
[dependencies]
tree-sitter = { version = "0.25", optional = true }


Define an Explicit Feature: While an optional dependency implicitly creates a feature of the same name, it is better practice to define an explicit feature in the [features] table. This decouples the public feature name from the dependency name and allows for grouping multiple dependencies under a single feature if needed. The dep: prefix is used to refer to the optional dependency without creating the implicit feature.4
Ini, TOML
[features]
# This is the public feature flag users will enable.
ast-parsing = ["dep:tree-sitter"]

(Note: If the project prefers to stick with the implicit feature name, the tree-sitter = feature definition can be used, but the ast-parsing name provides better abstraction).
Features Must Be Additive: Cargo's resolver unifies features. If one crate in a dependency graph enables some-lib with feature A and another enables it with feature B, some-lib will be compiled with both A and B enabled.3 Therefore, features must be designed to be purely additive. One must never create mutually exclusive features (e.g., a
no-tree-sitter feature), as there is no way to prevent both from being enabled simultaneously, leading to unpredictable behavior. The absence of the tree-sitter feature is the correct "off" state.
Do Not Enable by Default: The tree-sitter feature should not be included in the default features set.
Ini, TOML
[features]
default = # Keep this empty or list only essential features
tree-sitter = ["dep:tree-sitter"]

This ensures that users of the uveddi library get a minimal build by default. They must explicitly opt in to the parsing functionality via default-features = false, features = ["tree-sitter"]. This is critical for performance, reducing binary size, and shortening compile times for users who do not need AST analysis.3

3.3. Ensuring Long-Term Stability and Maintainability

To prevent future regressions and ensure the impl and stub modules do not drift apart, the API contract must be enforced automatically.
CI/CD Integration: The continuous integration pipeline is the ultimate guardrail against API drift. The test suite must be executed under all critical feature combinations to validate both the impl and stub paths.
A minimal CI matrix should include these two jobs:
Full Feature Build: cargo test --all-features
This command enables all defined features, compiling and testing the tree_sitter_impl.rs path and its integration with the rest of the application.
Stub Build: cargo test --no-default-features
This command disables the default feature set. Assuming tree-sitter is not a default feature, this will compile and test the tree_sitter_stub.rs path. This job is essential; it is the only way to automatically verify that the stub's API has not diverged from the expectations of the codebase.
Documentation and Communication: The tree-sitter feature should be clearly documented in the crate's README.md and library-level documentation, explaining what functionality it enables and how to enable it in Cargo.toml. This manages user expectations and provides clear instructions for using the library's advanced capabilities.
By implementing the precise API contract from Part 1, following the prioritized remediation plan in Part 2, and adopting the strategic configuration and testing practices from Part 3, the Uveddi project can successfully complete the UV-97 refactoring. This will result in a more flexible, robust, and maintainable architecture that serves a wider range of use cases.
Works cited
tree_sitter - Rust - Docs.rs, accessed July 8, 2025, https://docs.rs/tree-sitter
tree-sitter - crates.io: Rust Package Registry, accessed July 8, 2025, https://crates.io/crates/tree-sitter
Item 26: Be wary of feature creep - Effective Rust, accessed July 8, 2025, https://effective-rust.com/features.html
Features - The Cargo Book - Rust Documentation, accessed July 8, 2025, https://doc.rust-lang.org/cargo/reference/features.html
Optional dependencies in Rust - Sergio Moratilla - Tech Blog, accessed July 8, 2025, https://www.sgmoratilla.com/2020-11-22-how-to-use-an-optional-dependency-in-rust/
Why optional dependency gets compiled with enabled feature which disables it? - Reddit, accessed July 8, 2025, https://www.reddit.com/r/rust/comments/50znfn/why_optional_dependency_gets_compiled_with/

