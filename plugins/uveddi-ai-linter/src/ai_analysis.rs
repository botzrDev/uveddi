use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

/// AI model providers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AiProvider {
    Ollama,
    OpenAi,
    Anthropic,
    Google,
    HuggingFace,
    Local,
}

/// AI model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiModelConfig {
    pub provider: AiProvider,
    pub model_name: String,
    pub endpoint: String,
    pub api_key: Option<String>,
    pub max_tokens: u32,
    pub temperature: f32,
    pub timeout_seconds: u32,
}

impl Default for AiModelConfig {
    fn default() -> Self {
        Self {
            provider: AiProvider::Ollama,
            model_name: "deepseek-coder:6.7b".to_string(),
            endpoint: "http://localhost:11434".to_string(),
            api_key: None,
            max_tokens: 2048,
            temperature: 0.1,
            timeout_seconds: 60,
        }
    }
}

/// Types of analysis the AI can perform
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisType {
    CodeQuality,
    SecurityScan,
    PerformanceReview,
    StyleCheck,
    DocumentationReview,
    TestGeneration,
    RefactoringSuggestions,
}

/// Types of AI claims
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimType {
    Factual,        // "Function X calls function Y"
    Structural,     // "This class has 10 methods"
    Behavioral,     // "This function returns an error when input is null"
    Quality,        // "This code violates DRY principle"
    Performance,    // "This loop has O(n²) complexity"
    Security,       // "This function is vulnerable to XSS"
}

/// Evidence supporting an AI claim
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub source: EvidenceSource,
    pub location: TextSpan,
    pub description: String,
    pub strength: f32, // 0.0 to 1.0
}

/// Sources of evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSource {
    AstAnalysis,
    StaticAnalysis,
    PatternMatching,
    CallGraph,
    DataFlow,
    ControlFlow,
    Documentation,
    Tests,
}

/// Text span for claims and evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSpan {
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
    pub byte_start: u32,
    pub byte_end: u32,
}

/// AI-generated claim about code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiClaim {
    pub id: String,
    pub claim_type: ClaimType,
    pub statement: String,
    pub confidence: f32,
    pub evidence: Vec<Evidence>,
    pub source_span: Option<TextSpan>,
    pub verifiable: bool,
    pub verified: Option<bool>,
}

/// Verification methods for claims
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationMethod {
    AstQuery,
    SymbolLookup,
    CallGraphAnalysis,
    DataFlowAnalysis,
    StaticAnalysis,
    PatternVerification,
    CrossReference,
}

/// Hallucination detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HallucinationCheck {
    pub claim_id: String,
    pub is_hallucination: bool,
    pub confidence: f32,
    pub contradictory_evidence: Vec<Evidence>,
    pub verification_method: VerificationMethod,
    pub details: String,
}

/// Verification status of claims
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VerificationStatus {
    Unverified,
    Verified,
    Disputed,
    Hallucination,
    Partial,
}

/// AI-generated issue with confidence scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiIssue {
    pub id: String,
    pub severity: String,
    pub message: String,
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub category: String,
    pub confidence: f32,
    pub reasoning: String,
    pub claims: Vec<AiClaim>,
    pub alternatives: Vec<String>,
    pub human_review_required: bool,
    pub verification_status: VerificationStatus,
    pub suggestion: Option<String>,
}

/// Priority levels for suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PriorityLevel {
    Critical,
    High,
    Medium,
    Low,
    NiceToHave,
}

/// Effort estimates for improvements
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EffortEstimate {
    Trivial,      // < 1 hour
    Minor,        // 1-4 hours
    Moderate,     // 1-2 days
    Major,        // 1 week
    Extensive,    // > 1 week
}

/// Improvement suggestion categories
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SuggestionCategory {
    Refactoring,
    Performance,
    Security,
    Maintainability,
    Testing,
    Documentation,
    Architecture,
}

/// Improvement suggestion from AI analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementSuggestion {
    pub category: SuggestionCategory,
    pub priority: PriorityLevel,
    pub description: String,
    pub rationale: String,
    pub code_example: Option<String>,
    pub estimated_effort: EffortEstimate,
}

/// Detected code patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodePattern {
    pub name: String,
    pub description: String,
    pub confidence: f32,
    pub instances: Vec<TextSpan>,
    pub pattern_type: PatternCategory,
    pub ai_generated_likely: bool,
}

/// Pattern categories
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatternCategory {
    AntiPattern,
    DesignPattern,
    Idiom,
    AiSignature,
    StyleViolation,
    BestPractice,
}

/// Quality trends for metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrendDirection {
    Improving,
    Stable,
    Degrading,
    Unknown,
}

/// Quality trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityTrend {
    pub metric_name: String,
    pub current_value: f64,
    pub trend_direction: TrendDirection,
    pub confidence: f32,
    pub interpretation: String,
}

/// AI code review result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiReview {
    pub summary: String,
    pub overall_quality_score: f32,
    pub issues: Vec<AiIssue>,
    pub suggestions: Vec<ImprovementSuggestion>,
    pub patterns_detected: Vec<CodePattern>,
    pub quality_trends: Vec<QualityTrend>,
}

/// Grounding context for claim verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundingContext {
    pub file_path: String,
    pub language: String,
    pub symbol_table: HashMap<String, String>,
    pub call_graph: Vec<(String, String)>,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub dependencies: Vec<String>,
}

/// AI analyzer implementation
#[wasm_bindgen]
pub struct AiAnalyzer {
    config: AiModelConfig,
    patterns: Vec<CodePattern>,
    claim_history: Vec<AiClaim>,
}

#[wasm_bindgen]
impl AiAnalyzer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            config: AiModelConfig::default(),
            patterns: Vec::new(),
            claim_history: Vec::new(),
        }
    }

    /// Configure AI model
    #[wasm_bindgen]
    pub fn configure_model(&mut self, config_json: &str) -> Result<(), JsValue> {
        self.config = serde_json::from_str(config_json)
            .map_err(|e| JsValue::from_str(&format!("Config parse error: {}", e)))?;
        Ok(())
    }

    /// Analyze code with AI
    #[wasm_bindgen]
    pub async fn analyze_code(&mut self, code: &str, language: &str, file_path: &str, analysis_type: &str) -> Result<JsValue, JsValue> {
        let analysis_type: AnalysisType = serde_json::from_str(&format!("\"{}\"", analysis_type))
            .map_err(|e| JsValue::from_str(&format!("Invalid analysis type: {}", e)))?;

        // Create grounding context
        let context = GroundingContext {
            file_path: file_path.to_string(),
            language: language.to_string(),
            symbol_table: self.extract_symbols(code, language),
            call_graph: self.build_call_graph(code, language),
            imports: self.extract_imports(code, language),
            exports: self.extract_exports(code, language),
            dependencies: Vec::new(),
        };

        // Generate AI prompt based on analysis type
        let prompt = self.create_analysis_prompt(code, language, &analysis_type);

        // Call AI model (placeholder - would integrate with actual AI services)
        let ai_response = self.call_ai_model(&prompt, &context).await?;

        // Extract and verify claims
        let claims = self.extract_claims(&ai_response, &context)?;
        let verified_claims = self.verify_claims(claims, &context)?;

        // Detect AI patterns
        let patterns = self.detect_ai_patterns(code, language)?;

        // Generate issues from verified claims
        let issues = self.claims_to_issues(&verified_claims, file_path);

        // Create review result
        let review = AiReview {
            summary: format!("AI analysis of {} found {} issues", file_path, issues.len()),
            overall_quality_score: self.calculate_quality_score(&issues),
            issues,
            suggestions: self.generate_suggestions(&verified_claims),
            patterns_detected: patterns,
            quality_trends: self.analyze_quality_trends(&verified_claims),
        };

        Ok(JsValue::from_serde(&review).unwrap())
    }

    /// Detect AI-generated code patterns
    #[wasm_bindgen]
    pub fn detect_ai_patterns(&self, code: &str, language: &str) -> Result<JsValue, JsValue> {
        let patterns = self.detect_ai_signatures(code, language)?;
        Ok(JsValue::from_serde(&patterns).unwrap())
    }

    /// Verify claims against code
    #[wasm_bindgen]
    pub fn verify_claims(&self, claims_json: &str, context_json: &str) -> Result<JsValue, JsValue> {
        let claims: Vec<AiClaim> = serde_json::from_str(claims_json)
            .map_err(|e| JsValue::from_str(&format!("Claims parse error: {}", e)))?;
        
        let context: GroundingContext = serde_json::from_str(context_json)
            .map_err(|e| JsValue::from_str(&format!("Context parse error: {}", e)))?;

        let verified_claims = self.verify_claims(claims, &context)?;
        Ok(JsValue::from_serde(&verified_claims).unwrap())
    }

    /// Get analyzer statistics
    #[wasm_bindgen]
    pub fn get_statistics(&self) -> JsValue {
        let stats = HashMap::from([
            ("detected_patterns", self.patterns.len() as f64),
            ("total_claims", self.claim_history.len() as f64),
            ("verified_claims", self.claim_history.iter().filter(|c| c.verified == Some(true)).count() as f64),
            ("hallucinations", self.claim_history.iter().filter(|c| c.verified == Some(false)).count() as f64),
        ]);

        JsValue::from_serde(&stats).unwrap()
    }
}

impl AiAnalyzer {
    fn extract_symbols(&self, code: &str, _language: &str) -> HashMap<String, String> {
        let mut symbols = HashMap::new();
        
        // Simple regex-based symbol extraction (would be replaced with proper AST analysis)
        let function_regex = regex::Regex::new(r"\b(?:fn|def|function)\s+(\w+)").unwrap();
        for cap in function_regex.captures_iter(code) {
            if let Some(name) = cap.get(1) {
                symbols.insert(name.as_str().to_string(), "function".to_string());
            }
        }

        let class_regex = regex::Regex::new(r"\b(?:class|struct)\s+(\w+)").unwrap();
        for cap in class_regex.captures_iter(code) {
            if let Some(name) = cap.get(1) {
                symbols.insert(name.as_str().to_string(), "class".to_string());
            }
        }

        symbols
    }

    fn build_call_graph(&self, code: &str, _language: &str) -> Vec<(String, String)> {
        // Simplified call graph construction
        let mut calls = Vec::new();
        
        let call_regex = regex::Regex::new(r"(\w+)\s*\(").unwrap();
        for cap in call_regex.captures_iter(code) {
            if let Some(func) = cap.get(1) {
                calls.push(("caller".to_string(), func.as_str().to_string()));
            }
        }

        calls
    }

    fn extract_imports(&self, code: &str, language: &str) -> Vec<String> {
        let mut imports = Vec::new();

        let import_pattern = match language {
            "rust" => r"use\s+([^;]+);",
            "python" => r"(?:from\s+(\S+)\s+)?import\s+([^;\n]+)",
            "javascript" | "typescript" => r"import\s+.*\s+from\s+['\"]([^'\"]+)['\"]",
            _ => return imports,
        };

        if let Ok(regex) = regex::Regex::new(import_pattern) {
            for cap in regex.captures_iter(code) {
                if let Some(import) = cap.get(1) {
                    imports.push(import.as_str().to_string());
                }
            }
        }

        imports
    }

    fn extract_exports(&self, code: &str, language: &str) -> Vec<String> {
        let mut exports = Vec::new();

        let export_pattern = match language {
            "rust" => r"pub\s+(?:fn|struct|enum|const)\s+(\w+)",
            "javascript" | "typescript" => r"export\s+(?:function|class|const|let|var)\s+(\w+)",
            _ => return exports,
        };

        if let Ok(regex) = regex::Regex::new(export_pattern) {
            for cap in regex.captures_iter(code) {
                if let Some(export) = cap.get(1) {
                    exports.push(export.as_str().to_string());
                }
            }
        }

        exports
    }

    fn create_analysis_prompt(&self, code: &str, language: &str, analysis_type: &AnalysisType) -> String {
        let instruction = match analysis_type {
            AnalysisType::CodeQuality => "Analyze this code for quality issues including code smells, anti-patterns, and maintainability concerns.",
            AnalysisType::SecurityScan => "Analyze this code for security vulnerabilities including injection attacks, authentication issues, and data exposure.",
            AnalysisType::PerformanceReview => "Analyze this code for performance issues including algorithmic complexity, resource usage, and optimization opportunities.",
            AnalysisType::StyleCheck => "Analyze this code for style and formatting issues according to best practices.",
            AnalysisType::DocumentationReview => "Analyze this code for documentation quality and completeness.",
            AnalysisType::TestGeneration => "Analyze this code and suggest test cases for comprehensive coverage.",
            AnalysisType::RefactoringSuggestions => "Analyze this code and suggest refactoring opportunities for better design.",
        };

        format!(
            "{instruction}

Language: {language}

Code:
```{language}
{code}
```

Please provide:
1. Specific issues found with line numbers
2. Severity level (critical, high, medium, low, info)
3. Detailed explanation of each issue
4. Concrete suggestions for improvement
5. Rate your confidence in each finding (0.0 to 1.0)

Format your response as structured analysis with clear reasoning.",
            instruction = instruction,
            language = language,
            code = code
        )
    }

    async fn call_ai_model(&self, prompt: &str, _context: &GroundingContext) -> Result<String, JsValue> {
        // Placeholder for actual AI model integration
        // In a real implementation, this would call Ollama, OpenAI, etc.
        
        // Simple mock response for demonstration
        let mock_response = format!(
            r#"
Analysis Results:

Issue 1 (Line 5): Potential code smell - long method
Severity: medium
Confidence: 0.8
Explanation: This method has too many responsibilities and should be refactored
Suggestion: Extract smaller methods or use composition

Issue 2 (Line 12): Performance concern - nested loops
Severity: high  
Confidence: 0.9
Explanation: Nested loops can cause O(n²) complexity
Suggestion: Consider using more efficient algorithms or data structures

Pattern Detected: This code appears to follow standard {} conventions
AI-Generated Likelihood: 0.2
"#,
            _context.language
        );

        Ok(mock_response)
    }

    fn extract_claims(&self, ai_response: &str, context: &GroundingContext) -> Result<Vec<AiClaim>, JsValue> {
        let mut claims = Vec::new();

        // Simple claim extraction from AI response
        // In a real implementation, this would be more sophisticated
        let lines: Vec<&str> = ai_response.lines().collect();
        
        for (i, line) in lines.iter().enumerate() {
            if line.contains("Issue") && line.contains("Line") {
                // Extract line number
                let line_num_regex = regex::Regex::new(r"Line (\d+)").unwrap();
                let line_num = if let Some(cap) = line_num_regex.captures(line) {
                    cap.get(1).unwrap().as_str().parse::<u32>().unwrap_or(1)
                } else {
                    1
                };

                // Extract severity from next few lines
                let severity = lines.get(i + 1)
                    .and_then(|s| s.strip_prefix("Severity: "))
                    .unwrap_or("medium");

                // Extract confidence
                let confidence = lines.get(i + 2)
                    .and_then(|s| s.strip_prefix("Confidence: "))
                    .and_then(|s| s.parse::<f32>().ok())
                    .unwrap_or(0.5);

                let claim_id = format!("claim-{}", claims.len());
                
                claims.push(AiClaim {
                    id: claim_id.clone(),
                    claim_type: ClaimType::Quality,
                    statement: line.to_string(),
                    confidence,
                    evidence: vec![Evidence {
                        source: EvidenceSource::StaticAnalysis,
                        location: TextSpan {
                            start_line: line_num,
                            start_column: 0,
                            end_line: line_num,
                            end_column: 0,
                            byte_start: 0,
                            byte_end: 0,
                        },
                        description: format!("AI detected issue at line {}", line_num),
                        strength: confidence,
                    }],
                    source_span: Some(TextSpan {
                        start_line: line_num,
                        start_column: 0,
                        end_line: line_num,
                        end_column: 0,
                        byte_start: 0,
                        byte_end: 0,
                    }),
                    verifiable: true,
                    verified: None,
                });
            }
        }

        Ok(claims)
    }

    fn verify_claims(&self, mut claims: Vec<AiClaim>, _context: &GroundingContext) -> Result<Vec<AiClaim>, JsValue> {
        // Simple verification logic
        // In a real implementation, this would cross-reference with AST, call graphs, etc.
        
        for claim in &mut claims {
            // Mock verification - in practice, this would check against deterministic sources
            claim.verified = Some(claim.confidence > 0.7);
        }

        Ok(claims)
    }

    fn detect_ai_signatures(&self, code: &str, _language: &str) -> Result<Vec<CodePattern>, JsValue> {
        let mut patterns = Vec::new();

        // Common AI-generated code indicators
        let ai_indicators = [
            ("TODO comments", r"(?i)todo[:\s]", PatternCategory::AiSignature),
            ("Generic variable names", r"\b(data|item|result|temp)\d*\b", PatternCategory::AiSignature),
            ("Placeholder comments", r"(?i)(placeholder|implement|add.*here)", PatternCategory::AiSignature),
            ("Generated by comments", r"(?i)(generated|auto-generated|automatically)", PatternCategory::AiSignature),
        ];

        for (name, pattern, category) in &ai_indicators {
            if let Ok(regex) = regex::Regex::new(pattern) {
                let matches: Vec<_> = regex.find_iter(code).collect();
                if !matches.is_empty() {
                    let instances = matches.iter().map(|m| {
                        let line_num = code[..m.start()].lines().count() as u32;
                        TextSpan {
                            start_line: line_num,
                            start_column: 0,
                            end_line: line_num,
                            end_column: m.len() as u32,
                            byte_start: m.start() as u32,
                            byte_end: m.end() as u32,
                        }
                    }).collect();

                    let confidence = (matches.len() as f32 / code.lines().count() as f32) * 2.0;
                    let confidence = confidence.min(1.0);

                    patterns.push(CodePattern {
                        name: name.to_string(),
                        description: format!("Detected {} instances of {}", matches.len(), name.to_lowercase()),
                        confidence,
                        instances,
                        pattern_type: category.clone(),
                        ai_generated_likely: confidence > 0.5,
                    });
                }
            }
        }

        Ok(patterns)
    }

    fn claims_to_issues(&self, claims: &[AiClaim], file_path: &str) -> Vec<AiIssue> {
        claims.iter().enumerate().map(|(i, claim)| {
            let span = claim.source_span.as_ref().unwrap_or(&TextSpan {
                start_line: 1,
                start_column: 0,
                end_line: 1,
                end_column: 0,
                byte_start: 0,
                byte_end: 0,
            });

            AiIssue {
                id: format!("ai-{}", i),
                severity: if claim.confidence > 0.8 { "high" } else if claim.confidence > 0.5 { "medium" } else { "low" }.to_string(),
                message: claim.statement.clone(),
                file: file_path.to_string(),
                line: span.start_line,
                column: span.start_column,
                category: "ai-analysis".to_string(),
                confidence: claim.confidence,
                reasoning: format!("AI analysis with {} confidence", claim.confidence),
                claims: vec![claim.clone()],
                alternatives: Vec::new(),
                human_review_required: claim.confidence < 0.8,
                verification_status: if claim.verified == Some(true) { 
                    VerificationStatus::Verified 
                } else if claim.verified == Some(false) { 
                    VerificationStatus::Hallucination 
                } else { 
                    VerificationStatus::Unverified 
                },
                suggestion: None,
            }
        }).collect()
    }

    fn calculate_quality_score(&self, issues: &[AiIssue]) -> f32 {
        if issues.is_empty() {
            return 1.0;
        }

        let total_severity: f32 = issues.iter().map(|issue| {
            match issue.severity.as_str() {
                "critical" => 1.0,
                "high" => 0.8,
                "medium" => 0.5,
                "low" => 0.3,
                _ => 0.1,
            }
        }).sum();

        let max_possible = issues.len() as f32;
        (max_possible - total_severity) / max_possible
    }

    fn generate_suggestions(&self, claims: &[AiClaim]) -> Vec<ImprovementSuggestion> {
        claims.iter().filter_map(|claim| {
            if claim.verified == Some(true) {
                Some(ImprovementSuggestion {
                    category: SuggestionCategory::Refactoring,
                    priority: if claim.confidence > 0.8 { PriorityLevel::High } else { PriorityLevel::Medium },
                    description: format!("Address: {}", claim.statement),
                    rationale: "AI-detected issue with high confidence".to_string(),
                    code_example: None,
                    estimated_effort: EffortEstimate::Minor,
                })
            } else {
                None
            }
        }).collect()
    }

    fn analyze_quality_trends(&self, _claims: &[AiClaim]) -> Vec<QualityTrend> {
        // Placeholder for quality trend analysis
        vec![
            QualityTrend {
                metric_name: "AI Confidence".to_string(),
                current_value: 0.75,
                trend_direction: TrendDirection::Stable,
                confidence: 0.8,
                interpretation: "AI analysis confidence remains stable".to_string(),
            }
        ]
    }
}