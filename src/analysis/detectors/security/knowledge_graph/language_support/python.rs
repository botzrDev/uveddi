//! Python-specific knowledge graph support

use crate::analysis::detectors::security::knowledge_graph::types::{CodeEntity, EntityType, CodeLocation};
use crate::analysis::AnalysisError;
use crate::ast::SourceLanguage;
use std::collections::HashMap;
use std::path::PathBuf;

/// Python-specific entity processor
pub struct PythonEntityProcessor;

impl PythonEntityProcessor {
    pub fn new() -> Self {
        Self
    }

    /// Process Python-specific entities
    pub async fn process_python_entities(&self, file_path: &PathBuf, content: &str) -> Result<Vec<CodeEntity>, AnalysisError> {
        let mut entities = Vec::new();

        entities.extend(self.extract_functions(file_path, content)?);
        entities.extend(self.extract_classes(file_path, content)?);

        Ok(entities)
    }

    fn extract_functions(&self, file_path: &PathBuf, content: &str) -> Result<Vec<CodeEntity>, AnalysisError> {
        let mut functions = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("def ") {
                if let Some(entity) = self.parse_function_entity(file_path, line, line_num + 1)? {
                    functions.push(entity);
                }
            }
        }

        Ok(functions)
    }

    fn extract_classes(&self, file_path: &PathBuf, content: &str) -> Result<Vec<CodeEntity>, AnalysisError> {
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

    fn parse_function_entity(&self, file_path: &PathBuf, line: &str, line_num: usize) -> Result<Option<CodeEntity>, AnalysisError> {
        let trimmed = line.trim();
        if let Some(def_part) = trimmed.strip_prefix("def ") {
            if let Some(name_end) = def_part.find('(') {
                let name = def_part[..name_end].trim();
                let mut metadata = HashMap::new();

                // Check for async
                if line.contains("async def") {
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
                    language: SourceLanguage::Python,
                }));
            }
        }
        Ok(None)
    }

    fn parse_class_entity(&self, file_path: &PathBuf, line: &str, line_num: usize) -> Result<Option<CodeEntity>, AnalysisError> {
        let trimmed = line.trim();
        if let Some(class_part) = trimmed.strip_prefix("class ") {
            let name = class_part.split(['(', ':']).next().unwrap_or("unknown").trim();
            let metadata = HashMap::new();

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
                language: SourceLanguage::Python,
            }));
        }
        Ok(None)
    }
}