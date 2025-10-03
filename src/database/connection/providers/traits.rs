//! Database Provider Traits
//!
//! This module defines the core traits that database providers must implement
//! to provide a consistent interface across different database backends.

use crate::database::models::{
    AnalysisRun, AnalysisStats, AntiPatternType, ArchitecturalIssue, Dependency,
};
use crate::error::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;

/// Main database provider trait that all backends must implement
#[async_trait]
pub trait DatabaseProvider: Send + Sync {
    /// Initialize the database provider (tables, indexes, etc.)
    async fn initialize(&self) -> Result<()>;

    /// Test database connectivity and health
    async fn test_connection(&self) -> Result<()>;

    /// Execute a query and return results
    async fn execute_query(&self, query: &str, params: &[&str]) -> Result<Vec<QueryResult>>;

    /// Execute a write operation and return affected rows
    async fn execute_write(&self, query: &str, params: &[&str]) -> Result<u64>;

    /// Begin a database transaction
    async fn begin_transaction(&self) -> Result<Box<dyn TransactionProvider>>;

    /// Get a database connection
    async fn get_connection(&self) -> Result<Box<dyn DatabaseConnection>>;

    /// Get or create a project ID for the given path
    async fn get_or_create_project_id(&self, project_path: &Path) -> Result<i64>;

    /// Create a new analysis run
    async fn create_analysis_run(&self, project_path: &Path) -> Result<AnalysisRun>;

    /// Update an existing analysis run
    async fn update_analysis_run(&self, run: &AnalysisRun) -> Result<()>;

    /// Store anti-pattern types in batch
    async fn store_anti_pattern_types_batch(
        &self,
        anti_pattern_types: &mut [AntiPatternType],
    ) -> Result<()>;

    /// Store architectural issues in batch
    async fn store_issues_batch(&self, issues: &[ArchitecturalIssue]) -> Result<()>;

    /// Store dependencies in batch
    async fn store_dependencies_batch(
        &self,
        run_id: i64,
        dependencies: &[Dependency],
    ) -> Result<()>;

    /// Get analysis run by ID
    async fn get_analysis_run(&self, run_id: i64) -> Result<Option<AnalysisRun>>;

    /// Get latest analysis run
    async fn get_latest_analysis_run(&self) -> Result<Option<AnalysisRun>>;

    /// Get recent analysis runs
    async fn get_recent_analysis_runs(&self, limit: u32) -> Result<Vec<AnalysisRun>>;

    /// Get issues for a specific analysis run
    async fn get_issues_for_run(&self, run_id: i64) -> Result<Vec<ArchitecturalIssue>>;

    /// Get dependencies for a specific analysis run
    async fn get_dependencies_for_run(&self, run_id: i64) -> Result<Vec<Dependency>>;

    /// Get issues with their anti-pattern types (prevents N+1 queries)
    async fn get_issues_with_types_for_run(
        &self,
        run_id: i64,
    ) -> Result<Vec<(ArchitecturalIssue, AntiPatternType)>>;

    /// Get analysis statistics efficiently
    async fn get_analysis_stats(&self, run_id: i64) -> Result<AnalysisStats>;

    /// Get paginated issues with filtering
    async fn get_issues_paginated(
        &self,
        run_id: i64,
        offset: u32,
        limit: u32,
        severity_filter: Option<&str>,
        detector_filter: Option<&str>,
    ) -> Result<Vec<ArchitecturalIssue>>;

    /// Get all anti-pattern types
    async fn get_all_anti_pattern_types(&self) -> Result<Vec<AntiPatternType>>;

    /// Get project path by ID
    async fn get_project_path(&self, project_id: i64) -> Result<String>;

    /// Cleanup expired connections and optimize performance
    async fn cleanup(&self) -> Result<u64>;

    /// Get health status of the database
    async fn get_health_status(&self) -> Result<super::DatabaseHealthStatus>;
}

/// Transaction provider trait for handling database transactions
#[async_trait]
pub trait TransactionProvider: Send + Sync {
    /// Execute a query within the transaction
    async fn execute_query(&self, query: &str, params: &[&str]) -> Result<Vec<QueryResult>>;

    /// Execute a write operation within the transaction
    async fn execute_write(&self, query: &str, params: &[&str]) -> Result<u64>;

    /// Commit the transaction
    async fn commit(self: Box<Self>) -> Result<()>;

    /// Rollback the transaction
    async fn rollback(self: Box<Self>) -> Result<()>;
}

/// Connection provider trait for managing database connections
#[async_trait]
pub trait ConnectionProvider: Send + Sync {
    /// Get a connection from the pool
    async fn get_connection(&self) -> Result<Box<dyn DatabaseConnection>>;

    /// Return a connection to the pool
    async fn return_connection(&self, connection: Box<dyn DatabaseConnection>) -> Result<()>;

    /// Get pool statistics
    async fn get_pool_stats(&self) -> Result<PoolStats>;
}

/// Individual database connection trait
#[async_trait]
pub trait DatabaseConnection: Send + Sync {
    /// Execute a query and return results
    async fn execute_query(&self, query: &str, params: &[&str]) -> Result<Vec<QueryResult>>;

    /// Execute a write operation
    async fn execute_write(&self, query: &str, params: &[&str]) -> Result<u64>;

    /// Check if connection is still valid
    async fn is_valid(&self) -> bool;

    /// Get connection age
    fn connection_age(&self) -> std::time::Duration;
}

/// Query result structure for database-agnostic result handling
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<QueryRow>,
}

#[derive(Debug, Clone)]
pub struct QueryRow {
    pub values: HashMap<String, QueryValue>,
}

#[derive(Debug, Clone)]
pub enum QueryValue {
    Integer(i64),
    Float(f64),
    Text(String),
    Blob(Vec<u8>),
    Null,
}

impl QueryValue {
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            QueryValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<String> {
        match self {
            QueryValue::Text(s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            QueryValue::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, QueryValue::Null)
    }
}

/// Connection pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub max_connections: usize,
    pub available_connections: usize,
    pub active_connections: usize,
    pub pending_requests: usize,
    pub total_connections_created: u64,
    pub total_connections_closed: u64,
}
