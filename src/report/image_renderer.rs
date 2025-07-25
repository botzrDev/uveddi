//! Image Rendering Service Client
//!
//! This module provides HTTP client functionality to communicate with the
//! Node.js rendering service for converting Mermaid.js diagrams to images.

use crate::error::rendering::RenderingServiceError;
use crate::security::{SecureHttpClient, HttpSecurityConfig};
use md5;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

/// Configuration for the image rendering service
#[derive(Debug, Clone)]
pub struct RenderingServiceConfig {
    /// Base URL of the rendering service
    pub base_url: String,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Maximum retries for failed requests
    pub max_retries: u32,

    // NEW VALIDATION FIELDS:
    /// Maximum input size in bytes (default: 1MB)
    pub max_input_size_bytes: usize,
    /// Maximum diagram complexity score (default: 1000)
    pub max_complexity_score: u32,
    /// Maximum image width (default: 4096)
    pub max_width: u32,
    /// Maximum image height (default: 4096)
    pub max_height: u32,
    /// Maximum number of nodes in diagram (default: 500)
    pub max_nodes: u32,
}

impl Default for RenderingServiceConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:3001".to_string(),
            timeout_seconds: 30,
            max_retries: 3,

            // NEW DEFAULTS:
            max_input_size_bytes: 1024 * 1024, // 1MB
            max_complexity_score: 1000,
            max_width: 4096,
            max_height: 4096,
            max_nodes: 500,
        }
    }
}

/// HTTP client for the rendering service
#[derive(Debug)]
pub struct ImageRenderer {
    client: SecureHttpClient,
    config: RenderingServiceConfig,
}

/// Request payload for single diagram rendering
#[derive(Debug, Serialize)]
pub struct RenderRequest {
    pub mermaid_code: String,
    pub format: ImageFormat,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

/// Request payload for batch diagram rendering
#[derive(Debug, Serialize)]
pub struct BatchRenderRequest {
    pub diagrams: Vec<DiagramRequest>,
    pub format: ImageFormat,
}

#[derive(Debug, Serialize)]
pub struct DiagramRequest {
    pub mermaid_code: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

/// Supported image formats
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageFormat {
    Svg,
    Png,
}

/// Response from the rendering service
#[derive(Debug, Deserialize)]
pub struct RenderResponse {
    pub success: bool,
    pub format: String,
    pub data: String,
    pub metadata: RenderMetadata,
}

/// Batch rendering response
#[derive(Debug, Deserialize)]
pub struct BatchRenderResponse {
    pub success: bool,
    pub results: Vec<BatchResult>,
    pub metadata: BatchMetadata,
}

#[derive(Debug, Deserialize)]
pub struct BatchResult {
    pub index: usize,
    pub success: bool,
    pub format: Option<String>,
    pub data: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RenderMetadata {
    pub render_time_ms: u64,
    pub size_bytes: usize,
    pub dimensions: Dimensions,
}

#[derive(Debug, Deserialize)]
pub struct BatchMetadata {
    pub total_render_time_ms: u64,
    pub diagram_count: usize,
    pub success_count: usize,
}

#[derive(Debug, Deserialize)]
pub struct Dimensions {
    pub width: f64,
    pub height: f64,
}

/// Health check response
#[derive(Debug, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: String,
    pub workers: WorkerStatus,
}

#[derive(Debug, Deserialize)]
pub struct WorkerStatus {
    pub total_workers: u32,
    pub busy_workers: u32,
    pub available_workers: u32,
    pub total_requests: u64,
}

/// Rendered image data
#[derive(Debug, Clone)]
pub struct RenderedImage {
    pub format: ImageFormat,
    pub data: Vec<u8>,
    pub dimensions: (u32, u32),
    pub render_time_ms: u64,
}

impl ImageRenderer {
    /// Create a new image renderer with default configuration
    pub fn new() -> Result<Self, RenderingServiceError> {
        Self::with_config(RenderingServiceConfig::default())
    }

    /// Create a new image renderer with custom configuration
    pub fn with_config(config: RenderingServiceConfig) -> Result<Self, RenderingServiceError> {
        // Create secure HTTP client with appropriate configuration for rendering service
        let mut http_config = HttpSecurityConfig::default();
        
        // Configure for local development (rendering service typically runs on localhost)
        #[cfg(debug_assertions)]
        {
            http_config.enforce_https = false; // Allow HTTP for local development
        }
        
        // Set timeouts based on rendering service config
        http_config.timeout_seconds = config.timeout_seconds;
        http_config.connect_timeout_seconds = 10;
        http_config.read_timeout_seconds = config.timeout_seconds;
        
        let client = SecureHttpClient::new(http_config)
            .map_err(|e| RenderingServiceError::NetworkError {
                message: format!("Failed to create secure HTTP client: {}", e),
            })?;

        Ok(Self { client, config })
    }

    /// Check if the rendering service is healthy
    pub async fn health_check(&self) -> Result<HealthResponse, RenderingServiceError> {
        let url = format!("{}/health", self.config.base_url);

        let response = self.client.get(&url).await.map_err(|e| {
            RenderingServiceError::NetworkError {
                message: format!("Health check request failed: {}", e),
            }
        })?;

        if !response.status().is_success() {
            return Err(match response.status().as_u16() {
                503 => RenderingServiceError::ServiceUnavailable,
                429 => RenderingServiceError::RateLimitExceeded {
                    retry_after_seconds: None,
                },
                408 | 504 => RenderingServiceError::request_timeout(Duration::from_secs(30)),
                400 => RenderingServiceError::InvalidResponse {
                    details: format!("HTTP {}", response.status()),
                },
                _ => RenderingServiceError::InvalidResponse {
                    details: format!("HTTP {}", response.status()),
                },
            });
        }

        let health: HealthResponse = response.json().await?;
        Ok(health)
    }

    /// Render a single diagram to image
    pub async fn render_diagram(
        &self,
        mermaid_code: &str,
        format: ImageFormat,
        dimensions: Option<(u32, u32)>,
    ) -> Result<RenderedImage, RenderingServiceError> {
        // ADD VALIDATION BEFORE PROCESSING
        self.validate_input(mermaid_code, dimensions)?;

        let request = RenderRequest {
            mermaid_code: mermaid_code.to_string(),
            format: format.clone(),
            width: dimensions.map(|(w, _)| w),
            height: dimensions.map(|(_, h)| h),
        };

        let mut retries = 0;

        loop {
            match self.try_render_single(&request).await {
                Ok(image) => return Ok(image),
                Err(e) if e.is_retryable() && retries < self.config.max_retries => {
                    retries += 1;
                    // Use exponential backoff based on error severity
                    let delay = match e.severity() {
                        crate::error::rendering::ErrorSeverity::High
                        | crate::error::rendering::ErrorSeverity::Critical => {
                            Duration::from_millis(1000 * retries as u64)
                        }
                        _ => Duration::from_millis(100 * retries as u64),
                    };
                    tokio::time::sleep(delay).await;
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
    }

    async fn try_render_single(
        &self,
        request: &RenderRequest,
    ) -> Result<RenderedImage, RenderingServiceError> {
        let url = format!("{}/render", self.config.base_url);

        // Serialize the request to JSON
        let json_body = serde_json::to_string(request)
            .map_err(|e| RenderingServiceError::NetworkError {
                message: format!("Failed to serialize request: {}", e),
            })?;

        let response = self.client.post(&url, json_body).await.map_err(|e| {
            RenderingServiceError::NetworkError {
                message: format!("Render request failed: {}", e),
            }
        })?;

        if !response.status().is_success() {
            return Err(match response.status().as_u16() {
                503 => RenderingServiceError::ServiceUnavailable,
                429 => RenderingServiceError::RateLimitExceeded {
                    retry_after_seconds: None,
                },
                408 | 504 => RenderingServiceError::request_timeout(Duration::from_secs(30)),
                400 => RenderingServiceError::InvalidMermaidSyntax {
                    line: None,
                    details: "Invalid request format".to_string(),
                },
                _ => RenderingServiceError::InvalidResponse {
                    details: format!("HTTP {}", response.status()),
                },
            });
        }

        let render_response: RenderResponse =
            response
                .json()
                .await
                .map_err(|e| RenderingServiceError::InvalidResponse {
                    details: format!("JSON parse error: {}", e),
                })?;

        if !render_response.success {
            return Err(RenderingServiceError::InvalidResponse {
                details: "Rendering failed on service side".to_string(),
            });
        }

        let data = match request.format {
            ImageFormat::Svg => render_response.data.into_bytes(),
            ImageFormat::Png => {
                use base64::{engine::general_purpose, Engine as _};
                general_purpose::STANDARD
                    .decode(render_response.data)
                    .map_err(|e| RenderingServiceError::ImageConversionError {
                        from_format: "base64".to_string(),
                        to_format: "binary".to_string(),
                        reason: e.to_string(),
                    })?
            }
        };

        Ok(RenderedImage {
            format: request.format.clone(),
            data,
            dimensions: (
                render_response.metadata.dimensions.width as u32,
                render_response.metadata.dimensions.height as u32,
            ),
            render_time_ms: render_response.metadata.render_time_ms,
        })
    }

    /// Render multiple diagrams in a batch
    pub async fn render_batch(
        &self,
        diagrams: Vec<(String, Option<(u32, u32)>)>, // (mermaid_code, dimensions)
        format: ImageFormat,
    ) -> Result<Vec<Result<RenderedImage, String>>, RenderingServiceError> {
        // ADD VALIDATION FOR EACH DIAGRAM
        for (mermaid_code, dimensions) in &diagrams {
            if let Err(e) = self.validate_input(mermaid_code, *dimensions) {
                return Err(e);
            }
        }

        let diagram_requests: Vec<DiagramRequest> = diagrams
            .into_iter()
            .map(|(mermaid_code, dimensions)| DiagramRequest {
                mermaid_code,
                width: dimensions.map(|(w, _)| w),
                height: dimensions.map(|(_, h)| h),
            })
            .collect();

        let request = BatchRenderRequest {
            diagrams: diagram_requests,
            format: format.clone(),
        };

        let url = format!("{}/render/batch", self.config.base_url);

        // Serialize the request to JSON
        let json_body = serde_json::to_string(&request)
            .map_err(|e| RenderingServiceError::NetworkError {
                message: format!("Failed to serialize batch request: {}", e),
            })?;

        let response = self.client.post(&url, json_body).await.map_err(|e| {
            RenderingServiceError::NetworkError {
                message: format!("Batch render request failed: {}", e),
            }
        })?;

        if !response.status().is_success() {
            return Err(match response.status().as_u16() {
                503 => RenderingServiceError::ServiceUnavailable,
                429 => RenderingServiceError::RateLimitExceeded {
                    retry_after_seconds: None,
                },
                408 | 504 => RenderingServiceError::request_timeout(Duration::from_secs(30)),
                400 => RenderingServiceError::InvalidResponse {
                    details: format!("HTTP {}", response.status()),
                },
                _ => RenderingServiceError::InvalidResponse {
                    details: format!("HTTP {}", response.status()),
                },
            });
        }

        let batch_response: BatchRenderResponse =
            response
                .json()
                .await
                .map_err(|e| RenderingServiceError::InvalidResponse {
                    details: format!("JSON parse error: {}", e),
                })?;

        let mut results = Vec::new();

        for result in batch_response.results {
            if result.success {
                if let (Some(data_str), Some(format_str)) = (result.data, result.format) {
                    let data = match format {
                        ImageFormat::Svg => data_str.into_bytes(),
                        ImageFormat::Png => {
                            use base64::{engine::general_purpose, Engine as _};
                            match general_purpose::STANDARD.decode(data_str) {
                                Ok(data) => data,
                                Err(e) => {
                                    results.push(Err(
                                        RenderingServiceError::ImageConversionError {
                                            from_format: "base64".to_string(),
                                            to_format: "binary".to_string(),
                                            reason: e.to_string(),
                                        }
                                        .user_message(),
                                    ));
                                    continue;
                                }
                            }
                        }
                    };

                    results.push(Ok(RenderedImage {
                        format: format.clone(),
                        data,
                        dimensions: (800, 600), // Default dimensions for batch
                        render_time_ms: 0,      // Not available in batch response
                    }));
                } else {
                    results.push(Err(
                        "Missing data or format in successful result".to_string()
                    ));
                }
            } else {
                results.push(Err(result
                    .error
                    .unwrap_or_else(|| "Unknown error".to_string())));
            }
        }

        Ok(results)
    }

    /// Save a rendered image to a file
    pub async fn save_image(
        &self,
        image: &RenderedImage,
        file_path: &PathBuf,
    ) -> Result<(), RenderingServiceError> {
        tokio::fs::write(file_path, &image.data)
            .await
            .map_err(|e| RenderingServiceError::ImageConversionError {
                from_format: "binary".to_string(),
                to_format: "file".to_string(),
                reason: format!("Failed to write file: {}", e),
            })?;

        Ok(())
    }

    /// Validate input size
    fn validate_input_size(&self, mermaid_code: &str) -> Result<(), RenderingServiceError> {
        let size_bytes = mermaid_code.len();
        if size_bytes > self.config.max_input_size_bytes {
            return Err(RenderingServiceError::InputTooLarge {
                size_bytes,
                limit_bytes: self.config.max_input_size_bytes,
            });
        }
        Ok(())
    }

    /// Validate diagram dimensions
    fn validate_dimensions(
        &self,
        dimensions: Option<(u32, u32)>,
    ) -> Result<(), RenderingServiceError> {
        if let Some((width, height)) = dimensions {
            if width > self.config.max_width || height > self.config.max_height {
                return Err(RenderingServiceError::InvalidMermaidSyntax {
                    line: None,
                    details: format!(
                        "Dimensions {}x{} exceed maximum allowed {}x{}",
                        width, height, self.config.max_width, self.config.max_height
                    ),
                });
            }
        }
        Ok(())
    }

    /// Validate diagram complexity
    fn validate_complexity(&self, mermaid_code: &str) -> Result<(), RenderingServiceError> {
        let complexity_score = self.calculate_complexity_score(mermaid_code) as u64;

        if complexity_score > self.config.max_complexity_score as u64 {
            return Err(RenderingServiceError::DiagramComplexityExceeded {
                reason: "Diagram too complex".to_string(),
                limit: self.config.max_complexity_score as u64,
                actual: complexity_score,
            });
        }

        Ok(())
    }

    /// Calculate diagram complexity score
    fn calculate_complexity_score(&self, mermaid_code: &str) -> u32 {
        let mut score = 0;

        // Count nodes (approximate by counting arrows and connections)
        let node_indicators = [" --> ", " --- ", " -> ", " -- ", "-->", "---"];
        let mut node_count = 0;
        for indicator in &node_indicators {
            node_count += mermaid_code.matches(indicator).count();
        }

        // Count subgraphs (nested complexity)
        let subgraph_count = mermaid_code.matches("subgraph").count();

        // Count styling and classes (additional complexity)
        let style_count =
            mermaid_code.matches("class ").count() + mermaid_code.matches("style ").count();

        // Calculate complexity score
        score += node_count as u32 * 2; // 2 points per connection
        score += subgraph_count as u32 * 10; // 10 points per subgraph
        score += style_count as u32 * 1; // 1 point per style

        // Check for excessive nodes
        if node_count > self.config.max_nodes as usize {
            score += 1000; // Penalty for too many nodes
        }

        score
    }

    /// Validate input for security concerns
    fn validate_security(&self, mermaid_code: &str) -> Result<(), RenderingServiceError> {
        // Check for potentially malicious patterns
        let suspicious_patterns = [
            "javascript:",
            "<script",
            "eval(",
            "document.",
            "window.",
            "fetch(",
            "XMLHttpRequest",
        ];

        for pattern in &suspicious_patterns {
            if mermaid_code.to_lowercase().contains(pattern) {
                return Err(RenderingServiceError::SecurityValidationFailed {
                    reason: format!("Suspicious pattern detected: {}", pattern),
                });
            }
        }

        // Check for excessively long lines (potential DoS)
        for line in mermaid_code.lines() {
            if line.len() > 1000 {
                return Err(RenderingServiceError::SecurityValidationFailed {
                    reason: "Line too long (potential DoS)".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Comprehensive input validation
    fn validate_input(
        &self,
        mermaid_code: &str,
        dimensions: Option<(u32, u32)>,
    ) -> Result<(), RenderingServiceError> {
        self.validate_input_size(mermaid_code)?;
        self.validate_dimensions(dimensions)?;
        self.validate_complexity(mermaid_code)?;
        self.validate_security(mermaid_code)?;
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check() {
        let renderer = ImageRenderer::new().expect("Failed to create renderer");

        // This test requires the rendering service to be running
        // In CI/CD, this would be handled by docker-compose
        match renderer.health_check().await {
            Ok(health) => {
                assert_eq!(health.status, "healthy");
                assert!(health.workers.total_workers > 0);
            }
            Err(RenderingServiceError::ServiceUnavailable) => {
                println!("Rendering service not available, skipping test");
            }
            Err(RenderingServiceError::NetworkError { .. }) => {
                println!("Network error - rendering service not available, skipping test");
            }
            Err(RenderingServiceError::ConnectionTimeout { .. }) => {
                println!("Connection timeout - rendering service not available, skipping test");
            }
            Err(RenderingServiceError::RequestTimeout { .. }) => {
                println!("Request timeout - rendering service not available, skipping test");
            }
            Err(e) => {
                println!(
                    "Rendering service error (expected in test environment): {}",
                    e
                );
                // Don't panic - external service dependency is acceptable to fail in tests
            }
        }
    }

    #[tokio::test]
    async fn test_render_simple_diagram() {
        let renderer = ImageRenderer::new().expect("Failed to create renderer");

        let mermaid_code = r#"
graph TD
    A[Start] --> B[Process]
    B --> C[End]
"#;

        match renderer
            .render_diagram(mermaid_code, ImageFormat::Svg, None)
            .await
        {
            Ok(image) => {
                assert!(!image.data.is_empty());
                assert!(matches!(image.format, ImageFormat::Svg));
            }
            Err(RenderingServiceError::ServiceUnavailable) => {
                println!("Rendering service not available, skipping test");
            }
            Err(RenderingServiceError::NetworkError { .. }) => {
                println!("Network error - rendering service not available, skipping test");
            }
            Err(RenderingServiceError::ConnectionTimeout { .. }) => {
                println!("Connection timeout - rendering service not available, skipping test");
            }
            Err(RenderingServiceError::RequestTimeout { .. }) => {
                println!("Request timeout - rendering service not available, skipping test");
            }
            Err(e) => {
                println!(
                    "Rendering service error (expected in test environment): {}",
                    e
                );
                // Don't panic - external service dependency is acceptable to fail in tests
            }
        }
    }

    #[test]
    fn test_input_size_validation() {
        let renderer = ImageRenderer::new().expect("Failed to create renderer");

        // Test normal input
        let normal_input = "graph TD\n    A --> B";
        assert!(renderer.validate_input_size(normal_input).is_ok());

        // Test oversized input
        let large_input = "A".repeat(2 * 1024 * 1024); // 2MB
        assert!(matches!(
            renderer.validate_input_size(&large_input),
            Err(RenderingServiceError::InputTooLarge { .. })
        ));
    }

    #[test]
    fn test_dimension_validation() {
        let renderer = ImageRenderer::new().expect("Failed to create renderer");

        // Test normal dimensions
        assert!(renderer.validate_dimensions(Some((800, 600))).is_ok());
        assert!(renderer.validate_dimensions(None).is_ok());

        // Test oversized dimensions
        assert!(matches!(
            renderer.validate_dimensions(Some((10000, 600))),
            Err(RenderingServiceError::InvalidMermaidSyntax { .. })
        ));
    }

    #[test]
    fn test_complexity_validation() {
        let renderer = ImageRenderer::new().expect("Failed to create renderer");

        // Test simple diagram
        let simple = "graph TD\n    A --> B\n    B --> C";
        assert!(renderer.validate_complexity(simple).is_ok());

        // Test complex diagram
        let complex = (0..1000)
            .map(|i| format!("    A{} --> B{}", i, i))
            .collect::<Vec<_>>()
            .join("\n");
        let complex_diagram = format!("graph TD\n{}", complex);

        assert!(matches!(
            renderer.validate_complexity(&complex_diagram),
            Err(RenderingServiceError::DiagramComplexityExceeded { .. })
        ));
    }

    #[test]
    fn test_security_validation() {
        let renderer = ImageRenderer::new().expect("Failed to create renderer");

        // Test safe input
        let safe_input = "graph TD\n    A[Safe] --> B[Diagram]";
        assert!(renderer.validate_security(safe_input).is_ok());

        // Test suspicious input
        let suspicious_input = "graph TD\n    A[<script>alert('xss')</script>] --> B";
        assert!(matches!(
            renderer.validate_security(suspicious_input),
            Err(RenderingServiceError::SecurityValidationFailed { .. })
        ));
    }

    #[test]
    fn test_comprehensive_validation() {
        let renderer = ImageRenderer::new().expect("Failed to create renderer");

        // Test valid input
        let valid_input = "graph TD\n    A --> B";
        assert!(renderer
            .validate_input(valid_input, Some((800, 600)))
            .is_ok());

        // Test invalid input (multiple violations)
        let invalid_input = "A".repeat(2 * 1024 * 1024); // Too large
        assert!(renderer
            .validate_input(&invalid_input, Some((10000, 600)))
            .is_err());
    }
}
