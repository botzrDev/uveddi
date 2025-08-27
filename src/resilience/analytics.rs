//! Historical alert analysis and trend reporting (UV-248)

use crate::resilience::alerting::{AlertHistory, AlertType, EnhancedAlert};
use crate::resilience::health::AlertSeverity;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, SystemTime};

/// Time series data point for trending analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    /// When this data point was recorded
    pub timestamp: SystemTime,
    /// Numerical value for this metric
    pub value: f64,
    /// Type of alert this measurement relates to
    pub alert_type: AlertType,
    /// Environment where this measurement was taken
    pub environment: String,
}

/// Alert pattern for detection of recurring issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertPattern {
    /// Unique identifier for this pattern
    pub pattern_id: String,
    /// Type of alert in this pattern
    pub alert_type: AlertType,
    /// Environment where pattern occurs
    pub environment: String,
    /// How often this pattern repeats
    pub frequency: Duration,
    /// Number of times pattern has been observed
    pub occurrences: u32,
    /// When this pattern was first detected
    pub first_seen: SystemTime,
    /// Most recent occurrence of this pattern
    pub last_seen: SystemTime,
    /// Confidence score for pattern detection (0.0 to 1.0)
    pub confidence_score: f64,
}

/// Trend analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    /// Type of alert being analyzed for trends
    pub alert_type: AlertType,
    /// Environment where trend analysis was performed
    pub environment: String,
    /// Direction of the trend
    pub trend_direction: TrendDirection,
    /// Strength of the trend (0.0 to 1.0)
    pub trend_strength: f64,
    /// Analysis period in days
    pub period_days: u32,
    /// Number of data points used in analysis
    pub data_points: u32,
    /// Future trend prediction if available
    pub prediction: Option<TrendPrediction>,
}

/// Direction of a trend in time series data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    /// Values are trending upward
    Increasing,
    /// Values are trending downward
    Decreasing,
    /// Values are relatively stable
    Stable,
    /// Values show high variance with no clear trend
    Volatile,
}

/// Future trend prediction based on historical data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendPrediction {
    /// Predicted value for the future time point
    pub predicted_value: f64,
    /// Lower and upper bounds of confidence interval
    pub confidence_interval: (f64, f64),
    /// How many hours into the future this prediction covers
    pub prediction_horizon_hours: u32,
}

/// Alert analytics dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertDashboard {
    /// Total number of alerts in the last 24 hours
    pub total_alerts_24h: u32,
    /// Alert counts grouped by severity level
    pub alerts_by_severity: HashMap<String, u32>,
    /// Alert counts grouped by alert type
    pub alerts_by_type: HashMap<String, u32>,
    /// Alert counts grouped by environment
    pub alerts_by_environment: HashMap<String, u32>,
    /// Top alert sources with their counts
    pub top_alert_sources: Vec<(String, u32)>,
    /// Average time to resolve alerts
    pub avg_resolution_time: Option<Duration>,
    /// Historical trend analysis for different alert types
    pub alert_trends: Vec<TrendAnalysis>,
    /// Detected recurring alert patterns
    pub recurring_patterns: Vec<AlertPattern>,
    /// Percentage of noise reduced by intelligent filtering
    pub noise_reduction_percentage: f64,
    /// When this dashboard data was generated
    pub generated_at: SystemTime,
}

/// Alert analytics engine
pub struct AlertAnalytics {
    history_buffer: VecDeque<AlertHistory>,
    time_series_data: HashMap<String, VecDeque<TimeSeriesPoint>>,
    detected_patterns: HashMap<String, AlertPattern>,
    max_history_size: usize,
    analysis_window_days: u32,
}

impl AlertAnalytics {
    /// Creates a new AlertAnalytics engine with default configuration
    pub fn new() -> Self {
        Self {
            history_buffer: VecDeque::with_capacity(10000),
            time_series_data: HashMap::new(),
            detected_patterns: HashMap::new(),
            max_history_size: 10000,
            analysis_window_days: 30,
        }
    }

    /// Add alert to historical data
    pub fn record_alert(&mut self, alert: &EnhancedAlert) {
        let history_record = AlertHistory {
            timestamp: alert.base_alert.created_at,
            alert_type: alert.alert_type.clone(),
            severity: alert.base_alert.severity.clone(),
            count: 1,
            environment: alert.environment.clone(),
        };

        // Add to history buffer
        self.history_buffer.push_back(history_record);

        // Maintain buffer size
        if self.history_buffer.len() > self.max_history_size {
            self.history_buffer.pop_front();
        }

        // Add to time series data
        let series_key = format!("{:?}-{}", alert.alert_type, alert.environment);
        let series = self
            .time_series_data
            .entry(series_key)
            .or_insert_with(VecDeque::new);

        let severity_weight = match alert.base_alert.severity {
            AlertSeverity::Critical => 3.0,
            AlertSeverity::Warning => 2.0,
            AlertSeverity::Info => 1.0,
        };

        series.push_back(TimeSeriesPoint {
            timestamp: alert.base_alert.created_at,
            value: severity_weight,
            alert_type: alert.alert_type.clone(),
            environment: alert.environment.clone(),
        });

        // Maintain series size
        if series.len() > 1000 {
            series.pop_front();
        }

        // Update pattern detection
        self.update_pattern_detection(alert);
    }

    /// Generate comprehensive dashboard analytics
    pub fn generate_dashboard(&self) -> AlertDashboard {
        let cutoff_24h = SystemTime::now() - Duration::from_secs(24 * 3600);
        let recent_alerts: Vec<&AlertHistory> = self
            .history_buffer
            .iter()
            .filter(|h| h.timestamp >= cutoff_24h)
            .collect();

        let total_alerts_24h = recent_alerts.len() as u32;

        // Group by severity
        let alerts_by_severity = recent_alerts.iter().fold(HashMap::new(), |mut acc, alert| {
            let key = format!("{:?}", alert.severity);
            *acc.entry(key).or_insert(0) += alert.count;
            acc
        });

        // Group by type
        let alerts_by_type = recent_alerts.iter().fold(HashMap::new(), |mut acc, alert| {
            let key = format!("{:?}", alert.alert_type);
            *acc.entry(key).or_insert(0) += alert.count;
            acc
        });

        // Group by environment
        let alerts_by_environment = recent_alerts.iter().fold(HashMap::new(), |mut acc, alert| {
            *acc.entry(alert.environment.clone()).or_insert(0) += alert.count;
            acc
        });

        // Top alert sources (simplified - would be more sophisticated in real implementation)
        let mut top_sources: Vec<(String, u32)> = alerts_by_type
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        top_sources.sort_by(|a, b| b.1.cmp(&a.1));
        top_sources.truncate(5);

        // Generate trend analyses
        let alert_trends = self.generate_trend_analyses();

        // Get recurring patterns
        let recurring_patterns: Vec<AlertPattern> = self
            .detected_patterns
            .values()
            .filter(|p| p.occurrences >= 3) // Only patterns with 3+ occurrences
            .cloned()
            .collect();

        // Calculate noise reduction (simplified metric)
        let noise_reduction_percentage = self.calculate_noise_reduction();

        AlertDashboard {
            total_alerts_24h,
            alerts_by_severity,
            alerts_by_type,
            alerts_by_environment,
            top_alert_sources: top_sources,
            avg_resolution_time: self.calculate_avg_resolution_time(),
            alert_trends,
            recurring_patterns,
            noise_reduction_percentage,
            generated_at: SystemTime::now(),
        }
    }

    /// Generate trend analyses for different alert types and environments
    fn generate_trend_analyses(&self) -> Vec<TrendAnalysis> {
        let mut analyses = Vec::new();

        for (series_key, data_points) in &self.time_series_data {
            if data_points.len() < 3 {
                continue; // Need minimum data points for trend analysis
            }

            let parts: Vec<&str> = series_key.split('-').collect();
            if parts.len() < 2 {
                continue;
            }

            let alert_type = match parts[0] {
                "CriticalFailureRate" => AlertType::CriticalFailureRate,
                "PerformanceRegression" => AlertType::PerformanceRegression,
                "InfrastructureIssues" => AlertType::InfrastructureIssues,
                "FlakyTestDetection" => AlertType::FlakyTestDetection,
                "ResourceUtilization" => AlertType::ResourceUtilization,
                _ => continue,
            };

            let environment = parts[1].to_string();

            let analysis = self.analyze_trend(data_points, alert_type, environment);
            analyses.push(analysis);
        }

        analyses
    }

    /// Analyze trend for specific time series data
    fn analyze_trend(
        &self,
        data_points: &VecDeque<TimeSeriesPoint>,
        alert_type: AlertType,
        environment: String,
    ) -> TrendAnalysis {
        let values: Vec<f64> = data_points.iter().map(|p| p.value).collect();

        // Simple linear trend calculation
        let n = values.len() as f64;
        let sum_x: f64 = (0..values.len()).map(|i| i as f64).sum();
        let sum_y: f64 = values.iter().sum();
        let sum_xy: f64 = values.iter().enumerate().map(|(i, &y)| i as f64 * y).sum();
        let sum_x2: f64 = (0..values.len()).map(|i| (i as f64).powi(2)).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x.powi(2));

        let (trend_direction, trend_strength) = if slope.abs() < 0.01 {
            (TrendDirection::Stable, slope.abs())
        } else if slope > 0.1 {
            (TrendDirection::Increasing, slope)
        } else if slope < -0.1 {
            (TrendDirection::Decreasing, slope.abs())
        } else {
            // Check for volatility
            let variance = self.calculate_variance(&values);
            if variance > 1.0 {
                (TrendDirection::Volatile, variance)
            } else {
                (TrendDirection::Stable, slope.abs())
            }
        };

        // Generate prediction for next 24 hours
        let prediction = if trend_strength > 0.1 {
            let predicted_value = values.last().unwrap_or(&0.0) + slope * 24.0; // 24 hours ahead
            let confidence_range = trend_strength * 0.5; // Simple confidence interval

            Some(TrendPrediction {
                predicted_value,
                confidence_interval: (
                    predicted_value - confidence_range,
                    predicted_value + confidence_range,
                ),
                prediction_horizon_hours: 24,
            })
        } else {
            None
        };

        TrendAnalysis {
            alert_type,
            environment,
            trend_direction,
            trend_strength,
            period_days: self.analysis_window_days,
            data_points: data_points.len() as u32,
            prediction,
        }
    }

    /// Update pattern detection with new alert
    fn update_pattern_detection(&mut self, alert: &EnhancedAlert) {
        let pattern_key = format!(
            "{:?}-{}-{}",
            alert.alert_type, alert.environment, alert.base_alert.component
        );

        let pattern = self
            .detected_patterns
            .entry(pattern_key.clone())
            .or_insert(AlertPattern {
                pattern_id: pattern_key.clone(),
                alert_type: alert.alert_type.clone(),
                environment: alert.environment.clone(),
                frequency: Duration::from_secs(3600), // Default 1 hour
                occurrences: 0,
                first_seen: alert.base_alert.created_at,
                last_seen: alert.base_alert.created_at,
                confidence_score: 0.0,
            });

        pattern.occurrences += 1;
        pattern.last_seen = alert.base_alert.created_at;

        // Update frequency calculation
        if pattern.occurrences > 1 {
            let total_duration = pattern
                .last_seen
                .duration_since(pattern.first_seen)
                .unwrap_or(Duration::from_secs(1));
            pattern.frequency = total_duration / (pattern.occurrences - 1);
        }

        // Calculate confidence score based on regularity (need to get pattern again to avoid borrow checker issues)
        let confidence = self.calculate_pattern_confidence_for_key(&pattern_key);
        self.detected_patterns
            .get_mut(&pattern_key)
            .unwrap()
            .confidence_score = confidence;
    }

    /// Calculate confidence score for alert pattern by key
    fn calculate_pattern_confidence_for_key(&self, pattern_key: &str) -> f64 {
        if let Some(pattern) = self.detected_patterns.get(pattern_key) {
            self.calculate_pattern_confidence(pattern)
        } else {
            0.0
        }
    }

    /// Calculate confidence score for alert pattern
    fn calculate_pattern_confidence(&self, pattern: &AlertPattern) -> f64 {
        if pattern.occurrences < 2 {
            return 0.0;
        }

        // Simple confidence calculation based on occurrence count and frequency regularity
        let occurrence_score = (pattern.occurrences as f64).min(10.0) / 10.0;
        let frequency_regularity = if pattern.frequency.as_secs() > 0 {
            1.0 / (1.0 + (pattern.frequency.as_secs() as f64).log10())
        } else {
            0.0
        };

        (occurrence_score + frequency_regularity) / 2.0
    }

    /// Calculate variance for volatility analysis
    fn calculate_variance(&self, values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64;

        variance
    }

    /// Calculate average resolution time (placeholder implementation)
    fn calculate_avg_resolution_time(&self) -> Option<Duration> {
        // This would require tracking when alerts are resolved/acknowledged
        // For now, return a placeholder based on alert patterns
        if self.history_buffer.len() > 10 {
            Some(Duration::from_secs(1800)) // 30 minutes average
        } else {
            None
        }
    }

    /// Calculate noise reduction percentage achieved by grouping
    fn calculate_noise_reduction(&self) -> f64 {
        // This would compare raw alert count vs grouped alert count
        // For now, return a target-based estimate
        let total_alerts = self.history_buffer.len() as f64;
        if total_alerts > 100.0 {
            // Simulate noise reduction based on pattern detection
            let patterns_detected = self.detected_patterns.len() as f64;
            let reduction_rate = (patterns_detected * 10.0 / total_alerts).min(0.7);
            reduction_rate * 100.0
        } else {
            0.0
        }
    }

    /// Get alerts for specific time period
    pub fn get_alerts_in_period(&self, start: SystemTime, end: SystemTime) -> Vec<&AlertHistory> {
        self.history_buffer
            .iter()
            .filter(|alert| alert.timestamp >= start && alert.timestamp <= end)
            .collect()
    }

    /// Get trend data for specific alert type and environment
    pub fn get_trend_data(
        &self,
        alert_type: AlertType,
        environment: &str,
        days: u32,
    ) -> Option<Vec<TimeSeriesPoint>> {
        let series_key = format!("{:?}-{}", alert_type, environment);
        let cutoff = SystemTime::now() - Duration::from_secs(days as u64 * 24 * 3600);

        self.time_series_data.get(&series_key).map(|series| {
            series
                .iter()
                .filter(|point| point.timestamp >= cutoff)
                .cloned()
                .collect()
        })
    }

    /// Generate alert frequency heatmap data
    pub fn generate_heatmap_data(&self, days: u32) -> HashMap<String, HashMap<u8, u32>> {
        let cutoff = SystemTime::now() - Duration::from_secs(days as u64 * 24 * 3600);
        let mut heatmap = HashMap::new();

        for alert in self.history_buffer.iter() {
            if alert.timestamp >= cutoff {
                let date_key = format!("{:?}", alert.timestamp); // Simplified - would use proper date formatting
                let hour = 12; // Placeholder - would extract actual hour from timestamp

                let day_data = heatmap.entry(date_key).or_insert_with(HashMap::new);
                *day_data.entry(hour).or_insert(0) += alert.count;
            }
        }

        heatmap
    }

    /// Export analytics data for external systems
    pub fn export_analytics_data(&self, format: AnalyticsExportFormat) -> String {
        match format {
            AnalyticsExportFormat::Json => serde_json::to_string_pretty(&self.generate_dashboard())
                .unwrap_or_else(|_| "{}".to_string()),
            AnalyticsExportFormat::Csv => {
                let mut csv = String::from("timestamp,alert_type,severity,environment,count\n");
                for alert in &self.history_buffer {
                    csv.push_str(&format!(
                        "{:?},{:?},{:?},{},{}\n",
                        alert.timestamp,
                        alert.alert_type,
                        alert.severity,
                        alert.environment,
                        alert.count
                    ));
                }
                csv
            }
        }
    }
}

#[derive(Debug, Clone)]
/// Export format for analytics data
pub enum AnalyticsExportFormat {
    /// Export as JSON format
    Json,
    /// Export as CSV format
    Csv,
}

impl Default for AlertAnalytics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resilience::health::{Alert, AlertSeverity};
    use std::collections::HashMap;

    fn create_test_alert(
        alert_type: AlertType,
        severity: AlertSeverity,
        environment: &str,
    ) -> EnhancedAlert {
        EnhancedAlert {
            base_alert: Alert {
                id: format!("test-{}", rand::random::<u32>()),
                component: "test-component".to_string(),
                message: "Test alert for analytics".to_string(),
                severity,
                created_at: SystemTime::now(),
            },
            alert_type,
            fingerprint: "test-fingerprint".to_string(),
            group_key: "test-group".to_string(),
            environment: environment.to_string(),
            metadata: HashMap::new(),
            acknowledged: false,
            acknowledged_at: None,
            acknowledged_by: None,
            escalation_level: 0,
            escalated_at: None,
        }
    }

    #[test]
    fn test_analytics_creation() {
        let analytics = AlertAnalytics::new();
        assert_eq!(analytics.history_buffer.len(), 0);
        assert_eq!(analytics.time_series_data.len(), 0);
        assert_eq!(analytics.detected_patterns.len(), 0);
    }

    #[test]
    fn test_record_alert() {
        let mut analytics = AlertAnalytics::new();
        let alert = create_test_alert(
            AlertType::CriticalFailureRate,
            AlertSeverity::Critical,
            "production",
        );

        analytics.record_alert(&alert);

        assert_eq!(analytics.history_buffer.len(), 1);
        assert_eq!(analytics.time_series_data.len(), 1);

        let series_key = "CriticalFailureRate-production";
        assert!(analytics.time_series_data.contains_key(series_key));
    }

    #[test]
    fn test_generate_dashboard() {
        let mut analytics = AlertAnalytics::new();

        // Add some test alerts
        for i in 0..5 {
            let alert = create_test_alert(
                AlertType::CriticalFailureRate,
                if i % 2 == 0 {
                    AlertSeverity::Critical
                } else {
                    AlertSeverity::Warning
                },
                "production",
            );
            analytics.record_alert(&alert);
        }

        let dashboard = analytics.generate_dashboard();
        assert_eq!(dashboard.total_alerts_24h, 5);
        assert!(!dashboard.alerts_by_severity.is_empty());
        assert!(!dashboard.alerts_by_type.is_empty());
        assert!(!dashboard.alerts_by_environment.is_empty());
    }

    #[test]
    fn test_pattern_detection() {
        let mut analytics = AlertAnalytics::new();
        let alert_type = AlertType::FlakyTestDetection;
        let environment = "staging";

        // Create multiple similar alerts to trigger pattern detection
        for _ in 0..4 {
            let alert = create_test_alert(alert_type.clone(), AlertSeverity::Warning, environment);
            analytics.record_alert(&alert);
        }

        assert!(!analytics.detected_patterns.is_empty());

        let pattern_key = format!("{:?}-{}-test-component", alert_type, environment);
        let pattern = analytics.detected_patterns.get(&pattern_key);
        assert!(pattern.is_some());
        assert_eq!(pattern.unwrap().occurrences, 4);
    }

    #[test]
    fn test_trend_analysis() {
        let mut analytics = AlertAnalytics::new();
        let alert_type = AlertType::PerformanceRegression;
        let environment = "production";

        // Add alerts with increasing severity to create trend
        let severities = [
            AlertSeverity::Info,
            AlertSeverity::Warning,
            AlertSeverity::Warning,
            AlertSeverity::Critical,
            AlertSeverity::Critical,
        ];

        for severity in &severities {
            let alert = create_test_alert(alert_type.clone(), severity.clone(), environment);
            analytics.record_alert(&alert);
        }

        let dashboard = analytics.generate_dashboard();
        assert!(!dashboard.alert_trends.is_empty());

        let trend = &dashboard.alert_trends[0];
        assert_eq!(trend.alert_type, alert_type);
        assert_eq!(trend.environment, environment);
    }

    #[test]
    fn test_export_analytics_data() {
        let mut analytics = AlertAnalytics::new();
        let alert = create_test_alert(
            AlertType::ResourceUtilization,
            AlertSeverity::Warning,
            "production",
        );
        analytics.record_alert(&alert);

        let json_export = analytics.export_analytics_data(AnalyticsExportFormat::Json);
        assert!(json_export.contains("total_alerts_24h"));
        assert!(json_export.contains("alerts_by_severity"));

        let csv_export = analytics.export_analytics_data(AnalyticsExportFormat::Csv);
        assert!(csv_export.contains("timestamp,alert_type,severity,environment,count"));
        assert!(csv_export.contains("ResourceUtilization"));
    }

    #[test]
    fn test_variance_calculation() {
        let analytics = AlertAnalytics::new();
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let variance = analytics.calculate_variance(&values);
        assert!(variance > 0.0);
    }

    #[test]
    fn test_get_alerts_in_period() {
        let mut analytics = AlertAnalytics::new();
        let now = SystemTime::now();
        let hour_ago = now - Duration::from_secs(3600);

        let alert = create_test_alert(
            AlertType::InfrastructureIssues,
            AlertSeverity::Critical,
            "production",
        );
        analytics.record_alert(&alert);

        let alerts = analytics.get_alerts_in_period(hour_ago, now + Duration::from_secs(60));
        assert_eq!(alerts.len(), 1);
    }
}
