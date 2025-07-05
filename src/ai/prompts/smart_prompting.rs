//! Smart prompting and RAG strategy implementation

// use crate::ast::CustomAst;
use crate::database::models::ArchitecturalIssue;
use crate::semantic_search::{IndexedChunk, VectorIndex};

/// Smart prompt builder for generating AI prompts for architectural issues
#[derive(Clone)]
pub struct SmartPromptBuilder {
    // Configuration options
    pub use_rag: bool,
    pub max_context_chunks: usize,
    pub include_code_snippets: bool,
    pub diversify_results: bool,
}

impl Default for SmartPromptBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SmartPromptBuilder {
    pub fn new() -> Self {
        SmartPromptBuilder {
            use_rag: true,
            max_context_chunks: 5,
            include_code_snippets: true,
            diversify_results: true,
        }
    }

    /// Build a prompt specifically for analyzing an architectural issue
    pub fn build_prompt_for_issue(&self, issue: &ArchitecturalIssue) -> String {
        let issue_context = format!(
            "Anti-pattern Type ID: {}\nDescription: {}\nFile: {}\nSeverity: {}",
            issue.anti_pattern_type_id, issue.description, issue.file_path, issue.severity
        );

        // Build basic prompt with code snippet if available
        let mut prompt = format!(
            "You are an expert software architect.\n\
Given the following architectural issue, provide a detailed explanation and recommendation.\n\
\n{issue_context}\n"
        );

        // Add code snippet context if available and enabled
        if self.include_code_snippets && issue.code_snippet.is_some() {
            prompt.push_str(&format!(
                "\nRelevant code snippet:\n```\n{}\n```\n",
                issue.code_snippet.as_ref().unwrap()
            ));
        }

        // Add response format instructions
        prompt.push_str(
            "\nRespond in the following JSON format:\n\
            {\n\
              \"title\": \"Brief title for the issue\",\n\
              \"description\": \"Detailed description of the problem\",\n\
              \"explanation\": \"Why this is an architectural concern\",\n\
              \"refactoring\": \"Recommended solution or refactoring steps\",\n\
              \"confidence\": \"high/medium/low confidence in this assessment\"\n\
            }\n",
        );

        add_hallucination_mitigation(&prompt)
    }

    /// Build a RAG-enhanced prompt with relevant context from codebase
    pub fn build_rag_prompt(
        &self,
        issue: &ArchitecturalIssue,
        vector_index: &VectorIndex,
        query_embedding: &ndarray::Array1<f32>,
    ) -> String {
        // Start with the basic prompt
        let mut prompt = self.build_prompt_for_issue(issue);

        if !self.use_rag {
            return prompt;
        }

        // Get relevant context chunks from the vector index
        let relevant_chunks = if self.diversify_results {
            // Use maximal marginal relevance to diversify results
            use crate::semantic_search::mmr::maximal_marginal_relevance;
            let search_results = vector_index.search(query_embedding, self.max_context_chunks * 2);
            // Convert search results to IndexedChunk references
            let candidates: Vec<&IndexedChunk> =
                search_results.iter().map(|(chunk, _)| *chunk).collect();

            // MMR returns the selected chunks directly, not indices
            maximal_marginal_relevance(
                query_embedding,
                &candidates,
                0.5, // Lambda (diversity parameter) - balance between relevance and diversity
                self.max_context_chunks, // k - number of results to return
            )
        } else {
            // Just use top-k most similar chunks
            vector_index
                .search(query_embedding, self.max_context_chunks)
                .into_iter()
                .map(|(chunk, _)| chunk)
                .collect::<Vec<_>>()
        };

        // Insert the RAG context
        if !relevant_chunks.is_empty() {
            let context_section = format!(
                "\n\n### RELEVANT CODEBASE CONTEXT ###\n{}",
                relevant_chunks
                    .iter()
                    .map(|chunk| format!(
                        "--- {} ---\n{}\n",
                        chunk.metadata.get("path").unwrap_or(&"unknown".to_string()),
                        chunk.text
                    ))
                    .collect::<Vec<_>>()
                    .join("\n")
            );

            // Insert context before the response format
            prompt.insert_str(
                prompt.find("Respond in the following").unwrap(),
                &context_section,
            );
        }

        prompt
    }
}

/// Add anti-hallucination guardrails to the prompt
fn add_hallucination_mitigation(prompt: &str) -> String {
    format!(
        "{}\n\nIMPORTANT: Base your explanations only on the provided information. DO NOT invent or hallucinate details not present in the issue description or context. If you're uncertain, indicate your level of confidence clearly. Focus on architectural principles and patterns that are relevant to the described issue.",
        prompt
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    // TODO: Add tests for SmartPromptBuilder once CustomAst is properly implemented
    #[test]
    fn test_smart_prompt_builder_basic() {
        let builder = SmartPromptBuilder::new();
        assert!(builder.use_rag);
        assert_eq!(builder.max_context_chunks, 5);
    }

    #[test]
    fn test_add_hallucination_mitigation_appends_instruction() {
        let prompt = "Explain the issue.";
        let mitigated = add_hallucination_mitigation(prompt);
        assert!(mitigated
            .contains("IMPORTANT: Base your explanations only on the provided information."));
    }
}
