use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

/// Halstead complexity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HalsteadMetrics {
    pub vocabulary: u32,         // n = n1 + n2
    pub length: u32,            // N = N1 + N2
    pub calculated_length: f64,  // n1*log2(n1) + n2*log2(n2)
    pub volume: f64,            // N * log2(n)
    pub difficulty: f64,        // (n1/2) * (N2/n2)
    pub effort: f64,            // difficulty * volume
    pub time: f64,              // effort / 18
    pub bugs: f64,              // volume / 3000
}

impl Default for HalsteadMetrics {
    fn default() -> Self {
        Self {
            vocabulary: 0,
            length: 0,
            calculated_length: 0.0,
            volume: 0.0,
            difficulty: 0.0,
            effort: 0.0,
            time: 0.0,
            bugs: 0.0,
        }
    }
}

/// Complexity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityMetrics {
    pub cyclomatic_complexity: u32,
    pub cognitive_complexity: u32,
    pub npath_complexity: u64,
    pub essential_complexity: u32,
    pub design_complexity: u32,
    pub halstead_metrics: HalsteadMetrics,
}

impl Default for ComplexityMetrics {
    fn default() -> Self {
        Self {
            cyclomatic_complexity: 1, // Base complexity
            cognitive_complexity: 0,
            npath_complexity: 1,
            essential_complexity: 1,
            design_complexity: 1,
            halstead_metrics: HalsteadMetrics::default(),
        }
    }
}

/// Coupling metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouplingMetrics {
    pub afferent_coupling: u32,     // Ca - incoming dependencies
    pub efferent_coupling: u32,     // Ce - outgoing dependencies
    pub instability: f64,           // I = Ce / (Ca + Ce)
    pub coupling_between_objects: u32, // CBO
    pub response_for_class: u32,    // RFC
    pub message_passing_coupling: u32, // MPC
    pub data_abstraction_coupling: u32, // DAC
}

impl Default for CouplingMetrics {
    fn default() -> Self {
        Self {
            afferent_coupling: 0,
            efferent_coupling: 0,
            instability: 0.0,
            coupling_between_objects: 0,
            response_for_class: 0,
            message_passing_coupling: 0,
            data_abstraction_coupling: 0,
        }
    }
}

/// Cohesion metrics  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohesionMetrics {
    pub lack_of_cohesion_methods_1: f64, // LCOM1
    pub lack_of_cohesion_methods_2: f64, // LCOM2
    pub lack_of_cohesion_methods_3: f64, // LCOM3
    pub lack_of_cohesion_methods_4: f64, // LCOM4
    pub tight_class_cohesion: f64,       // TCC
    pub loose_class_cohesion: f64,       // LCC
}

impl Default for CohesionMetrics {
    fn default() -> Self {
        Self {
            lack_of_cohesion_methods_1: 0.0,
            lack_of_cohesion_methods_2: 0.0,
            lack_of_cohesion_methods_3: 0.0,
            lack_of_cohesion_methods_4: 0.0,
            tight_class_cohesion: 1.0,
            loose_class_cohesion: 1.0,
        }
    }
}

/// Size metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeMetrics {
    pub lines_of_code: u32,
    pub logical_lines_of_code: u32,
    pub comment_lines: u32,
    pub blank_lines: u32,
    pub number_of_methods: u32,
    pub number_of_classes: u32,
    pub number_of_functions: u32,
    pub number_of_statements: u32,
    pub number_of_files: u32,
    pub weighted_methods_per_class: u32, // WMC
}

impl Default for SizeMetrics {
    fn default() -> Self {
        Self {
            lines_of_code: 0,
            logical_lines_of_code: 0,
            comment_lines: 0,
            blank_lines: 0,
            number_of_methods: 0,
            number_of_classes: 0,
            number_of_functions: 0,
            number_of_statements: 0,
            number_of_files: 1,
            weighted_methods_per_class: 0,
        }
    }
}

/// Inheritance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InheritanceMetrics {
    pub depth_of_inheritance_tree: u32,     // DIT
    pub number_of_children: u32,            // NOC
    pub class_interface_size: u32,          // CIS
    pub number_of_methods_inherited: u32,   // NMI
    pub number_of_methods_overridden: u32,  // NMO
}

impl Default for InheritanceMetrics {
    fn default() -> Self {
        Self {
            depth_of_inheritance_tree: 0,
            number_of_children: 0,
            class_interface_size: 0,
            number_of_methods_inherited: 0,
            number_of_methods_overridden: 0,
        }
    }
}

/// Change coupling between files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouplingRelationship {
    pub file_a: String,
    pub file_b: String,
    pub coupling_strength: f64,
    pub co_change_frequency: u32,
    pub confidence: f64,
}

/// Historical metrics from Git analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalMetrics {
    pub change_frequency: u32,
    pub bug_fix_frequency: u32,
    pub lines_added: u32,
    pub lines_deleted: u32,
    pub commits_count: u32,
    pub unique_contributors: u32,
    pub last_modified: u64, // Unix timestamp
    pub age_days: u32,
    pub hotspot_score: f64,
    pub change_coupling: Vec<CouplingRelationship>,
}

impl Default for HistoricalMetrics {
    fn default() -> Self {
        Self {
            change_frequency: 0,
            bug_fix_frequency: 0,
            lines_added: 0,
            lines_deleted: 0,
            commits_count: 0,
            unique_contributors: 0,
            last_modified: 0,
            age_days: 0,
            hotspot_score: 0.0,
            change_coupling: Vec::new(),
        }
    }
}

/// Health rating levels
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthRating {
    Excellent,
    Good,
    Fair,
    Poor,
    Critical,
}

/// Refactoring types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefactoringType {
    ExtractMethod,
    ExtractClass,
    MoveMethod,
    Rename,
    SimplifyConditional,
    RemoveDuplicate,
    DecomposeComplexMethod,
    ReduceCoupling,
    ImproveCohesion,
}

/// Priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PriorityLevel {
    Critical,
    High,
    Medium,
    Low,
}

/// Effort levels
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EffortLevel {
    Minimal,
    Small,
    Medium,
    Large,
    Epic,
}

/// Refactoring suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringSuggestion {
    pub refactoring_type: RefactoringType,
    pub priority: PriorityLevel,
    pub description: String,
    pub estimated_effort: EffortLevel,
    pub benefits: Vec<String>,
    pub risks: Vec<String>,
    pub affected_files: Vec<String>,
}

/// Quality gate result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateResult {
    pub gate_name: String,
    pub passed: bool,
    pub threshold: f64,
    pub actual_value: f64,
    pub metric_name: String,
    pub message: String,
}

/// Maintainability metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintainabilityMetrics {
    pub maintainability_index: f64,
    pub technical_debt_ratio: f64,
    pub code_health_rating: HealthRating,
    pub refactoring_recommendations: Vec<RefactoringSuggestion>,
    pub quality_gates: Vec<QualityGateResult>,
}

impl Default for MaintainabilityMetrics {
    fn default() -> Self {
        Self {
            maintainability_index: 100.0,
            technical_debt_ratio: 0.0,
            code_health_rating: HealthRating::Excellent,
            refactoring_recommendations: Vec::new(),
            quality_gates: Vec::new(),
        }
    }
}

/// Algorithmic complexity classes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplexityClass {
    Constant,      // O(1)
    Logarithmic,   // O(log n)
    Linear,        // O(n)
    Linearithmic,  // O(n log n)
    Quadratic,     // O(n²)
    Cubic,         // O(n³)
    Exponential,   // O(2^n)
    Factorial,     // O(n!)
    Unknown,
}

/// Performance bottleneck types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BottleneckType {
    NestedLoops,
    RecursiveCalls,
    IoOperations,
    MemoryAllocation,
    Synchronization,
    DatabaseQueries,
    NetworkCalls,
}

/// Severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SeverityLevel {
    Critical,
    Major,
    Minor,
    Info,
}

/// Text span for locations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSpan {
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
    pub byte_start: u32,
    pub byte_end: u32,
}

/// Performance hotspot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotspot {
    pub function_name: String,
    pub file: String,
    pub span: TextSpan,
    pub complexity: ComplexityClass,
    pub estimated_cpu_impact: f64,
    pub estimated_memory_impact: f64,
    pub suggestions: Vec<String>,
}

/// Performance bottleneck
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bottleneck {
    pub bottleneck_type: BottleneckType,
    pub location: TextSpan,
    pub description: String,
    pub severity: SeverityLevel,
    pub optimization_suggestions: Vec<String>,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub algorithmic_complexity: ComplexityClass,
    pub memory_complexity: ComplexityClass,
    pub performance_hotspots: Vec<Hotspot>,
    pub potential_bottlenecks: Vec<Bottleneck>,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            algorithmic_complexity: ComplexityClass::Linear,
            memory_complexity: ComplexityClass::Constant,
            performance_hotspots: Vec::new(),
            potential_bottlenecks: Vec::new(),
        }
    }
}

/// Comprehensive metrics suite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensiveMetrics {
    pub complexity: ComplexityMetrics,
    pub coupling: CouplingMetrics,
    pub cohesion: CohesionMetrics,
    pub size: SizeMetrics,
    pub inheritance: InheritanceMetrics,
    pub historical: Option<HistoricalMetrics>,
    pub maintainability: MaintainabilityMetrics,
    pub performance: PerformanceMetrics,
    pub custom_metrics: HashMap<String, f64>,
}

/// Metrics analyzer implementation
#[wasm_bindgen]
pub struct MetricsAnalyzer {
    cached_metrics: HashMap<String, ComprehensiveMetrics>,
}

#[wasm_bindgen]
impl MetricsAnalyzer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            cached_metrics: HashMap::new(),
        }
    }

    /// Calculate comprehensive metrics for code
    #[wasm_bindgen]
    pub fn calculate_comprehensive_metrics(&mut self, code: &str, language: &str, file_path: &str) -> Result<JsValue, JsValue> {
        // Check cache first
        let cache_key = format!("{}:{}", file_path, Self::hash_code(code));
        if let Some(cached) = self.cached_metrics.get(&cache_key) {
            return Ok(JsValue::from_serde(cached).unwrap());
        }

        let mut metrics = ComprehensiveMetrics {
            complexity: self.calculate_complexity_metrics(code, language)?,
            coupling: self.calculate_coupling_metrics(code, language)?,
            cohesion: self.calculate_cohesion_metrics(code, language)?,
            size: self.calculate_size_metrics(code, language)?,
            inheritance: self.calculate_inheritance_metrics(code, language)?,
            historical: None, // Would be populated by Git analysis
            maintainability: self.calculate_maintainability_metrics(code, language)?,
            performance: self.calculate_performance_metrics(code, language)?,
            custom_metrics: HashMap::new(),
        };

        // Add custom metrics
        metrics.custom_metrics.insert("comment_ratio".to_string(), 
            metrics.size.comment_lines as f64 / metrics.size.lines_of_code.max(1) as f64);
        
        metrics.custom_metrics.insert("complexity_density".to_string(),
            metrics.complexity.cyclomatic_complexity as f64 / metrics.size.logical_lines_of_code.max(1) as f64);

        // Cache the result
        self.cached_metrics.insert(cache_key, metrics.clone());

        Ok(JsValue::from_serde(&metrics).unwrap())
    }

    /// Detect performance hotspots
    #[wasm_bindgen]
    pub fn detect_hotspots(&self, metrics_json: &str, thresholds_json: &str) -> Result<JsValue, JsValue> {
        let metrics: ComprehensiveMetrics = serde_json::from_str(metrics_json)
            .map_err(|e| JsValue::from_str(&format!("Metrics parse error: {}", e)))?;
        
        let thresholds: HashMap<String, f64> = serde_json::from_str(thresholds_json)
            .map_err(|e| JsValue::from_str(&format!("Thresholds parse error: {}", e)))?;

        let mut hotspots = Vec::new();

        // Check cyclomatic complexity
        if let Some(&threshold) = thresholds.get("cyclomatic_complexity") {
            if metrics.complexity.cyclomatic_complexity as f64 > threshold {
                hotspots.push(Hotspot {
                    function_name: "main_function".to_string(), // Would be extracted from AST
                    file: "current_file".to_string(),
                    span: TextSpan {
                        start_line: 1,
                        start_column: 0,
                        end_line: metrics.size.lines_of_code,
                        end_column: 0,
                        byte_start: 0,
                        byte_end: 0,
                    },
                    complexity: ComplexityClass::Quadratic, // Based on analysis
                    estimated_cpu_impact: 0.8,
                    estimated_memory_impact: 0.6,
                    suggestions: vec![
                        "Break down complex methods into smaller functions".to_string(),
                        "Consider using guard clauses to reduce nesting".to_string(),
                    ],
                });
            }
        }

        Ok(JsValue::from_serde(&hotspots).unwrap())
    }

    /// Generate refactoring suggestions
    #[wasm_bindgen]
    pub fn suggest_refactoring(&self, metrics_json: &str) -> Result<JsValue, JsValue> {
        let metrics: ComprehensiveMetrics = serde_json::from_str(metrics_json)
            .map_err(|e| JsValue::from_str(&format!("Metrics parse error: {}", e)))?;

        let mut suggestions = Vec::new();

        // High complexity suggests method extraction
        if metrics.complexity.cyclomatic_complexity > 10 {
            suggestions.push(RefactoringSuggestion {
                refactoring_type: RefactoringType::ExtractMethod,
                priority: PriorityLevel::High,
                description: "High cyclomatic complexity detected - consider extracting methods".to_string(),
                estimated_effort: EffortLevel::Medium,
                benefits: vec![
                    "Improved readability".to_string(),
                    "Better testability".to_string(),
                    "Reduced maintenance cost".to_string(),
                ],
                risks: vec![
                    "May introduce additional coupling".to_string(),
                ],
                affected_files: vec!["current_file".to_string()],
            });
        }

        // High coupling suggests architectural refactoring
        if metrics.coupling.coupling_between_objects > 7 {
            suggestions.push(RefactoringSuggestion {
                refactoring_type: RefactoringType::ReduceCoupling,
                priority: PriorityLevel::Medium,
                description: "High coupling detected - consider dependency injection or interfaces".to_string(),
                estimated_effort: EffortLevel::Large,
                benefits: vec![
                    "Improved modularity".to_string(),
                    "Better testability".to_string(),
                    "Easier to change".to_string(),
                ],
                risks: vec![
                    "Increased initial complexity".to_string(),
                ],
                affected_files: vec!["current_file".to_string()],
            });
        }

        // Low cohesion suggests class restructuring
        if metrics.cohesion.lack_of_cohesion_methods_4 > 0.8 {
            suggestions.push(RefactoringSuggestion {
                refactoring_type: RefactoringType::ExtractClass,
                priority: PriorityLevel::Medium,
                description: "Low cohesion detected - consider splitting class responsibilities".to_string(),
                estimated_effort: EffortLevel::Large,
                benefits: vec![
                    "Better separation of concerns".to_string(),
                    "Improved maintainability".to_string(),
                ],
                risks: vec![
                    "May require interface changes".to_string(),
                ],
                affected_files: vec!["current_file".to_string()],
            });
        }

        Ok(JsValue::from_serde(&suggestions).unwrap())
    }

    /// Evaluate quality gates
    #[wasm_bindgen]
    pub fn evaluate_quality_gates(&self, metrics_json: &str, gates_json: &str) -> Result<JsValue, JsValue> {
        let metrics: ComprehensiveMetrics = serde_json::from_str(metrics_json)
            .map_err(|e| JsValue::from_str(&format!("Metrics parse error: {}", e)))?;

        let gate_thresholds: HashMap<String, f64> = serde_json::from_str(gates_json)
            .map_err(|e| JsValue::from_str(&format!("Gates parse error: {}", e)))?;

        let mut results = Vec::new();

        // Evaluate each quality gate
        for (gate_name, threshold) in gate_thresholds {
            let (passed, actual_value, message) = match gate_name.as_str() {
                "max_complexity" => {
                    let value = metrics.complexity.cyclomatic_complexity as f64;
                    let passed = value <= threshold;
                    let message = if passed {
                        "Complexity within acceptable limits".to_string()
                    } else {
                        format!("Complexity {} exceeds threshold {}", value, threshold)
                    };
                    (passed, value, message)
                },
                "min_maintainability" => {
                    let value = metrics.maintainability.maintainability_index;
                    let passed = value >= threshold;
                    let message = if passed {
                        "Maintainability index acceptable".to_string()
                    } else {
                        format!("Maintainability index {} below threshold {}", value, threshold)
                    };
                    (passed, value, message)
                },
                "max_coupling" => {
                    let value = metrics.coupling.coupling_between_objects as f64;
                    let passed = value <= threshold;
                    let message = if passed {
                        "Coupling within acceptable limits".to_string()
                    } else {
                        format!("Coupling {} exceeds threshold {}", value, threshold)
                    };
                    (passed, value, message)
                },
                _ => continue,
            };

            results.push(QualityGateResult {
                gate_name: gate_name.clone(),
                passed,
                threshold,
                actual_value,
                metric_name: gate_name,
                message,
            });
        }

        Ok(JsValue::from_serde(&results).unwrap())
    }

    /// Get analyzer statistics
    #[wasm_bindgen]
    pub fn get_statistics(&self) -> JsValue {
        let stats = HashMap::from([
            ("cached_files", self.cached_metrics.len() as f64),
            ("total_analyses", self.cached_metrics.len() as f64),
        ]);

        JsValue::from_serde(&stats).unwrap()
    }
}

impl MetricsAnalyzer {
    fn hash_code(code: &str) -> String {
        format!("{:x}", md5::compute(code))
    }

    fn calculate_complexity_metrics(&self, code: &str, _language: &str) -> Result<ComplexityMetrics, JsValue> {
        let mut complexity = ComplexityMetrics::default();

        // Simple cyclomatic complexity calculation
        let decision_points = ["if", "else", "while", "for", "match", "case", "catch", "&&", "||"];
        for keyword in &decision_points {
            complexity.cyclomatic_complexity += code.matches(keyword).len() as u32;
        }

        // Ensure minimum complexity of 1
        complexity.cyclomatic_complexity = complexity.cyclomatic_complexity.max(1);

        // Cognitive complexity (simplified)
        complexity.cognitive_complexity = self.calculate_cognitive_complexity(code);

        // Halstead metrics (simplified)
        complexity.halstead_metrics = self.calculate_halstead_metrics(code)?;

        Ok(complexity)
    }

    fn calculate_cognitive_complexity(&self, code: &str) -> u32 {
        let mut complexity = 0;
        let mut nesting_level = 0;

        for line in code.lines() {
            let line = line.trim();
            
            // Increment nesting for blocks
            if line.contains('{') {
                nesting_level += 1;
            }
            
            // Decrement nesting for closing blocks
            if line.contains('}') {
                nesting_level = nesting_level.saturating_sub(1);
            }

            // Add complexity based on constructs and nesting
            let constructs = ["if", "while", "for", "match"];
            for construct in &constructs {
                if line.contains(construct) {
                    complexity += 1 + nesting_level;
                }
            }
        }

        complexity
    }

    fn calculate_halstead_metrics(&self, code: &str) -> Result<HalsteadMetrics, JsValue> {
        let operators = ["+", "-", "*", "/", "=", "==", "!=", "<", ">", "&&", "||"];
        let mut operator_count = 0;
        let mut unique_operators = std::collections::HashSet::new();

        for op in &operators {
            let count = code.matches(op).len();
            if count > 0 {
                unique_operators.insert(*op);
                operator_count += count;
            }
        }

        // Simplified operand detection (words that are not keywords)
        let keywords = ["if", "else", "while", "for", "fn", "let", "const", "var", "function"];
        let words: Vec<&str> = code.split_whitespace().collect();
        let mut operand_count = 0;
        let mut unique_operands = std::collections::HashSet::new();

        for word in words {
            let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric());
            if !clean_word.is_empty() && !keywords.contains(&clean_word) {
                unique_operands.insert(clean_word);
                operand_count += 1;
            }
        }

        let n1 = unique_operators.len() as u32;
        let n2 = unique_operands.len() as u32;
        let n = n1 + n2;
        let N1 = operator_count as u32;
        let N2 = operand_count as u32;
        let N = N1 + N2;

        let vocabulary = n;
        let length = N;
        let calculated_length = if n1 > 0 && n2 > 0 {
            n1 as f64 * (n1 as f64).log2() + n2 as f64 * (n2 as f64).log2()
        } else {
            0.0
        };
        let volume = if n > 0 { N as f64 * (n as f64).log2() } else { 0.0 };
        let difficulty = if n2 > 0 { (n1 as f64 / 2.0) * (N2 as f64 / n2 as f64) } else { 0.0 };
        let effort = difficulty * volume;
        let time = effort / 18.0;
        let bugs = volume / 3000.0;

        Ok(HalsteadMetrics {
            vocabulary,
            length,
            calculated_length,
            volume,
            difficulty,
            effort,
            time,
            bugs,
        })
    }

    fn calculate_coupling_metrics(&self, code: &str, _language: &str) -> Result<CouplingMetrics, JsValue> {
        let mut coupling = CouplingMetrics::default();

        // Simple coupling calculation based on imports and function calls
        let import_patterns = ["use ", "import ", "include ", "require("];
        for pattern in &import_patterns {
            coupling.efferent_coupling += code.matches(pattern).len() as u32;
        }

        // Function call pattern (simplified)
        let function_call_regex = regex::Regex::new(r"\w+\s*\(").unwrap();
        coupling.response_for_class = function_call_regex.find_iter(code).count() as u32;

        // Calculate instability
        coupling.instability = if coupling.afferent_coupling + coupling.efferent_coupling > 0 {
            coupling.efferent_coupling as f64 / (coupling.afferent_coupling + coupling.efferent_coupling) as f64
        } else {
            0.0
        };

        Ok(coupling)
    }

    fn calculate_cohesion_metrics(&self, code: &str, _language: &str) -> Result<CohesionMetrics, JsValue> {
        let mut cohesion = CohesionMetrics::default();

        // Simplified LCOM4 calculation
        // In a real implementation, this would analyze method-field relationships
        let method_count = code.matches("fn ").len() + code.matches("def ").len() + code.matches("function ").len();
        let field_count = code.matches("let ").len() + code.matches("var ").len() + code.matches("const ").len();

        if method_count > 1 && field_count > 0 {
            // Simplified calculation - in practice would need proper AST analysis
            cohesion.lack_of_cohesion_methods_4 = 1.0 - (field_count as f64 / (method_count * field_count) as f64);
        }

        Ok(cohesion)
    }

    fn calculate_size_metrics(&self, code: &str, _language: &str) -> Result<SizeMetrics, JsValue> {
        let mut size = SizeMetrics::default();

        let lines: Vec<&str> = code.lines().collect();
        size.lines_of_code = lines.len() as u32;

        for line in &lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                size.blank_lines += 1;
            } else if trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("/*") {
                size.comment_lines += 1;
            } else {
                size.logical_lines_of_code += 1;
            }
        }

        // Count functions, classes, etc.
        size.number_of_functions = (code.matches("fn ").len() + code.matches("def ").len() + code.matches("function ").len()) as u32;
        size.number_of_classes = (code.matches("class ").len() + code.matches("struct ").len()) as u32;
        size.number_of_statements = size.logical_lines_of_code; // Simplified

        Ok(size)
    }

    fn calculate_inheritance_metrics(&self, code: &str, _language: &str) -> Result<InheritanceMetrics, JsValue> {
        let mut inheritance = InheritanceMetrics::default();

        // Simple inheritance detection
        let inheritance_patterns = ["extends", "implements", ":", "inherit"];
        for pattern in &inheritance_patterns {
            if code.contains(pattern) {
                inheritance.depth_of_inheritance_tree = 1; // Simplified
                break;
            }
        }

        Ok(inheritance)
    }

    fn calculate_maintainability_metrics(&self, code: &str, _language: &str) -> Result<MaintainabilityMetrics, JsValue> {
        let mut maintainability = MaintainabilityMetrics::default();

        // Simplified maintainability index calculation
        // Real formula: MI = 171 - 5.2 * ln(HalsteadVolume) - 0.23 * CyclomaticComplexity - 16.2 * ln(LOC)
        let lines = code.lines().count() as f64;
        let complexity = self.calculate_cognitive_complexity(code) as f64;
        
        maintainability.maintainability_index = if lines > 0.0 {
            (171.0 - 5.2 * (lines.ln()) - 0.23 * complexity - 16.2 * (lines.ln())).max(0.0)
        } else {
            100.0
        };

        // Determine health rating
        maintainability.code_health_rating = match maintainability.maintainability_index {
            mi if mi >= 85.0 => HealthRating::Excellent,
            mi if mi >= 70.0 => HealthRating::Good,
            mi if mi >= 50.0 => HealthRating::Fair,
            mi if mi >= 25.0 => HealthRating::Poor,
            _ => HealthRating::Critical,
        };

        Ok(maintainability)
    }

    fn calculate_performance_metrics(&self, code: &str, _language: &str) -> Result<PerformanceMetrics, JsValue> {
        let mut performance = PerformanceMetrics::default();

        // Detect nested loops (potential O(n²) complexity)
        let nested_loop_count = self.count_nested_constructs(code, &["for", "while"]);
        performance.algorithmic_complexity = match nested_loop_count {
            0 => ComplexityClass::Constant,
            1 => ComplexityClass::Linear,
            2 => ComplexityClass::Quadratic,
            3 => ComplexityClass::Cubic,
            _ => ComplexityClass::Exponential,
        };

        // Detect potential bottlenecks
        if nested_loop_count >= 2 {
            performance.potential_bottlenecks.push(Bottleneck {
                bottleneck_type: BottleneckType::NestedLoops,
                location: TextSpan {
                    start_line: 1,
                    start_column: 0,
                    end_line: 1,
                    end_column: 0,
                    byte_start: 0,
                    byte_end: 0,
                },
                description: format!("{} levels of nested loops detected", nested_loop_count),
                severity: SeverityLevel::Major,
                optimization_suggestions: vec![
                    "Consider using more efficient algorithms".to_string(),
                    "Look for opportunities to reduce iteration complexity".to_string(),
                ],
            });
        }

        Ok(performance)
    }

    fn count_nested_constructs(&self, code: &str, constructs: &[&str]) -> u32 {
        let mut max_nesting = 0;
        let mut current_nesting = 0;

        for line in code.lines() {
            let line = line.trim();
            
            // Check for construct keywords
            for construct in constructs {
                if line.contains(construct) {
                    current_nesting += 1;
                    max_nesting = max_nesting.max(current_nesting);
                }
            }

            // Simple brace counting for nesting
            if line.contains('{') {
                // Opening brace
            }
            if line.contains('}') {
                current_nesting = current_nesting.saturating_sub(1);
            }
        }

        max_nesting
    }
}