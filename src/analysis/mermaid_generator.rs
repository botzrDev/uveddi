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
        let mut tera = Tera::new("templates/*.tera").unwrap_or_else(|_| Tera::default());

        // Register built-in templates
        Self::register_builtin_templates(&mut tera)?;

        let diagram_specs = Self::create_default_specs();

        Ok(Self {
            template_engine: tera,
            diagram_specs,
        })
    }

    /// Generate a tight coupling diagram highlighting high coupling issues
    /// FIXED: UV-214 - Proper context creation with string keys for Tera template
    pub fn generate_tight_coupling_diagram(
        &self,
        components: &[ArchitecturalComponent],
        coupling_pairs: &[(Uuid, Uuid)],
        coupling_scores: &HashMap<(Uuid, Uuid), f64>,
    ) -> Result<crate::models::visualization::DiagramResult, MermaidGenerationError> {
        let mut context = Context::new();
        context.insert("layout", "TD");

        let coupled_components: HashSet<Uuid> = coupling_pairs
            .iter()
            .flat_map(|(a, b)| vec![*a, *b])
            .collect();

        // Transform components with coupling information for template
        let template_components: Vec<Value> = components
            .iter()
            .map(|c| {
                let is_coupled = coupled_components.contains(&c.component_id);
                let coupling_count = c.metrics.afferent_coupling + c.metrics.efferent_coupling;
                let coupling_level = if coupling_count > 15 {
                    "high"
                } else if coupling_count > 8 {
                    "medium"
                } else {
                    "low"
                };

                json!({
                    "id": c.component_id.to_string(),
                    "name": c.name,
                    "type": c.component_type.as_str(),
                    "is_coupled": is_coupled,
                    "coupling_level": coupling_level,
                    "coupling_count": coupling_count,
                    "icon": match coupling_level {
                        "high" => "[H]",
                        "medium" => "[M]",
                        _ => "[L]"
                    }
                })
            })
            .collect();

        // Transform dependencies with coupling strength information
        let template_dependencies: Vec<Value> = coupling_pairs
            .iter()
            .map(|(from_id, to_id)| {
                let score = coupling_scores.get(&(*from_id, *to_id)).unwrap_or(&0.0);
                let coupling_strength = if *score > 0.8 {
                    "very_high"
                } else if *score > 0.6 {
                    "high"
                } else if *score > 0.4 {
                    "medium"
                } else {
                    "low"
                };

                json!({
                    "from_id": from_id.to_string(),
                    "to_id": to_id.to_string(),
                    "type": "coupling",
                    "coupling_strength": coupling_strength,
                    "strength_value": format!("{:.1}", score)
                })
            })
            .collect();

        context.insert("components", &template_components);
        context.insert("dependencies", &template_dependencies);
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

    /// Generate a general diagram from components and diagram type
    pub fn generate_diagram(
        &self,
        components: &[ArchitecturalComponent],
        diagram_type: DiagramType,
    ) -> Result<crate::models::visualization::DiagramResult, MermaidGenerationError> {
        let mut context = Context::new();
        context.insert("layout", "TD");
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
            DiagramType::Dependency => "dependency_graph",
            DiagramType::Sequence => "sequence_diagram",
            _ => "component_diagram",
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
        context.insert("layout", "TD");

        let template_components: Vec<Value> = components
            .iter()
            .map(|c| {
                let is_dead = dead_components.contains(&c.component_id);
                json!({
                    "id": c.component_id.to_string(),
                    "name": c.name,
                    "type": c.component_type.as_str(),
                    "is_dead": is_dead,
                    "css_class": if is_dead { "dead-code-critical" } else { "normal-component" },
                    "usage_level": if is_dead { "dead" } else { "active" }
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
            DiagramType::Component,
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
        context.insert("layout", "TD");

        let template_components: Vec<Value> = components
            .iter()
            .map(|c| {
                let is_large = large_classes.contains(&c.component_id);
                let size = class_sizes.get(&c.component_id).unwrap_or(&0);
                let size_category = if *size > 500 {
                    "extra-large"
                } else if *size > 300 {
                    "large"
                } else {
                    "normal"
                };

                json!({
                    "id": c.component_id.to_string(),
                    "name": c.name,
                    "type": c.component_type.as_str(),
                    "is_large": is_large,
                    "size": size,
                    "size_category": size_category,
                    "css_class": if is_large { "large-class-critical" } else { "normal-component" }
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

    /// Generate a Cyclic Dependencies diagram highlighting circular references
    pub fn generate_cyclic_dependencies_diagram(
        &self,
        components: &[ArchitecturalComponent],
        cycles: &[Vec<Uuid>],
        cycle_edges: &[(Uuid, Uuid)],
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
                    "css_class": if is_in_cycle { "cycle-critical" } else { "normal-component" }
                })
            })
            .collect();

        context.insert("components", &template_components);
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
        // Component diagram template
        tera.add_raw_template(
            "component_diagram",
            r#"graph {{ layout }}
{% for component in components -%}
    {{ component.id }}["{{ component.name }}"]
{% endfor %}

{% for dependency in dependencies -%}
    {{ dependency.from }} --> {{ dependency.to }}
{% endfor %}

classDef default fill:#e1f5fe,stroke:#01579b,stroke-width:2px;"#,
        )
        .map_err(|e| MermaidGenerationError::TemplateLoadError(e.to_string()))?;

        Ok(())
    }

    /// Create default diagram specifications
    fn create_default_specs() -> HashMap<DiagramType, DiagramSpec> {
        let mut specs = HashMap::new();

        specs.insert(
            DiagramType::Component,
            DiagramSpec {
                spec_id: Uuid::new_v4(),
                anti_pattern_type_id: 1,
                diagram_type: DiagramType::Component,
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
                    "to": dep.to.id,
                    "type": "dependency"
                }));
            }
        }

        dependencies
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

#[cfg(test)]
mod tests {
    use super::*;

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
