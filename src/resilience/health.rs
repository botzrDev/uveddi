//! Service health monitoring and alerting system (UV-173)
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};

/// Health status for individual components
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComponentHealth {
    Operational,
    Degraded,
    Unavailable,
}

/// Health status representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Overall health score (0-100)
    pub score: u8,
    /// Detailed component statuses
    pub components: HashMap<String, ComponentHealth>,
    /// Active alerts
    pub alerts: Vec<Alert>,
    /// Timestamp of last update
    pub last_updated: SystemTime,
}

/// Alert representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub component: String,
    pub message: String,
    pub severity: AlertSeverity,
    pub created_at: SystemTime,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

impl HealthStatus {
    /// Creates a new health status instance
    pub fn new() -> Self {
        Self {
            score: 100,
            components: HashMap::new(),
            alerts: Vec::new(),
            last_updated: SystemTime::now(),
        }
    }

    /// Updates health score based on metrics
    pub fn update_from_metrics(&mut self, metrics: &MetricsData) {
        // TODO: Implement scoring algorithm (UV-173)
        self.last_updated = SystemTime::now();
    }

    /// Adds an alert to the system
    pub fn add_alert(&mut self, component: &str, message: &str, severity: AlertSeverity) {
        let alert = Alert {
            id: format!("alert-{}", self.alerts.len() + 1),
            component: component.to_string(),
            message: message.to_string(),
            severity,
            created_at: SystemTime::now(),
        };
        self.alerts.push(alert);
    }
}

use std::sync::Arc;
use tokio::sync::RwLock;

/// Health monitoring service
#[derive(Debug, Clone)]
pub struct HealthMonitor {
    status: Arc<RwLock<HealthStatus>>,
}

impl HealthMonitor {
    pub fn new() -> Self {
        Self {
            status: Arc::new(RwLock::new(HealthStatus::new())),
        }
    }

    pub async fn get_status(&self) -> HealthStatus {
        self.status.read().await.clone()
    }

    pub async fn get_detailed_status(&self) -> HealthStatus {
        self.get_status().await
    }

    pub async fn get_alerts(&self) -> Vec<Alert> {
        self.status.read().await.alerts.clone()
    }

    pub async fn update_from_metrics(&self, metrics: &MetricsData) {
        let mut status = self.status.write().await;
        status.update_from_metrics(metrics);
    }

    pub async fn add_alert(&self, component: &str, message: &str, severity: AlertSeverity) {
        let mut status = self.status.write().await;
        status.add_alert(component, message, severity);
    }
}

/// Metrics data structure (placeholder - will integrate with UV-174)
#[derive(Debug, Clone)]
pub struct MetricsData {
    pub error_rate: f32,
    pub latency: Duration,
    pub throughput: u32,
}
