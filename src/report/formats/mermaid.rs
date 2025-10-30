//! Mermaid diagram generation helpers
//!
//! This module contains utilities for working with Mermaid diagrams in reports.

use crate::report::generators::ReportGenerator;

impl ReportGenerator {
    /// Generate a unique diagram ID to prevent conflicts
    pub fn generate_diagram_id(&self, prefix: &str, index: usize) -> String {
        format!("{}-{}-{}", prefix, index, chrono::Utc::now().timestamp())
    }

    /// Wrap Mermaid code with proper HTML container
    pub fn wrap_mermaid_diagram(&self, diagram_code: &str, diagram_id: &str) -> String {
        format!(
            r#"<div class="mermaid" id="{}">{}</div>"#,
            diagram_id, diagram_code
        )
    }
}
