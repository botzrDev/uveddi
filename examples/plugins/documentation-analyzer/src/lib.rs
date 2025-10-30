//! Code Documentation Analyzer Plugin
//! 
//! This plugin analyzes code documentation coverage, quality, and compliance
//! with documentation standards. It demonstrates file I/O operations and
//! comprehensive AST analysis for documentation extraction.

use std::collections::{HashMap, HashSet};

wit_bindgen::generate!({
    world: "core-analysis",
    path: "../../../wit/core-analysis.wit",
});

use exports::{initialize, analyze, get_info, cleanup};

static mut PLUGIN_STATE: Option<DocumentationAnalyzer> = None;

struct DocumentationAnalyzer {
    config: PluginConfig,
    documentation_stats: HashMap<String, DocumentationMetrics>,
    analysis_count: u32,
    standards: DocumentationStandards,
    language_configs: HashMap<String, LanguageDocConfig>,
}

#[derive(Debug, Clone)]
struct DocumentationMetrics {
    total_functions: u32,
    documented_functions: u32,
    total_structs: u32,
    documented_structs: u32,
    total_modules: u32,
    documented_modules: u32,
    doc_comment_lines: u32,
    total_code_lines: u32,
    documentation_coverage: f64,
    quality_score: f64,
    missing_docs: Vec<UndocumentedItem>,
}

#[derive(Debug, Clone)]
struct UndocumentedItem {
    item_type: String,
    name: String,
    line: u32,
    visibility: String,
    complexity_score: u32,
}

#[derive(Debug, Clone)]
struct DocumentationStandards {
    min_function_doc_coverage: f64,
    min_public_api_coverage: f64,
    require_examples: bool,
    require_error_docs: bool,
    require_param_docs: bool,
    require_return_docs: bool,
    max_line_length: u32,
}

#[derive(Debug, Clone)]
struct LanguageDocConfig {
    doc_comment_patterns: Vec<String>,
    function_patterns: Vec<String>,
    struct_patterns: Vec<String>,
    module_patterns: Vec<String>,
    public_visibility_patterns: Vec<String>,
}

impl DocumentationAnalyzer {
    fn new() -> Self {
        Self {
            config: PluginConfig {
                severity_threshold: SeverityLevel::Low,
                max_issues_per_file: 100,
                include_patterns: vec![
                    "*.rs".to_string(),
                    "*.js".to_string(), 
                    "*.ts".to_string(),
                    "*.py".to_string(),
                    "*.java".to_string(),
                    "*.go".to_string(),
                    "*.md".to_string(),
                    "README*".to_string(),
                ],
                exclude_patterns: vec![
                    "*.test.*".to_string(),
                    "*.spec.*".to_string(),
                    "test_*".to_string(),
                    "*_test.*".to_string(),
                ],
                rule_overrides: vec![],
                custom_settings: vec![
                    ("min_coverage".to_string(), "80.0".to_string()),
                    ("require_public_docs".to_string(), "true".to_string()),
                    ("check_examples".to_string(), "true".to_string()),
                ],
            },
            documentation_stats: HashMap::new(),
            analysis_count: 0,
            standards: DocumentationStandards {
                min_function_doc_coverage: 80.0,
                min_public_api_coverage: 95.0,
                require_examples: true,
                require_error_docs: true,
                require_param_docs: true,
                require_return_docs: true,
                max_line_length: 80,
            },
            language_configs: Self::initialize_language_configs(),
        }
    }

    fn initialize_language_configs() -> HashMap<String, LanguageDocConfig> {
        let mut configs = HashMap::new();

        // Rust configuration
        configs.insert("rust".to_string(), LanguageDocConfig {
            doc_comment_patterns: vec!["///".to_string(), "//!".to_string(), "/**".to_string()],
            function_patterns: vec!["fn ".to_string()],
            struct_patterns: vec!["struct ".to_string(), "enum ".to_string(), "trait ".to_string()],
            module_patterns: vec!["mod ".to_string()],
            public_visibility_patterns: vec!["pub fn".to_string(), "pub struct".to_string(), "pub enum".to_string()],
        });

        // JavaScript/TypeScript configuration
        configs.insert("javascript".to_string(), LanguageDocConfig {
            doc_comment_patterns: vec!["/**".to_string()],
            function_patterns: vec!["function ".to_string(), "const ".to_string(), "let ".to_string(), "=> ".to_string()],
            struct_patterns: vec!["class ".to_string(), "interface ".to_string()],
            module_patterns: vec!["export ".to_string(), "module.exports".to_string()],
            public_visibility_patterns: vec!["export function".to_string(), "export class".to_string()],
        });

        // Python configuration
        configs.insert("python".to_string(), LanguageDocConfig {
            doc_comment_patterns: vec!["\"\"\"".to_string(), "'''".to_string()],
            function_patterns: vec!["def ".to_string()],
            struct_patterns: vec!["class ".to_string()],
            module_patterns: vec!["# ".to_string()],
            public_visibility_patterns: vec!["def ".to_string(), "class ".to_string()], // Python doesn't have explicit public
        });

        configs
    }

    fn analyze_file(&mut self, file: SourceFile) -> Result<AnalysisResult, String> {
        self.analysis_count += 1;
        
        let start_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u32;

        log(LogLevel::Info, &format!("Analyzing documentation in {}", file.path));

        let mut issues = Vec::new();
        
        // Skip documentation files themselves
        if self.is_documentation_file(&file.path) {
            return self.analyze_documentation_file(file);
        }

        // Get language-specific configuration
        let lang_config = self.language_configs.get(&file.language).cloned()
            .unwrap_or_else(|| self.get_default_language_config());

        // Analyze code documentation
        let doc_metrics = self.analyze_code_documentation(&file, &lang_config)?;
        
        // Generate issues based on documentation coverage
        let coverage_issues = self.generate_coverage_issues(&doc_metrics, &file);
        issues.extend(coverage_issues);

        // Analyze documentation quality
        let quality_issues = self.analyze_documentation_quality(&file, &lang_config)?;
        issues.extend(quality_issues);

        // Check for missing README or project documentation
        if file.path.ends_with("main.rs") || file.path.ends_with("index.js") || file.path.ends_with("__init__.py") {
            if let Some(readme_issue) = self.check_readme_exists(&file.path)? {
                issues.push(readme_issue);
            }
        }

        // Store metrics
        self.documentation_stats.insert(file.path.clone(), doc_metrics.clone());

        // Calculate aggregate metrics
        let overall_coverage = doc_metrics.documentation_coverage;
        let quality_score = doc_metrics.quality_score;
        let undocumented_public_items = doc_metrics.missing_docs.iter()
            .filter(|item| item.visibility == "public")
            .count() as f64;

        let metrics = Metrics {
            lines_of_code: file.content.lines().count() as u32,
            lines_of_comments: doc_metrics.doc_comment_lines,
            complexity: 1,
            maintainability_index: 100.0,
            technical_debt_minutes: self.calculate_documentation_debt(&issues),
            custom_metrics: vec![
                ("documentation_coverage".to_string(), overall_coverage),
                ("documentation_quality_score".to_string(), quality_score),
                ("total_functions".to_string(), doc_metrics.total_functions as f64),
                ("documented_functions".to_string(), doc_metrics.documented_functions as f64),
                ("undocumented_public_items".to_string(), undocumented_public_items),
                ("doc_comment_ratio".to_string(), 
                 doc_metrics.doc_comment_lines as f64 / doc_metrics.total_code_lines.max(1) as f64),
            ],
        };

        let end_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u32;

        Ok(AnalysisResult {
            issues,
            metrics,
            dependencies: vec![],
            exports: vec![], // Could extract documented exports
            duration_ms: end_time - start_time,
            plugin_version: "1.0.0".to_string(),
        })
    }

    fn is_documentation_file(&self, path: &str) -> bool {
        let path_lower = path.to_lowercase();
        path_lower.ends_with(".md") || 
        path_lower.contains("readme") ||
        path_lower.contains("doc") ||
        path_lower.ends_with(".rst") ||
        path_lower.ends_with(".adoc")
    }

    fn analyze_documentation_file(&self, file: SourceFile) -> Result<AnalysisResult, String> {
        let mut issues = Vec::new();
        
        // Check documentation file quality
        let content_lines: Vec<&str> = file.content.lines().collect();
        
        // Check for common documentation issues
        if content_lines.len() < 10 {
            issues.push(Issue {
                id: "DOC007".to_string(),
                severity: SeverityLevel::Low,
                category: IssueCategory::Documentation,
                message: "Documentation file is very short".to_string(),
                description: Some("Documentation files should provide comprehensive information about the project or component.".to_string()),
                file: file.path.clone(),
                span: Span {
                    start: Position { line: 1, column: 1, byte_offset: 0 },
                    end: Position { line: content_lines.len() as u32, column: 1, byte_offset: 0 },
                },
                rule_id: Some("short_documentation".to_string()),
                suggestion: Some("Expand the documentation with more details, examples, and usage instructions".to_string()),
                fix: None,
                metadata: vec![
                    ("line_count".to_string(), content_lines.len().to_string()),
                    ("file_type".to_string(), "documentation".to_string()),
                ],
            });
        }

        // Check for broken links or missing sections
        let has_examples = content_lines.iter().any(|line| line.to_lowercase().contains("example"));
        let has_installation = content_lines.iter().any(|line| line.to_lowercase().contains("install"));
        
        if file.path.to_lowercase().contains("readme") && !has_installation {
            issues.push(Issue {
                id: "DOC008".to_string(),
                severity: SeverityLevel::Low,
                category: IssueCategory::Documentation,
                message: "README missing installation section".to_string(),
                description: Some("README files should include installation or setup instructions.".to_string()),
                file: file.path.clone(),
                span: Span {
                    start: Position { line: 1, column: 1, byte_offset: 0 },
                    end: Position { line: 1, column: 1, byte_offset: 0 },
                },
                rule_id: Some("missing_installation_section".to_string()),
                suggestion: Some("Add an installation or setup section to the README".to_string()),
                fix: None,
                metadata: vec![],
            });
        }

        Ok(AnalysisResult {
            issues,
            metrics: Metrics {
                lines_of_code: content_lines.len() as u32,
                lines_of_comments: content_lines.len() as u32, // All lines are "documentation"
                complexity: 1,
                maintainability_index: 100.0,
                technical_debt_minutes: issues.len() as u32 * 15,
                custom_metrics: vec![
                    ("has_examples".to_string(), if has_examples { 1.0 } else { 0.0 }),
                    ("has_installation".to_string(), if has_installation { 1.0 } else { 0.0 }),
                ],
            },
            dependencies: vec![],
            exports: vec![],
            duration_ms: 0,
            plugin_version: "1.0.0".to_string(),
        })
    }

    fn analyze_code_documentation(&self, file: &SourceFile, lang_config: &LanguageDocConfig) -> Result<DocumentationMetrics, String> {
        let lines: Vec<&str> = file.content.lines().collect();
        let mut metrics = DocumentationMetrics {
            total_functions: 0,
            documented_functions: 0,
            total_structs: 0,
            documented_structs: 0,
            total_modules: 0,
            documented_modules: 0,
            doc_comment_lines: 0,
            total_code_lines: lines.len() as u32,
            documentation_coverage: 0.0,
            quality_score: 0.0,
            missing_docs: vec![],
        };

        // Count documentation comment lines
        for line in &lines {
            let line_trimmed = line.trim();
            for pattern in &lang_config.doc_comment_patterns {
                if line_trimmed.starts_with(pattern) {
                    metrics.doc_comment_lines += 1;
                    break;
                }
            }
        }

        // Analyze with AST if available
        if let Some(ast) = &file.ast {
            self.analyze_ast_documentation(ast, &mut metrics, lang_config)?;
        } else {
            // Fallback to text-based analysis
            self.analyze_text_documentation(&lines, &mut metrics, lang_config)?;
        }

        // Calculate coverage and quality scores
        let total_items = metrics.total_functions + metrics.total_structs + metrics.total_modules;
        let documented_items = metrics.documented_functions + metrics.documented_structs + metrics.documented_modules;
        
        if total_items > 0 {
            metrics.documentation_coverage = (documented_items as f64 / total_items as f64) * 100.0;
        }

        metrics.quality_score = self.calculate_quality_score(&metrics, &lines);

        Ok(metrics)
    }

    fn analyze_ast_documentation(&self, ast: &AstNode, metrics: &mut DocumentationMetrics, lang_config: &LanguageDocConfig) -> Result<(), String> {
        // Query for functions
        let function_queries = vec![
            "(function_item) @function",
            "(function_declaration) @function", 
            "(function_definition) @function",
            "(method_definition) @function",
        ];

        for query in function_queries {
            if let Ok(functions) = query_ast(ast.clone(), query) {
                for func_node in functions {
                    metrics.total_functions += 1;
                    let func_name = self.extract_item_name(&func_node, "function");
                    
                    // Check if function has documentation
                    if self.has_documentation(&func_node, lang_config) {
                        metrics.documented_functions += 1;
                    } else {
                        let visibility = self.determine_visibility(&func_node, lang_config);
                        metrics.missing_docs.push(UndocumentedItem {
                            item_type: "function".to_string(),
                            name: func_name,
                            line: func_node.span.start.line,
                            visibility,
                            complexity_score: self.estimate_complexity(&func_node),
                        });
                    }
                }
            }
        }

        // Query for structs/classes
        let struct_queries = vec![
            "(struct_item) @struct",
            "(enum_item) @struct",
            "(trait_item) @struct",
            "(class_declaration) @struct",
            "(interface_declaration) @struct",
        ];

        for query in struct_queries {
            if let Ok(structs) = query_ast(ast.clone(), query) {
                for struct_node in structs {
                    metrics.total_structs += 1;
                    let struct_name = self.extract_item_name(&struct_node, "struct");
                    
                    if self.has_documentation(&struct_node, lang_config) {
                        metrics.documented_structs += 1;
                    } else {
                        let visibility = self.determine_visibility(&struct_node, lang_config);
                        metrics.missing_docs.push(UndocumentedItem {
                            item_type: "struct".to_string(),
                            name: struct_name,
                            line: struct_node.span.start.line,
                            visibility,
                            complexity_score: self.estimate_complexity(&struct_node),
                        });
                    }
                }
            }
        }

        // Query for modules
        if let Ok(modules) = query_ast(ast.clone(), "(mod_item) @module") {
            for mod_node in modules {
                metrics.total_modules += 1;
                let mod_name = self.extract_item_name(&mod_node, "module");
                
                if self.has_documentation(&mod_node, lang_config) {
                    metrics.documented_modules += 1;
                } else {
                    let visibility = self.determine_visibility(&mod_node, lang_config);
                    metrics.missing_docs.push(UndocumentedItem {
                        item_type: "module".to_string(),
                        name: mod_name,
                        line: mod_node.span.start.line,
                        visibility,
                        complexity_score: 1,
                    });
                }
            }
        }

        Ok(())
    }

    fn analyze_text_documentation(&self, lines: &[&str], metrics: &mut DocumentationMetrics, lang_config: &LanguageDocConfig) -> Result<(), String> {
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();
            
            // Check for functions
            for pattern in &lang_config.function_patterns {
                if line.contains(pattern) {
                    metrics.total_functions += 1;
                    
                    // Look for documentation above this line
                    let has_doc = self.has_preceding_documentation(lines, i, lang_config);
                    if has_doc {
                        metrics.documented_functions += 1;
                    } else {
                        let func_name = self.extract_function_name_from_line(line);
                        let visibility = if lang_config.public_visibility_patterns.iter()
                            .any(|p| line.contains(p)) { "public" } else { "private" };
                            
                        metrics.missing_docs.push(UndocumentedItem {
                            item_type: "function".to_string(),
                            name: func_name,
                            line: (i + 1) as u32,
                            visibility: visibility.to_string(),
                            complexity_score: 1,
                        });
                    }
                    break;
                }
            }
            
            // Check for structs/classes
            for pattern in &lang_config.struct_patterns {
                if line.contains(pattern) {
                    metrics.total_structs += 1;
                    
                    let has_doc = self.has_preceding_documentation(lines, i, lang_config);
                    if has_doc {
                        metrics.documented_structs += 1;
                    } else {
                        let struct_name = self.extract_struct_name_from_line(line);
                        let visibility = if lang_config.public_visibility_patterns.iter()
                            .any(|p| line.contains(p)) { "public" } else { "private" };
                            
                        metrics.missing_docs.push(UndocumentedItem {
                            item_type: "struct".to_string(),
                            name: struct_name,
                            line: (i + 1) as u32,
                            visibility: visibility.to_string(),
                            complexity_score: 1,
                        });
                    }
                    break;
                }
            }
            
            i += 1;
        }
        
        Ok(())
    }

    fn has_preceding_documentation(&self, lines: &[&str], current_line: usize, lang_config: &LanguageDocConfig) -> bool {
        // Look at the few lines before the current line for documentation
        let start = if current_line >= 5 { current_line - 5 } else { 0 };
        
        for i in start..current_line {
            let line = lines[i].trim();
            for pattern in &lang_config.doc_comment_patterns {
                if line.starts_with(pattern) {
                    return true;
                }
            }
        }
        false
    }

    fn has_documentation(&self, node: &AstNode, lang_config: &LanguageDocConfig) -> bool {
        // Check if the node content or surrounding context contains documentation patterns
        for pattern in &lang_config.doc_comment_patterns {
            if node.content.contains(pattern) {
                return true;
            }
        }
        false
    }

    fn extract_item_name(&self, node: &AstNode, item_type: &str) -> String {
        // Extract name from AST node based on type
        match item_type {
            "function" => self.extract_function_name_from_node(node),
            "struct" => self.extract_struct_name_from_node(node),
            "module" => self.extract_module_name_from_node(node),
            _ => "unknown".to_string(),
        }
    }

    fn extract_function_name_from_line(&self, line: &str) -> String {
        // Extract function name from line (language-agnostic)
        if let Some(fn_pos) = line.find("fn ") {
            if let Some(paren_pos) = line[fn_pos + 3..].find("(") {
                return line[fn_pos + 3..fn_pos + 3 + paren_pos].trim().to_string();
            }
        } else if let Some(func_pos) = line.find("function ") {
            if let Some(paren_pos) = line[func_pos + 9..].find("(") {
                return line[func_pos + 9..func_pos + 9 + paren_pos].trim().to_string();
            }
        } else if let Some(def_pos) = line.find("def ") {
            if let Some(paren_pos) = line[def_pos + 4..].find("(") {
                return line[def_pos + 4..def_pos + 4 + paren_pos].trim().to_string();
            }
        }
        "anonymous".to_string()
    }

    fn extract_function_name_from_node(&self, node: &AstNode) -> String {
        // Simplified name extraction from AST node
        if node.content.contains("fn ") {
            self.extract_function_name_from_line(&node.content)
        } else {
            "function".to_string()
        }
    }

    fn extract_struct_name_from_line(&self, line: &str) -> String {
        if let Some(struct_pos) = line.find("struct ") {
            if let Some(space_pos) = line[struct_pos + 7..].find(" ") {
                return line[struct_pos + 7..struct_pos + 7 + space_pos].trim().to_string();
            }
        } else if let Some(class_pos) = line.find("class ") {
            if let Some(space_pos) = line[class_pos + 6..].find(" ") {
                return line[class_pos + 6..class_pos + 6 + space_pos].trim().to_string();
            }
        }
        "unnamed".to_string()
    }

    fn extract_struct_name_from_node(&self, node: &AstNode) -> String {
        self.extract_struct_name_from_line(&node.content)
    }

    fn extract_module_name_from_node(&self, node: &AstNode) -> String {
        if let Some(mod_pos) = node.content.find("mod ") {
            if let Some(space_pos) = node.content[mod_pos + 4..].find(" ") {
                return node.content[mod_pos + 4..mod_pos + 4 + space_pos].trim().to_string();
            }
        }
        "module".to_string()
    }

    fn determine_visibility(&self, node: &AstNode, lang_config: &LanguageDocConfig) -> String {
        for pattern in &lang_config.public_visibility_patterns {
            if node.content.contains(pattern) {
                return "public".to_string();
            }
        }
        "private".to_string()
    }

    fn estimate_complexity(&self, node: &AstNode) -> u32 {
        // Simple complexity estimation based on node content
        let decision_points = node.content.matches("if ").count() +
                             node.content.matches("while ").count() +
                             node.content.matches("for ").count() +
                             node.content.matches("match ").count();
        (decision_points as u32).max(1)
    }

    fn calculate_quality_score(&self, metrics: &DocumentationMetrics, lines: &[&str]) -> f64 {
        let mut quality_score = 0.0;
        let mut total_checks = 0;

        // Documentation density check
        if metrics.total_code_lines > 0 {
            let doc_density = metrics.doc_comment_lines as f64 / metrics.total_code_lines as f64;
            quality_score += if doc_density > 0.2 { 25.0 } else { doc_density * 125.0 };
        }
        total_checks += 1;

        // Coverage check
        quality_score += (metrics.documentation_coverage / 100.0) * 25.0;
        total_checks += 1;

        // Check for examples in documentation
        let has_examples = lines.iter().any(|line| 
            line.to_lowercase().contains("example") || 
            line.contains("```") ||
            line.contains("# Example")
        );
        if has_examples {
            quality_score += 25.0;
        }
        total_checks += 1;

        // Check for comprehensive documentation (parameters, returns, errors)
        let has_params = lines.iter().any(|line| line.contains("@param") || line.contains("# Arguments"));
        let has_returns = lines.iter().any(|line| line.contains("@returns") || line.contains("@return"));
        if has_params && has_returns {
            quality_score += 25.0;
        }
        total_checks += 1;

        if total_checks > 0 {
            quality_score / total_checks as f64
        } else {
            0.0
        }
    }

    fn generate_coverage_issues(&self, metrics: &DocumentationMetrics, file: &SourceFile) -> Vec<Issue> {
        let mut issues = Vec::new();

        // Overall coverage issue
        if metrics.documentation_coverage < self.standards.min_function_doc_coverage {
            issues.push(Issue {
                id: "DOC001".to_string(),
                severity: SeverityLevel::Medium,
                category: IssueCategory::Documentation,
                message: format!("Low documentation coverage: {:.1}%", metrics.documentation_coverage),
                description: Some(format!(
                    "Documentation coverage is {:.1}%, which is below the minimum threshold of {:.1}%. Consider documenting more functions, structs, and modules.",
                    metrics.documentation_coverage, self.standards.min_function_doc_coverage
                )),
                file: file.path.clone(),
                span: Span {
                    start: Position { line: 1, column: 1, byte_offset: 0 },
                    end: Position { line: 1, column: 1, byte_offset: 0 },
                },
                rule_id: Some("low_documentation_coverage".to_string()),
                suggestion: Some("Add documentation comments to functions, structs, and modules".to_string()),
                fix: None,
                metadata: vec![
                    ("coverage_percentage".to_string(), metrics.documentation_coverage.to_string()),
                    ("threshold".to_string(), self.standards.min_function_doc_coverage.to_string()),
                    ("missing_items".to_string(), metrics.missing_docs.len().to_string()),
                ],
            });
        }

        // Individual missing documentation issues
        for missing in &metrics.missing_docs {
            let severity = if missing.visibility == "public" {
                SeverityLevel::Medium
            } else if missing.complexity_score > 5 {
                SeverityLevel::Low
            } else {
                SeverityLevel::Info
            };

            issues.push(Issue {
                id: "DOC002".to_string(),
                severity,
                category: IssueCategory::Documentation,
                message: format!("Missing documentation for {} '{}'", missing.item_type, missing.name),
                description: Some(format!(
                    "The {} '{}' lacks documentation. {} items should be documented to improve code maintainability.",
                    missing.item_type, missing.name,
                    if missing.visibility == "public" { "Public" } else { "Complex" }
                )),
                file: file.path.clone(),
                span: Span {
                    start: Position { line: missing.line, column: 1, byte_offset: 0 },
                    end: Position { line: missing.line, column: 1, byte_offset: 0 },
                },
                rule_id: Some("missing_documentation".to_string()),
                suggestion: Some(format!("Add a documentation comment above the {} declaration", missing.item_type)),
                fix: None,
                metadata: vec![
                    ("item_type".to_string(), missing.item_type.clone()),
                    ("item_name".to_string(), missing.name.clone()),
                    ("visibility".to_string(), missing.visibility.clone()),
                    ("complexity".to_string(), missing.complexity_score.to_string()),
                ],
            });
        }

        issues
    }

    fn analyze_documentation_quality(&self, file: &SourceFile, lang_config: &LanguageDocConfig) -> Result<Vec<Issue>, String> {
        let mut issues = Vec::new();
        let lines: Vec<&str> = file.content.lines().collect();

        // Check for overly long documentation lines
        for (line_num, line) in lines.iter().enumerate() {
            let line_num = (line_num + 1) as u32;
            
            // Check if this is a documentation line
            let is_doc_line = lang_config.doc_comment_patterns.iter()
                .any(|pattern| line.trim().starts_with(pattern));
                
            if is_doc_line && line.len() > self.standards.max_line_length as usize {
                issues.push(Issue {
                    id: "DOC003".to_string(),
                    severity: SeverityLevel::Low,
                    category: IssueCategory::Style,
                    message: format!("Documentation line too long: {} characters", line.len()),
                    description: Some(format!(
                        "Documentation line exceeds the maximum length of {} characters. Consider breaking it into multiple lines.",
                        self.standards.max_line_length
                    )),
                    file: file.path.clone(),
                    span: Span {
                        start: Position { line: line_num, column: 1, byte_offset: 0 },
                        end: Position { line: line_num, column: line.len() as u32, byte_offset: 0 },
                    },
                    rule_id: Some("documentation_line_too_long".to_string()),
                    suggestion: Some("Break long documentation lines into multiple lines".to_string()),
                    fix: None,
                    metadata: vec![
                        ("line_length".to_string(), line.len().to_string()),
                        ("max_length".to_string(), self.standards.max_line_length.to_string()),
                    ],
                });
            }
        }

        Ok(issues)
    }

    fn check_readme_exists(&self, main_file_path: &str) -> Result<Option<Issue>, String> {
        // Extract directory from main file path
        let dir_path = if let Some(parent) = std::path::Path::new(main_file_path).parent() {
            parent.to_string_lossy().to_string()
        } else {
            ".".to_string()
        };

        // Check for README files using file_exists host function
        let readme_variants = vec![
            "README.md",
            "readme.md", 
            "README.txt",
            "README.rst",
            "README",
        ];

        for readme in readme_variants {
            let readme_path = format!("{}/{}", dir_path, readme);
            if file_exists(&readme_path) {
                return Ok(None); // README exists
            }
        }

        // No README found
        Ok(Some(Issue {
            id: "DOC006".to_string(),
            severity: SeverityLevel::Low,
            category: IssueCategory::Documentation,
            message: "Missing README file".to_string(),
            description: Some("Project lacks a README file. A README provides important information about the project, installation, and usage.".to_string()),
            file: main_file_path.to_string(),
            span: Span {
                start: Position { line: 1, column: 1, byte_offset: 0 },
                end: Position { line: 1, column: 1, byte_offset: 0 },
            },
            rule_id: Some("missing_readme".to_string()),
            suggestion: Some("Create a README.md file with project description, installation, and usage instructions".to_string()),
            fix: None,
            metadata: vec![
                ("project_directory".to_string(), dir_path),
            ],
        }))
    }

    fn get_default_language_config(&self) -> LanguageDocConfig {
        LanguageDocConfig {
            doc_comment_patterns: vec!["//".to_string(), "#".to_string(), "/*".to_string()],
            function_patterns: vec!["function".to_string(), "def".to_string(), "fn".to_string()],
            struct_patterns: vec!["class".to_string(), "struct".to_string(), "interface".to_string()],
            module_patterns: vec!["module".to_string(), "package".to_string()],
            public_visibility_patterns: vec!["public".to_string(), "export".to_string()],
        }
    }

    fn calculate_documentation_debt(&self, issues: &[Issue]) -> u32 {
        issues.iter().map(|issue| {
            match issue.id.as_str() {
                "DOC001" => 120, // Low coverage = 2 hours
                "DOC002" => match issue.severity {
                    SeverityLevel::Medium => 30, // Public undocumented = 30 minutes
                    SeverityLevel::Low => 15,    // Private undocumented = 15 minutes
                    _ => 5,                       // Info = 5 minutes
                },
                "DOC003" => 5,   // Long line = 5 minutes
                "DOC006" => 60,  // Missing README = 1 hour
                "DOC007" => 30,  // Short documentation = 30 minutes
                "DOC008" => 45,  // Missing installation = 45 minutes
                _ => 10,
            }
        }).sum()
    }
}

// Plugin lifecycle implementation
impl initialize {
    fn call(config: PluginConfig, _limits: ResourceLimits) -> Result<(), String> {
        log(LogLevel::Info, "Initializing Documentation Analyzer Plugin");
        
        unsafe {
            let mut analyzer = DocumentationAnalyzer::new();
            analyzer.config = config;
            
            // Apply custom settings
            if let Some(min_coverage) = analyzer.config.custom_settings.iter()
                .find(|(k, _)| k == "min_coverage")
                .and_then(|(_, v)| v.parse().ok())
            {
                analyzer.standards.min_function_doc_coverage = min_coverage;
            }
            
            PLUGIN_STATE = Some(analyzer);
        }
        
        log(LogLevel::Info, "Documentation Analyzer Plugin initialized successfully");
        Ok(())
    }
}

impl analyze {
    fn call(file: SourceFile) -> Result<AnalysisResult, String> {
        unsafe {
            if let Some(ref mut state) = PLUGIN_STATE {
                state.analyze_file(file)
            } else {
                Err("Plugin not initialized".to_string())
            }
        }
    }
}

impl get_info {
    fn call() -> PluginInfo {
        PluginInfo {
            id: "documentation-analyzer".to_string(),
            name: "Code Documentation Analyzer".to_string(),
            version: "1.0.0".to_string(),
            description: "Analyzes code documentation coverage, quality, and compliance with documentation standards".to_string(),
            author: "Uveddi Team".to_string(),
            license: "MIT".to_string(),
            homepage: Some("https://github.com/uveddi/plugins/documentation-analyzer".to_string()),
            supported_languages: vec![
                "rust".to_string(),
                "javascript".to_string(),
                "typescript".to_string(),
                "python".to_string(),
                "java".to_string(),
                "go".to_string(),
                "markdown".to_string(),
            ],
            detector_types: vec![IssueCategory::Documentation, IssueCategory::Style, IssueCategory::Maintainability],
            api_version: "1.0".to_string(),
            required_permissions: vec![Permission::ReadFiles.into()],
        }
    }
}

impl cleanup {
    fn call() -> Result<(), String> {
        log(LogLevel::Info, "Cleaning up Documentation Analyzer Plugin");
        
        unsafe {
            if let Some(ref state) = PLUGIN_STATE {
                log(LogLevel::Info, &format!("Analyzed documentation in {} files during session", state.analysis_count));
                log(LogLevel::Info, &format!("Tracked documentation metrics for {} files", state.documentation_stats.len()));
            }
            PLUGIN_STATE = None;
        }
        
        Ok(())
    }
}