//! JavaScript/TypeScript-specific knowledge graph support

use crate::analysis::detectors::security::knowledge_graph::types::{
    CodeEntity, CodeLocation, EntityType,
};
use crate::analysis::AnalysisError;
use crate::ast::SourceLanguage;
use std::collections::HashMap;
use std::path::PathBuf;

/// JavaScript/TypeScript-specific entity processor
pub struct JavaScriptEntityProcessor;

impl JavaScriptEntityProcessor {
    pub fn new() -> Self {
        Self
    }

    /// Process JavaScript/TypeScript-specific entities
    pub async fn process_js_entities(
        &self,
        file_path: &PathBuf,
        content: &str,
    ) -> Result<Vec<CodeEntity>, AnalysisError> {
        let mut entities = Vec::new();

        entities.extend(self.extract_functions(file_path, content)?);
        entities.extend(self.extract_classes(file_path, content)?);
        entities.extend(self.extract_interfaces(file_path, content)?);

        Ok(entities)
    }

    fn extract_functions(
        &self,
        file_path: &PathBuf,
        content: &str,
    ) -> Result<Vec<CodeEntity>, AnalysisError> {
        let mut functions = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("function ")
                || trimmed.contains("=> ")
                || trimmed.contains(": function")
            {
                if let Some(entity) = self.parse_function_entity(file_path, line, line_num + 1)? {
                    functions.push(entity);
                }
            }
        }

        Ok(functions)
    }

    fn extract_classes(
        &self,
        file_path: &PathBuf,
        content: &str,
    ) -> Result<Vec<CodeEntity>, AnalysisError> {
        let mut classes = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("class ") {
                if let Some(entity) = self.parse_class_entity(file_path, line, line_num + 1)? {
                    classes.push(entity);
                }
            }
        }

        Ok(classes)
    }

    fn extract_interfaces(
        &self,
        file_path: &PathBuf,
        content: &str,
    ) -> Result<Vec<CodeEntity>, AnalysisError> {
        let mut interfaces = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("interface ") {
                if let Some(entity) = self.parse_interface_entity(file_path, line, line_num + 1)? {
                    interfaces.push(entity);
                }
            }
        }

        Ok(interfaces)
    }

    fn parse_function_entity(
        &self,
        file_path: &PathBuf,
        line: &str,
        line_num: usize,
    ) -> Result<Option<CodeEntity>, AnalysisError> {
        let trimmed = line.trim();
        let mut metadata = HashMap::new();

        let name = if let Some(func_part) = trimmed.strip_prefix("function ") {
            func_part.split(['(', ' ']).next().unwrap_or("unknown")
        } else if trimmed.contains("=> ") {
            // Arrow function
            let parts: Vec<&str> = trimmed.split("=>").collect();
            if !parts.is_empty() {
                let left = parts[0].trim();
                if let Some(eq_pos) = left.rfind('=') {
                    let name_part = left[..eq_pos].trim();
                    if let Some(name) = name_part.split_whitespace().last() {
                        metadata.insert("arrow_function".to_string(), true.into());
                        name
                    } else {
                        "anonymous"
                    }
                } else {
                    "anonymous"
                }
            } else {
                "anonymous"
            }
        } else {
            "unknown"
        };

        // Check for async
        if line.contains("async ") {
            metadata.insert("async".to_string(), true.into());
        }

        return Ok(Some(CodeEntity {
            id: format!("{}::{}", file_path.display(), name),
            name: name.to_string(),
            entity_type: EntityType::Function,
            location: CodeLocation {
                file_path: file_path.clone(),
                start_line: line_num as u32,
                end_line: line_num as u32,
                start_column: 0,
                end_column: 0,
            },
            metadata,
            language: SourceLanguage::JavaScript,
        }));
    }

    fn parse_class_entity(
        &self,
        file_path: &PathBuf,
        line: &str,
        line_num: usize,
    ) -> Result<Option<CodeEntity>, AnalysisError> {
        let trimmed = line.trim();
        if let Some(class_part) = trimmed.strip_prefix("class ") {
            let name = class_part
                .split([' ', '{'])
                .next()
                .unwrap_or("unknown")
                .trim();
            let mut metadata = HashMap::new();

            if line.contains("extends ") {
                metadata.insert("has_inheritance".to_string(), true.into());
            }

            return Ok(Some(CodeEntity {
                id: format!("{}::{}", file_path.display(), name),
                name: name.to_string(),
                entity_type: EntityType::Class,
                location: CodeLocation {
                    file_path: file_path.clone(),
                    start_line: line_num as u32,
                    end_line: line_num as u32,
                    start_column: 0,
                    end_column: 0,
                },
                metadata,
                language: SourceLanguage::JavaScript,
            }));
        }
        Ok(None)
    }

    fn parse_interface_entity(
        &self,
        file_path: &PathBuf,
        line: &str,
        line_num: usize,
    ) -> Result<Option<CodeEntity>, AnalysisError> {
        let trimmed = line.trim();
        if let Some(interface_part) = trimmed.strip_prefix("interface ") {
            let name = interface_part
                .split([' ', '{'])
                .next()
                .unwrap_or("unknown")
                .trim();
            let metadata = HashMap::new();

            return Ok(Some(CodeEntity {
                id: format!("{}::{}", file_path.display(), name),
                name: name.to_string(),
                entity_type: EntityType::Interface,
                location: CodeLocation {
                    file_path: file_path.clone(),
                    start_line: line_num as u32,
                    end_line: line_num as u32,
                    start_column: 0,
                    end_column: 0,
                },
                metadata,
                language: SourceLanguage::JavaScript,
            }));
        }
        Ok(None)
    }
}
