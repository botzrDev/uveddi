//! Mermaid Template Engine for Diagram Generation
//!
//! This module provides template-based generation of Mermaid.js diagrams
//! from architectural components. It supports multiple diagram types and
//! severity-based styling for comprehensive visualization.

use crate::models::visualization::{
    ArchitecturalComponent, DiagramMetadata, DiagramSpec, DiagramType,
};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use tera::{Context, Tera};
use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur during Mermaid diagram generation
#[derive(Debug, Error)]
pub enum MermaidGenerationError {
    #[error("Template load error: {0}")]
    TemplateLoadError(String),
    #[error("Template render error: {0}")]
    TemplateRenderError(String),
    #[error("Invalid diagram specification: {0}")]
    InvalidSpecError(String),
    #[error("Component serialization error: {0}")]
    SerializationError(String),
}

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
        let mut tera = Tera::new("templates/diagrams/*")
            .unwrap_or_else(|_| Tera::new("").expect("Failed to create empty Tera instance"));

        // Register built-in templates
        Self::register_builtin_templates(&mut tera)?;

        let diagram_specs = Self::create_default_specs();

        Ok(Self {
            template_engine: tera,
            diagram_specs,
        })
    }

    /// Generate a God Object diagram highlighting classes with excessive responsibilities
    pub fn generate_god_object_diagram(
        &self,
        components: &[ArchitecturalComponent],
        god_object_components: &[Uuid],
        member_counts: &HashMap<Uuid, u32>,
    ) -> Result<DiagramMetadata, MermaidGenerationError> {
        let mut context = Context::new();
        context.insert("layout", "TD");

        // Transform components with God Object information
        let template_components: Vec<Value> = components
            .iter()
            .map(|c| {
                let is_god_object = god_object_components.contains(&c.component_id);
                let member_count = member_counts.get(&c.component_id).unwrap_or(&0);

                json!({
                    "id": c.component_id.to_string(),
                    "name": c.name,
                    "type": c.component_type.as_str(),
                    "is_god_object": is_god_object,
                    "member_count": member_count,
                    "css_class": if is_god_object { "god-object-critical" } else { "normal-component" },
                    "style": if is_god_object {
                        "fill:#FF4444,stroke:#FF0000,stroke-width:3px"
                    } else {
                        "fill:#E6F3FF,stroke:#1E88E5"
                    }
                })
            })
            .collect();

        context.insert("components", &template_components);
        context.insert(
            "dependencies",
            &self.extract_dependencies_for_template(components),
        );
        context.insert("title", "God Object Detection");

        let mermaid_src = self
            .template_engine
            .render("god_object_diagram", &context)
            .map_err(|e| MermaidGenerationError::TemplateRenderError(e.to_string()))?;

        Ok(DiagramMetadata {
            diagram_type: DiagramType::Class,
            mermaid_src: Self::clean_generated_diagram(&mermaid_src),
            image_path: None,
            generated_at: chrono::Utc::now(),
            components: components.iter().map(|c| c.component_id).collect(),
            validation_metrics: None,
        })
    }

    /// Generate a Cyclic Dependencies diagram highlighting circular references
    pub fn generate_cyclic_dependencies_diagram(
        &self,
        components: &[ArchitecturalComponent],
        cycles: &[Vec<Uuid>],         // Each inner Vec represents a cycle
        cycle_edges: &[(Uuid, Uuid)], // Edges that create cycles
    ) -> Result<DiagramMetadata, MermaidGenerationError> {
        let mut context = Context::new();
        context.insert("layout", "TD");

        // Flatten all cycle components
        let cycle_components: HashSet<Uuid> = cycles.iter().flatten().cloned().collect();
        let cycle_edge_set: HashSet<(Uuid, Uuid)> = cycle_edges.iter().cloned().collect();

        // Transform components with cycle information
        let template_components: Vec<Value> = components
            .iter()
            .map(|c| {
                let is_in_cycle = cycle_components.contains(&c.component_id);

                json!({
                    "id": c.component_id.to_string(),
                    "name": c.name,
                    "type": c.component_type.as_str(),
                    "is_in_cycle": is_in_cycle,
                    "css_class": if is_in_cycle { "cycle-component" } else { "normal-component" },
                    "style": if is_in_cycle {
                        "fill:#FF4444,stroke:#FF0000,stroke-width:3px"
                    } else {
                        "fill:#E6F3FF,stroke:#1E88E5"
                    }
                })
            })
            .collect();

        // Transform dependencies with cycle information (UV-150: error propagation)
        let template_dependencies: Vec<Value> = {
            let deps = self.extract_dependencies_for_template(components);
            let mut result = Vec::with_capacity(deps.len());
            for mut dep in deps {
                let dep_obj = dep.as_object_mut().ok_or_else(|| {
                    MermaidGenerationError::InvalidSpecError(
                        "Invalid dependency structure in diagram data (see UV-150 error handling policy)".to_string()
                    )
                })?;
                let from = dep_obj.get("from").and_then(|v| v.as_str()).unwrap_or("");
                let to = dep_obj.get("to").and_then(|v| v.as_str()).unwrap_or("");
                if let (Ok(from_uuid), Ok(to_uuid)) = (Uuid::parse_str(from), Uuid::parse_str(to)) {
                    let is_cycle_edge = cycle_edge_set.contains(&(from_uuid, to_uuid));
                    dep_obj.insert("is_cycle_edge".to_string(), json!(is_cycle_edge));
                    if is_cycle_edge {
                        dep_obj.insert(
                            "style".to_string(),
                            json!("stroke:#FF0000,stroke-width:4px"),
                        );
                    }
                }
                result.push(dep);
            }
            result
        };

        context.insert("components", &template_components);
        context.insert("dependencies", &template_dependencies);
        context.insert("cycles", cycles);
        context.insert("title", "Cyclic Dependencies Detection");

        let mermaid_src = self
            .template_engine
            .render("cyclic_dependencies_diagram", &context)
            .map_err(|e| MermaidGenerationError::TemplateRenderError(e.to_string()))?;

        Ok(DiagramMetadata {
            diagram_type: DiagramType::Graph,
            mermaid_src: Self::clean_generated_diagram(&mermaid_src),
            image_path: None,
            generated_at: chrono::Utc::now(),
            components: components.iter().map(|c| c.component_id).collect(),
            validation_metrics: None,
        })
    }

    /// Register built-in templates for diagram generation
    fn register_builtin_templates(tera: &mut Tera) -> Result<(), MermaidGenerationError> {
        // God Object diagram template
        tera.add_raw_template(
            "god_object_diagram",
            r#"graph {{ layout }}
{% for component in components -%}
    {{ component.id }}["{{ component.name }}"]
{% if component.css_class -%}
    class {{ component.id }} {{ component.css_class }}
{% endif -%}
{% endfor %}

{% for dependency in dependencies -%}
    {{ dependency.from }} --> {{ dependency.to }}
{% endfor %}

classDef god-object-critical fill:#ffdddd,stroke:#f00,stroke-width:2px;
classDef normal-component fill:#e1f5fe,stroke:#01579b,stroke-width:2px;"#,
        )
        .map_err(|e| MermaidGenerationError::TemplateLoadError(e.to_string()))?;

        // Cyclic Dependencies diagram template
        tera.add_raw_template(
            "cyclic_dependencies_diagram",
            r#"graph {{ layout }}
{% for component in components -%}
    {{ component.id }}["{{ component.name }}"]
{% if component.css_class -%}
    class {{ component.id }} {{ component.css_class }}
{% endif -%}
{% endfor %}

{% for dependency in dependencies -%}
    {{ dependency.from }} --> {{ dependency.to }}
{% if dependency.is_cycle_edge -%}
    linkStyle {{ loop.index0 }} stroke:#FF0000,stroke-width:4px
{% endif -%}
{% endfor %}

classDef cycle-component fill:#ffdddd,stroke:#f00,stroke-width:2px;
classDef normal-component fill:#e1f5fe,stroke:#01579b,stroke-width:2px;"#,
        )
        .map_err(|e| MermaidGenerationError::TemplateLoadError(e.to_string()))?;

        Ok(())
    }

    /// Create default diagram specifications
    fn create_default_specs() -> HashMap<DiagramType, DiagramSpec> {
        let mut specs = HashMap::new();

        specs.insert(
            DiagramType::Class,
            DiagramSpec {
                spec_id: Uuid::new_v4(),
                anti_pattern_type_id: 1,
                diagram_type: DiagramType::Class,
                mermaid_template:
                    "classDiagram\n{{#each components}}\n    class {{name}}\n{{/each}}".to_string(),
                severity_styles: HashMap::new(),
                layout: crate::models::visualization::DiagramLayout::TopDown,
            },
        );

        specs.insert(
            DiagramType::Graph,
            DiagramSpec {
                spec_id: Uuid::new_v4(),
                anti_pattern_type_id: 2,
                diagram_type: DiagramType::Graph,
                mermaid_template: "graph TD\n{{#each components}}\n    {{id}}[{{name}}]\n{{/each}}"
                    .to_string(),
                severity_styles: HashMap::new(),
                layout: crate::models::visualization::DiagramLayout::TopDown,
            },
        );

        specs
    }

    /// Extract dependencies in template-friendly format
    fn extract_dependencies_for_template(
        &self,
        components: &[ArchitecturalComponent],
    ) -> Vec<Value> {
        let mut dependencies = Vec::new();

        for component in components {
            for dep in &component.dependencies {
                dependencies.push(json!({
                    "from": component.component_id.to_string(),
                    "to": dep,
                    "type": "dependency"
                }));
            }
        }

        dependencies
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

    /// Generate a general diagram from components and diagram type
    pub fn generate_diagram(
        &self,
        components: &[ArchitecturalComponent],
        diagram_type: DiagramType,
    ) -> Result<crate::models::visualization::DiagramResult, MermaidGenerationError> {
        let mut context = Context::new();
        context.insert(
            "components",
            &self.extract_components_for_template(components),
        );
        context.insert(
            "dependencies",
            &self.extract_dependencies_for_template(components),
        );

        let template_name = match diagram_type {
            DiagramType::Component => "component_diagram",
            DiagramType::Class => "class_diagram",
            DiagramType::Dependency => "dependency_diagram",
            _ => "default_diagram",
        };

        let mermaid_src = self
            .template_engine
            .render(template_name, &context)
            .map_err(|e| MermaidGenerationError::TemplateRenderError(e.to_string()))
            .map(|s| Self::clean_generated_diagram(&s))?;

        let component_ids = components.iter().map(|c| c.component_id).collect();
        Ok(crate::models::visualization::DiagramResult::new(
            diagram_type,
            mermaid_src,
            component_ids,
        ))
    }

    /// Generate a dead code diagram highlighting unused components
    pub fn generate_dead_code_diagram(
        &self,
        components: &[ArchitecturalComponent],
        dead_components: &[Uuid],
    ) -> Result<crate::models::visualization::DiagramResult, MermaidGenerationError> {
        let mut context = Context::new();

        let template_components: Vec<Value> = components
            .iter()
            .map(|c| {
                let is_dead = dead_components.contains(&c.component_id);
                json!({
                    "id": c.component_id.to_string(),
                    "name": c.name,
                    "type": c.component_type.as_str(),
                    "is_dead": is_dead,
                    "style": if is_dead {
                        "fill:#FFDDDD,stroke:#FF0000,stroke-dasharray: 5 5"
                    } else {
                        "fill:#E6F3FF,stroke:#1E88E5"
                    }
                })
            })
            .collect();

        context.insert("components", &template_components);
        context.insert(
            "dependencies",
            &self.extract_dependencies_for_template(components),
        );
        context.insert("title", "Dead Code Detection");

        let mermaid_src = self
            .template_engine
            .render("dead_code_diagram", &context)
            .map_err(|e| MermaidGenerationError::TemplateRenderError(e.to_string()))
            .map(|s| Self::clean_generated_diagram(&s))?;

        let component_ids = components.iter().map(|c| c.component_id).collect();
        Ok(crate::models::visualization::DiagramResult::new(
            DiagramType::Component, // or a specific dead code diagram type
            mermaid_src,
            component_ids,
        ))
    }

    /// Generate a large class diagram highlighting oversized classes
    pub fn generate_large_class_diagram(
        &self,
        components: &[ArchitecturalComponent],
        large_classes: &[Uuid],
        class_sizes: &HashMap<Uuid, u32>,
    ) -> Result<crate::models::visualization::DiagramResult, MermaidGenerationError> {
        let mut context = Context::new();

        let template_components: Vec<Value> = components
            .iter()
            .map(|c| {
                let is_large = large_classes.contains(&c.component_id);
                let size = class_sizes.get(&c.component_id).unwrap_or(&0);
                json!({
                    "id": c.component_id.to_string(),
                    "name": c.name,
                    "type": c.component_type.as_str(),
                    "is_large": is_large,
                    "size": size,
                    "style": if is_large {
                        "fill:#FFE6CC,stroke:#FF8800,stroke-width:3px"
                    } else {
                        "fill:#E6F3FF,stroke:#1E88E5"
                    }
                })
            })
            .collect();

        context.insert("components", &template_components);
        context.insert(
            "dependencies",
            &self.extract_dependencies_for_template(components),
        );
        context.insert("title", "Large Class Detection");

        let mermaid_src = self
            .template_engine
            .render("large_class_diagram", &context)
            .map_err(|e| MermaidGenerationError::TemplateRenderError(e.to_string()))
            .map(|s| Self::clean_generated_diagram(&s))?;

        let component_ids = components.iter().map(|c| c.component_id).collect();
        Ok(crate::models::visualization::DiagramResult::new(
            DiagramType::Class,
            mermaid_src,
            component_ids,
        ))
    }

    /// Generate a tight coupling diagram highlighting high coupling issues
    pub fn generate_tight_coupling_diagram(
        &self,
        components: &[ArchitecturalComponent],
        coupling_pairs: &[(Uuid, Uuid)],
        coupling_scores: &HashMap<(Uuid, Uuid), f64>,
    ) -> Result<crate::models::visualization::DiagramResult, MermaidGenerationError> {
        let mut context = Context::new();

        let coupled_components: std::collections::HashSet<Uuid> = coupling_pairs
            .iter()
            .flat_map(|(a, b)| vec![*a, *b])
            .collect();

        let template_components: Vec<Value> = components
            .iter()
            .map(|c| {
                let is_coupled = coupled_components.contains(&c.component_id);
                json!({
                    "id": c.component_id.to_string(),
                    "name": c.name,
                    "type": c.component_type.as_str(),
                    "is_coupled": is_coupled,
                    "style": if is_coupled {
                        "fill:#FFCCCC,stroke:#CC0000,stroke-width:2px"
                    } else {
                        "fill:#E6F3FF,stroke:#1E88E5"
                    }
                })
            })
            .collect();

        context.insert("components", &template_components);
        context.insert(
            "dependencies",
            &self.extract_dependencies_for_template(components),
        );
        context.insert("coupling_pairs", &coupling_pairs);
        context.insert("coupling_scores", &coupling_scores);
        context.insert("title", "Tight Coupling Detection");

        let mermaid_src = self
            .template_engine
            .render("tight_coupling_diagram", &context)
            .map_err(|e| MermaidGenerationError::TemplateRenderError(e.to_string()))
            .map(|s| Self::clean_generated_diagram(&s))?;

        let component_ids = components.iter().map(|c| c.component_id).collect();
        Ok(crate::models::visualization::DiagramResult::new(
            DiagramType::Dependency,
            mermaid_src,
            component_ids,
        ))
    }

    /// Extract components in a format suitable for templates
    fn extract_components_for_template(&self, components: &[ArchitecturalComponent]) -> Vec<Value> {
        components
            .iter()
            .map(|c| {
                json!({
                    "id": c.component_id.to_string(),
                    "name": c.name,
                    "type": c.component_type.as_str(),
                    "file_path": c.file_path.to_string_lossy(),
                    "group": c.group
                })
            })
            .collect()
    }
}

impl Default for MermaidGenerator {
    fn default() -> Self {
        Self::new().expect("Failed to create default MermaidGenerator")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_mermaid_generator_creation() {
        let generator = MermaidGenerator::new();
        assert!(generator.is_ok());
    }

    #[test]
    fn test_clean_generated_diagram() {
        let messy_diagram = r#"graph TD
  A --> B  

  
  B --> C  "#;
        let cleaned = MermaidGenerator::clean_generated_diagram(messy_diagram);

        assert_eq!(cleaned, "graph TD\nA --> B\nB --> C");
    }
}
