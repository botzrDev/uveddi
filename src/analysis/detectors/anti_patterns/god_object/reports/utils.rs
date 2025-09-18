//! Shared utilities for report formatting

use crate::database::models::ArchitecturalIssue;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Calculate severity distribution across issues
pub fn calculate_severity_distribution(issues: &[ArchitecturalIssue]) -> HashMap<String, usize> {
    let mut distribution = HashMap::new();
    for issue in issues {
        *distribution.entry(issue.severity.clone()).or_insert(0) += 1;
    }
    distribution
}

/// Get emoji icon for severity level
pub fn get_severity_icon(severity: &str) -> &'static str {
    match severity {
        "Critical" => "🔴",
        "High" => "🟠",
        "Medium" => "🟡",
        "Low" => "🔵",
        _ => "⚪",
    }
}

/// Extract class name from issue description
pub fn extract_class_name(description: &str) -> String {
    // Look for pattern: "God Object detected: 'ClassName'"
    if let Some(start) = description.find("'") {
        if let Some(end) = description[start + 1..].find("'") {
            return description[start + 1..start + 1 + end].to_string();
        }
    }
    "Unknown".to_string()
}

/// Extract metrics information from description
pub fn extract_metrics_from_description(description: &str) -> Value {
    let mut metrics = json!({});

    // Extract method count
    if let Some(methods_start) = description.find(" has ") {
        if let Some(methods_end) = description[methods_start..].find(" methods") {
            let methods_text = &description[methods_start + 5..methods_start + methods_end];
            if let Ok(count) = methods_text.parse::<u32>() {
                metrics["method_count"] = json!(count);
            }
        }
    }

    // Extract field count
    if let Some(fields_start) = description.find(" and ") {
        if let Some(fields_end) = description[fields_start..].find(" fields") {
            let fields_text = &description[fields_start + 5..fields_start + fields_end];
            if let Ok(count) = fields_text.parse::<u32>() {
                metrics["field_count"] = json!(count);
            }
        }
    }

    // Extract LCOM4 score if present
    if let Some(lcom4_start) = description.find("LCOM4 score: ") {
        if let Some(lcom4_end) = description[lcom4_start + 13..].find(" ") {
            let lcom4_text = &description[lcom4_start + 13..lcom4_start + 13 + lcom4_end];
            if let Ok(score) = lcom4_text.parse::<u32>() {
                metrics["lcom4_score"] = json!(score);
            }
        }
    }

    metrics
}

/// Generate specific recommendations based on the issue
pub fn generate_recommendations(description: &str) -> String {
    let mut recommendations = Vec::new();

    // Check severity and metrics to provide tailored advice
    if description.contains("Critical") {
        recommendations.push("🚨 **Immediate Action Required**: This class requires urgent refactoring");
        recommendations.push("📦 **Extract Multiple Classes**: Break this into 3-5 smaller, focused classes");
        recommendations.push("🎯 **Identify Core Responsibilities**: List all responsibilities and group related ones");
    } else if description.contains("High") {
        recommendations.push("⚠️ **High Priority**: Schedule refactoring in the next sprint");
        recommendations.push("📦 **Extract Classes**: Identify 2-3 separate concerns that can be extracted");
    } else {
        recommendations.push("📋 **Monitor**: Consider refactoring when making future changes to this class");
        recommendations.push("🔍 **Review**: Ensure new methods have a clear reason to belong in this class");
    }

    // Add specific recommendations based on metrics
    if description.contains("methods") {
        recommendations.push("🔧 **Extract Methods**: Move related methods to new utility classes or services");
    }

    if description.contains("fields") {
        recommendations.push("📊 **Group Related Fields**: Create value objects or data structures for related fields");
    }

    if description.contains("LCOM4") {
        recommendations.push("🔗 **Improve Cohesion**: Methods should work with related fields and call each other");
    }

    // Add language-specific recommendations
    if description.contains(".rs") || description.contains("struct") {
        recommendations.push("🦀 **Rust-specific**: Consider using composition with traits instead of large impl blocks");
    } else if description.contains(".py") || description.contains("class") {
        recommendations.push("🐍 **Python-specific**: Use mixins or composition to break down responsibilities");
    } else if description.contains(".js") || description.contains(".ts") {
        recommendations.push("📜 **JS/TS-specific**: Consider using composition, modules, or the strategy pattern");
    }

    recommendations.join("\n")
}