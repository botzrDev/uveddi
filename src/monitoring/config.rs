//! Configuration management for the UV-246 Automated Reporting System
//!
//! Provides structured configuration loading, validation, and management
//! for report generation, distribution, and scheduling.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::distribution::{DistributionChannel, SlackWebhookConfig, SmtpConfig};
use super::reporting::{BrandingConfig, ReportType, StakeholderRole};
use super::scheduler::{RetryConfig, ScheduleConfig};

/// Main configuration structure for the reporting system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportingSystemConfig {
    pub system: SystemConfig,
    pub reports: Vec<ReportJobConfig>,
    pub distribution: DistributionConfig,
    pub templates: TemplateConfig,
    pub storage: StorageConfig,
    pub monitoring: MonitoringConfig,
}

/// System-level configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub enabled: bool,
    pub log_level: String,
    pub max_concurrent_reports: usize,
    pub report_timeout_seconds: u64,
    pub cleanup_retention_days: u32,
}

/// Individual report job configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportJobConfig {
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub report_type: ReportType,
    pub stakeholder_role: StakeholderRole,
    pub schedule: ScheduleConfig,
    pub distribution_channels: Vec<String>, // References to configured channels
    pub template_overrides: Option<TemplateOverrides>,
    pub filters: Option<ReportFilters>,
}

/// Template configuration and overrides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    pub default_branding: BrandingConfig,
    pub custom_templates: HashMap<String, String>, // template_name -> file_path
    pub template_cache_enabled: bool,
    pub template_cache_ttl_seconds: u64,
}

/// Template overrides for specific reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateOverrides {
    pub branding: Option<BrandingConfig>,
    pub custom_sections: Option<Vec<String>>,
    pub metrics_focus: Option<Vec<String>>,
    pub template_name: Option<String>,
}

/// Report filtering configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportFilters {
    pub test_suites: Option<Vec<String>>,
    pub test_names: Option<Vec<String>>,
    pub exclude_patterns: Option<Vec<String>>,
    pub min_duration_ms: Option<u64>,
    pub max_duration_ms: Option<u64>,
    pub failure_categories: Option<Vec<String>>,
}

/// Distribution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionConfig {
    pub email: Option<EmailConfig>,
    pub slack: Option<SlackConfig>,
    pub channels: HashMap<String, ChannelConfig>,
    pub default_retry_config: RetryConfig,
}

/// Email configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub smtp: SmtpConfig,
    pub default_subject_template: String,
    pub rate_limit_per_hour: u32,
}

/// Slack configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    pub webhooks: HashMap<String, SlackWebhookConfig>,
    pub default_webhook: String,
    pub rate_limit_per_minute: u32,
    pub message_format: SlackMessageFormat,
}

/// Slack message formatting options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackMessageFormat {
    pub use_rich_formatting: bool,
    pub include_charts: bool,
    pub max_message_length: usize,
    pub truncate_strategy: TruncateStrategy,
}

/// Text truncation strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TruncateStrategy {
    Simple,
    Smart,
    Summary,
}

/// Channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    pub channel_type: ChannelType,
    pub enabled: bool,
    pub priority: ChannelPriority,
    pub retry_config: Option<RetryConfig>,
    pub rate_limits: Option<RateLimits>,
}

/// Channel type configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelType {
    Email {
        address: String,
        subject_template: Option<String>,
    },
    Slack {
        webhook_name: String,
        channel: String,
        username: Option<String>,
        icon_emoji: Option<String>,
    },
}

/// Channel priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimits {
    pub max_per_minute: u32,
    pub max_per_hour: u32,
    pub max_per_day: u32,
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub report_archive_path: String,
    pub max_reports_in_memory: usize,
    pub archive_retention_days: u32,
    pub compression_enabled: bool,
    pub encryption_enabled: bool,
    pub backup_config: Option<BackupConfig>,
}

/// Backup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    pub enabled: bool,
    pub backup_path: String,
    pub backup_frequency_hours: u32,
    pub max_backups: u32,
}

/// Monitoring and observability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_enabled: bool,
    pub health_check_interval_seconds: u64,
    pub performance_tracking: bool,
    pub error_alerting: AlertingConfig,
    pub audit_logging: AuditConfig,
}

/// Alerting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertingConfig {
    pub enabled: bool,
    pub alert_channels: Vec<String>,
    pub failure_threshold: u32,
    pub escalation_timeout_minutes: u32,
}

/// Audit logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    pub enabled: bool,
    pub log_level: String,
    pub log_format: AuditLogFormat,
    pub retention_days: u32,
}

/// Audit log format options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditLogFormat {
    Json,
    Structured,
    Plain,
}

/// Configuration loader and manager
pub struct ConfigManager {
    config: ReportingSystemConfig,
    config_path: Option<String>,
}

impl ConfigManager {
    /// Load configuration from file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config_str = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.as_ref().display()))?;

        let config: ReportingSystemConfig =
            match path.as_ref().extension().and_then(|s| s.to_str()) {
                Some("toml") => {
                    toml::from_str(&config_str).context("Failed to parse TOML configuration")?
                }
                Some("json") => serde_json::from_str(&config_str)
                    .context("Failed to parse JSON configuration")?,
                Some("yaml") | Some("yml") => {
                    #[cfg(feature = "yaml")]
                    {
                        serde_yaml::from_str(&config_str)
                            .context("Failed to parse YAML configuration")?
                    }
                    #[cfg(not(feature = "yaml"))]
                    {
                        anyhow::bail!("YAML support not enabled in build")
                    }
                }
                _ => anyhow::bail!("Unsupported configuration file format"),
            };

        let manager = Self {
            config,
            config_path: Some(path.as_ref().to_string_lossy().to_string()),
        };

        manager.validate()?;
        Ok(manager)
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self {
            config: ReportingSystemConfig::default(),
            config_path: None,
        }
    }

    /// Get the full configuration
    pub fn config(&self) -> &ReportingSystemConfig {
        &self.config
    }

    /// Get system configuration
    pub fn system_config(&self) -> &SystemConfig {
        &self.config.system
    }

    /// Get all report job configurations
    pub fn report_jobs(&self) -> &[ReportJobConfig] {
        &self.config.reports
    }

    /// Get distribution configuration
    pub fn distribution_config(&self) -> &DistributionConfig {
        &self.config.distribution
    }

    /// Get template configuration
    pub fn template_config(&self) -> &TemplateConfig {
        &self.config.templates
    }

    /// Get storage configuration
    pub fn storage_config(&self) -> &StorageConfig {
        &self.config.storage
    }

    /// Get monitoring configuration
    pub fn monitoring_config(&self) -> &MonitoringConfig {
        &self.config.monitoring
    }

    /// Get enabled report jobs
    pub fn enabled_report_jobs(&self) -> Vec<&ReportJobConfig> {
        self.config
            .reports
            .iter()
            .filter(|job| job.enabled && self.config.system.enabled)
            .collect()
    }

    /// Get distribution channels for a report job
    pub fn get_distribution_channels(
        &self,
        job: &ReportJobConfig,
    ) -> Result<Vec<DistributionChannel>> {
        let mut channels = Vec::new();

        for channel_name in &job.distribution_channels {
            if let Some(channel_config) = self.config.distribution.channels.get(channel_name) {
                if channel_config.enabled {
                    let channel = self.convert_channel_config(channel_config)?;
                    channels.push(channel);
                }
            } else {
                tracing::warn!(
                    "Distribution channel '{}' not found in configuration",
                    channel_name
                );
            }
        }

        Ok(channels)
    }

    /// Convert channel config to distribution channel
    fn convert_channel_config(&self, config: &ChannelConfig) -> Result<DistributionChannel> {
        match &config.channel_type {
            ChannelType::Email {
                address,
                subject_template,
            } => Ok(DistributionChannel::Email {
                address: address.clone(),
                subject_template: subject_template.clone(),
            }),
            ChannelType::Slack {
                webhook_name,
                channel,
                username,
                icon_emoji,
            } => {
                let webhook_config = self
                    .config
                    .distribution
                    .slack
                    .as_ref()
                    .and_then(|slack| slack.webhooks.get(webhook_name))
                    .context("Slack webhook configuration not found")?;

                Ok(DistributionChannel::Slack {
                    webhook_url: webhook_config.webhook_url.clone(),
                    channel: channel.clone(),
                    username: username.clone(),
                    icon_emoji: icon_emoji.clone(),
                })
            }
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate system config
        if self.config.system.max_concurrent_reports == 0 {
            anyhow::bail!("max_concurrent_reports must be greater than 0");
        }

        // Validate report jobs
        for job in &self.config.reports {
            if job.name.is_empty() {
                anyhow::bail!("Report job name cannot be empty");
            }

            // Validate distribution channels exist
            for channel_name in &job.distribution_channels {
                if !self.config.distribution.channels.contains_key(channel_name) {
                    anyhow::bail!(
                        "Distribution channel '{}' referenced in job '{}' does not exist",
                        channel_name,
                        job.name
                    );
                }
            }
        }

        // Validate distribution config
        if let Some(email_config) = &self.config.distribution.email {
            if email_config.smtp.server.is_empty() {
                anyhow::bail!("SMTP server cannot be empty");
            }
            if email_config.smtp.from_address.is_empty() {
                anyhow::bail!("SMTP from_address cannot be empty");
            }
        }

        if let Some(slack_config) = &self.config.distribution.slack {
            if slack_config.webhooks.is_empty() {
                anyhow::bail!("At least one Slack webhook must be configured");
            }
            if !slack_config
                .webhooks
                .contains_key(&slack_config.default_webhook)
            {
                anyhow::bail!(
                    "Default Slack webhook '{}' does not exist",
                    slack_config.default_webhook
                );
            }
        }

        // Validate storage config
        if self.config.storage.max_reports_in_memory == 0 {
            anyhow::bail!("max_reports_in_memory must be greater than 0");
        }

        Ok(())
    }

    /// Save configuration to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = match path.as_ref().extension().and_then(|s| s.to_str()) {
            Some("toml") => toml::to_string_pretty(&self.config)
                .context("Failed to serialize configuration to TOML")?,
            Some("json") => serde_json::to_string_pretty(&self.config)
                .context("Failed to serialize configuration to JSON")?,
            Some("yaml") | Some("yml") => {
                #[cfg(feature = "yaml")]
                {
                    serde_yaml::to_string(&self.config)
                        .context("Failed to serialize configuration to YAML")?
                }
                #[cfg(not(feature = "yaml"))]
                {
                    anyhow::bail!("YAML support not enabled in build")
                }
            }
            _ => anyhow::bail!("Unsupported configuration file format"),
        };

        fs::write(&path, content)
            .with_context(|| format!("Failed to write config file: {}", path.as_ref().display()))?;

        Ok(())
    }

    /// Update configuration and validate
    pub fn update_config(&mut self, config: ReportingSystemConfig) -> Result<()> {
        config.validate()?;
        self.config = config;
        Ok(())
    }

    /// Add a new report job
    pub fn add_report_job(&mut self, job: ReportJobConfig) -> Result<()> {
        // Check for duplicate names
        if self
            .config
            .reports
            .iter()
            .any(|existing| existing.name == job.name)
        {
            anyhow::bail!("Report job with name '{}' already exists", job.name);
        }

        // Validate the job
        for channel_name in &job.distribution_channels {
            if !self.config.distribution.channels.contains_key(channel_name) {
                anyhow::bail!("Distribution channel '{}' does not exist", channel_name);
            }
        }

        self.config.reports.push(job);
        Ok(())
    }

    /// Remove a report job by name
    pub fn remove_report_job(&mut self, name: &str) -> Result<()> {
        let initial_len = self.config.reports.len();
        self.config.reports.retain(|job| job.name != name);

        if self.config.reports.len() == initial_len {
            anyhow::bail!("Report job with name '{}' not found", name);
        }

        Ok(())
    }

    /// Add a distribution channel
    pub fn add_distribution_channel(&mut self, name: String, config: ChannelConfig) -> Result<()> {
        if self.config.distribution.channels.contains_key(&name) {
            anyhow::bail!("Distribution channel '{}' already exists", name);
        }

        self.config.distribution.channels.insert(name, config);
        Ok(())
    }
}

impl Default for ReportingSystemConfig {
    fn default() -> Self {
        Self {
            system: SystemConfig::default(),
            reports: Vec::new(),
            distribution: DistributionConfig::default(),
            templates: TemplateConfig::default(),
            storage: StorageConfig::default(),
            monitoring: MonitoringConfig::default(),
        }
    }
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_level: "info".to_string(),
            max_concurrent_reports: 5,
            report_timeout_seconds: 300,
            cleanup_retention_days: 30,
        }
    }
}

impl Default for TemplateConfig {
    fn default() -> Self {
        Self {
            default_branding: BrandingConfig::default(),
            custom_templates: HashMap::new(),
            template_cache_enabled: true,
            template_cache_ttl_seconds: 3600,
        }
    }
}

impl Default for DistributionConfig {
    fn default() -> Self {
        Self {
            email: None,
            slack: None,
            channels: HashMap::new(),
            default_retry_config: RetryConfig::default(),
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            report_archive_path: "./reports".to_string(),
            max_reports_in_memory: 1000,
            archive_retention_days: 90,
            compression_enabled: true,
            encryption_enabled: false,
            backup_config: None,
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            metrics_enabled: true,
            health_check_interval_seconds: 60,
            performance_tracking: true,
            error_alerting: AlertingConfig::default(),
            audit_logging: AuditConfig::default(),
        }
    }
}

impl Default for AlertingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            alert_channels: Vec::new(),
            failure_threshold: 3,
            escalation_timeout_minutes: 30,
        }
    }
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_level: "info".to_string(),
            log_format: AuditLogFormat::Json,
            retention_days: 30,
        }
    }
}

impl Default for SlackMessageFormat {
    fn default() -> Self {
        Self {
            use_rich_formatting: true,
            include_charts: false,
            max_message_length: 4000,
            truncate_strategy: TruncateStrategy::Smart,
        }
    }
}

/// Configuration builder for easier setup
pub struct ConfigBuilder {
    config: ReportingSystemConfig,
}

impl ConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            config: ReportingSystemConfig::default(),
        }
    }

    /// Configure system settings
    pub fn with_system_config(mut self, config: SystemConfig) -> Self {
        self.config.system = config;
        self
    }

    /// Add a report job
    pub fn with_report_job(mut self, job: ReportJobConfig) -> Self {
        self.config.reports.push(job);
        self
    }

    /// Configure distribution
    pub fn with_distribution_config(mut self, config: DistributionConfig) -> Self {
        self.config.distribution = config;
        self
    }

    /// Configure templates
    pub fn with_template_config(mut self, config: TemplateConfig) -> Self {
        self.config.templates = config;
        self
    }

    /// Configure storage
    pub fn with_storage_config(mut self, config: StorageConfig) -> Self {
        self.config.storage = config;
        self
    }

    /// Configure monitoring
    pub fn with_monitoring_config(mut self, config: MonitoringConfig) -> Self {
        self.config.monitoring = config;
        self
    }

    /// Build the configuration
    pub fn build(self) -> Result<ReportingSystemConfig> {
        self.config.validate()?;
        Ok(self.config)
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait for configuration validation
trait ConfigValidation {
    fn validate(&self) -> Result<()>;
}

impl ConfigValidation for ReportingSystemConfig {
    fn validate(&self) -> Result<()> {
        // This is implemented in ConfigManager::validate
        // but we could add more specific validation here
        Ok(())
    }
}

/// Helper functions for creating common configurations
pub mod presets {
    use super::*;
    use crate::monitoring::scheduler::schedules;

    /// Create a basic daily health report configuration
    pub fn daily_health_report(
        stakeholder: StakeholderRole,
        channels: Vec<String>,
    ) -> Result<ReportJobConfig> {
        Ok(ReportJobConfig {
            name: format!("Daily Health Report - {:?}", stakeholder),
            description: Some("Automated daily health summary".to_string()),
            enabled: true,
            report_type: ReportType::DailyHealth,
            stakeholder_role: stakeholder,
            schedule: schedules::daily("09:00")?,
            distribution_channels: channels,
            template_overrides: None,
            filters: None,
        })
    }

    /// Create a weekly trend analysis configuration
    pub fn weekly_trend_report(
        stakeholder: StakeholderRole,
        channels: Vec<String>,
    ) -> Result<ReportJobConfig> {
        Ok(ReportJobConfig {
            name: format!("Weekly Trend Analysis - {:?}", stakeholder),
            description: Some("Weekly performance and trend analysis".to_string()),
            enabled: true,
            report_type: ReportType::WeeklyTrend,
            stakeholder_role: stakeholder,
            schedule: schedules::weekly("Monday", "10:00")?,
            distribution_channels: channels,
            template_overrides: None,
            filters: None,
        })
    }

    /// Create a monthly executive summary configuration
    pub fn monthly_executive_report(channels: Vec<String>) -> Result<ReportJobConfig> {
        Ok(ReportJobConfig {
            name: "Monthly Executive Summary".to_string(),
            description: Some("Monthly high-level quality and performance summary".to_string()),
            enabled: true,
            report_type: ReportType::MonthlyExecutive,
            stakeholder_role: StakeholderRole::Executive,
            schedule: schedules::monthly(1, "08:00")?,
            distribution_channels: channels,
            template_overrides: None,
            filters: None,
        })
    }
}
