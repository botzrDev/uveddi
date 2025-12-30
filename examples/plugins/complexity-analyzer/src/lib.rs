//! Code Complexity Analyzer Plugin
//! 
//! This plugin analyzes code complexity using multiple metrics including
//! Cyclomatic Complexity, Halstead Metrics, and Cognitive Complexity.
//! It demonstrates sophisticated AST analysis and metrics calculation.

use std::collections::HashMap;
use std::cell::RefCell;

wit_bindgen::generate!({
    world: "core-analysis",
    path: "../../../wit/core-analysis.wit",
});

struct ComplexityAnalyzerPlugin {
    state: RefCell<Option<ComplexityAnalyzer>>,
}

impl Default for ComplexityAnalyzerPlugin {
    fn default() -> Self {
        Self {
            state: RefCell::new(None),
        }
    }
}

export!(ComplexityAnalyzerPlugin);

impl Guest for ComplexityAnalyzerPlugin {
    fn initialize(&self, config: PluginConfig, _limits: ResourceLimits) -> Result<(), String> {
        log(LogLevel::Info, "Initializing Complexity Analyzer Plugin");
        
        let mut analyzer = ComplexityAnalyzer::new();
        analyzer.config = config;
        
        // Apply custom thresholds from config
        if let Some(cyclomatic_threshold) = analyzer.config.custom_settings.iter()
            .find(|(k, _)| k == "cyclomatic_threshold")
            .and_then(|(_, v)| v.parse().ok())
        {
            analyzer.complexity_thresholds.cyclomatic_complexity = cyclomatic_threshold;
        }
        
        *self.state.borrow_mut() = Some(analyzer);
        
        log(LogLevel::Info, "Complexity Analyzer Plugin initialized successfully");
        Ok(())
    }

    fn analyze(&self, file: SourceFile) -> Result<AnalysisResult, String> {
        let mut state_guard = self.state.borrow_mut();
        if let Some(ref mut state) = *state_guard {
            state.analyze_file(file)
        } else {
            Err("Plugin not initialized".to_string())
        }
    }

    fn get_info(&self) -> PluginInfo {
        PluginInfo {
            id: "complexity-analyzer".to_string(),
            name: "Code Complexity Analyzer".to_string(),
            version: "1.0.0".to_string(),
            description: "Analyzes code complexity using multiple metrics including Cyclomatic Complexity, Halstead Metrics, and Cognitive Complexity".to_string(),
            author: "Uveddi Team".to_string(),
            license: "MIT".to_string(),
            homepage: Some("https://github.com/uveddi/plugins/complexity-analyzer".to_string()),
            supported_languages: vec![
                "rust".to_string(),
                "javascript".to_string(), 
                "typescript".to_string(),
                "python".to_string(),
                "java".to_string(),
                "go".to_string(),
            ],
            detector_types: vec![IssueCategory::Complexity, IssueCategory::Maintainability],
            api_version: "1.0".to_string(),
            required_permissions: vec![Permission::ReadFiles],
        }
    }

    fn cleanup(&self) -> Result<(), String> {
        log(LogLevel::Info, "Cleaning up Complexity Analyzer Plugin");
        
        let mut state_guard = self.state.borrow_mut();
        if let Some(ref state) = *state_guard {
            log(LogLevel::Info, &format!("Analyzed {} files for complexity during session", state.analysis_count));
            log(LogLevel::Info, &format!("Tracked complexity for {} functions", state.function_stats.len()));
        }
        *state_guard = None;
        
        Ok(())
    }
}

// Internal structures and logic (kept from original)

struct ComplexityAnalyzer {
    config: PluginConfig,
    analysis_count: u32,
    complexity_thresholds: ComplexityThresholds,
    function_stats: HashMap<String, FunctionComplexity>,
}

struct ComplexityThresholds {
    cyclomatic_complexity: u32,
    cognitive_complexity: u32,
    max_function_length: u32,
    max_parameters: u32,
    max_nesting_depth: u32,
}

#[derive(Debug, Clone)]
struct FunctionComplexity {
    name: String,
    cyclomatic_complexity: u32,
    cognitive_complexity: u32,
    length_in_lines: u32,
    parameter_count: u32,
    nesting_depth: u32,
    halstead_difficulty: f64,
    maintainability_index: f64,
}

impl ComplexityAnalyzer {
    fn new() -> Self {
        Self {
            config: PluginConfig {
                severity_threshold: SeverityLevel::Medium,
                max_issues_per_file: 50,
                include_patterns: vec!["*.rs".to_string(), "*.js".to_string(), "*.ts".to_string(), "*.py".to_string()],
                exclude_patterns: vec!["*.test.*".to_string(), "*.spec.*".to_string()],
                rule_overrides: vec![],
                custom_settings: vec![
                    ("cyclomatic_threshold".to_string(), "10".to_string()),
                    ("cognitive_threshold".to_string(), "15".to_string()),
                    ("max_function_lines".to_string(), "50".to_string()),
                ],
            },
            analysis_count: 0,
            complexity_thresholds: ComplexityThresholds {
                cyclomatic_complexity: 10,
                cognitive_complexity: 15,
                max_function_length: 50,
                max_parameters: 6,
                max_nesting_depth: 4,
            },
            function_stats: HashMap::new(),
        }
    }

    fn analyze_file(&mut self, file: SourceFile) -> Result<AnalysisResult, String> {
        self.analysis_count += 1;
        
        let start_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u32;

        log(LogLevel::Info, &format!("Analyzing complexity for {}", file.path));

        let mut issues = Vec::new();
        let mut total_complexity = 0;
        let mut function_count = 0;

        // Parse AST if not provided
        let ast = if let Some(ast) = file.ast {
            ast
        } else {
            parse_ast(&file.content, &file.language)
                .map_err(|e| format!("Failed to parse AST: {}", e))?
        };

        // Analyze functions and methods
        let function_issues = self.analyze_functions(&ast, &file)?;
        issues.extend(function_issues);

        // Analyze overall file complexity
        let file_issues = self.analyze_file_complexity(&ast, &file)?;
        issues.extend(file_issues);

        // Calculate aggregate metrics
        for func_stat in self.function_stats.values() {
            total_complexity += func_stat.cyclomatic_complexity;
            function_count += 1;
        }

        let average_complexity = if function_count > 0 {
            total_complexity as f64 / function_count as f64
        } else {
            0.0
        };

        let high_complexity_functions = self.function_stats.values()
            .filter(|f| f.cyclomatic_complexity > self.complexity_thresholds.cyclomatic_complexity)
            .count() as f64;

        let maintainability_index = self.calculate_maintainability_index(&file, &self.function_stats);
        
        let metrics = Metrics {
            lines_of_code: file.content.lines().count() as u32,
            lines_of_comments: self.count_comment_lines(&file.content),
            complexity: total_complexity,
            maintainability_index,
            technical_debt_minutes: self.calculate_complexity_debt(&issues),
            custom_metrics: vec![
                ("average_cyclomatic_complexity".to_string(), average_complexity),
                ("high_complexity_functions".to_string(), high_complexity_functions),
                ("function_count".to_string(), function_count as f64),
                ("max_nesting_depth".to_string(), self.find_max_nesting_depth(&ast) as f64),
                ("cognitive_complexity_total".to_string(), self.calculate_total_cognitive_complexity()),
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
            exports: vec![],
            duration_ms: end_time - start_time,
            plugin_version: "1.0.0".to_string(),
        })
    }

    fn analyze_functions(&mut self, ast: &AstNode, file: &SourceFile) -> Result<Vec<Issue>, String> {
        let mut issues = Vec::new();

        // Query for function definitions (works for Rust, JavaScript, Python, etc.)
        let function_queries = vec![
            "(function_item) @function",           // Rust
            "(function_declaration) @function",    // JavaScript/TypeScript  
            "(function_definition) @function",     // Python
            "(method_definition) @function",       // JavaScript/TypeScript methods
        ];

        for query in function_queries {
            if let Ok(functions) = query_ast(ast, query) {
                for func_node in functions {
                    let func_complexity = self.analyze_single_function(&func_node, file)?;
                    let func_name = self.extract_function_name(&func_node);
                    
                    self.function_stats.insert(func_name.clone(), func_complexity.clone());
                    
                    // Generate issues based on complexity thresholds
                    let func_issues = self.generate_complexity_issues(&func_complexity, file, &func_node.span);
                    issues.extend(func_issues);
                }
            }
        }

        Ok(issues)
    }

    fn analyze_single_function(&self, func_node: &AstNode, _file: &SourceFile) -> Result<FunctionComplexity, String> {
        let function_name = self.extract_function_name(func_node);
        
        // Calculate cyclomatic complexity
        let cyclomatic_complexity = self.calculate_cyclomatic_complexity(func_node)?;
        
        // Calculate cognitive complexity
        let cognitive_complexity = self.calculate_cognitive_complexity(func_node)?;
        
        // Calculate function metrics
        let length_in_lines = self.calculate_function_length(func_node);
        let parameter_count = self.count_parameters(func_node)?;
        let nesting_depth = self.calculate_nesting_depth(func_node)?;
        
        // Calculate Halstead metrics
        let halstead_difficulty = self.calculate_halstead_difficulty(func_node)?;
        
        // Calculate maintainability index for this function
        let maintainability_index = self.calculate_function_maintainability(
            cyclomatic_complexity,
            halstead_difficulty,
            length_in_lines,
        );

        Ok(FunctionComplexity {
            name: function_name,
            cyclomatic_complexity,
            cognitive_complexity,
            length_in_lines,
            parameter_count,
            nesting_depth,
            halstead_difficulty,
            maintainability_index,
        })
    }

    fn calculate_cyclomatic_complexity(&self, func_node: &AstNode) -> Result<u32, String> {
        let mut complexity = 1; // Base complexity

        // Count decision points
        let decision_queries = vec![
            "(if_statement)",
            "(while_statement)",  
            "(for_statement)",
            "(match_expression)",
            "(switch_statement)",
            "(catch_clause)",
            "(conditional_expression)",
            "(logical_expression)",
        ];

        for query in decision_queries {
            if let Ok(nodes) = query_ast(func_node, query) {
                complexity += nodes.len() as u32;
            }
        }

        // Count logical operators in conditions
        if let Ok(logical_ops) = query_ast(func_node, "(binary_expression operator: [\"&&\" \"||\"]) @op") {
            complexity += logical_ops.len() as u32;
        }

        Ok(complexity)
    }

    fn calculate_cognitive_complexity(&self, func_node: &AstNode) -> Result<u32, String> {
        let mut complexity = 0;
        let mut nesting_level = 0;

        // This is a simplified cognitive complexity calculation
        // Real implementation would need more sophisticated AST traversal
        
        // Count nesting structures with penalties
        let nesting_structures = vec![
            "(if_statement)",
            "(while_statement)",
            "(for_statement)",
            "(loop_expression)",
        ];

        for structure in nesting_structures {
            if let Ok(nodes) = query_ast(func_node, structure) {
                for _ in nodes {
                    complexity += 1 + nesting_level; // Base cost + nesting penalty
                    nesting_level += 1;
                }
            }
        }

        Ok(complexity)
    }

    fn calculate_function_length(&self, func_node: &AstNode) -> u32 {
        let start_line = func_node.span.start.line;
        let end_line = func_node.span.end.line;
        end_line - start_line + 1
    }

    fn count_parameters(&self, func_node: &AstNode) -> Result<u32, String> {
        // Query for parameter lists
        if let Ok(params) = query_ast(func_node, "(parameters (parameter)) @param") {
            Ok(params.len() as u32)
        } else {
            Ok(0)
        }
    }

    fn calculate_nesting_depth(&self, func_node: &AstNode) -> Result<u32, String> {
        // This would need a recursive traversal of the AST
        // For now, return a simplified calculation
        let mut max_depth = 0;
        
        let nested_structures = vec![
            "(if_statement (if_statement))",
            "(while_statement (if_statement))",
            "(for_statement (if_statement))",
        ];

        for structure in nested_structures {
            if let Ok(nodes) = query_ast(func_node, structure) {
                if !nodes.is_empty() {
                    max_depth = max_depth.max(2);
                }
            }
        }

        Ok(max_depth)
    }

    fn calculate_halstead_difficulty(&self, func_node: &AstNode) -> Result<f64, String> {
        // Simplified Halstead difficulty calculation
        // Real implementation would count unique operators and operands
        
        let mut operators = 0;
        let mut operands = 0;

        // Count operators (simplified)
        let operator_queries = vec![
            "(binary_expression)",
            "(unary_expression)", 
            "(assignment_expression)",
            "(call_expression)",
        ];

        for query in operator_queries {
            if let Ok(nodes) = query_ast(func_node, query) {
                operators += nodes.len();
            }
        }

        // Count operands (simplified)
        if let Ok(identifiers) = query_ast(func_node, "(identifier) @id") {
            operands = identifiers.len();
        }

        if operands == 0 {
            return Ok(0.0);
        }

        // Simplified difficulty = operators / operands
        Ok(operators as f64 / operands as f64)
    }

    fn calculate_function_maintainability(&self, cyclomatic: u32, halstead: f64, lines: u32) -> f64 {
        // Simplified maintainability index calculation
        let cyclomatic_penalty = cyclomatic as f64 * 2.0;
        let halstead_penalty = halstead * 10.0;
        let length_penalty = lines as f64 * 0.5;
        
        let base_score = 100.0;
        (base_score - cyclomatic_penalty - halstead_penalty - length_penalty).max(0.0)
    }

    fn generate_complexity_issues(
        &self,
        func_complexity: &FunctionComplexity,
        file: &SourceFile,
        span: &Span,
    ) -> Vec<Issue> {
        let mut issues = Vec::new();

        // Cyclomatic complexity issues
        if func_complexity.cyclomatic_complexity > self.complexity_thresholds.cyclomatic_complexity {
            issues.push(Issue {
                id: "COMP001".to_string(),
                severity: if func_complexity.cyclomatic_complexity > 20 {
                    SeverityLevel::High
                } else {
                    SeverityLevel::Medium
                },
                category: IssueCategory::Complexity,
                message: format!("High cyclomatic complexity: {}", func_complexity.cyclomatic_complexity),
                description: Some(format!(
                    "Function '{}' has cyclomatic complexity of {}, which exceeds the threshold of {}. Consider breaking this function into smaller, more focused functions.",
                    func_complexity.name, func_complexity.cyclomatic_complexity, self.complexity_thresholds.cyclomatic_complexity
                )),
                file: file.path.clone(),
                span: span.clone(),
                rule_id: Some("high_cyclomatic_complexity".to_string()),
                suggestion: Some("Break this function into smaller, more focused functions".to_string()),
                fix: None,
                metadata: vec![
                    ("function_name".to_string(), func_complexity.name.clone()),
                    ("complexity_value".to_string(), func_complexity.cyclomatic_complexity.to_string()),
                    ("threshold".to_string(), self.complexity_thresholds.cyclomatic_complexity.to_string()),
                ],
            });
        }

        // Cognitive complexity issues
        if func_complexity.cognitive_complexity > self.complexity_thresholds.cognitive_complexity {
            issues.push(Issue {
                id: "COMP002".to_string(),
                severity: SeverityLevel::Medium,
                category: IssueCategory::Complexity,
                message: format!("High cognitive complexity: {}", func_complexity.cognitive_complexity),
                description: Some(format!(
                    "Function '{}' has cognitive complexity of {}, making it difficult to understand and maintain.",
                    func_complexity.name, func_complexity.cognitive_complexity
                )),
                file: file.path.clone(),
                span: span.clone(),
                rule_id: Some("high_cognitive_complexity".to_string()),
                suggestion: Some("Reduce nesting and simplify control flow".to_string()),
                fix: None,
                metadata: vec![
                    ("function_name".to_string(), func_complexity.name.clone()),
                    ("cognitive_complexity".to_string(), func_complexity.cognitive_complexity.to_string()),
                ],
            });
        }

        // Function length issues
        if func_complexity.length_in_lines > self.complexity_thresholds.max_function_length {
            issues.push(Issue {
                id: "COMP003".to_string(),
                severity: SeverityLevel::Low,
                category: IssueCategory::Maintainability,
                message: format!("Long function: {} lines", func_complexity.length_in_lines),
                description: Some(format!(
                    "Function '{}' is {} lines long, which exceeds the recommended maximum of {} lines.",
                    func_complexity.name, func_complexity.length_in_lines, self.complexity_thresholds.max_function_length
                )),
                file: file.path.clone(),
                span: span.clone(),
                rule_id: Some("long_function".to_string()),
                suggestion: Some("Consider extracting parts of this function into separate methods".to_string()),
                fix: None,
                metadata: vec![
                    ("function_name".to_string(), func_complexity.name.clone()),
                    ("line_count".to_string(), func_complexity.length_in_lines.to_string()),
                ],
            });
        }

        // Too many parameters
        if func_complexity.parameter_count > self.complexity_thresholds.max_parameters {
            issues.push(Issue {
                id: "COMP004".to_string(),
                severity: SeverityLevel::Low,
                category: IssueCategory::Maintainability,
                message: format!("Too many parameters: {}", func_complexity.parameter_count),
                description: Some(format!(
                    "Function '{}' has {} parameters, which makes it hard to use and test.",
                    func_complexity.name, func_complexity.parameter_count
                )),
                file: file.path.clone(),
                span: span.clone(),
                rule_id: Some("too_many_parameters".to_string()),
                suggestion: Some("Consider using a parameter object or breaking the function down".to_string()),
                fix: None,
                metadata: vec![
                    ("function_name".to_string(), func_complexity.name.clone()),
                    ("parameter_count".to_string(), func_complexity.parameter_count.to_string()),
                ],
            });
        }

        issues
    }

    fn analyze_file_complexity(&self, ast: &AstNode, file: &SourceFile) -> Result<Vec<Issue>, String> {
        let mut issues = Vec::new();

        // Check for deeply nested structures
        let max_nesting = self.find_max_nesting_depth(ast);
        if max_nesting > self.complexity_thresholds.max_nesting_depth {
            issues.push(Issue {
                id: "COMP005".to_string(),
                severity: SeverityLevel::Medium,
                category: IssueCategory::Complexity,
                message: format!("Deep nesting detected: {} levels", max_nesting),
                description: Some("Deep nesting makes code harder to read and understand. Consider extracting nested logic into separate functions.".to_string()),
                file: file.path.clone(),
                span: Span {
                    start: Position { line: 1, column: 1, byte_offset: 0 },
                    end: Position { line: 1, column: 1, byte_offset: 0 },
                },
                rule_id: Some("deep_nesting".to_string()),
                suggestion: Some("Extract nested logic into separate functions".to_string()),
                fix: None,
                metadata: vec![
                    ("nesting_depth".to_string(), max_nesting.to_string()),
                    ("threshold".to_string(), self.complexity_thresholds.max_nesting_depth.to_string()),
                ],
            });
        }

        Ok(issues)
    }

    fn extract_function_name(&self, func_node: &AstNode) -> String {
        // Extract function name from AST node
        // This is language-specific and simplified
        if func_node.content.contains("fn ") {
            // Rust function
            if let Some(start) = func_node.content.find("fn ") {
                if let Some(end) = func_node.content[start + 3..].find("(") {
                    return func_node.content[start + 3..start + 3 + end].trim().to_string();
                }
            }
        } else if func_node.content.contains("function ") {
            // JavaScript function
            if let Some(start) = func_node.content.find("function ") {
                if let Some(end) = func_node.content[start + 9..].find("(") {
                    return func_node.content[start + 9..start + 9 + end].trim().to_string();
                }
            }
        }
        
        "anonymous".to_string()
    }

    fn find_max_nesting_depth(&self, ast: &AstNode) -> u32 {
        // Simplified nesting depth calculation
        // Real implementation would traverse AST recursively
        let nested_patterns = vec![
            "(if_statement (if_statement (if_statement)))",
            "(while_statement (if_statement (for_statement)))",
            "(for_statement (if_statement (while_statement)))",
        ];

        let mut max_depth = 1;
        for pattern in nested_patterns {
            if let Ok(nodes) = query_ast(ast, pattern) {
                if !nodes.is_empty() {
                    max_depth = max_depth.max(3);
                }
            }
        }

        max_depth
    }

    fn count_comment_lines(&self, content: &str) -> u32 {
        content.lines()
            .map(|line| line.trim())
            .filter(|line| line.starts_with("//") || line.starts_with("#") || line.starts_with("/*"))
            .count() as u32
    }

    fn calculate_maintainability_index(&self, file: &SourceFile, functions: &HashMap<String, FunctionComplexity>) -> f64 {
        if functions.is_empty() {
            return 100.0;
        }

        let avg_maintainability: f64 = functions.values()
            .map(|f| f.maintainability_index)
            .sum::<f64>() / functions.len() as f64;

        // Factor in file-level metrics
        let lines_of_code = file.content.lines().count() as f64;
        let file_size_penalty = if lines_of_code > 500.0 { 10.0 } else { 0.0 };

        (avg_maintainability - file_size_penalty).max(0.0)
    }

    fn calculate_total_cognitive_complexity(&self) -> f64 {
        self.function_stats.values()
            .map(|f| f.cognitive_complexity as f64)
            .sum()
    }

    fn calculate_complexity_debt(&self, issues: &[Issue]) -> u32 {
        issues.iter().map(|issue| {
            match issue.id.as_str() {
                "COMP001" => 60, // High cyclomatic complexity = 1 hour
                "COMP002" => 45, // High cognitive complexity = 45 minutes
                "COMP003" => 30, // Long function = 30 minutes
                "COMP004" => 20, // Too many parameters = 20 minutes
                "COMP005" => 40, // Deep nesting = 40 minutes
                _ => 15,
            }
        }).sum()
    }
}