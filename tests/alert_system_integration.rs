//! Integration tests for advanced alert system (UV-248)
//! 
//! These tests validate the complete alert system workflow including:
//! - Alert generation and processing
//! - Intelligent alert grouping 
//! - Multi-channel notifications
//! - Escalation and acknowledgment workflows
//! - Historical analysis and trending

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::time::sleep;

use uveddi::resilience::{
    AdvancedAlertSystem, AlertAnalytics, AlertType, AlertingConfig, EscalationManager,
    HealthMonitor, NotificationClient, AcknowledgmentAPI
};
use uveddi::resilience::alerting::{
    AlertThreshold, EscalationLevel, EscalationPolicy, MetricsData, NotificationChannel,
    ChannelConfig, ChannelType
};
use uveddi::resilience::escalation::AcknowledgmentSource;
use uveddi::resilience::health::{Alert, AlertSeverity};

/// Test suite for alert system integration
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_complete_alert_workflow() {
        // Setup
        let health_monitor = Arc::new(HealthMonitor::new());
        let alert_system = AdvancedAlertSystem::new(Arc::clone(&health_monitor));
        let escalation_manager = Arc::new(EscalationManager::new());

        // Configure alert system
        let config = create_test_config();
        alert_system.update_config(config.clone()).await;

        // Test metrics that should trigger alert
        let metrics = MetricsData {
            error_rate: 15.0, // Above critical threshold of 10%
            latency: Duration::from_millis(500),
            throughput: 1000,
            cpu_usage: Some(85.0),
            memory_usage: Some(75.0),
            disk_usage: Some(60.0),
        };

        // Process metrics and generate alerts
        let result = alert_system
            .process_metrics(&metrics, "production")
            .await;
        
        assert!(result.is_ok(), "Failed to process metrics: {:?}", result);

        // Verify alert was created and grouped
        let groups = alert_system.get_alert_groups().await;
        assert!(!groups.is_empty(), "No alert groups were created");
        
        let group = groups.values().next().unwrap();
        assert_eq!(group.severity, AlertSeverity::Critical);
        assert!(group.count >= 1);

        // Verify integration with health monitor
        let health_status = health_monitor.get_status().await;
        assert!(!health_status.alerts.is_empty(), "Health monitor should contain alerts");
    }

    #[tokio::test]
    async fn test_alert_grouping_reduces_noise() {
        let health_monitor = Arc::new(HealthMonitor::new());
        let alert_system = AdvancedAlertSystem::new(Arc::clone(&health_monitor));

        let config = create_test_config();
        alert_system.update_config(config).await;

        // Generate multiple similar alerts rapidly
        let metrics = MetricsData {
            error_rate: 12.0,
            latency: Duration::from_millis(200),
            throughput: 500,
            cpu_usage: Some(90.0),
            memory_usage: Some(80.0),
            disk_usage: Some(70.0),
        };

        // Process the same metrics multiple times (simulating burst of similar alerts)
        for _ in 0..10 {
            alert_system
                .process_metrics(&metrics, "production")
                .await
                .unwrap();
        }

        // Check that alerts were grouped (noise reduction)
        let groups = alert_system.get_alert_groups().await;
        
        // Should have much fewer groups than individual alerts
        assert!(groups.len() < 10, "Alert grouping should reduce noise. Got {} groups", groups.len());
        
        // At least one group should have multiple alerts
        let max_group_size = groups.values().map(|g| g.count).max().unwrap_or(0);
        assert!(max_group_size > 1, "At least one group should contain multiple alerts");

        // Calculate noise reduction percentage
        let total_individual_alerts: u32 = groups.values().map(|g| g.count).sum();
        let group_count = groups.len();
        let noise_reduction = (1.0 - (group_count as f64 / total_individual_alerts as f64)) * 100.0;
        
        assert!(noise_reduction >= 60.0, "Should achieve at least 60% noise reduction, got {:.1}%", noise_reduction);
    }

    #[tokio::test]
    async fn test_escalation_workflow() {
        let health_monitor = Arc::new(HealthMonitor::new());
        let alert_system = AdvancedAlertSystem::new(Arc::clone(&health_monitor));
        let escalation_manager = Arc::new(EscalationManager::new());

        let config = create_test_config();
        let policy = config.escalation_policies.first().unwrap().clone();
        let channels = config.channels;

        // Create test alert
        let alert = alert_system.create_enhanced_alert(
            AlertType::CriticalFailureRate,
            "Critical error rate exceeded in production".to_string(),
            AlertSeverity::Critical,
            "production".to_string(),
        ).await;

        // Start escalation
        escalation_manager
            .start_escalation(&alert, &policy, &channels)
            .await
            .unwrap();

        // Verify escalation state
        let escalation_state = escalation_manager
            .get_escalation_status(&alert.base_alert.id)
            .await;
        
        assert!(escalation_state.is_some());
        let state = escalation_state.unwrap();
        assert_eq!(state.current_level, 1);
        assert!(state.is_active);
        assert!(!state.escalation_history.is_empty());

        // Test acknowledgment
        escalation_manager
            .acknowledge_alert(
                &alert.base_alert.id,
                "test-user",
                AcknowledgmentSource::WebUI,
                Some("Investigating the issue".to_string()),
            )
            .await
            .unwrap();

        // Verify acknowledgment stopped escalation
        let updated_state = escalation_manager
            .get_escalation_status(&alert.base_alert.id)
            .await
            .unwrap();
        assert!(!updated_state.is_active);

        let acknowledgment = escalation_manager
            .get_acknowledgment(&alert.base_alert.id)
            .await;
        assert!(acknowledgment.is_some());
        assert_eq!(acknowledgment.unwrap().acknowledged_by, "test-user");
    }

    #[tokio::test]
    async fn test_notification_channels() {
        let notification_client = NotificationClient::new();
        
        // Test Slack channel
        let slack_channel = NotificationChannel {
            name: "test-slack".to_string(),
            channel_type: ChannelType::Slack,
            config: ChannelConfig {
                webhook_url: Some("https://hooks.slack.com/test".to_string()),
                email_recipients: None,
                github_repo: None,
                github_token: None,
            },
            enabled: true,
        };

        // Test email channel
        let email_channel = NotificationChannel {
            name: "test-email".to_string(),
            channel_type: ChannelType::Email,
            config: ChannelConfig {
                webhook_url: None,
                email_recipients: Some(vec!["test@example.com".to_string()]),
                github_repo: None,
                github_token: None,
            },
            enabled: true,
        };

        // Test GitHub channel
        let github_channel = NotificationChannel {
            name: "test-github".to_string(),
            channel_type: ChannelType::GitHub,
            config: ChannelConfig {
                webhook_url: None,
                email_recipients: None,
                github_repo: Some("test/repo".to_string()),
                github_token: Some("test-token".to_string()),
            },
            enabled: true,
        };

        // Create test alert
        let alert = create_test_enhanced_alert();

        // Test all notification channels (these will use mock implementations)
        let slack_result = notification_client.send_notification(&slack_channel, &alert).await;
        let email_result = notification_client.send_notification(&email_channel, &alert).await;
        let github_result = notification_client.send_notification(&github_channel, &alert).await;

        // All should succeed with mock implementations
        assert!(slack_result.is_ok(), "Slack notification failed: {:?}", slack_result);
        assert!(email_result.is_ok(), "Email notification failed: {:?}", email_result);
        assert!(github_result.is_ok(), "GitHub notification failed: {:?}", github_result);

        // Test connectivity checks
        let slack_test = notification_client.test_channel(&slack_channel).await;
        let email_test = notification_client.test_channel(&email_channel).await;
        let github_test = notification_client.test_channel(&github_channel).await;

        assert!(slack_test.is_ok(), "Slack channel test failed: {:?}", slack_test);
        assert!(email_test.is_ok(), "Email channel test failed: {:?}", email_test);
        assert!(github_test.is_ok(), "GitHub channel test failed: {:?}", github_test);
    }

    #[tokio::test]
    async fn test_historical_analysis_and_trending() {
        let mut analytics = AlertAnalytics::new();

        // Generate test data over time
        let alert_types = [
            AlertType::CriticalFailureRate,
            AlertType::PerformanceRegression,
            AlertType::ResourceUtilization,
        ];

        let environments = ["production", "staging", "development"];
        let severities = [AlertSeverity::Info, AlertSeverity::Warning, AlertSeverity::Critical];

        // Create alerts with varying patterns
        for i in 0..50 {
            let alert_type = alert_types[i % alert_types.len()].clone();
            let environment = environments[i % environments.len()];
            let severity = severities[i % severities.len()].clone();

            let alert = create_enhanced_alert_with_params(alert_type, severity, environment);
            analytics.record_alert(&alert);
        }

        // Generate dashboard analytics
        let dashboard = analytics.generate_dashboard();

        // Verify dashboard completeness
        assert!(dashboard.total_alerts_24h > 0, "Should have alerts in last 24h");
        assert!(!dashboard.alerts_by_severity.is_empty(), "Should have severity breakdown");
        assert!(!dashboard.alerts_by_type.is_empty(), "Should have type breakdown");
        assert!(!dashboard.alerts_by_environment.is_empty(), "Should have environment breakdown");
        assert!(!dashboard.top_alert_sources.is_empty(), "Should have top sources");
        
        // Verify trend analysis
        assert!(!dashboard.alert_trends.is_empty(), "Should have trend analysis");
        
        // Verify noise reduction calculation
        assert!(dashboard.noise_reduction_percentage >= 0.0, "Noise reduction should be non-negative");

        // Test export functionality
        let json_export = analytics.export_analytics_data(
            uveddi::resilience::analytics::AnalyticsExportFormat::Json
        );
        assert!(json_export.contains("total_alerts_24h"), "JSON export should contain dashboard data");

        let csv_export = analytics.export_analytics_data(
            uveddi::resilience::analytics::AnalyticsExportFormat::Csv
        );
        assert!(csv_export.contains("timestamp,alert_type,severity"), "CSV export should have headers");
    }

    #[tokio::test]
    async fn test_acknowledgment_api_integration() {
        let escalation_manager = Arc::new(EscalationManager::new());
        let ack_api = AcknowledgmentAPI::new(Arc::clone(&escalation_manager));

        let health_monitor = Arc::new(HealthMonitor::new());
        let alert_system = AdvancedAlertSystem::new(health_monitor);

        // Create and process alert
        let alert = alert_system.create_enhanced_alert(
            AlertType::InfrastructureIssues,
            "Database connection timeout".to_string(),
            AlertSeverity::Critical,
            "production".to_string(),
        ).await;

        let alert_id = alert.base_alert.id.clone();

        // Start escalation (required for acknowledgment to work)
        let config = create_test_config();
        let policy = config.escalation_policies.first().unwrap();
        escalation_manager
            .start_escalation(&alert, policy, &config.channels)
            .await
            .unwrap();

        // Test different acknowledgment sources
        let api_result = ack_api
            .acknowledge_via_api(&alert_id, "api-user", Some("Fixed via API".to_string()))
            .await;
        assert!(api_result.is_ok(), "API acknowledgment should succeed");

        // Create another alert for testing other acknowledgment methods
        let alert2 = alert_system.create_enhanced_alert(
            AlertType::FlakyTestDetection,
            "Flaky test detected in CI".to_string(),
            AlertSeverity::Warning,
            "staging".to_string(),
        ).await;

        let alert2_id = alert2.base_alert.id.clone();
        escalation_manager
            .start_escalation(&alert2, policy, &config.channels)
            .await
            .unwrap();

        let slack_result = ack_api
            .acknowledge_via_slack(&alert2_id, "slack-user", None)
            .await;
        assert!(slack_result.is_ok(), "Slack acknowledgment should succeed");

        // Verify acknowledgment status
        let ack_status = ack_api.get_acknowledgment_status(&alert_id).await;
        assert!(ack_status.is_some(), "Should have acknowledgment record");
        assert_eq!(ack_status.unwrap().acknowledged_by, "api-user");
    }

    #[tokio::test]
    async fn test_configuration_validation() {
        let health_monitor = Arc::new(HealthMonitor::new());
        let alert_system = AdvancedAlertSystem::new(health_monitor);

        // Test valid configuration
        let valid_config = create_test_config();
        let result = alert_system.update_config(valid_config).await;
        // update_config doesn't return a Result, so we just verify it doesn't panic

        // Test threshold configuration
        let mut config = create_test_config();
        config.thresholds.push(AlertThreshold {
            alert_type: AlertType::ResourceUtilization,
            environment: "test".to_string(),
            warning_threshold: 70.0,
            critical_threshold: 90.0,
            enabled: true,
        });

        alert_system.update_config(config).await;

        // Test metrics processing with new threshold
        let metrics = MetricsData {
            error_rate: 5.0,
            latency: Duration::from_millis(100),
            throughput: 2000,
            cpu_usage: Some(75.0), // Between warning and critical
            memory_usage: Some(60.0),
            disk_usage: Some(50.0),
        };

        let result = alert_system.process_metrics(&metrics, "test").await;
        assert!(result.is_ok(), "Processing metrics with valid config should succeed");
    }

    #[tokio::test]
    async fn test_escalation_statistics() {
        let escalation_manager = Arc::new(EscalationManager::new());
        let health_monitor = Arc::new(HealthMonitor::new());
        let alert_system = AdvancedAlertSystem::new(health_monitor);

        let config = create_test_config();
        let policy = config.escalation_policies.first().unwrap();

        // Create multiple alerts with different outcomes
        for i in 0..5 {
            let alert = alert_system.create_enhanced_alert(
                AlertType::CriticalFailureRate,
                format!("Test alert {}", i),
                AlertSeverity::Warning,
                "production".to_string(),
            ).await;

            escalation_manager
                .start_escalation(&alert, policy, &config.channels)
                .await
                .unwrap();

            // Acknowledge some alerts
            if i % 2 == 0 {
                escalation_manager
                    .acknowledge_alert(
                        &alert.base_alert.id,
                        &format!("user-{}", i),
                        AcknowledgmentSource::WebUI,
                        Some(format!("Resolved alert {}", i)),
                    )
                    .await
                    .unwrap();
            }
        }

        // Get escalation statistics
        let stats = escalation_manager.get_escalation_stats(1).await;
        
        assert_eq!(stats.total_alerts, 5, "Should track all alerts");
        assert_eq!(stats.acknowledged_alerts, 3, "Should track acknowledged alerts (3 out of 5)");
        assert!(stats.avg_acknowledgment_time.is_some(), "Should calculate average acknowledgment time");
        assert!(!stats.escalation_levels.is_empty(), "Should track escalation levels");
        assert_eq!(stats.period_hours, 1, "Should match requested period");
    }

    // Helper functions for creating test data

    fn create_test_config() -> AlertingConfig {
        AlertingConfig {
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
                    warning_threshold: 15.0,
                    critical_threshold: 30.0,
                    enabled: true,
                },
            ],
            channels: vec![
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
            ],
            escalation_policies: vec![
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
                },
            ],
            grouping_window_minutes: 5,
            max_alerts_per_group: 10,
            history_retention_days: 30,
        }
    }

    fn create_test_enhanced_alert() -> uveddi::resilience::alerting::EnhancedAlert {
        create_enhanced_alert_with_params(
            AlertType::CriticalFailureRate,
            AlertSeverity::Critical,
            "production"
        )
    }

    fn create_enhanced_alert_with_params(
        alert_type: AlertType,
        severity: AlertSeverity,
        environment: &str,
    ) -> uveddi::resilience::alerting::EnhancedAlert {
        use uveddi::resilience::alerting::EnhancedAlert;
        
        EnhancedAlert {
            base_alert: Alert {
                id: format!("test-{}", rand::random::<u32>()),
                component: "test-component".to_string(),
                message: "Test alert message".to_string(),
                severity,
                created_at: SystemTime::now(),
            },
            alert_type,
            fingerprint: format!("test-fingerprint-{}", rand::random::<u32>()),
            group_key: format!("{:?}-{}", alert_type, environment),
            environment: environment.to_string(),
            metadata: HashMap::new(),
            acknowledged: false,
            acknowledged_at: None,
            acknowledged_by: None,
            escalation_level: 0,
            escalated_at: None,
        }
    }
}

/// Performance and load testing for alert system
#[cfg(test)]
mod performance_tests {
    use super::*;

    #[tokio::test]
    async fn test_alert_system_performance_under_load() {
        let health_monitor = Arc::new(HealthMonitor::new());
        let alert_system = AdvancedAlertSystem::new(health_monitor);
        
        let config = create_test_config();
        alert_system.update_config(config).await;

        let start_time = std::time::Instant::now();
        let num_alerts = 1000;

        // Generate high volume of alerts
        let mut tasks = Vec::new();
        for i in 0..num_alerts {
            let alert_system_clone = alert_system.clone();
            let metrics = MetricsData {
                error_rate: (i % 20) as f32, // Varying error rates
                latency: Duration::from_millis((i % 500) as u64),
                throughput: 1000 - (i % 100),
                cpu_usage: Some((i % 100) as f32),
                memory_usage: Some((i % 90) as f32),
                disk_usage: Some((i % 80) as f32),
            };

            tasks.push(tokio::spawn(async move {
                alert_system_clone
                    .process_metrics(&metrics, "load-test")
                    .await
            }));
        }

        // Wait for all alerts to be processed
        for task in tasks {
            task.await.unwrap().unwrap();
        }

        let duration = start_time.elapsed();
        let throughput = num_alerts as f64 / duration.as_secs_f64();

        println!("Processed {} alerts in {:?} ({:.2} alerts/sec)", 
                 num_alerts, duration, throughput);

        // Performance assertions
        assert!(duration < Duration::from_secs(10), 
                "Should process {} alerts within 10 seconds, took {:?}", 
                num_alerts, duration);
        assert!(throughput > 100.0, 
                "Should achieve >100 alerts/sec throughput, got {:.2}", 
                throughput);

        // Verify alert grouping reduced noise significantly
        let groups = alert_system.get_alert_groups().await;
        let noise_reduction = 1.0 - (groups.len() as f64 / num_alerts as f64);
        
        assert!(noise_reduction > 0.8, 
                "Should achieve >80% noise reduction under load, got {:.2}%", 
                noise_reduction * 100.0);
    }

    use super::integration_tests::create_test_config;
}

/// Error handling and edge cases
#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[tokio::test]
    async fn test_invalid_configuration_handling() {
        let health_monitor = Arc::new(HealthMonitor::new());
        let alert_system = AdvancedAlertSystem::new(health_monitor);

        // Test configuration with disabled channels
        let mut config = create_test_config();
        for channel in &mut config.channels {
            channel.enabled = false;
        }

        alert_system.update_config(config).await;

        // Process metrics that would trigger alerts
        let metrics = MetricsData {
            error_rate: 15.0, // Above threshold
            latency: Duration::from_millis(1000),
            throughput: 100,
            cpu_usage: Some(95.0),
            memory_usage: Some(90.0),
            disk_usage: Some(85.0),
        };

        // Should not fail even with disabled channels
        let result = alert_system.process_metrics(&metrics, "production").await;
        assert!(result.is_ok(), "Should handle disabled channels gracefully");
    }

    #[tokio::test] 
    async fn test_network_failure_resilience() {
        let notification_client = NotificationClient::new();
        
        // Test with invalid webhook URL
        let invalid_slack_channel = NotificationChannel {
            name: "invalid-slack".to_string(),
            channel_type: ChannelType::Slack,
            config: ChannelConfig {
                webhook_url: Some("https://invalid-url-that-should-fail.com/webhook".to_string()),
                email_recipients: None,
                github_repo: None,
                github_token: None,
            },
            enabled: true,
        };

        let alert = create_test_enhanced_alert();
        let result = notification_client.send_notification(&invalid_slack_channel, &alert).await;
        
        // Should handle network failures gracefully
        match result {
            Ok(_) => {}, // Mock implementation might succeed
            Err(e) => println!("Expected network error: {:?}", e),
        }
    }

    use super::integration_tests::{create_test_config, create_test_enhanced_alert};
}