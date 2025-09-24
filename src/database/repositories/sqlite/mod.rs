//! SQLite repository implementations
//!
//! This module provides concrete SQLite implementations for all repository traits.

pub mod analysis_repository;
pub mod cache_repository;
pub mod debt_repository;
pub mod dependency_repository;
pub mod event_repository;
pub mod issue_repository;
pub mod metrics_repository;
pub mod project_repository;
pub mod security_repository;

pub use analysis_repository::SqliteAnalysisRepository;
pub use cache_repository::SqliteCacheRepository;
pub use debt_repository::SqliteDebtRepository;
pub use dependency_repository::SqliteDependencyRepository;
pub use event_repository::SqliteEventRepository;
pub use issue_repository::SqliteIssueRepository;
pub use metrics_repository::SqliteMetricsRepository;
pub use project_repository::SqliteProjectRepository;
pub use security_repository::SqliteSecurityRepository;
