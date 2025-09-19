//! Detection of data structure leakage patterns.

use crate::analysis::AnalysisError;
use crate::ast::tree_sitter_impl::ParsedFile;
use crate::database::models::ArchitecturalIssue;
use crate::analysis::detectors::anti_patterns::leaky_abstraction::types::{
    AnalysisContext, LeakType
};

/// Detects patterns where internal data structures are inappropriately exposed.
pub struct DataStructureLeaksPattern;

impl DataStructureLeaksPattern {
    /// Creates a new data structure leaks pattern detector.
    pub fn new() -> Self {
        Self
    }

    /// Detects data structure leak patterns in a parsed file.
    pub fn detect_patterns(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        match parsed_file.language {
            crate::ast::SourceLanguage::Rust => {
                issues.extend(self.detect_rust_data_structure_leaks(parsed_file, context)?);
            }
            crate::ast::SourceLanguage::Python => {
                issues.extend(self.detect_python_data_structure_leaks(parsed_file, context)?);
            }
            crate::ast::SourceLanguage::JavaScript | crate::ast::SourceLanguage::TypeScript => {
                issues.extend(self.detect_js_data_structure_leaks(parsed_file, context)?);
            }
        }

        Ok(issues)
    }

    /// Detects Rust-specific data structure leak patterns.
    fn detect_rust_data_structure_leaks(
        &self,
        _parsed_file: &ParsedFile,
        _context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Detect Vec<T> in public APIs instead of &[T]
        // Detect HashMap<K,V> in public APIs instead of traits
        // Detect concrete collection types in interfaces
        // Implementation would go here

        Ok(issues)
    }

    /// Detects Python-specific data structure leak patterns.
    fn detect_python_data_structure_leaks(
        &self,
        _parsed_file: &ParsedFile,
        _context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Detect list/dict returns instead of iterators/protocols
        // Detect direct access to collection internals
        // Detect mutable collection exposure
        // Implementation would go here

        Ok(issues)
    }

    /// Detects JavaScript/TypeScript-specific data structure leak patterns.
    fn detect_js_data_structure_leaks(
        &self,
        _parsed_file: &ParsedFile,
        _context: &AnalysisContext,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Detect Array instead of interface/protocol
        // Detect Object instead of specific types
        // Detect mutable array/object exposure
        // Implementation would go here

        Ok(issues)
    }

    /// Checks if a type represents a leaky data structure.
    pub fn is_leaky_data_structure(&self, type_name: &str) -> bool {
        let leaky_types = [
            "Vec<", "HashMap<", "HashSet<", "BTreeMap<", "BTreeSet<",  // Rust collections
            "list", "dict", "set",                                      // Python collections
            "Array<", "Object", "Map<", "Set<",                        // JS/TS collections
        ];

        leaky_types.iter().any(|pattern| type_name.contains(pattern))
    }

    /// Checks if a return type exposes internal collection structure.
    pub fn is_collection_exposing_return_type(&self, return_type: &str) -> bool {
        self.is_leaky_data_structure(return_type) && !self.is_safe_collection_interface(return_type)
    }

    /// Checks if a collection type uses a safe interface.
    fn is_safe_collection_interface(&self, type_name: &str) -> bool {
        let safe_interfaces = [
            "&[", "Iterator<", "IntoIterator<",  // Rust safe interfaces
            "Iterable", "Iterator", "Generator",  // Python safe interfaces
            "ReadonlyArray<", "Readonly<",       // TypeScript safe interfaces
        ];

        safe_interfaces.iter().any(|pattern| type_name.contains(pattern))
    }

    /// Analyzes method signatures for data structure leaks.
    pub fn analyze_method_signature(&self, signature: &str) -> Vec<String> {
        let mut issues = Vec::new();

        if self.is_collection_exposing_return_type(signature) {
            issues.push(format!("Method returns concrete collection type: {}", signature));
        }

        if signature.contains("&mut Vec<") || signature.contains("&mut HashMap<") {
            issues.push("Method parameter exposes mutable collection reference".to_string());
        }

        issues
    }

    /// Analyzes field declarations for data structure leaks.
    pub fn analyze_field_declaration(&self, field_decl: &str, visibility: &str) -> Option<String> {
        if visibility == "pub" && self.is_leaky_data_structure(field_decl) {
            Some(format!("Public field exposes concrete data structure: {}", field_decl))
        } else {
            None
        }
    }

    /// Checks if a type alias creates a data structure leak.
    pub fn is_leaky_type_alias(&self, alias_definition: &str) -> bool {
        alias_definition.contains("type ") &&
        alias_definition.contains("pub ") &&
        self.is_leaky_data_structure(alias_definition)
    }

    /// Analyzes generic type parameters for potential leaks.
    pub fn analyze_generic_constraints(&self, generic_def: &str) -> Vec<String> {
        let mut issues = Vec::new();

        if generic_def.contains(": Vec<") || generic_def.contains(": HashMap<") {
            issues.push("Generic constraint exposes concrete collection type".to_string());
        }

        issues
    }

    /// Suggests alternative patterns for exposed data structures.
    pub fn suggest_alternative(&self, exposed_type: &str) -> Option<String> {
        match exposed_type {
            t if t.contains("Vec<") => Some("Consider using &[T] or Iterator<Item=T>".to_string()),
            t if t.contains("HashMap<") => Some("Consider using a trait or &dyn Iterator".to_string()),
            t if t.contains("list") => Some("Consider using an iterator or protocol".to_string()),
            t if t.contains("Array<") => Some("Consider using ReadonlyArray<T> or Iterable<T>".to_string()),
            _ => None,
        }
    }

    /// Helper function to create an architectural issue.
    fn create_issue(
        &self,
        context: &AnalysisContext,
        description: &str,
        line_number: u32,
        severity: &str,
    ) -> ArchitecturalIssue {
        let mut issue = ArchitecturalIssue::new(
            context.analysis_run_id,
            self.get_anti_pattern_id_for_leak_type(&LeakType::ImplementationExposure),
            context.file_path.clone(),
            Some(line_number as i32),
            description.to_string(),
            "DataStructureLeaksPattern".to_string(),
            severity.to_string(),
            description.to_string(),
        );
        issue.start_line = Some(line_number as i32);
        issue.end_line = Some(line_number as i32);
        issue
    }

    /// Maps a `LeakType` to its corresponding `anti_pattern_type_id`.
    fn get_anti_pattern_id_for_leak_type(&self, leak_type: &LeakType) -> i64 {
        match leak_type {
            LeakType::VisibilityViolation => 1,
            LeakType::LayerViolation => 2,
            LeakType::ImplementationExposure => 3,
            LeakType::FrameworkCoupling => 4,
            LeakType::ErrorPropagation => 5,
            LeakType::PerformanceLeak => 6,
        }
    }
}

impl Default for DataStructureLeaksPattern {
    fn default() -> Self {
        Self::new()
    }
}