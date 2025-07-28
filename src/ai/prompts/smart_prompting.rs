//! Smart prompting and RAG strategy implementation

// use crate::ast::CustomAst;
use crate::database::models::ArchitecturalIssue;
use crate::semantic_search::{IndexedChunk, VectorIndex};
use crate::ai::knowledge::KnowledgeContext;
use crate::error::UveddiError;

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

    /// Build knowledge-enhanced prompt for architectural issues
    ///
    /// This method creates AI prompts that incorporate relevant knowledge patterns
    /// from the knowledge library to provide more context-aware analysis.
    ///
    /// # Arguments
    ///
    /// * `issue` - The architectural issue to analyze
    /// * `knowledge_context` - Selected knowledge patterns relevant to the issue
    ///
    /// # Returns
    ///
    /// * `Result<String, UveddiError>` - Enhanced prompt with knowledge context
    pub fn build_knowledge_enhanced_prompt(
        &self,
        issue: &ArchitecturalIssue,
        knowledge_context: &KnowledgeContext,
    ) -> Result<String, UveddiError> {
        // Start with base prompt structure
        let mut prompt = self.build_base_prompt_structure(issue)?;

        // Add knowledge context section if patterns are available
        if !knowledge_context.selected_patterns.is_empty() {
            let knowledge_section = self.build_knowledge_section(knowledge_context)?;
            prompt = self.insert_knowledge_section(prompt, knowledge_section)?;
        }

        // Add knowledge-specific instructions
        prompt = self.add_knowledge_instructions(prompt, knowledge_context)?;

        // Apply final formatting and validation
        let final_prompt = self.finalize_prompt(prompt)?;

        Ok(final_prompt)
    }

    /// Build the base prompt structure
    fn build_base_prompt_structure(&self, issue: &ArchitecturalIssue) -> Result<String, UveddiError> {
        let issue_context = format!(
            "**Architectural Issue Analysis**\n\
            Anti-pattern Type: {}\n\
            Description: {}\n\
            File: {}\n\
            Severity: {}\n\
            Location: Line {} to {}",
            issue.anti_pattern_type_id,
            issue.description,
            issue.file_path,
            issue.severity,
            issue.start_line.unwrap_or(0),
            issue.end_line.unwrap_or(0)
        );

        let mut prompt = format!(
            "You are an expert software architect and code quality specialist.\n\
            Your task is to analyze the following architectural issue and provide actionable guidance.\n\n\
            {}\n",
            issue_context
        );

        // Add code snippet if available
        if let Some(code_snippet) = &issue.code_snippet {
            prompt.push_str(&format!(
                "\n**Code Context:**\n```\n{}\n```\n",
                code_snippet
            ));
        }

        Ok(prompt)
    }

    /// Build knowledge context section
    fn build_knowledge_section(&self, knowledge_context: &KnowledgeContext) -> Result<String, UveddiError> {
        let mut section = String::from("\n**ARCHITECTURAL KNOWLEDGE BASE**\n");
        section.push_str(&format!(
            "*Relevance Score: {:.1}% | Patterns: {} | Library Version: {}*\n\n",
            knowledge_context.relevance_score * 100.0,
            knowledge_context.selected_patterns.len(),
            knowledge_context.library_version
        ));

        for (i, pattern) in knowledge_context.selected_patterns.iter().enumerate() {
            section.push_str(&format!(
                "**Pattern {}: {}**\n\
                • **Type**: {:?}\n\
                • **Description**: {}\n\
                • **Impact**: {:?}\n\
                • **Common Symptoms**: {}\n\
                • **Detection Methods**: Available\n\
                • **Solutions Available**: {}\n\n",
                i + 1,
                pattern.name,
                pattern.category,
                pattern.definition.as_str(),
                pattern.impact,
                pattern.symptoms.iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
                pattern.solutions.len()
            ));

            // Add top 2 solutions for each pattern
            for (j, solution) in pattern.solutions.iter().take(2).enumerate() {
                section.push_str(&format!(
                    "  **Solution {}.{}**: {} (Effort: {:?}, Impact: {:?})\n",
                    i + 1,
                    j + 1,
                    solution.title,
                    solution.effort_level,
                    solution.expected_impact
                ));
            }
            section.push('\n');
        }

        Ok(section)
    }

    /// Insert knowledge section into prompt
    fn insert_knowledge_section(&self, mut prompt: String, knowledge_section: String) -> Result<String, UveddiError> {
        // Find insertion point (before response format instructions)
        let insertion_markers = [
            "Respond in the following",
            "Please provide",
            "Your response should",
            "Format your response"
        ];

        let mut insertion_point = None;
        for marker in &insertion_markers {
            if let Some(pos) = prompt.find(marker) {
                insertion_point = Some(pos);
                break;
            }
        }

        match insertion_point {
            Some(pos) => {
                prompt.insert_str(pos, &knowledge_section);
                Ok(prompt)
            }
            None => {
                // If no insertion point found, append before the end
                prompt.push_str(&knowledge_section);
                Ok(prompt)
            }
        }
    }

    /// Add knowledge-specific instructions
    fn add_knowledge_instructions(&self, mut prompt: String, knowledge_context: &KnowledgeContext) -> Result<String, UveddiError> {
        let instructions = format!(
            "\n**ANALYSIS INSTRUCTIONS:**\n\
            1. **Use the provided architectural knowledge** to enhance your analysis\n\
            2. **Reference specific patterns and solutions** from the knowledge base\n\
            3. **Prioritize solutions** based on effort/impact ratios provided\n\
            4. **Indicate confidence level** based on knowledge completeness ({:.1}% relevance)\n\
            5. **Provide implementation steps** based on proven architectural patterns\n\
            6. **Consider the specific context** of the detected anti-pattern\n\
            7. **Base recommendations on established best practices** from the knowledge base\n\n",
            knowledge_context.relevance_score * 100.0
        );

        prompt.push_str(&instructions);
        Ok(prompt)
    }

    /// Finalize prompt with response format and anti-hallucination measures
    fn finalize_prompt(&self, mut prompt: String) -> Result<String, UveddiError> {
        // Add response format
        prompt.push_str(
            "**RESPONSE FORMAT:**\n\
            Provide your analysis in the following JSON structure:\n\
            ```json\n\
            {\n\
              \"title\": \"Brief, specific title for this architectural issue\",\n\
              \"severity_assessment\": \"Your assessment of the severity (Critical/High/Medium/Low)\",\n\
              \"root_cause_analysis\": \"Detailed explanation of why this issue exists\",\n\
              \"architectural_impact\": \"How this affects the overall system architecture\",\n\
              \"recommended_solution\": \"Primary recommended solution with implementation steps\",\n\
              \"alternative_solutions\": [\"List of alternative approaches if applicable\"],\n\
              \"implementation_effort\": \"Estimated effort level (Low/Medium/High/Very High)\",\n\
              \"business_impact\": \"How fixing this issue benefits the business/team\",\n\
              \"confidence_level\": \"Your confidence in this analysis (High/Medium/Low)\",\n\
              \"knowledge_patterns_used\": [\"List of knowledge patterns that informed this analysis\"]\n\
            }\n\
            ```\n\n"
        );

        // Add anti-hallucination instructions
        prompt.push_str(
            "**CRITICAL GUIDELINES:**\n\
            • Base your analysis ONLY on the provided issue details and knowledge base patterns\n\
            • DO NOT invent or assume details not present in the context\n\
            • If information is insufficient, clearly state your limitations\n\
            • Reference specific knowledge patterns when making recommendations\n\
            • Indicate uncertainty levels clearly in your confidence assessment\n\
            • Focus on actionable, practical solutions based on proven patterns\n\
            • Avoid generic advice - tailor recommendations to the specific issue context\n"
        );

        Ok(prompt)
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
