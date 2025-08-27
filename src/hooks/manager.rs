//! Git hooks management and installation
//!
//! This module provides functionality to install, update, and manage
//! Git hooks for automated Uveddi analysis.

use super::{HookConfig, HookType, InstallationResult};
use crate::error::UveddiError;
use std::fs;
use std::path::{Path, PathBuf};
use std::os::unix::fs::PermissionsExt;

pub struct HookManager {
    repo_path: PathBuf,
    hooks_dir: PathBuf,
    config: HookConfig,
}

impl HookManager {
    /// Create a new hook manager for a Git repository
    pub fn new(repo_path: PathBuf, config: HookConfig) -> Result<Self, UveddiError> {
        let hooks_dir = super::get_git_hooks_dir(&repo_path)?;
        
        // Ensure hooks directory exists
        if !hooks_dir.exists() {
            fs::create_dir_all(&hooks_dir).map_err(|e| UveddiError::io_error("creating hooks directory", &hooks_dir.to_string_lossy(), e))?;
        }
        
        Ok(Self {
            repo_path,
            hooks_dir,
            config,
        })
    }
    
    /// Install all configured hooks
    pub async fn install_hooks(&self, force_overwrite: bool) -> Result<InstallationResult, UveddiError> {
        let mut installed_hooks = Vec::new();
        let mut updated_hooks = Vec::new();
        let mut errors = Vec::new();
        
        // Install pre-commit hook
        if self.config.pre_commit {
            match self.install_hook(&HookType::PreCommit, force_overwrite).await {
                Ok(true) => installed_hooks.push(HookType::PreCommit),
                Ok(false) => updated_hooks.push(HookType::PreCommit),
                Err(e) => errors.push(format!("Pre-commit hook: {}", e)),
            }
        }
        
        // Install pre-push hook
        if self.config.pre_push {
            match self.install_hook(&HookType::PrePush, force_overwrite).await {
                Ok(true) => installed_hooks.push(HookType::PrePush),
                Ok(false) => updated_hooks.push(HookType::PrePush),
                Err(e) => errors.push(format!("Pre-push hook: {}", e)),
            }
        }
        
        // Install commit-msg hook
        if self.config.commit_msg {
            match self.install_hook(&HookType::CommitMsg, force_overwrite).await {
                Ok(true) => installed_hooks.push(HookType::CommitMsg),
                Ok(false) => updated_hooks.push(HookType::CommitMsg),
                Err(e) => errors.push(format!("Commit-msg hook: {}", e)),
            }
        }
        
        // Save hook configuration
        let config_path = self.save_hook_config().await?;
        
        Ok(InstallationResult {
            installed_hooks,
            updated_hooks,
            errors,
            config_path: Some(config_path),
        })
    }
    
    /// Install a specific hook type
    async fn install_hook(&self, hook_type: &HookType, force_overwrite: bool) -> Result<bool, UveddiError> {
        let hook_path = self.hooks_dir.join(hook_type.filename());
        let was_new = !hook_path.exists();
        
        // Check if hook already exists and we're not forcing overwrite
        if hook_path.exists() && !force_overwrite {
            // Check if it's already an Uveddi hook
            if let Ok(content) = fs::read_to_string(&hook_path) {
                if content.contains("# Uveddi Hook") {
                    // Update existing Uveddi hook
                    self.write_hook_script(&hook_path, hook_type).await?;
                    return Ok(false); // Updated, not newly installed
                }
            }
            
            // Backup existing non-Uveddi hook
            let backup_path = hook_path.with_extension("backup");
            fs::rename(&hook_path, &backup_path).map_err(|e| UveddiError::io_error("backing up existing hook", &hook_path.to_string_lossy(), e))?;
        }
        
        self.write_hook_script(&hook_path, hook_type).await?;
        self.make_executable(&hook_path)?;
        
        Ok(was_new)
    }
    
    /// Write the hook script content
    async fn write_hook_script(&self, hook_path: &Path, hook_type: &HookType) -> Result<(), UveddiError> {
        let script_content = self.generate_hook_script(hook_type);
        
        fs::write(hook_path, script_content).map_err(|e| UveddiError::io_error("writing hook script", &hook_path.to_string_lossy(), e))?;
        
        Ok(())
    }
    
    /// Generate hook script content
    fn generate_hook_script(&self, hook_type: &HookType) -> String {
        match hook_type {
            HookType::PreCommit => self.generate_pre_commit_script(),
            HookType::PrePush => self.generate_pre_push_script(),
            HookType::CommitMsg => self.generate_commit_msg_script(),
            _ => String::new(),
        }
    }
    
    /// Generate pre-commit hook script
    fn generate_pre_commit_script(&self) -> String {
        format!(r#"#!/bin/sh
# Uveddi Hook - Pre-commit Analysis
# Generated automatically by Uveddi
# DO NOT EDIT MANUALLY

set -e

echo "🔍 Running Uveddi pre-commit analysis..."

# Check if we should skip analysis
if git log -1 --pretty=%B | grep -E "(WIP:|\\[skip ci\\]|\\[no-analyze\\])" > /dev/null 2>&1; then
    echo "⏭️  Skipping analysis (skip pattern detected in commit message)"
    exit 0
fi

# Get staged files
STAGED_FILES=$(git diff --cached --name-only --diff-filter=ACMR)

if [ -z "$STAGED_FILES" ]; then
    echo "⚠️  No staged files to analyze"
    exit 0
fi

# Filter for analyzable files
ANALYZABLE_FILES=""
for file in $STAGED_FILES; do
    case "$file" in
        *.rs|*.py|*.js|*.ts|*.tsx|*.jsx|*.go|*.java|*.c|*.cpp|*.h|*.hpp)
            if [ -f "$file" ]; then
                ANALYZABLE_FILES="$ANALYZABLE_FILES $file"
            fi
            ;;
    esac
done

if [ -z "$ANALYZABLE_FILES" ]; then
    echo "✅ No code files to analyze"
    exit 0
fi

# Run analysis with timeout
timeout {} uveddi analyze $ANALYZABLE_FILES \
    --output-format json \
    --confidence-threshold {} \
    --timeout {} \
    --progress-format {} \
    {} > /tmp/uveddi-precommit.json 2>/tmp/uveddi-precommit.err

ANALYSIS_EXIT_CODE=$?

if [ $ANALYSIS_EXIT_CODE -eq 0 ]; then
    # Parse results
    ISSUES_COUNT=$(cat /tmp/uveddi-precommit.json | jq -r '.issues | length // 0' 2>/dev/null || echo "0")
    CRITICAL_COUNT=$(cat /tmp/uveddi-precommit.json | jq -r '.critical_issues // 0' 2>/dev/null || echo "0")
    
    if [ "$CRITICAL_COUNT" -gt 0 ]; then
        echo "❌ Found $CRITICAL_COUNT critical issues. Commit blocked."
        echo "🔧 Run 'uveddi analyze $ANALYZABLE_FILES' to see details"
        exit 1
    fi
    
    if [ "$ISSUES_COUNT" -gt {} ]; then
        echo "⚠️  Found $ISSUES_COUNT issues (threshold: {}). Commit blocked."
        echo "💡 Consider fixing issues or use --no-verify to skip"
        exit 1
    fi
    
    if [ "$ISSUES_COUNT" -gt 0 ]; then
        echo "✅ Analysis passed with $ISSUES_COUNT issues (under threshold)"
    else
        echo "✅ Analysis passed - no issues found"
    fi
elif [ $ANALYSIS_EXIT_CODE -eq 124 ]; then
    echo "⏰ Analysis timed out after {} seconds"
    echo "💡 Consider analyzing fewer files or increasing timeout"
    exit 1
else
    echo "❌ Analysis failed:"
    cat /tmp/uveddi-precommit.err
    exit 1
fi

# Cleanup
rm -f /tmp/uveddi-precommit.json /tmp/uveddi-precommit.err

echo "🎉 Pre-commit analysis completed successfully"
"#,
            self.config.timeout_seconds,
            self.config.min_confidence,
            self.config.timeout_seconds,
            if self.config.show_progress { "terminal" } else { "silent" },
            if self.config.fail_fast { "--fail-fast" } else { "" },
            self.config.max_issues.unwrap_or(999),
            self.config.max_issues.unwrap_or(999),
            self.config.timeout_seconds,
        )
    }
    
    /// Generate pre-push hook script
    fn generate_pre_push_script(&self) -> String {
        format!(r#"#!/bin/sh
# Uveddi Hook - Pre-push Analysis
# Generated automatically by Uveddi
# DO NOT EDIT MANUALLY

set -e

echo "🚀 Running Uveddi pre-push analysis..."

# Get the remote and branch being pushed to
remote="$1"
url="$2"

# Check if we have any commits to push
if ! git log @{{u}}..HEAD --oneline > /dev/null 2>&1; then
    echo "✅ No new commits to analyze"
    exit 0
fi

# Check if any commit has skip patterns
if git log @{{u}}..HEAD --pretty=%B | grep -E "(WIP:|\\[skip ci\\]|\\[no-analyze\\])" > /dev/null 2>&1; then
    echo "⏭️  Skipping analysis (skip pattern detected in commit history)"
    exit 0
fi

# Run full analysis on the repository
echo "🔍 Analyzing entire repository..."
timeout {} uveddi analyze . \
    --output-format json \
    --confidence-threshold {} \
    --timeout {} \
    --progress-format {} \
    {} > /tmp/uveddi-prepush.json 2>/tmp/uveddi-prepush.err

ANALYSIS_EXIT_CODE=$?

if [ $ANALYSIS_EXIT_CODE -eq 0 ]; then
    # Parse results
    ISSUES_COUNT=$(cat /tmp/uveddi-prepush.json | jq -r '.issues | length // 0' 2>/dev/null || echo "0")
    CRITICAL_COUNT=$(cat /tmp/uveddi-prepush.json | jq -r '.critical_issues // 0' 2>/dev/null || echo "0")
    
    if [ "$CRITICAL_COUNT" -gt 0 ]; then
        echo "❌ Found $CRITICAL_COUNT critical issues. Push blocked."
        echo "🔧 Run 'uveddi analyze .' to see details"
        exit 1
    fi
    
    if [ "$ISSUES_COUNT" -gt {} ]; then
        echo "⚠️  Found $ISSUES_COUNT issues (threshold: {}). Push blocked."
        echo "💡 Fix issues or use --no-verify to skip"
        exit 1
    fi
    
    echo "✅ Pre-push analysis passed ($ISSUES_COUNT issues found, under threshold)"
elif [ $ANALYSIS_EXIT_CODE -eq 124 ]; then
    echo "⏰ Analysis timed out after {} seconds"
    exit 1
else
    echo "❌ Analysis failed:"
    cat /tmp/uveddi-prepush.err
    exit 1
fi

# Cleanup
rm -f /tmp/uveddi-prepush.json /tmp/uveddi-prepush.err

echo "🎉 Pre-push analysis completed successfully"
"#,
            self.config.timeout_seconds,
            self.config.min_confidence,
            self.config.timeout_seconds,
            if self.config.show_progress { "terminal" } else { "silent" },
            if self.config.fail_fast { "--fail-fast" } else { "" },
            self.config.max_issues.unwrap_or(999),
            self.config.max_issues.unwrap_or(999),
            self.config.timeout_seconds,
        )
    }
    
    /// Generate commit-msg hook script
    fn generate_commit_msg_script(&self) -> String {
        r#"#!/bin/sh
# Uveddi Hook - Commit Message Validation
# Generated automatically by Uveddi
# DO NOT EDIT MANUALLY

COMMIT_MSG_FILE="$1"
COMMIT_MSG=$(cat "$COMMIT_MSG_FILE")

echo "📝 Validating commit message..."

# Check for skip patterns
if echo "$COMMIT_MSG" | grep -E "(WIP:|\\[skip ci\\]|\\[no-analyze\\])" > /dev/null 2>&1; then
    echo "⏭️  Skipping validation (skip pattern detected)"
    exit 0
fi

# Basic commit message validation
if [ ${#COMMIT_MSG} -lt 10 ]; then
    echo "❌ Commit message too short (minimum 10 characters)"
    echo "💡 Provide a more descriptive commit message"
    exit 1
fi

if [ ${#COMMIT_MSG} -gt 72 ]; then
    echo "⚠️  Commit message is quite long (${#COMMIT_MSG} characters)"
    echo "💡 Consider keeping the first line under 72 characters"
fi

# Check for conventional commit format (optional)
if echo "$COMMIT_MSG" | grep -E "^(feat|fix|docs|style|refactor|perf|test|chore|ci|build|revert)(\(.+\))?: .+" > /dev/null 2>&1; then
    echo "✅ Conventional commit format detected"
fi

echo "✅ Commit message validation passed"
"#.to_string()
    }
    
    /// Make a file executable
    fn make_executable(&self, path: &Path) -> Result<(), UveddiError> {
        #[cfg(unix)]
        {
            let mut perms = fs::metadata(path).map_err(|e| UveddiError::io_error("reading file metadata", &path.to_string_lossy(), e))?.permissions();
            
            perms.set_mode(0o755);
            
            fs::set_permissions(path, perms).map_err(|e| UveddiError::io_error("setting file permissions", &path.to_string_lossy(), e))?;
        }
        
        Ok(())
    }
    
    /// Save hook configuration to file
    async fn save_hook_config(&self) -> Result<PathBuf, UveddiError> {
        let config_path = self.repo_path.join(".uveddi").join("hooks.toml");
        
        // Ensure config directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).map_err(|e| UveddiError::io_error("creating config directory", &parent.to_string_lossy(), e))?;
        }
        
        let toml_content = toml::to_string_pretty(&self.config)
            .map_err(|e| UveddiError::config_error(&format!("Failed to serialize hook config: {}", e), "system"))?;
        
        let header = format!(
            r#"# Uveddi Git Hooks Configuration
# Generated on: {}
# 
# This file configures the behavior of Uveddi Git hooks.
# You can modify these settings and reinstall hooks with 'uveddi hooks install --force'

"#,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );
        
        let full_content = format!("{}{}", header, toml_content);
        
        fs::write(&config_path, full_content).map_err(|e| UveddiError::io_error("writing hook config", &config_path.to_string_lossy(), e))?;
        
        Ok(config_path)
    }
    
    /// Uninstall Uveddi hooks
    pub async fn uninstall_hooks(&self) -> Result<Vec<HookType>, UveddiError> {
        let mut removed_hooks = Vec::new();
        
        let hook_types = [
            HookType::PreCommit,
            HookType::PrePush,
            HookType::CommitMsg,
        ];
        
        for hook_type in &hook_types {
            let hook_path = self.hooks_dir.join(hook_type.filename());
            
            if hook_path.exists() {
                // Check if it's an Uveddi hook
                if let Ok(content) = fs::read_to_string(&hook_path) {
                    if content.contains("# Uveddi Hook") {
                        fs::remove_file(&hook_path).map_err(|e| UveddiError::io_error("removing hook", &hook_path.to_string_lossy(), e))?;
                        
                        removed_hooks.push(hook_type.clone());
                        
                        // Restore backup if it exists
                        let backup_path = hook_path.with_extension("backup");
                        if backup_path.exists() {
                            fs::rename(&backup_path, &hook_path).map_err(|e| UveddiError::io_error("restoring backup hook", &backup_path.to_string_lossy(), e))?;
                        }
                    }
                }
            }
        }
        
        Ok(removed_hooks)
    }
    
    /// List installed Uveddi hooks
    pub async fn list_hooks(&self) -> Result<Vec<(HookType, bool, PathBuf)>, UveddiError> {
        let mut hooks = Vec::new();
        
        let hook_types = [
            HookType::PreCommit,
            HookType::PrePush, 
            HookType::CommitMsg,
        ];
        
        for hook_type in &hook_types {
            let hook_path = self.hooks_dir.join(hook_type.filename());
            let is_uveddi_hook = if hook_path.exists() {
                fs::read_to_string(&hook_path)
                    .map(|content| content.contains("# Uveddi Hook"))
                    .unwrap_or(false)
            } else {
                false
            };
            
            hooks.push((hook_type.clone(), is_uveddi_hook, hook_path));
        }
        
        Ok(hooks)
    }
    
    /// Load hook configuration from file
    pub fn load_config(repo_path: &Path) -> Result<HookConfig, UveddiError> {
        let config_path = repo_path.join(".uveddi").join("hooks.toml");
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path).map_err(|e| UveddiError::io_error("reading hook config", &config_path.to_string_lossy(), e))?;
            
            toml::from_str(&content).map_err(|e| UveddiError::config_error(
                &format!("Failed to parse hook config: {}", e), "system"
            ))
        } else {
            Ok(HookConfig::default())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_hook_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().to_path_buf();
        
        // Create .git directory
        fs::create_dir_all(repo_path.join(".git/hooks")).unwrap();
        
        let config = HookConfig::default();
        let manager = HookManager::new(repo_path, config);
        
        assert!(manager.is_ok());
    }
    
    #[test]
    fn test_pre_commit_script_generation() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().to_path_buf();
        fs::create_dir_all(repo_path.join(".git/hooks")).unwrap();
        
        let config = HookConfig::default();
        let manager = HookManager::new(repo_path, config).unwrap();
        
        let script = manager.generate_pre_commit_script();
        assert!(script.contains("# Uveddi Hook"));
        assert!(script.contains("pre-commit"));
        assert!(script.contains("git diff --cached"));
    }
}