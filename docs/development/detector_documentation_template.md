# Detector Documentation Template

Use this template when creating or improving detector documentation. Copy the Rust code template below and customize it for your specific detector.

## Module-Level Documentation Template

```rust
//! [Detector Name] Anti-pattern Detector
//!
//! ## Overview
//! Brief description of what this detector identifies and why it matters.
//! Include the business impact and technical debt implications.
//!
//! [Detector Name] typically exhibits:
//! - Characteristic 1 (specific behavior)
//! - Characteristic 2 (measurable symptom)
//! - Characteristic 3 (architectural impact)
//!
//! ## Detection Strategy
//! Detailed explanation of the algorithm and approach used:
//! 1. **Step 1**: What happens first (e.g., AST parsing, symbol extraction)
//! 2. **Step 2**: How analysis proceeds (e.g., pattern matching, metric calculation)
//! 3. **Step 3**: How results are generated (e.g., threshold comparison, severity assignment)
//!
//! The detector uses [specific techniques] to ensure [accuracy/performance/completeness].
//!
//! ## Supported Languages
//! - **Rust**: Specific patterns detected, Tree-sitter queries used
//!   - Pattern 1: Description and detection method
//!   - Pattern 2: Language-specific considerations
//!   - Handles: Specific language features (e.g., impl blocks, traits)
//! - **Python**: Language-specific considerations
//!   - Pattern 1: Description and detection method
//!   - Pattern 2: Framework-specific patterns
//!   - Handles: Specific language features (e.g., metaclasses, decorators)
//! - **JavaScript**: Framework-specific patterns
//!   - Pattern 1: Description and detection method
//!   - Pattern 2: Modern JS features
//!   - Handles: Specific language features (e.g., ES6 classes, prototypes)
//!
//! ## Configuration
//! Available configuration options and their effects:
//! - `threshold_name`: Description and recommended values (default: X)
//! - `ignore_patterns`: How to exclude files/patterns (default: ["test", "spec"])
//! - `severity_levels`: How severity is calculated (default: threshold-based)
//!
//! These settings can be customized via CLI arguments, environment variables, or configuration files.
//!
//! ## Examples
//! 
//! ### Detected Pattern (Rust)
//! ```rust
//! // This would be flagged as [anti-pattern name]
//! struct ProblematicCode {
//!     // Example showing the specific issue
//!     field1: Type1,
//!     field2: Type2,
//!     // ... more fields demonstrating the problem
//! }
//!
//! impl ProblematicCode {
//!     // Methods that demonstrate the anti-pattern
//!     fn problematic_method(&self) {
//!         // Code that violates the principle
//!     }
//! }
//! ```
//!
//! ### Good Pattern (Rust)
//! ```rust
//! // This follows best practices
//! struct WellDesignedCode {
//!     // Example showing the correct approach
//!     focused_field: Type1,
//! }
//!
//! impl WellDesignedCode {
//!     // Methods that follow good design principles
//!     fn focused_method(&self) -> Result<Output, Error> {
//!         // Code that follows the principle
//!     }
//! }
//! ```
//!
//! ### Detected Pattern (Python)
//! ```python
//! # This would be flagged as [anti-pattern name]
//! class ProblematicClass:
//!     def __init__(self):
//!         # Example showing the specific issue
//!         pass
//! ```
//!
//! ### Detected Pattern (JavaScript)
//! ```javascript
//! // This would be flagged as [anti-pattern name]
//! class ProblematicClass {
//!     constructor() {
//!         // Example showing the specific issue
//!     }
//! }
//! ```
//!
//! ## Performance Considerations
//! - **Time Complexity**: O(f(n)) where n is [input size description]
//! - **Space Complexity**: O(g(m)) where m is [memory usage description]
//! - **Optimization Notes**: 
//!   - Specific performance optimizations used
//!   - Caching strategies employed
//!   - Parallelization opportunities
//!
//! ## Limitations
//! - **Analysis Scope**: What the detector cannot analyze (e.g., single-file vs cross-file)
//! - **Dynamic Behavior**: Runtime patterns that cannot be detected statically
//! - **Language Specifics**: Language features that may cause false positives/negatives
//! - **Threshold Sensitivity**: How threshold values affect accuracy
//! - **Semantic Understanding**: Limitations in understanding code semantics
//!
//! ## References
//! - [Academic Paper Title](link) - Research foundation
//! - [Industry Standard](link) - Coding standards reference
//! - [Documentation Source](link) - Implementation guidance
//! - [Tool/Framework](link) - Related tools or inspiration
```

## Struct Documentation Template

```rust
/// [Detector Name] detector that identifies [specific anti-pattern]
///
/// This detector implements a [approach description] to identify [pattern description]
/// that [impact description]. It [detection method] and compares them against
/// configurable thresholds.
///
/// # Configuration
///
/// The detector accepts [number] main parameters:
/// - `param1`: Description and purpose
/// - `param2`: Description and purpose
///
/// # Examples
///
/// ```rust
/// use uveddi::analysis::detectors::anti_patterns::[module]::[DetectorName];
///
/// // Create detector with custom configuration
/// let detector = [DetectorName]::new(param1, param2);
///
/// // Create detector with default configuration
/// let detector = [DetectorName]::default();
/// ```
pub struct [DetectorName]Detector {
    /// Description of field1 and its purpose
    field1: Type1,
    /// Description of field2 and its purpose
    field2: Type2,
}
```

## Function Documentation Template

```rust
/// Brief description of what this function does
///
/// Longer description explaining the algorithm, approach, or important details.
/// Include information about when this function should be used and any important
/// side effects or assumptions.
///
/// # Arguments
///
/// * `param1` - Description of the first parameter and its constraints
/// * `param2` - Description of the second parameter and expected format
///
/// # Returns
///
/// Description of what is returned and under what conditions. Include information
/// about the structure of returned data and any guarantees about its content.
///
/// # Errors
///
/// Description of error conditions and what causes them:
/// - `ErrorType1`: When this error occurs and how to handle it
/// - `ErrorType2`: Specific conditions that trigger this error
///
/// # Examples
///
/// ```rust
/// let detector = MyDetector::new();
/// let result = detector.analyze_pattern(&parsed_file)?;
/// assert!(result.len() > 0);
/// ```
///
/// # Performance
///
/// Notes about performance characteristics if relevant:
/// - Time complexity: O(n) where n is...
/// - Space complexity: O(m) where m is...
/// - Caching behavior or optimization notes
pub fn analyze_pattern(&self, file: &ParsedFile) -> Result<Vec<Issue>, Error> {
    // Implementation
}
```

## Configuration Documentation Template

```rust
/// Configuration for the [detector name] detector
///
/// This configuration controls how the detector identifies [anti-pattern name]
/// and determines the sensitivity of detection. All fields are optional and
/// will fall back to sensible defaults if not provided.
///
/// # Examples
///
/// ```rust
/// use uveddi::analysis::detectors::anti_patterns::[module]::[DetectorName]Config;
///
/// // Strict configuration for high-quality codebases
/// let strict_config = [DetectorName]Config {
///     threshold1: 5,
///     threshold2: 3,
///     ignore_patterns: vec!["test".to_string()],
///     ..Default::default()
/// };
///
/// // Lenient configuration for legacy codebases
/// let lenient_config = [DetectorName]Config {
///     threshold1: 20,
///     threshold2: 15,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone)]
pub struct [DetectorName]Config {
    /// Description of threshold1 and its impact on detection
    /// 
    /// Lower values make detection more sensitive, higher values more lenient.
    /// Recommended range: X-Y, default: Z
    pub threshold1: u32,
    
    /// Description of threshold2 and its purpose
    /// 
    /// This threshold controls [specific behavior]. Values should be chosen
    /// based on [criteria]. Default: Z
    pub threshold2: u32,
    
    /// Patterns to ignore during detection
    /// 
    /// Files matching these patterns will be excluded from analysis.
    /// Supports glob patterns. Default: ["test", "spec", "mock"]
    pub ignore_patterns: Vec<String>,
}
```

## Checklist for Documentation Quality

### Module Level ✅
- [ ] Clear overview explaining the anti-pattern and its impact
- [ ] Detailed detection strategy with numbered steps
- [ ] Language-specific behavior documented for each supported language
- [ ] Configuration options explained with defaults and recommendations
- [ ] Code examples showing both problematic and good patterns
- [ ] Performance characteristics documented
- [ ] Limitations clearly stated
- [ ] References to authoritative sources provided

### Struct Level ✅
- [ ] Purpose and approach clearly explained
- [ ] Configuration parameters documented
- [ ] Usage examples provided
- [ ] Field purposes documented

### Function Level ✅
- [ ] Clear description of purpose and behavior
- [ ] All parameters documented with constraints
- [ ] Return values explained
- [ ] Error conditions documented
- [ ] Examples provided
- [ ] Performance notes when relevant

### Configuration ✅
- [ ] All fields documented with purpose and impact
- [ ] Default values and recommended ranges provided
- [ ] Examples showing different use cases
- [ ] Relationship between fields explained

Use this template to ensure consistent, comprehensive documentation across all detectors in the Uveddi codebase.