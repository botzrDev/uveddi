//! Mermaid Template Engine for Diagram Generation
//!
//! This module provides template-based generation of Mermaid.js diagrams
//! from architectural components. It supports multiple diagram types and
//! severity-based styling for comprehensive visualization.

use crate::models::visualization::{
    ArchitecturalComponent, DiagramMetadata, DiagramSpec, DiagramType, StyleConfig, ValidationMetrics,
};
use serde_json::Value;
use std::collections::HashMap;
use tera::{Context, Tera};
use uuid::Uuid;

/// Mermaid.js diagram generator using template-based approach
pub struct MermaidGenerator {
    /// Template engine for diagram generation
    template_engine: Tera,
    /// Predefined diagram specifications
    diagram_specs: HashMap<DiagramType, DiagramSpec>,
}

impl MermaidGenerator {
    /// Create a new Mermaid generator with default templates
    pub fn new() -> Result<Self, MermaidGenerationError> {
        let mut tera = Tera::new("templates/diagrams/*").unwrap_or_else(|_| Tera::new("").expect("Failed to create empty Tera instance"));
        
        // Register built-in templates
        Self::register_builtin_templates(&mut tera)?;
        
        let diagram_specs = Self::create_default_specs();
        
        Ok(Self {
            template_engine: tera,
            diagram_specs,
        })
    }

    /// Generate a Mermaid.js diagram from architectural components
    ///
    /// # Arguments
    ///
    /// * `components` - The architectural components to visualize
    /// * `diagram_type` - Type of diagram to generate
    /// * `severity_data` - Optional severity information for styling
    ///
    /// # Returns
    ///
    /// * `DiagramMetadata` - Generated diagram with metadata
    pub fn generate_diagram(
        &self,
        components: &[ArchitecturalComponent],
        diagram_type: DiagramType,
        severity_data: Option<&HashMap<Uuid, String>>,
    ) -> Result<DiagramMetadata, MermaidGenerationError> {
        let spec = self
            .diagram_specs
            .get(&diagram_type)
            .ok_or_else(|| MermaidGenerationError::UnsupportedDiagramType(diagram_type.clone()))?;

        let template_name = self.get_template_name(&diagram_type);
        let context = self.build_template_context(components, spec, severity_data)?;
        
        let mermaid_src = self
            .template_engine
            .render(&template_name, &context)
            .map_err(|e| MermaidGenerationError::TemplateRenderError(e.to_string()))?;

        let metadata = DiagramMetadata {
            diagram_type,
            mermaid_src: Self::clean_generated_diagram(&mermaid_src),
            image_path: None,
            generated_at: chrono::Utc::now(),
            components: components.iter().map(|c| c.component_id).collect(),
            validation_metrics: None,
        };

        Ok(metadata)
    }

    /// Generate a component diagram for anti-pattern visualization
    pub fn generate_component_diagram_for_antipattern(
        &self,
        components: &[ArchitecturalComponent],
        anti_pattern_type_id: i64,
        severity_data: Option<&HashMap<Uuid, String>>,
    ) -> Result<DiagramMetadata, MermaidGenerationError> {
        // Filter components relevant to the anti-pattern
        let filtered_components = self.filter_components_for_antipattern(components, anti_pattern_type_id);
        
        self.generate_diagram(&filtered_components, DiagramType::Component, severity_data)
    }

    /// Generate a dependency graph for cyclic dependency detection
    pub fn generate_dependency_graph(
        &self,
        components: &[ArchitecturalComponent],
        highlighted_components: &[Uuid],
    ) -> Result<DiagramMetadata, MermaidGenerationError> {
        let mut severity_data = HashMap::new();
        
        // Mark highlighted components as critical
        for component_id in highlighted_components {
            severity_data.insert(*component_id, "critical".to_string());
        }
        
        self.generate_diagram(components, DiagramType::Dependency, Some(&severity_data))
    }

    /// Generate a sequence diagram for interaction flows
    pub fn generate_sequence_diagram(
        &self,
        components: &[ArchitecturalComponent],
        interaction_flow: &[Uuid],
    ) -> Result<DiagramMetadata, MermaidGenerationError> {
        // Filter components to those in the interaction flow
        let flow_components: Vec<_> = components
            .iter()
            .filter(|c| interaction_flow.contains(&c.component_id))
            .cloned()
            .collect();
        
        self.generate_diagram(&flow_components, DiagramType::Sequence, None)
    }

    /// Build template context from components and specifications
    fn build_template_context(
        &self,
        components: &[ArchitecturalComponent],
        spec: &DiagramSpec,
        severity_data: Option<&HashMap<Uuid, String>>,
    ) -> Result<Context, MermaidGenerationError> {
        let mut context = Context::new();
        
        // Add basic diagram properties
        context.insert("layout", spec.layout.to_mermaid_syntax());
        context.insert("diagram_type", &spec.diagram_type);
        
        // Transform components into template-friendly format
        let template_components: Vec<Value> = components
            .iter()
            .map(|c| self.component_to_template_value(c, severity_data))
            .collect();
        
        context.insert("components", &template_components);
        
        // Add dependencies
        let dependencies = self.extract_dependencies_for_template(components);
        context.insert("dependencies", &dependencies);
        
        // Add styling information
        let styles = self.build_style_context(spec, severity_data);
        context.insert("styles", &styles);
        
        Ok(context)
    }

    /// Convert an architectural component to template value
    fn component_to_template_value(
        &self,
        component: &ArchitecturalComponent,
        severity_data: Option<&HashMap<Uuid, String>>,
    ) -> Value {
        let mut value = serde_json::json!({
            "id": component.component_id,
            "name": component.name,
            "type": component.component_type,
            "group": component.group,
            "icon": component.component_type.icon(),
            "metrics": component.metrics
        });

        // Add severity styling if available
        if let Some(severity_map) = severity_data {
            if let Some(severity) = severity_map.get(&component.component_id) {
                value["severity"] = Value::String(severity.clone());
                value["css_class"] = Value::String(format!("severity-{}", severity));
            }
        }

        value
    }

    /// Extract dependencies in template-friendly format
    fn extract_dependencies_for_template(
        &self,
        components: &[ArchitecturalComponent],
    ) -> Vec<Value> {
        let mut dependencies = Vec::new();
        let component_map: HashMap<Uuid, &ArchitecturalComponent> = components
            .iter()
            .map(|c| (c.component_id, c))
            .collect();

        for component in components {
            for dep in &component.dependencies {
                if let Some(target_component) = component_map.get(&dep.target_component_id) {
                    dependencies.push(serde_json::json!({
                        "from_id": component.component_id,
                        "from_name": component.name,
                        "to_id": target_component.component_id,
                        "to_name": target_component.name,
                        "type": dep.dependency_type,
                        "properties": dep.properties
                    }));
                }
            }
        }

        dependencies
    }

    /// Build style context for template rendering
    fn build_style_context(
        &self,
        spec: &DiagramSpec,
        severity_data: Option<&HashMap<Uuid, String>>,
    ) -> HashMap<String, StyleConfig> {
        let mut styles = spec.severity_styles.clone();
        
        // Add default styles if not present
        if !styles.contains_key("default") {
            styles.insert("default".to_string(), StyleConfig::default());
        }
        
        // Ensure all severity levels have styles
        if let Some(severity_map) = severity_data {
            for severity in severity_map.values() {
                if !styles.contains_key(severity) {
                    styles.insert(severity.clone(), Self::get_default_severity_style(severity));
                }
            }
        }
        
        styles
    }

    /// Get default style for a severity level
    fn get_default_severity_style(severity: &str) -> StyleConfig {
        match severity {
            "critical" => StyleConfig {
                fill_color: "#ff6b6b".to_string(),
                stroke_color: "#d63031".to_string(),
                stroke_width: 3,
                text_color: Some("#ffffff".to_string()),
                css_classes: vec!["critical".to_string()],
            },
            "high" => StyleConfig {
                fill_color: "#feca57".to_string(),
                stroke_color: "#f39c12".to_string(),
                stroke_width: 2,
                text_color: Some("#2d3436".to_string()),
                css_classes: vec!["high".to_string()],
            },
            "medium" => StyleConfig {
                fill_color: "#48dbfb".to_string(),
                stroke_color: "#0abde3".to_string(),
                stroke_width: 2,
                text_color: Some("#2d3436".to_string()),
                css_classes: vec!["medium".to_string()],
            },
            "low" => StyleConfig {
                fill_color: "#1dd1a1".to_string(),
                stroke_color: "#10ac84".to_string(),
                stroke_width: 1,
                text_color: Some("#2d3436".to_string()),
                css_classes: vec!["low".to_string()],
            },
            _ => StyleConfig::default(),
        }
    }

    /// Filter components relevant to a specific anti-pattern
    fn filter_components_for_antipattern(
        &self,
        components: &[ArchitecturalComponent],
        _anti_pattern_type_id: i64,
    ) -> Vec<ArchitecturalComponent> {
        // For now, return all components. In a full implementation,
        // this would filter based on anti-pattern relevance
        components.to_vec()
    }

    /// Get template name for diagram type
    fn get_template_name(&self, diagram_type: &DiagramType) -> String {
        match diagram_type {
            DiagramType::Component => "component_diagram.tera".to_string(),
            DiagramType::Sequence => "sequence_diagram.tera".to_string(),
            DiagramType::Class => "class_diagram.tera".to_string(),
            DiagramType::Dependency => "dependency_graph.tera".to_string(),
            DiagramType::DataFlow => "dataflow_diagram.tera".to_string(),
            DiagramType::Deployment => "deployment_diagram.tera".to_string(),
            DiagramType::Security => "security_diagram.tera".to_string(),
        }
    }

    /// Register built-in templates
    fn register_builtin_templates(tera: &mut Tera) -> Result<(), MermaidGenerationError> {
        // Component diagram template
        tera.add_raw_template(
            "component_diagram.tera",
            r#"graph {{ layout }}
{% for component in components -%}
    {{ component.id }}["{{ component.icon }} {{ component.name }}"]
{% if component.css_class -%}
    class {{ component.id }} {{ component.css_class }}
{% endif -%}
{% endfor %}

{% for dependency in dependencies -%}
    {{ dependency.from_id }} --> {{ dependency.to_id }}
{% endfor %}

{%- for style_name, style in styles %}
    classDef {{ style_name }} fill:{{ style.fill_color }},stroke:{{ style.stroke_color }},stroke-width:{{ style.stroke_width }}px
{%- if style.text_color %},color:{{ style.text_color }}{% endif -%}
{% endfor %}"#,
        )
        .map_err(|e| MermaidGenerationError::TemplateLoadError(e.to_string()))?;

        // Dependency graph template
        tera.add_raw_template(
            "dependency_graph.tera",
            r#"graph {{ layout }}
{% for component in components -%}
    {{ component.id }}[{{ component.name }}]
{% if component.css_class -%}
    class {{ component.id }} {{ component.css_class }}
{% endif -%}
{% endfor %}

{% for dependency in dependencies -%}
    {{ dependency.from_id }} -->|{{ dependency.type }}| {{ dependency.to_id }}
{% endfor %}

{%- for style_name, style in styles %}
    classDef {{ style_name }} fill:{{ style.fill_color }},stroke:{{ style.stroke_color }},stroke-width:{{ style.stroke_width }}px
{%- if style.text_color %},color:{{ style.text_color }}{% endif -%}
{% endfor %}"#,
        )
        .map_err(|e| MermaidGenerationError::TemplateLoadError(e.to_string()))?;

        // Sequence diagram template
        tera.add_raw_template(
            "sequence_diagram.tera",
            r#"sequenceDiagram
{% for component in components -%}
    participant {{ component.id }} as {{ component.name }}
{% endfor %}

{% for dependency in dependencies -%}
    {{ dependency.from_id }}->>{{ dependency.to_id }}: {{ dependency.type }}
{% endfor %}"#,
        )
        .map_err(|e| MermaidGenerationError::TemplateLoadError(e.to_string()))?;

        Ok(())
    }

    /// Create default diagram specifications
    fn create_default_specs() -> HashMap<DiagramType, DiagramSpec> {
        let mut specs = HashMap::new();

        // Component diagram spec
        specs.insert(
            DiagramType::Component,
            DiagramSpec {
                spec_id: Uuid::new_v4(),
                anti_pattern_type_id: 0, // Generic
                diagram_type: DiagramType::Component,
                mermaid_template: "component_template".to_string(),
                severity_styles: HashMap::new(),
                layout: crate::models::visualization::DiagramLayout::TopDown,
            },
        );

        // Dependency diagram spec
        specs.insert(
            DiagramType::Dependency,
            DiagramSpec {
                spec_id: Uuid::new_v4(),
                anti_pattern_type_id: 0, // Generic
                diagram_type: DiagramType::Dependency,
                mermaid_template: "dependency_template".to_string(),
                severity_styles: HashMap::new(),
                layout: crate::models::visualization::DiagramLayout::TopDown,
            },
        );

        specs
    }

    /// Clean up generated diagram by removing extra whitespace and formatting
    fn clean_generated_diagram(mermaid_src: &str) -> String {
        mermaid_src
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Default for MermaidGenerator {
    fn default() -> Self {
        Self::new().expect("Failed to create default MermaidGenerator")
    }
}

/// Errors that can occur during Mermaid diagram generation
#[derive(Debug, thiserror::Error)]
pub enum MermaidGenerationError {
    #[error("Unsupported diagram type: {0:?}")]
    UnsupportedDiagramType(DiagramType),
    
    #[error("Template render error: {0}")]
    TemplateRenderError(String),
    
    #[error("Template load error: {0}")]
    TemplateLoadError(String),
    
    #[error("Component processing error: {0}")]
    ComponentProcessingError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::visualization::{ComponentMetrics, ComponentType};
    use std::path::PathBuf;

    fn create_test_component() -> ArchitecturalComponent {
        ArchitecturalComponent {
            component_id: Uuid::new_v4(),
            name: "TestService".to_string(),
            file_path: PathBuf::from("src/services/test.rs"),
            component_type: ComponentType::Service,
            dependencies: Vec::new(),
            metrics: ComponentMetrics::default(),
            group: Some("services".to_string()),
        }
    }

    #[test]
    fn test_mermaid_generator_creation() {
        let generator = MermaidGenerator::new();
        assert!(generator.is_ok());
    }

    #[test]
    fn test_component_to_template_value() {
        let generator = MermaidGenerator::new().unwrap();
        let component = create_test_component();
        let severity_data = HashMap::new();
        
        let value = generator.component_to_template_value(&component, Some(&severity_data));
        
        assert_eq!(value["name"], "TestService");
        assert_eq!(value["type"], "Service");
    }

    #[test]
    fn test_default_severity_styles() {
        let critical_style = MermaidGenerator::get_default_severity_style("critical");
        assert_eq!(critical_style.fill_color, "#ff6b6b");
        assert_eq!(critical_style.stroke_width, 3);
        
        let low_style = MermaidGenerator::get_default_severity_style("low");
        assert_eq!(low_style.fill_color, "#1dd1a1");
        assert_eq!(low_style.stroke_width, 1);
    }

    #[test]
    fn test_clean_generated_diagram() {
        let messy_diagram = "  graph TD\n\n  A --> B  \n\n  \n  B --> C  ";
        let cleaned = MermaidGenerator::clean_generated_diagram(messy_diagram);
        
        assert_eq!(cleaned, "graph TD\nA --> B\nB --> C");
    }
}
