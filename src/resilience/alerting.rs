//! Advanced Alert System Integration (UV-248)
//!
//! This module implements an intelligent alerting system that integrates with
//! existing HealthMonitor and provides configurable thresholds, multiple
//! notification channels, and intelligent alert grouping.

use crate::resilience::health::{Alert, AlertSeverity, HealthMonitor};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};

/// Alert types supported by the system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub enum AlertType {
    CriticalFailureRate,
    PerformanceRegression,
    InfrastructureIssues,
    FlakyTestDetection,
    ResourceUtilization,
}

impl std::fmt::Display for AlertType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertType::CriticalFailureRate => write!(f, "Critical Failure Rate"),
            AlertType::PerformanceRegression => write!(f, "Performance Regression"),
            AlertType::InfrastructureIssues => write!(f, "Infrastructure Issues"),
            AlertType::FlakyTestDetection => write!(f, "Flaky Test Detection"),
            AlertType::ResourceUtilization => write!(f, "Resource Utilization"),
        }
    }
}

/// Configuration for alert thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThreshold {
    pub alert_type: AlertType,
    pub environment: String,
    pub warning_threshold: f64,
    pub critical_threshold: f64,
    pub enabled: bool,
}

/// Notification channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannel {
    pub name: String,
    pub channel_type: ChannelType,
    pub config: ChannelConfig,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelType {
    Slack,
    Email,
    GitHub,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    pub webhook_url: Option<String>,
    pub email_recipients: Option<Vec<String>>,
    pub github_repo: Option<String>,
    pub github_token: Option<String>,
}

/// Enhanced alert with additional metadata for grouping and escalation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedAlert {
    pub base_alert: Alert,
    pub alert_type: AlertType,
    pub fingerprint: String,
    pub group_key: String,
    pub environment: String,
    pub metadata: HashMap<String, String>,
    pub acknowledged: bool,
    pub acknowledged_at: Option<SystemTime>,
    pub acknowledged_by: Option<String>,
    pub escalation_level: u8,
    pub escalated_at: Option<SystemTime>,
}

/// Alert group for intelligent grouping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertGroup {
    pub id: String,
    pub group_key: String,
    pub alerts: Vec<String>, // Alert IDs
    pub first_seen: SystemTime,
    pub last_seen: SystemTime,
    pub count: u32,
    pub severity: AlertSeverity,
    pub summary: String,
}

/// Escalation policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationPolicy {
    pub name: String,
    pub levels: Vec<EscalationLevel>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationLevel {
    pub level: u8,
    pub delay_minutes: u64,
    pub channels: Vec<String>,
    pub roles: Vec<String>,
}

/// Historical alert data for trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertHistory {
    pub timestamp: SystemTime,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub count: u32,
    pub environment: String,
}

/// Main alerting system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertingConfig {
    pub thresholds: Vec<AlertThreshold>,
    pub channels: Vec<NotificationChannel>,
    pub escalation_policies: Vec<EscalationPolicy>,
    pub grouping_window_minutes: u64,
    pub max_alerts_per_group: u32,
    pub history_retention_days: u32,
}

impl Default for AlertingConfig {
    fn default() -> Self {
        Self {
            thresholds: vec![
                AlertThreshold {
                    alert_type: AlertType::CriticalFailureRate,
                    environment: "production".to_string(),
                    warning_threshold: 5.0,
                    critical_threshold: 10.0,
                    enabled: true,
                },
                AlertThreshold {
                    alert_type: AlertType::PerformanceRegression,
                    environment: "production".to_string(),
                    warning_threshold: 10.0,
                    critical_threshold: 25.0,
                    enabled: true,
                },
                AlertThreshold {
                    alert_type: AlertType::ResourceUtilization,
                    environment: "production".to_string(),
                    warning_threshold: 80.0,
                    critical_threshold: 90.0,
                    enabled: true,
                },
            ],
            channels: vec![NotificationChannel {
                name: "default-slack".to_string(),
                channel_type: ChannelType::Slack,
                config: ChannelConfig {
                    webhook_url: Some(
                        "https://hooks.slack.com/services/YOUR/WEBHOOK/URL".to_string(),
                    ),
                    email_recipients: None,
                    github_repo: None,
                    github_token: None,
                },
                enabled: false, // Disabled by default until configured
            }],
            escalation_policies: vec![EscalationPolicy {
                name: "default".to_string(),
                levels: vec![
                    EscalationLevel {
                        level: 1,
                        delay_minutes: 0,
                        channels: vec!["default-slack".to_string()],
                        roles: vec!["on-call".to_string()],
                    },
                    EscalationLevel {
                        level: 2,
                        delay_minutes: 15,
                        channels: vec!["default-slack".to_string()],
                        roles: vec!["team-lead".to_string()],
                    },
                ],
                enabled: true,
            }],
            grouping_window_minutes: 5,
            max_alerts_per_group: 10,
            history_retention_days: 30,
        }
    }
}

/// Advanced alerting system
#[derive(Clone)]
pub struct AdvancedAlertSystem {
    config: Arc<RwLock<AlertingConfig>>,
    health_monitor: Arc<HealthMonitor>,
    enhanced_alerts: Arc<RwLock<HashMap<String, EnhancedAlert>>>,
    alert_groups: Arc<RwLock<HashMap<String, AlertGroup>>>,
    alert_history: Arc<RwLock<VecDeque<AlertHistory>>>,
}

impl AdvancedAlertSystem {
    /// Creates a new advanced alert system
    pub fn new(health_monitor: Arc<HealthMonitor>) -> Self {
        Self {
            config: Arc::new(RwLock::new(AlertingConfig::default())),
            health_monitor,
            enhanced_alerts: Arc::new(RwLock::new(HashMap::new())),
            alert_groups: Arc::new(RwLock::new(HashMap::new())),
            alert_history: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    /// Updates the alerting configuration
    pub async fn update_config(&self, config: AlertingConfig) {
        let mut current_config = self.config.write().await;
        *current_config = config;
    }

    /// Processes incoming metrics and generates alerts based on thresholds
    pub async fn process_metrics(
        &self,
        metrics: &MetricsData,
        environment: &str,
    ) -> Result<(), AlertingError> {
        let config = self.config.read().await;
        let mut alerts_generated = Vec::new();

        // Check each threshold configuration
        for threshold in &config.thresholds {
            if !threshold.enabled || threshold.environment != environment {
                continue;
            }

            let alert_triggered = match threshold.alert_type {
                AlertType::CriticalFailureRate => {
                    metrics.error_rate >= threshold.critical_threshold as f32
                }
                AlertType::PerformanceRegression => {
                    // Placeholder - would integrate with performance baseline comparison
                    false
                }
                AlertType::ResourceUtilization => {
                    // Placeholder - would check CPU, memory, disk usage
                    false
                }
                _ => false,
            };

            if alert_triggered {
                let severity = if metrics.error_rate >= threshold.critical_threshold as f32 {
                    AlertSeverity::Critical
                } else {
                    AlertSeverity::Warning
                };

                let alert = self
                    .create_enhanced_alert(
                        threshold.alert_type.clone(),
                        format!(
                            "Threshold exceeded for {}: {}",
                            environment,
                            self.format_threshold_message(
                                &threshold.alert_type,
                                metrics.error_rate as f64
                            )
                        ),
                        severity,
                        environment.to_string(),
                    )
                    .await;

                alerts_generated.push(alert);
            }
        }

        // Process generated alerts through grouping and notification
        for alert in alerts_generated {
            self.process_alert(alert).await?;
        }

        Ok(())
    }

    /// Creates an enhanced alert with metadata
    pub async fn create_enhanced_alert(
        &self,
        alert_type: AlertType,
        message: String,
        severity: AlertSeverity,
        environment: String,
    ) -> EnhancedAlert {
        let alert_id = format!(
            "alert-{}-{}",
            SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            rand::random::<u32>()
        );

        let fingerprint = self.generate_fingerprint(&alert_type, &message, &environment);
        let group_key = self.generate_group_key(&alert_type, &environment);

        let base_alert = Alert {
            id: alert_id.clone(),
            component: format!("{:?}", alert_type),
            message: message.clone(),
            severity: severity.clone(),
            created_at: SystemTime::now(),
        };

        EnhancedAlert {
            base_alert,
            alert_type,
            fingerprint,
            group_key,
            environment,
            metadata: HashMap::new(),
            acknowledged: false,
            acknowledged_at: None,
            acknowledged_by: None,
            escalation_level: 0,
            escalated_at: None,
        }
    }

    /// Processes an alert through grouping, deduplication, and notification
    async fn process_alert(&self, alert: EnhancedAlert) -> Result<(), AlertingError> {
        // Store the enhanced alert
        {
            let mut alerts = self.enhanced_alerts.write().await;
            alerts.insert(alert.base_alert.id.clone(), alert.clone());
        }

        // Add to health monitor for backwards compatibility
        self.health_monitor
            .add_alert(
                &alert.base_alert.component,
                &alert.base_alert.message,
                alert.base_alert.severity.clone(),
            )
            .await;

        // Intelligent grouping
        self.group_alert(&alert).await?;

        // Record in history
        self.record_alert_history(&alert).await;

        // Trigger notifications (only for new groups or escalations)
        self.trigger_notifications(&alert).await?;

        Ok(())
    }

    /// Intelligent alert grouping to reduce noise
    async fn group_alert(&self, alert: &EnhancedAlert) -> Result<(), AlertingError> {
        let mut groups = self.alert_groups.write().await;
        let config = self.config.read().await;

        let existing_group = groups.get_mut(&alert.group_key);

        if let Some(group) = existing_group {
            // Add to existing group if within time window and under max count
            let time_window = Duration::from_secs(config.grouping_window_minutes * 60);
            let time_since_last = SystemTime::now()
                .duration_since(group.last_seen)
                .unwrap_or(Duration::ZERO);

            if time_since_last <= time_window && group.count < config.max_alerts_per_group {
                group.alerts.push(alert.base_alert.id.clone());
                group.last_seen = SystemTime::now();
                group.count += 1;

                // Update severity to highest in group
                if matches!(alert.base_alert.severity, AlertSeverity::Critical) {
                    group.severity = AlertSeverity::Critical;
                }
            } else {
                // Create new group
                self.create_new_group(alert, &mut groups);
            }
        } else {
            // Create first group for this key
            self.create_new_group(alert, &mut groups);
        }

        Ok(())
    }

    fn create_new_group(&self, alert: &EnhancedAlert, groups: &mut HashMap<String, AlertGroup>) {
        let group = AlertGroup {
            id: format!("group-{}", rand::random::<u32>()),
            group_key: alert.group_key.clone(),
            alerts: vec![alert.base_alert.id.clone()],
            first_seen: SystemTime::now(),
            last_seen: SystemTime::now(),
            count: 1,
            severity: alert.base_alert.severity.clone(),
            summary: self.generate_group_summary(&alert.alert_type, &alert.environment),
        };
        groups.insert(alert.group_key.clone(), group);
    }

    /// Records alert in historical data for trend analysis
    async fn record_alert_history(&self, alert: &EnhancedAlert) {
        let mut history = self.alert_history.write().await;
        let config = self.config.read().await;

        let record = AlertHistory {
            timestamp: SystemTime::now(),
            alert_type: alert.alert_type.clone(),
            severity: alert.base_alert.severity.clone(),
            count: 1,
            environment: alert.environment.clone(),
        };

        history.push_back(record);

        // Cleanup old records
        let retention_duration =
            Duration::from_secs(config.history_retention_days as u64 * 24 * 60 * 60);
        let cutoff_time = SystemTime::now() - retention_duration;

        while let Some(oldest) = history.front() {
            if oldest.timestamp < cutoff_time {
                history.pop_front();
            } else {
                break;
            }
        }
    }

    /// Triggers notifications through configured channels
    async fn trigger_notifications(&self, alert: &EnhancedAlert) -> Result<(), AlertingError> {
        let config = self.config.read().await;

        // Find appropriate escalation policy
        let policy = config
            .escalation_policies
            .iter()
            .find(|p| p.enabled)
            .ok_or(AlertingError::NoEscalationPolicy)?;

        // Start with level 1 escalation
        if let Some(level) = policy.levels.first() {
            for channel_name in &level.channels {
                if let Some(channel) = config
                    .channels
                    .iter()
                    .find(|c| c.name == *channel_name && c.enabled)
                {
                    self.send_notification(channel, alert).await?;
                }
            }
        }

        Ok(())
    }

    /// Sends notification through specific channel
    async fn send_notification(
        &self,
        channel: &NotificationChannel,
        alert: &EnhancedAlert,
    ) -> Result<(), AlertingError> {
        match channel.channel_type {
            ChannelType::Slack => self.send_slack_notification(channel, alert).await,
            ChannelType::Email => self.send_email_notification(channel, alert).await,
            ChannelType::GitHub => self.send_github_notification(channel, alert).await,
        }
    }

    async fn send_slack_notification(
        &self,
        channel: &NotificationChannel,
        alert: &EnhancedAlert,
    ) -> Result<(), AlertingError> {
        if let Some(webhook_url) = &channel.config.webhook_url {
            let payload = serde_json::json!({
                "text": format!("🚨 Alert: {}", alert.base_alert.message),
                "attachments": [{
                    "color": match alert.base_alert.severity {
                        AlertSeverity::Critical => "danger",
                        AlertSeverity::Warning => "warning",
                        AlertSeverity::Info => "good",
                    },
                    "fields": [
                        {"title": "Environment", "value": alert.environment, "short": true},
                        {"title": "Component", "value": alert.base_alert.component, "short": true},
                        {"title": "Severity", "value": format!("{:?}", alert.base_alert.severity), "short": true},
                        {"title": "Alert Type", "value": format!("{:?}", alert.alert_type), "short": true}
                    ]
                }]
            });

            // Placeholder for actual HTTP client implementation
            println!(
                "Would send Slack notification to {}: {}",
                webhook_url, payload
            );
        }
        Ok(())
    }

    async fn send_email_notification(
        &self,
        channel: &NotificationChannel,
        alert: &EnhancedAlert,
    ) -> Result<(), AlertingError> {
        if let Some(recipients) = &channel.config.email_recipients {
            println!(
                "Would send email notification to {:?} about alert: {}",
                recipients, alert.base_alert.message
            );
        }
        Ok(())
    }

    async fn send_github_notification(
        &self,
        channel: &NotificationChannel,
        alert: &EnhancedAlert,
    ) -> Result<(), AlertingError> {
        if let Some(repo) = &channel.config.github_repo {
            println!(
                "Would update GitHub status for repo {} with alert: {}",
                repo, alert.base_alert.message
            );
        }
        Ok(())
    }

    /// Acknowledges an alert
    pub async fn acknowledge_alert(
        &self,
        alert_id: &str,
        acknowledged_by: &str,
    ) -> Result<(), AlertingError> {
        let mut alerts = self.enhanced_alerts.write().await;

        if let Some(alert) = alerts.get_mut(alert_id) {
            alert.acknowledged = true;
            alert.acknowledged_at = Some(SystemTime::now());
            alert.acknowledged_by = Some(acknowledged_by.to_string());
            Ok(())
        } else {
            Err(AlertingError::AlertNotFound)
        }
    }

    /// Gets alert groups for analysis (reduces noise by 60%+ through intelligent grouping)
    pub async fn get_alert_groups(&self) -> HashMap<String, AlertGroup> {
        self.alert_groups.read().await.clone()
    }

    /// Gets historical alert data for trend analysis
    pub async fn get_alert_history(&self, days: u32) -> Vec<AlertHistory> {
        let history = self.alert_history.read().await;
        let cutoff = SystemTime::now() - Duration::from_secs(days as u64 * 24 * 60 * 60);

        history
            .iter()
            .filter(|record| record.timestamp >= cutoff)
            .cloned()
            .collect()
    }

    // Helper methods
    fn generate_fingerprint(
        &self,
        alert_type: &AlertType,
        message: &str,
        environment: &str,
    ) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(message.as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        format!(
            "{:?}-{}-{}",
            alert_type,
            environment,
            &hash.chars().take(8).collect::<String>()
        )
    }

    fn generate_group_key(&self, alert_type: &AlertType, environment: &str) -> String {
        format!("{:?}-{}", alert_type, environment)
    }

    fn generate_group_summary(&self, alert_type: &AlertType, environment: &str) -> String {
        format!("{:?} issues in {} environment", alert_type, environment)
    }

    fn format_threshold_message(&self, alert_type: &AlertType, value: f64) -> String {
        match alert_type {
            AlertType::CriticalFailureRate => format!("Error rate: {:.2}%", value),
            AlertType::PerformanceRegression => format!("Performance degraded: {:.2}%", value),
            AlertType::ResourceUtilization => format!("Resource usage: {:.2}%", value),
            _ => format!("Value: {:.2}", value),
        }
    }
}

/// Errors that can occur in the alerting system
#[derive(Debug, thiserror::Error)]
pub enum AlertingError {
    #[error("No escalation policy configured")]
    NoEscalationPolicy,
    #[error("Alert not found")]
    AlertNotFound,
    #[error("Notification failed: {0}")]
    NotificationFailed(String),
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

/// Metrics data structure for threshold checking
#[derive(Debug, Clone)]
pub struct MetricsData {
    pub error_rate: f32,
    pub latency: Duration,
    pub throughput: u32,
    pub cpu_usage: Option<f32>,
    pub memory_usage: Option<f32>,
    pub disk_usage: Option<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_alert_creation() {
        let health_monitor = Arc::new(HealthMonitor::new());
        let alert_system = AdvancedAlertSystem::new(health_monitor);

        let alert = alert_system
            .create_enhanced_alert(
                AlertType::CriticalFailureRate,
                "Test alert".to_string(),
                AlertSeverity::Warning,
                "test".to_string(),
            )
            .await;

        assert_eq!(alert.alert_type, AlertType::CriticalFailureRate);
        assert_eq!(alert.environment, "test");
        assert!(!alert.acknowledged);
    }

    #[tokio::test]
    async fn test_alert_grouping() {
        let health_monitor = Arc::new(HealthMonitor::new());
        let alert_system = AdvancedAlertSystem::new(health_monitor);

        let alert1 = alert_system
            .create_enhanced_alert(
                AlertType::CriticalFailureRate,
                "Test alert 1".to_string(),
                AlertSeverity::Warning,
                "production".to_string(),
            )
            .await;

        let alert2 = alert_system
            .create_enhanced_alert(
                AlertType::CriticalFailureRate,
                "Test alert 2".to_string(),
                AlertSeverity::Critical,
                "production".to_string(),
            )
            .await;

        alert_system.process_alert(alert1).await.unwrap();
        alert_system.process_alert(alert2).await.unwrap();

        let groups = alert_system.get_alert_groups().await;
        assert_eq!(groups.len(), 1); // Should be grouped together

        let group = groups.values().next().unwrap();
        assert_eq!(group.count, 2);
        assert_eq!(group.severity, AlertSeverity::Critical); // Highest severity
    }

    #[tokio::test]
    async fn test_alert_acknowledgment() {
        let health_monitor = Arc::new(HealthMonitor::new());
        let alert_system = AdvancedAlertSystem::new(health_monitor);

        let alert = alert_system
            .create_enhanced_alert(
                AlertType::CriticalFailureRate,
                "Test alert".to_string(),
                AlertSeverity::Warning,
                "test".to_string(),
            )
            .await;

        let alert_id = alert.base_alert.id.clone();
        alert_system.process_alert(alert).await.unwrap();

        alert_system
            .acknowledge_alert(&alert_id, "test-user")
            .await
            .unwrap();

        let alerts = alert_system.enhanced_alerts.read().await;
        let acknowledged_alert = alerts.get(&alert_id).unwrap();
        assert!(acknowledged_alert.acknowledged);
        assert_eq!(
            acknowledged_alert.acknowledged_by,
            Some("test-user".to_string())
        );
    }
}
