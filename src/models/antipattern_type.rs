/// AntiPatternType represents a type of architectural anti-pattern.
/// This will be used for DSL-based anti-pattern specification and reporting.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AntiPatternType {
    /// Unique identifier for the anti-pattern type.
    pub id: i64,
    /// Name of the anti-pattern.
    pub name: String,
    /// Description of the anti-pattern.
    pub description: String,
    /// DSL rule for detection (optional for now)
    pub dsl_rule: Option<String>,
    /// Optional category for classification.
    pub category: Option<String>,
}

impl AntiPatternType {
    /// Creates a new `AntiPatternType` instance.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the anti-pattern type.
    /// * `name` - Name of the anti-pattern.
    /// * `description` - Description of the anti-pattern.
    /// * `dsl_rule` - Optional DSL rule for detection.
    /// * `category` - Optional category for classification.
    ///
    /// # Returns
    ///
    /// * `AntiPatternType` - The constructed anti-pattern type.
    ///
    /// # Example
    /// ```rust
    /// use uveddi::models::antipattern_type::AntiPatternType;
    /// let ap = AntiPatternType::new(1, "God Object", "A class that does too much", None, Some("OO"));
    /// ```
    pub fn new(
        id: i64,
        name: &str,
        description: &str,
        dsl_rule: Option<&str>,
        category: Option<&str>,
    ) -> Self {
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
    /// Tests serialization and deserialization of AntiPatternType.
    #[test]
    fn serialization_roundtrip_preserves_antipattern_type() {
        let ap = AntiPatternType::new(
            1,
            "God Object",
            "A class that does too much",
            Some("size > 1000"),
            Some("OO"),
        );
        let json = serde_json::to_string(&ap).unwrap();
        let de: AntiPatternType = serde_json::from_str(&json).unwrap();
        assert_eq!(ap, de);
    }
}
