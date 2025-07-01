wit_bindgen::generate!({
    world: "plugin",
    path: "../../../interface/plugin.wit",
});

use exports::uveddi::plugins::plugin::Guest;
use uveddi::plugins::plugin::*;
use std::collections::HashMap;

struct GodObjectDetector;

impl Guest for GodObjectDetector {
    fn info() -> PluginInfo {
        PluginInfo {
            name: "WASM God Object Detector".to_string(),
            version: "0.1.0".to_string(),
            description: "Detects classes/modules that are too large (God Objects)".to_string(),
            author: "Uveddi Team".to_string(),
        }
    }
    
    fn analyze(dependencies: Vec<Dependency>) -> Vec<ArchitecturalIssue> {
        let mut issues = Vec::new();
        
        // Simple heuristic: if a module has more than 10 outgoing dependencies, flag it
        let mut dep_counts = HashMap::new();
        
        for dep in dependencies {
            *dep_counts.entry(dep.from_module.clone()).or_insert(0) += 1;
        }
        
        for (module, count) in dep_counts {
            if count > 10 {
                issues.push(ArchitecturalIssue {
                    file_path: module.clone(),
                    start_line: 1,
                    end_line: 1,
                    issue_type: "god_object".to_string(),
                    severity: "medium".to_string(),
                    message: format!("Module '{}' has {} dependencies, indicating it may be a God Object", module, count),
                    code_snippet: None,
                });
            }
        }
        
        // Also flag modules that appear in many dependencies (high fan-in)
        let mut fan_in_counts = HashMap::new();
        for dep in &dependencies {
            *fan_in_counts.entry(dep.to_module.clone()).or_insert(0) += 1;
        }
        
        for (module, count) in fan_in_counts {
            if count > 15 {
                issues.push(ArchitecturalIssue {
                    file_path: module.clone(),
                    start_line: 1,
                    end_line: 1,
                    issue_type: "god_object".to_string(),
                    severity: "high".to_string(),
                    message: format!("Module '{}' is used by {} other modules, indicating it may be a God Object", module, count),
                    code_snippet: None,
                });
            }
        }
        
        issues
    }
}

export!(GodObjectDetector);