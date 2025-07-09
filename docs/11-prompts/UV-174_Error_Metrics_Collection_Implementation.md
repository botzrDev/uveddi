# 🤖 **GPT Prompt for UV-174: Error Metrics Collection Implementation**

```markdown
# UV-174: Implement Error Metrics Collection for Performance Monitoring

You are a senior Rust developer working on the Uveddi project, a sophisticated static code analysis tool. Your task is to implement comprehensive error metrics collection that leverages the structured error system from UV-169 and structured logging from UV-170 to provide detailed performance monitoring and observability.

## 🎯 **Objective**
Create a robust metrics collection system that tracks error patterns, categorizes failures, and provides actionable insights for monitoring the rendering service performance and reliability.

## 📋 **Current State Analysis**
The foundation is already in place:
- ✅ RenderingServiceError system (UV-169) with error categories and severity levels
- ✅ Structured logging (UV-170) with JSON output and error metadata
- ✅ Type-safe retry logic (UV-168) with error classification
- ✅ Circuit breaker (UV-172) with intelligent failure detection

## 🔧 **Required Implementation**

### **1. Create Metrics Collection System**
Create file: `src/resilience/metrics.rs`

```rust
use crate::error::{RenderingServiceError, ErrorCategory, ErrorSeverity};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use tracing::{info, debug};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMetrics {
    pub total_errors: u64,
    pub errors_by_category: HashMap<String, u64>,
    pub errors_by_severity: HashMap<String, u64>,
    pub error_rate_per_minute: f64,
    pub last_error_time: Option<SystemTime>,
    pub error_trends: Vec<ErrorTrend>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorTrend {
    pub timestamp: SystemTime,
    pub category: String,
    pub severity: String,
    pub count: u64,
    pub window_duration_seconds: u64,
}

#[derive(Debug, Clone)]
pub struct MetricsConfig {
    pub collection_interval: Duration,
    pub trend_window_size: usize,
    pub enable_detailed_tracking: bool,
    pub export_format: MetricsFormat,
}

#[derive(Debug, Clone)]
pub enum MetricsFormat {
    Json,
    Prometheus,
    InfluxDB,
}

pub struct MetricsCollector {
    config: MetricsConfig,
    metrics: Arc<Mutex<ErrorMetrics>>,
    error_history: Arc<Mutex<Vec<ErrorEvent>>>,
    start_time: Instant,
}

#[derive(Debug, Clone)]
struct ErrorEvent {
    timestamp: SystemTime,
    category: ErrorCategory,
    severity: ErrorSeverity,
    error_type: String,
}
```

### **2. Core Metrics Collection Logic**

```rust
impl MetricsCollector {
    pub fn new(config: MetricsConfig) -> Self {
        Self {
            config,
            metrics: Arc::new(Mutex::new(ErrorMetrics::default())),
            error_history: Arc::new(Mutex::new(Vec::new())),
            start_time: Instant::now(),
        }
    }

    pub fn record_error(&self, error: &RenderingServiceError) {
        // Record error event with timestamp
        let event = ErrorEvent {
            timestamp: SystemTime::now(),
            category: error.category(),
            severity: error.severity(),
            error_type: format!("{:?}", error).split('(').next().unwrap_or("Unknown").to_string(),
        };

        // Update metrics atomically
        {
            let mut metrics = self.metrics.lock().unwrap();
            metrics.total_errors += 1;
            
            let category_key = error.category().as_str().to_string();
            *metrics.errors_by_category.entry(category_key).or_insert(0) += 1;
            
            let severity_key = error.severity().as_str().to_string();
            *metrics.errors_by_severity.entry(severity_key).or_insert(0) += 1;
            
            metrics.last_error_time = Some(event.timestamp);
        }

        // Store in history for trend analysis
        {
            let mut history = self.error_history.lock().unwrap();
            history.push(event);
            
            // Keep only recent events (sliding window)
            let cutoff = SystemTime::now() - Duration::from_secs(3600); // 1 hour
            history.retain(|e| e.timestamp > cutoff);
        }

        // Log the metric event
        debug!(
            error_category = ?error.category(),
            error_severity = ?error.severity(),
            total_errors = self.get_total_errors(),
            "Error metric recorded"
        );
    }

    pub fn calculate_error_rate(&self) -> f64 {
        let history = self.error_history.lock().unwrap();
        let now = SystemTime::now();
        let one_minute_ago = now - Duration::from_secs(60);
        
        let recent_errors = history.iter()
            .filter(|e| e.timestamp > one_minute_ago)
            .count();
        
        recent_errors as f64 // errors per minute
    }

    pub fn generate_trends(&self) -> Vec<ErrorTrend> {
        let history = self.error_history.lock().unwrap();
        let mut trends = Vec::new();
        
        // Group by 5-minute windows
        let window_size = Duration::from_secs(300);
        let now = SystemTime::now();
        
        for i in 0..self.config.trend_window_size {
            let window_start = now - window_size * (i + 1) as u32;
            let window_end = now - window_size * i as u32;
            
            let mut category_counts: HashMap<String, u64> = HashMap::new();
            let mut severity_counts: HashMap<String, u64> = HashMap::new();
            
            for event in history.iter() {
                if event.timestamp >= window_start && event.timestamp < window_end {
                    *category_counts.entry(event.category.as_str().to_string()).or_insert(0) += 1;
                    *severity_counts.entry(event.severity.as_str().to_string()).or_insert(0) += 1;
                }
            }
            
            // Create trends for each category
            for (category, count) in category_counts {
                trends.push(ErrorTrend {
                    timestamp: window_start,
                    category,
                    severity: "all".to_string(),
                    count,
                    window_duration_seconds: window_size.as_secs(),
                });
            }
        }
        
        trends
    }

    pub fn get_metrics_snapshot(&self) -> ErrorMetrics {
        let mut metrics = self.metrics.lock().unwrap().clone();
        metrics.error_rate_per_minute = self.calculate_error_rate();
        metrics.error_trends = self.generate_trends();
        metrics
    }

    pub fn export_metrics(&self, format: &MetricsFormat) -> String {
        let metrics = self.get_metrics_snapshot();
        
        match format {
            MetricsFormat::Json => serde_json::to_string_pretty(&metrics).unwrap_or_default(),
            MetricsFormat::Prometheus => self.to_prometheus_format(&metrics),
            MetricsFormat::InfluxDB => self.to_influxdb_format(&metrics),
        }
    }

    fn to_prometheus_format(&self, metrics: &ErrorMetrics) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("# HELP uveddi_total_errors Total number of errors\n"));
        output.push_str(&format!("# TYPE uveddi_total_errors counter\n"));
        output.push_str(&format!("uveddi_total_errors {}\n", metrics.total_errors));
        
        output.push_str(&format!("# HELP uveddi_error_rate_per_minute Current error rate per minute\n"));
        output.push_str(&format!("# TYPE uveddi_error_rate_per_minute gauge\n"));
        output.push_str(&format!("uveddi_error_rate_per_minute {}\n", metrics.error_rate_per_minute));
        
        for (category, count) in &metrics.errors_by_category {
            output.push_str(&format!("uveddi_errors_by_category{{category=\"{}\"}} {}\n", category, count));
        }
        
        for (severity, count) in &metrics.errors_by_severity {
            output.push_str(&format!("uveddi_errors_by_severity{{severity=\"{}\"}} {}\n", severity, count));
        }
        
        output
    }

    fn to_influxdb_format(&self, metrics: &ErrorMetrics) -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        
        let mut lines = Vec::new();
        
        lines.push(format!("uveddi_errors,type=total value={} {}", metrics.total_errors, timestamp));
        lines.push(format!("uveddi_error_rate,type=per_minute value={} {}", metrics.error_rate_per_minute, timestamp));
        
        for (category, count) in &metrics.errors_by_category {
            lines.push(format!("uveddi_errors,category={} value={} {}", category, count, timestamp));
        }
        
        for (severity, count) in &metrics.errors_by_severity {
            lines.push(format!("uveddi_errors,severity={} value={} {}", severity, count, timestamp));
        }
        
        lines.join("\n")
    }

    pub fn get_total_errors(&self) -> u64 {
        self.metrics.lock().unwrap().total_errors
    }

    pub fn reset_metrics(&self) {
        let mut metrics = self.metrics.lock().unwrap();
        *metrics = ErrorMetrics::default();
        
        let mut history = self.error_history.lock().unwrap();
        history.clear();
        
        info!("Error metrics reset");
    }
}
```

### **3. Default Implementations**

```rust
impl Default for ErrorMetrics {
    fn default() -> Self {
        Self {
            total_errors: 0,
            errors_by_category: HashMap::new(),
            errors_by_severity: HashMap::new(),
            error_rate_per_minute: 0.0,
            last_error_time: None,
            error_trends: Vec::new(),
        }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            collection_interval: Duration::from_secs(60),
            trend_window_size: 12, // 12 * 5 minutes = 1 hour of trends
            enable_detailed_tracking: true,
            export_format: MetricsFormat::Json,
        }
    }
}
```

### **4. Integration with Resilience Module**
Update `src/resilience/mod.rs`:
```rust
pub mod metrics;
pub use metrics::{MetricsCollector, MetricsConfig, ErrorMetrics, MetricsFormat};
```

### **5. Comprehensive Testing**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_error_recording() {
        let config = MetricsConfig::default();
        let collector = MetricsCollector::new(config);
        
        let error = RenderingServiceError::ServiceUnavailable;
        collector.record_error(&error);
        
        assert_eq!(collector.get_total_errors(), 1);
        
        let metrics = collector.get_metrics_snapshot();
        assert_eq!(metrics.errors_by_category.get("service_communication"), Some(&1));
        assert_eq!(metrics.errors_by_severity.get("critical"), Some(&1));
    }

    #[test]
    fn test_error_rate_calculation() {
        let config = MetricsConfig::default();
        let collector = MetricsCollector::new(config);
        
        // Record multiple errors
        for _ in 0..5 {
            collector.record_error(&RenderingServiceError::ServiceUnavailable);
        }
        
        let rate = collector.calculate_error_rate();
        assert_eq!(rate, 5.0); // 5 errors in the last minute
    }

    #[test]
    fn test_metrics_export_json() {
        let config = MetricsConfig::default();
        let collector = MetricsCollector::new(config);
        
        collector.record_error(&RenderingServiceError::ServiceUnavailable);
        collector.record_error(&RenderingServiceError::InvalidMermaidSyntax { 
            line: None, 
            details: "test".to_string() 
        });
        
        let json_output = collector.export_metrics(&MetricsFormat::Json);
        assert!(json_output.contains("total_errors"));
        assert!(json_output.contains("errors_by_category"));
    }

    #[test]
    fn test_metrics_export_prometheus() {
        let config = MetricsConfig::default();
        let collector = MetricsCollector::new(config);
        
        collector.record_error(&RenderingServiceError::ServiceUnavailable);
        
        let prometheus_output = collector.export_metrics(&MetricsFormat::Prometheus);
        assert!(prometheus_output.contains("uveddi_total_errors"));
        assert!(prometheus_output.contains("# HELP"));
        assert!(prometheus_output.contains("# TYPE"));
    }

    #[test]
    fn test_trend_generation() {
        let config = MetricsConfig {
            trend_window_size: 3,
            ..Default::default()
        };
        let collector = MetricsCollector::new(config);
        
        // Record errors
        collector.record_error(&RenderingServiceError::ServiceUnavailable);
        collector.record_error(&RenderingServiceError::ConnectionTimeout { timeout: 5000 });
        
        let trends = collector.generate_trends();
        assert!(!trends.is_empty());
    }

    #[test]
    fn test_metrics_reset() {
        let config = MetricsConfig::default();
        let collector = MetricsCollector::new(config);
        
        collector.record_error(&RenderingServiceError::ServiceUnavailable);
        assert_eq!(collector.get_total_errors(), 1);
        
        collector.reset_metrics();
        assert_eq!(collector.get_total_errors(), 0);
    }

    #[test]
    fn test_concurrent_error_recording() {
        let config = MetricsConfig::default();
        let collector = Arc::new(MetricsCollector::new(config));
        
        let mut handles = vec![];
        
        for i in 0..10 {
            let collector_clone = collector.clone();
            let handle = thread::spawn(move || {
                for _ in 0..10 {
                    collector_clone.record_error(&RenderingServiceError::ServiceUnavailable);
                }
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        assert_eq!(collector.get_total_errors(), 100);
    }

    #[test]
    fn test_error_history_cleanup() {
        let config = MetricsConfig::default();
        let collector = MetricsCollector::new(config);
        
        // This test would need to manipulate time or use a shorter retention period
        // to verify that old errors are cleaned up from history
        collector.record_error(&RenderingServiceError::ServiceUnavailable);
        
        let history = collector.error_history.lock().unwrap();
        assert_eq!(history.len(), 1);
    }
}
```

## ✅ **Acceptance Criteria**

Before submitting your implementation, ensure:

1. **Functionality**: 
   - Error metrics are collected using error categories from UV-169
   - Structured logging integration from UV-170 works correctly
   - Multiple export formats (JSON, Prometheus, InfluxDB) are supported
   - Error rate calculations are accurate
   - Trend analysis provides meaningful insights

2. **Performance**:
   - Thread-safe concurrent error recording
   - Efficient memory usage with sliding window cleanup
   - Minimal overhead on error recording (< 1ms)

3. **Testing**:
   - All 8 unit tests pass
   - Thread safety verified with concurrent tests
   - Export format validation
   - Metrics accuracy verification

4. **Integration**:
   - Works with RenderingServiceError from UV-169
   - Uses structured logging from UV-170
   - Ready for integration with monitoring systems

## 🚨 **Important Notes**

- **Thread Safety**: Use Arc<Mutex<>> for shared state
- **Memory Management**: Implement sliding window cleanup for error history
- **Performance**: Keep error recording fast and non-blocking
- **Export Formats**: Support multiple monitoring system formats
- **Error Classification**: Use the existing error category/severity system

## 🔍 **Validation Commands**

Run these commands to validate your implementation:
```bash
cargo check --all-targets
cargo test resilience::metrics
cargo clippy -- -D warnings
cargo fmt --check
```

## 📁 **Files to Modify**

- `src/resilience/metrics.rs` (new file - primary implementation)
- `src/resilience/mod.rs` (add metrics module export)

Your implementation should be production-ready and integrate seamlessly with the existing Uveddi resilience architecture. Focus on accuracy, performance, and providing actionable metrics for monitoring and alerting.
```

---

## 📋 **Prompt Usage Instructions**

### **For Project Managers:**
1. Copy the entire prompt content between the markdown code blocks
2. Paste into your AI coding assistant (ChatGPT, Claude, etc.)
3. Provide access to the current codebase files mentioned
4. Review the implementation before merging

### **For Developers:**
This prompt provides:
- ✅ Complete implementation with all required methods
- ✅ Multiple export formats (JSON, Prometheus, InfluxDB)
- ✅ Thread-safe concurrent error recording
- ✅ 8 comprehensive test cases covering all functionality
- ✅ Performance and memory management considerations
- ✅ Integration with existing error and logging systems

### **Key Features:**
- **Error Categorization**: Uses RenderingServiceError categories and severity levels
- **Multiple Export Formats**: JSON, Prometheus, and InfluxDB line protocol
- **Trend Analysis**: 5-minute window trending with configurable history
- **Thread Safety**: Concurrent error recording with Arc<Mutex<>>
- **Memory Efficiency**: Sliding window cleanup for error history
- **Performance**: Minimal overhead error recording

### **Integration Points:**
This implementation enables:
- **UV-173**: Health monitoring using error metrics
- **Monitoring Systems**: Prometheus, Grafana, InfluxDB integration
- **Alerting**: Error rate and pattern-based alerts
- **Observability**: Complete error tracking and analysis

### **Dependencies:**
- **Requires**: UV-169 (RenderingServiceError system) - ✅ Complete
- **Requires**: UV-170 (Structured Logging) - ✅ Complete
- **Enables**: UV-173 (Health Monitoring), observability systems

**Ready to assign to your AI coding assistant!** 🚀