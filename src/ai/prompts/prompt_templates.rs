use crate::database::models::ArchitecturalIssue;

pub fn for_issue(issue: &ArchitecturalIssue) -> String {
    let anti_pattern_name = match issue.anti_pattern_type_id {
        1 => "God Object",
        2 => "Cyclic Dependency",
        _ => "Unknown Anti-Pattern",
    };

    let code_context = issue.code_snippet.as_deref().unwrap_or("No code snippet available.");

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
        "#,
        anti_pattern_name,
        issue.file_path,
        issue.start_line.unwrap_or(0),
        issue.description,
        code_context,
        anti_pattern_name
    )
}
