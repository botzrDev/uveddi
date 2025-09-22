//! Core types for taint analysis

use crate::ast::SourceLanguage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Represents a taint source in the program
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaintSource {
    /// Unique identifier for this source
    pub id: String,
    /// Source pattern (e.g., "std::env::args", "request.body")
    pub pattern: String,
    /// Programming language this source applies to
    pub language: Option<SourceLanguage>,
    /// Description of what this source represents
    pub description: String,
    /// Severity level of vulnerabilities from this source
    pub default_severity: crate::analysis::detectors::security::types::SecuritySeverity,
    /// Source location if found in code
    pub location: Option<SourceLocation>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl TaintSource {
    pub fn new(id: String, pattern: String, description: String) -> Self {
        Self {
            id,
            pattern,
            language: None,
            description,
            default_severity: crate::analysis::detectors::security::types::SecuritySeverity::Medium,
            location: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_language(mut self, language: SourceLanguage) -> Self {
        self.language = Some(language);
        self
    }

    pub fn with_severity(
        mut self,
        severity: crate::analysis::detectors::security::types::SecuritySeverity,
    ) -> Self {
        self.default_severity = severity;
        self
    }

    pub fn with_location(mut self, location: SourceLocation) -> Self {
        self.location = Some(location);
        self
    }
}

/// Represents a taint sink (dangerous operation)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaintSink {
    /// Unique identifier for this sink
    pub id: String,
    /// Sink pattern (e.g., "sqlx::query", "os.system")
    pub pattern: String,
    /// Programming language this sink applies to
    pub language: Option<SourceLanguage>,
    /// Type of vulnerability this sink can cause
    pub vulnerability_type: crate::analysis::detectors::security::types::SecurityIssueType,
    /// Description of the vulnerability
    pub description: String,
    /// Severity of vulnerabilities reaching this sink
    pub severity: crate::analysis::detectors::security::types::SecuritySeverity,
    /// Source location if found in code
    pub location: Option<SourceLocation>,
    /// Which parameters are vulnerable (0-indexed, None means all)
    pub vulnerable_parameters: Option<Vec<usize>>,
}

impl TaintSink {
    pub fn new(
        id: String,
        pattern: String,
        vulnerability_type: crate::analysis::detectors::security::types::SecurityIssueType,
        description: String,
    ) -> Self {
        let severity = vulnerability_type.default_severity();
        Self {
            id,
            pattern,
            language: None,
            vulnerability_type,
            description,
            severity,
            location: None,
            vulnerable_parameters: None,
        }
    }

    pub fn with_language(mut self, language: SourceLanguage) -> Self {
        self.language = Some(language);
        self
    }

    pub fn with_location(mut self, location: SourceLocation) -> Self {
        self.location = Some(location);
        self
    }

    pub fn with_vulnerable_params(mut self, params: Vec<usize>) -> Self {
        self.vulnerable_parameters = Some(params);
        self
    }
}

/// Represents a sanitization point that cleans tainted data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SanitizationPoint {
    /// Unique identifier for this sanitizer
    pub id: String,
    /// Sanitizer pattern (e.g., "html_escape", "parameterized_query")
    pub pattern: String,
    /// Programming language this sanitizer applies to
    pub language: Option<SourceLanguage>,
    /// Types of vulnerabilities this sanitizer prevents
    pub prevents: Vec<crate::analysis::detectors::security::types::SecurityIssueType>,
    /// Description of what this sanitizer does
    pub description: String,
    /// Source location if found in code
    pub location: Option<SourceLocation>,
    /// Effectiveness score (0.0 - 1.0)
    pub effectiveness: f64,
}

impl SanitizationPoint {
    pub fn new(
        id: String,
        pattern: String,
        prevents: Vec<crate::analysis::detectors::security::types::SecurityIssueType>,
    ) -> Self {
        Self {
            id,
            pattern,
            language: None,
            prevents,
            description: String::new(),
            location: None,
            effectiveness: 1.0,
        }
    }

    pub fn with_language(mut self, language: SourceLanguage) -> Self {
        self.language = Some(language);
        self
    }

    pub fn with_effectiveness(mut self, effectiveness: f64) -> Self {
        self.effectiveness = effectiveness.clamp(0.0, 1.0);
        self
    }
}

/// Source location in code
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file_path: PathBuf,
    pub line: u32,
    pub column: u32,
    pub length: u32,
}

/// Taint level enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaintLevel {
    Untainted,    // No taint
    Low,          // Low taint level
    Medium,       // Medium taint level
    High,         // High taint level
    Tainted,      // Fully tainted (legacy)
    Clean,        // No taint (legacy)
    Sanitized,    // Was tainted but sanitized
    Partial(f64), // Partially tainted (0.0 - 1.0)
}

impl TaintLevel {
    pub fn is_dangerous(&self) -> bool {
        match self {
            TaintLevel::High | TaintLevel::Tainted => true,
            TaintLevel::Medium => true,
            TaintLevel::Partial(level) => *level > 0.5,
            TaintLevel::Untainted | TaintLevel::Clean | TaintLevel::Low | TaintLevel::Sanitized => {
                false
            }
        }
    }

    pub fn score(&self) -> f64 {
        match self {
            TaintLevel::Untainted | TaintLevel::Clean => 0.0,
            TaintLevel::Sanitized => 0.1, // Small residual risk
            TaintLevel::Low => 0.3,
            TaintLevel::Medium => 0.6,
            TaintLevel::High => 0.9,
            TaintLevel::Partial(level) => *level,
            TaintLevel::Tainted => 1.0,
        }
    }

    pub fn combine(&self, other: &TaintLevel) -> TaintLevel {
        match (self, other) {
            (TaintLevel::Untainted | TaintLevel::Clean, other) => other.clone(),
            (other, TaintLevel::Untainted | TaintLevel::Clean) => other.clone(),
            (TaintLevel::High | TaintLevel::Tainted, _)
            | (_, TaintLevel::High | TaintLevel::Tainted) => TaintLevel::High,
            (TaintLevel::Medium, TaintLevel::Low) | (TaintLevel::Low, TaintLevel::Medium) => {
                TaintLevel::Medium
            }
            (TaintLevel::Medium, _) | (_, TaintLevel::Medium) => TaintLevel::Medium,
            (TaintLevel::Low, TaintLevel::Low) => TaintLevel::Low,
            (TaintLevel::Partial(a), TaintLevel::Partial(b)) => {
                TaintLevel::Partial((*a + *b).min(1.0))
            }
            (TaintLevel::Partial(level), TaintLevel::Sanitized)
            | (TaintLevel::Sanitized, TaintLevel::Partial(level)) => {
                TaintLevel::Partial(*level * 0.5) // Sanitization reduces risk
            }
            (TaintLevel::Sanitized, TaintLevel::Sanitized) => TaintLevel::Sanitized,
            // Handle combinations with discrete levels
            (TaintLevel::Low, TaintLevel::Sanitized) | (TaintLevel::Sanitized, TaintLevel::Low) => {
                TaintLevel::Low
            }
            (TaintLevel::Medium, TaintLevel::Sanitized)
            | (TaintLevel::Sanitized, TaintLevel::Medium) => TaintLevel::Medium,
            (TaintLevel::High, TaintLevel::Sanitized)
            | (TaintLevel::Sanitized, TaintLevel::High) => TaintLevel::High,
            // Handle remaining combinations with Partial
            (TaintLevel::Low, TaintLevel::Partial(level))
            | (TaintLevel::Partial(level), TaintLevel::Low) => {
                TaintLevel::Partial((*level + 0.3).min(1.0))
            }
            (TaintLevel::Medium, TaintLevel::Partial(level))
            | (TaintLevel::Partial(level), TaintLevel::Medium) => {
                TaintLevel::Partial((*level + 0.6).min(1.0))
            }
            (TaintLevel::High, TaintLevel::Partial(level))
            | (TaintLevel::Partial(level), TaintLevel::High) => {
                TaintLevel::Partial((*level + 0.9).min(1.0))
            }
            (TaintLevel::Tainted, TaintLevel::Partial(_))
            | (TaintLevel::Partial(_), TaintLevel::Tainted) => TaintLevel::Tainted,
        }
    }
}

/// Language-specific taint patterns
#[derive(Debug, Clone)]
pub struct LanguageTaintPatterns {
    pub source_patterns: Vec<String>,
    pub sink_patterns: Vec<String>,
    pub sanitizer_patterns: Vec<String>,
}

impl LanguageTaintPatterns {
    pub fn new() -> Self {
        Self {
            source_patterns: Vec::new(),
            sink_patterns: Vec::new(),
            sanitizer_patterns: Vec::new(),
        }
    }
}

impl Default for LanguageTaintPatterns {
    fn default() -> Self {
        Self::new()
    }
}
