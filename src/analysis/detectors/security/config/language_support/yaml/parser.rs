//! YAML and JSON parsing functionality
//!
//! This module handles parsing and syntax validation for YAML and JSON
//! configuration files with proper error handling.

use crate::analysis::AnalysisError;
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;

/// YAML/JSON parser with error handling
pub struct YamlParser;

impl YamlParser {
    /// Parse content as YAML, falling back to JSON if YAML parsing fails
    pub fn parse_content(content: &str) -> Result<YamlValue, AnalysisError> {
        // Try YAML first (which can also handle JSON)
        match serde_yaml::from_str::<YamlValue>(content) {
            Ok(yaml_value) => Ok(yaml_value),
            Err(yaml_err) => {
                // If YAML parsing fails, try JSON
                match serde_json::from_str::<JsonValue>(content) {
                    Ok(json_value) => Self::json_to_yaml(json_value),
                    Err(_json_err) => Err(AnalysisError::ParseError {
                        message: format!("Invalid YAML/JSON: {}", yaml_err),
                    }),
                }
            }
        }
    }

    /// Validate syntax without full parsing
    pub fn validate_syntax(content: &str) -> Result<(), AnalysisError> {
        if serde_yaml::from_str::<YamlValue>(content).is_ok() {
            Ok(())
        } else if serde_json::from_str::<JsonValue>(content).is_ok() {
            Ok(())
        } else {
            Err(AnalysisError::ParseError {
                message: "Invalid YAML/JSON syntax".to_string(),
            })
        }
    }

    /// Convert JSON value to YAML value
    pub fn json_to_yaml(json_value: JsonValue) -> Result<YamlValue, AnalysisError> {
        match json_value {
            JsonValue::Null => Ok(YamlValue::Null),
            JsonValue::Bool(b) => Ok(YamlValue::Bool(b)),
            JsonValue::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(YamlValue::Number(serde_yaml::Number::from(i)))
                } else if let Some(u) = n.as_u64() {
                    Ok(YamlValue::Number(serde_yaml::Number::from(u)))
                } else if let Some(f) = n.as_f64() {
                    Ok(YamlValue::Number(serde_yaml::Number::from(f)))
                } else {
                    Err(AnalysisError::ParseError {
                        message: "Invalid number format".to_string(),
                    })
                }
            }
            JsonValue::String(s) => Ok(YamlValue::String(s)),
            JsonValue::Array(arr) => {
                let yaml_seq: Result<Vec<YamlValue>, AnalysisError> =
                    arr.into_iter().map(Self::json_to_yaml).collect();
                Ok(YamlValue::Sequence(yaml_seq?))
            }
            JsonValue::Object(obj) => {
                let mut yaml_map = serde_yaml::Mapping::new();
                for (key, value) in obj {
                    let yaml_key = YamlValue::String(key);
                    let yaml_value = Self::json_to_yaml(value)?;
                    yaml_map.insert(yaml_key, yaml_value);
                }
                Ok(YamlValue::Mapping(yaml_map))
            }
        }
    }

    /// Get a value from a mapping by checking multiple possible keys
    pub fn get_mapping_value<'a>(
        map: &'a serde_yaml::Mapping,
        keys: &[&str],
    ) -> Option<&'a YamlValue> {
        for &key in keys {
            if let Some(value) = map.get(&YamlValue::String(key.to_string())) {
                return Some(value);
            }
            // Also try lowercase
            if let Some(value) = map.get(&YamlValue::String(key.to_lowercase())) {
                return Some(value);
            }
        }
        None
    }

    /// Check if a string value looks like a credential
    pub fn looks_like_credential(value: &str) -> bool {
        // Skip obvious non-credentials
        if value.is_empty() || value.len() < 6 {
            return false;
        }

        // Skip common test/placeholder values
        let test_values = [
            "test",
            "example",
            "demo",
            "placeholder",
            "xxx",
            "***",
            "changeme",
            "password",
            "secret",
            "token",
            "key",
        ];

        let value_lower = value.to_lowercase();
        if test_values.iter().any(|&test| value_lower.contains(test)) {
            return false;
        }

        // Skip values that are all the same character
        if value.chars().all(|c| c == value.chars().next().unwrap()) {
            return false;
        }

        // Look for characteristics of real credentials
        let has_mixed_case =
            value.chars().any(|c| c.is_lowercase()) && value.chars().any(|c| c.is_uppercase());
        let has_numbers = value.chars().any(|c| c.is_numeric());
        let has_special = value.chars().any(|c| !c.is_alphanumeric());
        let reasonable_length = value.len() >= 8 && value.len() <= 128;

        // If it has good entropy characteristics, likely a credential
        (has_mixed_case || has_numbers || has_special) && reasonable_length
    }

    /// Extract path components from a nested YAML structure
    pub fn extract_path_components(value: &YamlValue, current_path: &str) -> Vec<String> {
        let mut paths = Vec::new();

        match value {
            YamlValue::Mapping(map) => {
                for (key, val) in map {
                    if let Some(key_str) = key.as_str() {
                        let new_path = if current_path.is_empty() {
                            key_str.to_string()
                        } else {
                            format!("{}.{}", current_path, key_str)
                        };
                        paths.push(new_path.clone());
                        paths.extend(Self::extract_path_components(val, &new_path));
                    }
                }
            }
            YamlValue::Sequence(seq) => {
                for (index, item) in seq.iter().enumerate() {
                    let new_path = if current_path.is_empty() {
                        format!("[{}]", index)
                    } else {
                        format!("{}[{}]", current_path, index)
                    };
                    paths.push(new_path.clone());
                    paths.extend(Self::extract_path_components(item, &new_path));
                }
            }
            _ => {}
        }

        paths
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yaml_parsing() {
        let yaml_content = r#"
database:
  host: localhost
  port: 5432
  ssl: true
"#;
        let result = YamlParser::parse_content(yaml_content);
        assert!(result.is_ok());
    }

    #[test]
    fn test_json_parsing() {
        let json_content = r#"
{
  "database": {
    "host": "localhost",
    "port": 5432,
    "ssl": true
  }
}
"#;
        let result = YamlParser::parse_content(json_content);
        assert!(result.is_ok());
    }

    #[test]
    fn test_credential_detection() {
        assert!(YamlParser::looks_like_credential("Xy9$kL2mN8pQ"));
        assert!(!YamlParser::looks_like_credential("test"));
        assert!(!YamlParser::looks_like_credential("password"));
        assert!(!YamlParser::looks_like_credential("xxx"));
        assert!(!YamlParser::looks_like_credential(""));
    }

    #[test]
    fn test_mapping_value_retrieval() {
        let yaml_content = r#"
ssl_enabled: true
SSL: false
"#;
        let parsed = YamlParser::parse_content(yaml_content).unwrap();
        if let YamlValue::Mapping(map) = parsed {
            let ssl_val = YamlParser::get_mapping_value(&map, &["ssl", "ssl_enabled"]);
            assert!(ssl_val.is_some());
            assert_eq!(ssl_val.unwrap().as_bool(), Some(true));
        }
    }

    #[test]
    fn test_syntax_validation() {
        let valid_yaml = "key: value";
        assert!(YamlParser::validate_syntax(valid_yaml).is_ok());

        let valid_json = r#"{"key": "value"}"#;
        assert!(YamlParser::validate_syntax(valid_json).is_ok());

        let invalid_content = "key: [unclosed";
        assert!(YamlParser::validate_syntax(invalid_content).is_err());
    }
}
