use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
