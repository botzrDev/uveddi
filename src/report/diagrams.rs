//! Mermaid.js diagram generation for code dependencies

use crate::analysis::dependency::Dependency;
use crate::models::ArchitecturalIssue;
use std::collections::HashSet;

/// Generates a Mermaid.js compatible graph representation of code dependencies
pub fn generate_mermaid_diagram(deps: &[Dependency], issues: &[ArchitecturalIssue]) -> String {
    let mut output = String::from("```mermaid\ngraph TD;\n");
    let mut processed_nodes = HashSet::new();

    for dep in deps {
        if processed_nodes.insert(&dep.from) {
            output.push_str(&format!("    {}[{}];\n", dep.from.id, dep.from.name));
        }
        if processed_nodes.insert(&dep.to) {
            output.push_str(&format!("    {}[{}];\n", dep.to.id, dep.to.name));
        }
        output.push_str(&format!("    {} --> {};\n", dep.from.id, dep.to.id));
    }

    // Highlight nodes with issues
    for issue in issues {
        if let Some(file_id) = &issue.file_id {
            output.push_str(&format!("    {}[\"{}\" class=\"issue\"];\n", file_id, file_id));
        }
    }
    output.push_str("classDef issue fill:#f96,stroke:#333,stroke-width:2px;\n");
    output.push_str("```\n");
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::dependency::{Dependency, Node};
    use crate::models::ArchitecturalIssue;

    #[test]
    fn test_generate_mermaid_diagram() {
        let deps = vec![
            Dependency {
                from: Node { id: "mod1".to_string(), name: "Module1".to_string() },
                to: Node { id: "mod2".to_string(), name: "Module2".to_string() },
                kind: "imports".to_string(),
            },
        ];
        let issues = vec![ArchitecturalIssue { file_id: Some("mod1".to_string()), ..Default::default() }];
        let diagram = generate_mermaid_diagram(&deps, &issues);
        assert!(diagram.contains("mod1 --> mod2"));
        assert!(diagram.contains("classDef issue"));
    }
}
