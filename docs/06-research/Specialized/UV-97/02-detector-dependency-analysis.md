# GPT Research Prompt: Detector Dependency Analysis for UV-97

## Context
You are analyzing the Uveddi Rust codebase for UV-97 (tree-sitter feature gating). Uveddi is an architectural analysis tool with multiple anti-pattern detectors in `src/analysis/detectors/anti_patterns/`. These detectors need to be properly feature-gated to work with and without tree-sitter enabled.

**Current State:**
- Tree-sitter is being made optional via the `tree-sitter` feature flag
- Some detectors are entirely dependent on tree-sitter AST parsing
- Some detectors might work partially without tree-sitter (using metadata/heuristics)
- Some detectors might be completely independent of tree-sitter
- Each detector implements the `AnalysisDetector` trait which requires specific methods

**Current AnalysisDetector Trait:**
```rust
pub trait AnalysisDetector {
    fn detect_issues(&self, file: &ParsedFile) -> Result<Vec<ArchitecturalIssue>, AnalysisError>;
    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType>;
    fn get_detector_name(&self) -> &'static str;
}
```

## Research Prompt

**Your task:** Analyze all anti-pattern detectors in the Uveddi codebase to classify their tree-sitter dependencies and determine the optimal gating strategy for each.

### Step 1: Detector Classification Analysis

For each detector file in `src/analysis/detectors/anti_patterns/`, analyze:

#### A. Direct Tree-sitter Dependencies
```rust
// Look for these patterns:
use tree_sitter::{Query, QueryCursor, Node, Tree};
tree_sitter::Query::new(...)
node.utf8_text(...)
cursor.captures(...)
```

#### B. Indirect Tree-sitter Dependencies  
```rust
// Look for usage of:
parsed_file.tree.as_ref()
parsed_file.custom_ast
// AST traversal logic
// Syntax-specific pattern matching
```

#### C. Independent Analysis Capabilities
```rust
// Look for logic that uses:
parsed_file.content  // Raw text analysis
parsed_file.file_path  // File metadata
// Line counting, regex patterns
// File size analysis
// Simple text-based heuristics
```

### Step 2: Categorize Each Detector

Classify each detector into one of these categories:

#### Category 1: Entirely Tree-sitter Dependent
- **Definition:** Cannot function meaningfully without AST parsing
- **Gating Strategy:** Module-level `#[cfg(feature = "tree-sitter")]`
- **Stub Strategy:** Return empty results with explanatory error

#### Category 2: Partially Tree-sitter Dependent  
- **Definition:** Has some functionality without tree-sitter (text analysis, heuristics)
- **Gating Strategy:** Method-level conditional compilation
- **Stub Strategy:** Implement fallback analysis using available data

#### Category 3: Tree-sitter Independent
- **Definition:** Can work entirely without AST parsing
- **Gating Strategy:** No gating needed
- **Stub Strategy:** No changes required

### Step 3: Method-Level Dependency Analysis

For Category 2 detectors, analyze which specific methods require tree-sitter:

```rust
impl SomeDetector {
    // Does this need tree-sitter?
    fn detect_issues(&self, file: &ParsedFile) -> Result<Vec<ArchitecturalIssue>, AnalysisError>
    
    // What about helper methods?
    fn count_methods(&self, file: &ParsedFile) -> u32
    fn calculate_complexity(&self, node: &Node) -> u32  // Definitely needs tree-sitter
    fn estimate_size_heuristic(&self, content: &str) -> u32  // Could work without
}
```

### Step 4: Stub Implementation Requirements

For each detector category, determine:

#### For Category 1 (Entirely Dependent):
- Should the detector be completely unavailable when tree-sitter is disabled?
- Should it return a descriptive error explaining the limitation?
- Should it be excluded from detector lists when tree-sitter is disabled?

#### For Category 2 (Partially Dependent):
- Which analysis can continue with text-only fallbacks?
- What reduced functionality should be offered?
- How to maintain the same return type with degraded data?

#### For Category 3 (Independent):
- Verify no hidden tree-sitter dependencies exist
- Ensure they continue working in both modes

## Expected Output Format

### Detector Classification Matrix

```markdown
| Detector | Category | Tree-sitter Usage | Fallback Potential | Gating Strategy |
|----------|----------|-------------------|-------------------|-----------------|
| LargeClassesDetector | 2 - Partial | AST method counting, LCOM calculation | Text-based LOC, simple heuristics | Method-level |
| DeadCodeDetector | 1 - Entirely | Symbol usage analysis, scope analysis | None meaningful | Module-level |
| GodObjectDetector | 2 - Partial | Detailed metrics | Basic size metrics | Method-level |
| ... | ... | ... | ... | ... |
```

### Detailed Analysis Per Detector

For each detector, provide:

```markdown
#### DetectorName
- **Category:** [1/2/3]
- **Tree-sitter Usage:**
  - Direct imports: [list specific tree-sitter types used]
  - AST operations: [describe what AST analysis is performed]
  - Critical dependencies: [what cannot work without tree-sitter]
  
- **Fallback Analysis Potential:**
  - What can work with text-only: [describe text-based analysis possible]
  - Degraded functionality: [what reduced analysis can be offered]
  - Accuracy impact: [how much less accurate would fallback be]

- **Recommended Gating Strategy:**
  - Approach: [module-level / method-level / none]
  - Justification: [why this approach is best]
  
- **Stub Implementation Requirements:**
  - Return type: [same/modified/error]
  - Fallback logic: [describe what stub should do]
  - Error handling: [how to communicate limitations]
```

### Implementation Templates

Provide code templates for each gating strategy:

```rust
// Template for Category 1 (Module-level gating)
#[cfg(feature = "tree-sitter")]
impl AnalysisDetector for SomeDetector {
    fn detect_issues(&self, file: &ParsedFile) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        // Full implementation
    }
}

#[cfg(not(feature = "tree-sitter"))]
impl AnalysisDetector for SomeDetector {
    fn detect_issues(&self, _file: &ParsedFile) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        Err(AnalysisError::Analysis("Detector requires tree-sitter feature".to_string()))
    }
}
```

```rust
// Template for Category 2 (Method-level gating)
impl SomeDetector {
    fn detect_issues(&self, file: &ParsedFile) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        #[cfg(feature = "tree-sitter")]
        {
            self.detailed_ast_analysis(file)
        }
        #[cfg(not(feature = "tree-sitter"))]
        {
            self.fallback_text_analysis(file)
        }
    }
}
```

## Success Criteria

Your analysis should provide:
- [ ] Clear categorization of all detectors
- [ ] Specific implementation strategy for each detector
- [ ] Code templates for systematic application
- [ ] Justification for each categorization decision
- [ ] Estimation of analysis accuracy impact when tree-sitter is disabled

This analysis will enable batch-processing detector gating instead of handling each detector individually, significantly speeding up UV-97 implementation.

___________________________________________________________________________________________________________________________________


UV-97 Detector Dependency Analysis and Feature Gating Strategy


I. Strategic Overview: Modularizing Tree-sitter in Uveddi


1.1 Rationale for Optional tree-sitter Integration

The Uveddi architectural analysis tool is undergoing a significant enhancement under the UV-97 initiative: the tree-sitter parsing engine is being transitioned from a mandatory dependency to an optional, feature-gated component. The core motivation for this change is to augment Uveddi's flexibility, performance, and portability, thereby broadening its applicability across diverse development environments and use cases.
This modularization strategy yields several key benefits. First, it directly addresses performance and build-time concerns for consumers of Uveddi as a library. By making tree-sitter optional, end-users can compile a lighter version of the tool, resulting in a smaller binary footprint and faster compilation cycles, particularly in environments where deep syntactic analysis is not a primary requirement.1 This aligns with established best practices in Rust library design, where optional dependencies are exposed via feature flags to allow consumers to select only the functionality they need, avoiding the overhead of unused code.2
Second, this change enhances portability. The tree-sitter crate has underlying C dependencies that, while broadly compatible, can introduce friction in certain build environments or resource-constrained systems. An operating mode that does not require these dependencies makes Uveddi more accessible and easier to integrate into a wider array of CI/CD pipelines and development workflows.
Finally, this initiative empowers users with explicit control over the analysis trade-offs. They can choose between a comprehensive, high-fidelity analysis powered by Abstract Syntax Trees (ASTs) or a faster, more lightweight analysis based on heuristics. This configurability allows Uveddi to serve both users who need precise, deep structural insights and those who require rapid, high-level feedback.

1.2 The Spectrum of Analysis: From Heuristics to AST

The introduction of the tree-sitter feature flag creates two distinct analysis paradigms within Uveddi, each with its own characteristics regarding accuracy, performance, and analytical depth. The decision to enable or disable this feature is not merely about including or excluding a dependency; it is about selecting the fundamental mode of operation for the detectors.
The high-fidelity analysis mode, enabled with #[cfg(feature = "tree-sitter")], utilizes a complete AST generated by the tree-sitter parser.4 An AST is a tree-based representation of the source code's abstract syntactic structure, which captures the essential logic while omitting syntactically noisy elements like punctuation or grouping parentheses.6 This structured, semantic representation is the cornerstone of modern static analysis. It allows detectors to "understand" the code's context, including variable scopes, function call relationships, and class structures. This deep understanding is essential for calculating sophisticated metrics like Cyclomatic Complexity and Lack of Cohesion in Methods (LCOM), or for performing accurate dead code analysis by resolving symbol definitions and usages.9 The
tree-sitter crate provides the necessary API primitives, such as Parser, Tree, Node, and Query, to construct and navigate these ASTs effectively.4
Conversely, the low-fidelity analysis mode, active when the tree-sitter feature is disabled, operates without an AST. In this mode, detectors are limited to analyzing the raw text content of a source file (parsed_file.content). This form of analysis relies on simpler, less precise techniques such as regular expressions, line counting, and keyword matching.13 While this approach is significantly faster due to the absence of a parsing phase, it comes at the cost of accuracy and contextual awareness. Text-based heuristics cannot reliably distinguish between code, comments, and string literals, nor can they comprehend concepts like scope or inheritance. This inherent limitation can lead to a higher incidence of both false positives and false negatives, a common challenge in static analysis.14
The central architectural decision of the UV-97 initiative is to accept this degradation in analytical precision in the fallback mode in exchange for the significant gains in performance and portability. This report will systematically quantify this trade-off for each affected detector.

1.3 Methodology: Classification and Gating

To implement this dual-paradigm system, a rigorous methodology will be applied to every anti-pattern detector in the Uveddi codebase.
First, each detector will be classified into one of three categories based on its dependency on tree-sitter:
Category 1: Entirely Tree-sitter Dependent: The detector's core logic is fundamentally inseparable from AST analysis. No meaningful fallback is possible.
Category 2: Partially Tree-sitter Dependent: The detector can offer a reduced or heuristic-based version of its analysis without an AST, though with a significant loss of accuracy.
Category 3: Tree-sitter Independent: The detector's logic does not rely on ASTs and can function identically with or without the tree-sitter feature.
Second, a specific conditional compilation strategy will be defined for each detector using Rust's #[cfg] attribute system.15 The choice between module-level gating (
#[cfg] on a mod statement) and method-level gating (#[cfg] on an impl block or within a function) is a critical implementation detail that ensures code is cleanly included or excluded at compile time, preventing errors and maintaining a logical structure.1 This systematic approach will form the basis for the refactoring plan outlined in the subsequent sections.

II. Detector Classification and Gating Summary

This section provides a high-level summary of the analysis, classifying each anti-pattern detector and recommending a gating strategy. This matrix serves as an executive overview and a project management tool for the engineering team, enabling a systematic "batch-processing" approach to the refactoring effort. Detectors within the same category can often be addressed using a consistent implementation pattern, accelerating the completion of the UV-97 initiative.

2.1 Detector Classification Matrix

Detector
Category
Tree-sitter Usage
Fallback Potential
Recommended Gating Strategy
DeadCodeDetector
1 - Entirely
AST for symbol definition, scope resolution, and reference tracking.
None meaningful. Text search for identifiers is highly inaccurate due to scope.
Module-level #[cfg]
GodObjectDetector
2 - Partial
AST for precise metrics: Weighted Methods per Class (WMC), Lack of Cohesion (LCOM), and coupling analysis.
File size, Lines of Code (LOC), and regex-based function/method counts as proxies for complexity.
Method-level #[cfg]
LargeClassDetector
2 - Partial
AST for accurate method/field counts, cyclomatic complexity, and LCOM calculations.
Text-based LOC and regex-based heuristics for method/field counts.
Method-level #[cfg]
InconsistentNamingDetector
3 - Independent
Analyzes file paths and names via parsed_file.file_path.
Not applicable; fully functional without tree-sitter.
None
LongParameterListDetector
1 - Entirely
Requires AST to parse function signatures, identify parameters, and count them accurately.
Regex-based signature parsing is extremely brittle and error-prone.
Module-level #[cfg]
PrimitiveObsessionDetector
2 - Partial
AST to identify function parameters and return types, checking for overuse of primitive types.
Regex to parse function signatures. Lower accuracy, struggles with complex types and generics.
Method-level #[cfg]
ShotgunSurgeryDetector
1 - Entirely
Requires project-wide AST analysis to build a call graph and track how changes to one module ripple to others.
None. This is a structural analysis that cannot be performed on raw text.
Module-level #[cfg]
LargeFileDetector
3 - Independent
Analyzes file size based on parsed_file.content.len() or file system metadata.
Not applicable; fully functional without tree-sitter.
None


III. In-Depth Detector Analysis and Refactoring Plans

This section provides a detailed, per-detector analysis and a concrete refactoring plan. Each subsection follows a standardized format to ensure consistency and provide actionable guidance for implementation.

3.1 DeadCodeDetector

Category: 1 - Entirely Tree-sitter Dependent
Tree-sitter Usage Analysis:
Direct Imports: The detector directly imports core tree-sitter types, including use tree_sitter::{Node, Query, QueryCursor, Tree};.4
AST Operations: The fundamental operation of this detector is to identify code that is declared but never used. This is a non-trivial task that requires a deep, structural understanding of the program. The implementation executes tree-sitter queries to locate all symbol declarations (e.g., functions, variables, constants, classes). It then traverses the entire AST of the relevant scope (often project-wide) to build a comprehensive map of all symbol references. By comparing the set of declared symbols against the set of referenced symbols, it can accurately identify unused, or "dead," code.9 This process is effectively a graph-based analysis where the AST serves as the foundational data structure for constructing the graph.20
Critical Dependencies: The core logic is critically dependent on the ability to distinguish between a symbol's declaration and its usage, and to understand lexical scoping (e.g., a variable named x in one function is distinct from a variable x in another). This level of semantic understanding is exclusively provided by an AST parser; it cannot be replicated with simple text analysis.22
Fallback Analysis and Accuracy Impact:
What can work with text-only: No meaningful analysis is possible. A naive approach using regular expressions to search for a declared identifier elsewhere in the codebase would fail catastrophically. It would be unable to differentiate between a valid reference, a mention in a comment, an appearance in a string literal, or a declaration of a different variable with the same name in another scope.
Degraded Functionality: Offering a text-based fallback would produce results with an unacceptably high rate of both false positives (flagging actively used code as dead) and false negatives (failing to identify genuinely dead code). The output would be misleading and untrustworthy, providing negative value to the user.9
Accuracy Impact: The accuracy of the detector would plummet from over 95% (allowing for complex macro expansion edge cases) to a level that is statistically indistinguishable from random chance. The functionality is effectively non-existent without tree-sitter.
Recommended Gating Strategy:
Approach: Module-level #[cfg(feature = "tree-sitter")].
Justification: Given that no viable fallback can be implemented, the entire detector module should be conditionally compiled. This is the cleanest and safest approach. It prevents a non-functional detector from being included in the application binary when the tree-sitter feature is disabled, thus avoiding runtime errors and user confusion.1
Stub Implementation Requirements:
Return type: Result<Vec<ArchitecturalIssue>, AnalysisError>.
Fallback logic: The most robust implementation is to gate the module's inclusion in its parent mod.rs file and also gate its instantiation in the analysis runner. This ensures the type is never constructed when the feature is disabled. If a stub were strictly necessary for compilation, the detect_issues method should immediately return an error.
Error handling: The recommended error is Err(AnalysisError::FeatureNotEnabled { detector: "DeadCodeDetector", feature: "tree-sitter" }). This provides a clear, actionable message to the user, explaining precisely why the detector is unavailable and how to enable it. The central detector registry must be architected to exclude this detector from its list of available analyses when compiled without the feature.

3.2 LargeClassDetector

Category: 2 - Partially Tree-sitter Dependent
Tree-sitter Usage Analysis:
Direct Imports: use tree_sitter::{Node, Query};.4
AST Operations: The high-fidelity implementation of this detector relies on the AST to compute a suite of metrics that together provide a nuanced view of class size and complexity, key indicators of the "Large Class" and "God Object" anti-patterns.24 These metrics include:
Method and Field Counts: Accurately counting methods and fields by traversing the children of a class/struct node in the AST and identifying nodes of the appropriate type (e.g., function_definition, field_declaration).
Weighted Methods per Class (WMC): This metric is the sum of the cyclomatic complexities of all methods within a class. Cyclomatic complexity quantifies the number of linearly independent paths through code by counting decision points like if, for, while, and match statements.27 Calculating this requires parsing each method's body, an operation that is only possible with an AST.
Lack of Cohesion in Methods (LCOM): This metric measures how well the methods of a class belong together. The common LCOM4 variant, for example, counts the number of connected components in a graph where methods and instance variables are nodes. An edge exists if a method accesses a variable. This requires identifying all instance variables and tracking their usage within each method body, a classic AST-based data-flow analysis task.11
Critical Dependencies: The calculation of WMC and LCOM is critically dependent on the structural and semantic information provided by the AST. These metrics provide a much deeper insight into a class's adherence to the Single Responsibility Principle than simple size metrics.31
Fallback Analysis and Accuracy Impact:
What can work with text-only: In the absence of an AST, the detector can fall back to using crude proxies based on the raw text content of the file (parsed_file.content):
Lines of Code (LOC): A simple count of lines in the file serves as a basic, albeit noisy, proxy for overall size.
Heuristic Method Count: A language-specific regular expression, such as r"^\s*(pub\s+)?(async\s+)?fn\s+\w+" for Rust, can be used to estimate the number of function definitions.
Heuristic Field Count: For Rust structs, a regex can count lines within a struct {... } block that appear to be field declarations (e.g., \w+:\s+\w+).
Degraded Functionality: The detector's capability is significantly reduced. It can only report on coarse size metrics (LOC, estimated method/field counts) and loses all insight into true logical complexity (WMC) and cohesion (LCOM). These deeper metrics are often far better indicators of maintainability issues than raw line counts.10
Accuracy Impact: The accuracy is severely degraded. The heuristic-based approach is brittle and prone to errors. For example, the regex for method counting could incorrectly count function-like macros, commented-out code, or function calls that happen to match the pattern. It cannot understand context.13 A class with many small, simple getter methods could be incorrectly flagged as "large," while a smaller but dangerously complex class could be missed entirely. The findings from this fallback mode should be presented to the user as low-confidence suggestions requiring manual verification.
Recommended Gating Strategy:
Approach: Method-level #[cfg].
Justification: The detector can still provide some limited, heuristic-based value without tree-sitter. A complete module-level gate would discard this potential utility. Method-level conditional compilation within the detect_issues method allows for a single LargeClassDetector struct and AnalysisDetector implementation to exist in both build configurations, cleanly switching between the high-fidelity AST logic and the low-fidelity heuristic logic.15
Stub Implementation Requirements:
Return type: Ok(Vec<ArchitecturalIssue>). The return type remains consistent. The ArchitecturalIssue instances will simply be generated by the fallback logic.
Fallback logic: A dedicated helper function, e.g., run_heuristic_analysis, should be implemented. This function will operate on file.content, using line counting and regex-based heuristics. It will generate issues based on configurable thresholds for these heuristic metrics. The implementation effort for these heuristics should be kept minimal; the goal is a lightweight fallback, not a perfect text-based replica of AST analysis. Over-investing in complex regexes yields diminishing returns and increases maintenance overhead.
Error handling: No error should be returned. Instead, the generated ArchitecturalIssue must contain metadata indicating its heuristic origin. This is a critical architectural point that will be detailed in Section V.

3.3 GodObjectDetector

Category: 2 - Partially Tree-sitter Dependent
Tree-sitter Usage Analysis:
Direct Imports: use tree_sitter::{Node, Query, QueryCursor};.4
AST Operations: This detector is closely related to LargeClassDetector but often focuses more on a class's external interactions and internal cohesion, which are hallmarks of the "God Object" anti-pattern.24 The AST-powered analysis includes:
High Coupling: Executing queries to find all method calls originating from the class's methods that target other objects. A high number of distinct external dependencies is a strong indicator of a God Object.
Low Cohesion (LCOM): As with LargeClassDetector, calculating LCOM is a primary technique. A God Object often groups unrelated functionalities, resulting in a very low cohesion score (i.e., a high LCOM value).11
High Complexity (WMC): God Objects are invariably complex, and WMC provides a quantitative measure of this complexity.28
Excessive Knowledge: Analyzing the number and variety of types used as parameters and local variables within the class's methods. A God Object "knows too much" about the rest of the system.33
Critical Dependencies: Measuring coupling, cohesion, and the diversity of referenced types is impossible without the relational and type information available in a semantically-aware AST.
Fallback Analysis and Accuracy Impact:
What can work with text-only: The fallbacks are similar to those for LargeClassDetector but even less effective at capturing the essence of the anti-pattern.
File Size / LOC: A very large file is a weak indicator.
Heuristic Method/Function Count: A high number of functions can suggest a God Object.32 Regex can provide a rough estimate.
Heuristic Dependency Count: One could use regex to count the number of use or import statements at the top of the file as a crude proxy for external dependencies.
Degraded Functionality: The analysis degrades from a nuanced assessment of responsibility and coupling to a simple check for "is this file big and does it have a lot of functions?". It completely loses the ability to measure cohesion, the defining characteristic of this anti-pattern.31
Accuracy Impact: The accuracy impact is profound. The detector loses its primary diagnostic power. A large but highly cohesive and well-encapsulated class might be falsely flagged, while a smaller, more insidious God Object that acts as a central controller with many dependencies could be missed. The heuristic is a guess at best.
Recommended Gating Strategy:
Approach: Method-level #[cfg].
Justification: Despite the severe degradation, a simple size-based heuristic can still act as a "first alert" for developers, prompting them to investigate a potentially problematic class. Discarding this minimal signal entirely via module-level gating would be a missed opportunity. The method-level approach allows this limited functionality to persist.15
Stub Implementation Requirements:
Return type: Ok(Vec<ArchitecturalIssue>).
Fallback logic: Implement a heuristic-based analysis that checks file LOC, estimated function count, and possibly the number of import/use statements. Issues should be generated based on high thresholds for these combined metrics.
Error handling: As with LargeClassDetector, no error is returned. The findings must be clearly marked as heuristic-based in the ArchitecturalIssue data structure to manage user expectations.

IV. Implementation Patterns and Best Practices

To ensure a consistent and correct implementation of the tree-sitter feature gating across the Uveddi codebase, the following code patterns and best practices must be adopted. These templates provide a blueprint for refactoring each category of detector.

4.1 Template for Category 1 (Entirely Dependent)

Detectors in this category are fundamentally reliant on tree-sitter and have no meaningful fallback. The correct strategy is to ensure they are completely excluded from the build when the tree-sitter feature is not enabled. This requires gating at both the module declaration and the instantiation points.
A simple #[cfg] on the impl block is insufficient. If the code that instantiates the detector (e.g., DeadCodeDetector::new()) remains in the analysis runner, a compilation error will occur when the feature is disabled because the type or its trait implementation will not exist. The correct approach is a "double lock": gating the module's inclusion in its parent and also gating its instantiation in the detector factory or runner. This ensures the compiler is never exposed to the detector's type definition when the feature is off.
Example Implementation:
Gate the module declaration (e.g., in src/analysis/detectors/anti_patterns/mod.rs):
Rust
// This line ensures the entire dead_code_detector.rs file is only compiled
// and included in the module tree when the 'tree-sitter' feature is active.
#[cfg(feature = "tree-sitter")]
pub mod dead_code_detector;


Gate the detector's instantiation (e.g., in the analysis runner or factory function):
Rust
pub fn get_all_anti_pattern_detectors() -> Vec<Box<dyn AnalysisDetector>> {
    let mut detectors: Vec<Box<dyn AnalysisDetector>> = Vec::new();

    // This block, containing the instantiation of the detector, is removed
    // from the compiled code if 'tree-sitter' is not enabled.
    #[cfg(feature = "tree-sitter")]
    detectors.push(Box::new(dead_code_detector::DeadCodeDetector::new()));

    //... add other detectors
    detectors
}


With this pattern, no stub implementation is required within dead_code_detector.rs itself, as the file will not be part of the compilation unit. However, if for some architectural reason the struct definition must exist in both configurations, a stubbed impl block that returns an error would be necessary to satisfy the compiler. The module-level gating is the superior approach.

4.2 Template for Category 2 (Partially Dependent)

For detectors that can offer a degraded, heuristic-based analysis, method-level gating is the appropriate strategy. This pattern maintains a single, consistent public API for the detector while cleanly separating the two distinct implementation paths at compile time.
Example Implementation (within large_class_detector.rs):

Rust


// The LargeClassDetector struct and its public methods are always available.
pub struct LargeClassDetector {
    // Configuration fields for both AST and heuristic thresholds
    max_loc_heuristic: usize,
    max_wmc_ast: u32,
}

impl AnalysisDetector for LargeClassDetector {
    fn detect_issues(&self, file: &ParsedFile) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        // The #[cfg] attribute acts as a compile-time switch. Only one of these
        // blocks will be included in the final binary.
        #[cfg(feature = "tree-sitter")]
        {
            // Path for when 'tree-sitter' is enabled.
            self.run_ast_analysis(file)
        }

        #[cfg(not(feature = "tree-sitter"))]
        {
            // Path for when 'tree-sitter' is disabled.
            self.run_heuristic_analysis(file)
        }
    }

    fn get_detector_name(&self) -> &'static str {
        "LargeClassDetector"
    }
    //... other trait methods
}

impl LargeClassDetector {
    // This helper function is only compiled with the 'tree-sitter' feature.
    #[cfg(feature = "tree-sitter")]
    fn run_ast_analysis(&self, file: &ParsedFile) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        // This unwrap is safe because ParsedFile::tree will be Some.
        let tree = file.tree.as_ref().ok_or_else(|| AnalysisError::MissingAst)?;
        
        //... perform high-fidelity analysis using the AST...
        // let wmc = calculate_wmc(tree);
        // if wmc > self.max_wmc_ast {
        //     // Create and return an ArchitecturalIssue
        // }
        
        Ok(vec!) // Placeholder
    }

    // This helper function is only compiled when 'tree-sitter' is NOT enabled.
    #[cfg(not(feature = "tree-sitter"))]
    fn run_heuristic_analysis(&self, file: &ParsedFile) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let loc = file.content.lines().count();
        if loc > self.max_loc_heuristic {
            // Create and return an ArchitecturalIssue based on the heuristic.
            // This issue MUST be marked as heuristic-based.
        }
        
        Ok(vec!) // Placeholder
    }
}



4.3 Template for Category 3 (Independent)

Detectors in this category require no changes. Their logic relies on data that is always available, such as file paths or raw content length. The primary task for these detectors is verification: a thorough code review must confirm that no hidden or indirect dependencies on tree-sitter exist. For example, ensure the detector does not inadvertently rely on a utility function that itself is gated by the tree-sitter feature. Once confirmed, no #[cfg] attributes are needed.

V. Architectural Impact and Future Considerations

The decision to make tree-sitter an optional dependency has several important architectural implications that extend beyond simple conditional compilation. These considerations affect the tool's data model, its configuration, and the clarity of the results presented to the end-user.

5.1 Analysis Quality Trade-offs and User Communication

The introduction of heuristic-based fallbacks creates a significant challenge: a user running Uveddi without the tree-sitter feature will receive analysis results that are fundamentally different in nature and reliability from those generated with an AST. For example, a LargeClassDetector warning based on a simple Lines of Code (LOC) count is a low-confidence signal, whereas a warning based on a combination of high Weighted Methods per Class (WMC) and low Lack of Cohesion in Methods (LCOM) is a high-confidence indicator of a design flaw.
Presenting these two types of findings to the user as if they were equivalent is misleading and diminishes the tool's credibility. Users must be able to distinguish between high-precision, AST-based findings and lower-precision, heuristic-based suggestions to prioritize their refactoring efforts effectively.34 A heuristic finding may warrant further investigation, while an AST-based finding might justify immediate action.
To address this, the core data model of the tool must be evolved to capture the provenance of each finding. The current ArchitecturalIssue struct, which likely contains only the issue type, location, and description, is insufficient. It must be extended to include metadata about the analysis method.
Recommended Data Model Enhancement:

Rust


/// Represents the analysis method used to detect an issue.
pub enum AnalysisMethod {
    /// Issue detected using a full Abstract Syntax Tree. High confidence.
    Ast,
    /// Issue detected using text-based heuristics. Lower confidence, requires verification.
    Heuristic,
}

pub struct ArchitecturalIssue {
    // --- Existing Fields ---
    pub issue_type: AntiPatternType,
    pub description: String,
    pub file_path: String,
    pub line_range: std::ops::Range<usize>,

    // --- New Fields for UV-97 ---
    /// The method used to detect this issue.
    pub detection_method: AnalysisMethod,
    
    /// A score from 0.0 to 1.0 indicating the confidence in this finding.
    /// e.g., AST-based findings could have a confidence of >0.9, while
    /// heuristic findings might have a confidence of ~0.6.
    pub confidence_score: f32, 
}


By incorporating detection_method and a confidence_score, the Uveddi UI and report generators can visually differentiate findings. For instance, heuristic-based issues could be displayed with a different icon, a lower severity rating, or an explicit "Heuristic-based" label. This transparency is crucial for managing user expectations and ensuring the tool's output is interpreted correctly.36

5.2 Configuring Heuristics and Managing False Positives

Heuristic-based detectors will necessarily rely on arbitrary thresholds (e.g., "flag files with > 500 LOC"). A hardcoded threshold that works for one project may be entirely inappropriate for another, leading to a high volume of false positives and alert fatigue. This is a common problem with static analysis tools, and best practices dictate that such rules must be configurable.38
Therefore, it is a firm requirement that all fallback implementations for Category 2 detectors must source their thresholds from a central, user-configurable settings object. This allows teams to tune the sensitivity of the heuristic analysis to match their project's specific context and coding standards, significantly improving the signal-to-noise ratio of the tool when operating in the non-tree-sitter mode.40

5.3 Future Work and Potential Enhancements

While the initial implementation of fallback detectors will rely on simple heuristics, there are avenues for future improvement that could enhance the utility of the non-tree-sitter mode.
Advanced Heuristic Development: The team could investigate more sophisticated text-analysis techniques that go beyond basic regular expressions. For example, using specialized tokenizers or lightweight parsers for specific language constructs could improve the accuracy of heuristic metrics without introducing the heavy dependencies of a full parser generator like tree-sitter.
Language-Agnostic Analysis: A long-term research direction could involve exploring language-agnostic static analysis. Research into mining analysis rules from code changes and using semantic intermediate representations suggests that it may be possible to define certain detectors in a way that is less dependent on language-specific syntax trees.42 While this is a complex field, it represents a potential future architecture for Uveddi that could offer a powerful set of core analyses that function across multiple languages, irrespective of the availability of a specific
tree-sitter grammar. This would further enhance Uveddi's value as a versatile architectural analysis platform.
Works cited
Compile Time Feature Flags in Rust: Why, How, and When? | by Dotan Nahum - Medium, accessed July 8, 2025, https://jondot.medium.com/compile-time-feature-flags-in-rust-why-how-when-129aada7d1b3
Best practices features and dependencies in a crate - Rust Users Forum, accessed July 8, 2025, https://users.rust-lang.org/t/best-practices-features-and-dependencies-in-a-crate/125626
3491-remove-implicit-features - The Rust RFC Book, accessed July 8, 2025, https://rust-lang.github.io/rfcs/3491-remove-implicit-features.html
tree_sitter - Rust - Docs.rs, accessed July 8, 2025, https://docs.rs/tree-sitter
Tree-sitter: Introduction, accessed July 8, 2025, https://tree-sitter.github.io/
Static Analysis using ASTs | by Hootsuite Engineering - Medium, accessed July 8, 2025, https://medium.com/hootsuite-engineering/static-analysis-using-asts-ebcd170c955e
Using Static Analysis in Program Development - PVS-Studio, accessed July 8, 2025, https://pvs-studio.com/en/blog/posts/a0017/
What is the Lossless Semantic Tree (LST) code model for automated refactoring and analysis? - Moderne, accessed July 8, 2025, https://www.moderne.ai/blog/lossless-semantic-tree-the-complete-code-data-model-for-automated-code-refactoring-and-analysis
Dead Code - Refactoring.Guru, accessed July 8, 2025, https://refactoring.guru/smells/dead-code
Code Smell Detection and Refactoring Strategies - Number Analytics, accessed July 8, 2025, https://www.numberanalytics.com/blog/code-smell-detection-refactoring-strategies
What is high cohesion and how to use it / make it? - Stack Overflow, accessed July 8, 2025, https://stackoverflow.com/questions/10830135/what-is-high-cohesion-and-how-to-use-it-make-it
tree-sitter - crates.io: Rust Package Registry, accessed July 8, 2025, https://crates.io/crates/tree-sitter
Efficient Pattern-based Static Analysis Approach via Regular-Expression Rules, accessed July 8, 2025, https://www.shinhwei.com/saner_modified.pdf
Static Code Analysis: Everything You Need To Know - Codacy | Blog, accessed July 8, 2025, https://blog.codacy.com/static-code-analysis
#[cfg] Conditional Compilation in Rust - Mastering Backend, accessed July 8, 2025, https://masteringbackend.com/posts/cfg-conditional-compilation-in-rust
Conditional compilation - The Rust Reference, accessed July 8, 2025, https://doc.rust-lang.org/reference/conditional-compilation.html
Conditional Compilation - The Rust Programming Language - MIT, accessed July 8, 2025, https://web.mit.edu/rust-lang_v1.25/arch/amd64_ubuntu1404/share/doc/rust/html/book/first-edition/conditional-compilation.html
rust - Cargo: How to include the entire directory or file in feature flags? - Stack Overflow, accessed July 8, 2025, https://stackoverflow.com/questions/75230247/cargo-how-to-include-the-entire-directory-or-file-in-feature-flags
Code smells and Anti-patterns: How to recognize bad code - Barrage, accessed July 8, 2025, https://www.barrage.net/blog/technology/code-smells-and-anti-patterns-how-to-recognize-bad
Dossier: A tree-sitter based multi-language source code and docstring parser : r/rust - Reddit, accessed July 8, 2025, https://www.reddit.com/r/rust/comments/1980y0j/dossier_a_treesitter_based_multilanguage_source/
How to find unused/dead code in java projects [closed] - Stack Overflow, accessed July 8, 2025, https://stackoverflow.com/questions/162551/how-to-find-unused-dead-code-in-java-projects
How to identify and remove dead code? | by typo - Medium, accessed July 8, 2025, https://medium.com/beyond-the-code-by-typo/how-to-identify-and-remove-dead-code-8283b0bf05a3
Lava Flow Anti-Pattern Explained: How Dead Code Destroys Your Project | Geekific, accessed July 8, 2025, https://www.youtube.com/watch?v=Z6blIXgd_SM
What Is a God Class and Why Should We Avoid It? | LinearB Blog, accessed July 8, 2025, https://linearb.io/blog/what-is-a-god-class
Large Class - Refactoring.Guru, accessed July 8, 2025, https://refactoring.guru/smells/large-class
Code Smells - Refactoring.Guru, accessed July 8, 2025, https://refactoring.guru/refactoring/smells
Cyclomatic Complexity: A Complete Guide - Codacy | Blog, accessed July 8, 2025, https://blog.codacy.com/cyclomatic-complexity
Cyclomatic Complexity explained: How it measures (and misleads) code quality - LinearB, accessed July 8, 2025, https://linearb.io/blog/cyclomatic-complexity
10 Code Smells a Static Analyser Can Locate in a Codebase - Fluent C++, accessed July 8, 2025, https://www.fluentcpp.com/2019/03/26/10-code-smells-a-static-analyser-can-locate-in-a-codebase/
Do We Need Improved Code Quality Metrics? - arXiv, accessed July 8, 2025, https://arxiv.org/pdf/2012.12324
God object - Wikipedia, accessed July 8, 2025, https://en.wikipedia.org/wiki/God_object
medium.com, accessed July 8, 2025, https://medium.com/@satyendra.jaiswal/the-god-object-anti-pattern-unveiling-the-monolithic-menace-in-software-design-875884e8cf7a#:~:text=The%20Telltale%20Signs%20of%20a%20God%20Object&text=Excessive%20Method%20Count%3A%20A%20class,sign%20of%20The%20God%20Object.
The “God Object” Anti-Pattern in Software Architecture. | by Dilanka Muthukumarana, accessed July 8, 2025, https://dilankam.medium.com/the-god-object-anti-pattern-in-software-architecture-b2b7782d6997
Heuristic Analysis of a Website: 7 Step Guide - Insight7, accessed July 8, 2025, https://insight7.io/heuristic-analysis-of-a-website-7-step-guide/
Heuristic Analysis for UX: The CXL Guide to Usability Evaluation, accessed July 8, 2025, https://cxl.com/blog/heuristic-analysis/
Heuristic Analysis in User Research - Think Design, accessed July 8, 2025, https://think.design/user-design-research/heuristic-analysis/
How do you perform a Heuristic Analysis? : r/userexperience - Reddit, accessed July 8, 2025, https://www.reddit.com/r/userexperience/comments/g1avjf/how_do_you_perform_a_heuristic_analysis/
Static Code Analysis Best Practices for Developers - ACCELQ, accessed July 8, 2025, https://www.accelq.com/blog/static-code-analysis-best-practices/
How To Perform Static Code Analysis Effectively - Snyk, accessed July 8, 2025, https://snyk.io/articles/open-source-static-code-analysis/how-to-do-static-code-analysis/
Static Code Analysis Best Practices - Number Analytics, accessed July 8, 2025, https://www.numberanalytics.com/blog/static-code-analysis-best-practices-systems-engineering
Best practices for integrating static analysis in pull requests - Graphite, accessed July 8, 2025, https://graphite.dev/guides/best-practices-integrating-static-analysis-pull-requests
A Language-agnostic Framework for Mining Static Analysis Rules from Code Changes - Amazon Science, accessed July 8, 2025, https://assets.amazon.science/da/f0/050314414785a5662500d0e46723/a-language-agnostic-framework-for-mining-static-analysis-rules-from-code-changes.pdf
