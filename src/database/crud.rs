use crate::database::{
    ArchitecturalIssue, CodeSnippet, DatabaseError, DatabaseManager, FromRow,
    system_time_to_unix_timestamp,
};
use rusqlite::params;
use std::time::SystemTime;

/// CRUD operations for ArchitecturalIssue and CodeSnippet
impl DatabaseManager {
    /// Insert a new ArchitecturalIssue and return its ID
    pub fn insert_architectural_issue(
        &mut self,
        issue: &ArchitecturalIssue,
    ) -> Result<i64, DatabaseError> {
        let ignored_at = issue.ignored_at.map(system_time_to_unix_timestamp);
        let is_ignored = if issue.is_ignored { 1 } else { 0 };

        let _rows_affected = self.connection.execute(
            r#"
            INSERT INTO architectural_issues (
                run_id, anti_pattern_type_id, file_path, line_start, line_end,
                severity, title, description, ai_refactoring_suggestion,
                is_ignored, ignored_by_user_id, ignored_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            "#,
            params![
                issue.run_id,
                issue.anti_pattern_type_id,
                issue.file_path,
                issue.line_start,
                issue.line_end,
                issue.severity,
                issue.title,
                issue.description,
                issue.ai_refactoring_suggestion,
                is_ignored,
                issue.ignored_by_user_id,
                ignored_at,
            ],
        )?;

        Ok(self.connection.last_insert_rowid())
    }

    /// Insert a new CodeSnippet and return its ID
    pub fn insert_code_snippet(&mut self, snippet: &CodeSnippet) -> Result<i64, DatabaseError> {
        self.connection.execute(
            r#"
            INSERT INTO code_snippets (
                issue_id, content, language, context_lines_before, context_lines_after
            ) VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            params![
                snippet.issue_id,
                snippet.content,
                snippet.language,
                snippet.context_lines_before,
                snippet.context_lines_after,
            ],
        )?;

        Ok(self.connection.last_insert_rowid())
    }

    /// Query ArchitecturalIssues by run_id with their associated CodeSnippets
    pub fn get_issues_with_snippets_by_run_id(
        &self,
        run_id: i64,
    ) -> Result<Vec<(ArchitecturalIssue, Vec<CodeSnippet>)>, DatabaseError> {
        // First, get all issues for the run
        let mut stmt = self.connection.prepare(
            r#"
            SELECT 
                issue_id, run_id, anti_pattern_type_id, file_path, line_start, line_end,
                severity, title, description, ai_refactoring_suggestion,
                is_ignored, ignored_by_user_id, ignored_at
            FROM architectural_issues 
            WHERE run_id = ?1
            ORDER BY severity DESC, file_path, line_start
            "#,
        )?;

        let issue_rows = stmt.query_map([run_id], |row| {
            ArchitecturalIssue::from_row(row)
        })?;

        let mut results = Vec::new();

        for issue_result in issue_rows {
            let issue = issue_result?;
            let issue_id = issue.issue_id.unwrap();

            // Get associated code snippets for this issue
            let snippets = self.get_code_snippets_by_issue_id(issue_id)?;
            results.push((issue, snippets));
        }

        Ok(results)
    }

    /// Get all CodeSnippets for a specific issue_id
    pub fn get_code_snippets_by_issue_id(
        &self,
        issue_id: i64,
    ) -> Result<Vec<CodeSnippet>, DatabaseError> {
        let mut stmt = self.connection.prepare(
            r#"
            SELECT snippet_id, issue_id, content, language, context_lines_before, context_lines_after
            FROM code_snippets 
            WHERE issue_id = ?1
            ORDER BY snippet_id
            "#,
        )?;

        let snippet_rows = stmt.query_map([issue_id], |row| {
            CodeSnippet::from_row(row)
        })?;

        let mut snippets = Vec::new();
        for snippet_result in snippet_rows {
            snippets.push(snippet_result?);
        }

        Ok(snippets)
    }

    /// Update an ArchitecturalIssue's is_ignored status
    pub fn update_issue_ignored_status(
        &mut self,
        issue_id: i64,
        is_ignored: bool,
        ignored_by_user_id: Option<i64>,
    ) -> Result<bool, DatabaseError> {
        let is_ignored_int = if is_ignored { 1 } else { 0 };
        let ignored_at = if is_ignored {
            Some(system_time_to_unix_timestamp(SystemTime::now()))
        } else {
            None
        };

        let rows_affected = self.connection.execute(
            r#"
            UPDATE architectural_issues 
            SET is_ignored = ?1, ignored_by_user_id = ?2, ignored_at = ?3
            WHERE issue_id = ?4
            "#,
            params![is_ignored_int, ignored_by_user_id, ignored_at, issue_id],
        )?;

        Ok(rows_affected > 0)
    }

    /// Get all ArchitecturalIssues for a specific run_id with optional filtering
    pub fn get_issues_by_run_id(
        &self,
        run_id: i64,
        include_ignored: bool,
        severity_filter: Option<&str>,
    ) -> Result<Vec<ArchitecturalIssue>, DatabaseError> {
        let mut sql = String::from(
            r#"
            SELECT 
                issue_id, run_id, anti_pattern_type_id, file_path, line_start, line_end,
                severity, title, description, ai_refactoring_suggestion,
                is_ignored, ignored_by_user_id, ignored_at
            FROM architectural_issues 
            WHERE run_id = ?1
            "#,
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(run_id)];

        if !include_ignored {
            sql.push_str(" AND is_ignored = 0");
        }

        if let Some(severity) = severity_filter {
            sql.push_str(" AND severity = ?");
            params.push(Box::new(severity.to_string()));
        }

        sql.push_str(" ORDER BY severity DESC, file_path, line_start");

        let mut stmt = self.connection.prepare(&sql)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        
        let issue_rows = stmt.query_map(param_refs.as_slice(), |row| {
            ArchitecturalIssue::from_row(row)
        })?;

        let mut issues = Vec::new();
        for issue_result in issue_rows {
            issues.push(issue_result?);
        }

        Ok(issues)
    }

    /// Example: Create a complete issue with code snippets in a transaction
    pub fn create_issue_with_snippets(
        &mut self,
        issue: &ArchitecturalIssue,
        snippets: &[CodeSnippet],
    ) -> Result<(i64, Vec<i64>), DatabaseError> {
        let tx = self.connection.transaction()?;

        // Insert the issue
        let issue_id = {
            let ignored_at = issue.ignored_at.map(system_time_to_unix_timestamp);
            let is_ignored = if issue.is_ignored { 1 } else { 0 };

            tx.execute(
                r#"
                INSERT INTO architectural_issues (
                    run_id, anti_pattern_type_id, file_path, line_start, line_end,
                    severity, title, description, ai_refactoring_suggestion,
                    is_ignored, ignored_by_user_id, ignored_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                "#,
                params![
                    issue.run_id,
                    issue.anti_pattern_type_id,
                    issue.file_path,
                    issue.line_start,
                    issue.line_end,
                    issue.severity,
                    issue.title,
                    issue.description,
                    issue.ai_refactoring_suggestion,
                    is_ignored,
                    issue.ignored_by_user_id,
                    ignored_at,
                ],
            )?;
            tx.last_insert_rowid()
        };

        // Insert associated code snippets
        let mut snippet_ids = Vec::new();
        for snippet in snippets {
            tx.execute(
                r#"
                INSERT INTO code_snippets (
                    issue_id, content, language, context_lines_before, context_lines_after
                ) VALUES (?1, ?2, ?3, ?4, ?5)
                "#,
                params![
                    issue_id,
                    snippet.content,
                    snippet.language,
                    snippet.context_lines_before,
                    snippet.context_lines_after,
                ],
            )?;
            snippet_ids.push(tx.last_insert_rowid());
        }

        tx.commit()?;
        Ok((issue_id, snippet_ids))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_crud_operations() -> Result<(), DatabaseError> {
        // Create a temporary database
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let mut db = DatabaseManager::new(Some(&db_path)).await?;

        // Create a sample ArchitecturalIssue
        let issue = ArchitecturalIssue {
            issue_id: None,
            run_id: 1,
            anti_pattern_type_id: 1,
            file_path: "src/main.rs".to_string(),
            line_start: Some(10),
            line_end: Some(20),
            severity: "high".to_string(),
            title: "God Object Detected".to_string(),
            description: "This class has too many responsibilities".to_string(),
            ai_refactoring_suggestion: Some("Split into smaller classes".to_string()),
            is_ignored: false,
            ignored_by_user_id: None,
            ignored_at: None,
        };

        // Create sample CodeSnippets
        let snippets = vec![
            CodeSnippet {
                snippet_id: None,
                issue_id: 0, // Will be set after issue creation
                content: "impl GodObject {\n    fn do_everything() {}\n}".to_string(),
                language: Some("rust".to_string()),
                context_lines_before: Some(2),
                context_lines_after: Some(2),
            },
        ];

        // Test creating issue with snippets
        let (issue_id, snippet_ids) = db.create_issue_with_snippets(&issue, &snippets)?;
        
        assert!(issue_id > 0);
        assert_eq!(snippet_ids.len(), 1);
        assert!(snippet_ids[0] > 0);

        // Test updating ignored status
        let updated = db.update_issue_ignored_status(issue_id, true, Some(1))?;
        assert!(updated);

        // Test querying issues with snippets
        let issues_with_snippets = db.get_issues_with_snippets_by_run_id(1)?;
        assert_eq!(issues_with_snippets.len(), 1);
        
        let (retrieved_issue, retrieved_snippets) = &issues_with_snippets[0];
        assert_eq!(retrieved_issue.issue_id, Some(issue_id));
        assert!(retrieved_issue.is_ignored);
        assert_eq!(retrieved_snippets.len(), 1);

        Ok(())
    }
}
