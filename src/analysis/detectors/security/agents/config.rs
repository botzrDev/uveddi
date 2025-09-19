//! Configuration types for the multi-agent security analysis system

use serde::{Deserialize, Serialize};

/// Configuration for individual security agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub timeout_seconds: u64,
    pub max_retries: u32,
}

impl AgentConfig {
    pub fn new(name: String) -> Self {
        Self {
            name,
            timeout_seconds: 30,
            max_retries: 3,
        }
    }
}

/// Configuration for the multi-agent security analysis system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiAgentConfig {
    pub enable_taint_agent: bool,
    pub enable_config_agent: bool,
    pub enable_dependency_agent: bool,
    pub enable_validation_agent: bool,
    pub enable_ai_agent: bool,
    pub max_concurrent_tasks: usize,
    pub analysis_timeout_seconds: u64,
}

impl Default for MultiAgentConfig {
    fn default() -> Self {
        Self::development()
    }
}

impl MultiAgentConfig {
    pub fn development() -> Self {
        Self {
            enable_taint_agent: true,
            enable_config_agent: true,
            enable_dependency_agent: false,
            enable_validation_agent: false,
            enable_ai_agent: false,
            max_concurrent_tasks: 4,
            analysis_timeout_seconds: 300,
        }
    }

    pub fn production() -> Self {
        Self {
            enable_taint_agent: true,
            enable_config_agent: true,
            enable_dependency_agent: true,
            enable_validation_agent: true,
            enable_ai_agent: true,
            max_concurrent_tasks: 8,
            analysis_timeout_seconds: 600,
        }
    }
}