//! Report Generation Module for Uveddi
//!
//! This module provides comprehensive report generation capabilities for analysis results.
//! It supports multiple output formats and includes advanced features like AI explanations,
//! code snippets, and visual diagrams.
//!
//! # Architecture
//!
//! The module is split into focused sub-modules for maintainability:
//! - `formats/`: Format-specific generators (HTML, Mermaid)
//! - `generators/`: Core orchestration and template helpers
//! - `templates/`: Static HTML/CSS/JS content
//! - `ai_integration/`: AI-powered features
//! - `html_generator`: HTML report generation with CSS/JS
//! - `executive_summary`: Business logic for summaries and metrics
//! - `mermaid_integration`: Mermaid.js diagram handling
//! - `markdown_generator`: Markdown report generation
//! - `interactive_generator`: Interactive web reports
//! - `modern_generator`: Template-based report generation
//!
//! # Supported Formats
//!
//! ## Markdown Reports
//! Rich, human-readable reports with:
//! - Executive summary with key metrics
//! - Issues organized by severity level
//! - Detailed issue descriptions with context
//! - Optional code snippets and AI explanations
//! - Mermaid.js diagrams for architectural visualization
//!
//! ## JSON Reports
//! Structured data format for:
//! - Integration with external tools
//! - Automated processing and analysis
//! - API consumption
//! - Custom toolchain integration
//!
//! # Key Features
//!
//! - **Multi-format Output**: Markdown for humans, JSON for machines
//! - **AI Integration**: Optional AI-generated explanations and recommendations
//! - **Visual Diagrams**: Mermaid.js integration for architectural visualization
//! - **Customizable Content**: Configure inclusion of code snippets, AI explanations, diagrams
//! - **Performance Optimized**: Efficient processing for large analysis results
//! - **ERD Compliant**: Follows Entity Relationship Diagram specifications (ER-F-011 to ER-F-014)

use crate::analysis::mermaid_generator::{MermaidGenerationError, MermaidGenerator};
use crate::database::models::{AnalysisRun, AntiPatternType, ArchitecturalIssue};
use crate::models::visualization::{ArchitecturalComponent, DiagramMetadata, DiagramType};
#[cfg(feature = "interactive-reports")]
use crate::report::svg_generator::SvgGenerator;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::Duration;
use tracing::{error, info, warn};

// Module declarations for new structure
pub mod ai_integration;
pub mod formats;
pub mod generators;
pub mod templates;

// Legacy modules (to be gradually migrated)
pub mod errors;

#[cfg(feature = "image-rendering")]
pub mod image_renderer;

#[cfg(feature = "interactive-reports")]
pub mod data_transformer;
pub mod diagrams;
#[cfg(feature = "interactive-reports")]
pub mod interactive_generator;
#[cfg(feature = "interactive-reports")]
pub mod interactive_models;
pub mod markdown_generator;
pub mod metrics;
pub mod modern_generator;
pub mod security;
#[cfg(feature = "interactive-reports")]
pub mod svg_generator;

// Keep these for now to avoid breaking changes
pub mod executive_summary;
pub mod html_generator;
pub mod mermaid_integration;

// Re-export core types
pub use ai_integration::*;
pub use formats::*;
pub use generators::ReportGenerator;
pub use templates::*;

/// Error types for report generation
pub use errors::*;

#[cfg(feature = "image-rendering")]
pub use image_renderer::{ImageFormat, ImageRenderer, RenderedImage};

// Export interactive report models
#[cfg(feature = "interactive-reports")]
pub use interactive_models::{
    AiInsights, AnalysisSummary, DependencyGraph, DiagramDefinition, Finding, GraphEdge, GraphNode,
    InteractiveReport, ProjectMetadata, ReportMetadata, REPORT_SCHEMA_VERSION,
};

// Stub types when interactive-reports is disabled
#[cfg(not(feature = "interactive-reports"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramDefinition {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub source: String,
    pub description: Option<String>,
    pub components: Vec<String>,
    pub metadata: DiagramRenderMetadata,
}

#[cfg(not(feature = "interactive-reports"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramRenderMetadata {
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[cfg(not(feature = "interactive-reports"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub uveddi_version: String,
    pub configuration: std::collections::HashMap<String, String>,
    pub performance: Option<serde_json::Value>,
}

// Export interactive report generator
#[cfg(feature = "interactive-reports")]
pub use interactive_generator::{InteractiveReportConfig, InteractiveReportGenerator};

// Export security utilities
pub use security::*;

// Error handling utility for Tera errors
fn log_tera_error_chain(e: &crate::report::modern_generator::ModernReportError) {
    error!("Tera template engine error: {}", e);
    // Simplified error logging without source chain
}

/// Diagram generation mode for hybrid rendering approach
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DiagramMode {
    /// Try image rendering, fallback to Mermaid-only if service unavailable (recommended)
    ImageWithFallback,
    /// Only generate Mermaid code (no image rendering attempted) - zero hosting costs
    MermaidOnly,
    /// Only generate images (fail if service unavailable) - requires hosting
    ImageOnly,
}

impl Default for DiagramMode {
    fn default() -> Self {
        DiagramMode::MermaidOnly // Default to zero-cost option
    }
}

impl DiagramMode {
    /// Returns true if this mode should attempt image rendering
    pub fn should_attempt_image_rendering(&self) -> bool {
        matches!(
            self,
            DiagramMode::ImageOnly | DiagramMode::ImageWithFallback
        )
    }

    /// Returns true if this mode allows fallback to Mermaid-only
    pub fn allows_fallback(&self) -> bool {
        matches!(
            self,
            DiagramMode::ImageWithFallback | DiagramMode::MermaidOnly
        )
    }
}

pub use crate::error::rendering::RenderingServiceError;

/// Enhanced report data structure for future extensibility
#[cfg(feature = "interactive-reports")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedReportData {
    pub analysis_run: AnalysisRun,
    pub issues: Vec<ArchitecturalIssue>,
    pub anti_pattern_types: HashMap<i64, AntiPatternType>,
    pub metadata: ReportMetadata,
}

#[cfg(feature = "interactive-reports")]
impl EnhancedReportData {
    pub fn new(
        analysis_run: AnalysisRun,
        issues: Vec<ArchitecturalIssue>,
        anti_pattern_types: HashMap<i64, AntiPatternType>,
    ) -> Self {
        Self {
            analysis_run,
            issues,
            anti_pattern_types,
            metadata: ReportMetadata {
                generated_at: chrono::Utc::now(),
                uveddi_version: env!("CARGO_PKG_VERSION").to_string(),
                configuration: std::collections::HashMap::new(),
                performance: None,
            },
        }
    }
}
