//! Rust-specific knowledge graph support

use crate::analysis::detectors::security::knowledge_graph::types::{CodeEntity, EntityType, CodeLocation};
use crate::analysis::AnalysisError;
use crate::ast::SourceLanguage;
use std::collections::HashMap;
use std::path::PathBuf;

/// Rust-specific entity processor
pub struct RustEntityProcessor;

impl RustEntityProcessor {
    pub fn new() -> Self {
        Self
    }

    /// Process Rust-specific entities
    pub async fn process_rust_entities(&self, file_path: &PathBuf, content: &str) -> Result<Vec<CodeEntity>, AnalysisError> {
        let mut entities = Vec::new();

        // Extract functions
        entities.extend(self.extract_functions(file_path, content)?);

        // Extract structs
        entities.extend(self.extract_structs(file_path, content)?);

        // Extract enums
        entities.extend(self.extract_enums(file_path, content)?);

        // Extract traits
        entities.extend(self.extract_traits(file_path, content)?);

        Ok(entities)
    }

    fn extract_functions(&self, file_path: &PathBuf, content: &str) -> Result<Vec<CodeEntity>, AnalysisError> {
        let mut functions = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            if line.trim_start().starts_with("fn ") {
                if let Some(entity) = self.parse_function_entity(file_path, line, line_num + 1)? {
                    functions.push(entity);
                }
            }
        }

        Ok(functions)
    }

    fn extract_structs(&self, file_path: &PathBuf, content: &str) -> Result<Vec<CodeEntity>, AnalysisError> {
        let mut structs = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            if line.trim_start().starts_with("struct ") {
                if let Some(entity) = self.parse_struct_entity(file_path, line, line_num + 1)? {
                    structs.push(entity);
                }
            }
        }

        Ok(structs)
    }

    fn extract_enums(&self, file_path: &PathBuf, content: &str) -> Result<Vec<CodeEntity>, AnalysisError> {
        let mut enums = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            if line.trim_start().starts_with("enum ") {
                if let Some(entity) = self.parse_enum_entity(file_path, line, line_num + 1)? {
                    enums.push(entity);
                }
            }
        }

        Ok(enums)
    }

    fn extract_traits(&self, file_path: &PathBuf, content: &str) -> Result<Vec<CodeEntity>, AnalysisError> {
        let mut traits = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            if line.trim_start().starts_with("trait ") {
                if let Some(entity) = self.parse_trait_entity(file_path, line, line_num + 1)? {
                    traits.push(entity);
                }
            }
        }

        Ok(traits)
    }

    fn parse_function_entity(&self, file_path: &PathBuf, line: &str, line_num: usize) -> Result<Option<CodeEntity>, AnalysisError> {
        let trimmed = line.trim();
        if let Some(fn_part) = trimmed.strip_prefix("fn ") {
            if let Some(name_end) = fn_part.find('(') {
                let name = fn_part[..name_end].trim();
                let mut metadata = HashMap::new();

                // Check for visibility
                if line.contains("pub ") {
                    metadata.insert("visibility".to_string(), "public".into());
                }

                // Check for unsafe
                if line.contains("unsafe ") {
                    metadata.insert("unsafe".to_string(), true.into());
                }

                return Ok(Some(CodeEntity {
                    id: format!("{}::{}", file_path.display(), name),
                    name: name.to_string(),
                    entity_type: EntityType::Function,
                    location: CodeLocation {
                        file_path: file_path.clone(),
                        start_line: line_num as u32,
                        end_line: line_num as u32, // Would need proper parsing for end
                        start_column: 0,
                        end_column: 0,
                    },
                    metadata,
                    language: SourceLanguage::Rust,
                }));
            }
        }
        Ok(None)
    }

    fn parse_struct_entity(&self, file_path: &PathBuf, line: &str, line_num: usize) -> Result<Option<CodeEntity>, AnalysisError> {
        let trimmed = line.trim();
        if let Some(struct_part) = trimmed.strip_prefix("struct ") {
            let name = struct_part.split_whitespace().next().unwrap_or("unknown");
            let mut metadata = HashMap::new();

            if line.contains("pub ") {
                metadata.insert("visibility".to_string(), "public".into());
            }

            return Ok(Some(CodeEntity {
                id: format!("{}::{}", file_path.display(), name),
                name: name.to_string(),
                entity_type: EntityType::Struct,
                location: CodeLocation {
                    file_path: file_path.clone(),
                    start_line: line_num as u32,
                    end_line: line_num as u32,
                    start_column: 0,
                    end_column: 0,
                },
                metadata,
                language: SourceLanguage::Rust,
            }));
        }
        Ok(None)
    }

    fn parse_enum_entity(&self, file_path: &PathBuf, line: &str, line_num: usize) -> Result<Option<CodeEntity>, AnalysisError> {
        let trimmed = line.trim();
        if let Some(enum_part) = trimmed.strip_prefix("enum ") {
            let name = enum_part.split_whitespace().next().unwrap_or("unknown");
            let mut metadata = HashMap::new();

            if line.contains("pub ") {
                metadata.insert("visibility".to_string(), "public".into());
            }

            return Ok(Some(CodeEntity {
                id: format!("{}::{}", file_path.display(), name),
                name: name.to_string(),
                entity_type: EntityType::Enum,
                location: CodeLocation {
                    file_path: file_path.clone(),
                    start_line: line_num as u32,
                    end_line: line_num as u32,
                    start_column: 0,
                    end_column: 0,
                },
                metadata,
                language: SourceLanguage::Rust,
            }));
        }
        Ok(None)
    }

    fn parse_trait_entity(&self, file_path: &PathBuf, line: &str, line_num: usize) -> Result<Option<CodeEntity>, AnalysisError> {
        let trimmed = line.trim();
        if let Some(trait_part) = trimmed.strip_prefix("trait ") {
            let name = trait_part.split_whitespace().next().unwrap_or("unknown");
            let mut metadata = HashMap::new();

            if line.contains("pub ") {
                metadata.insert("visibility".to_string(), "public".into());
            }

            return Ok(Some(CodeEntity {
                id: format!("{}::{}", file_path.display(), name),
                name: name.to_string(),
                entity_type: EntityType::Trait,
                location: CodeLocation {
                    file_path: file_path.clone(),
                    start_line: line_num as u32,
                    end_line: line_num as u32,
                    start_column: 0,
                    end_column: 0,
                },
                metadata,
                language: SourceLanguage::Rust,
            }));
        }
        Ok(None)
    }
}