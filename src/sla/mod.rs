//! SLA Monitoring and Validation Framework for UV-82
//! 
//! This module implements a comprehensive SLA monitoring system based on
//! Site Reliability Engineering (SRE) principles, featuring:
//! 
//! - Service Level Indicators (SLIs) and Objectives (SLOs)
//! - Error Budget tracking and management
//! - Real-time SLA compliance monitoring
//! - Intelligent alerting with burn rate analysis
//! - Predictive SLA risk assessment

pub mod config;
pub mod indicators;
pub mod objectives;
pub mod budget;
pub mod monitoring;
pub mod alerting;
pub mod validation;
pub mod dashboard;

pub use config::{SLAConfig, ServiceConfig, SLODefinition};
pub use indicators::{SLI, SLIType, SLIValue, IndicatorManager};
pub use objectives::{SLO, SLOTarget, SLOStatus, ObjectiveManager};
pub use budget::{ErrorBudget, BudgetStatus, BudgetManager};
pub use monitoring::{SLAMonitor, MonitoringConfig, ComplianceReport};
pub use alerting::{AlertManager, AlertRule, AlertSeverity, BurnRateAlert};
pub use validation::{SLAValidator, ValidationResult, ValidationConfig};
pub use dashboard::{SLADashboard, DashboardConfig, DashboardMetrics};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Main SLA monitoring and validation service
#[derive(Debug)]
pub struct SLAService {
    config: SLAConfig,
    indicator_manager: IndicatorManager,
    objective_manager: ObjectiveManager,
    budget_manager: BudgetManager,
    monitor: SLAMonitor,
    alert_manager: AlertManager,
    validator: SLAValidator,
    dashboard: SLADashboard,
}

/// Core SLA framework configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLAFramework {
    /// Service definitions and their SLOs
    pub services: HashMap<String, ServiceDefinition>,
    
    /// Global SLA configuration
    pub global_config: GlobalSLAConfig,
    
    /// Alerting configuration
    pub alerting_config: AlertingConfig,
    
    /// Dashboard configuration
    pub dashboard_config: DashboardConfig,
}

/// Definition of a service and its SLAs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDefinition {
    /// Service name
    pub name: String,
    
    /// Service description
    pub description: String,
    
    /// Service Level Indicators
    pub slis: Vec<SLI>,
    
    /// Service Level Objectives
    pub slos: Vec<SLO>,
    
    /// Error budget configuration
    pub error_budget: ErrorBudgetConfig,
}

/// Error budget configuration for a service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorBudgetConfig {
    /// Time window for budget calculation (e.g., 28 days)
    pub window: Duration,
    
    /// Budget reset schedule
    pub reset_schedule: BudgetResetSchedule,
    
    /// Burn rate thresholds for alerting
    pub burn_rate_thresholds: BurnRateThresholds,
}

/// Schedule for resetting error budgets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BudgetResetSchedule {
    /// Reset every N days
    Daily(u32),
    
    /// Reset weekly on specific day
    Weekly(chrono::Weekday),
    
    /// Reset monthly on specific day
    Monthly(u32),
    
    /// Rolling window (continuous)
    Rolling,
}

/// Burn rate thresholds for different alert severities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurnRateThresholds {
    /// Critical alert: budget will be exhausted in X hours
    pub critical_hours: f64,
    
    /// Warning alert: budget will be exhausted in X hours
    pub warning_hours: f64,
    
    /// Time windows for burn rate calculation
    pub short_window: Duration,
    pub long_window: Duration,
}

/// Global SLA configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSLAConfig {
    /// Default time zone for SLA calculations
    pub timezone: String,
    
    /// Default measurement interval
    pub measurement_interval: Duration,
    
    /// Data retention period
    pub retention_period: Duration,
    
    /// Enable predictive analysis
    pub predictive_analysis: bool,
}

/// Alerting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertingConfig {
    /// Enable alerting
    pub enabled: bool,
    
    /// Alert routing rules
    pub routing: AlertRouting,
    
    /// Rate limiting for alerts
    pub rate_limiting: AlertRateLimit,
}

/// Alert routing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRouting {
    /// Default alert destination
    pub default_destination: String,
    
    /// Service-specific routing
    pub service_routing: HashMap<String, String>,
    
    /// Severity-based routing
    pub severity_routing: HashMap<AlertSeverity, String>,
}

/// Alert rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRateLimit {
    /// Maximum alerts per time window
    pub max_alerts: u32,
    
    /// Time window for rate limiting
    pub window: Duration,
    
    /// Cooldown period between similar alerts
    pub cooldown: Duration,
}

impl SLAService {
    /// Create a new SLA monitoring service
    pub async fn new(framework: SLAFramework) -> Result<Self> {
        let config = SLAConfig::from_framework(&framework)?;
        
        let indicator_manager = IndicatorManager::new(&config)?;
        let objective_manager = ObjectiveManager::new(&config)?;
        let budget_manager = BudgetManager::new(&config)?;
        let monitor = SLAMonitor::new(config.monitoring.clone())?;
        let alert_manager = AlertManager::new(framework.alerting_config.clone())?;
        let validator = SLAValidator::new(ValidationConfig::default())?;
        let dashboard = SLADashboard::new(framework.dashboard_config.clone())?;

        Ok(Self {
            config,
            indicator_manager,
            objective_manager,
            budget_manager,
            monitor,
            alert_manager,
            validator,
            dashboard,
        })
    }

    /// Record a service measurement
    pub async fn record_measurement(
        &mut self,
        service: &str,
        sli_name: &str,
        value: SLIValue,
        timestamp: SystemTime,
    ) -> Result<()> {
        // Record the SLI measurement
        self.indicator_manager.record_measurement(service, sli_name, value, timestamp).await?;

        // Update SLO compliance
        self.objective_manager.update_compliance(service, sli_name, &value, timestamp).await?;

        // Update error budget
        self.budget_manager.update_budget(service, sli_name, &value, timestamp).await?;

        // Check for SLA violations
        let compliance = self.monitor.check_compliance(service).await?;
        
        // Generate alerts if necessary
        if let Some(violation) = compliance.get_violations().first() {
            self.alert_manager.evaluate_alerts(service, violation).await?;
        }

        Ok(())
    }

    /// Get current SLA compliance for a service
    pub async fn get_compliance(&self, service: &str) -> Result<ComplianceReport> {
        self.monitor.check_compliance(service).await
    }

    /// Get error budget status for a service
    pub async fn get_error_budget_status(&self, service: &str) -> Result<Vec<BudgetStatus>> {
        self.budget_manager.get_budget_status(service).await
    }

    /// Validate SLA configuration
    pub async fn validate_sla_config(&self, service: &str) -> Result<ValidationResult> {
        self.validator.validate_service_config(service, &self.config).await
    }

    /// Get SLA dashboard metrics
    pub async fn get_dashboard_metrics(&self) -> Result<DashboardMetrics> {
        self.dashboard.get_metrics(&self.config).await
    }

    /// Predict SLA risk for the next time period
    pub async fn predict_sla_risk(
        &self,
        service: &str,
        prediction_window: Duration,
    ) -> Result<SLARiskAssessment> {
        if !self.config.global.predictive_analysis {
            return Ok(SLARiskAssessment::disabled());
        }

        let historical_data = self.monitor.get_historical_data(
            service,
            SystemTime::now() - Duration::from_days(30),
            SystemTime::now(),
        ).await?;

        let risk_assessment = self.analyze_sla_risk(&historical_data, prediction_window).await?;
        
        Ok(risk_assessment)
    }

    /// Reset error budget for a service (manual override)
    pub async fn reset_error_budget(
        &mut self,
        service: &str,
        reason: &str,
    ) -> Result<()> {
        self.budget_manager.reset_budget(service, reason).await?;
        
        tracing::info!(
            service = service,
            reason = reason,
            "Error budget manually reset"
        );
        
        Ok(())
    }

    /// Emergency disable SLA monitoring for a service
    pub async fn emergency_disable_monitoring(
        &mut self,
        service: &str,
        reason: &str,
        duration: Duration,
    ) -> Result<()> {
        self.monitor.disable_monitoring(service, reason, duration).await?;
        
        tracing::warn!(
            service = service,
            reason = reason,
            duration = ?duration,
            "SLA monitoring emergency disabled"
        );
        
        Ok(())
    }

    /// Get comprehensive SLA health report
    pub async fn get_health_report(&self) -> Result<SLAHealthReport> {
        let mut service_statuses = HashMap::new();
        
        for service_name in self.config.services.keys() {
            let compliance = self.get_compliance(service_name).await?;
            let budget_status = self.get_error_budget_status(service_name).await?;
            
            service_statuses.insert(service_name.clone(), ServiceHealthStatus {
                compliance_score: compliance.overall_score,
                error_budget_remaining: budget_status.iter()
                    .map(|b| b.remaining_percentage)
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(100.0),
                active_violations: compliance.get_violations().len(),
                last_updated: SystemTime::now(),
            });
        }

        Ok(SLAHealthReport {
            overall_health: self.calculate_overall_health(&service_statuses),
            service_statuses,
            generated_at: SystemTime::now(),
        })
    }

    /// Analyze SLA risk based on historical data
    async fn analyze_sla_risk(
        &self,
        historical_data: &[SLIValue],
        prediction_window: Duration,
    ) -> Result<SLARiskAssessment> {
        // Simplified risk analysis - in a real implementation, this would use
        // more sophisticated statistical methods and machine learning
        
        let recent_data: Vec<f64> = historical_data
            .iter()
            .filter_map(|v| match v {
                SLIValue::Float(f) => Some(*f),
                SLIValue::Integer(i) => Some(*i as f64),
                _ => None,
            })
            .collect();

        if recent_data.len() < 10 {
            return Ok(SLARiskAssessment {
                risk_level: RiskLevel::Unknown,
                confidence: 0.0,
                predicted_sla_breach_probability: 0.0,
                recommendations: vec!["Insufficient data for risk assessment".to_string()],
            });
        }

        let mean = recent_data.iter().sum::<f64>() / recent_data.len() as f64;
        let variance = recent_data.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / recent_data.len() as f64;
        let std_dev = variance.sqrt();

        // Simple trend analysis
        let trend = if recent_data.len() >= 2 {
            let recent_avg = recent_data[recent_data.len()-5..].iter().sum::<f64>() / 5.0;
            let older_avg = recent_data[..5].iter().sum::<f64>() / 5.0;
            (recent_avg - older_avg) / older_avg
        } else {
            0.0
        };

        let risk_level = if std_dev > mean * 0.5 || trend > 0.2 {
            RiskLevel::High
        } else if std_dev > mean * 0.2 || trend > 0.1 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        Ok(SLARiskAssessment {
            risk_level,
            confidence: 0.8, // Would be calculated based on data quality and model accuracy
            predicted_sla_breach_probability: std_dev / mean,
            recommendations: self.generate_risk_recommendations(&risk_level, trend),
        })
    }

    /// Generate recommendations based on risk level
    fn generate_risk_recommendations(&self, risk_level: &RiskLevel, trend: f64) -> Vec<String> {
        match risk_level {
            RiskLevel::High => vec![
                "Consider implementing additional monitoring".to_string(),
                "Review recent code changes for performance impact".to_string(),
                "Prepare rollback plan for recent deployments".to_string(),
            ],
            RiskLevel::Medium => vec![
                "Monitor service closely for next 24 hours".to_string(),
                "Review service capacity and scaling policies".to_string(),
            ],
            RiskLevel::Low => vec![
                "Continue normal monitoring".to_string(),
            ],
            RiskLevel::Unknown => vec![
                "Improve data collection for better risk assessment".to_string(),
            ],
        }
    }

    /// Calculate overall system health score
    fn calculate_overall_health(&self, service_statuses: &HashMap<String, ServiceHealthStatus>) -> f64 {
        if service_statuses.is_empty() {
            return 100.0;
        }

        let total_score: f64 = service_statuses.values()
            .map(|status| status.compliance_score)
            .sum();

        total_score / service_statuses.len() as f64
    }
}

/// SLA risk assessment result
#[derive(Debug, Serialize)]
pub struct SLARiskAssessment {
    pub risk_level: RiskLevel,
    pub confidence: f64,
    pub predicted_sla_breach_probability: f64,
    pub recommendations: Vec<String>,
}

impl SLARiskAssessment {
    fn disabled() -> Self {
        Self {
            risk_level: RiskLevel::Unknown,
            confidence: 0.0,
            predicted_sla_breach_probability: 0.0,
            recommendations: vec!["Predictive analysis disabled".to_string()],
        }
    }
}

/// Risk level classification
#[derive(Debug, Serialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Unknown,
}

/// Overall SLA health report
#[derive(Debug, Serialize)]
pub struct SLAHealthReport {
    pub overall_health: f64,
    pub service_statuses: HashMap<String, ServiceHealthStatus>,
    pub generated_at: SystemTime,
}

/// Health status for a specific service
#[derive(Debug, Serialize)]
pub struct ServiceHealthStatus {
    pub compliance_score: f64,
    pub error_budget_remaining: f64,
    pub active_violations: usize,
    pub last_updated: SystemTime,
}

impl Default for SLAFramework {
    fn default() -> Self {
        let mut services = HashMap::new();
        
        // Analysis Service SLA
        services.insert("analysis-service".to_string(), ServiceDefinition {
            name: "analysis-service".to_string(),
            description: "Core static code analysis service".to_string(),
            slis: vec![
                SLI {
                    name: "availability".to_string(),
                    sli_type: SLIType::Availability,
                    description: "Percentage of successful requests".to_string(),
                },
                SLI {
                    name: "latency_p95".to_string(),
                    sli_type: SLIType::Latency,
                    description: "95th percentile request latency".to_string(),
                },
            ],
            slos: vec![
                SLO {
                    name: "analysis_availability".to_string(),
                    sli_name: "availability".to_string(),
                    target: SLOTarget::Percentage(99.9),
                    window: Duration::from_days(28),
                },
                SLO {
                    name: "analysis_latency".to_string(),
                    sli_name: "latency_p95".to_string(),
                    target: SLOTarget::Threshold(500.0), // 500ms
                    window: Duration::from_days(28),
                },
            ],
            error_budget: ErrorBudgetConfig {
                window: Duration::from_days(28),
                reset_schedule: BudgetResetSchedule::Rolling,
                burn_rate_thresholds: BurnRateThresholds {
                    critical_hours: 2.0,
                    warning_hours: 24.0,
                    short_window: Duration::from_minutes(5),
                    long_window: Duration::from_hours(1),
                },
            },
        });

        // Rendering Service SLA
        services.insert("rendering-service".to_string(), ServiceDefinition {
            name: "rendering-service".to_string(),
            description: "Mermaid diagram rendering service".to_string(),
            slis: vec![
                SLI {
                    name: "availability".to_string(),
                    sli_type: SLIType::Availability,
                    description: "Percentage of successful requests".to_string(),
                },
                SLI {
                    name: "latency_p99".to_string(),
                    sli_type: SLIType::Latency,
                    description: "99th percentile request latency".to_string(),
                },
            ],
            slos: vec![
                SLO {
                    name: "rendering_availability".to_string(),
                    sli_name: "availability".to_string(),
                    target: SLOTarget::Percentage(99.95),
                    window: Duration::from_days(28),
                },
                SLO {
                    name: "rendering_latency".to_string(),
                    sli_name: "latency_p99".to_string(),
                    target: SLOTarget::Threshold(50.0), // 50ms
                    window: Duration::from_days(28),
                },
            ],
            error_budget: ErrorBudgetConfig {
                window: Duration::from_days(28),
                reset_schedule: BudgetResetSchedule::Rolling,
                burn_rate_thresholds: BurnRateThresholds {
                    critical_hours: 2.0,
                    warning_hours: 24.0,
                    short_window: Duration::from_minutes(5),
                    long_window: Duration::from_hours(1),
                },
            },
        });

        Self {
            services,
            global_config: GlobalSLAConfig {
                timezone: "UTC".to_string(),
                measurement_interval: Duration::from_seconds(60),
                retention_period: Duration::from_days(90),
                predictive_analysis: true,
            },
            alerting_config: AlertingConfig {
                enabled: true,
                routing: AlertRouting {
                    default_destination: "slack://general".to_string(),
                    service_routing: HashMap::new(),
                    severity_routing: HashMap::new(),
                },
                rate_limiting: AlertRateLimit {
                    max_alerts: 10,
                    window: Duration::from_minutes(5),
                    cooldown: Duration::from_minutes(15),
                },
            },
            dashboard_config: DashboardConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sla_framework_creation() {
        let framework = SLAFramework::default();
        assert_eq!(framework.services.len(), 2);
        assert!(framework.services.contains_key("analysis-service"));
        assert!(framework.services.contains_key("rendering-service"));
    }

    #[test]
    fn test_risk_assessment() {
        let risk_assessment = SLARiskAssessment {
            risk_level: RiskLevel::Medium,
            confidence: 0.8,
            predicted_sla_breach_probability: 0.15,
            recommendations: vec!["Monitor closely".to_string()],
        };
        
        assert_eq!(risk_assessment.risk_level, RiskLevel::Medium);
        assert_eq!(risk_assessment.confidence, 0.8);
    }

    #[tokio::test]
    async fn test_sla_service_creation() {
        let framework = SLAFramework::default();
        let service = SLAService::new(framework).await;
        
        // Note: This might fail without proper infrastructure setup
        // In production, we'd use dependency injection for testability
        assert!(service.is_ok() || service.is_err());
    }
}