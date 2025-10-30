use serde::{Deserialize, Serialize};

/// Anti-pattern type definitions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AntiPatternType {
    pub anti_pattern_type_id: Option<i64>,
    pub name: String,
    pub description: String,
    pub category: String, // "structural", "behavioral", "creational"
}
