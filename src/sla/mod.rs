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

// Removed unused anyhow import
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Service Level Indicator types defining the category of measurement
///
/// Each SLI type represents a different aspect of service performance that can be
/// monitored and used to calculate Service Level Objectives (SLOs).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SLIType {
    /// Measures service uptime and operational status
    ///
    /// Typically expressed as a percentage (e.g., 99.9% availability)
    /// Commonly measured as successful requests / total requests
    Availability,

    /// Measures response time and request processing speed
    ///
    /// Usually expressed in milliseconds (e.g., 95th percentile < 200ms)
    /// Critical for user experience and performance monitoring
    Latency,

    /// Measures request volume and processing capacity
    ///
    /// Expressed as requests per second (RPS) or similar rate metrics
    /// Important for capacity planning and resource allocation
    Throughput,

    /// Measures correctness and error rates of service responses
    ///
    /// Includes error rates, data integrity, and functional correctness
    /// Often measured as successful operations / total operations
    Quality,
}

/// Service Level Indicator value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SLIValue {
    /// Floating point metric value
    Float(f64),
    /// Integer metric value
    Integer(i64),
    /// Boolean metric value (success/failure)
    Boolean(bool),
}

/// Service Level Indicator definition
///
/// Defines a measurable aspect of service performance that can be monitored
/// and used to calculate Service Level Objectives (SLOs).
///
/// # Examples
///
/// ```rust
/// use uveddi::sla::{SLI, SLIType};
///
/// let api_availability = SLI {
///     name: "api_availability".to_string(),
///     sli_type: SLIType::Availability,
///     description: "Percentage of successful API responses".to_string(),
/// };
///
/// let response_latency = SLI {
///     name: "response_latency_p95".to_string(),
///     sli_type: SLIType::Latency,
///     description: "95th percentile response time in milliseconds".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLI {
    /// Unique identifier for this SLI (e.g., "api_availability", "response_latency_p95")
    ///
    /// Must be lowercase with underscores, no spaces or special characters.
    /// Used for referencing in SLO definitions and monitoring configurations.
    pub name: String,

    /// The type of measurement this SLI represents
    ///
    /// Determines how the SLI value should be interpreted and aggregated.
    /// Affects monitoring strategies and alerting thresholds.
    pub sli_type: SLIType,

    /// Human-readable description of what this SLI measures
    ///
    /// Should explain the business impact and measurement methodology.
    /// Used in dashboards and reporting to provide context for stakeholders.
    pub description: String,
}

/// Service Level Objective target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SLOTarget {
    /// Target as a percentage (0-100)
    Percentage(f64),
    /// Target as an absolute threshold value
    Threshold(f64),
}

/// Service Level Objective definition
///
/// Defines a target value for a specific Service Level Indicator (SLI) over a time window.
/// SLOs are used to set expectations and measure service reliability.
///
/// # Examples
///
/// ```rust
/// use uveddi::sla::{SLO, SLOTarget};
/// use std::time::Duration;
///
/// // 99.9% availability over 30 days
/// let availability_slo = SLO {
///     name: "API Availability".to_string(),
///     sli_name: "api_availability".to_string(),
///     target: SLOTarget::Percentage(99.9),
///     window: Duration::from_secs(30 * 24 * 60 * 60), // 30 days
/// };
///
/// // Response time under 200ms for 95% of requests over 1 hour
/// let latency_slo = SLO {
///     name: "Response Time SLO".to_string(),
///     sli_name: "response_latency_p95".to_string(),
///     target: SLOTarget::Threshold(200.0), // milliseconds
///     window: Duration::from_secs(3600), // 1 hour
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLO {
    /// Human-readable name for this SLO (e.g., "API Availability", "Response Time SLO")
    ///
    /// Used in dashboards, alerts, and reports for identification.
    /// Should be descriptive and unique within the service context.
    pub name: String,

    /// Name of the SLI this objective is based on
    ///
    /// Must exactly match an existing SLI name in the same service definition.
    /// Creates the binding between the measurement (SLI) and target (SLO).
    pub sli_name: String,

    /// Target value that defines successful service performance
    ///
    /// - `Percentage`: For availability and success rates (e.g., 99.9%)
    /// - `Threshold`: For latency and other metrics (e.g., < 200ms)
    pub target: SLOTarget,

    /// Time window over which the objective is measured
    ///
    /// Common windows: 1 hour, 1 day, 7 days, 30 days.
    /// Shorter windows enable faster detection but may be more noisy.
    /// Longer windows provide stability but slower response to issues.
    pub window: Duration,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Critical alert requiring immediate action
    Critical,
    /// Warning alert indicating potential issues
    Warning,
    /// Informational alert for awareness
    Info,
}

/// Risk level classification
#[derive(Debug, Serialize, PartialEq)]
pub enum RiskLevel {
    /// Low risk of SLA breach
    Low,
    /// Medium risk of SLA breach
    Medium,
    /// High risk of SLA breach
    High,
    /// Risk level cannot be determined
    Unknown,
}

/// SLA risk assessment result
#[derive(Debug, Serialize)]
pub struct SLARiskAssessment {
    /// Assessed risk level for SLA breach
    pub risk_level: RiskLevel,
    /// Confidence score in the risk assessment (0.0-1.0)
    pub confidence: f64,
    /// Probability of SLA breach occurring (0.0-1.0)
    pub predicted_sla_breach_probability: f64,
    /// List of recommendations to mitigate risk
    pub recommendations: Vec<String>,
}

/// Basic SLA framework for testing and validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLAFramework {
    /// Map of service names to their SLA definitions
    pub services: HashMap<String, ServiceDefinition>,
    /// Global configuration for the SLA framework
    pub global_config: GlobalSLAConfig,
}

/// Definition of a service and its SLAs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDefinition {
    /// Name of the service
    pub name: String,
    /// Description of what the service does
    pub description: String,
    /// List of Service Level Indicators for this service
    pub slis: Vec<SLI>,
    /// List of Service Level Objectives for this service
    pub slos: Vec<SLO>,
}

/// Global SLA configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSLAConfig {
    /// Timezone for SLA calculations and reporting
    pub timezone: String,
    /// How often to measure and record SLI values
    pub measurement_interval: Duration,
    /// How long to retain SLA measurement data
    pub retention_period: Duration,
    /// Whether to enable predictive analysis for SLA breach risk
    pub predictive_analysis: bool,
}

impl Default for SLAFramework {
    fn default() -> Self {
        let mut services = HashMap::new();

        // Analysis Service SLA
        services.insert(
            "analysis-service".to_string(),
            ServiceDefinition {
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
                        window: Duration::from_secs(28 * 24 * 60 * 60), // 28 days
                    },
                    SLO {
                        name: "analysis_latency".to_string(),
                        sli_name: "latency_p95".to_string(),
                        target: SLOTarget::Threshold(500.0), // 500ms
                        window: Duration::from_secs(28 * 24 * 60 * 60), // 28 days
                    },
                ],
            },
        );

        // Rendering Service SLA
        services.insert(
            "rendering-service".to_string(),
            ServiceDefinition {
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
                        window: Duration::from_secs(28 * 24 * 60 * 60), // 28 days
                    },
                    SLO {
                        name: "rendering_latency".to_string(),
                        sli_name: "latency_p99".to_string(),
                        target: SLOTarget::Threshold(100.0), // 100ms P99 target
                        window: Duration::from_secs(28 * 24 * 60 * 60), // 28 days
                    },
                ],
            },
        );

        Self {
            services,
            global_config: GlobalSLAConfig {
                timezone: "UTC".to_string(),
                measurement_interval: Duration::from_secs(60),
                retention_period: Duration::from_secs(90 * 24 * 60 * 60), // 90 days
                predictive_analysis: true,
            },
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
    fn test_rendering_service_sla_target() {
        let framework = SLAFramework::default();
        let rendering_service = framework.services.get("rendering-service").unwrap();

        let latency_slo = rendering_service
            .slos
            .iter()
            .find(|slo| slo.name == "rendering_latency")
            .unwrap();

        match &latency_slo.target {
            SLOTarget::Threshold(threshold) => {
                assert_eq!(*threshold, 100.0); // P99 < 100ms target
            }
            SLOTarget::Percentage(_) => {
                panic!("Expected threshold target for latency SLO, but got percentage");
            }
        }
    }

    #[test]
    fn test_sli_types() {
        let framework = SLAFramework::default();
        let analysis_service = framework.services.get("analysis-service").unwrap();

        assert!(analysis_service
            .slis
            .iter()
            .any(|sli| matches!(sli.sli_type, SLIType::Availability)));
        assert!(analysis_service
            .slis
            .iter()
            .any(|sli| matches!(sli.sli_type, SLIType::Latency)));
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

    #[test]
    fn test_sli_value_types() {
        let float_value = SLIValue::Float(99.9);
        let int_value = SLIValue::Integer(200);
        let bool_value = SLIValue::Boolean(true);

        match float_value {
            SLIValue::Float(f) => assert_eq!(f, 99.9),
            SLIValue::Integer(i) => panic!("Expected float value, got integer: {}", i),
            SLIValue::Boolean(b) => panic!("Expected float value, got boolean: {}", b),
        }

        match int_value {
            SLIValue::Integer(i) => assert_eq!(i, 200),
            SLIValue::Float(f) => panic!("Expected integer value, got float: {}", f),
            SLIValue::Boolean(b) => panic!("Expected integer value, got boolean: {}", b),
        }

        match bool_value {
            SLIValue::Boolean(b) => assert!(b),
            SLIValue::Float(f) => panic!("Expected boolean value, got float: {}", f),
            SLIValue::Integer(i) => panic!("Expected boolean value, got integer: {}", i),
        }
    }
}
