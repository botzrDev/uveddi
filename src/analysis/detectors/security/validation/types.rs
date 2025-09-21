use crate::analysis::detectors::security::types::SecurityIssue;
use crate::analysis::AnalysisError;
use async_trait::async_trait;
use tracing::debug;

#[async_trait]
pub trait SecurityIssueFilter: Send + Sync {
    async fn apply(&self, issues: Vec<SecurityIssue>) -> Result<Vec<SecurityIssue>, AnalysisError>;
}

pub type DynSecurityIssueFilter = Box<dyn SecurityIssueFilter>;

pub struct ValidationStage {
    name: &'static str,
    filters: Vec<DynSecurityIssueFilter>,
}

impl ValidationStage {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            filters: Vec::new(),
        }
    }

    pub fn with_filters(name: &'static str, filters: Vec<DynSecurityIssueFilter>) -> Self {
        Self { name, filters }
    }

    pub fn push(&mut self, filter: DynSecurityIssueFilter) {
        self.filters.push(filter);
    }

    pub fn is_empty(&self) -> bool {
        self.filters.is_empty()
    }

    pub fn len(&self) -> usize {
        self.filters.len()
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub async fn run(
        &self,
        mut issues: Vec<SecurityIssue>,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        if self.filters.is_empty() {
            return Ok(issues);
        }

        debug!("Running validation stage: {}", self.name);
        for filter in &self.filters {
            issues = filter.apply(issues).await?;
        }
        debug!("Stage {} completed with {} issues", self.name, issues.len());
        Ok(issues)
    }
}
