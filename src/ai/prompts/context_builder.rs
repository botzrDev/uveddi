use crate::database::models::ArchitecturalIssue;
use crate::ast::tree_sitter::ParsedFile;

/// Contextual information for AI analysis
#[derive(Debug)]
pub struct IssueContext {
    pub language: String,
    pub code_snippet: String,
    pub surrounding_context: String,
    pub project_patterns: Vec<String>, // For future RAG implementation
}

/// Context builder for rich AI prompts
pub struct ContextBuilder {
    context: IssueContext,
}

impl ContextBuilder {
    pub fn new() -> Self {
        Self {
            context: IssueContext {
                language: String::new(),
                code_snippet: String::new(),
                surrounding_context: String::new(),
                project_patterns: Vec::new(),
            }
        }
    }

    pub fn add_issue_context(mut self, issue: &ArchitecturalIssue) -> Self {
        if let Some(snippet) = &issue.code_snippet {
            self.context.code_snippet = snippet.clone();
        }
        self
    }

    pub fn add_code_context(mut self, file: &ParsedFile) -> Self {
        self.context.language = format!("{:?}", file.language);
        // TODO: Add surrounding context extraction logic
        self
    }

    pub fn add_project_context(self) -> Self {
        // Future: Add project-wide patterns and conventions
        self
    }

    pub fn build(self) -> IssueContext {
        self.context
    }
}
