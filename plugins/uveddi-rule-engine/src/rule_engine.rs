use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

/// Rule definition format
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleFormat {
    Toml,
    Yaml,
    Json,
    Dsl,
}

/// Pattern types for rule matching
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatternType {
    TreeSitterQuery,
    Regex,
    AstPattern,
    StructuralPattern,
}

/// Rule severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

/// Individual rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub pattern: String,
    pub pattern_type: PatternType,
    pub severity: RuleSeverity,
    pub category: String,
    pub message: String,
    pub suggestion: Option<String>,
    pub auto_fix: Option<String>,
    pub languages: Vec<String>,
    pub enabled: bool,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl Default for RuleDefinition {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            pattern: String::new(),
            pattern_type: PatternType::Regex,
            severity: RuleSeverity::Warning,
            category: String::new(),
            message: String::new(),
            suggestion: None,
            auto_fix: None,
            languages: vec!["*".to_string()],
            enabled: true,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}

/// Rule set containing multiple rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSet {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub format: RuleFormat,
    pub rules: Vec<RuleDefinition>,
    pub extends: Vec<String>,
    pub overrides: Vec<RuleOverride>,
}

/// Rule override configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleOverride {
    pub rule_id: String,
    pub enabled: Option<bool>,
    pub severity: Option<RuleSeverity>,
    pub message: Option<String>,
    pub languages: Option<Vec<String>>,
}

/// Rule match result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleMatch {
    pub rule_id: String,
    pub spans: Vec<TextSpan>,
    pub captured_groups: HashMap<String, String>,
    pub confidence: f32,
    pub context: HashMap<String, String>,
}

/// Text span for rule matches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSpan {
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
    pub byte_start: u32,
    pub byte_end: u32,
}

/// Policy violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolation {
    pub rule_id: String,
    pub policy_id: String,
    pub severity: RuleSeverity,
    pub message: String,
    pub file: String,
    pub span: TextSpan,
    pub remediation: Option<String>,
    pub exemption: Option<ExemptionInfo>,
}

/// Exemption information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExemptionInfo {
    pub reason: String,
    pub expires: Option<u64>, // Unix timestamp
    pub approver: String,
    pub ticket: Option<String>,
}

/// Organizational policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationalPolicy {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rules: Vec<String>, // Rule IDs
    pub enforcement_level: EnforcementLevel,
    pub scope: PolicyScope,
    pub exceptions: Vec<PolicyException>,
}

/// Policy enforcement levels
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EnforcementLevel {
    Disabled,
    Warning,
    Error,
    Blocking,
}

/// Policy scope definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyScope {
    pub languages: Vec<String>,
    pub file_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub directories: Vec<String>,
}

/// Policy exception
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyException {
    pub pattern: String,
    pub reason: String,
    pub expires: Option<u64>,
    pub approver: String,
}

/// Rule engine implementation
#[wasm_bindgen]
pub struct RuleEngine {
    rule_sets: Vec<RuleSet>,
    active_rules: HashMap<String, RuleDefinition>,
    policies: Vec<OrganizationalPolicy>,
}

#[wasm_bindgen]
impl RuleEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            rule_sets: Vec::new(),
            active_rules: HashMap::new(),
            policies: Vec::new(),
        }
    }

    /// Load a rule set from JSON string
    #[wasm_bindgen]
    pub fn load_rule_set(&mut self, rule_set_json: &str) -> Result<(), JsValue> {
        let rule_set: RuleSet = serde_json::from_str(rule_set_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse rule set: {}", e)))?;

        self.add_rule_set(rule_set);
        Ok(())
    }

    /// Load a rule set from TOML string
    #[wasm_bindgen]
    pub fn load_rule_set_toml(&mut self, rule_set_toml: &str) -> Result<(), JsValue> {
        let rule_set: RuleSet = toml::from_str(rule_set_toml)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse TOML rule set: {}", e)))?;

        self.add_rule_set(rule_set);
        Ok(())
    }

    /// Load a rule set from YAML string
    #[wasm_bindgen]
    pub fn load_rule_set_yaml(&mut self, rule_set_yaml: &str) -> Result<(), JsValue> {
        let rule_set: RuleSet = serde_yaml::from_str(rule_set_yaml)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse YAML rule set: {}", e)))?;

        self.add_rule_set(rule_set);
        Ok(())
    }

    /// Evaluate rules against code
    #[wasm_bindgen]
    pub fn evaluate_rules(&self, code: &str, language: &str, file_path: &str) -> Result<JsValue, JsValue> {
        let mut matches = Vec::new();

        for rule in self.active_rules.values() {
            // Check if rule applies to this language
            if !rule.languages.contains(&language.to_string()) && !rule.languages.contains(&"*".to_string()) {
                continue;
            }

            if !rule.enabled {
                continue;
            }

            // Apply the rule based on its pattern type
            let rule_matches = match rule.pattern_type {
                PatternType::Regex => self.apply_regex_rule(rule, code, file_path)?,
                PatternType::TreeSitterQuery => self.apply_tree_sitter_rule(rule, code, language, file_path)?,
                PatternType::AstPattern => self.apply_ast_pattern_rule(rule, code, file_path)?,
                PatternType::StructuralPattern => self.apply_structural_pattern_rule(rule, code, file_path)?,
            };

            matches.extend(rule_matches);
        }

        Ok(JsValue::from_serde(&matches).unwrap())
    }

    /// Check policy violations
    #[wasm_bindgen]
    pub fn check_policy(&self, policy_id: &str, code: &str, language: &str, file_path: &str) -> Result<JsValue, JsValue> {
        let policy = self.policies.iter()
            .find(|p| p.id == policy_id)
            .ok_or_else(|| JsValue::from_str(&format!("Policy not found: {}", policy_id)))?;

        // Check if file is in scope
        if !self.is_file_in_scope(file_path, &policy.scope) {
            return Ok(JsValue::from_serde(&Vec::<PolicyViolation>::new()).unwrap());
        }

        let mut violations = Vec::new();

        // Evaluate applicable rules
        for rule_id in &policy.rules {
            if let Some(rule) = self.active_rules.get(rule_id) {
                if !rule.languages.contains(&language.to_string()) && !rule.languages.contains(&"*".to_string()) {
                    continue;
                }

                // Check for exemptions
                if self.is_exempted(file_path, rule_id, &policy.exceptions) {
                    continue;
                }

                // Apply rule and convert matches to violations
                let rule_matches = match rule.pattern_type {
                    PatternType::Regex => self.apply_regex_rule(rule, code, file_path)?,
                    PatternType::TreeSitterQuery => self.apply_tree_sitter_rule(rule, code, language, file_path)?,
                    PatternType::AstPattern => self.apply_ast_pattern_rule(rule, code, file_path)?,
                    PatternType::StructuralPattern => self.apply_structural_pattern_rule(rule, code, file_path)?,
                };

                for rule_match in rule_matches {
                    for span in rule_match.spans {
                        violations.push(PolicyViolation {
                            rule_id: rule_id.clone(),
                            policy_id: policy_id.to_string(),
                            severity: rule.severity.clone(),
                            message: rule.message.clone(),
                            file: file_path.to_string(),
                            span,
                            remediation: rule.suggestion.clone(),
                            exemption: None,
                        });
                    }
                }
            }
        }

        Ok(JsValue::from_serde(&violations).unwrap())
    }

    /// Validate rule syntax
    #[wasm_bindgen]
    pub fn validate_rule(&self, rule_json: &str) -> Result<(), JsValue> {
        let rule: RuleDefinition = serde_json::from_str(rule_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid rule JSON: {}", e)))?;

        // Validate rule fields
        if rule.id.is_empty() {
            return Err(JsValue::from_str("Rule ID cannot be empty"));
        }

        if rule.pattern.is_empty() {
            return Err(JsValue::from_str("Rule pattern cannot be empty"));
        }

        // Validate pattern based on type
        match rule.pattern_type {
            PatternType::Regex => {
                regex::Regex::new(&rule.pattern)
                    .map_err(|e| JsValue::from_str(&format!("Invalid regex pattern: {}", e)))?;
            }
            PatternType::TreeSitterQuery => {
                // Tree-sitter query validation would go here
                // For now, just check it's not empty
                if rule.pattern.trim().is_empty() {
                    return Err(JsValue::from_str("Tree-sitter query cannot be empty"));
                }
            }
            PatternType::AstPattern | PatternType::StructuralPattern => {
                // Additional validation for AST patterns would go here
            }
        }

        Ok(())
    }

    /// Get statistics about loaded rules
    #[wasm_bindgen]
    pub fn get_statistics(&self) -> JsValue {
        let stats = HashMap::from([
            ("total_rule_sets", self.rule_sets.len() as f64),
            ("active_rules", self.active_rules.len() as f64),
            ("policies", self.policies.len() as f64),
            ("enabled_rules", self.active_rules.values().filter(|r| r.enabled).count() as f64),
        ]);

        JsValue::from_serde(&stats).unwrap()
    }
}

impl RuleEngine {
    fn add_rule_set(&mut self, mut rule_set: RuleSet) {
        // Apply overrides
        for rule in &mut rule_set.rules {
            for override_rule in &rule_set.overrides {
                if override_rule.rule_id == rule.id {
                    if let Some(enabled) = override_rule.enabled {
                        rule.enabled = enabled;
                    }
                    if let Some(ref severity) = override_rule.severity {
                        rule.severity = severity.clone();
                    }
                    if let Some(ref message) = override_rule.message {
                        rule.message = message.clone();
                    }
                    if let Some(ref languages) = override_rule.languages {
                        rule.languages = languages.clone();
                    }
                }
            }
        }

        // Add rules to active set
        for rule in rule_set.rules.clone() {
            self.active_rules.insert(rule.id.clone(), rule);
        }

        self.rule_sets.push(rule_set);
    }

    fn apply_regex_rule(&self, rule: &RuleDefinition, code: &str, file_path: &str) -> Result<Vec<RuleMatch>, JsValue> {
        let regex = regex::Regex::new(&rule.pattern)
            .map_err(|e| JsValue::from_str(&format!("Invalid regex: {}", e)))?;

        let mut matches = Vec::new();

        for (line_num, line) in code.lines().enumerate() {
            for mat in regex.find_iter(line) {
                let span = TextSpan {
                    start_line: line_num as u32 + 1,
                    start_column: mat.start() as u32,
                    end_line: line_num as u32 + 1,
                    end_column: mat.end() as u32,
                    byte_start: mat.start() as u32,
                    byte_end: mat.end() as u32,
                };

                let mut captured_groups = HashMap::new();
                for cap in regex.captures_iter(line) {
                    if let Some(matched) = cap.get(0) {
                        captured_groups.insert("full_match".to_string(), matched.as_str().to_string());
                    }
                }

                matches.push(RuleMatch {
                    rule_id: rule.id.clone(),
                    spans: vec![span],
                    captured_groups,
                    confidence: 1.0,
                    context: HashMap::from([
                        ("file".to_string(), file_path.to_string()),
                        ("line".to_string(), line.to_string()),
                    ]),
                });
            }
        }

        Ok(matches)
    }

    fn apply_tree_sitter_rule(&self, rule: &RuleDefinition, _code: &str, _language: &str, _file_path: &str) -> Result<Vec<RuleMatch>, JsValue> {
        // Tree-sitter implementation would go here
        // For now, return empty matches
        Ok(Vec::new())
    }

    fn apply_ast_pattern_rule(&self, rule: &RuleDefinition, _code: &str, _file_path: &str) -> Result<Vec<RuleMatch>, JsValue> {
        // AST pattern implementation would go here
        // For now, return empty matches
        Ok(Vec::new())
    }

    fn apply_structural_pattern_rule(&self, rule: &RuleDefinition, _code: &str, _file_path: &str) -> Result<Vec<RuleMatch>, JsValue> {
        // Structural pattern implementation would go here
        // For now, return empty matches
        Ok(Vec::new())
    }

    fn is_file_in_scope(&self, file_path: &str, scope: &PolicyScope) -> bool {
        // Check include patterns
        if !scope.file_patterns.is_empty() {
            let included = scope.file_patterns.iter().any(|pattern| {
                // Simple glob-like matching
                file_path.contains(pattern) || pattern == "*"
            });
            if !included {
                return false;
            }
        }

        // Check exclude patterns
        if scope.exclude_patterns.iter().any(|pattern| file_path.contains(pattern)) {
            return false;
        }

        // Check directories
        if !scope.directories.is_empty() {
            let in_directory = scope.directories.iter().any(|dir| file_path.starts_with(dir));
            if !in_directory {
                return false;
            }
        }

        true
    }

    fn is_exempted(&self, _file_path: &str, _rule_id: &str, _exceptions: &[PolicyException]) -> bool {
        // Exemption checking logic would go here
        // For now, no exemptions
        false
    }
}