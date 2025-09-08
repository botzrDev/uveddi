# Detector Development Guide

This guide provides comprehensive instructions for developing new anti-pattern detectors in Uveddi, including documentation standards, testing strategies, and performance guidelines.

## 🏗️ Detector Architecture

### Core Components

Every detector in Uveddi follows a consistent architecture:

```rust
//! [Detector Name] Anti-pattern Detector
//!
//! [Comprehensive documentation following the standard template]

use crate::analysis::{AnalysisDetector, AnalysisError};
use crate::ast::tree_sitter::{ParsedFile, SourceLanguage};
use crate::database::models::{AntiPatternType, ArchitecturalIssue};

/// Configuration for the [detector name] detector
#[derive(Debug, Clone)]
pub struct [DetectorName]Config {
    // Configuration fields with documentation
}

/// [Detector Name] detector implementation
pub struct [DetectorName]Detector {
    config: [DetectorName]Config,
}

impl AnalysisDetector for [DetectorName]Detector {
    // Required trait implementations
}
```

### Required Trait Implementation

All detectors must implement the `AnalysisDetector` trait:

```rust
pub trait AnalysisDetector {
    /// Returns the name of this detector
    fn get_detector_name(&self) -> &'static str;
    
    /// Returns the anti-pattern types this detector can identify
    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType>;
    
    /// Detects issues in a single parsed file
    fn detect_issues(&self, file: &ParsedFile) -> Result<Vec<ArchitecturalIssue>, AnalysisError>;
    
    /// Optional: Detects issues across multiple files using dependency graph
    fn detect_graph_issues(&self, graph: &LocalDependencyGraph, analysis_run_id: i64) -> Vec<ArchitecturalIssue> {
        vec![] // Default implementation
    }
}
```

## 📚 Documentation Standards

### Module-Level Documentation Template

```rust
//! [Detector Name] Anti-pattern Detector
//!
//! ## Overview
//! Brief description of what this detector identifies and why it matters.
//! Include the business impact and technical debt implications.
//!
//! ## Detection Strategy
//! Detailed explanation of the algorithm and approach used:
//! 1. **Step 1**: What happens first
//! 2. **Step 2**: How analysis proceeds
//! 3. **Step 3**: How results are generated
//!
//! ## Supported Languages
//! - **Rust**: Specific patterns detected, Tree-sitter queries used
//! - **Python**: Language-specific considerations
//! - **JavaScript**: Framework-specific patterns
//!
//! ## Configuration
//! Available configuration options and their effects:
//! - `threshold`: Description and recommended values
//! - `ignore_patterns`: How to exclude files/patterns
//! - `severity_levels`: How severity is calculated
//!
//! ## Examples
//! 
//! ### Detected Pattern (Rust)
//! ```rust
//! // This would be flagged as [anti-pattern]
//! struct ProblematicCode {
//!     // Example of detected issue
//! }
//! ```
//!
//! ### Good Pattern (Rust)
//! ```rust
//! // This follows best practices
//! struct WellDesignedCode {
//!     // Example of good design
//! }
//! ```
//!
//! ## Performance Considerations
//! - **Time Complexity**: O(n) where n is...
//! - **Space Complexity**: O(m) where m is...
//! - **Optimization Notes**: Specific performance optimizations used
//!
//! ## Limitations
//! - Known edge cases that may not be detected
//! - Language features that may cause false positives
//! - Architectural patterns that may interfere
//!
//! ## References
//! - [Academic Paper Title](link)
//! - [Industry Standard](link)
//! - [Documentation Source](link)
```

### Function Documentation Standards

```rust
/// Brief description of what this function does
///
/// Longer description explaining the algorithm, approach, or important details.
/// Include information about when this function should be used.
///
/// # Arguments
///
/// * `param1` - Description of the first parameter
/// * `param2` - Description of the second parameter
///
/// # Returns
///
/// Description of what is returned and under what conditions.
///
/// # Errors
///
/// Description of error conditions and what causes them.
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
/// Notes about performance characteristics if relevant.
pub fn analyze_pattern(&self, file: &ParsedFile) -> Result<Vec<Issue>, Error> {
    // Implementation
}
```

## 🧪 Testing Strategy

### Test Structure

Create comprehensive tests in `src/analysis/tests/universal/[detector_name]_detection.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::tree_sitter::{AstParser, SourceLanguage};
    use std::path::PathBuf;

    #[test]
    fn test_basic_detection() {
        // Test basic functionality
    }

    #[test]
    fn test_language_specific_patterns() {
        // Test each supported language
    }

    #[test]
    fn test_configuration_options() {
        // Test different configuration settings
    }

    #[test]
    fn test_edge_cases() {
        // Test boundary conditions and edge cases
    }

    #[test]
    fn test_false_positives() {
        // Ensure good code isn't flagged
    }

    #[test]
    fn test_performance() {
        // Test with large files/complex patterns
    }
}
```

### Required Test Categories

1. **Basic Functionality Tests**
   - Detector correctly identifies target patterns
   - Proper severity assignment
   - Accurate line number reporting

2. **Language-Specific Tests**
   - Test each supported language independently
   - Verify language-specific Tree-sitter queries
   - Test language-specific configuration

3. **Configuration Tests**
   - Test all configuration options
   - Verify threshold behavior
   - Test ignore patterns

4. **Edge Case Tests**
   - Empty files
   - Very large files
   - Malformed code
   - Boundary conditions

5. **Integration Tests**
   - Test with real-world code samples
   - Test with multiple files
   - Test performance characteristics

## 🚀 Performance Guidelines

### Tree-sitter Query Optimization

1. **Use Specific Queries**
   ```rust
   // Good: Specific query
   const FUNCTION_QUERY: &str = r#"
   (function_item
     name: (identifier) @name
     body: (block) @body
   )
   "#;
   
   // Bad: Overly broad query
   const BAD_QUERY: &str = r#"
   (identifier) @name
   "#;
   ```

2. **Minimize Query Complexity**
   - Use multiple simple queries instead of one complex query
   - Cache compiled queries when possible
   - Avoid nested queries when not necessary

3. **Efficient Node Traversal**
   ```rust
   // Good: Direct node access
   let name_node = capture.node;
   let text = name_node.utf8_text(source)?;
   
   // Bad: Unnecessary traversal
   let parent = capture.node.parent().unwrap();
   let children: Vec<_> = parent.children(&mut cursor).collect();
   ```

### Memory Management

1. **Avoid Unnecessary Cloning**
   ```rust
   // Good: Use references
   fn analyze_node(&self, node: &Node, source: &[u8]) -> Result<Issue, Error>
   
   // Bad: Unnecessary cloning
   fn analyze_node(&self, node: Node, source: Vec<u8>) -> Result<Issue, Error>
   ```

2. **Efficient String Handling**
   ```rust
   // Good: Use string slices when possible
   let text = node.utf8_text(source)?;
   
   // Bad: Unnecessary string allocation
   let text = node.utf8_text(source)?.to_string();
   ```

### Caching Strategies

1. **Cache Compiled Queries**
   ```rust
   pub struct MyDetector {
       query_cache: HashMap<SourceLanguage, Query>,
   }
   ```

2. **Cache Analysis Results**
   ```rust
   pub struct MyDetector {
       result_cache: HashMap<String, Vec<Issue>>,
   }
   ```

## 🔧 Integration Guidelines

### Adding to Analysis Engine

1. **Import the Detector**
   ```rust
   // In src/analysis/engine.rs
   use crate::analysis::detectors::anti_patterns::my_detector::MyDetector;
   ```

2. **Add to Detector List**
   ```rust
   detectors: vec![
       // ... existing detectors
       Box::new(MyDetector::with_default_config()),
   ],
   ```

3. **Add Configuration Support**
   ```rust
   // In src/config/mod.rs
   pub struct Config {
       // ... existing fields
       pub my_detector: Option<MyDetectorConfig>,
   }
   ```

### CLI Integration

1. **Add CLI Options**
   ```rust
   // In src/cli/analyze_command.rs
   #[arg(long)]
   pub my_detector_threshold: Option<u32>,
   ```

2. **Update Application Config**
   ```rust
   // In src/application/mod.rs
   pub struct AnalysisConfig {
       // ... existing fields
       pub my_detector_threshold: Option<u32>,
   }
   ```

## 📋 Checklist for New Detectors

### Documentation ✅
- [ ] Comprehensive module-level documentation
- [ ] All public functions documented
- [ ] Configuration options explained
- [ ] Examples provided for each language
- [ ] Performance characteristics documented
- [ ] Limitations clearly stated

### Implementation ✅
- [ ] Implements `AnalysisDetector` trait
- [ ] Supports all target languages
- [ ] Has configurable thresholds
- [ ] Provides meaningful error messages
- [ ] Includes severity scoring
- [ ] Handles edge cases gracefully

### Testing ✅
- [ ] Basic functionality tests
- [ ] Language-specific tests
- [ ] Configuration tests
- [ ] Edge case tests
- [ ] Performance tests
- [ ] Integration tests

### Integration ✅
- [ ] Added to analysis engine
- [ ] CLI options implemented
- [ ] Configuration support added
- [ ] Documentation updated
- [ ] Examples provided

### Performance ✅
- [ ] Tree-sitter queries optimized
- [ ] Memory usage minimized
- [ ] Caching implemented where appropriate
- [ ] Performance benchmarks created
- [ ] Scalability tested

## 🎯 Best Practices

### Code Organization
- Keep detector logic focused and single-purpose
- Separate configuration from implementation
- Use clear, descriptive naming
- Follow Rust idioms and conventions

### Error Handling
- Provide meaningful error messages
- Use appropriate error types
- Handle malformed input gracefully
- Log important events for debugging

### Maintainability
- Write self-documenting code
- Use consistent patterns across detectors
- Minimize dependencies
- Follow the established architecture

### User Experience
- Provide actionable feedback
- Include code snippets in reports
- Offer configuration guidance
- Support common use cases out-of-the-box

This guide ensures that all new detectors maintain high quality standards and integrate seamlessly with the existing Uveddi architecture.