//! Data plane implementation using Apache Arrow for zero-copy AST transfer

#[cfg(feature = "wasm-plugins")]
use std::{collections::HashMap, io::Cursor, sync::Arc};

use crate::{
    ast::tree_sitter_impl::ParsedFile,
    plugins::{errors::*, types::AstHandle},
};
use serde::{Deserialize, Serialize};
#[cfg(feature = "tree-sitter")]
use tree_sitter;

#[cfg(not(feature = "wasm-plugins"))]
use std::collections::HashMap;

/// Data plane for efficient AST serialization (simplified without Arrow for now)
#[derive(Debug, Clone)]
pub struct AstDataPlane {
    #[cfg(feature = "wasm-plugins")]
    _dummy: u8,
}

impl AstDataPlane {
    /// Create a new AST data plane
    pub fn new() -> Result<Self, DataPlaneError> {
        #[cfg(feature = "wasm-plugins")]
        {
            Ok(Self { _dummy: 0 })
        }

        #[cfg(not(feature = "wasm-plugins"))]
        Ok(Self {})
    }

    /// Serialize a parsed file's AST (simplified JSON format for now)
    #[cfg(feature = "wasm-plugins")]
    pub fn serialize_ast(&self, parsed_file: &ParsedFile) -> Result<Vec<u8>, DataPlaneError> {
        let nodes = self.extract_ast_nodes(parsed_file)?;
        let json = serde_json::to_vec(&nodes).map_err(|e| {
            DataPlaneError::InvalidFormat(format!("JSON serialization failed: {}", e))
        })?;
        Ok(json)
    }

    #[cfg(not(feature = "wasm-plugins"))]
    pub fn serialize_ast(&self, _parsed_file: &ParsedFile) -> Result<Vec<u8>, DataPlaneError> {
        Err(DataPlaneError::InvalidFormat(
            "WASM plugins not enabled".to_string(),
        ))
    }

    /// Deserialize AST data from JSON format for validation
    #[cfg(feature = "wasm-plugins")]
    pub fn deserialize_ast(&self, buffer: &[u8]) -> Result<AstInfo, DataPlaneError> {
        let nodes: Vec<AstNodeData> = serde_json::from_slice(buffer).map_err(|e| {
            DataPlaneError::InvalidFormat(format!("JSON deserialization failed: {}", e))
        })?;

        let total_nodes = nodes.len();
        let mut languages = std::collections::HashSet::new();
        let mut files = std::collections::HashSet::new();

        for node in &nodes {
            languages.insert(node.language.clone());
            files.insert(node.file_path.clone());
        }

        Ok(AstInfo {
            total_nodes,
            languages: languages.into_iter().collect(),
            files: files.into_iter().collect(),
            size_bytes: buffer.len() as u64,
        })
    }

    #[cfg(not(feature = "wasm-plugins"))]
    pub fn deserialize_ast(&self, _buffer: &[u8]) -> Result<AstInfo, DataPlaneError> {
        Err(DataPlaneError::InvalidFormat(
            "WASM plugins not enabled".to_string(),
        ))
    }

    /// Extract AST nodes from a parsed file
    #[cfg(feature = "wasm-plugins")]
    fn extract_ast_nodes(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<AstNodeData>, DataPlaneError> {
        let mut nodes = Vec::new();
        let mut node_id = 0u64;

        if let Some(ref tree) = parsed_file.tree {
            let root_node = tree.root_node();
            let source_bytes = parsed_file.source.as_bytes();

            fn visit_tree_sitter_node(
                node: tree_sitter::Node,
                parent_id: Option<u64>,
                nodes: &mut Vec<AstNodeData>,
                node_id: &mut u64,
                file_path: &str,
                language: &str,
                depth: u32,
                source_bytes: &[u8],
            ) {
                let current_id = *node_id;
                *node_id += 1;

                let text = if node.byte_range().len() < 1000 {
                    // Limit text size
                    node.utf8_text(source_bytes).ok().map(|s| s.to_string())
                } else {
                    None
                };

                nodes.push(AstNodeData {
                    node_id: current_id,
                    parent_id,
                    node_type: node.kind().to_string(),
                    text,
                    start_line: node.start_position().row as u32,
                    end_line: node.end_position().row as u32,
                    start_column: node.start_position().column as u32,
                    end_column: node.end_position().column as u32,
                    file_path: file_path.to_string(),
                    language: language.to_string(),
                    depth,
                    is_named: node.is_named(),
                });

                let mut cursor = node.walk();
                if cursor.goto_first_child() {
                    loop {
                        visit_tree_sitter_node(
                            cursor.node(),
                            Some(current_id),
                            nodes,
                            node_id,
                            file_path,
                            language,
                            depth + 1,
                            source_bytes,
                        );

                        if !cursor.goto_next_sibling() {
                            break;
                        }
                    }
                }
            }

            visit_tree_sitter_node(
                root_node,
                None,
                &mut nodes,
                &mut node_id,
                &parsed_file.file_path.as_ref().display().to_string(),
                &format!("{:?}", parsed_file.language),
                0,
                source_bytes,
            );
        }

        Ok(nodes)
    }
}

impl Default for AstDataPlane {
    fn default() -> Self {
        Self::new().expect("Failed to create AstDataPlane")
    }
}

/// Internal representation of AST node data
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AstNodeData {
    node_id: u64,
    parent_id: Option<u64>,
    node_type: String,
    text: Option<String>,
    start_line: u32,
    end_line: u32,
    start_column: u32,
    end_column: u32,
    file_path: String,
    language: String,
    depth: u32,
    is_named: bool,
}

/// Information about deserialized AST data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstInfo {
    pub total_nodes: usize,
    pub languages: Vec<String>,
    pub files: Vec<String>,
    pub size_bytes: u64,
}

/// AST handle manager for tracking handles in plugin memory
#[derive(Debug, Default, Clone)]
pub struct AstHandleManager {
    handles: HashMap<u32, AstHandleData>,
    next_id: u32,
}

#[derive(Debug, Clone)]
struct AstHandleData {
    buffer: Vec<u8>,
    info: AstInfo,
    created_at: std::time::Instant,
}

impl AstHandleManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Store AST buffer and return a handle
    pub fn create_handle(&mut self, buffer: Vec<u8>, info: AstInfo) -> AstHandle {
        let id = self.next_id;
        self.next_id += 1;

        let language = info.languages.first().cloned().unwrap_or_default();
        let size = buffer.len() as u64;

        self.handles.insert(
            id,
            AstHandleData {
                buffer,
                info,
                created_at: std::time::Instant::now(),
            },
        );

        AstHandle { id, size, language }
    }

    /// Get AST buffer by handle
    pub fn get_buffer(&self, handle_id: u32) -> Result<&[u8], DataPlaneError> {
        self.handles
            .get(&handle_id)
            .map(|data| data.buffer.as_slice())
            .ok_or(DataPlaneError::HandleNotFound { handle_id })
    }

    /// Get AST info by handle
    pub fn get_info(&self, handle_id: u32) -> Result<&AstInfo, DataPlaneError> {
        self.handles
            .get(&handle_id)
            .map(|data| &data.info)
            .ok_or(DataPlaneError::HandleNotFound { handle_id })
    }

    /// Remove handle and free memory
    pub fn free_handle(&mut self, handle_id: u32) -> Result<(), DataPlaneError> {
        self.handles
            .remove(&handle_id)
            .map(|_| ())
            .ok_or(DataPlaneError::HandleNotFound { handle_id })
    }

    /// Get number of active handles
    pub fn handle_count(&self) -> usize {
        self.handles.len()
    }

    /// Get total memory usage of all handles
    pub fn total_memory_usage(&self) -> u64 {
        self.handles
            .values()
            .map(|data| data.buffer.len() as u64)
            .sum()
    }

    /// Clean up old handles (older than timeout)
    pub fn cleanup_old_handles(&mut self, timeout_secs: u64) {
        let timeout = std::time::Duration::from_secs(timeout_secs);
        let now = std::time::Instant::now();

        self.handles
            .retain(|_, data| now.duration_since(data.created_at) < timeout);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::tree_sitter::ParsedFile;
    use std::path::PathBuf;

    #[test]
    fn test_ast_handle_manager() {
        let mut manager = AstHandleManager::new();

        let buffer = vec![1, 2, 3, 4];
        let info = AstInfo {
            total_nodes: 10,
            languages: vec!["rust".to_string()],
            files: vec!["test.rs".to_string()],
            size_bytes: 4,
        };

        let handle = manager.create_handle(buffer.clone(), info.clone());
        assert_eq!(handle.id, 0);
        assert_eq!(handle.size, 4);
        assert_eq!(handle.language, "rust");

        assert_eq!(manager.get_buffer(handle.id).unwrap(), &buffer);
        assert_eq!(manager.handle_count(), 1);
        assert_eq!(manager.total_memory_usage(), 4);

        manager.free_handle(handle.id).unwrap();
        assert_eq!(manager.handle_count(), 0);
        assert_eq!(manager.total_memory_usage(), 0);
    }

    #[cfg(feature = "wasm-plugins")]
    #[test]
    fn test_ast_data_plane_creation() {
        let data_plane = AstDataPlane::new().unwrap();
        // Basic creation test
        assert!(data_plane._dummy == 0);
    }
}
