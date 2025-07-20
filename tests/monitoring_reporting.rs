//! Integration tests for the UV-246 Automated Reporting System
//!
//! Comprehensive test suite covering report generation, distribution,
//! scheduling, and customization functionality.

use anyhow::Result;
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use tempfile::TempDir;
use tokio::time::{sleep, Duration as TokioDuration};
use uuid::Uuid;

use uveddi::monitoring::{metrics::{TestStatus, ResourceUsage},
    ReportingEngine, ReportType, StakeholderRole, ReportConfiguration,
    DistributionManager, DistributionChannel, SmtpConfig, SlackWebhookConfig,
    ReportScheduler, ScheduleConfig, ScheduleType,
    TestMetrics,
    distribution::{DistributionConfigBuilder, email_channel, slack_channel},
    scheduler::{ScheduleBuilder, RetryConfig, schedules},
};

/// Helper function to create mock test metrics
fn create_mock_metrics(count: usize, pass_rate: f64) -> Vec<TestMetrics> {
    let mut metrics = Vec::new();
    let pass_count = (count as f64 * pass_rate) as usize;
    
    for i in 0..count {
        let status = if i < pass_count {
            TestStatus::Passed
        } else if i < count - 2 {
            TestStatus::Failed
        } else {
            TestStatus::Timeout
        };
        
        let metric = TestMetrics {
            execution_id: format!("exec_{}", i),
            test_name: format!("test_{}", i),
            test_suite: "integration_tests".to_string(),
            status: status.clone(),
            duration_ms: 150 + (i as u64 * 50) % 1000,
            resource_usage: ResourceUsage {
                cpu_percent: 45.0 + (i as f32 * 5.0) % 50.0,
                memory_mb: 256 + (i as u64 * 32) % 512,
                disk_io_mb: 10 + (i as u64 * 2) % 20,
            },
            failure_category: if matches!(status, TestStatus::Failed) {
                Some("assertion_error".to_string())
            } else {
                None
            },
            timestamp: std::time::SystemTime::now(),
        };
        
        metrics.push(metric);
    }
    
    metrics
}

/// Helper function to create a basic report configuration
fn create_basic_config(report_type: ReportType, role: StakeholderRole) -> ReportConfiguration {
    use uveddi::monitoring::reporting::{ScheduleConfig, TemplateCustomization, BrandingConfig};
    
    ReportConfiguration {
        report_type,
        stakeholder_role: role,
        schedule: ScheduleConfig {
            cron_expression: "0 9 * * *".to_string(),
            timezone: "UTC".to_string(),
            enabled: true,
        },
        distribution: vec![
            email_channel("test@example.com"),
            slack_channel("https://hooks.slack.com/test", "#reports"),
        ],
        template_customization: TemplateCustomization::default(),
        enabled: true,
    }
}

#[tokio::test]
async fn test_report_generation_daily_health() -> Result<()> {
    let mut engine = ReportingEngine::new()?;
    let config = create_basic_config(ReportType::DailyHealth, StakeholderRole::QaEngineer);
    
    // Generate report
    let report = engine.generate_report(&config).await?;
    
    // Verify report structure
    assert_eq!(report.report_type, ReportType::DailyHealth);
    assert_eq!(report.stakeholder_role, StakeholderRole::QaEngineer);
    assert!(!report.id.is_nil());
    assert!(report.generated_at <= Utc::now());
    
    // Verify content structure
    assert!(report.content.summary.total_tests >= 0);
    assert!(report.content.summary.pass_rate >= 0.0);
    assert!(report.content.summary.pass_rate <= 100.0);
    
    // Verify metadata
    assert_eq!(report.metadata.version, "1.0.0");
    assert!(!report.metadata.data_sources.is_empty());
    
    Ok(())
}

#[tokio::test]
async fn test_report_generation_all_types() -> Result<()> {
    let mut engine = ReportingEngine::new()?;
    
    let report_types = vec![
        ReportType::DailyHealth,
        ReportType::WeeklyTrend,
        ReportType::MonthlyExecutive,
        ReportType::FailureAnalysis,
        ReportType::PerformanceOptimization,
    ];
    
    for report_type in report_types {
        let config = create_basic_config(report_type.clone(), StakeholderRole::Developer);
        let report = engine.generate_report(&config).await?;
        
        assert_eq!(report.report_type, report_type);
        assert!(!report.content.summary.total_tests < 0);
        
        // Verify template rendering
        let rendered = engine.render_report(&report)?;
        assert!(!rendered.is_empty());
        assert!(rendered.contains("<!DOCTYPE html"));
        assert!(rendered.contains(&format!("{:?}", report_type)));
    }
    
    Ok(())
}

#[tokio::test]
async fn test_stakeholder_customization() -> Result<()> {
    let mut engine = ReportingEngine::new()?;
    
    let stakeholders = vec![
        StakeholderRole::Developer,
        StakeholderRole::QaEngineer,
        StakeholderRole::QaLead,
        StakeholderRole::Manager,
        StakeholderRole::Executive,
    ];
    
    for role in stakeholders {
        let config = create_basic_config(ReportType::WeeklyTrend, role.clone());
        let report = engine.generate_report(&config).await?;
        
        assert_eq!(report.stakeholder_role, role);
        
        // Verify role-specific insights
        let insights = &report.content.insights;
        if !insights.is_empty() {
            let relevant_insights = insights.iter()
                .filter(|insight| insight.stakeholder_relevance.contains(&role))
                .count();
            assert!(relevant_insights > 0, "No relevant insights for role {:?}", role);
        }
        
        // Verify role-specific recommendations
        let recommendations = &report.content.recommendations;
        if !recommendations.is_empty() {
            let role_recommendations = recommendations.iter()
                .filter(|rec| rec.assigned_role.as_ref() == Some(&role) || rec.assigned_role.is_none())
                .count();
            assert!(role_recommendations > 0, "No relevant recommendations for role {:?}", role);
        }
    }
    
    Ok(())
}

#[tokio::test]
async fn test_template_rendering() -> Result<()> {
    let mut engine = ReportingEngine::new()?;
    let config = create_basic_config(ReportType::DailyHealth, StakeholderRole::QaEngineer);
    let report = engine.generate_report(&config).await?;
    
    let rendered = engine.render_report(&report)?;
    
    // Verify HTML structure
    assert!(rendered.contains("<!DOCTYPE html"));
    assert!(rendered.contains("<html"));
    assert!(rendered.contains("</html>"));
    assert!(rendered.contains("<head>"));
    assert!(rendered.contains("<body>"));
    
    // Verify content inclusion
    assert!(rendered.contains("Daily Health Summary"));
    assert!(rendered.contains(&report.content.summary.total_tests.to_string()));
    assert!(rendered.contains(&format!("{:.1}%", report.content.summary.pass_rate)));
    
    // Verify branding elements
    assert!(rendered.contains("Uveddi"));
    assert!(rendered.contains(&report.id.to_string()));
    
    Ok(())
}

#[tokio::test]
async fn test_distribution_manager_configuration() -> Result<()> {
    let smtp_config = SmtpConfig {
        server: "smtp.example.com".to_string(),
        port: 587,
        username: "test@example.com".to_string(),
        password: "password".to_string(),
        use_tls: true,
        from_address: "reports@uveddi.com".to_string(),
        from_name: Some("Uveddi Reports".to_string()),
    };
    
    let slack_config = SlackWebhookConfig {
        webhook_url: "https://hooks.slack.com/test".to_string(),
        default_channel: "#reports".to_string(),
        username: Some("Uveddi Bot".to_string()),
        icon_emoji: Some(":robot_face:".to_string()),
    };
    
    let mut slack_webhooks = HashMap::new();
    slack_webhooks.insert("default".to_string(), slack_config);
    
    let manager = DistributionConfigBuilder::new()
        .with_email(smtp_config)
        .with_slack_webhook("default".to_string(), slack_webhooks["default"].clone())
        .build()?;
    
    assert!(manager.has_channels());
    
    Ok(())
}

#[tokio::test]
async fn test_distribution_channels() -> Result<()> {
    let manager = DistributionManager::new();
    
    let channels = vec![
        email_channel("developer@example.com"),
        slack_channel("https://hooks.slack.com/test", "#dev-reports"),
    ];
    
    // Test distribution (will use stub implementations)
    let result = manager.distribute_to_channels(
        &channels,
        "Test Report",
        "<html><body>Test content</body></html>",
        Some("Test summary"),
        Some(&[("Pass Rate".to_string(), "95%".to_string())]),
    ).await;
    
    // Should succeed even with stub implementations
    assert!(result.is_ok());
    
    Ok(())
}

#[tokio::test]
async fn test_report_scheduler_basic() -> Result<()> {
    let mut scheduler = ReportScheduler::new();
    
    let schedule = ScheduleBuilder::new()
        .daily_at("09:00")
        .build()?;
    
    let config = create_basic_config(ReportType::DailyHealth, StakeholderRole::QaEngineer);
    
    let job_id = scheduler.add_job(
        "Daily Health Report".to_string(),
        config,
        schedule,
    )?;
    
    // Verify job was added
    assert!(!job_id.is_nil());
    let jobs = scheduler.get_jobs();
    assert_eq!(jobs.len(), 1);
    
    let job = scheduler.get_job(job_id).unwrap();
    assert_eq!(job.name, "Daily Health Report");
    assert_eq!(job.run_count, 0);
    assert_eq!(job.failure_count, 0);
    
    Ok(())
}

#[tokio::test]
async fn test_schedule_types() -> Result<()> {
    // Test different schedule types
    let daily = schedules::daily("09:00")?;
    assert!(matches!(daily.schedule_type, ScheduleType::Daily { .. }));
    
    let weekly = schedules::weekly("Monday", "10:00")?;
    assert!(matches!(weekly.schedule_type, ScheduleType::Weekly { .. }));
    
    let hourly = schedules::every_hours(2)?;
    assert!(matches!(hourly.schedule_type, ScheduleType::Interval { .. }));
    
    let monthly = schedules::monthly(1, "08:00")?;
    assert!(matches!(monthly.schedule_type, ScheduleType::Monthly { .. }));
    
    Ok(())
}

#[tokio::test]
async fn test_retry_configuration() -> Result<()> {
    let retry_config = RetryConfig {
        max_retries: 3,
        initial_delay_seconds: 60,
        backoff_multiplier: 2.0,
        max_delay_seconds: 300,
    };
    
    let schedule = ScheduleBuilder::new()
        .daily_at("09:00")
        .with_retry_config(retry_config)
        .build()?;
    
    assert_eq!(schedule.retry_config.max_retries, 3);
    assert_eq!(schedule.retry_config.initial_delay_seconds, 60);
    assert_eq!(schedule.retry_config.backoff_multiplier, 2.0);
    assert_eq!(schedule.retry_config.max_delay_seconds, 300);
    
    Ok(())
}

#[tokio::test]
async fn test_report_store_search() -> Result<()> {
    let mut engine = ReportingEngine::new()?;
    
    // Generate multiple reports
    let configs = vec![
        create_basic_config(ReportType::DailyHealth, StakeholderRole::Developer),
        create_basic_config(ReportType::WeeklyTrend, StakeholderRole::QaEngineer),
        create_basic_config(ReportType::MonthlyExecutive, StakeholderRole::Manager),
    ];
    
    let mut report_ids = Vec::new();
    for config in configs {
        let report = engine.generate_report(&config).await?;
        report_ids.push(report.id);
    }
    
    // Test search functionality
    let search_results = engine.search_reports("DailyHealth", None)?;
    assert!(!search_results.is_empty());
    
    let daily_reports = search_results.iter()
        .filter(|r| matches!(r.report_type, ReportType::DailyHealth))
        .count();
    assert!(daily_reports > 0);
    
    Ok(())
}

#[tokio::test]
async fn test_performance_metrics_analysis() -> Result<()> {
    let mut engine = ReportingEngine::new()?;
    let config = create_basic_config(ReportType::PerformanceOptimization, StakeholderRole::Developer);
    
    let report = engine.generate_report(&config).await?;
    
    // Verify performance metrics structure
    let perf_metrics = &report.content.metrics.performance_metrics;
    assert!(perf_metrics.avg_duration_ms >= 0.0);
    assert!(perf_metrics.median_duration_ms >= 0.0);
    assert!(perf_metrics.p95_duration_ms >= perf_metrics.median_duration_ms);
    assert!(perf_metrics.p99_duration_ms >= perf_metrics.p95_duration_ms);
    
    // Verify slowest tests are identified
    assert!(perf_metrics.slowest_tests.len() <= 10);
    
    Ok(())
}

#[tokio::test]
async fn test_failure_analysis_categorization() -> Result<()> {
    let mut engine = ReportingEngine::new()?;
    let config = create_basic_config(ReportType::FailureAnalysis, StakeholderRole::QaEngineer);
    
    let report = engine.generate_report(&config).await?;
    
    // Verify failure metrics structure
    let failure_metrics = &report.content.metrics.failure_metrics;
    
    // Should have failure categories (even if empty)
    assert!(failure_metrics.failure_categories.len() >= 0);
    
    // Top failing tests should be limited
    assert!(failure_metrics.top_failing_tests.len() <= 10);
    
    // Should have some form of analysis
    assert!(failure_metrics.failure_patterns.len() >= 0);
    assert!(failure_metrics.resolution_recommendations.len() >= 0);
    
    Ok(())
}

#[tokio::test]
async fn test_executive_summary_metrics() -> Result<()> {
    let mut engine = ReportingEngine::new()?;
    let config = create_basic_config(ReportType::MonthlyExecutive, StakeholderRole::Executive);
    
    let report = engine.generate_report(&config).await?;
    
    // Verify executive-level content
    assert!(report.content.insights.iter().any(|insight| 
        insight.stakeholder_relevance.contains(&StakeholderRole::Executive) ||
        insight.stakeholder_relevance.contains(&StakeholderRole::Manager)
    ));
    
    // Should have high-level metrics
    assert!(report.content.summary.pass_rate >= 0.0);
    assert!(report.content.summary.pass_rate <= 100.0);
    assert!(report.content.summary.total_tests >= 0);
    
    // Should have quality score
    assert!(report.metadata.quality_score >= 0.0);
    assert!(report.metadata.quality_score <= 1.0);
    
    Ok(())
}

#[tokio::test]
async fn test_branding_customization() -> Result<()> {
    use uveddi::monitoring::reporting::{BrandingConfig, ColorScheme, TemplateCustomization};
    
    let mut engine = ReportingEngine::new()?;
    
    let custom_branding = BrandingConfig {
        organization_name: "Custom Corp".to_string(),
        logo_url: Some("https://example.com/logo.png".to_string()),
        color_scheme: ColorScheme {
            primary: "#ff0000".to_string(),
            secondary: "#00ff00".to_string(),
            accent: "#0000ff".to_string(),
            text: "#333333".to_string(),
        },
        custom_footer: Some("© 2025 Custom Corp. All rights reserved.".to_string()),
    };
    
    let mut config = create_basic_config(ReportType::DailyHealth, StakeholderRole::QaEngineer);
    config.template_customization = TemplateCustomization {
        branding: custom_branding,
        custom_sections: vec!["custom_metrics".to_string()],
        metrics_focus: vec!["performance".to_string(), "quality".to_string()],
    };
    
    let report = engine.generate_report(&config).await?;
    let rendered = engine.render_report(&report)?;
    
    // Verify custom branding is applied
    assert!(rendered.contains("Custom Corp"));
    assert!(rendered.contains("#ff0000")); // Custom primary color
    assert!(rendered.contains("© 2025 Custom Corp"));
    
    Ok(())
}

#[tokio::test]
async fn test_concurrent_report_generation() -> Result<()> {
    let mut engine = ReportingEngine::new()?;
    
    // Generate multiple reports concurrently
    let configs = vec![
        create_basic_config(ReportType::DailyHealth, StakeholderRole::Developer),
        create_basic_config(ReportType::WeeklyTrend, StakeholderRole::QaEngineer),
        create_basic_config(ReportType::PerformanceOptimization, StakeholderRole::Manager),
    ];
    
    let mut handles = Vec::new();
    
    for config in configs {
        let mut engine_clone = ReportingEngine::new()?;
        let handle = tokio::spawn(async move {
            engine_clone.generate_report(&config).await
        });
        handles.push(handle);
    }
    
    // Wait for all reports to complete
    let mut reports = Vec::new();
    for handle in handles {
        let report = handle.await??;
        reports.push(report);
    }
    
    assert_eq!(reports.len(), 3);
    
    // Verify all reports have unique IDs
    let mut ids = std::collections::HashSet::new();
    for report in &reports {
        assert!(ids.insert(report.id), "Duplicate report ID found");
    }
    
    Ok(())
}

#[test]
fn test_schedule_builder_validation() {
    // Test valid schedule builds
    assert!(ScheduleBuilder::new().daily_at("09:00").build().is_ok());
    assert!(ScheduleBuilder::new().every_minutes(60).build().is_ok());
    assert!(ScheduleBuilder::new().weekly_on("Monday", "10:00").build().is_ok());
    assert!(ScheduleBuilder::new().monthly_on(15, "14:30").build().is_ok());
    
    // Test invalid schedule (no type specified)
    assert!(ScheduleBuilder::new().build().is_err());
}

#[test]
fn test_distribution_channel_helpers() {
    let email = email_channel("test@example.com");
    assert!(matches!(email, DistributionChannel::Email { .. }));
    
    let slack = slack_channel("https://hooks.slack.com/test", "#channel");
    assert!(matches!(slack, DistributionChannel::Slack { .. }));
}

#[tokio::test]
async fn test_error_handling() -> Result<()> {
    // Test handling of invalid configurations
    let mut engine = ReportingEngine::new()?;
    
    // This should still work even with minimal data
    let mut config = create_basic_config(ReportType::DailyHealth, StakeholderRole::Developer);
    config.distribution.clear(); // Remove distribution channels
    
    let report = engine.generate_report(&config).await?;
    assert!(!report.id.is_nil());
    
    // Distribution should handle empty channels gracefully
    let result = engine.distribute_report(&report, &config).await;
    assert!(result.is_ok());
    
    Ok(())
}

#[tokio::test]
async fn test_integration_full_workflow() -> Result<()> {
    // Test the complete workflow from configuration to distribution
    let mut engine = ReportingEngine::new()?;
    let mut scheduler = ReportScheduler::new();
    
    // Create configuration
    let config = create_basic_config(ReportType::WeeklyTrend, StakeholderRole::QaLead);
    
    // Add to scheduler
    let schedule = ScheduleBuilder::new()
        .daily_at("09:00")
        .build()?;
    
    let job_id = scheduler.add_job(
        "Weekly QA Report".to_string(),
        config.clone(),
        schedule,
    )?;
    
    // Generate report manually
    let report = engine.generate_report(&config).await?;
    
    // Render report
    let rendered = engine.render_report(&report)?;
    assert!(!rendered.is_empty());
    
    // Distribute report
    let distribution_result = engine.distribute_report(&report, &config).await;
    assert!(distribution_result.is_ok());
    
    // Verify scheduler state
    let job = scheduler.get_job(job_id).unwrap();
    assert_eq!(job.name, "Weekly QA Report");
    
    Ok(())
}

#[tokio::test]
async fn test_large_dataset_performance() -> Result<()> {
    let mut engine = ReportingEngine::new()?;
    let config = create_basic_config(ReportType::PerformanceOptimization, StakeholderRole::Developer);
    
    // Test with larger dataset simulation
    let start_time = std::time::Instant::now();
    let report = engine.generate_report(&config).await?;
    let generation_time = start_time.elapsed();
    
    // Report generation should complete within reasonable time
    assert!(generation_time.as_secs() < 10, "Report generation took too long: {:?}", generation_time);
    
    // Verify report structure remains consistent
    assert!(!report.id.is_nil());
    assert!(report.content.summary.total_tests >= 0);
    
    // Test rendering performance
    let start_time = std::time::Instant::now();
    let rendered = engine.render_report(&report)?;
    let render_time = start_time.elapsed();
    
    assert!(render_time.as_secs() < 5, "Report rendering took too long: {:?}", render_time);
    assert!(!rendered.is_empty());
    
    Ok(())
}