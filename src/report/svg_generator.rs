//! SVG Generation Module for Static Mermaid Diagrams
//!
//! This module provides server-side SVG generation from Mermaid diagrams,
//! eliminating client-side rendering issues and providing reliable diagram output.

use log::{debug, error, warn};
use std::path::{Path, PathBuf};
use tempfile::{TempDir};
use thiserror::Error;
use tokio::process::Command;

/// Errors that can occur during SVG generation
#[derive(Debug, Error)]
pub enum SvgGenerationError {
    #[error("Failed to create temporary directory: {0}")]
    TempDirError(#[from] std::io::Error),
    #[error("Mermaid CLI not found or failed to execute: {0}")]
    MermaidCliError(String),
    #[error("SVG file generation failed: {0}")]
    SvgOutputError(String),
    #[error("Failed to read generated SVG: {0}")]
    SvgReadError(String),
}

/// Configuration for SVG generation
#[derive(Debug, Clone)]
pub struct SvgConfig {
    /// Mermaid theme (default, dark, forest, etc.)
    pub theme: String,
    /// Background color for the diagram
    pub background_color: String,
    /// Output width (optional)
    pub width: Option<u32>,
    /// Output height (optional)
    pub height: Option<u32>,
    /// Timeout for CLI execution in seconds
    pub timeout_seconds: u64,
}

impl Default for SvgConfig {
    fn default() -> Self {
        Self {
            theme: "default".to_string(),
            background_color: "white".to_string(),
            width: None,
            height: None,
            timeout_seconds: 30,
        }
    }
}

/// Generator for converting Mermaid diagrams to static SVG
pub struct SvgGenerator {
    temp_dir: TempDir,
    config: SvgConfig,
}

impl SvgGenerator {
    /// Create a new SVG generator with default configuration
    pub fn new() -> Result<Self, SvgGenerationError> {
        Self::with_config(SvgConfig::default())
    }

    /// Create a new SVG generator with custom configuration
    pub fn with_config(config: SvgConfig) -> Result<Self, SvgGenerationError> {
        let temp_dir = TempDir::new()?;
        debug!("Created temporary directory for SVG generation: {:?}", temp_dir.path());
        
        Ok(Self { temp_dir, config })
    }

    /// Generate SVG from Mermaid code with fallback support
    pub async fn generate_svg_with_fallback(
        &self,
        mermaid_code: &str,
        diagram_id: &str,
    ) -> String {
        match self.generate_svg_from_mermaid(mermaid_code, diagram_id).await {
            Ok(svg) => {
                debug!("Successfully generated SVG for diagram: {}", diagram_id);
                svg
            }
            Err(e) => {
                warn!("SVG generation failed for {}, using fallback: {}", diagram_id, e);
                self.create_fallback_html(mermaid_code, &e.to_string())
            }
        }
    }

    /// Generate SVG from Mermaid code
    pub async fn generate_svg_from_mermaid(
        &self,
        mermaid_code: &str,
        diagram_id: &str,
    ) -> Result<String, SvgGenerationError> {
        debug!("Generating SVG for diagram: {}", diagram_id);

        // Create temp file paths
        let mmd_path = self.temp_dir.path().join(format!("{}.mmd", diagram_id));
        let svg_path = self.temp_dir.path().join(format!("{}.svg", diagram_id));

        // Write Mermaid code to temp file
        tokio::fs::write(&mmd_path, mermaid_code)
            .await
            .map_err(|e| SvgGenerationError::SvgOutputError(format!("Failed to write mermaid file: {}", e)))?;

        // Check if mermaid CLI is available
        self.check_mermaid_cli_available().await?;

        // Build command arguments
        let mut args = vec![
            "-i", mmd_path.to_str().unwrap(),
            "-o", svg_path.to_str().unwrap(),
            "-t", &self.config.theme,
            "--backgroundColor", &self.config.background_color,
        ];

        // Add optional dimensions
        let width_str;
        let height_str;
        if let Some(width) = self.config.width {
            width_str = width.to_string();
            args.extend(&["--width", &width_str]);
        }
        if let Some(height) = self.config.height {
            height_str = height.to_string();
            args.extend(&["--height", &height_str]);
        }

        // Execute mermaid CLI
        debug!("Executing: mmdc {}", args.join(" "));
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(self.config.timeout_seconds),
            Command::new("mmdc").args(&args).output()
        )
        .await
        .map_err(|_| SvgGenerationError::MermaidCliError("Command timed out".to_string()))?
        .map_err(|e| SvgGenerationError::MermaidCliError(format!("Failed to execute mmdc: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(SvgGenerationError::MermaidCliError(format!(
                "mmdc failed with exit code {}: stderr: {}, stdout: {}",
                output.status.code().unwrap_or(-1),
                stderr,
                stdout
            )));
        }

        // Verify SVG file was created
        if !svg_path.exists() {
            return Err(SvgGenerationError::SvgOutputError(
                "SVG file was not created by mmdc".to_string()
            ));
        }

        // Read and return SVG content
        let mut svg_content = tokio::fs::read_to_string(&svg_path)
            .await
            .map_err(|e| SvgGenerationError::SvgReadError(format!("Failed to read SVG file: {}", e)))?;

        // Post-process SVG to improve browser compatibility
        svg_content = self.post_process_svg(svg_content, diagram_id);

        debug!("Successfully generated {} bytes of SVG for {}", svg_content.len(), diagram_id);
        Ok(svg_content)
    }

    /// Check if Mermaid CLI is available
    async fn check_mermaid_cli_available(&self) -> Result<(), SvgGenerationError> {
        let output = Command::new("mmdc")
            .arg("--version")
            .output()
            .await
            .map_err(|e| SvgGenerationError::MermaidCliError(format!(
                "Mermaid CLI (mmdc) not found. Please install with: npm install -g @mermaid-js/mermaid-cli. Error: {}", e
            )))?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            debug!("Found mermaid CLI version: {}", version.trim());
            Ok(())
        } else {
            Err(SvgGenerationError::MermaidCliError(
                "Mermaid CLI found but returned error on --version".to_string()
            ))
        }
    }

    /// Create fallback HTML when SVG generation fails
    fn create_fallback_html(&self, mermaid_code: &str, error_msg: &str) -> String {
        let escaped_code = html_escape::encode_text(mermaid_code);
        format!(
            r#"<div class="mermaid-fallback">
                <div class="fallback-warning">
                    ⚠️ Diagram rendering failed: {}
                </div>
                <details>
                    <summary>View Mermaid Code</summary>
                    <pre class="mermaid-code">{}</pre>
                </details>
            </div>
            <style>
                .mermaid-fallback {{
                    border: 2px dashed #ffa726;
                    border-radius: 8px;
                    padding: 16px;
                    margin: 16px 0;
                    background-color: #fff3e0;
                }}
                .fallback-warning {{
                    color: #ef6c00;
                    font-weight: bold;
                    margin-bottom: 8px;
                }}
                .mermaid-code {{
                    background-color: #f5f5f5;
                    padding: 12px;
                    border-radius: 4px;
                    font-family: 'Courier New', monospace;
                    font-size: 0.9em;
                    overflow-x: auto;
                }}
            </style>"#,
            error_msg, escaped_code
        )
    }

    /// Generate multiple SVGs from a collection of Mermaid diagrams
    pub async fn generate_multiple_svgs(
        &self,
        diagrams: &[(String, String)], // (diagram_id, mermaid_code)
    ) -> Vec<(String, String)> { // (diagram_id, svg_or_fallback)
        let mut results = Vec::new();
        
        for (diagram_id, mermaid_code) in diagrams {
            let svg_result = self.generate_svg_with_fallback(mermaid_code, diagram_id).await;
            results.push((diagram_id.clone(), svg_result));
        }
        
        results
    }

    /// Get the temporary directory path (useful for debugging)
    pub fn temp_dir_path(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Post-process SVG to improve browser compatibility
    fn post_process_svg(&self, mut svg_content: String, diagram_id: &str) -> String {
        debug!("Post-processing SVG for diagram: {}", diagram_id);
        
        // Fix common SVG rendering issues
        
        // 1. Ensure SVG has proper height attribute
        if !svg_content.contains("height=") && svg_content.contains("viewBox=") {
            // Extract viewBox dimensions
            if let Some(viewbox_start) = svg_content.find("viewBox=\"") {
                let viewbox_start = viewbox_start + 9; // Length of "viewBox=\""
                if let Some(viewbox_end) = svg_content[viewbox_start..].find("\"") {
                    let viewbox = &svg_content[viewbox_start..viewbox_start + viewbox_end];
                    let parts: Vec<&str> = viewbox.split_whitespace().collect();
                    if parts.len() == 4 {
                        if let Ok(height) = parts[3].parse::<f32>() {
                            // Add explicit height
                            svg_content = svg_content.replace(
                                "width=\"100%\"",
                                &format!("width=\"100%\" height=\"{}px\"", height)
                            );
                        }
                    }
                }
            }
        }
        
        // 2. Add better styling for text elements
        if svg_content.contains("<style>") {
            let style_addition = r#"
                .classDiagram .nodeLabel, .classDiagram .edgeLabel { visibility: visible !important; }
                .classDiagram text { font-family: Arial, sans-serif !important; }
                .classDiagram foreignObject { overflow: visible !important; }
            "#;
            svg_content = svg_content.replace("</style>", &format!("{}</style>", style_addition));
        }
        
        // 3. Ensure SVG has proper namespace declarations
        if !svg_content.contains("xmlns=\"http://www.w3.org/2000/svg\"") {
            svg_content = svg_content.replace(
                "<svg",
                "<svg xmlns=\"http://www.w3.org/2000/svg\""
            );
        }
        
        debug!("SVG post-processing completed for: {}", diagram_id);
        svg_content
    }
}

/// Utility function to extract diagrams from existing Mermaid code blocks
pub fn extract_diagrams_from_html(html_content: &str) -> Vec<(String, String)> {
    let mut diagrams = Vec::new();
    
    // Simple regex-based extraction (in production, use a proper HTML parser)
    let diagram_pattern = regex::Regex::new(r#"<div class="mermaid"[^>]*id="([^"]+)"[^>]*>(.*?)</div>"#).unwrap();
    
    for captures in diagram_pattern.captures_iter(html_content) {
        if let (Some(id_match), Some(content_match)) = (captures.get(1), captures.get(2)) {
            let diagram_id = id_match.as_str().to_string();
            let mermaid_code = content_match.as_str()
                .trim()
                .replace("&lt;", "<")
                .replace("&gt;", ">")
                .replace("&amp;", "&");
            
            diagrams.push((diagram_id, mermaid_code));
        }
    }
    
    diagrams
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_svg_generator_creation() {
        let generator = SvgGenerator::new();
        assert!(generator.is_ok());
    }

    #[tokio::test]
    async fn test_fallback_html_generation() {
        let generator = SvgGenerator::new().unwrap();
        let mermaid_code = "graph TD\n    A --> B";
        let fallback = generator.create_fallback_html(mermaid_code, "Test error");
        
        assert!(fallback.contains("Diagram rendering failed"));
        assert!(fallback.contains("graph TD"));
        assert!(fallback.contains("mermaid-fallback"));
    }

    #[test]
    fn test_extract_diagrams_from_html() {
        let html = r#"
            <div class="mermaid" id="diagram-1">
                graph TD
                A --> B
            </div>
            <div class="mermaid" id="diagram-2">
                classDiagram
                class Foo
            </div>
        "#;
        
        let diagrams = extract_diagrams_from_html(html);
        assert_eq!(diagrams.len(), 2);
        assert_eq!(diagrams[0].0, "diagram-1");
        assert!(diagrams[0].1.contains("graph TD"));
    }
}