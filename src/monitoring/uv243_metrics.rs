use prometheus::{Counter, Gauge, Histogram, Registry, IntCounter, IntGauge};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{Duration, Instant};
use tracing::{info, warn, error};

/// UV-243 Quality Metrics Collection System
/// 
/// Provides comprehensive monitoring of testing, documentation, and deployment quality
/// metrics to ensure continuous validation of UV-243 requirements.
#[derive(Clone)]
pub struct UV243Metrics {
    // Test Coverage Metrics
    pub test_coverage: Gauge,
    pub test_execution_time: Histogram,
    pub test_failure_count: IntCounter,
    pub test_success_count: IntCounter,
    
    // Documentation Metrics
    pub documentation_freshness: Gauge,
    pub documentation_build_time: Histogram,
    pub documentation_errors: IntCounter,
    pub api_documentation_coverage: Gauge,
    
    // Performance Metrics
    pub performance_regression_count: IntCounter,
    pub benchmark_execution_time: Histogram,
    pub memory_usage_peak: Gauge,
    pub throughput_metrics_per_second: Gauge,
    
    // Security Metrics
    pub security_vulnerability_count: IntCounter,
    pub security_audit_duration: Histogram,
    pub security_scan_success: IntCounter,
    
    // Deployment Metrics
    pub deployment_success_rate: Gauge,
    pub deployment_duration: Histogram,
    pub rollback_count: IntCounter,
    pub health_check_failures: IntCounter,
    
    // Quality Gate Metrics
    pub quality_gate_pass_rate: Gauge,
    pub quality_gate_violations: IntCounter,
    pub coverage_threshold_violations: IntCounter,
    
    // System Health Metrics
    pub system_uptime: Gauge,
    pub error_rate: Gauge,
    pub response_time_p95: Gauge,
}

impl UV243Metrics {
    /// Create a new UV243Metrics instance with all metrics registered
    pub fn new(registry: &Registry) -> Result<Self, Box<dyn std::error::Error>> {
        // Test Coverage Metrics
        let test_coverage = Gauge::new(
            "uv243_test_coverage_percentage",
            "Current test coverage percentage for UV-243 requirements"
        )?;
        
        let test_execution_time = Histogram::new(
            "uv243_test_execution_seconds",
            "Time taken to execute comprehensive test suite"
        )?;
        
        let test_failure_count = IntCounter::new(
            "uv243_test_failures_total",
            "Total number of test failures detected"
        )?;
        
        let test_success_count = IntCounter::new(
            "uv243_test_successes_total",
            "Total number of successful test runs"
        )?;
        
        // Documentation Metrics
        let documentation_freshness = Gauge::new(
            "uv243_documentation_age_days",
            "Days since documentation was last updated"
        )?;
        
        let documentation_build_time = Histogram::new(
            "uv243_documentation_build_seconds",
            "Time taken to build all documentation"
        )?;
        
        let documentation_errors = IntCounter::new(
            "uv243_documentation_errors_total",
            "Total number of documentation build errors"
        )?;
        
        let api_documentation_coverage = Gauge::new(
            "uv243_api_documentation_coverage_percentage",
            "Percentage of API endpoints with documentation"
        )?;
        
        // Performance Metrics
        let performance_regression_count = IntCounter::new(
            "uv243_performance_regressions_total",
            "Total number of performance regressions detected"
        )?;
        
        let benchmark_execution_time = Histogram::new(
            "uv243_benchmark_execution_seconds",
            "Time taken to execute performance benchmarks"
        )?;
        
        let memory_usage_peak = Gauge::new(
            "uv243_memory_usage_peak_bytes",
            "Peak memory usage during testing and analysis"
        )?;
        
        let throughput_metrics_per_second = Gauge::new(
            "uv243_throughput_metrics_per_second",
            "Current metrics processing throughput (target: 4.3M+/sec)"
        )?;
        
        // Security Metrics
        let security_vulnerability_count = IntCounter::new(
            "uv243_security_vulnerabilities_total",
            "Total number of security vulnerabilities found"
        )?;
        
        let security_audit_duration = Histogram::new(
            "uv243_security_audit_seconds",
            "Time taken to complete security audit"
        )?;
        
        let security_scan_success = IntCounter::new(
            "uv243_security_scans_successful_total",
            "Total number of successful security scans"
        )?;
        
        // Deployment Metrics
        let deployment_success_rate = Gauge::new(
            "uv243_deployment_success_rate",
            "Success rate of production deployments (0.0-1.0)"
        )?;
        
        let deployment_duration = Histogram::new(
            "uv243_deployment_duration_seconds",
            "Time taken for complete deployment process"
        )?;
        
        let rollback_count = IntCounter::new(
            "uv243_rollbacks_total",
            "Total number of deployment rollbacks"
        )?;
        
        let health_check_failures = IntCounter::new(
            "uv243_health_check_failures_total",
            "Total number of health check failures"
        )?;
        
        // Quality Gate Metrics
        let quality_gate_pass_rate = Gauge::new(
            "uv243_quality_gate_pass_rate",
            "Percentage of quality gate validations that pass"
        )?;
        
        let quality_gate_violations = IntCounter::new(
            "uv243_quality_gate_violations_total",
            "Total number of quality gate violations"
        )?;
        
        let coverage_threshold_violations = IntCounter::new(
            "uv243_coverage_threshold_violations_total",
            "Total number of times coverage fell below 90% threshold"
        )?;
        
        // System Health Metrics
        let system_uptime = Gauge::new(
            "uv243_system_uptime_seconds",
            "System uptime in seconds"
        )?;
        
        let error_rate = Gauge::new(
            "uv243_error_rate",
            "Current system error rate (0.0-1.0)"
        )?;
        
        let response_time_p95 = Gauge::new(
            "uv243_response_time_p95_milliseconds",
            "95th percentile response time in milliseconds"
        )?;
        
        // Register all metrics
        registry.register(Box::new(test_coverage.clone()))?;
        registry.register(Box::new(test_execution_time.clone()))?;
        registry.register(Box::new(test_failure_count.clone()))?;
        registry.register(Box::new(test_success_count.clone()))?;
        
        registry.register(Box::new(documentation_freshness.clone()))?;
        registry.register(Box::new(documentation_build_time.clone()))?;
        registry.register(Box::new(documentation_errors.clone()))?;
        registry.register(Box::new(api_documentation_coverage.clone()))?;
        
        registry.register(Box::new(performance_regression_count.clone()))?;
        registry.register(Box::new(benchmark_execution_time.clone()))?;
        registry.register(Box::new(memory_usage_peak.clone()))?;
        registry.register(Box::new(throughput_metrics_per_second.clone()))?;
        
        registry.register(Box::new(security_vulnerability_count.clone()))?;
        registry.register(Box::new(security_audit_duration.clone()))?;
        registry.register(Box::new(security_scan_success.clone()))?;
        
        registry.register(Box::new(deployment_success_rate.clone()))?;
        registry.register(Box::new(deployment_duration.clone()))?;
        registry.register(Box::new(rollback_count.clone()))?;
        registry.register(Box::new(health_check_failures.clone()))?;
        
        registry.register(Box::new(quality_gate_pass_rate.clone()))?;
        registry.register(Box::new(quality_gate_violations.clone()))?;
        registry.register(Box::new(coverage_threshold_violations.clone()))?;
        
        registry.register(Box::new(system_uptime.clone()))?;
        registry.register(Box::new(error_rate.clone()))?;
        registry.register(Box::new(response_time_p95.clone()))?;
        
        Ok(Self {
            test_coverage,
            test_execution_time,
            test_failure_count,
            test_success_count,
            documentation_freshness,
            documentation_build_time,
            documentation_errors,
            api_documentation_coverage,
            performance_regression_count,
            benchmark_execution_time,
            memory_usage_peak,
            throughput_metrics_per_second,
            security_vulnerability_count,
            security_audit_duration,
            security_scan_success,
            deployment_success_rate,
            deployment_duration,
            rollback_count,
            health_check_failures,
            quality_gate_pass_rate,
            quality_gate_violations,
            coverage_threshold_violations,
            system_uptime,
            error_rate,
            response_time_p95,
        })
    }
    
    /// Update test coverage metrics
    pub fn update_coverage(&self, coverage: f64) {
        self.test_coverage.set(coverage);
        
        // Check if coverage meets UV-243 requirement (≥90%)
        if coverage < 90.0 {
            self.coverage_threshold_violations.inc();
            warn!("Coverage {} below 90% threshold", coverage);
        } else {
            info!("Coverage {} meets UV-243 requirement", coverage);
        }
    }
    
    /// Record test execution metrics
    pub fn record_test_execution(&self, duration: Duration, success: bool) {
        self.test_execution_time.observe(duration.as_secs_f64());
        
        if success {
            self.test_success_count.inc();
        } else {
            self.test_failure_count.inc();
        }
    }
    
    /// Record documentation build metrics
    pub fn record_documentation_build(&self, duration: Duration, success: bool) {
        self.documentation_build_time.observe(duration.as_secs_f64());
        
        if !success {
            self.documentation_errors.inc();
        }
    }
    
    /// Record performance benchmark results
    pub fn record_benchmark_execution(&self, duration: Duration, throughput: f64) {
        self.benchmark_execution_time.observe(duration.as_secs_f64());
        self.throughput_metrics_per_second.set(throughput);
        
        // Check if throughput meets UV-243 requirement (4.3M+ metrics/sec)
        if throughput < 4_300_000.0 {
            self.performance_regression_count.inc();
            warn!("Throughput {} below 4.3M metrics/sec requirement", throughput);
        }
    }
    
    /// Record security audit results
    pub fn record_security_audit(&self, duration: Duration, vulnerabilities: u64) {
        self.security_audit_duration.observe(duration.as_secs_f64());
        
        if vulnerabilities > 0 {
            self.security_vulnerability_count.inc_by(vulnerabilities);
            error!("Security audit found {} vulnerabilities", vulnerabilities);
        } else {
            self.security_scan_success.inc();
            info!("Security audit completed successfully");
        }
    }
    
    /// Record deployment metrics
    pub fn record_deployment(&self, duration: Duration, success: bool) {
        self.deployment_duration.observe(duration.as_secs_f64());
        
        if !success {
            self.rollback_count.inc();
        }
        
        // Update deployment success rate (simplified calculation)
        let total_deployments = self.deployment_duration.get_sample_count();
        let successful_deployments = total_deployments - self.rollback_count.get() as u64;
        let success_rate = if total_deployments > 0 {
            successful_deployments as f64 / total_deployments as f64
        } else {
            1.0
        };
        self.deployment_success_rate.set(success_rate);
    }
    
    /// Record quality gate validation
    pub fn record_quality_gate(&self, passed: bool) {
        if !passed {
            self.quality_gate_violations.inc();
        }
        
        // Update quality gate pass rate
        let total_validations = self.quality_gate_violations.get() + 1; // Simplified
        let violations = self.quality_gate_violations.get();
        let pass_rate = if total_validations > 0 {
            (total_validations - violations) as f64 / total_validations as f64
        } else {
            1.0
        };
        self.quality_gate_pass_rate.set(pass_rate);
    }
    
    /// Update system health metrics
    pub fn update_system_health(&self, uptime: Duration, error_rate: f64, response_time_p95: f64) {
        self.system_uptime.set(uptime.as_secs_f64());
        self.error_rate.set(error_rate);
        self.response_time_p95.set(response_time_p95);
    }
    
    /// Get comprehensive metrics summary for reporting
    pub fn get_metrics_summary(&self) -> UV243MetricsSummary {
        UV243MetricsSummary {
            test_coverage: self.test_coverage.get(),
            test_success_rate: {
                let total = self.test_success_count.get() + self.test_failure_count.get();
                if total > 0 {
                    self.test_success_count.get() as f64 / total as f64
                } else {
                    1.0
                }
            },
            documentation_freshness_days: self.documentation_freshness.get(),
            performance_throughput: self.throughput_metrics_per_second.get(),
            security_vulnerabilities: self.security_vulnerability_count.get(),
            deployment_success_rate: self.deployment_success_rate.get(),
            quality_gate_pass_rate: self.quality_gate_pass_rate.get(),
            system_uptime_hours: self.system_uptime.get() / 3600.0,
            error_rate: self.error_rate.get(),
        }
    }

    /// Generate comprehensive quality report for dashboards
    pub fn generate_quality_report(&self) -> QualityReport {
        let summary = self.get_metrics_summary();
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        QualityReport {
            timestamp,
            overall_score: self.calculate_overall_quality_score(),
            summary,
            quality_gates: QualityGatesStatus {
                coverage_gate: summary.test_coverage >= 90.0,
                security_gate: summary.security_vulnerabilities == 0,
                performance_gate: summary.performance_throughput >= 4_300_000.0,
                documentation_gate: summary.documentation_freshness_days <= 7.0,
                deployment_gate: summary.deployment_success_rate >= 0.95,
            },
            alerts: self.generate_alerts(),
        }
    }

    /// Calculate overall quality score (0-100)
    fn calculate_overall_quality_score(&self) -> f64 {
        let summary = self.get_metrics_summary();
        let weights = [
            (summary.test_coverage / 100.0, 25.0),           // 25% weight
            (summary.test_success_rate, 20.0),               // 20% weight
            ((1.0 - (summary.documentation_freshness_days / 30.0).min(1.0)), 15.0), // 15% weight
            ((summary.performance_throughput / 10_000_000.0).min(1.0), 20.0), // 20% weight
            (if summary.security_vulnerabilities == 0 { 1.0 } else { 0.0 }, 15.0), // 15% weight
            (summary.deployment_success_rate, 5.0),          // 5% weight
        ];

        let weighted_sum: f64 = weights.iter().map(|(score, weight)| score * weight).sum();
        let total_weight: f64 = weights.iter().map(|(_, weight)| weight).sum();
        
        (weighted_sum / total_weight * 100.0).min(100.0)
    }

    /// Generate alerts based on current metrics
    fn generate_alerts(&self) -> Vec<QualityAlert> {
        let mut alerts = Vec::new();
        let summary = self.get_metrics_summary();

        if summary.test_coverage < 90.0 {
            alerts.push(QualityAlert {
                severity: AlertSeverity::Critical,
                component: "Test Coverage".to_string(),
                message: format!("Coverage at {}%, below 90% requirement", summary.test_coverage),
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            });
        }

        if summary.security_vulnerabilities > 0 {
            alerts.push(QualityAlert {
                severity: AlertSeverity::High,
                component: "Security".to_string(),
                message: format!("{} security vulnerabilities detected", summary.security_vulnerabilities),
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            });
        }

        if summary.performance_throughput < 4_300_000.0 {
            alerts.push(QualityAlert {
                severity: AlertSeverity::Medium,
                component: "Performance".to_string(),
                message: format!("Throughput at {}/sec, below 4.3M requirement", summary.performance_throughput),
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            });
        }

        if summary.documentation_freshness_days > 7.0 {
            alerts.push(QualityAlert {
                severity: AlertSeverity::Medium,
                component: "Documentation".to_string(),
                message: format!("Documentation {} days old, exceeds 7-day limit", summary.documentation_freshness_days),
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            });
        }

        if summary.deployment_success_rate < 0.95 {
            alerts.push(QualityAlert {
                severity: AlertSeverity::High,
                component: "Deployment".to_string(),
                message: format!("Deployment success rate at {}%, below 95%", summary.deployment_success_rate * 100.0),
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            });
        }

        alerts
    }
}

/// Summary of UV-243 metrics for reporting and alerting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UV243MetricsSummary {
    pub test_coverage: f64,
    pub test_success_rate: f64,
    pub documentation_freshness_days: f64,
    pub performance_throughput: f64,
    pub security_vulnerabilities: u64,
    pub deployment_success_rate: f64,
    pub quality_gate_pass_rate: f64,
    pub system_uptime_hours: f64,
    pub error_rate: f64,
}

impl UV243MetricsSummary {
    /// Check if all UV-243 requirements are met
    pub fn meets_uv243_requirements(&self) -> bool {
        self.test_coverage >= 90.0
            && self.test_success_rate >= 0.95
            && self.documentation_freshness_days <= 7.0
            && self.performance_throughput >= 4_300_000.0
            && self.security_vulnerabilities == 0
            && self.deployment_success_rate >= 0.95
            && self.quality_gate_pass_rate >= 0.95
            && self.error_rate <= 0.01
    }
    
    /// Get list of requirement violations
    pub fn get_violations(&self) -> Vec<String> {
        let mut violations = Vec::new();
        
        if self.test_coverage < 90.0 {
            violations.push(format!("Test coverage {}% below 90% requirement", self.test_coverage));
        }
        
        if self.test_success_rate < 0.95 {
            violations.push(format!("Test success rate {}% below 95% requirement", self.test_success_rate * 100.0));
        }
        
        if self.documentation_freshness_days > 7.0 {
            violations.push(format!("Documentation {} days old, exceeds 7-day freshness requirement", self.documentation_freshness_days));
        }
        
        if self.performance_throughput < 4_300_000.0 {
            violations.push(format!("Performance throughput {} below 4.3M metrics/sec requirement", self.performance_throughput));
        }
        
        if self.security_vulnerabilities > 0 {
            violations.push(format!("{} security vulnerabilities found", self.security_vulnerabilities));
        }
        
        if self.deployment_success_rate < 0.95 {
            violations.push(format!("Deployment success rate {}% below 95% requirement", self.deployment_success_rate * 100.0));
        }
        
        if self.quality_gate_pass_rate < 0.95 {
            violations.push(format!("Quality gate pass rate {}% below 95% requirement", self.quality_gate_pass_rate * 100.0));
        }
        
        if self.error_rate > 0.01 {
            violations.push(format!("Error rate {}% above 1% threshold", self.error_rate * 100.0));
        }
        
        violations
    }
}

/// Complete quality report with dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityReport {
    pub timestamp: u64,
    pub overall_score: f64,
    pub summary: UV243MetricsSummary,
    pub quality_gates: QualityGatesStatus,
    pub alerts: Vec<QualityAlert>,
}

/// Status of all quality gates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGatesStatus {
    pub coverage_gate: bool,
    pub security_gate: bool,
    pub performance_gate: bool,
    pub documentation_gate: bool,
    pub deployment_gate: bool,
}

impl QualityGatesStatus {
    /// Check if all quality gates are passing
    pub fn all_passing(&self) -> bool {
        self.coverage_gate && 
        self.security_gate && 
        self.performance_gate && 
        self.documentation_gate && 
        self.deployment_gate
    }

    /// Get count of passing gates
    pub fn passing_count(&self) -> usize {
        [self.coverage_gate, self.security_gate, self.performance_gate, 
         self.documentation_gate, self.deployment_gate]
            .iter()
            .filter(|&&gate| gate)
            .count()
    }
}

/// Quality alert for monitoring dashboards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityAlert {
    pub severity: AlertSeverity,
    pub component: String,
    pub message: String,
    pub timestamp: u64,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// UV-243 Metrics Collector Service
/// 
/// Runs continuously to collect and update UV-243 quality metrics
pub struct UV243MetricsCollector {
    metrics: Arc<UV243Metrics>,
    collection_interval: Duration,
}

impl UV243MetricsCollector {
    pub fn new(metrics: Arc<UV243Metrics>, collection_interval: Duration) -> Self {
        Self {
            metrics,
            collection_interval,
        }
    }
    
    /// Start the metrics collection service
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut interval = tokio::time::interval(self.collection_interval);
        let start_time = Instant::now();
        
        info!("Starting UV-243 metrics collection service");
        
        loop {
            interval.tick().await;
            
            // Update system uptime
            let uptime = start_time.elapsed();
            self.metrics.update_system_health(uptime, 0.001, 45.0); // Example values
            
            // Log metrics summary periodically
            if uptime.as_secs() % 300 == 0 { // Every 5 minutes
                let summary = self.metrics.get_metrics_summary();
                if summary.meets_uv243_requirements() {
                    info!("✅ All UV-243 requirements met: {:?}", summary);
                } else {
                    let violations = summary.get_violations();
                    warn!("❌ UV-243 requirement violations: {:?}", violations);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prometheus::Registry;
    
    #[test]
    fn test_uv243_metrics_creation() {
        let registry = Registry::new();
        let metrics = UV243Metrics::new(&registry).unwrap();
        
        // Test coverage update
        metrics.update_coverage(95.0);
        assert_eq!(metrics.test_coverage.get(), 95.0);
        
        // Test quality gate recording
        metrics.record_quality_gate(true);
        assert_eq!(metrics.quality_gate_pass_rate.get(), 1.0);
    }
    
    #[test]
    fn test_metrics_summary_requirements() {
        let summary = UV243MetricsSummary {
            test_coverage: 95.0,
            test_success_rate: 0.98,
            documentation_freshness_days: 2.0,
            performance_throughput: 5_000_000.0,
            security_vulnerabilities: 0,
            deployment_success_rate: 0.99,
            quality_gate_pass_rate: 0.97,
            system_uptime_hours: 720.0,
            error_rate: 0.005,
        };
        
        assert!(summary.meets_uv243_requirements());
        assert!(summary.get_violations().is_empty());
    }
    
    #[test]
    fn test_metrics_summary_violations() {
        let summary = UV243MetricsSummary {
            test_coverage: 85.0, // Below 90%
            test_success_rate: 0.98,
            documentation_freshness_days: 10.0, // Above 7 days
            performance_throughput: 3_000_000.0, // Below 4.3M
            security_vulnerabilities: 2, // Above 0
            deployment_success_rate: 0.99,
            quality_gate_pass_rate: 0.97,
            system_uptime_hours: 720.0,
            error_rate: 0.005,
        };
        
        assert!(!summary.meets_uv243_requirements());
        let violations = summary.get_violations();
        assert_eq!(violations.len(), 4);
    }
}