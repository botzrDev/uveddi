//! Distribution channels for automated reports
//!
//! Provides email and Slack integration for report delivery

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(feature = "reqwest")]
use reqwest;

/// Distribution channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DistributionChannel {
    Email {
        address: String,
        subject_template: Option<String>,
    },
    Slack {
        webhook_url: String,
        channel: String,
        username: Option<String>,
        icon_emoji: Option<String>,
    },
}

/// Email client for sending reports
pub struct EmailClient {
    smtp_config: SmtpConfig,
}

/// Slack client for sending notifications
pub struct SlackClient {
    #[cfg(feature = "reqwest")]
    http_client: reqwest::Client,
    webhook_configs: HashMap<String, SlackWebhookConfig>,
}

/// SMTP configuration for email delivery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub use_tls: bool,
    pub from_address: String,
    pub from_name: Option<String>,
}

/// Slack webhook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackWebhookConfig {
    pub webhook_url: String,
    pub default_channel: String,
    pub username: Option<String>,
    pub icon_emoji: Option<String>,
}

/// Slack message payload
#[derive(Debug, Serialize)]
struct SlackMessage {
    text: String,
    channel: Option<String>,
    username: Option<String>,
    icon_emoji: Option<String>,
    attachments: Option<Vec<SlackAttachment>>,
}

/// Slack attachment for rich formatting
#[derive(Debug, Serialize)]
struct SlackAttachment {
    color: String,
    title: String,
    text: String,
    fields: Option<Vec<SlackField>>,
}

/// Slack field for structured data
#[derive(Debug, Serialize)]
struct SlackField {
    title: String,
    value: String,
    short: bool,
}

/// Distribution manager handles all delivery channels
pub struct DistributionManager {
    email_client: Option<EmailClient>,
    slack_client: Option<SlackClient>,
}

impl EmailClient {
    /// Create a new email client with SMTP configuration
    pub fn new(config: SmtpConfig) -> Self {
        Self {
            smtp_config: config,
        }
    }

    /// Send an email report
    pub async fn send_email(&self, to: &str, subject: &str, html_content: &str) -> Result<()> {
        // For now, we'll log the email sending attempt
        // In a real implementation, this would use an SMTP library like lettre
        tracing::info!("Sending email to {} with subject: {}", to, subject);
        tracing::debug!("Email content length: {} characters", html_content.len());

        // TODO: Implement actual SMTP sending using lettre crate
        // This would require adding lettre to Cargo.toml dependencies

        Ok(())
    }

    /// Validate email configuration
    pub fn validate_config(&self) -> Result<()> {
        if self.smtp_config.server.is_empty() {
            anyhow::bail!("SMTP server not configured");
        }
        if self.smtp_config.from_address.is_empty() {
            anyhow::bail!("From address not configured");
        }
        Ok(())
    }
}

impl SlackClient {
    /// Create a new Slack client
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "reqwest")]
            http_client: reqwest::Client::new(),
            webhook_configs: HashMap::new(),
        }
    }

    /// Add a webhook configuration
    pub fn add_webhook(&mut self, name: String, config: SlackWebhookConfig) {
        self.webhook_configs.insert(name, config);
    }

    /// Send a simple text message to Slack
    pub async fn send_message(
        &self,
        webhook_name: &str,
        channel: &str,
        message: &str,
    ) -> Result<()> {
        let config = self
            .webhook_configs
            .get(webhook_name)
            .context("Webhook configuration not found")?;

        let payload = SlackMessage {
            text: message.to_string(),
            channel: Some(channel.to_string()),
            username: config.username.clone(),
            icon_emoji: config.icon_emoji.clone(),
            attachments: None,
        };

        self.send_slack_payload(&config.webhook_url, &payload).await
    }

    /// Send a rich report notification to Slack
    pub async fn send_report_notification(
        &self,
        webhook_name: &str,
        channel: &str,
        report_title: &str,
        summary: &str,
        metrics: &[(String, String)],
        report_url: Option<&str>,
    ) -> Result<()> {
        let config = self
            .webhook_configs
            .get(webhook_name)
            .context("Webhook configuration not found")?;

        let mut fields = metrics
            .iter()
            .map(|(key, value)| SlackField {
                title: key.clone(),
                value: value.clone(),
                short: true,
            })
            .collect::<Vec<_>>();

        if let Some(url) = report_url {
            fields.push(SlackField {
                title: "View Report".to_string(),
                value: format!("<{}|Open Full Report>", url),
                short: false,
            });
        }

        let attachment = SlackAttachment {
            color: "#007bff".to_string(),
            title: report_title.to_string(),
            text: summary.to_string(),
            fields: Some(fields),
        };

        let payload = SlackMessage {
            text: format!("📊 New Report Available: {}", report_title),
            channel: Some(channel.to_string()),
            username: config.username.clone(),
            icon_emoji: config.icon_emoji.clone(),
            attachments: Some(vec![attachment]),
        };

        self.send_slack_payload(&config.webhook_url, &payload).await
    }

    /// Send payload to Slack webhook
    #[cfg(feature = "reqwest")]
    async fn send_slack_payload(&self, webhook_url: &str, payload: &SlackMessage) -> Result<()> {
        let response = self
            .http_client
            .post(webhook_url)
            .json(payload)
            .send()
            .await
            .context("Failed to send Slack message")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Slack API error {}: {}", status, body);
        }

        tracing::info!("Successfully sent Slack notification");
        Ok(())
    }

    /// Send payload to Slack webhook (stub for when reqwest is not available)
    #[cfg(not(feature = "reqwest"))]
    async fn send_slack_payload(&self, webhook_url: &str, payload: &SlackMessage) -> Result<()> {
        tracing::info!("Would send Slack message to {}: {:?}", webhook_url, payload);
        Ok(())
    }
}

impl DistributionManager {
    /// Create a new distribution manager
    pub fn new() -> Self {
        Self {
            email_client: None,
            slack_client: None,
        }
    }

    /// Configure email client
    pub fn configure_email(&mut self, config: SmtpConfig) -> Result<()> {
        let client = EmailClient::new(config);
        client.validate_config()?;
        self.email_client = Some(client);
        Ok(())
    }

    /// Configure Slack client
    pub fn configure_slack(&mut self, webhooks: HashMap<String, SlackWebhookConfig>) {
        let mut client = SlackClient::new();
        for (name, config) in webhooks {
            client.add_webhook(name, config);
        }
        self.slack_client = Some(client);
    }

    /// Send report via email
    pub async fn send_email(&self, to: &str, subject: &str, content: &str) -> Result<()> {
        match &self.email_client {
            Some(client) => client.send_email(to, subject, content).await,
            None => {
                tracing::warn!("Email client not configured, skipping email delivery");
                Ok(())
            }
        }
    }

    /// Send simple Slack message
    pub async fn send_slack_message(
        &self,
        webhook_name: &str,
        channel: &str,
        message: &str,
    ) -> Result<()> {
        match &self.slack_client {
            Some(client) => client.send_message(webhook_name, channel, message).await,
            None => {
                tracing::warn!("Slack client not configured, skipping Slack delivery");
                Ok(())
            }
        }
    }

    /// Send rich Slack notification for reports
    pub async fn send_slack_report_notification(
        &self,
        webhook_name: &str,
        channel: &str,
        report_title: &str,
        summary: &str,
        metrics: &[(String, String)],
        report_url: Option<&str>,
    ) -> Result<()> {
        match &self.slack_client {
            Some(client) => {
                client
                    .send_report_notification(
                        webhook_name,
                        channel,
                        report_title,
                        summary,
                        metrics,
                        report_url,
                    )
                    .await
            }
            None => {
                tracing::warn!("Slack client not configured, skipping Slack delivery");
                Ok(())
            }
        }
    }

    /// Distribute content to all configured channels
    pub async fn distribute_to_channels(
        &self,
        channels: &[DistributionChannel],
        subject: &str,
        content: &str,
        summary: Option<&str>,
        metrics: Option<&[(String, String)]>,
    ) -> Result<()> {
        for channel in channels {
            match channel {
                DistributionChannel::Email {
                    address,
                    subject_template,
                } => {
                    let email_subject = subject_template
                        .as_ref()
                        .map(|template| template.replace("{subject}", subject))
                        .unwrap_or_else(|| subject.to_string());

                    self.send_email(address, &email_subject, content).await?;
                }
                DistributionChannel::Slack {
                    webhook_url: _,
                    channel,
                    username: _,
                    icon_emoji: _,
                } => {
                    if let (Some(summary), Some(metrics)) = (summary, metrics) {
                        self.send_slack_report_notification(
                            "default", // webhook name
                            channel, subject, summary, metrics, None, // report URL
                        )
                        .await?;
                    } else {
                        self.send_slack_message(
                            "default",
                            channel,
                            &format!("{}\n\n{}", subject, content),
                        )
                        .await?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Check if any distribution channels are configured
    pub fn has_channels(&self) -> bool {
        self.email_client.is_some() || self.slack_client.is_some()
    }
}

impl Default for DistributionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating distribution configurations
pub struct DistributionConfigBuilder {
    email_config: Option<SmtpConfig>,
    slack_webhooks: HashMap<String, SlackWebhookConfig>,
}

impl DistributionConfigBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            email_config: None,
            slack_webhooks: HashMap::new(),
        }
    }

    /// Configure email settings
    pub fn with_email(mut self, config: SmtpConfig) -> Self {
        self.email_config = Some(config);
        self
    }

    /// Add a Slack webhook configuration
    pub fn with_slack_webhook(mut self, name: String, config: SlackWebhookConfig) -> Self {
        self.slack_webhooks.insert(name, config);
        self
    }

    /// Build the distribution manager
    pub fn build(self) -> Result<DistributionManager> {
        let mut manager = DistributionManager::new();

        if let Some(email_config) = self.email_config {
            manager.configure_email(email_config)?;
        }

        if !self.slack_webhooks.is_empty() {
            manager.configure_slack(self.slack_webhooks);
        }

        Ok(manager)
    }
}

impl Default for DistributionConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to create email distribution channel
pub fn email_channel(address: &str) -> DistributionChannel {
    DistributionChannel::Email {
        address: address.to_string(),
        subject_template: None,
    }
}

/// Helper function to create email distribution channel with custom subject
pub fn email_channel_with_subject(address: &str, subject_template: &str) -> DistributionChannel {
    DistributionChannel::Email {
        address: address.to_string(),
        subject_template: Some(subject_template.to_string()),
    }
}

/// Helper function to create Slack distribution channel
pub fn slack_channel(webhook_url: &str, channel: &str) -> DistributionChannel {
    DistributionChannel::Slack {
        webhook_url: webhook_url.to_string(),
        channel: channel.to_string(),
        username: None,
        icon_emoji: None,
    }
}

/// Helper function to create customized Slack distribution channel
pub fn slack_channel_custom(
    webhook_url: &str,
    channel: &str,
    username: Option<&str>,
    icon_emoji: Option<&str>,
) -> DistributionChannel {
    DistributionChannel::Slack {
        webhook_url: webhook_url.to_string(),
        channel: channel.to_string(),
        username: username.map(|s| s.to_string()),
        icon_emoji: icon_emoji.map(|s| s.to_string()),
    }
}
