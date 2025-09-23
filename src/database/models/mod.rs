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

pub mod analysis;
pub mod anti_pattern;
pub mod architectural_issue;
pub mod cache;
pub mod dependency;
pub mod lifecycle;
pub mod performance;
pub mod project;

pub use analysis::{AnalysisRun, AnalysisStats};
pub use anti_pattern::AntiPatternType;
pub use architectural_issue::ArchitecturalIssue;
pub use cache::{CacheEntry, CacheEntryRecord, CacheMetadata};
pub use dependency::{Dependency, DependencyType};
pub use lifecycle::{LifecycleEvent, LifecycleEventType};
pub use performance::{ComponentPerformanceMetrics, PerformanceMetricsConfig, BenchmarkResult, BenchmarkConfig};
pub use project::{Project, ProjectRecord, ProjectConfig};
