//! Example usage of the UV-246 Automated Reporting System
//!
//! This example demonstrates how to set up and use the complete
//! automated reporting system with configuration, scheduling,
//! and distribution.

use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;

// Import the reporting system components
use uveddi::monitoring::{
    config::presets,
    distribution::{DistributionConfigBuilder, SlackWebhookConfig, SmtpConfig},
    metrics::{ResourceUsage, TestStatus},
    scheduler::{schedules, ScheduleBuilder},
    ConfigManager, DistributionManager, ReportScheduler, ReportType, ReportingEngine,
    StakeholderRole, TestMetrics,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();

    println!("🚀 Starting UV-246 Automated Reporting System Example");

    // Example 1: Load configuration from file
    example_config_loading().await?;

    // Example 2: Programmatic configuration
    example_programmatic_setup().await?;

    // Example 3: Manual report generation
    example_manual_reports().await?;

    // Example 4: Scheduled reporting
    example_scheduled_reporting().await?;

    println!("✅ All examples completed successfully!");

    Ok(())
}

/// Example 1: Loading configuration from file
async fn example_config_loading() -> Result<()> {
    println!("\n📋 Example 1: Loading Configuration from File");

    // Load configuration from TOML file
    let config_manager = ConfigManager::load_from_file("examples/reporting_config.toml")?;

    println!("✓ Configuration loaded successfully");
    println!(
        "  - System enabled: {}",
        config_manager.system_config().enabled
    );
    println!("  - Report jobs: {}", config_manager.report_jobs().len());
    println!(
        "  - Distribution channels: {}",
        config_manager.distribution_config().channels.len()
    );

    // Display enabled report jobs
    for job in config_manager.enabled_report_jobs() {
        println!(
            "  - Job: {} ({:?} for {:?})",
            job.name, job.report_type, job.stakeholder_role
        );
    }

    Ok(())
}

/// Example 2: Programmatic configuration setup
async fn example_programmatic_setup() -> Result<()> {
    println!("\n⚙️ Example 2: Programmatic Configuration");

    // Create SMTP configuration
    let smtp_config = SmtpConfig {
        server: "smtp.example.com".to_string(),
        port: 587,
        username: "reports@uveddi.com".to_string(),
        password: "your-app-password".to_string(),
        use_tls: true,
        from_address: "reports@uveddi.com".to_string(),
        from_name: Some("Uveddi Automated Reports".to_string()),
    };

    // Create Slack webhook configuration
    let mut slack_webhooks = std::collections::HashMap::new();
    slack_webhooks.insert(
        "main".to_string(),
        SlackWebhookConfig {
            webhook_url: "https://hooks.slack.com/services/YOUR/WEBHOOK/URL".to_string(),
            default_channel: "#reports".to_string(),
            username: Some("Uveddi Bot".to_string()),
            icon_emoji: Some(":robot_face:".to_string()),
        },
    );

    // Build distribution manager
    let distribution_manager = DistributionConfigBuilder::new()
        .with_email(smtp_config)
        .with_slack_webhook("main".to_string(), slack_webhooks["main"].clone())
        .build()?;

    println!("✓ Distribution manager configured");
    println!("  - Email: configured");
    println!("  - Slack: {} webhooks", slack_webhooks.len());

    Ok(())
}

/// Example 3: Manual report generation
async fn example_manual_reports() -> Result<()> {
    println!("\n📊 Example 3: Manual Report Generation");

    // Create reporting engine
    let mut reporting_engine = ReportingEngine::new()?;

    // Generate different types of reports
    let report_types = vec![
        (ReportType::DailyHealth, StakeholderRole::QaEngineer),
        (ReportType::WeeklyTrend, StakeholderRole::Manager),
        (ReportType::MonthlyExecutive, StakeholderRole::Executive),
        (ReportType::FailureAnalysis, StakeholderRole::Developer),
        (
            ReportType::PerformanceOptimization,
            StakeholderRole::Developer,
        ),
    ];

    for (report_type, stakeholder) in report_types {
        println!(
            "  Generating {:?} report for {:?}...",
            report_type, stakeholder
        );

        // Create configuration using presets, always convert to ReportConfiguration
        let config: uveddi::monitoring::reporting::ReportConfiguration = match report_type {
            ReportType::DailyHealth => {
                let job_cfg = presets::daily_health_report(
                    stakeholder.clone(),
                    vec!["dev-email".to_string()],
                )?;
                ReportConfiguration::from(job_cfg)
            }
            ReportType::WeeklyTrend => {
                let job_cfg = presets::weekly_trend_report(
                    stakeholder.clone(),
                    vec!["management-email".to_string()],
                )?;
                ReportConfiguration::from(job_cfg)
            }
            ReportType::MonthlyExecutive => {
                let job_cfg = presets::monthly_executive_report(vec!["exec-email".to_string()])?;
                ReportConfiguration::from(job_cfg)
            }
            _ => {
                use uveddi::monitoring::reporting::{
                    ReportConfiguration, ScheduleConfig, TemplateCustomization,
                };
                ReportConfiguration {
                    report_type: report_type.clone(),
                    stakeholder_role: stakeholder.clone(),
                    schedule: ScheduleConfig {
                        cron_expression: "0 9 * * *".to_string(),
                        timezone: "UTC".to_string(),
                        enabled: true,
                    },
                    distribution: vec![],
                    template_customization: TemplateCustomization::default(),
                    enabled: true,
                }
            }
        };

        // Generate the report
        let report = reporting_engine.generate_report(&config).await?;

        println!("    ✓ Report generated: ID {}", report.id);
        println!("    ✓ Total tests: {}", report.content.summary.total_tests);
        println!("    ✓ Pass rate: {:.1}%", report.content.summary.pass_rate);

        // Render the report to HTML
        let rendered_html = reporting_engine.render_report(&report)?;
        println!("    ✓ Rendered HTML: {} characters", rendered_html.len());

        // Save to file (optional)
        let filename = format!(
            "example_report_{}_{:?}.html",
            format!("{:?}", report_type).to_lowercase(),
            format!("{:?}", stakeholder).to_lowercase()
        );
        std::fs::write(&filename, &rendered_html)?;
        println!("    ✓ Saved to: {}", filename);
    }

    Ok(())
}

/// Example 4: Scheduled reporting with monitoring
async fn example_scheduled_reporting() -> Result<()> {
    println!("\n⏰ Example 4: Scheduled Reporting");

    // Create scheduler and reporting engine
    let mut scheduler = ReportScheduler::new();
    let reporting_engine = ReportingEngine::new()?;

    // Add some scheduled jobs
    let daily_schedule = schedules::daily("09:00")?;
    let weekly_schedule = schedules::weekly("Monday", "10:00")?;
    let hourly_schedule = schedules::every_hours(1)?; // For demo purposes

    // Daily health report for developers
    let daily_config =
        presets::daily_health_report(StakeholderRole::Developer, vec!["dev-email".to_string()])?;

    let job1_id = scheduler.add_job(
        "Daily Developer Health".to_string(),
        ReportConfiguration::from(daily_config),
        daily_schedule,
    )?;

    // Weekly trend report for managers
    let weekly_config = presets::weekly_trend_report(
        StakeholderRole::Manager,
        vec!["management-email".to_string()],
    )?;

    let job2_id = scheduler.add_job(
        "Weekly Management Trends".to_string(),
        ReportConfiguration::from(weekly_config),
        weekly_schedule,
    )?;

    // Hourly performance monitoring (demo)
    let perf_config = {
        use uveddi::monitoring::reporting::{
            ReportConfiguration, ScheduleConfig, TemplateCustomization,
        };
        ReportConfiguration {
            report_type: ReportType::PerformanceOptimization,
            stakeholder_role: StakeholderRole::Developer,
            schedule: ScheduleConfig {
                cron_expression: "0 * * * *".to_string(),
                timezone: "UTC".to_string(),
                enabled: true,
            },
            distribution: vec![],
            template_customization: TemplateCustomization::default(),
            enabled: true,
        }
    };

    let job3_id = scheduler.add_job(
        "Hourly Performance Check".to_string(),
        perf_config,
        hourly_schedule,
    )?;

    println!("✓ Scheduled jobs added:");
    println!("  - Daily Developer Health: {}", job1_id);
    println!("  - Weekly Management Trends: {}", job2_id);
    println!("  - Hourly Performance Check: {}", job3_id);

    // Display job status
    for job in scheduler.get_jobs() {
        println!("  Job: {} - Next run: {}", job.name, job.next_run);
    }

    // In a real application, you would start the scheduler and let it run
    // For this example, we'll just demonstrate the setup
    println!("✓ Scheduler configured and ready");
    println!("  (In production, call scheduler.start(reporting_engine).await to begin)");

    // Demonstrate job management
    println!("\n🔧 Job Management:");

    // Disable a job
    scheduler.set_job_enabled(job3_id, false)?;
    println!("  ✓ Disabled hourly performance check");

    // Get execution history (will be empty for this example)
    let recent_history = scheduler.get_recent_history(10);
    println!("  ✓ Recent executions: {}", recent_history.len());

    // Remove a job
    scheduler.remove_job(job3_id)?;
    println!("  ✓ Removed hourly performance check");

    println!("  ✓ Final job count: {}", scheduler.get_jobs().len());

    Ok(())
}

/// Helper function to create mock test metrics for examples
fn create_example_metrics() -> Vec<TestMetrics> {
    let mut metrics = Vec::new();

    for i in 0..100 {
        let status = if i < 85 {
            TestStatus::Passed
        } else if i < 95 {
            TestStatus::Failed
        } else {
            TestStatus::Timeout
        };

        let metric = TestMetrics {
            execution_id: format!("example_exec_{}", i),
            test_name: format!("example_test_{}", i),
            test_suite: if i % 3 == 0 {
                "unit_tests"
            } else {
                "integration_tests"
            }
            .to_string(),
            status: status.clone(),
            duration_ms: 100 + (i as u64 * 10) % 500,
            resource_usage: ResourceUsage {
                cpu_percent: 20.0 + (i as f32 * 2.0) % 60.0,
                memory_mb: 128 + (i as u64 * 8) % 256,
                disk_io_mb: 5 + (i as u64) % 15,
            },
            failure_category: if matches!(status, TestStatus::Failed) {
                Some("assertion_failure".to_string())
            } else {
                None
            },
            timestamp: std::time::SystemTime::now(),
        };

        metrics.push(metric);
    }

    metrics
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_example_functions() -> Result<()> {
        // Test that our example functions work
        example_programmatic_setup().await?;
        example_manual_reports().await?;

        Ok(())
    }
}
