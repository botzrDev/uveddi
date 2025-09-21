//! Environment variable parsing functionality
//!
//! This module handles parsing and syntax validation for environment
//! variable files (.env) with proper error handling.

use crate::analysis::AnalysisError;

/// Environment variable parser
pub struct EnvParser;

impl EnvParser {
    /// Parse a single environment variable line
    pub fn parse_env_line(line: &str) -> Option<(String, String)> {
        let line = line.trim();

        if let Some(equals_pos) = line.find('=') {
            let key = line[..equals_pos].trim().to_string();
            let value = line[equals_pos + 1..].trim();

            // Handle quoted values
            let value = if (value.starts_with('"') && value.ends_with('"'))
                || (value.starts_with('\'') && value.ends_with('\''))
            {
                value[1..value.len() - 1].to_string()
            } else {
                value.to_string()
            };

            Some((key, value))
        } else {
            None
        }
    }

    /// Validate syntax of environment variable content
    pub fn validate_syntax(content: &str) -> Result<(), AnalysisError> {
        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if !line.contains('=') {
                return Err(AnalysisError::ParseError(format!(
                    "Invalid environment variable syntax at line {}: {}",
                    line_num + 1,
                    line
                )));
            }

            // Validate key format (should be valid identifier)
            if let Some((key, _)) = Self::parse_env_line(line) {
                if !Self::is_valid_env_key(&key) {
                    return Err(AnalysisError::ParseError(format!(
                        "Invalid environment variable name at line {}: {}",
                        line_num + 1,
                        key
                    )));
                }
            }
        }
        Ok(())
    }

    /// Check if environment variable key is valid
    pub fn is_valid_env_key(key: &str) -> bool {
        if key.is_empty() {
            return false;
        }

        // Must start with letter or underscore
        if !key.chars().next().unwrap().is_ascii_alphabetic() && !key.starts_with('_') {
            return false;
        }

        // Can only contain alphanumeric characters and underscores
        key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    }

    /// Extract all environment variables from content
    pub fn extract_all_vars(content: &str) -> Vec<(String, String, usize)> {
        let mut vars = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = Self::parse_env_line(line) {
                vars.push((key, value, line_num + 1));
            }
        }

        vars
    }

    /// Check for duplicate variable definitions
    pub fn find_duplicates(content: &str) -> Vec<(String, Vec<usize>)> {
        let mut var_lines: std::collections::HashMap<String, Vec<usize>> =
            std::collections::HashMap::new();

        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, _)) = Self::parse_env_line(line) {
                var_lines
                    .entry(key)
                    .or_insert_with(Vec::new)
                    .push(line_num + 1);
            }
        }

        var_lines
            .into_iter()
            .filter(|(_, lines)| lines.len() > 1)
            .collect()
    }

    /// Normalize environment variable value (remove quotes, handle escaping)
    pub fn normalize_value(value: &str) -> String {
        let mut result = String::new();
        let mut chars = value.chars().peekable();
        let mut in_quotes = false;
        let mut quote_char = '\0';

        while let Some(ch) = chars.next() {
            match ch {
                '"' | '\'' if !in_quotes => {
                    in_quotes = true;
                    quote_char = ch;
                }
                ch if in_quotes && ch == quote_char => {
                    in_quotes = false;
                }
                '\\' if in_quotes => {
                    if let Some(next_ch) = chars.next() {
                        match next_ch {
                            'n' => result.push('\n'),
                            't' => result.push('\t'),
                            'r' => result.push('\r'),
                            '\\' => result.push('\\'),
                            '"' => result.push('"'),
                            '\'' => result.push('\''),
                            ch => {
                                result.push('\\');
                                result.push(ch);
                            }
                        }
                    }
                }
                ch => result.push(ch),
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_env_line() {
        assert_eq!(
            EnvParser::parse_env_line("KEY=value"),
            Some(("KEY".to_string(), "value".to_string()))
        );

        assert_eq!(
            EnvParser::parse_env_line("KEY=\"quoted value\""),
            Some(("KEY".to_string(), "quoted value".to_string()))
        );

        assert_eq!(
            EnvParser::parse_env_line("KEY='single quoted'"),
            Some(("KEY".to_string(), "single quoted".to_string()))
        );

        assert_eq!(EnvParser::parse_env_line("INVALID LINE"), None);
    }

    #[test]
    fn test_is_valid_env_key() {
        assert!(EnvParser::is_valid_env_key("VALID_KEY"));
        assert!(EnvParser::is_valid_env_key("_PRIVATE_KEY"));
        assert!(EnvParser::is_valid_env_key("KEY123"));

        assert!(!EnvParser::is_valid_env_key("123INVALID"));
        assert!(!EnvParser::is_valid_env_key("INVALID-KEY"));
        assert!(!EnvParser::is_valid_env_key(""));
    }

    #[test]
    fn test_validate_syntax() {
        let valid_content = r#"
# Comment
KEY1=value1
KEY2="quoted value"
KEY3='single quoted'
"#;
        assert!(EnvParser::validate_syntax(valid_content).is_ok());

        let invalid_content = "INVALID LINE WITHOUT EQUALS";
        assert!(EnvParser::validate_syntax(invalid_content).is_err());
    }

    #[test]
    fn test_find_duplicates() {
        let content = r#"
KEY1=value1
KEY2=value2
KEY1=duplicate
"#;
        let duplicates = EnvParser::find_duplicates(content);
        assert_eq!(duplicates.len(), 1);
        assert_eq!(duplicates[0].0, "KEY1");
        assert_eq!(duplicates[0].1, vec![2, 4]);
    }

    #[test]
    fn test_normalize_value() {
        assert_eq!(EnvParser::normalize_value("simple"), "simple");
        assert_eq!(EnvParser::normalize_value("\"quoted\""), "quoted");
        assert_eq!(EnvParser::normalize_value("'single'"), "single");
        assert_eq!(
            EnvParser::normalize_value("\"with\\nescapes\""),
            "with\nescapes"
        );
    }
}
