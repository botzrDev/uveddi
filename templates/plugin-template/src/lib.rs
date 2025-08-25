use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export commonly used types
pub use wasm_bindgen::JsValue;
pub use js_sys::{Array, Object, Reflect, JSON};

/// Main plugin struct
#[wasm_bindgen]
pub struct {{plugin_struct}} {
    config: PluginConfig,
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
impl {{plugin_struct}} {
    /// Create a new plugin instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // Initialize logging for the plugin
        console_error_panic_hook::set_once();

        Self {
            config: PluginConfig::default(),
            state: HashMap::new(),
        }
    }

    /// Initialize the plugin with configuration
    #[wasm_bindgen]
    pub fn initialize(&mut self, config_json: &str) -> Result<(), JsValue> {
        self.config = serde_json::from_str(config_json)
            .map_err(|e| JsValue::from_str(&format!("Config parse error: {}", e)))?;
        
        log(&format!("{{plugin_name}} plugin initialized"));
        Ok(())
    }

    /// Main analysis function
    #[wasm_bindgen]
    pub fn analyze(&self, code: &str, language: &str, file_path: &str) -> Result<JsValue, JsValue> {
        let mut issues = Vec::new();
        let mut metrics = Metrics::default();

        // Basic metrics calculation
        metrics.lines_of_code = code.lines().count();

        // TODO: Implement your analysis logic here
        // Example: Simple pattern detection
        if code.contains("TODO") {
            issues.push(Issue {
                id: "TODO-001".to_string(),
                severity: "info".to_string(),
                message: "TODO comment found".to_string(),
                file: file_path.to_string(),
                line: self.find_line_number(code, "TODO"),
                column: 0,
                category: "maintenance".to_string(),
                suggestion: Some("Consider completing or removing this TODO".to_string()),
            });
        }

        let result = AnalysisResult { issues, metrics };
        
        // Convert to JsValue for return
        JsValue::from_serde(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Get plugin metadata
    #[wasm_bindgen]
    pub fn get_info(&self) -> JsValue {
        let info = serde_json::json!({
            "name": "{{plugin_name}}",
            "version": "{{version}}",
            "description": "{{description}}",
            "author": "{{author}}"
        });

        JsValue::from_serde(&info).unwrap_or(JsValue::NULL)
    }

    /// Cleanup resources
    #[wasm_bindgen]
    pub fn cleanup(&mut self) -> Result<(), JsValue> {
        self.state.clear();
        log(&format!("{{plugin_name}} plugin cleaned up"));
        Ok(())
    }
}

impl {{plugin_struct}} {
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
    log("{{plugin_name}} plugin loaded");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = {{plugin_struct}}::new();
        assert_eq!(plugin.config.severity_threshold, 0.5);
    }

    #[test]
    fn test_line_number_finding() {
        let plugin = {{plugin_struct}}::new();
        let code = "fn main() {\n    // TODO: implement\n    println!(\"Hello\");\n}";
        
        assert_eq!(plugin.find_line_number(code, "TODO"), 2);
        assert_eq!(plugin.find_line_number(code, "nonexistent"), 0);
    }

    #[test]
    fn test_complexity_calculation() {
        let plugin = {{plugin_struct}}::new();
        let simple_code = "fn hello() { println!(\"hello\"); }";
        let complex_code = "fn test() { if x { if y { while z { for i in items { } } } } }";
        
        assert_eq!(plugin.calculate_complexity(simple_code), 1);
        assert!(plugin.calculate_complexity(complex_code) > 1);
    }
}