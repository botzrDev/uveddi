use crate::database::models::ArchitecturalIssue;

/// Generates a detailed prompt for an architectural issue to be used by an AI model.
///
/// This function formats the issue type, file, line, description, and code snippet
/// into a structured prompt with instructions for the AI to explain and suggest remediation.
///
/// # Arguments
///
/// * `issue` - Reference to the `ArchitecturalIssue` to generate a prompt for.
///
/// # Returns
///
/// * `String` - The formatted prompt for the AI model.
///
/// # Example
/// ```rust
/// use uveddi::database::models::ArchitecturalIssue;
/// use uveddi::ai::prompts::prompt_templates::for_issue;
/// let issue = ArchitecturalIssue { /* ... */ };
/// let prompt = for_issue(&issue);
/// ```
pub fn for_issue(issue: &ArchitecturalIssue) -> String {
    let anti_pattern_name = match issue.anti_pattern_type_id {
        1 => "God Object",
        2 => "Cyclic Dependency",
        3 => "Unstable Interface",
        4 => "Modularity Violation",
        _ => "Unknown Anti-Pattern",
    };

    let code_context = issue
        .code_snippet
        .as_deref()
        .unwrap_or("No code snippet available.");

    format!(
        r#"
        **Architectural Issue Analysis**

        **Issue Type:** {}
        **File:** `{}`
        **Line:** {}

        **Description:**
        {}

        **Code Snippet:**
        ```
        {}
        ```

        **Instructions:**
        1. Briefly explain what a "{}" anti-pattern is in software architecture.
        2. Explain why the provided code is an example of this anti-pattern.
        3. Provide a concise, actionable suggestion for how to refactor this code to resolve the issue.
        4. If you are unsure or lack enough context, say "Not enough information to provide a reliable explanation." Do not make up details or speculate.
        5. Only use information present in the description and code snippet above.
        "#,
        anti_pattern_name,
        issue.file_path,
        issue.start_line.unwrap_or(0),
        issue.description,
        code_context,
        anti_pattern_name
    )
}
