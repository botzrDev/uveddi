//! Health monitoring system for production deployments
//! 
//! Provides comprehensive health monitoring including:
//! - Service health checks
//! - Performance monitoring
//! - Alert management
//! - Auto-healing capabilities

use std::time::{Duration, SystemTime};
use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, timeout, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug, instrument};
use anyhow::{Result, anyhow};

use super::{HealthStatus, HealthCheckConfig, HealthCheckResult};

/// Health monitoring service
#[derive(Debug)]
pub struct HealthMonitor {
    config: HealthMonitorConfig,
    state: Arc<RwLock<MonitorState>>,
    alert_manager: AlertManager,
}

/// Health monitor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMonitorConfig {
    pub check_interval: Duration,
    pub alert_cooldown: Duration,
    pub auto_healing_enabled: bool,
    pub max_consecutive_failures: u32,
    pub escalation_threshold: u32,
    pub targets: Vec<MonitorTarget>,
}

/// Monitoring target configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorTarget {
    pub name: String,
    pub health_check: HealthCheckConfig,
    pub alerts: Vec<AlertRule>,
    pub auto_healing: Option<AutoHealingConfig>,
}

/// Alert rule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub name: String,
    pub condition: AlertCondition,
    pub severity: AlertSeverity,
    pub cooldown: Duration,
    pub notification_channels: Vec<String>,
}

/// Alert condition types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    HealthStatus(HealthStatus),
    ResponseTimeThreshold(Duration),
    ErrorRateThreshold(f64),
    ConsecutiveFailures(u32),
    CustomMetric { name: String, operator: String, threshold: f64 },
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// Auto-healing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoHealingConfig {
    pub enabled: bool,
    pub restart_pods: bool,
    pub scale_up: Option<ScaleConfig>,
    pub failover: Option<FailoverConfig>,
    pub max_attempts: u32,
    pub backoff_multiplier: f64,
}

/// Scaling configuration for auto-healing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaleConfig {
    pub min_replicas: u32,
    pub max_replicas: u32,
    pub scale_factor: f64,
}

/// Failover configuration for auto-healing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailoverConfig {
    pub target_environment: String,
    pub automatic: bool,
    pub approval_required: bool,
}

/// Monitor state
#[derive(Debug, Default)]
pub struct MonitorState {
    pub target_states: HashMap<String, TargetState>,
    pub active_alerts: HashMap<String, Alert>,
    pub health_history: HashMap<String, Vec<HealthCheckResult>>,
    pub last_check_time: Option<SystemTime>,
}

/// Target state information
#[derive(Debug, Clone)]
pub struct TargetState {
    pub name: String,
    pub last_health_check: Option<HealthCheckResult>,
    pub consecutive_failures: u32,
    pub last_alert_time: Option<SystemTime>,
    pub auto_healing_attempts: u32,
    pub metrics: HashMap<String, f64>,
}

/// Active alert information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub target: String,
    pub rule: AlertRule,
    pub triggered_at: SystemTime,
    pub acknowledged: bool,
    pub resolved: bool,
    pub resolution_time: Option<SystemTime>,
    pub metadata: HashMap<String, String>,
}

/// Alert manager for handling notifications
#[derive(Debug)]
pub struct AlertManager {
    notification_channels: HashMap<String, NotificationChannel>,
}

/// Notification channel types
#[derive(Debug)]
pub enum NotificationChannel {
    Slack { webhook_url: String, channel: String },
    Email { smtp_config: SmtpConfig, recipients: Vec<String> },
    Webhook { url: String, headers: HashMap<String, String> },
    PagerDuty { integration_key: String },
}

/// SMTP configuration for email notifications
#[derive(Debug)]
pub struct SmtpConfig {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub tls: bool,
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new() -> Self {
        Self::with_config(HealthMonitorConfig::default())
    }

    /// Create a health monitor with custom configuration
    pub fn with_config(config: HealthMonitorConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(MonitorState::default())),
            alert_manager: AlertManager::new(),
        }
    }

    /// Start the health monitoring service
    #[instrument(skip(self))]
    pub async fn start(&self) -> Result<()> {
        info!("Starting health monitoring service");

        // Initialize target states
        {
            let mut state = self.state.write().await;
            for target in &self.config.targets {
                state.target_states.insert(
                    target.name.clone(),
                    TargetState {
                        name: target.name.clone(),
                        last_health_check: None,
                        consecutive_failures: 0,
                        last_alert_time: None,
                        auto_healing_attempts: 0,
                        metrics: HashMap::new(),
                    },
                );
            }
        }

        // Start monitoring loop
        self.monitoring_loop().await
    }

    /// Perform a single health check
    #[instrument(skip(self))]
    pub async fn check_health(&self, config: &HealthCheckConfig) -> Result<HealthCheckResult> {
        debug!(
            endpoint = %config.endpoint,
            timeout = ?config.timeout,
            "Performing health check"
        );

        let start_time = Instant::now();
        let mut last_error = None;

        for attempt in 1..=config.retries {
            match timeout(config.timeout, self.execute_health_check(config)).await {
                Ok(Ok(result)) => {
                    debug!(
                        endpoint = %config.endpoint,
                        attempt = %attempt,
                        response_time = ?result.response_time,
                        status = ?result.status,
                        "Health check completed"
                    );
                    return Ok(result);
                }
                Ok(Err(e)) => {
                    warn!(
                        endpoint = %config.endpoint,
                        attempt = %attempt,
                        error = %e,
                        "Health check failed"
                    );
                    last_error = Some(e);
                }
                Err(_) => {
                    warn!(
                        endpoint = %config.endpoint,
                        attempt = %attempt,
                        timeout = ?config.timeout,
                        "Health check timed out"
                    );
                    last_error = Some(anyhow!("Health check timed out"));
                }
            }

            if attempt < config.retries {
                sleep(config.interval).await;
            }
        }

        // All retries failed
        let response_time = start_time.elapsed();
        Ok(HealthCheckResult {
            endpoint: config.endpoint.clone(),
            status: HealthStatus::Unhealthy,
            response_time,
            timestamp: SystemTime::now(),
            error_message: last_error.map(|e| e.to_string()),
            metadata: HashMap::new(),
        })
    }

    /// Get current monitoring status
    pub async fn get_status(&self) -> Result<MonitoringStatus> {
        let state = self.state.read().await;
        
        let mut target_statuses = HashMap::new();
        for (name, target_state) in &state.target_states {
            target_statuses.insert(name.clone(), target_state.clone());
        }

        let active_alerts: Vec<Alert> = state.active_alerts.values().cloned().collect();

        Ok(MonitoringStatus {
            targets: target_statuses,
            active_alerts,
            last_check_time: state.last_check_time,
            overall_health: self.calculate_overall_health(&state),
        })
    }

    /// Acknowledge an alert
    #[instrument(skip(self))]
    pub async fn acknowledge_alert(&self, alert_id: &str, user: &str) -> Result<()> {
        let mut state = self.state.write().await;
        
        if let Some(alert) = state.active_alerts.get_mut(alert_id) {
            alert.acknowledged = true;
            alert.metadata.insert("acknowledged_by".to_string(), user.to_string());
            alert.metadata.insert("acknowledged_at".to_string(), 
                SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs().to_string());
            
            info!(
                alert_id = %alert_id,
                user = %user,
                "Alert acknowledged"
            );
        } else {
            return Err(anyhow!("Alert not found: {}", alert_id));
        }

        Ok(())
    }

    /// Main monitoring loop
    #[instrument(skip(self))]
    async fn monitoring_loop(&self) -> Result<()> {
        info!("Starting monitoring loop");

        loop {
            let start_time = Instant::now();
            
            // Perform health checks for all targets
            for target in &self.config.targets {
                if let Err(e) = self.check_target_health(target).await {
                    error!(
                        target = %target.name,
                        error = %e,
                        "Failed to check target health"
                    );
                }
            }

            // Update last check time
            {
                let mut state = self.state.write().await;
                state.last_check_time = Some(SystemTime::now());
            }

            // Process alerts
            if let Err(e) = self.process_alerts().await {
                error!(error = %e, "Failed to process alerts");
            }

            // Auto-healing
            if self.config.auto_healing_enabled {
                if let Err(e) = self.perform_auto_healing().await {
                    error!(error = %e, "Auto-healing failed");
                }
            }

            // Sleep until next check
            let elapsed = start_time.elapsed();
            if elapsed < self.config.check_interval {
                sleep(self.config.check_interval - elapsed).await;
            }
        }
    }

    /// Check health for a specific target
    #[instrument(skip(self))]
    async fn check_target_health(&self, target: &MonitorTarget) -> Result<()> {
        let health_result = self.check_health(&target.health_check).await?;
        
        // Update target state
        {
            let mut state = self.state.write().await;
            if let Some(target_state) = state.target_states.get_mut(&target.name) {
                // Update consecutive failures
                if health_result.status == HealthStatus::Healthy {
                    target_state.consecutive_failures = 0;
                } else {
                    target_state.consecutive_failures += 1;
                }

                target_state.last_health_check = Some(health_result.clone());

                // Store health history (keep last 100 results)
                let history = state.health_history.entry(target.name.clone()).or_default();
                history.push(health_result.clone());
                if history.len() > 100 {
                    history.remove(0);
                }
            }
        }

        // Evaluate alert conditions
        for alert_rule in &target.alerts {
            if self.evaluate_alert_condition(&target.name, alert_rule, &health_result).await? {
                self.trigger_alert(&target.name, alert_rule.clone(), &health_result).await?;
            }
        }

        Ok(())
    }

    /// Execute a health check HTTP request
    async fn execute_health_check(&self, config: &HealthCheckConfig) -> Result<HealthCheckResult> {
        let start_time = Instant::now();
        
        // In a real implementation, this would make actual HTTP requests
        // For simulation, we'll create mock responses
        sleep(Duration::from_millis(50)).await; // Simulate network delay
        
        let response_time = start_time.elapsed();
        let status = if config.endpoint.contains("unhealthy") {
            HealthStatus::Unhealthy
        } else {
            HealthStatus::Healthy
        };

        Ok(HealthCheckResult {
            endpoint: config.endpoint.clone(),
            status,
            response_time,
            timestamp: SystemTime::now(),
            error_message: None,
            metadata: HashMap::new(),
        })
    }

    /// Evaluate if an alert condition is met
    async fn evaluate_alert_condition(
        &self,
        target_name: &str,
        rule: &AlertRule,
        health_result: &HealthCheckResult,
    ) -> Result<bool> {
        let state = self.state.read().await;
        let target_state = state.target_states.get(target_name)
            .ok_or_else(|| anyhow!("Target not found: {}", target_name))?;

        match &rule.condition {
            AlertCondition::HealthStatus(expected_status) => {
                Ok(health_result.status == *expected_status)
            }
            AlertCondition::ResponseTimeThreshold(threshold) => {
                Ok(health_result.response_time > *threshold)
            }
            AlertCondition::ErrorRateThreshold(_threshold) => {
                // Would calculate error rate from recent history
                Ok(false) // Simplified
            }
            AlertCondition::ConsecutiveFailures(threshold) => {
                Ok(target_state.consecutive_failures >= *threshold)
            }
            AlertCondition::CustomMetric { name, operator, threshold } => {
                if let Some(value) = target_state.metrics.get(name) {
                    match operator.as_str() {
                        ">" => Ok(*value > *threshold),
                        "<" => Ok(*value < *threshold),
                        ">=" => Ok(*value >= *threshold),
                        "<=" => Ok(*value <= *threshold),
                        "==" => Ok((*value - *threshold).abs() < f64::EPSILON),
                        _ => Ok(false),
                    }
                } else {
                    Ok(false)
                }
            }
        }
    }

    /// Trigger an alert
    #[instrument(skip(self))]
    async fn trigger_alert(
        &self,
        target_name: &str,
        rule: AlertRule,
        health_result: &HealthCheckResult,
    ) -> Result<()> {
        let alert_id = format!("{}_{}", target_name, uuid::Uuid::new_v4().simple());
        
        // Check cooldown
        let should_alert = {
            let state = self.state.read().await;
            if let Some(target_state) = state.target_states.get(target_name) {
                if let Some(last_alert_time) = target_state.last_alert_time {
                    SystemTime::now().duration_since(last_alert_time)
                        .unwrap_or_default() > rule.cooldown
                } else {
                    true
                }
            } else {
                true
            }
        };

        if !should_alert {
            return Ok(());
        }

        let alert = Alert {
            id: alert_id.clone(),
            target: target_name.to_string(),
            rule: rule.clone(),
            triggered_at: SystemTime::now(),
            acknowledged: false,
            resolved: false,
            resolution_time: None,
            metadata: health_result.metadata.clone(),
        };

        // Store alert
        {
            let mut state = self.state.write().await;
            state.active_alerts.insert(alert_id.clone(), alert.clone());
            
            if let Some(target_state) = state.target_states.get_mut(target_name) {
                target_state.last_alert_time = Some(SystemTime::now());
            }
        }

        // Send notifications
        self.alert_manager.send_alert_notification(&alert).await?;

        warn!(
            alert_id = %alert_id,
            target = %target_name,
            severity = ?rule.severity,
            condition = ?rule.condition,
            "Alert triggered"
        );

        Ok(())
    }

    /// Process and resolve alerts
    async fn process_alerts(&self) -> Result<()> {
        let mut alerts_to_resolve = Vec::new();
        
        {
            let state = self.state.read().await;
            for (alert_id, alert) in &state.active_alerts {
                if alert.resolved {
                    continue;
                }

                // Check if alert condition is no longer met
                if let Some(target_state) = state.target_states.get(&alert.target) {
                    if let Some(health_result) = &target_state.last_health_check {
                        let should_resolve = match &alert.rule.condition {
                            AlertCondition::HealthStatus(status) => {
                                health_result.status != *status
                            }
                            AlertCondition::ConsecutiveFailures(_) => {
                                target_state.consecutive_failures == 0
                            }
                            // Add other resolution conditions
                            _ => false,
                        };

                        if should_resolve {
                            alerts_to_resolve.push(alert_id.clone());
                        }
                    }
                }
            }
        }

        // Resolve alerts
        for alert_id in alerts_to_resolve {
            self.resolve_alert(&alert_id).await?;
        }

        Ok(())
    }

    /// Resolve an alert
    async fn resolve_alert(&self, alert_id: &str) -> Result<()> {
        let mut state = self.state.write().await;
        
        if let Some(alert) = state.active_alerts.get_mut(alert_id) {
            alert.resolved = true;
            alert.resolution_time = Some(SystemTime::now());
            
            info!(
                alert_id = %alert_id,
                target = %alert.target,
                "Alert resolved"
            );
        }

        Ok(())
    }

    /// Perform auto-healing actions
    async fn perform_auto_healing(&self) -> Result<()> {
        let targets_needing_healing = {
            let state = self.state.read().await;
            let mut targets = Vec::new();
            
            for (name, target_state) in &state.target_states {
                if target_state.consecutive_failures >= self.config.max_consecutive_failures {
                    if let Some(target_config) = self.config.targets.iter()
                        .find(|t| t.name == *name) {
                        if let Some(auto_healing) = &target_config.auto_healing {
                            if auto_healing.enabled {
                                targets.push((name.clone(), auto_healing.clone()));
                            }
                        }
                    }
                }
            }
            
            targets
        };

        for (target_name, auto_healing_config) in targets_needing_healing {
            self.execute_auto_healing(&target_name, &auto_healing_config).await?;
        }

        Ok(())
    }

    /// Execute auto-healing actions for a target
    #[instrument(skip(self))]
    async fn execute_auto_healing(
        &self,
        target_name: &str,
        config: &AutoHealingConfig,
    ) -> Result<()> {
        info!(
            target = %target_name,
            "Executing auto-healing"
        );

        // Check if we've exceeded max attempts
        let attempts = {
            let state = self.state.read().await;
            state.target_states.get(target_name)
                .map(|s| s.auto_healing_attempts)
                .unwrap_or(0)
        };

        if attempts >= config.max_attempts {
            warn!(
                target = %target_name,
                attempts = %attempts,
                max_attempts = %config.max_attempts,
                "Auto-healing max attempts reached"
            );
            return Ok(());
        }

        // Restart pods if configured
        if config.restart_pods {
            self.restart_target_pods(target_name).await?;
        }

        // Scale up if configured
        if let Some(scale_config) = &config.scale_up {
            self.scale_target(target_name, scale_config).await?;
        }

        // Failover if configured
        if let Some(failover_config) = &config.failover {
            if failover_config.automatic {
                self.initiate_failover(target_name, failover_config).await?;
            }
        }

        // Update attempt count
        {
            let mut state = self.state.write().await;
            if let Some(target_state) = state.target_states.get_mut(target_name) {
                target_state.auto_healing_attempts += 1;
            }
        }

        Ok(())
    }

    /// Restart pods for a target
    async fn restart_target_pods(&self, target_name: &str) -> Result<()> {
        info!(target = %target_name, "Restarting target pods");
        // In real implementation, would use Kubernetes API to restart pods
        sleep(Duration::from_secs(1)).await; // Simulate action
        Ok(())
    }

    /// Scale target deployment
    async fn scale_target(&self, target_name: &str, _config: &ScaleConfig) -> Result<()> {
        info!(target = %target_name, "Scaling target deployment");
        // In real implementation, would use Kubernetes API to scale deployment
        sleep(Duration::from_secs(1)).await; // Simulate action
        Ok(())
    }

    /// Initiate failover
    async fn initiate_failover(&self, target_name: &str, _config: &FailoverConfig) -> Result<()> {
        warn!(target = %target_name, "Initiating failover");
        // In real implementation, would trigger failover procedures
        sleep(Duration::from_secs(1)).await; // Simulate action
        Ok(())
    }

    /// Calculate overall health status
    fn calculate_overall_health(&self, state: &MonitorState) -> HealthStatus {
        let mut healthy_count = 0;
        let mut total_count = 0;

        for target_state in state.target_states.values() {
            total_count += 1;
            if let Some(health_result) = &target_state.last_health_check {
                if health_result.status == HealthStatus::Healthy {
                    healthy_count += 1;
                }
            }
        }

        if total_count == 0 {
            HealthStatus::Unknown
        } else if healthy_count == total_count {
            HealthStatus::Healthy
        } else if healthy_count > total_count / 2 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        }
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for HealthMonitorConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(30),
            alert_cooldown: Duration::from_secs(300),
            auto_healing_enabled: true,
            max_consecutive_failures: 3,
            escalation_threshold: 5,
            targets: Vec::new(),
        }
    }
}

impl AlertManager {
    pub fn new() -> Self {
        Self {
            notification_channels: HashMap::new(),
        }
    }

    async fn send_alert_notification(&self, _alert: &Alert) -> Result<()> {
        // In real implementation, would send notifications via configured channels
        info!("Sending alert notification");
        Ok(())
    }
}

/// Overall monitoring status
#[derive(Debug, Clone)]
pub struct MonitoringStatus {
    pub targets: HashMap<String, TargetState>,
    pub active_alerts: Vec<Alert>,
    pub last_check_time: Option<SystemTime>,
    pub overall_health: HealthStatus,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_monitor_creation() {
        let monitor = HealthMonitor::new();
        let status = monitor.get_status().await.unwrap();
        assert_eq!(status.overall_health, HealthStatus::Unknown);
    }

    #[tokio::test]
    async fn test_health_check() {
        let monitor = HealthMonitor::new();
        let config = HealthCheckConfig {
            endpoint: "http://test/health".to_string(),
            timeout: Duration::from_secs(5),
            retries: 1,
            interval: Duration::from_secs(1),
            expected_status: 200,
            critical: true,
        };

        let result = monitor.check_health(&config).await.unwrap();
        assert_eq!(result.endpoint, "http://test/health");
    }
}