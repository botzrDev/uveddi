
A Technical Report on Idiomatic Data Conversion and Architectural Refactoring in Rust for Static Analysis Tooling


Introduction: Navigating Rust's Type System for High-Integrity Static Analysis

The development of sophisticated static analysis tools presents a unique intersection of challenges, demanding high performance, absolute correctness, and maintainable architecture. When integrating external C-based libraries such as tree-sitter into a Rust project, developers must navigate the boundary between Rust's rich, safe type system and the more primitive data representations expected by C APIs.1 This report addresses two critical, yet common, challenges that arise in this context: a low-level data conversion problem and a high-level architectural flaw.
The first challenge involves a persistent compilation error when converting an Option<String> to a byte slice (&[u8]), a necessary step for feeding source code to the tree-sitter parsing and query engine. The second concerns a type mismatch in a detector module, where an abstraction has become "leaky," allowing primitive string types to penetrate deep into domain logic that expects a well-defined ArchitecturalLayer enum.
These issues, while seemingly distinct, are two facets of a single, fundamental goal in Rust programming: ensuring type safety and correctness at every level of the application stack. This report provides a definitive analysis and a set of robust solutions for these problems. It will demonstrate that mastering Rust's powerful trait system—specifically the AsRef, Deref, and FromStr traits—and adopting type-driven design patterns are the keys to building high-integrity, performant, and maintainable analysis tools. The analysis will progress from the specific data marshalling problem to the broader architectural refactoring, culminating in a synthesis of principles and a practical implementation blueprint for a robust static analysis system in Rust.

Part I: Mastering Data Conversion for tree-sitter Integration

This section systematically deconstructs the data conversion problem encountered when interfacing with tree-sitter. It provides a root-cause analysis of the compiler error and presents a suite of idiomatic, performant solutions, detailing the trade-offs of each approach.

Section 1.1: The tree-sitter API and the &[u8] Requirement

To correctly interface with any external library, one must first understand the contract of its API. The tree-sitter library, being a parser generator written in C, is designed to be language-agnostic and operate directly on raw bytes.1 The official Rust bindings for
tree-sitter reflect this design choice.
The primary method for parsing source code, tree_sitter::Parser::parse, and the subsequent methods for executing queries, such as tree_sitter::QueryCursor::matches, expect the source code to be provided in a format that can be viewed as a byte slice. The signature for the parse method is particularly instructive:

Rust


pub fn parse<'a, P: AsRef<[u8]>, T: TextProvider<'a>>(
    &mut self,
    text: P,
    old_tree: Option<&Tree>
) -> Option<Tree>


The key constraint here is P: AsRef<[u8]>.2 This generic bound means the function will accept any type
P that can provide a cheap, borrowed reference to a byte slice (&[u8]). This is a common and idiomatic pattern in Rust for creating flexible APIs that can accept various string-like or byte-like inputs without forcing the caller to perform costly conversions or allocations.4
While Rust's native String and &str types are guaranteed to be valid UTF-8 sequences, tree-sitter itself does not enforce this constraint at its C API boundary.6 By accepting
&[u8], the library remains maximally general, capable of parsing files with different encodings or even binary data, although individual language grammars may impose their own encoding expectations. Therefore, the task of converting a source file, which may be represented as an Option<String>, into a &[u8] is a fundamental prerequisite for using tree-sitter effectively. The goal is to perform this conversion with zero allocations and correct lifetime management.

Section 1.2: Deconstructing the AsRef Type Annotation Error

The compilation error described—cannot resolve <std::string::String as AsRef<T>>::as_ref—is a classic case of ambiguity in Rust's trait resolution system. It arises not from a mistake in logic, but from the compiler having too many valid choices and lacking the context to select the correct one.
Let's analyze the source of this ambiguity. The AsRef<T> trait is designed for cheap, reference-to-reference conversions.5 A single type can implement
AsRef<T> for multiple different target types T. The std::string::String type is a prime example of this, as it can be viewed in several ways 5:
As a string slice: impl AsRef<str> for String
As a byte slice: impl AsRef<[u8]> for String
As a path slice: impl AsRef<std::path::Path> for String
When the compiler encounters a method call like some_string.as_ref(), it searches for implementations of the AsRef trait for the type of some_string. Because String provides multiple such implementations, the compiler cannot infer what the target type T should be. Is the desired output a &str, a &[u8], or a &Path? Without further information, the compiler must stop and report an error, requesting that the developer disambiguate the call.8
This situation is common when using generic traits and is a deliberate feature of Rust's design, preventing the compiler from making potentially incorrect assumptions.10 The error message is the compiler's request for explicit instruction.
While one could resolve this ambiguity directly using the fully qualified "turbofish" syntax, this approach is often verbose and unidiomatic for this specific problem:

Rust


// Verbose and explicit, but works
let source_bytes = parsed_file.content.as_ref()
   .map(|s| AsRef::<[u8]>::as_ref(s))
   .unwrap_or(&);


Another way is to provide a type hint through a variable binding:

Rust


// Also works, but can be cumbersome
let source_bytes: &[u8] = parsed_file.content.as_ref()
   .map(|s| s.as_ref()) // The type hint on `source_bytes` helps the compiler here
   .unwrap_or(&);


However, these solutions treat the symptom rather than the root cause. The most idiomatic Rust solutions leverage more specific methods and traits that avoid this ambiguity altogether, leading to cleaner and more expressive code.

Section 1.3: Idiomatic and Robust Conversion Patterns

The most effective solutions to this problem involve using methods on Option and String that are more specific than the generic AsRef::as_ref method, thereby avoiding the ambiguity that causes the compilation error.

Primary Solution: Option::as_deref

Since Rust 1.40, the standard library has provided Option::as_deref, a method specifically designed for this type of conversion. It is the most concise and idiomatic solution.11

Rust


// The recommended, idiomatic solution
let source_bytes: &[u8] = parsed_file.content.as_deref()
   .map(str::as_bytes)
   .unwrap_or(&);


This approach is superior for several reasons. It leverages the Deref trait, which is a more fundamental relationship than AsRef. The String type implements Deref<Target = str>, signifying that a String can be transparently treated as a &str via deref coercion. The as_deref method is built to utilize this core relationship, directly converting an &Option<String> into an Option<&str>.
Once we have an Option<&str>, the subsequent call to .map(str::as_bytes) is unambiguous. The str::as_bytes method is a concrete function that takes a &str and returns a &[u8].6 There is no trait ambiguity. This chain of operations is zero-cost, involving no allocations, and correctly manages the lifetime of the resulting byte slice, ensuring it cannot outlive the original
String from which it is borrowed. The use of as_deref signals a clear understanding of Rust's ownership and borrowing model, particularly the role of smart pointers and deref coercion.

Alternative Solution: Option::map_or

Another highly idiomatic and explicit pattern uses the map_or method on Option. This method elegantly combines the logic of mapping a Some value and providing a default for a None value into a single function call.11

Rust


// A clean, explicit, and highly readable alternative
let source_bytes: &[u8] = parsed_file.content.as_ref()
   .map_or(&, |s| s.as_bytes());


This solution works by first calling .as_ref() on the Option<String>, which produces an Option<&String>. This step is unambiguous. Then, map_or is called. Its first argument, &, provides the default empty byte slice for the None case. The second argument is a closure, |s| s.as_bytes(), which is executed if the option is Some. The closure's parameter s has the type &String. The call s.as_bytes() is a direct method call on String, not a trait method, so again, no ambiguity arises. This pattern is extremely clear and effective, making it an excellent choice.

Advanced Consideration: Cow<str>

In more complex scenarios where the default value might also need to be an owned type (e.g., a non-empty default string), the std::borrow::Cow (Clone-on-Write) smart pointer can be useful. While not necessary for the specific problem of providing an empty slice, it is a powerful tool for writing functions that can efficiently handle both borrowed and owned data.11 For this use case, however, it would be overkill.

Comparison of Conversion Methods

To provide a clear reference, the following table compares the viable conversion patterns.
Method
Example Snippet
Pros
Cons
Idiomatic Level
Option::as_deref
opt.as_deref().map(str::as_bytes).unwrap_or(&)
Most concise, leverages fundamental Deref trait, zero-cost.
Requires understanding Deref coercion.
High (Recommended)
Option::map_or
`opt.as_ref().map_or(&,
s
s.as_bytes())`
Very explicit, combines map and unwrap, avoids ambiguity.
Option::as_ref + map
`opt.as_ref().map(
s
s.as_bytes()).unwrap_or(&)`
Step-by-step logic is clear.
match statement
match &opt { Some(s) => s.as_bytes(), None => & }
Most explicit, ultimate control flow.
Most verbose, generally overkill for this task.
Situational

For the task of supplying source code to tree-sitter, both the as_deref and map_or patterns are excellent, idiomatic choices. The as_deref approach is arguably the most elegant due to its direct use of the Deref relationship.

Part II: Refactoring for Architectural Integrity

This section addresses the second, more architectural problem in the leaky_abstraction.rs module. It provides a clear path to a robust, type-safe design by applying standard Rust idioms for validation and error handling.

Section 2.1: Diagnosing the Architectural Flaw in leaky_abstraction.rs

The core issue described—a function expecting &ArchitecturalLayer being called with a &String and failing in a pattern match—is a symptom of a deeper design flaw known as a "leaky abstraction." The implementation detail of how an architectural layer is represented in an external source (e.g., a configuration file or source code annotation), namely as a string, is "leaking" into the core domain logic of the detector.
This design violates a fundamental principle of robust software development, which is particularly well-supported by Rust's type system: making illegal states unrepresentable. In the current design, it is possible for a String that does not correspond to a valid ArchitecturalLayer to be passed deep into the application. The error is only discovered late, at runtime, when a match statement fails to find a corresponding arm.
A more robust architecture would enforce the validity of the architectural layer at the boundary of the system. The conversion from a raw String or &str into the ArchitecturalLayer enum should be a distinct, fallible step that occurs as early as possible. Once that conversion succeeds, the rest of the program can operate on a variable of type ArchitecturalLayer with the compile-time guarantee that it represents a valid state. The goal of the refactoring, therefore, is not merely to fix the failing match statement but to redesign the flow of data to leverage the type system to prevent invalid data from propagating.

Section 2.2: The FromStr Trait: An Idiomatic Bridge from &str to enum

The canonical Rust pattern for a fallible conversion from a string slice (&str) to a custom type is the std::str::FromStr trait.13 Its single method,
from_str, returns a Result, which perfectly models the possibility of a parsing failure. Implementing this trait for the ArchitecturalLayer enum establishes a clear and reusable validation boundary.

Manual FromStr Implementation

One can implement the trait manually. This involves defining a custom error type and writing a match statement to handle the string-to-enum-variant mapping.

Rust


// Assumed enum definition for architectural layers
#
pub enum ArchitecturalLayer {
    Application,
    Domain,
    Infrastructure,
}

// A simple, custom error type for parsing failures
#
pub struct ParseLayerError {
    invalid_input: String,
}

impl std::fmt::Display for ParseLayerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid architectural layer: {}", self.invalid_input)
    }
}

impl std::error::Error for ParseLayerError {}

// Manual implementation of the FromStr trait
impl std::str::FromStr for ArchitecturalLayer {
    type Err = ParseLayerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "application" => Ok(Self::Application),
            "domain" => Ok(Self::Domain),
            "infrastructure" => Ok(Self::Infrastructure),
            _ => Err(ParseLayerError { invalid_input: s.to_owned() }),
        }
    }
}


While functional, this approach requires significant boilerplate, especially for the error type, and the match statement can become a source of maintenance errors if new enum variants are added but not included in the from_str implementation.

Automated Implementation with the strum Crate

A more modern, maintainable, and pragmatic approach is to use the strum and strum_macros crates. These crates provide a derive macro, EnumString, that automatically generates a robust FromStr implementation.13
First, add the dependencies to your Cargo.toml:

Ini, TOML


[dependencies]
strum = { version = "0.26", features = ["derive"] }
strum_macros = "0.26"


Then, the implementation becomes trivial:

Rust


use strum_macros::EnumString;

#
#[strum(serialize_all = "snake_case", ascii_case_insensitive)]
pub enum ArchitecturalLayer {
    Application,
    Domain,
    Infrastructure,
}


The # attribute instructs strum to generate the FromStr implementation. The #[strum(...)] attribute provides additional configuration. Here, serialize_all = "snake_case" ensures that the string representation matches the variant name in snake case (e.g., Application matches "application"), and ascii_case_insensitive allows for flexible parsing. This approach eliminates boilerplate, reduces the risk of human error, and makes the code significantly cleaner and easier to maintain.

Section 2.3: Architecting a Robust Detector with Custom Errors

With a reliable FromStr implementation in place, the next step is to integrate it into the detector's workflow with best-practice error handling. The thiserror crate is the industry standard for creating ergonomic, library-friendly error enums.16
First, define a comprehensive DetectorError enum that encapsulates all possible failure modes within the detector module. This creates a single, unified error type that can be propagated cleanly.

Rust


// Add to Cargo.toml:
// thiserror = "1.0"

use thiserror::Error;
use strum::ParseError; // The error type from strum

#
pub enum DetectorError {
    #[error("Failed to parse architectural layer from string")]
    LayerParseError(#[from] ParseError),

    #[error("Architectural violation detected: {0}")]
    Violation(String),

    #
    QueryError(#[from] tree_sitter::QueryError),
    
    // Add other potential errors from the detector's logic
    #[error("I/O error during analysis")]
    Io(#[from] std::io::Error),
}


The #[derive(Error)] macro from thiserror implements the std::error::Error trait. The #[error("...")] attribute generates the Display implementation, and the #[from] attribute generates From implementations, allowing for seamless error conversion using the ? operator.16
Now, the detector function can be refactored. Instead of incorrectly accepting an &ArchitecturalLayer, it should accept the raw &str from the source code annotation and perform the parsing itself. This establishes the function's entry point as the validation boundary.

Rust


// The refactored, robust detector function
pub fn analyze_dependency(
    source_layer_str: &str,
    dependency_layer_str: &str,
    //... other arguments like the tree-sitter node
) -> Result<(), DetectorError> {
    // Validation happens immediately at the boundary.
    // The `?` operator will automatically convert a `ParseError` from strum
    // into a `DetectorError::LayerParseError` via the `#[from]` attribute.
    let source_layer: ArchitecturalLayer = source_layer_str.parse()?;
    let dependency_layer: ArchitecturalLayer = dependency_layer_str.parse()?;

    // The rest of the function now operates on *guaranteed valid* types.
    // The logic is cleaner and safer, with no need for runtime checks on the enum.
    match (source_layer, dependency_layer) {
        (ArchitecturalLayer::Domain, ArchitecturalLayer::Application) => {
            let error_message = "Domain layer cannot depend on Application layer.".to_string();
            Err(DetectorError::Violation(error_message))
        }
        (ArchitecturalLayer::Domain, ArchitecturalLayer::Infrastructure) => {
            let error_message = "Domain layer cannot depend on Infrastructure layer.".to_string();
            Err(DetectorError::Violation(error_message))
        }
        //... other valid and invalid dependency rules
        _ => {
            // This dependency is allowed
            Ok(())
        }
    }
}


This refactored design is vastly superior. It adheres to the "fail fast" principle by validating inputs at the earliest possible moment. It leverages the type system to enforce architectural invariants, making the core logic that follows the parsing step inherently safer and easier to reason about. The use of thiserror and strum ensures the implementation is both robust and idiomatic.

Part III: Synthesis and Advanced Considerations


Section 3.1: Unifying Principles: The Centrality of Rust's Trait System

The solutions to both the data conversion and architectural integrity problems are deeply rooted in a single, unifying concept: the masterful use of Rust's trait system. The standard library traits are not merely a collection of disparate tools; they form a cohesive, expressive vocabulary for managing data views, conversions, ownership, and failure modes. Understanding their distinct roles is a cornerstone of writing idiomatic Rust.
Deref and AsRef: These traits handle cheap, non-failable borrowing and the creation of "views" into data. Deref represents a primary, "dereferenced" view of a type (like String to str), enabling automatic coercion. AsRef is a more general conversion to a reference, which can be ambiguous if a type offers multiple views.5 The
Option<String> problem was solved by preferring the more specific Deref-based as_deref method over the ambiguous AsRef.
From/Into: These traits are for infallible, ownership-transferring conversions. They are used when a conversion is guaranteed to succeed and a new, owned value is desired.
TryFrom/FromStr: These traits are the fallible counterparts to From/Into. They are essential for creating validation boundaries, as seen in the architectural refactoring. By returning a Result, they explicitly model that a conversion from a less-structured type (like a &str) to a more-structured one (like an enum) might fail.13
Error: This trait standardizes how failure states are represented, displayed, and propagated. Combined with crates like thiserror, it allows for the creation of rich, structured error types that integrate seamlessly with Rust's ? operator and overall error handling philosophy.17
By selecting the correct trait for each specific task, a developer communicates intent clearly to both the compiler and to other programmers. This leads to code that is not only correct and performant but also self-documenting and maintainable.

Section 3.2: Proposed Implementation and Integration

To demonstrate how these principles come together in a realistic project, the following code provides a miniature but complete implementation blueprint. It simulates a static analysis tool that reads a file, parses its content with tree-sitter, and runs a detector for architectural violations.
Project Structure:



.
├── Cargo.toml
└── src
    ├── detectors
    │   ├── leaky_abstraction.rs
    │   └── mod.rs
    ├── main.rs
    └── parsing.rs


Cargo.toml:

Ini, TOML


[package]
name = "rust-ast-analyzer"
version = "0.1.0"
edition = "2021"

[dependencies]
strum = { version = "0.26", features = ["derive"] }
strum_macros = "0.26"
thiserror = "1.0"
tree-sitter = "0.22"


src/parsing.rs:

Rust


//! Module for handling source file parsing and data conversion.

/// Represents a parsed source file.
pub struct ParsedFile {
    pub path: String,
    pub content: Option<String>,
}

impl ParsedFile {
    /// Provides the file content as a byte slice for tree-sitter,
    /// returning an empty slice if content is None.
    pub fn source_bytes(&self) -> &[u8] {
        // The recommended, idiomatic solution using `as_deref`.
        self.content.as_deref()
           .map(str::as_bytes)
           .unwrap_or(&)
    }
}


src/detectors/mod.rs:

Rust


//! The main module for all anti-pattern detectors.

pub mod leaky_abstraction;


src/detectors/leaky_abstraction.rs:

Rust


//! Detector for leaky abstractions between architectural layers.

use strum_macros::EnumString;
use thiserror::Error;

#
pub enum DetectorError {
    #[error("Failed to parse architectural layer from string: {0}")]
    LayerParseError(#[from] strum::ParseError),

    #[error("Architectural violation: {0}")]
    Violation(String),
}

#
#[strum(serialize_all = "snake_case", ascii_case_insensitive)]
pub enum ArchitecturalLayer {
    Application,
    Domain,
    Infrastructure,
}

/// Analyzes a dependency relationship between two layers.
pub fn analyze_dependency(
    source_layer_str: &str,
    dependency_layer_str: &str,
) -> Result<(), DetectorError> {
    let source_layer: ArchitecturalLayer = source_layer_str.parse()?;
    let dependency_layer: ArchitecturalLayer = dependency_layer_str.parse()?;

    match (source_layer, dependency_layer) {
        (ArchitecturalLayer::Domain, ArchitecturalLayer::Application) => {
            Err(DetectorError::Violation("Domain layer cannot depend on Application layer.".into()))
        }
        _ => Ok(()),
    }
}


src/main.rs:

Rust


mod detectors;
mod parsing;

use detectors::leaky_abstraction::{self, DetectorError};
use parsing::ParsedFile;

fn main() {
    // --- Scenario 1: Successful analysis ---
    println!("--- Running Scenario 1: Valid Dependency ---");
    let file_content = "some rust code here".to_string();
    let parsed_file = ParsedFile {
        path: "src/domain/service.rs".to_string(),
        content: Some(file_content),
    };

    // 1. Convert Option<String> to &[u8] for tree-sitter
    let source_bytes = parsed_file.source_bytes();
    println!("Successfully converted source to byte slice of length: {}", source_bytes.len());
    // In a real app, you would now pass `source_bytes` to a tree_sitter::Parser.

    // 2. Run the architectural detector with valid inputs
    let result = leaky_abstraction::analyze_dependency("application", "domain");
    handle_analysis_result(result);


    // --- Scenario 2: Architectural Violation ---
    println!("\n--- Running Scenario 2: Architectural Violation ---");
    let result_violation = leaky_abstraction::analyze_dependency("domain", "application");
    handle_analysis_result(result_violation);


    // --- Scenario 3: Invalid Layer String ---
    println!("\n--- Running Scenario 3: Invalid Layer String ---");
    let result_parse_error = leaky_abstraction::analyze_dependency("domain", "presentation");
    handle_analysis_result(result_parse_error);
    
    // --- Scenario 4: No file content ---
    println!("\n--- Running Scenario 4: No File Content ---");
    let empty_file = ParsedFile { path: "empty.rs".to_string(), content: None };
    let empty_bytes = empty_file.source_bytes();
    println!("Byte slice for empty file has length: {}", empty_bytes.len());
    assert!(empty_bytes.is_empty());
}

fn handle_analysis_result(result: Result<(), DetectorError>) {
    match result {
        Ok(()) => println!("Analysis successful: No violations found."),
        Err(e) => eprintln!("Analysis failed: {}", e),
    }
}


This complete blueprint serves as a practical, runnable guide, demonstrating how the recommended patterns for data conversion and architectural design integrate into a cohesive and robust system.

Conclusion: Building Maintainable and Performant Rust Systems

The challenges of interfacing with external libraries and maintaining a sound internal architecture are central to systems programming. This report has provided definitive, idiomatic solutions to two such challenges within a Rust-based static analysis project.
For converting an Option<String> to a &[u8] for tree-sitter integration, the recommended approach is to use Option::as_deref, which leverages Rust's Deref coercion for a concise, zero-cost, and unambiguous conversion. This pattern is superior to more generic approaches that can lead to compiler ambiguity.
For refactoring the leaky_abstraction.rs detector, the solution lies in establishing a firm validation boundary. This is achieved by implementing the FromStr trait for the ArchitecturalLayer enum—preferably automated via the strum crate—and defining a comprehensive error type with thiserror. This transforms the architecture to one where illegal states are unrepresentable in the core logic, a hallmark of robust Rust design.
These solutions are not merely about satisfying the compiler or fixing bugs. They embody core Rust principles: leveraging the type system to enforce invariants, creating explicit and fallible validation boundaries, and using the standard library's trait ecosystem to write expressive, maintainable code. Adopting these practices is fundamental to building complex systems in Rust that are not only performant but also correct and resilient to change over their entire lifecycle.
Works cited
Tree-sitter: Introduction, accessed July 11, 2025, https://tree-sitter.github.io/
tree-sitter - crates.io: Rust Package Registry, accessed July 11, 2025, https://crates.io/crates/tree-sitter
Parser in tree_sitter - Rust - Docs.rs, accessed July 11, 2025, https://docs.rs/tree-sitter/latest/tree_sitter/struct.Parser.html
When to use AsRef
AsRef in std::convert - Rust, accessed July 11, 2025, https://doc.rust-lang.org/std/convert/trait.AsRef.html
The Slice Type - The Rust Programming Language, accessed July 11, 2025, https://doc.rust-lang.org/book/ch04-03-slices.html
Strings - Rust By Example, accessed July 11, 2025, https://rustwiki.org/en/rust-by-example/std/str.html
How to provide a type-annotation for `AsRef`? - Stack Overflow, accessed July 11, 2025, https://stackoverflow.com/questions/29643305/how-to-provide-a-type-annotation-for-asref
Type annotation required when using `as_ref()` in `assert_eq!()` - Stack Overflow, accessed July 11, 2025, https://stackoverflow.com/questions/29278940/type-annotation-required-when-using-as-ref-in-assert-eq
Why type annotations needed? - help - The Rust Programming Language Forum, accessed July 11, 2025, https://users.rust-lang.org/t/why-type-annotations-needed/71243
rust - Converting from Option
How to convert a &str to a &[u8] - Stack Overflow, accessed July 11, 2025, https://stackoverflow.com/questions/31289588/how-to-convert-a-str-to-a-u8
Can I convert a string to enum without macros in Rust? - Stack Overflow, accessed July 11, 2025, https://stackoverflow.com/questions/39070244/can-i-convert-a-string-to-enum-without-macros-in-rust
String to/from enum - help - The Rust Programming Language Forum, accessed July 11, 2025, https://users.rust-lang.org/t/string-to-from-enum/46581
EnumString in strum - Rust - Docs.rs, accessed July 11, 2025, https://docs.rs/strum/latest/strum/derive.EnumString.html
thiserror - Rust - Docs.rs, accessed July 11, 2025, https://docs.rs/thiserror
thiserror, anyhow, or How I Handle Errors in Rust Apps | Shakacode, accessed July 11, 2025, https://www.shakacode.com/blog/thiserror-anyhow-or-how-i-handle-errors-in-rust-apps/
A Comprehensive Guide to robust code with thiserror for Rust | by loudsilence | Rustaceans | Medium, accessed July 11, 2025, https://medium.com/rustaceans/a-comprehensive-guide-to-robust-code-with-thiserror-for-rust-43778b1b3906
Getting Started with the thiserror Crate for Rust | by loudsilence | Rustaceans - Medium, accessed July 11, 2025, https://medium.com/rustaceans/getting-started-with-the-thiserror-crate-for-rust-0ea33415eee0
How to Use the “thiserror” Crate in Rust | by Pandula Weerasooriya | Better Programming, accessed July 11, 2025, https://betterprogramming.pub/a-simple-guide-to-using-thiserror-crate-in-rust-eee6e442409b
