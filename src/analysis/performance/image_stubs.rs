//! Stub implementations for image rendering types when the image-rendering feature is not enabled

use crate::error::rendering::RenderingServiceError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageFormat {
    Svg,
    Png,
}

#[derive(Debug, Clone)]
pub struct ImageRenderer;

#[derive(Debug, Clone)]
pub struct RenderingServiceConfig {
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub max_complexity_score: u32,
    pub max_width: u32,
    pub max_height: u32,
}

#[derive(Debug, Clone)]
pub struct RenderResult {
    pub data: Vec<u8>,
    pub dimensions: (u32, u32),
}

impl Default for RenderingServiceConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
            max_retries: 1,
            max_complexity_score: 1000,
            max_width: 1920,
            max_height: 1080,
        }
    }
}

impl ImageRenderer {
    pub fn with_config(_config: RenderingServiceConfig) -> Result<Self, RenderingServiceError> {
        Ok(Self)
    }

    pub fn new() -> Result<Self, RenderingServiceError> {
        Ok(Self)
    }

    pub async fn render_diagram(
        &self,
        _diagram: &str,
        _format: ImageFormat,
        options: Option<(u32, u32)>,
    ) -> Result<RenderResult, Box<dyn std::error::Error + Send + Sync>> {
        let dimensions = options.unwrap_or((1200, 800));
        Ok(RenderResult {
            data: vec![],
            dimensions,
        })
    }
}
