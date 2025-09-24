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

use crate::database::repositories::errors::RepositoryError;
use chrono::{DateTime, Utc};

/// Shared helper functions for SQLite repositories
pub mod helpers {
    use super::*;

    /// Parse RFC3339 datetime string to UTC DateTime
    pub fn parse_rfc3339_datetime(datetime_str: &str) -> Result<DateTime<Utc>, RepositoryError> {
        DateTime::parse_from_rfc3339(datetime_str)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| {
                RepositoryError::validation("datetime", format!("Invalid RFC3339 datetime: {}", e))
            })
    }

    /// Parse optional RFC3339 datetime string to UTC DateTime
    pub fn parse_optional_rfc3339_datetime(
        datetime_str: Option<String>,
    ) -> Result<Option<DateTime<Utc>>, RepositoryError> {
        match datetime_str {
            Some(s) => Ok(Some(parse_rfc3339_datetime(&s)?)),
            None => Ok(None),
        }
    }

    /// Convert Option<i32> to 0 if None (for database storage)
    pub fn option_i32_to_default(value: Option<i32>) -> i32 {
        value.unwrap_or(0)
    }

    /// Convert i32 to Some(i32), or None if 0 (for model creation)
    pub fn i32_to_option(value: i32) -> Option<i32> {
        if value == 0 {
            None
        } else {
            Some(value)
        }
    }
}
