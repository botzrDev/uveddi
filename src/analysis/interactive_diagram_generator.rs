//! Interactive Diagram Generator - UV-89 Phase 1 Backend Integration
//!
//! This module extends the existing MermaidGenerator to provide metadata
//! and support for interactive diagram features.

use crate::analysis::mermaid_generator::{MermaidGenerator, MermaidGenerationError};
use crate::models::visualization::{ArchitecturalComponent, DiagramType as VizDiagramType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Extended metadata for interactive diagrams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveNodeMetadata {
    /// Node identifier in the diagram
    pub node_id: String,
    /// Component this node represents
    pub component_id: Uuid,
    /// Display name in the diagram
    pub name: String,
    /// Component type for styling and behavior
    pub component_type: String,
    /// File path for navigation
    pub file_path: String,
    /// Position in the diagram (if known)
    pub position: Option<NodePosition>,
    /// Connected node IDs
    pub connections: Vec<String>,
    /// Component metrics for tooltips
    pub metrics: Option<ComponentMetrics>,
    /// Additional metadata for context menus
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Position coordinates for diagram nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePosition {
    pub x: f64,
    pub y: f64,
}

/// Component metrics for interactive display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMetrics {
    pub lines_of_code: Option<u32>,
    pub complexity: Option<f64>,
    pub coupling: Option<f64>,
    pub cohesion: Option<f64>,
    pub maintainability_index: Option<f64>,
    pub test_coverage: Option<f64>,
}

/// Configuration for interactive features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionConfig {
    pub enable_double_click: bool,
    pub enable_context_menu: bool,
    pub enable_tooltips: bool,
    pub enable_pan_zoom: bool,
    pub enable_keyboard_nav: bool,
    pub double_click_delay: u32,
    pub tooltip_delay: u32,
}

impl Default for InteractionConfig {
    fn default() -> Self {
        Self {
            enable_double_click: true,
            enable_context_menu: true,
            enable_tooltips: true,
            enable_pan_zoom: true,
            enable_keyboard_nav: true,
            double_click_delay: 300,
            tooltip_delay: 500,
        }
    }
}

/// Result of interactive diagram generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveDiagramResult {
    /// Generated Mermaid diagram code
    pub mermaid_code: String,
    /// Metadata for each node in the diagram
    pub node_metadata: HashMap<String, InteractiveNodeMetadata>,
    /// Configuration for interactive features
    pub interaction_config: InteractionConfig,
    /// Diagram type
    pub diagram_type: String,
    /// Components included in this diagram
    pub components: Vec<Uuid>,
}

/// Interactive diagram generator that extends MermaidGenerator
pub struct InteractiveDiagramGenerator {
    /// Base Mermaid generator
    mermaid_generator: MermaidGenerator,
    /// Default interaction configuration
    default_config: InteractionConfig,
}

impl InteractiveDiagramGenerator {
    /// Create a new interactive diagram generator
    pub fn new() -> Result<Self, MermaidGenerationError> {
        let mermaid_generator = MermaidGenerator::new()?;
        Ok(Self {
            mermaid_generator,
            default_config: InteractionConfig::default(),
        })
    }

    /// Generate an interactive diagram from architectural components
    pub async fn generate_interactive_diagram(
        &self,
        components: &[ArchitecturalComponent],
        diagram_type: VizDiagramType,
        config: Option<InteractionConfig>,
    ) -> Result<InteractiveDiagramResult, MermaidGenerationError> {
        // Generate the base Mermaid diagram
        let diagram_result = self.mermaid_generator.generate_diagram(components, diagram_type.clone())?;
        
        // Extract node metadata from components
        let node_metadata = self.extract_node_metadata(components, &diagram_result.mermaid_src)?;
        
        // Use provided config or default
        let interaction_config = config.unwrap_or_else(|| self.default_config.clone());
        
        Ok(InteractiveDiagramResult {
            mermaid_code: diagram_result.mermaid_src,
            node_metadata,
            interaction_config,
            diagram_type: format!("{:?}", diagram_type),
            components: diagram_result.components,
        })
    }

    /// Extract node metadata from architectural components
    fn extract_node_metadata(
        &self,
        components: &[ArchitecturalComponent],
        mermaid_code: &str,
    ) -> Result<HashMap<String, InteractiveNodeMetadata>, MermaidGenerationError> {
        let mut metadata = HashMap::new();
        
        for component in components {
            let node_id = self.generate_node_id(component);
            
            // Find connections by analyzing the mermaid code
            let connections = self.extract_connections(&node_id, mermaid_code);
            
            let node_metadata = InteractiveNodeMetadata {
                node_id: node_id.clone(),
                component_id: component.component_id,
                name: component.name.clone(),
                component_type: format!("{:?}", component.component_type),
                file_path: component.file_path.to_string_lossy().to_string(),
                position: None, // Position will be determined by frontend rendering
                connections,
                metrics: Some(self.convert_metrics(&component.metrics)),
                metadata: self.generate_additional_metadata(component),
            };
            
            metadata.insert(node_id, node_metadata);
        }
        
        Ok(metadata)
    }

    /// Generate a consistent node ID for a component
    fn generate_node_id(&self, component: &ArchitecturalComponent) -> String {
        // Create a stable, unique node ID based on component data
        format!("node_{}", component.component_id.to_string().replace('-', "_"))
    }

    /// Extract connections for a node from Mermaid code
    fn extract_connections(&self, node_id: &str, mermaid_code: &str) -> Vec<String> {
        let mut connections = Vec::new();
        
        // Parse Mermaid code to find connections
        // This is a simplified implementation - could be enhanced with proper parsing
        for line in mermaid_code.lines() {
            let line = line.trim();
            if line.contains("-->") || line.contains("-.->") || line.contains("===") {
                if line.contains(node_id) {
                    // Extract connected node IDs
                    let parts: Vec<&str> = line.split("-->").collect();
                    if parts.len() == 2 {
                        let source = parts[0].trim();
                        let target = parts[1].trim();
                        
                        if source.contains(node_id) && !target.contains(node_id) {
                            if let Some(target_id) = self.extract_node_id_from_line(target) {
                                connections.push(target_id);
                            }
                        } else if target.contains(node_id) && !source.contains(node_id) {
                            if let Some(source_id) = self.extract_node_id_from_line(source) {
                                connections.push(source_id);
                            }
                        }
                    }
                }
            }
        }
        
        connections
    }

    /// Extract node ID from a Mermaid line
    fn extract_node_id_from_line(&self, line: &str) -> Option<String> {
        // Extract node ID from various Mermaid formats
        // E.g., "A[Component Name]" -> "A", "node_123" -> "node_123"
        if let Some(bracket_pos) = line.find('[') {
            Some(line[..bracket_pos].trim().to_string())
        } else if let Some(paren_pos) = line.find('(') {
            Some(line[..paren_pos].trim().to_string())
        } else {
            Some(line.trim().to_string())
        }
    }

    /// Convert component metrics to interactive format
    fn convert_metrics(&self, metrics: &crate::models::visualization::ComponentMetrics) -> ComponentMetrics {
        ComponentMetrics {
            lines_of_code: metrics.lines_of_code,
            complexity: metrics.complexity,
            coupling: Some((metrics.afferent_coupling + metrics.efferent_coupling) as f64),
            cohesion: metrics.coupling_between_objects, // Use coupling_between_objects as a proxy for cohesion
            maintainability_index: None, // Not available in the source metrics
            test_coverage: None, // Not available in the source metrics
        }
    }

    /// Generate additional metadata for context menus and tooltips
    fn generate_additional_metadata(&self, component: &ArchitecturalComponent) -> HashMap<String, serde_json::Value> {
        let mut metadata = HashMap::new();
        
        metadata.insert("group".to_string(), serde_json::Value::String(
            component.group.clone().unwrap_or_else(|| "default".to_string())
        ));
        
        metadata.insert("dependency_count".to_string(), serde_json::Value::Number(
            component.dependencies.len().into()
        ));
        
        // Add more metadata as needed
        if let Some(parent) = component.file_path.parent() {
            metadata.insert("directory".to_string(), serde_json::Value::String(
                parent.to_string_lossy().to_string()
            ));
        }
        
        metadata
    }

    /// Update interaction configuration
    pub fn set_interaction_config(&mut self, config: InteractionConfig) {
        self.default_config = config;
    }

    /// Get current interaction configuration
    pub fn get_interaction_config(&self) -> &InteractionConfig {
        &self.default_config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::visualization::{ComponentType, Dependency};
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_interactive_diagram_generation() {
        let generator = InteractiveDiagramGenerator::new().unwrap();
        
        let component = ArchitecturalComponent {
            component_id: Uuid::new_v4(),
            name: "TestComponent".to_string(),
            file_path: PathBuf::from("src/test.rs"),
            component_type: ComponentType::Module,
            dependencies: vec![],
            metrics: crate::models::visualization::ComponentMetrics::default(),
            group: Some("test_group".to_string()),
        };
        
        let result = generator.generate_interactive_diagram(
            &[component],
            VizDiagramType::Component,
            None,
        ).await;
        
        assert!(result.is_ok());
        let interactive_result = result.unwrap();
        assert!(!interactive_result.mermaid_code.is_empty());
        assert!(!interactive_result.node_metadata.is_empty());
    }

    #[test]
    fn test_node_id_generation() {
        let generator = InteractiveDiagramGenerator::new().unwrap();
        let component_id = Uuid::new_v4();
        
        let component = ArchitecturalComponent {
            component_id,
            name: "TestComponent".to_string(),
            file_path: PathBuf::from("src/test.rs"),
            component_type: ComponentType::Module,
            dependencies: vec![],
            metrics: crate::models::visualization::ComponentMetrics::default(),
            group: None,
        };
        
        let node_id = generator.generate_node_id(&component);
        assert!(node_id.starts_with("node_"));
        assert!(node_id.contains(&component_id.to_string().replace('-', "_")));
    }

    #[test]
    fn test_connection_extraction() {
        let generator = InteractiveDiagramGenerator::new().unwrap();
        let mermaid_code = "
            graph TD
            A[Component A] --> B[Component B]
            B --> C[Component C]
            node_123 --> node_456
        ";
        
        let connections = generator.extract_connections("A", mermaid_code);
        assert!(connections.contains(&"B".to_string()));
    }
}