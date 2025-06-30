/// AntiPatternType represents a type of architectural anti-pattern.
/// This will be used for DSL-based anti-pattern specification and reporting.
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AntiPatternType {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub dsl_rule: Option<String>, // DSL rule for detection (optional for now)
    pub category: Option<String>,
}

impl AntiPatternType {
    pub fn new(id: i64, name: &str, description: &str, dsl_rule: Option<&str>, category: Option<&str>) -> Self {
        Self {
            id,
            name: name.to_string(),
            description: description.to_string(),
            dsl_rule: dsl_rule.map(|s| s.to_string()),
            category: category.map(|s| s.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_antipatterntype_serialization() {
        let ap = AntiPatternType::new(1, "God Object", "A class that does too much", Some("size > 1000"), Some("OO"));
        let json = serde_json::to_string(&ap).unwrap();
        let de: AntiPatternType = serde_json::from_str(&json).unwrap();
        assert_eq!(ap, de);
    }
}
