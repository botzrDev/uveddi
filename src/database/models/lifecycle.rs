use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
