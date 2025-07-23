//! GraphQL context for sharing state across resolvers

use crate::database::DatabaseManager;
use crate::analysis::engine::AnalysisEngine;
use std::sync::Arc;

/// GraphQL context that provides access to shared resources
#[derive(Clone)]
pub struct GraphQLContext {
    pub database: Arc<DatabaseManager>,
    pub analysis_engine: Arc<AnalysisEngine>,
}

impl GraphQLContext {
    pub fn new(database: DatabaseManager, analysis_engine: AnalysisEngine) -> Self {
        Self {
            database: Arc::new(database),
            analysis_engine: Arc::new(analysis_engine),
        }
    }

    /// Get a reference to the database manager
    pub fn db(&self) -> &DatabaseManager {
        &self.database
    }

    /// Get a reference to the analysis engine  
    pub fn engine(&self) -> &AnalysisEngine {
        &self.analysis_engine
    }
}