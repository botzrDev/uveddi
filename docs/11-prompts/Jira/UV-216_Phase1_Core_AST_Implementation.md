# UV-216 Phase 1: Core AST Implementation - Senior Developer Prompt

## 🎯 **Project Intelligence Officer (PIO) Activation**

You are a **Senior Rust Developer** tasked with implementing the core AST analysis foundation for the Long Methods Detector. This is **Phase 1** of UV-216, focusing on establishing robust Tree-sitter integration and basic method detection infrastructure.

## 📋 **Issue Context**

**Jira Issue**: UV-216 - "Enhance Long Methods Detector Implementation"  
**Phase**: 1 of 4 - Core AST Implementation  
**Epic**: UV-211  
**Status**: Dev & Test  
**Priority**: P2 - Medium  
**Story Points**: 2 (of 5 total)  

### **Phase 1 Scope**
Establish the foundational AST analysis infrastructure using Tree-sitter to enable accurate method detection and length calculation across multiple programming languages.

### **Current State Analysis**
- **Existing File**: `src/analysis/detectors/anti_patterns/long_methods.rs` - incomplete implementation
- **Dependencies**: Tree-sitter (UV-212) - should be enabled
- **Test Status**: All tests failing due to missing core logic

## 🔧 **Technical Implementation Requirements**

### **Core Objectives**
1. **Implement Tree-sitter Integration**: Establish robust AST parsing foundation
2. **Create Method Detection Framework**: Generic method identification system
3. **Implement Length Calculation**: Lines, statements, and token counting
4. **Add Configuration System**: Basic threshold management
5. **Establish Error Handling**: Graceful failure for parsing errors

### **Files to Modify**
```
src/analysis/detectors/anti_patterns/long_methods.rs  # Primary implementation
src/analysis/detectors/anti_patterns/mod.rs          # Module exports
Cargo.toml                                           # Dependencies verification
```

## 🏗️ **Implementation Strategy**

### **1. Tree-sitter Integration Foundation**

#### **1.1 Core Detector Structure**
```rust
use crate::ast::tree_sitter::{AstParser, SourceLanguage, ParsedFile};
use crate::analysis::{AnalysisDetector, AnalysisError, Issue, IssueType, Severity};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tree_sitter::{Query, QueryCursor, Node, Tree};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongMethodConfig {
    pub max_lines: usize,
    pub max_statements: usize,
    pub max_tokens: usize,
    pub language_specific_thresholds: HashMap<String, LanguageThresholds>,
    pub exclude_patterns: Vec<String>,
    pub include_comments_in_count: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageThresholds {
    pub max_lines: usize,
    pub max_statements: usize,
    pub max_tokens: usize,
}

impl Default for LongMethodConfig {
    fn default() -> Self {
        let mut language_thresholds = HashMap::new();
        
        // Rust-specific thresholds
        language_thresholds.insert("rust".to_string(), LanguageThresholds {
            max_lines: 50,
            max_statements: 30,
            max_tokens: 200,
        });
        
        // Python-specific thresholds
        language_thresholds.insert("python".to_string(), LanguageThresholds {
            max_lines: 40,
            max_statements: 25,
            max_tokens: 180,
        });
        
        // JavaScript-specific thresholds
        language_thresholds.insert("javascript".to_string(), LanguageThresholds {
            max_lines: 45,
            max_statements: 28,
            max_tokens: 190,
        });
        
        Self {
            max_lines: 50,
            max_statements: 30,
            max_tokens: 200,
            language_specific_thresholds: language_thresholds,
            exclude_patterns: vec![
                "test_*".to_string(),
                "*_test".to_string(),
                "benchmark_*".to_string(),
            ],
            include_comments_in_count: false,
        }
    }
}

pub struct LongMethodDetector {
    config: LongMethodConfig,
}

impl LongMethodDetector {
    pub fn new(config: LongMethodConfig) -> Self {
        Self { config }
    }
    
    pub fn with_default_config() -> Self {
        Self::new(LongMethodConfig::default())
    }
}
```

#### **1.2 Method Detection Framework**
```rust
#[derive(Debug, Clone)]
struct MethodInfo {
    name: String,
    start_line: usize,
    end_line: usize,
    line_count: usize,
    statement_count: usize,
    token_count: usize,
    language: String,
    file_path: String,
}

impl LongMethodDetector {
    fn detect_methods_in_file(&self, parsed_file: &ParsedFile) -> Result<Vec<MethodInfo>, AnalysisError> {
        let language = self.determine_language(&parsed_file.file_path)?;
        let query = self.create_method_query(&language)?;
        
        let mut methods = Vec::new();
        let mut cursor = QueryCursor::new();
        
        let matches = cursor.matches(&query, parsed_file.tree.root_node(), parsed_file.content.as_bytes());
        
        for match_ in matches {
            if let Some(method_info) = self.extract_method_info(match_, parsed_file, &language)? {
                methods.push(method_info);
            }
        }
        
        Ok(methods)
    }
    
    fn determine_language(&self, file_path: &str) -> Result<String, AnalysisError> {
        let extension = std::path::Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| AnalysisError::InvalidInput("Unable to determine file extension".to_string()))?;
            
        match extension {
            "rs" => Ok("rust".to_string()),
            "py" => Ok("python".to_string()),
            "js" | "ts" => Ok("javascript".to_string()),
            _ => Err(AnalysisError::UnsupportedLanguage(extension.to_string())),
        }
    }
}
```

### **2. Tree-sitter Query Implementation**

#### **2.1 Language-Specific Queries**
```rust
impl LongMethodDetector {
    fn create_method_query(&self, language: &str) -> Result<Query, AnalysisError> {
        let query_string = match language {
            "rust" => self.create_rust_method_query(),
            "python" => self.create_python_method_query(),
            "javascript" => self.create_javascript_method_query(),
            _ => return Err(AnalysisError::UnsupportedLanguage(language.to_string())),
        };
        
        let language_obj = match language {
            "rust" => tree_sitter_rust::language(),
            "python" => tree_sitter_python::language(),
            "javascript" => tree_sitter_javascript::language(),
            _ => return Err(AnalysisError::UnsupportedLanguage(language.to_string())),
        };
        
        Query::new(language_obj, &query_string)
            .map_err(|e| AnalysisError::QueryError(format!("Failed to create query: {}", e)))
    }
    
    fn create_rust_method_query(&self) -> String {
        r#"
        (function_item
          name: (identifier) @method.name
          body: (block) @method.body
        ) @method.definition
        
        (impl_item
          body: (declaration_list
            (function_item
              name: (identifier) @method.name
              body: (block) @method.body
            ) @method.definition
          )
        )
        "#.to_string()
    }
    
    fn create_python_method_query(&self) -> String {
        r#"
        (function_definition
          name: (identifier) @method.name
          body: (block) @method.body
        ) @method.definition
        
        (class_definition
          body: (block
            (function_definition
              name: (identifier) @method.name
              body: (block) @method.body
            ) @method.definition
          )
        )
        "#.to_string()
    }
    
    fn create_javascript_method_query(&self) -> String {
        r#"
        (function_declaration
          name: (identifier) @method.name
          body: (statement_block) @method.body
        ) @method.definition
        
        (method_definition
          name: (property_identifier) @method.name
          value: (function
            body: (statement_block) @method.body
          )
        ) @method.definition
        
        (arrow_function
          body: (statement_block) @method.body
        ) @method.definition
        "#.to_string()
    }
}
```

### **3. Method Analysis Implementation**

#### **3.1 Length Calculation Logic**
```rust
impl LongMethodDetector {
    fn extract_method_info(
        &self,
        match_: tree_sitter::QueryMatch,
        parsed_file: &ParsedFile,
        language: &str,
    ) -> Result<Option<MethodInfo>, AnalysisError> {
        let mut method_name = String::new();
        let mut method_body: Option<Node> = None;
        let mut method_definition: Option<Node> = None;
        
        // Extract captures from the query match
        for capture in match_.captures {
            let node = capture.node;
            let capture_name = &parsed_file.content[node.byte_range()];
            
            match capture.index {
                0 => method_name = capture_name.to_string(), // @method.name
                1 => method_body = Some(node),               // @method.body
                2 => method_definition = Some(node),         // @method.definition
                _ => {}
            }
        }
        
        if let (Some(body), Some(definition)) = (method_body, method_definition) {
            let method_info = self.calculate_method_metrics(
                method_name,
                definition,
                body,
                parsed_file,
                language,
            )?;
            
            // Apply exclusion patterns
            if self.should_exclude_method(&method_info.name) {
                return Ok(None);
            }
            
            Ok(Some(method_info))
        } else {
            Ok(None)
        }
    }
    
    fn calculate_method_metrics(
        &self,
        name: String,
        definition_node: Node,
        body_node: Node,
        parsed_file: &ParsedFile,
        language: &str,
    ) -> Result<MethodInfo, AnalysisError> {
        let start_line = definition_node.start_position().row + 1;
        let end_line = definition_node.end_position().row + 1;
        let line_count = end_line - start_line + 1;
        
        // Count statements within the method body
        let statement_count = self.count_statements(body_node, language)?;
        
        // Count tokens (non-whitespace, non-comment nodes)
        let token_count = self.count_tokens(body_node, parsed_file)?;
        
        Ok(MethodInfo {
            name,
            start_line,
            end_line,
            line_count,
            statement_count,
            token_count,
            language: language.to_string(),
            file_path: parsed_file.file_path.clone(),
        })
    }
    
    fn count_statements(&self, body_node: Node, language: &str) -> Result<usize, AnalysisError> {
        let mut count = 0;
        let mut cursor = body_node.walk();
        
        // Language-specific statement counting
        let statement_types = match language {
            "rust" => vec![
                "expression_statement",
                "let_declaration",
                "if_expression",
                "while_expression",
                "for_expression",
                "loop_expression",
                "match_expression",
                "return_expression",
            ],
            "python" => vec![
                "expression_statement",
                "assignment",
                "if_statement",
                "while_statement",
                "for_statement",
                "return_statement",
                "import_statement",
                "import_from_statement",
            ],
            "javascript" => vec![
                "expression_statement",
                "variable_declaration",
                "if_statement",
                "while_statement",
                "for_statement",
                "return_statement",
                "import_statement",
                "export_statement",
            ],
            _ => return Err(AnalysisError::UnsupportedLanguage(language.to_string())),
        };
        
        if cursor.goto_first_child() {
            loop {
                let node = cursor.node();
                if statement_types.contains(&node.kind()) {
                    count += 1;
                }
                
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        
        Ok(count)
    }
    
    fn count_tokens(&self, body_node: Node, parsed_file: &ParsedFile) -> Result<usize, AnalysisError> {
        let mut count = 0;
        let mut cursor = body_node.walk();
        
        // Traverse all nodes and count non-whitespace, non-comment tokens
        self.traverse_and_count(&mut cursor, &mut count, parsed_file);
        
        Ok(count)
    }
    
    fn traverse_and_count(&self, cursor: &mut tree_sitter::TreeCursor, count: &mut usize, parsed_file: &ParsedFile) {
        let node = cursor.node();
        
        // Skip whitespace and comments
        if !node.kind().contains("comment") && !node.kind().contains("whitespace") && node.child_count() == 0 {
            let text = &parsed_file.content[node.byte_range()];
            if !text.trim().is_empty() {
                *count += 1;
            }
        }
        
        if cursor.goto_first_child() {
            loop {
                self.traverse_and_count(cursor, count, parsed_file);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }
    }
    
    fn should_exclude_method(&self, method_name: &str) -> bool {
        self.config.exclude_patterns.iter().any(|pattern| {
            if pattern.starts_with('*') && pattern.ends_with('*') {
                let inner = &pattern[1..pattern.len()-1];
                method_name.contains(inner)
            } else if pattern.starts_with('*') {
                let suffix = &pattern[1..];
                method_name.ends_with(suffix)
            } else if pattern.ends_with('*') {
                let prefix = &pattern[..pattern.len()-1];
                method_name.starts_with(prefix)
            } else {
                method_name == pattern
            }
        })
    }
}
```

### **4. Issue Generation and Reporting**

#### **4.1 Long Method Issue Creation**
```rust
impl LongMethodDetector {
    fn create_long_method_issue(&self, method_info: &MethodInfo) -> Issue {
        let thresholds = self.get_language_thresholds(&method_info.language);
        
        let mut violations = Vec::new();
        if method_info.line_count > thresholds.max_lines {
            violations.push(format!("Lines: {} (max: {})", method_info.line_count, thresholds.max_lines));
        }
        if method_info.statement_count > thresholds.max_statements {
            violations.push(format!("Statements: {} (max: {})", method_info.statement_count, thresholds.max_statements));
        }
        if method_info.token_count > thresholds.max_tokens {
            violations.push(format!("Tokens: {} (max: {})", method_info.token_count, thresholds.max_tokens));
        }
        
        let severity = if violations.len() >= 2 {
            Severity::High
        } else {
            Severity::Medium
        };
        
        Issue {
            issue_type: IssueType::LongMethod,
            severity,
            message: format!(
                "Method '{}' is too long. {}",
                method_info.name,
                violations.join(", ")
            ),
            file_path: method_info.file_path.clone(),
            start_line: Some(method_info.start_line),
            end_line: Some(method_info.end_line),
            suggestion: Some(format!(
                "Consider breaking down '{}' into smaller, more focused methods. \
                 Each method should have a single responsibility.",
                method_info.name
            )),
            metadata: Some(serde_json::json!({
                "method_name": method_info.name,
                "line_count": method_info.line_count,
                "statement_count": method_info.statement_count,
                "token_count": method_info.token_count,
                "language": method_info.language,
                "thresholds": thresholds
            })),
        }
    }
    
    fn get_language_thresholds(&self, language: &str) -> &LanguageThresholds {
        self.config.language_specific_thresholds
            .get(language)
            .unwrap_or(&LanguageThresholds {
                max_lines: self.config.max_lines,
                max_statements: self.config.max_statements,
                max_tokens: self.config.max_tokens,
            })
    }
}
```

### **5. AnalysisDetector Implementation**

#### **5.1 Async Trait Implementation**
```rust
#[async_trait]
impl AnalysisDetector for LongMethodDetector {
    async fn detect_issues(&self, content: &str) -> Result<Vec<Issue>, AnalysisError> {
        // For single content string, create a temporary file
        let temp_file = tempfile::NamedTempFile::new()
            .map_err(|e| AnalysisError::IoError(e.to_string()))?;
        
        std::fs::write(&temp_file, content)
            .map_err(|e| AnalysisError::IoError(e.to_string()))?;
        
        let file_path = temp_file.path().to_string_lossy().to_string();
        self.detect_issues_in_file(&file_path).await
    }
    
    async fn detect_issues_in_file(&self, file_path: &str) -> Result<Vec<Issue>, AnalysisError> {
        // Parse the file using Tree-sitter
        let mut parser = AstParser::new()
            .map_err(|e| AnalysisError::ParsingError(format!("Failed to create parser: {}", e)))?;
        
        let parsed_file = parser.parse_file(std::path::Path::new(file_path))
            .map_err(|e| AnalysisError::ParsingError(format!("Failed to parse file: {}", e)))?;
        
        // Detect methods in the parsed file
        let methods = self.detect_methods_in_file(&parsed_file)?;
        
        // Filter methods that exceed thresholds and create issues
        let mut issues = Vec::new();
        for method in methods {
            if self.is_method_too_long(&method) {
                issues.push(self.create_long_method_issue(&method));
            }
        }
        
        Ok(issues)
    }
    
    fn name(&self) -> &'static str {
        "long_methods"
    }
    
    fn description(&self) -> &'static str {
        "Detects methods that are excessively long and should be refactored"
    }
}

impl LongMethodDetector {
    fn is_method_too_long(&self, method: &MethodInfo) -> bool {
        let thresholds = self.get_language_thresholds(&method.language);
        
        method.line_count > thresholds.max_lines ||
        method.statement_count > thresholds.max_statements ||
        method.token_count > thresholds.max_tokens
    }
}
```

## ✅ **Acceptance Criteria for Phase 1**

### **Must Have**
- [ ] Tree-sitter integration working for Rust, Python, JavaScript
- [ ] Basic method detection using Tree-sitter queries
- [ ] Method length calculation (lines, statements, tokens)
- [ ] Configurable thresholds per language
- [ ] Proper error handling for parsing failures
- [ ] Basic issue generation with metadata

### **Should Have**
- [ ] Exclusion patterns for test methods
- [ ] Performance optimization for large files
- [ ] Comprehensive logging for debugging
- [ ] Memory-efficient AST traversal

### **Could Have**
- [ ] Caching of parsed ASTs
- [ ] Parallel processing of multiple files
- [ ] Advanced filtering options

## 🧪 **Testing Strategy for Phase 1**

### **Unit Tests Required**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rust_method_detection() {
        // Test basic Rust function detection
    }
    
    #[test]
    fn test_python_method_detection() {
        // Test Python function/method detection
    }
    
    #[test]
    fn test_javascript_method_detection() {
        // Test JavaScript function detection
    }
    
    #[test]
    fn test_method_length_calculation() {
        // Test accurate line/statement/token counting
    }
    
    #[test]
    fn test_configurable_thresholds() {
        // Test language-specific threshold application
    }
    
    #[test]
    fn test_exclusion_patterns() {
        // Test method name exclusion logic
    }
    
    #[test]
    fn test_error_handling() {
        // Test graceful handling of parsing errors
    }
}
```

### **Integration Tests**
- Test with real code files from each language
- Verify Tree-sitter dependency integration
- Test performance with large files

## 🚀 **Implementation Timeline**

### **Day 1-2: Foundation Setup**
1. Update `long_methods.rs` with core structure
2. Implement Tree-sitter integration
3. Create basic configuration system

### **Day 3-4: Query Implementation**
1. Develop language-specific Tree-sitter queries
2. Implement method detection framework
3. Add length calculation logic

### **Day 5: Testing & Validation**
1. Create comprehensive unit tests
2. Test with sample code files
3. Verify error handling

## 🔍 **Verification Commands**

```bash
# Test the implementation
cargo test long_methods

# Test with specific languages
cargo test test_rust_method_detection
cargo test test_python_method_detection
cargo test test_javascript_method_detection

# Check Tree-sitter integration
cargo test --features tree-sitter

# Performance testing
cargo test test_large_file_performance
```

## 📖 **Reference Documentation**

- **Primary Research**: `docs/06-research/Specialized/UV-216/UV-216_Research.md`
- **Tree-sitter Documentation**: [Tree-sitter Query Syntax](https://tree-sitter.github.io/tree-sitter/using-parsers#query-syntax)
- **AST Integration**: `src/ast/tree_sitter/` modules

## 🎯 **Success Definition**

**Phase 1 is complete when:**
1. Tree-sitter integration is functional for all target languages
2. Basic method detection works accurately
3. Length calculation produces correct metrics
4. Configuration system supports language-specific thresholds
5. All unit tests pass
6. Error handling is robust and informative

**This phase establishes the foundation for all subsequent Long Methods Detector enhancements.**