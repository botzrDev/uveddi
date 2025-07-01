use rusqlite::{Connection, Result};
use crate::database::models::{AnalysisRun, ArchitecturalIssue, AntiPatternType};
use std::path::Path;
use chrono::Utc;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self> {
        let conn = Connection::open("uveddi.db")?;
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS analysis_runs (
                run_id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                start_time TEXT NOT NULL,
                end_time TEXT,
                status TEXT NOT NULL,
                total_files_analyzed INTEGER,
                total_issues_found INTEGER,
                analysis_config TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS anti_pattern_types (
                type_id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                description TEXT NOT NULL,
                category TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS architectural_issues (
                issue_id INTEGER PRIMARY KEY AUTOINCREMENT,
                analysis_run_id INTEGER NOT NULL,
                anti_pattern_type_id INTEGER NOT NULL,
                file_path TEXT NOT NULL,
                start_line INTEGER,
                end_line INTEGER,
                severity TEXT NOT NULL,
                description TEXT NOT NULL,
                code_snippet TEXT,
                ai_explanation TEXT,
                FOREIGN KEY (analysis_run_id) REFERENCES analysis_runs(run_id),
                FOREIGN KEY (anti_pattern_type_id) REFERENCES anti_pattern_types(type_id)
            );
            CREATE TABLE IF NOT EXISTS projects (
                project_id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT NOT NULL UNIQUE
            );
        ")?;
        Ok(Database { conn })
    }

    pub fn get_or_create_project_id(&self, project_path: &Path) -> Result<i64> {
        let path_str = project_path.to_string_lossy().to_string();
        let mut stmt = self.conn.prepare("SELECT project_id FROM projects WHERE path = ?")?;
        let mut rows = stmt.query([&path_str])?;

        if let Some(row) = rows.next()? {
            Ok(row.get(0)?)
        } else {
            self.conn.execute("INSERT INTO projects (path) VALUES (?)", [&path_str])?;
            Ok(self.conn.last_insert_rowid())
        }
    }

    pub fn create_analysis_run(&self, project_path: &Path) -> Result<AnalysisRun> {
        let project_id = self.get_or_create_project_id(project_path)?;
        let analysis_run = AnalysisRun {
            run_id: None,
            project_id,
            start_time: Utc::now(),
            end_time: None,
            status: "running".to_string(),
            total_files_analyzed: None,
            total_issues_found: None,
            analysis_config: "{}".to_string(), // Default empty JSON config
        };

        self.conn.execute(
            "INSERT INTO analysis_runs (project_id, start_time, status, analysis_config) VALUES (?, ?, ?, ?)",
            rusqlite::params![
                analysis_run.project_id,
                analysis_run.start_time.to_rfc3339(),
                analysis_run.status,
                analysis_run.analysis_config,
            ],
        )?;

        let last_id = self.conn.last_insert_rowid();
        Ok(AnalysisRun { run_id: Some(last_id), ..analysis_run })
    }

    pub fn update_analysis_run(&self, run: &AnalysisRun) -> Result<()> {
        self.conn.execute(
            "UPDATE analysis_runs SET end_time = ?, status = ?, total_files_analyzed = ?, total_issues_found = ? WHERE run_id = ?",
            rusqlite::params![
                run.end_time.map(|dt| dt.to_rfc3339()),
                run.status,
                run.total_files_analyzed,
                run.total_issues_found,
                run.run_id,
            ],
        )?;
        Ok(())
    }

    pub fn store_anti_pattern_type(&self, anti_pattern_type: &mut AntiPatternType) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO anti_pattern_types (name, description, category) VALUES (?, ?, ?)",
            rusqlite::params![
                anti_pattern_type.name,
                anti_pattern_type.description,
                anti_pattern_type.category,
            ],
        )?;
        if anti_pattern_type.type_id.is_none() {
            let mut stmt = self.conn.prepare("SELECT type_id FROM anti_pattern_types WHERE name = ?")?;
            anti_pattern_type.type_id = Some(stmt.query_row([&anti_pattern_type.name], |row| row.get(0))?);
        }
        Ok(())
    }

    pub fn store_issues(&mut self, issues: &[ArchitecturalIssue]) -> Result<()> {
        let tx = self.conn.transaction()?;
        for issue in issues {
            tx.execute(
                "INSERT INTO architectural_issues (analysis_run_id, anti_pattern_type_id, file_path, start_line, end_line, severity, description, code_snippet, ai_explanation) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                rusqlite::params![
                    issue.analysis_run_id,
                    issue.anti_pattern_type_id,
                    issue.file_path,
                    issue.start_line,
                    issue.end_line,
                    issue.severity,
                    issue.description,
                    issue.code_snippet,
                    issue.ai_explanation,
                ],
            )?;
        }
        tx.commit()
    }
}