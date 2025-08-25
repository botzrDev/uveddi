use uveddi_rule_engine::*;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_rule_set() -> String {
        serde_json::json!({
            "name": "test-rules",
            "version": "1.0.0",
            "description": "Test rule set",
            "author": "Test",
            "format": "json",
            "rules": [
                {
                    "id": "test-todo",
                    "name": "Test TODO",
                    "description": "Find TODO comments",
                    "pattern": "(?i)todo",
                    "pattern_type": "regex",
                    "severity": "warning",
                    "category": "maintenance",
                    "message": "TODO found",
                    "languages": ["*"],
                    "enabled": true,
                    "tags": ["test"],
                    "metadata": {}
                },
                {
                    "id": "test-console-log", 
                    "name": "Test Console Log",
                    "description": "Find console.log",
                    "pattern": "console\\.log",
                    "pattern_type": "regex",
                    "severity": "warning",
                    "category": "debug",
                    "message": "Console log found",
                    "languages": ["javascript", "typescript"],
                    "enabled": true,
                    "tags": ["test"],
                    "metadata": {}
                }
            ],
            "extends": [],
            "overrides": []
        }).to_string()
    }

    #[test]
    fn test_rule_engine_creation() {
        let engine = RuleEngine::new();
        let stats = engine.get_statistics();
        // Should start with no rules
        assert!(stats.as_string().unwrap().contains("\"active_rules\":0"));
    }

    #[test]
    fn test_load_rule_set() {
        let mut engine = RuleEngine::new();
        let rule_set_json = create_test_rule_set();
        
        let result = engine.load_rule_set(&rule_set_json);
        assert!(result.is_ok());
        
        let stats = engine.get_statistics();
        let stats_str = stats.as_string().unwrap();
        assert!(stats_str.contains("\"active_rules\":2"));
    }

    #[test]
    fn test_rule_validation() {
        let engine = RuleEngine::new();
        
        // Valid rule
        let valid_rule = serde_json::json!({
            "id": "test-rule",
            "name": "Test Rule",
            "description": "Test",
            "pattern": "test",
            "pattern_type": "regex",
            "severity": "warning",
            "category": "test",
            "message": "Test message",
            "languages": ["*"],
            "enabled": true,
            "tags": [],
            "metadata": {}
        }).to_string();
        
        assert!(engine.validate_rule(&valid_rule).is_ok());
        
        // Invalid rule (empty ID)
        let invalid_rule = serde_json::json!({
            "id": "",
            "name": "Test Rule",
            "description": "Test",
            "pattern": "test",
            "pattern_type": "regex",
            "severity": "warning",
            "category": "test",
            "message": "Test message",
            "languages": ["*"],
            "enabled": true,
            "tags": [],
            "metadata": {}
        }).to_string();
        
        assert!(engine.validate_rule(&invalid_rule).is_err());
    }

    #[test]
    fn test_regex_rule_matching() {
        let mut engine = RuleEngine::new();
        let rule_set_json = create_test_rule_set();
        engine.load_rule_set(&rule_set_json).unwrap();
        
        let code = "fn main() {\n    // TODO: implement this\n    println!(\"Hello\");\n}";
        let result = engine.evaluate_rules(code, "rust", "test.rs");
        
        assert!(result.is_ok());
        let matches: Vec<RuleMatch> = result.unwrap().into_serde().unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].rule_id, "test-todo");
    }

    #[test]
    fn test_language_filtering() {
        let mut engine = RuleEngine::new();
        let rule_set_json = create_test_rule_set();
        engine.load_rule_set(&rule_set_json).unwrap();
        
        // JavaScript code with console.log - should match
        let js_code = "console.log('hello');";
        let js_result = engine.evaluate_rules(js_code, "javascript", "test.js");
        let js_matches: Vec<RuleMatch> = js_result.unwrap().into_serde().unwrap();
        
        // Should find console.log rule
        assert!(js_matches.iter().any(|m| m.rule_id == "test-console-log"));
        
        // Same code in Rust - should not match console.log rule (language filter)
        let rust_result = engine.evaluate_rules(js_code, "rust", "test.rs");
        let rust_matches: Vec<RuleMatch> = rust_result.unwrap().into_serde().unwrap();
        
        // Should not find console.log rule for Rust
        assert!(!rust_matches.iter().any(|m| m.rule_id == "test-console-log"));
    }

    #[test]
    fn test_invalid_regex_pattern() {
        let engine = RuleEngine::new();
        
        let invalid_regex_rule = serde_json::json!({
            "id": "bad-regex",
            "name": "Bad Regex",
            "description": "Test",
            "pattern": "[unclosed",
            "pattern_type": "regex",
            "severity": "warning",
            "category": "test",
            "message": "Test",
            "languages": ["*"],
            "enabled": true,
            "tags": [],
            "metadata": {}
        }).to_string();
        
        assert!(engine.validate_rule(&invalid_regex_rule).is_err());
    }

    #[wasm_bindgen_test]
    fn test_plugin_creation() {
        let plugin = UveddiRuleEngine::new();
        let info = plugin.get_info();
        assert!(!info.is_null());
    }

    #[wasm_bindgen_test]
    fn test_plugin_rule_loading() {
        let mut plugin = UveddiRuleEngine::new();
        let rule_set_json = create_test_rule_set();
        
        let result = plugin.load_rule_set(&rule_set_json);
        assert!(result.is_ok());
        
        let stats = plugin.get_rule_statistics();
        assert!(!stats.is_null());
    }

    #[wasm_bindgen_test]
    fn test_plugin_analysis() {
        let mut plugin = UveddiRuleEngine::new();
        let rule_set_json = create_test_rule_set();
        plugin.load_rule_set(&rule_set_json).unwrap();
        
        let code = "// TODO: implement feature\nfn main() {}";
        let result = plugin.analyze(code, "rust", "test.rs");
        
        assert!(result.is_ok());
        
        let analysis_result: AnalysisResult = result.unwrap().into_serde().unwrap();
        assert!(analysis_result.issues.len() > 0);
        assert_eq!(analysis_result.issues[0].id, "test-todo");
    }

    #[test]
    fn test_toml_rule_loading() {
        let mut engine = RuleEngine::new();
        let toml_content = r#"
[rule_set]
name = "test-toml"
version = "1.0.0"
description = "Test TOML rules"
author = "Test"
format = "toml"

[[rules]]
id = "test-toml-rule"
name = "Test TOML Rule"
description = "Test rule from TOML"
pattern = "test"
pattern_type = "regex"
severity = "info"
category = "test"
message = "Test from TOML"
languages = ["*"]
enabled = true
tags = []

[rules.metadata]
"#;
        
        let result = engine.load_rule_set_toml(toml_content);
        assert!(result.is_ok());
        
        let stats = engine.get_statistics();
        let stats_str = stats.as_string().unwrap();
        assert!(stats_str.contains("\"active_rules\":1"));
    }

    #[test]
    fn test_yaml_rule_loading() {
        let mut engine = RuleEngine::new();
        let yaml_content = r#"
rule_set:
  name: "test-yaml"
  version: "1.0.0"
  description: "Test YAML rules"
  author: "Test"
  format: "yaml"
rules:
  - id: "test-yaml-rule"
    name: "Test YAML Rule"
    description: "Test rule from YAML"
    pattern: "test"
    pattern_type: "regex"
    severity: "info"
    category: "test"
    message: "Test from YAML"
    languages: ["*"]
    enabled: true
    tags: []
    metadata: {}
extends: []
overrides: []
"#;
        
        let result = engine.load_rule_set_yaml(yaml_content);
        assert!(result.is_ok());
        
        let stats = engine.get_statistics();
        let stats_str = stats.as_string().unwrap();
        assert!(stats_str.contains("\"active_rules\":1"));
    }
}