//! Escalation and acknowledgment workflows for alert system (UV-248)

use crate::resilience::alerting::{
use tracing::{info, warn, error, debug};
    AlertingError, EnhancedAlert, EscalationLevel, EscalationPolicy, NotificationChannel,
};
use crate::resilience::notifications::NotificationClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tokio::time::{sleep, Instant};
use tracing::{info, warn, error, debug};

/// Escalation state for tracking alert escalation progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationState {
    pub alert_id: String,
    pub policy_name: String,
    pub current_level: u8,
    pub next_escalation_time: Option<SystemTime>,
    pub escalation_history: Vec<EscalationEvent>,
    pub is_active: bool,
}

/// Event in escalation history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationEvent {
    pub level: u8,
    pub timestamp: SystemTime,
    pub channels_notified: Vec<String>,
    pub roles_notified: Vec<String>,
    pub successful: bool,
    pub error_message: Option<String>,
}

/// Acknowledgment record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcknowledgmentRecord {
    pub alert_id: String,
    pub acknowledged_by: String,
    pub acknowledged_at: SystemTime,
    pub acknowledgment_source: AcknowledgmentSource,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AcknowledgmentSource {
    WebUI,
    Slack,
    Email,
    API,
    CLI,
}

/// Escalation manager handles alert escalation workflows
pub struct EscalationManager {
    escalation_states: Arc<RwLock<HashMap<String, EscalationState>>>,
    acknowledgments: Arc<RwLock<HashMap<String, AcknowledgmentRecord>>>,
    notification_client: Arc<NotificationClient>,
    policies: Arc<RwLock<HashMap<String, EscalationPolicy>>>,
}

impl EscalationManager {
    pub fn new() -> Self {
        Self {
            escalation_states: Arc::new(RwLock::new(HashMap::new())),
            acknowledgments: Arc::new(RwLock::new(HashMap::new())),
            notification_client: Arc::new(NotificationClient::new()),
            policies: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start escalation workflow for an alert
    pub async fn start_escalation(
        &self,
        alert: &EnhancedAlert,
        policy: &EscalationPolicy,
        channels: &[NotificationChannel],
    ) -> Result<(), AlertingError> {
        if !policy.enabled {
            return Ok(());
        }

        let escalation_state = EscalationState {
            alert_id: alert.base_alert.id.clone(),
            policy_name: policy.name.clone(),
            current_level: 0,
            next_escalation_time: None,
            escalation_history: Vec::new(),
            is_active: true,
        };

        // Store initial state
        {
            let mut states = self.escalation_states.write().await;
            states.insert(alert.base_alert.id.clone(), escalation_state);
        }

        // Start with level 1 immediately
        self.escalate_to_level(alert, policy, channels, 1).await?;

        // Schedule future escalations if policy has multiple levels
        if policy.levels.len() > 1 {
            self.schedule_escalations(alert.base_alert.id.clone(), policy, channels.to_vec())
                .await;
        }

        Ok(())
    }

    /// Escalate alert to specific level
    async fn escalate_to_level(
        &self,
        alert: &EnhancedAlert,
        policy: &EscalationPolicy,
        channels: &[NotificationChannel],
        level: u8,
    ) -> Result<(), AlertingError> {
        let escalation_level =
            policy
                .levels
                .iter()
                .find(|l| l.level == level)
                .ok_or_else(|| {
                    AlertingError::ConfigurationError(format!(
                        "Escalation level {} not found in policy {}",
                        level, policy.name
                    ))
                })?;

        let mut successful_channels = Vec::new();
        let mut error_message = None;

        // Send notifications to all channels for this level
        for channel_name in &escalation_level.channels {
            if let Some(channel) = channels
                .iter()
                .find(|c| c.name == *channel_name && c.enabled)
            {
                match self
                    .notification_client
                    .send_notification(channel, alert)
                    .await
                {
                    Ok(_) => successful_channels.push(channel_name.clone()),
                    Err(e) => {
                        error!(
                            "Failed to send escalation notification via {}: {}",
                            channel_name, e
                        );
                        error_message = Some(format!("Channel {} failed: {}", channel_name, e));
                    }
                }
            }
        }

        // Record escalation event
        let event = EscalationEvent {
            level,
            timestamp: SystemTime::now(),
            channels_notified: successful_channels,
            roles_notified: escalation_level.roles.clone(),
            successful: error_message.is_none(),
            error_message,
        };

        // Update escalation state
        {
            let mut states = self.escalation_states.write().await;
            if let Some(state) = states.get_mut(&alert.base_alert.id) {
                state.current_level = level;
                state.escalation_history.push(event);

                // Set next escalation time if there's a higher level
                if let Some(next_level) = policy.levels.iter().find(|l| l.level == level + 1) {
                    let delay = Duration::from_secs(next_level.delay_minutes * 60);
                    state.next_escalation_time = Some(SystemTime::now() + delay);
                }
            }
        }

        Ok(())
    }

    /// Schedule future escalations
    async fn schedule_escalations(
        &self,
        alert_id: String,
        policy: &EscalationPolicy,
        channels: Vec<NotificationChannel>,
    ) {
        let manager = self.clone();
        let policy_clone = policy.clone();

        tokio::spawn(async move {
            for level in policy_clone.levels.iter().skip(1) {
                // Wait for the delay period
                let delay = Duration::from_secs(level.delay_minutes * 60);
                sleep(delay).await;

                // Check if alert is still active and not acknowledged
                let should_escalate = {
                    let states = manager.escalation_states.read().await;
                    let acks = manager.acknowledgments.read().await;

                    match states.get(&alert_id) {
                        Some(state) => state.is_active && !acks.contains_key(&alert_id),
                        None => false,
                    }
                };

                if should_escalate {
                    // Get the alert from the enhanced alerts store (would need reference to main system)
                    // For now, we'll just log the escalation
                    println!(
                        "Escalating alert {} to level {} after {} minutes",
                        alert_id, level.level, level.delay_minutes
                    );

                    // In a real implementation, we'd retrieve the alert and escalate
                    // manager.escalate_to_level(&alert, &policy_clone, &channels, level.level).await;
                } else {
                    // Alert was acknowledged or deactivated, stop escalation
                    println!(
                        "Stopping escalation for alert {} - acknowledged or deactivated",
                        alert_id
                    );
                    break;
                }
            }
        });
    }

    /// Acknowledge an alert and stop escalation
    pub async fn acknowledge_alert(
        &self,
        alert_id: &str,
        acknowledged_by: &str,
        source: AcknowledgmentSource,
        message: Option<String>,
    ) -> Result<(), AlertingError> {
        let acknowledgment = AcknowledgmentRecord {
            alert_id: alert_id.to_string(),
            acknowledged_by: acknowledged_by.to_string(),
            acknowledged_at: SystemTime::now(),
            acknowledgment_source: source,
            message,
        };

        // Clone for logging before moving
        let acknowledgment_source = acknowledgment.acknowledgment_source.clone();

        // Store acknowledgment
        {
            let mut acks = self.acknowledgments.write().await;
            acks.insert(alert_id.to_string(), acknowledgment);
        }

        // Deactivate escalation
        {
            let mut states = self.escalation_states.write().await;
            if let Some(state) = states.get_mut(alert_id) {
                state.is_active = false;
                state.next_escalation_time = None;
            }
        }

        println!(
            "Alert {} acknowledged by {} via {:?}",
            alert_id, acknowledged_by, acknowledgment_source
        );
        Ok(())
    }

    /// Get escalation status for an alert
    pub async fn get_escalation_status(&self, alert_id: &str) -> Option<EscalationState> {
        let states = self.escalation_states.read().await;
        states.get(alert_id).cloned()
    }

    /// Get acknowledgment record for an alert
    pub async fn get_acknowledgment(&self, alert_id: &str) -> Option<AcknowledgmentRecord> {
        let acks = self.acknowledgments.read().await;
        acks.get(alert_id).cloned()
    }

    /// Update escalation policies
    pub async fn update_policy(&self, policy: EscalationPolicy) {
        let mut policies = self.policies.write().await;
        policies.insert(policy.name.clone(), policy);
    }

    /// Get all active escalations
    pub async fn get_active_escalations(&self) -> Vec<EscalationState> {
        let states = self.escalation_states.read().await;
        states
            .values()
            .filter(|state| state.is_active)
            .cloned()
            .collect()
    }

    /// Get escalation statistics
    pub async fn get_escalation_stats(&self, hours: u32) -> EscalationStats {
        let cutoff = SystemTime::now() - Duration::from_secs(hours as u64 * 3600);
        let states = self.escalation_states.read().await;
        let acks = self.acknowledgments.read().await;

        let total_alerts = states.len();
        let acknowledged_alerts = acks
            .values()
            .filter(|ack| ack.acknowledged_at >= cutoff)
            .count();

        let escalation_levels: HashMap<u8, u32> = states
            .values()
            .filter(|state| {
                state
                    .escalation_history
                    .iter()
                    .any(|event| event.timestamp >= cutoff)
            })
            .map(|state| state.current_level)
            .fold(HashMap::new(), |mut acc, level| {
                *acc.entry(level).or_insert(0) += 1;
                acc
            });

        let avg_acknowledgment_time = if acknowledged_alerts > 0 {
            let total_time: Duration = acks
                .values()
                .filter(|ack| ack.acknowledged_at >= cutoff)
                .filter_map(|ack| {
                    states.get(&ack.alert_id).and_then(|state| {
                        state.escalation_history.first().map(|first_event| {
                            ack.acknowledged_at
                                .duration_since(first_event.timestamp)
                                .unwrap_or(Duration::ZERO)
                        })
                    })
                })
                .sum();

            Some(total_time / acknowledged_alerts as u32)
        } else {
            None
        };

        EscalationStats {
            total_alerts,
            acknowledged_alerts,
            escalation_levels,
            avg_acknowledgment_time,
            period_hours: hours,
        }
    }

    /// Force escalation for testing
    pub async fn force_escalate(&self, alert_id: &str, level: u8) -> Result<(), AlertingError> {
        let mut states = self.escalation_states.write().await;

        if let Some(state) = states.get_mut(alert_id) {
            if state.is_active {
                state.current_level = level;
                state.next_escalation_time = Some(SystemTime::now());
                println!("Forced escalation of alert {} to level {}", alert_id, level);
                Ok(())
            } else {
                Err(AlertingError::ConfigurationError(
                    "Cannot escalate inactive alert".to_string(),
                ))
            }
        } else {
            Err(AlertingError::AlertNotFound)
        }
    }
}

/// Statistics for escalation analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationStats {
    pub total_alerts: usize,
    pub acknowledged_alerts: usize,
    pub escalation_levels: HashMap<u8, u32>,
    pub avg_acknowledgment_time: Option<Duration>,
    pub period_hours: u32,
}

// Implement Clone for EscalationManager to support async spawning
impl Clone for EscalationManager {
    fn clone(&self) -> Self {
        Self {
            escalation_states: Arc::clone(&self.escalation_states),
            acknowledgments: Arc::clone(&self.acknowledgments),
            notification_client: Arc::clone(&self.notification_client),
            policies: Arc::clone(&self.policies),
        }
    }
}

/// Acknowledgment API for external integrations
pub struct AcknowledgmentAPI {
    escalation_manager: Arc<EscalationManager>,
}

impl AcknowledgmentAPI {
    pub fn new(escalation_manager: Arc<EscalationManager>) -> Self {
        Self { escalation_manager }
    }

    /// Acknowledge alert via API
    pub async fn acknowledge_via_api(
        &self,
        alert_id: &str,
        user_id: &str,
        message: Option<String>,
    ) -> Result<(), AlertingError> {
        self.escalation_manager
            .acknowledge_alert(alert_id, user_id, AcknowledgmentSource::API, message)
            .await
    }

    /// Acknowledge alert via Slack (webhook callback)
    pub async fn acknowledge_via_slack(
        &self,
        alert_id: &str,
        slack_user: &str,
        message: Option<String>,
    ) -> Result<(), AlertingError> {
        self.escalation_manager
            .acknowledge_alert(alert_id, slack_user, AcknowledgmentSource::Slack, message)
            .await
    }

    /// Acknowledge alert via web UI
    pub async fn acknowledge_via_web(
        &self,
        alert_id: &str,
        user_id: &str,
        message: Option<String>,
    ) -> Result<(), AlertingError> {
        self.escalation_manager
            .acknowledge_alert(alert_id, user_id, AcknowledgmentSource::WebUI, message)
            .await
    }

    /// Get acknowledgment status
    pub async fn get_acknowledgment_status(&self, alert_id: &str) -> Option<AcknowledgmentRecord> {
        self.escalation_manager.get_acknowledgment(alert_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resilience::alerting::{AlertType, ChannelConfig, ChannelType};
    use crate::resilience::health::{Alert, AlertSeverity};
    use std::collections::HashMap;

    fn create_test_alert() -> EnhancedAlert {
        EnhancedAlert {
            base_alert: Alert {
                id: "test-escalation-123".to_string(),
                component: "test-component".to_string(),
                message: "Test escalation alert".to_string(),
                severity: AlertSeverity::Critical,
                created_at: SystemTime::now(),
            },
            alert_type: AlertType::CriticalFailureRate,
            fingerprint: "test-escalation-fingerprint".to_string(),
            group_key: "test-escalation-group".to_string(),
            environment: "production".to_string(),
            metadata: HashMap::new(),
            acknowledged: false,
            acknowledged_at: None,
            acknowledged_by: None,
            escalation_level: 0,
            escalated_at: None,
        }
    }

    fn create_test_policy() -> EscalationPolicy {
        EscalationPolicy {
            name: "test-policy".to_string(),
            levels: vec![
                EscalationLevel {
                    level: 1,
                    delay_minutes: 0,
                    channels: vec!["test-slack".to_string()],
                    roles: vec!["on-call".to_string()],
                },
                EscalationLevel {
                    level: 2,
                    delay_minutes: 5,
                    channels: vec!["test-email".to_string()],
                    roles: vec!["team-lead".to_string()],
                },
            ],
            enabled: true,
        }
    }

    fn create_test_channels() -> Vec<NotificationChannel> {
        vec![
            NotificationChannel {
                name: "test-slack".to_string(),
                channel_type: ChannelType::Slack,
                config: ChannelConfig {
                    webhook_url: Some("https://hooks.slack.com/test".to_string()),
                    email_recipients: None,
                    github_repo: None,
                    github_token: None,
                },
                enabled: true,
            },
            NotificationChannel {
                name: "test-email".to_string(),
                channel_type: ChannelType::Email,
                config: ChannelConfig {
                    webhook_url: None,
                    email_recipients: Some(vec!["test@example.com".to_string()]),
                    github_repo: None,
                    github_token: None,
                },
                enabled: true,
            },
        ]
    }

    #[tokio::test]
    async fn test_escalation_manager_creation() {
        let manager = EscalationManager::new();

        let states = manager.escalation_states.read().await;
        assert!(states.is_empty());
    }

    #[tokio::test]
    async fn test_start_escalation() {
        let manager = EscalationManager::new();
        let alert = create_test_alert();
        let policy = create_test_policy();
        let channels = create_test_channels();

        let result = manager.start_escalation(&alert, &policy, &channels).await;
        assert!(result.is_ok());

        let state = manager.get_escalation_status(&alert.base_alert.id).await;
        assert!(state.is_some());

        let state = state.unwrap();
        assert_eq!(state.alert_id, alert.base_alert.id);
        assert_eq!(state.current_level, 1);
        assert!(state.is_active);
        assert!(!state.escalation_history.is_empty());
    }

    #[tokio::test]
    async fn test_acknowledge_alert() {
        let manager = EscalationManager::new();
        let alert = create_test_alert();
        let policy = create_test_policy();
        let channels = create_test_channels();

        // Start escalation
        manager
            .start_escalation(&alert, &policy, &channels)
            .await
            .unwrap();

        // Acknowledge alert
        let result = manager
            .acknowledge_alert(
                &alert.base_alert.id,
                "test-user",
                AcknowledgmentSource::WebUI,
                Some("Investigating the issue".to_string()),
            )
            .await;
        assert!(result.is_ok());

        // Check acknowledgment record
        let ack = manager.get_acknowledgment(&alert.base_alert.id).await;
        assert!(ack.is_some());

        let ack = ack.unwrap();
        assert_eq!(ack.acknowledged_by, "test-user");
        assert_eq!(ack.message, Some("Investigating the issue".to_string()));

        // Check escalation state is deactivated
        let state = manager.get_escalation_status(&alert.base_alert.id).await;
        assert!(state.is_some());
        assert!(!state.unwrap().is_active);
    }

    #[tokio::test]
    async fn test_acknowledgment_api() {
        let manager = Arc::new(EscalationManager::new());
        let api = AcknowledgmentAPI::new(Arc::clone(&manager));

        let alert = create_test_alert();
        let policy = create_test_policy();
        let channels = create_test_channels();

        // Start escalation
        manager
            .start_escalation(&alert, &policy, &channels)
            .await
            .unwrap();

        // Acknowledge via API
        let result = api
            .acknowledge_via_api(
                &alert.base_alert.id,
                "api-user",
                Some("Fixed via API".to_string()),
            )
            .await;
        assert!(result.is_ok());

        // Verify acknowledgment
        let ack_status = api.get_acknowledgment_status(&alert.base_alert.id).await;
        assert!(ack_status.is_some());
        assert_eq!(ack_status.unwrap().acknowledged_by, "api-user");
    }

    #[tokio::test]
    async fn test_escalation_stats() {
        let manager = EscalationManager::new();
        let alert = create_test_alert();
        let policy = create_test_policy();
        let channels = create_test_channels();

        // Start escalation
        manager
            .start_escalation(&alert, &policy, &channels)
            .await
            .unwrap();

        // Acknowledge alert
        manager
            .acknowledge_alert(
                &alert.base_alert.id,
                "test-user",
                AcknowledgmentSource::WebUI,
                None,
            )
            .await
            .unwrap();

        // Get stats
        let stats = manager.get_escalation_stats(1).await;
        assert_eq!(stats.total_alerts, 1);
        assert_eq!(stats.acknowledged_alerts, 1);
        assert!(stats.avg_acknowledgment_time.is_some());
    }
}
