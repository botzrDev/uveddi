//! AST to Architectural Component Mapping
//!
//! This module provides functionality to extract architectural components
//! from parsed ASTs, implementing the first stage of the visualization pipeline.
//! It transforms tree-sitter AST nodes into structured architectural components
//! that can be used for diagram generation.

use crate::ast::tree_sitter::{CustomAst, ParsedFile};
use crate::models::visualization::{
    ArchitecturalComponent, ComponentMetrics, ComponentType, Dependency, DependencyType,
};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// Component extractor that transforms AST data into architectural components
pub struct ComponentExtractor {
    /// Cache of extracted components by file path
    component_cache: HashMap<PathBuf, Vec<ArchitecturalComponent>>,
    /// Global component registry for dependency resolution
    component_registry: HashMap<String, Uuid>,
}

impl ComponentExtractor {
    /// Create a new component extractor
    pub fn new() -> Self {
        Self {
            component_cache: HashMap::new(),
            component_registry: HashMap::new(),
        }
    }

    /// Extract architectural components from a collection of parsed files
    ///
    /// # Arguments
    ///
    /// * `parsed_files` - Collection of parsed source files with AST data
    ///
    /// # Returns
    ///
    /// * `Vec<ArchitecturalComponent>` - List of extracted architectural components
    pub fn extract_components(
        &mut self,
        parsed_files: &[ParsedFile],
    ) -> Result<Vec<ArchitecturalComponent>, ComponentExtractionError> {
        let mut all_components = Vec::new();

        // First pass: Extract all components and build registry
        for parsed_file in parsed_files {
            let components = self.extract_from_file(parsed_file)?;
            
            // Register components for dependency resolution
            for component in &components {
                self.component_registry
                    .insert(component.name.clone(), component.component_id);
            }
            
            self.component_cache
                .insert(parsed_file.path.clone(), components.clone());
            all_components.extend(components);
        }

        // Second pass: Resolve dependencies
        self.resolve_dependencies(&mut all_components, parsed_files)?;

        Ok(all_components)
    }

    /// Extract components from a single parsed file
    fn extract_from_file(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalComponent>, ComponentExtractionError> {
        let mut components = Vec::new();

        if let Some(ref ast) = parsed_file.custom_ast {
            components.extend(self.extract_from_ast_node(ast, &parsed_file.path)?);
        }

        Ok(components)
    }

    /// Extract components from an AST node recursively
    fn extract_from_ast_node(
        &self,
        ast_node: &CustomAst,
        file_path: &PathBuf,
    ) -> Result<Vec<ArchitecturalComponent>, ComponentExtractionError> {
        let mut components = Vec::new();

        match ast_node {
            CustomAst::File { items } => {
                // For file nodes, extract all child components
                for item in items {
                    components.extend(self.extract_from_ast_node(item, file_path)?);
                }
            }
            CustomAst::Struct { name, methods } => {
                // Create a component for the struct/class
                let component = ArchitecturalComponent {
                    component_id: Uuid::new_v4(),
                    name: name.clone(),
                    file_path: file_path.clone(),
                    component_type: ComponentType::Class,
                    dependencies: Vec::new(), // Will be resolved in second pass
                    metrics: ComponentMetrics {
                        public_methods: Some(methods.len() as u32),
                        ..ComponentMetrics::default()
                    },
                    group: self.infer_group_from_path(file_path),
                };
                components.push(component);
            }
            CustomAst::Function { name, params } => {
                // Create a component for the function
                let component = ArchitecturalComponent {
                    component_id: Uuid::new_v4(),
                    name: name.clone(),
                    file_path: file_path.clone(),
                    component_type: ComponentType::Function,
                    dependencies: Vec::new(), // Will be resolved in second pass
                    metrics: ComponentMetrics {
                        complexity: Some(self.estimate_complexity(params.len())),
                        ..ComponentMetrics::default()
                    },
                    group: self.infer_group_from_path(file_path),
                };
                components.push(component);
            }
            CustomAst::Variable { name } => {
                // For variables, we might create components only for significant ones
                // like database connections, services, etc.
                if self.is_architectural_variable(name) {
                    let component = ArchitecturalComponent {
                        component_id: Uuid::new_v4(),
                        name: name.clone(),
                        file_path: file_path.clone(),
                        component_type: self.infer_component_type_from_name(name),
                        dependencies: Vec::new(),
                        metrics: ComponentMetrics::default(),
                        group: self.infer_group_from_path(file_path),
                    };
                    components.push(component);
                }
            }
        }

        Ok(components)
    }

    /// Resolve dependencies between components in a second pass
    fn resolve_dependencies(
        &self,
        components: &mut [ArchitecturalComponent],
        parsed_files: &[ParsedFile],
    ) -> Result<(), ComponentExtractionError> {
        // Build a lookup map for quick component resolution
        let mut name_to_component: HashMap<String, usize> = HashMap::new();
        for (index, component) in components.iter().enumerate() {
            name_to_component.insert(component.name.clone(), index);
        }

        // Analyze each file for dependency patterns
        for parsed_file in parsed_files {
            if let Some(ref ast) = parsed_file.custom_ast {
                self.analyze_dependencies_in_ast(
                    ast,
                    components,
                    &name_to_component,
                    &parsed_file.path,
                )?;
            }
        }

        Ok(())
    }

    /// Analyze AST for dependency patterns and update component dependencies
    fn analyze_dependencies_in_ast(
        &self,
        ast_node: &CustomAst,
        components: &mut [ArchitecturalComponent],
        name_to_component: &HashMap<String, usize>,
        current_file: &PathBuf,
    ) -> Result<(), ComponentExtractionError> {
        match ast_node {
            CustomAst::File { items } => {
                for item in items {
                    self.analyze_dependencies_in_ast(
                        item,
                        components,
                        name_to_component,
                        current_file,
                    )?;
                }
            }
            CustomAst::Struct { name, methods } => {
                // Look for method calls that might indicate dependencies
                for method in methods {
                    if let Some(target_index) = self.find_method_target(method, name_to_component) {
                        if let Some(source_index) = name_to_component.get(name) {
                            let target_id = components[target_index].component_id;
                            let dependency = Dependency {
                                target_component_id: target_id,
                                dependency_type: DependencyType::Calls,
                                properties: HashMap::new(),
                            };
                            
                            // Avoid duplicate dependencies
                            if !components[*source_index]
                                .dependencies
                                .iter()
                                .any(|d| d.target_component_id == target_id)
                            {
                                components[*source_index].dependencies.push(dependency);
                            }
                        }
                    }
                }
            }
            CustomAst::Function { name, params: _ } => {
                // Analyze function body for calls (simplified heuristic)
                if let Some(target_name) = self.extract_function_call_target(name) {
                    if let Some(target_index) = name_to_component.get(&target_name) {
                        if let Some(source_index) = name_to_component.get(name) {
                            let target_id = components[*target_index].component_id;
                            let dependency = Dependency {
                                target_component_id: target_id,
                                dependency_type: DependencyType::Calls,
                                properties: HashMap::new(),
                            };
                            
                            if !components[*source_index]
                                .dependencies
                                .iter()
                                .any(|d| d.target_component_id == target_id)
                            {
                                components[*source_index].dependencies.push(dependency);
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Infer the architectural group from file path
    fn infer_group_from_path(&self, file_path: &PathBuf) -> Option<String> {
        if let Some(parent) = file_path.parent() {
            if let Some(parent_name) = parent.file_name() {
                return Some(parent_name.to_string_lossy().to_string());
            }
        }
        None
    }

    /// Check if a variable name indicates an architectural component
    fn is_architectural_variable(&self, name: &str) -> bool {
        let architectural_patterns = [
            "db", "database", "connection", "client", "service", "handler",
            "router", "middleware", "cache", "queue", "broker", "gateway",
        ];
        
        let name_lower = name.to_lowercase();
        architectural_patterns
            .iter()
            .any(|pattern| name_lower.contains(pattern))
    }

    /// Infer component type from variable name
    fn infer_component_type_from_name(&self, name: &str) -> ComponentType {
        let name_lower = name.to_lowercase();
        
        if name_lower.contains("db") || name_lower.contains("database") {
            ComponentType::Database
        } else if name_lower.contains("cache") {
            ComponentType::Cache
        } else if name_lower.contains("service") {
            ComponentType::Service
        } else if name_lower.contains("handler") || name_lower.contains("router") {
            ComponentType::ApiEndpoint
        } else if name_lower.contains("queue") || name_lower.contains("broker") {
            ComponentType::MessageBroker
        } else {
            ComponentType::Module
        }
    }

    /// Estimate complexity based on parameter count (simplified heuristic)
    fn estimate_complexity(&self, param_count: usize) -> f64 {
        // Simple linear estimation - in reality would use proper AST analysis
        1.0 + (param_count as f64 * 0.5)
    }

    /// Find target component for a method call (simplified)
    fn find_method_target(
        &self,
        method_name: &str,
        name_to_component: &HashMap<String, usize>,
    ) -> Option<usize> {
        // Very simplified heuristic - in reality would need proper call analysis
        for (component_name, &index) in name_to_component {
            if method_name.to_lowercase().contains(&component_name.to_lowercase()) {
                return Some(index);
            }
        }
        None
    }

    /// Extract function call target from function name (simplified)
    fn extract_function_call_target(&self, function_name: &str) -> Option<String> {
        // Simplified heuristic - look for common function naming patterns
        if function_name.contains("_") {
            let parts: Vec<&str> = function_name.split('_').collect();
            if parts.len() > 1 {
                return Some(parts[0].to_string());
            }
        }
        None
    }
}

impl Default for ComponentExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during component extraction
#[derive(Debug, thiserror::Error)]
pub enum ComponentExtractionError {
    #[error("Failed to parse AST node: {0}")]
    AstParseError(String),
    
    #[error("Dependency resolution failed: {0}")]
    DependencyResolutionError(String),
    
    #[error("Component registration failed: {0}")]
    ComponentRegistrationError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::tree_sitter::SourceLanguage;
    use std::time::SystemTime;

    fn create_test_parsed_file() -> ParsedFile {
        ParsedFile {
            path: PathBuf::from("src/services/user.rs"),
            language: SourceLanguage::Rust,
            tree: None,
            source: "".to_string(),
            custom_ast: Some(CustomAst::File {
                items: vec![
                    CustomAst::Struct {
                        name: "UserService".to_string(),
                        methods: vec!["create_user".to_string(), "get_user".to_string()],
                    },
                    CustomAst::Function {
                        name: "validate_email".to_string(),
                        params: vec!["email".to_string()],
                    },
                ],
            }),
            modified_at: SystemTime::now(),
        }
    }

    #[test]
    fn test_component_extraction() {
        let mut extractor = ComponentExtractor::new();
        let parsed_files = vec![create_test_parsed_file()];
        
        let components = extractor.extract_components(&parsed_files).unwrap();
        
        assert_eq!(components.len(), 2);
        assert!(components.iter().any(|c| c.name == "UserService"));
        assert!(components.iter().any(|c| c.name == "validate_email"));
    }

    #[test]
    fn test_architectural_variable_detection() {
        let extractor = ComponentExtractor::new();
        
        assert!(extractor.is_architectural_variable("database_client"));
        assert!(extractor.is_architectural_variable("user_service"));
        assert!(extractor.is_architectural_variable("redis_cache"));
        assert!(!extractor.is_architectural_variable("temp_var"));
    }

    #[test]
    fn test_component_type_inference() {
        let extractor = ComponentExtractor::new();
        
        assert_eq!(
            extractor.infer_component_type_from_name("database_client"),
            ComponentType::Database
        );
        assert_eq!(
            extractor.infer_component_type_from_name("user_service"),
            ComponentType::Service
        );
        assert_eq!(
            extractor.infer_component_type_from_name("redis_cache"),
            ComponentType::Cache
        );
    }
}
