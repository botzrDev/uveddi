use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub database_url: String,
    pub websocket_port: u16,
    pub github_webhook_secret: String,
    // ...other fields...
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            database_url: "postgresql://localhost:5432/uveddi_monitoring".to_string(),
            websocket_port: 8081,
            github_webhook_secret: std::env::var("GITHUB_WEBHOOK_SECRET").unwrap_or_default(),
            // ...other defaults...
        }
    }
}
