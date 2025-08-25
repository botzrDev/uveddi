use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export commonly used types
pub use wasm_bindgen::JsValue;
pub use js_sys::{Array, Object, Reflect, JSON};

// Import rule engine module
mod rule_engine;
pub use rule_engine::*;

/// Main plugin struct
#[wasm_bindgen]
pub struct UveddiRuleEngine {
    config: PluginConfig,
    rule_engine: RuleEngine,
    state: HashMap<String, String>,
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub severity_threshold: f32,
    pub max_issues_per_file: u32,
    pub custom_settings: HashMap<String, String>,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            severity_threshold: 0.5,
            max_issues_per_file: 100,
            custom_settings: HashMap::new(),
        }
    }
}

/// Analysis result returned by the plugin
#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub issues: Vec<Issue>,
    pub metrics: Metrics,
}

/// Represents an issue found by the plugin
#[derive(Debug, Serialize, Deserialize)]
pub struct Issue {
    pub id: String,
    pub severity: String,
    pub message: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub category: String,
    pub suggestion: Option<String>,
}

/// Code metrics calculated by the plugin
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Metrics {
    pub lines_of_code: usize,
    pub complexity: usize,
    pub maintainability_index: f64,
    pub custom_metrics: HashMap<String, f64>,
}

#[wasm_bindgen]
impl UveddiRuleEngine {
    /// Create a new plugin instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // Initialize logging for the plugin
        console_error_panic_hook::set_once();

        Self {
            config: PluginConfig::default(),
            rule_engine: RuleEngine::new(),
            state: HashMap::new(),
        }
    }

    /// Initialize the plugin with configuration
    #[wasm_bindgen]
    pub fn initialize(&mut self, config_json: &str) -> Result<(), JsValue> {
        self.config = serde_json::from_str(config_json)
            .map_err(|e| JsValue::from_str(&format!("Config parse error: {}", e)))?;
        
        log(&format!("uveddi-rule-engine plugin initialized"));
        Ok(())
    }

    /// Main analysis function
    #[wasm_bindgen]
    pub fn analyze(&self, code: &str, language: &str, file_path: &str) -> Result<JsValue, JsValue> {
        let mut issues = Vec::new();
        let mut metrics = Metrics::default();

        // Basic metrics calculation
        metrics.lines_of_code = code.lines().count();
        
        // Use rule engine to evaluate rules
        let rule_matches = self.rule_engine.evaluate_rules(code, language, file_path)?;
        let matches: Vec<RuleMatch> = rule_matches.into_serde()
            .map_err(|e| JsValue::from_str(&format!("Failed to deserialize rule matches: {}", e)))?;

        // Convert rule matches to issues
        for rule_match in matches {
            for span in rule_match.spans {
                issues.push(Issue {
                    id: rule_match.rule_id.clone(),
                    severity: "medium".to_string(), // Default severity, should be from rule
                    message: rule_match.context.get("message")
                        .unwrap_or(&format!("Rule violation: {}", rule_match.rule_id))
                        .clone(),
                    file: file_path.to_string(),
                    line: span.start_line as usize,
                    column: span.start_column as usize,
                    category: "rule-violation".to_string(),
                    suggestion: rule_match.context.get("suggestion").cloned(),
                });
            }
        }

        // Add complexity calculation
        metrics.complexity = self.calculate_complexity(code);

        let result = AnalysisResult { issues, metrics };
        
        // Convert to JsValue for return
        JsValue::from_serde(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Get plugin metadata
    #[wasm_bindgen]
    pub fn get_info(&self) -> JsValue {
        let info = serde_json::json!({
            "name": "uveddi-rule-engine",
            "version": "1.0.0",
            "description": "Custom rule and policy engine for organizational coding standards",
            "author": "Uveddi Team"
        });

        JsValue::from_serde(&info).unwrap_or(JsValue::NULL)
    }

    /// Load rule set from JSON
    #[wasm_bindgen]
    pub fn load_rule_set(&mut self, rule_set_json: &str) -> Result<(), JsValue> {
        self.rule_engine.load_rule_set(rule_set_json)
    }

    /// Load rule set from TOML
    #[wasm_bindgen] 
    pub fn load_rule_set_toml(&mut self, rule_set_toml: &str) -> Result<(), JsValue> {
        self.rule_engine.load_rule_set_toml(rule_set_toml)
    }

    /// Load rule set from YAML
    #[wasm_bindgen]
    pub fn load_rule_set_yaml(&mut self, rule_set_yaml: &str) -> Result<(), JsValue> {
        self.rule_engine.load_rule_set_yaml(rule_set_yaml)
    }

    /// Check policy violations
    #[wasm_bindgen]
    pub fn check_policy(&self, policy_id: &str, code: &str, language: &str, file_path: &str) -> Result<JsValue, JsValue> {
        self.rule_engine.check_policy(policy_id, code, language, file_path)
    }

    /// Validate rule syntax
    #[wasm_bindgen]
    pub fn validate_rule(&self, rule_json: &str) -> Result<(), JsValue> {
        self.rule_engine.validate_rule(rule_json)
    }

    /// Get rule engine statistics
    #[wasm_bindgen]
    pub fn get_rule_statistics(&self) -> JsValue {
        self.rule_engine.get_statistics()
    }

    /// Cleanup resources
    #[wasm_bindgen]
    pub fn cleanup(&mut self) -> Result<(), JsValue> {
        self.state.clear();
        log(&format!("uveddi-rule-engine plugin cleaned up"));
        Ok(())
    }
}

impl UveddiRuleEngine {
    /// Find line number of a pattern in code
    fn find_line_number(&self, code: &str, pattern: &str) -> usize {
        code.lines()
            .position(|line| line.contains(pattern))
            .map(|pos| pos + 1)
            .unwrap_or(0)
    }

    /// Calculate basic complexity metrics
    fn calculate_complexity(&self, code: &str) -> usize {
        // Simple cyclomatic complexity approximation
        let keywords = ["if", "else", "while", "for", "match", "case", "catch"];
        let mut complexity = 1; // Base complexity

        for keyword in &keywords {
            complexity += code.matches(keyword).count();
        }

        complexity
    }
}

// Host function bindings
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    
    #[wasm_bindgen(js_namespace = console)]
    fn warn(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);
}

// Plugin panic hook for better error reporting
#[cfg(feature = "console_error_panic_hook")]
extern crate console_error_panic_hook;

// Export plugin metadata for host discovery
#[wasm_bindgen(start)]
pub fn main() {
    log("uveddi-rule-engine plugin loaded");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = UveddiRuleEngine::new();
        assert_eq!(plugin.config.severity_threshold, 0.5);
    }

    #[test]
    fn test_line_number_finding() {
        let plugin = UveddiRuleEngine::new();
        let code = "fn main() {\n    // TODO: implement\n    println!(\"Hello\");\n}";
        
        assert_eq!(plugin.find_line_number(code, "TODO"), 2);
        assert_eq!(plugin.find_line_number(code, "nonexistent"), 0);
    }

    #[test]
    fn test_complexity_calculation() {
        let plugin = UveddiRuleEngine::new();
        let simple_code = "fn hello() { println!(\"hello\"); }";
        let complex_code = "fn test() { if x { if y { while z { for i in items { } } } } }";
        
        assert_eq!(plugin.calculate_complexity(simple_code), 1);
        assert!(plugin.calculate_complexity(complex_code) > 1);
    }
}