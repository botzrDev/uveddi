//! Enhanced context builder for AI analysis

use std::collections::HashMap;

pub struct ContextBuilder {
    max_context_size: usize,
    project_patterns: Vec<String>,
    file_contexts: HashMap<String, String>,
}

impl ContextBuilder {
    pub fn new(max_size: usize) -> Self {
        Self {
            max_context_size: max_size,
            project_patterns: Vec::new(),
            file_contexts: HashMap::new(),
        }
    }

    /// Add project-level patterns detected during analysis
    pub fn add_pattern(&mut self, pattern_description: String) {
        self.project_patterns.push(pattern_description);
    }

    /// Add file context with surrounding code for a specific issue
    pub fn add_file_context(&mut self, file_path: &str, context: String) {
        self.file_contexts.insert(file_path.to_string(), context);
    }

    /// Build the final context for AI analysis
    pub fn build_for_issue(&self, file_path: &str) -> String {
        let mut context = String::new();
        context.push_str("Project Patterns:\n");
        for pattern in &self.project_patterns {
            context.push_str(&format!"- {}\n", pattern));
        }
        if let Some(file_context) = self.file_contexts.get(file_path) {
            context.push_str("\nRelevant Code:\n```");
            context.push_str(file_context);
            context.push_str("\n```");
        }
        // Truncate if exceeds max_context_size
        if context.len() > self.max_context_size {
            context.truncate(self.max_context_size);
        }
        context
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_context_builder() {
        let mut builder = ContextBuilder::new(1000);
        builder.add_pattern("Singleton pattern detected".to_string());
        builder.add_file_context("main.rs", "fn main() { println!(\"hi\"); }".to_string());
        let ctx = builder.build_for_issue("main.rs");
        assert!(ctx.contains("Singleton pattern"));
        assert!(ctx.contains("fn main()"));
    }
}
