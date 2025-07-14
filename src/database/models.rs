//! Database Models for Uveddi
//!
//! This module defines the data structures that represent entities in the Uveddi database.
//! These models correspond to the database schema and are used for serialization,
//! deserialization, and data transfer between application layers.
//!
//! # Entity Relationships
//!
//! The models follow this relationship hierarchy:
//! - **Project** → **AnalysisRun** → **ArchitecturalIssue**
//! - **AntiPatternType** ← **ArchitecturalIssue**
//! - **Dependency** (standalone, linked by analysis run)
//!
//! # Serialization
//!
//! All models implement `Serialize` and `Deserialize` for JSON export/import
//! and API compatibility. This enables easy integration with reporting systems
//! and external tools.
//!
//! # Usage
//!
//! ```rust
//! use uveddi::database::models::{AnalysisRun, ArchitecturalIssue};
//! use chrono::Utc;
//!
//! // Create a new analysis run
//! let run = AnalysisRun {
//!     run_id: None,
//!     project_id: 1,
//!     start_time: Utc::now(),
//!     end_time: None,
//!     status: "running".to_string(),
//!     total_files_analyzed: None,
//!     total_issues_found: None,
//!     analysis_config: "{}".to_string(),
//! };
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Core analysis run tracking entity
///
/// Represents a single execution of the analysis engine on a project.
/// This is the primary entity that groups all analysis results and
/// tracks execution metadata.
///
/// # Database Mapping
///
/// Maps to the `analysis_runs` table in the database schema.
///
/// # Status Values
///
/// - `"running"`: Analysis is currently in progress
/// - `"completed"`: Analysis finished successfully
/// - `"failed"`: Analysis encountered an error and stopped
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalysisRun {
    /// Unique identifier for this analysis run (auto-generated)
    pub run_id: Option<i64>,
    /// Foreign key reference to the project being analyzed
    pub project_id: i64,
    /// Timestamp when the analysis started
    pub start_time: DateTime<Utc>,
    /// Timestamp when the analysis completed (None if still running)
    pub end_time: Option<DateTime<Utc>>,
    /// Current status of the analysis run
    pub status: String,
    /// Total number of source files processed
    pub total_files_analyzed: Option<i32>,
    /// Total number of issues detected across all files
    pub total_issues_found: Option<i32>,
    /// JSON-serialized configuration used for this analysis
    pub analysis_config: String,
}

/// Architectural issue detected during analysis
///
/// Represents a specific code quality issue, anti-pattern, or architectural
/// problem identified by the analysis engine. Each issue is associated with
/// a particular analysis run and anti-pattern type.
///
/// # Database Mapping
///
/// Maps to the `architectural_issues` table in the database schema.
///
/// # Severity Levels
///
/// - `"low"`: Minor issues that don't significantly impact maintainability
/// - `"medium"`: Moderate issues that should be addressed over time  
/// - `"high"`: Important issues that impact code quality
/// - `"critical"`: Severe issues that require immediate attention
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArchitecturalIssue {
    /// Unique identifier for this issue (auto-generated)
    pub issue_id: Option<i64>,
    /// Foreign key reference to the analysis run that found this issue
    pub analysis_run_id: i64,
    /// Foreign key reference to the type of anti-pattern detected
    pub anti_pattern_type_id: i64,
    /// Path to the file where this issue was found
    pub file_path: String,
    /// Starting line number of the problematic code (optional)
    pub start_line: Option<i32>,
    /// Ending line number of the problematic code (optional)
    pub end_line: Option<i32>,
    /// Severity level of this issue
    pub severity: String,
    /// Human-readable description of the issue
    pub description: String,
    /// Optional code snippet showing the problematic code
    pub code_snippet: Option<String>,
    /// Optional AI-generated explanation and remediation advice
    pub ai_explanation: Option<String>,
}

/// Dependency relationship between code modules
///
/// Represents a dependency edge in the codebase dependency graph.
/// Dependencies track how different parts of the code reference each other,
/// enabling architectural analysis and cycle detection.
///
/// # Usage in Analysis
///
/// Dependencies are used for:
/// - Cycle detection in module relationships
/// - Architectural pattern recognition
/// - Impact analysis for changes
/// - Complexity measurement
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Dependency {
    /// Path to the source file that has the dependency
    pub from_file: PathBuf,
    /// Name or path of the target module being depended upon
    pub to_module: String,
    /// Type of dependency relationship
    pub dependency_type: DependencyType,
    /// Line number where the dependency is declared (optional)
    pub line_number: Option<u32>,
}

/// Types of dependency relationships
///
/// Categorizes different ways that code can depend on other code,
/// enabling more sophisticated analysis of architectural patterns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq)]
pub enum DependencyType {
    /// Rust `use` statement or similar language-specific import
    Use,
    /// Rust `mod` declaration for module inclusion
    Mod,
    /// External crate/package dependency
    External,
    /// Generic import statement (Python, JavaScript, etc.)
    Import,
    /// Data flow dependency (e.g., variable assignments, data transformations)
    DataFlow, // UV-2: Added for semantic dependency classification
    /// Control flow dependency (e.g., function calls, conditional execution)
    ControlFlow, // UV-2: Added for semantic dependency classification
}

/// Anti-pattern type definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiPatternType {
    pub anti_pattern_type_id: Option<i64>,
    pub name: String,
    pub description: String,
    pub category: String, // "structural", "behavioral", "creational"
}

// UV-2: Lifecycle event tracking for components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleEvent {
    pub event_id: Option<i64>,
    pub component_id: String,
    pub event_type: LifecycleEventType,
    pub timestamp: DateTime<Utc>,
    pub details: Option<String>,
    pub run_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LifecycleEventType {
    Created,
    Modified,
    Used,
    Deleted,
}

// UV-2: Component-level performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentPerformanceMetrics {
    pub metric_id: Option<i64>,
    pub component_id: String,
    pub analysis_run_id: i64,
    pub execution_time_ms: u64,
    pub memory_usage_bytes: u64,
    pub ast_parse_time_ms: Option<u64>,
    pub symbol_resolution_time_ms: Option<u64>,
    pub dependency_extraction_time_ms: Option<u64>,
    pub timestamp: DateTime<Utc>,
}

// UV-2: Performance metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetricsConfig {
    pub enabled: bool,
    pub sampling_rate: f64, // 1.0 = every component, 0.1 = every 10th
    pub adaptive_sampling: bool,
    pub memory_sampling_interval_ms: u64,
    pub async_storage: bool,
    pub buffer_size: usize,
    pub flush_interval_seconds: u64,
}

impl Default for PerformanceMetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sampling_rate: 1.0,
            adaptive_sampling: true,
            memory_sampling_interval_ms: 100,
            async_storage: true,
            buffer_size: 100,
            flush_interval_seconds: 30,
        }
    }
}
