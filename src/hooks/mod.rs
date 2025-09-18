//! Git hooks integration for automated code analysis
//!
//! This module provides comprehensive Git hooks management for running
//! Uveddi analysis automatically during development workflows.

pub mod manager;
// pub mod templates;  // TODO: Create templates module

use crate::error::UveddiError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookConfig {
    /// Enable pre-commit hook
    pub pre_commit: bool,
    /// Enable pre-push hook
    pub pre_push: bool,
    /// Enable commit-msg hook for conventional commits
    pub commit_msg: bool,
    /// Analysis timeout in seconds
    pub timeout_seconds: u64,
    /// Run analysis only on changed files
    pub changed_files_only: bool,
    /// Fail fast on first critical issue
    pub fail_fast: bool,
    /// Show progress during analysis
    pub show_progress: bool,
    /// Skip analysis for certain commit patterns
    pub skip_patterns: Vec<String>,
    /// Custom hook scripts directory
    pub hooks_dir: Option<PathBuf>,
    /// Minimum confidence threshold for blocking commits
    pub min_confidence: f64,
    /// Maximum number of issues before blocking
    pub max_issues: Option<u32>,
}

impl Default for HookConfig {
    fn default() -> Self {
        Self {
            pre_commit: true,
            pre_push: false,
            commit_msg: false,
            timeout_seconds: 300, // 5 minutes
            changed_files_only: true,
            fail_fast: true,
            show_progress: true,
            skip_patterns: vec![
                "WIP:".to_string(),
                "[skip ci]".to_string(),
                "[no-analyze]".to_string(),
            ],
            hooks_dir: None,
            min_confidence: 0.8,
            max_issues: Some(5),
        }
    }
}

#[derive(Debug, Clone)]
pub enum HookType {
    PreCommit,
    PrePush,
    CommitMsg,
    PostCommit,
    PostMerge,
}

impl HookType {
    pub fn filename(&self) -> &'static str {
        match self {
            HookType::PreCommit => "pre-commit",
            HookType::PrePush => "pre-push",
            HookType::CommitMsg => "commit-msg",
            HookType::PostCommit => "post-commit",
            HookType::PostMerge => "post-merge",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookExecutionResult {
    pub hook_type: String,
    pub success: bool,
    pub issues_found: u32,
    pub critical_issues: u32,
    pub execution_time: f64,
    pub files_analyzed: u32,
    pub message: String,
    pub suggestions: Vec<String>,
}

/// Result of hook installation
#[derive(Debug)]
pub struct InstallationResult {
    pub installed_hooks: Vec<HookType>,
    pub updated_hooks: Vec<HookType>,
    pub errors: Vec<String>,
    pub config_path: Option<PathBuf>,
}

/// Get the git hooks directory for a repository
pub fn get_git_hooks_dir(repo_path: &Path) -> Result<PathBuf, UveddiError> {
    let git_dir = repo_path.join(".git");

    if git_dir.is_file() {
        // Handle git worktrees and submodules
        let content = std::fs::read_to_string(&git_dir).map_err(|e| {
            UveddiError::io_error("reading .git file", &git_dir.to_string_lossy(), e)
        })?;

        if let Some(gitdir_line) = content.lines().find(|line| line.starts_with("gitdir:")) {
            let gitdir = gitdir_line[8..].trim(); // Remove "gitdir: " prefix
            let hooks_dir = if gitdir.starts_with('/') {
                PathBuf::from(gitdir).join("hooks")
            } else {
                repo_path.join(gitdir).join("hooks")
            };
            return Ok(hooks_dir);
        }
    }

    if git_dir.is_dir() {
        return Ok(git_dir.join("hooks"));
    }

    Err(UveddiError::config_error(
        &format!("Not a git repository: {}", repo_path.display()),
        "system",
    ))
}

/// Check if a git repository exists at the given path
pub fn is_git_repository(path: &Path) -> bool {
    get_git_hooks_dir(path).is_ok()
}

/// Get the list of staged files for commit
pub fn get_staged_files(repo_path: &Path) -> Result<Vec<PathBuf>, UveddiError> {
    use std::process::Command;

    let output = Command::new("git")
        .args(&["diff", "--cached", "--name-only", "--diff-filter=ACMR"])
        .current_dir(repo_path)
        .output()
        .map_err(|e| UveddiError::CliError {
            command: "git diff --cached --name-only".to_string(),
            message: e.to_string(),
            suggestion: "Ensure git is installed and the directory is a git repository".to_string(),
            source: None,
        })?;

    if !output.status.success() {
        return Err(UveddiError::CliError {
            command: "git diff --cached --name-only".to_string(),
            message: format!(
                "Git command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
            suggestion: "Check git repository status and permissions".to_string(),
            source: None,
        });
    }

    let files = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| repo_path.join(line))
        .collect();

    Ok(files)
}

/// Check if commit message should skip analysis
pub fn should_skip_analysis(commit_msg: &str, skip_patterns: &[String]) -> bool {
    let msg_lower = commit_msg.to_lowercase();
    skip_patterns.iter().any(|pattern| {
        let pattern_lower = pattern.to_lowercase();
        msg_lower.contains(&pattern_lower)
    })
}

/// Execute Uveddi analysis for git hooks
pub async fn execute_analysis_for_hook(
    repo_path: &Path,
    config: &HookConfig,
    hook_type: &HookType,
    files: Option<&[PathBuf]>,
) -> Result<HookExecutionResult, UveddiError> {
    use std::process::Command;
    use std::time::Instant;

    let start_time = Instant::now();

    // Build command
    let mut cmd = Command::new("uveddi");
    cmd.arg("analyze");

    // Add target path
    if let Some(files) = files {
        if config.changed_files_only && !files.is_empty() {
            // Analyze only changed files
            for file in files {
                if file.exists() && is_analyzable_file(file) {
                    cmd.arg(file);
                }
            }
        } else {
            cmd.arg(".");
        }
    } else {
        cmd.arg(".");
    }

    // Add analysis options
    cmd.args(&[
        "--output-format",
        "json",
        "--timeout",
        &config.timeout_seconds.to_string(),
    ]);

    if config.fail_fast {
        cmd.arg("--fail-fast");
    }

    if !config.show_progress {
        cmd.args(&["--progress-format", "silent"]);
    }

    // Set confidence threshold
    cmd.args(&["--confidence-threshold", &config.min_confidence.to_string()]);

    // Execute analysis
    cmd.current_dir(repo_path);
    let output = cmd.output().map_err(|e| UveddiError::CliError {
        command: format!("{:?}", cmd),
        message: e.to_string(),
        suggestion: "Ensure Uveddi is properly installed and accessible".to_string(),
        source: None,
    })?;

    let execution_time = start_time.elapsed().as_secs_f64();

    // Parse results
    let (issues_found, critical_issues, message, suggestions) = if output.status.success() {
        parse_analysis_output(&output.stdout)
    } else {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        (0, 0, format!("Analysis failed: {}", error_msg), vec![])
    };

    let success = output.status.success()
        && (config.max_issues.is_none() || issues_found <= config.max_issues.unwrap_or(u32::MAX))
        && critical_issues == 0;

    let files_analyzed = if let Some(files) = files {
        files.len() as u32
    } else {
        count_analyzable_files(repo_path)?
    };

    Ok(HookExecutionResult {
        hook_type: format!("{:?}", hook_type),
        success,
        issues_found,
        critical_issues,
        execution_time,
        files_analyzed,
        message,
        suggestions,
    })
}

/// Check if a file should be analyzed
pub fn is_analyzable_file(path: &Path) -> bool {
    if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
        matches!(
            extension,
            "rs" | "py" | "js" | "ts" | "tsx" | "jsx" | "go" | "java" | "c" | "cpp" | "h" | "hpp"
        )
    } else {
        false
    }
}

/// Count analyzable files in a directory
fn count_analyzable_files(path: &Path) -> Result<u32, UveddiError> {
    use std::fs;

    let mut count = 0;

    fn visit_dir(dir: &Path, count: &mut u32) -> Result<(), UveddiError> {
        let entries = fs::read_dir(dir)
            .map_err(|e| UveddiError::io_error("reading directory", &dir.to_string_lossy(), e))?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                UveddiError::io_error("reading directory entry", &dir.to_string_lossy(), e)
            })?;

            let path = entry.path();

            if path.is_dir() {
                let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !dir_name.starts_with('.')
                    && !matches!(
                        dir_name,
                        "node_modules" | "target" | "dist" | "build" | "__pycache__"
                    )
                {
                    visit_dir(&path, count)?;
                }
            } else if is_analyzable_file(&path) {
                *count += 1;
            }
        }

        Ok(())
    }

    visit_dir(path, &mut count)?;
    Ok(count)
}

/// Parse analysis output from JSON
fn parse_analysis_output(output: &[u8]) -> (u32, u32, String, Vec<String>) {
    if let Ok(json_str) = String::from_utf8_lossy(output).parse::<serde_json::Value>() {
        let issues = json_str
            .get("issues")
            .and_then(|i| i.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0) as u32;
        let critical = json_str
            .get("critical_issues")
            .and_then(|i| i.as_u64())
            .unwrap_or(0) as u32;
        let message = json_str
            .get("summary")
            .and_then(|s| s.as_str())
            .unwrap_or("Analysis completed")
            .to_string();
        let suggestions = json_str
            .get("suggestions")
            .and_then(|s| s.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        (issues, critical, message, suggestions)
    } else {
        (0, 0, "Could not parse analysis results".to_string(), vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_hook_config_default() {
        let config = HookConfig::default();
        assert!(config.pre_commit);
        assert!(!config.pre_push);
        assert!(config.changed_files_only);
        assert_eq!(config.timeout_seconds, 300);
    }

    #[test]
    fn test_should_skip_analysis() {
        let skip_patterns = vec!["WIP:".to_string(), "[skip ci]".to_string()];

        assert!(should_skip_analysis(
            "WIP: working on feature",
            &skip_patterns
        ));
        assert!(should_skip_analysis("Fix bug [skip ci]", &skip_patterns));
        assert!(!should_skip_analysis(
            "Regular commit message",
            &skip_patterns
        ));
    }

    #[test]
    fn test_is_analyzable_file() {
        assert!(is_analyzable_file(Path::new("main.rs")));
        assert!(is_analyzable_file(Path::new("app.py")));
        assert!(is_analyzable_file(Path::new("component.tsx")));
        assert!(!is_analyzable_file(Path::new("README.md")));
        assert!(!is_analyzable_file(Path::new("data.json")));
    }

    #[test]
    fn test_hook_type_filename() {
        assert_eq!(HookType::PreCommit.filename(), "pre-commit");
        assert_eq!(HookType::PrePush.filename(), "pre-push");
        assert_eq!(HookType::CommitMsg.filename(), "commit-msg");
    }
}
