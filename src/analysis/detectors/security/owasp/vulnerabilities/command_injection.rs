//! Command Injection Vulnerability Detector
//!
//! Specialized detector for command injection vulnerabilities that occur when
//! user input is used to construct system commands without proper sanitization.

use crate::analysis::detectors::security::owasp::types::{
    OwaspCategory, OwaspCategoryDetector, OwaspVulnerability,
};
use crate::analysis::detectors::security::types::{
    SecurityIssueType, SecurityLocation, SecuritySeverity, VulnerabilityMetadata,
};
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;

/// Command injection pattern definition
#[derive(Debug, Clone)]
pub struct CommandInjectionPattern {
    pub pattern: String,
    pub description: String,
    pub confidence: f64,
    pub severity: SecuritySeverity,
    pub command_type: CommandInjectionType,
    pub execution_context: String,
}

#[derive(Debug, Clone)]
pub enum CommandInjectionType {
    DirectExecution,
    ShellExecution,
    ProcessSpawn,
    ScriptExecution,
    SystemCall,
    FileSystemCommand,
}

impl CommandInjectionType {
    fn description(&self) -> &'static str {
        match self {
            CommandInjectionType::DirectExecution => "Direct command execution vulnerability",
            CommandInjectionType::ShellExecution => "Shell command execution vulnerability",
            CommandInjectionType::ProcessSpawn => "Process spawning vulnerability",
            CommandInjectionType::ScriptExecution => "Script execution vulnerability",
            CommandInjectionType::SystemCall => "System call vulnerability",
            CommandInjectionType::FileSystemCommand => "File system command vulnerability",
        }
    }
}

/// Specialized command injection detector
pub struct CommandInjectionDetector {
    patterns: HashMap<SourceLanguage, Vec<CommandInjectionPattern>>,
}

impl CommandInjectionDetector {
    pub fn new() -> Self {
        Self {
            patterns: Self::init_patterns(),
        }
    }

    fn init_patterns() -> HashMap<SourceLanguage, Vec<CommandInjectionPattern>> {
        let mut patterns = HashMap::new();

        patterns.insert(SourceLanguage::Rust, Self::rust_patterns());
        patterns.insert(SourceLanguage::Python, Self::python_patterns());
        patterns.insert(SourceLanguage::JavaScript, Self::javascript_patterns());
        patterns.insert(SourceLanguage::TypeScript, Self::javascript_patterns());

        patterns
    }

    fn rust_patterns() -> Vec<CommandInjectionPattern> {
        vec![
            CommandInjectionPattern {
                pattern: r#"Command::new\(.*\+.*\)"#.to_string(),
                description: "String concatenation in Command::new()".to_string(),
                confidence: 0.85,
                severity: SecuritySeverity::High,
                command_type: CommandInjectionType::ProcessSpawn,
                execution_context: "Process builder".to_string(),
            },
            CommandInjectionPattern {
                pattern: r#"Command::new\(format!"#.to_string(),
                description: "String formatting in Command::new()".to_string(),
                confidence: 0.9,
                severity: SecuritySeverity::High,
                command_type: CommandInjectionType::ProcessSpawn,
                execution_context: "Process builder with formatting".to_string(),
            },
        ]
    }

    fn python_patterns() -> Vec<CommandInjectionPattern> {
        vec![
            CommandInjectionPattern {
                pattern: r#"os\.system\(.*\+.*\)"#.to_string(),
                description: "String concatenation in os.system()".to_string(),
                confidence: 0.9,
                severity: SecuritySeverity::Critical,
                command_type: CommandInjectionType::SystemCall,
                execution_context: "os.system call".to_string(),
            },
            CommandInjectionPattern {
                pattern: r#"os\.system\(f".*"#.to_string(),
                description: "F-string in os.system()".to_string(),
                confidence: 0.95,
                severity: SecuritySeverity::Critical,
                command_type: CommandInjectionType::SystemCall,
                execution_context: "os.system with f-string".to_string(),
            },
        ]
    }

    fn javascript_patterns() -> Vec<CommandInjectionPattern> {
        vec![
            CommandInjectionPattern {
                pattern: r#"exec\(.*\+.*\)"#.to_string(),
                description: "String concatenation in exec()".to_string(),
                confidence: 0.9,
                severity: SecuritySeverity::Critical,
                command_type: CommandInjectionType::DirectExecution,
                execution_context: "child_process.exec".to_string(),
            },
            CommandInjectionPattern {
                pattern: r#"exec\(`.*\$\{.*\}.*`\)"#.to_string(),
                description: "Template literals in exec()".to_string(),
                confidence: 0.95,
                severity: SecuritySeverity::Critical,
                command_type: CommandInjectionType::DirectExecution,
                execution_context: "exec with template literals".to_string(),
            },
        ]
    }

    fn analyze_line(
        &self,
        line: &str,
        line_number: usize,
        patterns: &[CommandInjectionPattern],
    ) -> Vec<OwaspVulnerability> {
        let mut vulnerabilities = Vec::new();

        for pattern in patterns {
            if let Ok(regex) = regex::Regex::new(&pattern.pattern) {
                if regex.is_match(line) {
                    let line_i32 = line_number as i32;
                    let location = SecurityLocation::new(
                        PathBuf::from("unknown"), // Will be updated by caller
                        line_i32,
                        line_i32,
                    );

                    let mut metadata = VulnerabilityMetadata::new();
                    metadata.add_metadata(
                        "command_type".to_string(),
                        pattern.command_type.description().to_string(),
                    );
                    metadata.add_metadata(
                        "execution_context".to_string(),
                        pattern.execution_context.clone(),
                    );
                    metadata.add_metadata("pattern_matched".to_string(), pattern.pattern.clone());
                    metadata.add_metadata("line_content".to_string(), line.trim().to_string());

                    let remediation = Self::generate_remediation(
                        &pattern.command_type,
                        &pattern.execution_context,
                    );

                    let vulnerability = OwaspVulnerability::new(
                        OwaspCategory::Injection,
                        SecurityIssueType::Injection,
                        format!("Command Injection: {}", pattern.command_type.description()),
                        pattern.description.clone(),
                        location,
                    )
                    .with_severity(pattern.severity)
                    .with_confidence(pattern.confidence)
                    .with_remediation(remediation)
                    .with_metadata(metadata);

                    vulnerabilities.push(vulnerability);
                }
            }
        }

        vulnerabilities
    }

    fn generate_remediation(command_type: &CommandInjectionType, context: &str) -> String {
        let base_advice = match command_type {
            CommandInjectionType::DirectExecution => {
                "Avoid using exec() or system() functions with user input. Use parameterized alternatives."
            }
            CommandInjectionType::ShellExecution => {
                "Disable shell execution or use whitelist validation. Prefer direct process execution."
            }
            CommandInjectionType::ProcessSpawn => {
                "Use argument arrays instead of string concatenation. Validate all input parameters."
            }
            CommandInjectionType::SystemCall => {
                "Replace system calls with safer library functions. Implement strict input validation."
            }
            _ => "Implement proper input validation and use safer execution methods.",
        };

        let context_advice = match context {
            c if c.contains("format") => " Avoid string formatting in command construction.",
            c if c.contains("concat") => " Use argument arrays instead of string concatenation.",
            c if c.contains("template") => {
                " Replace template literals with parameterized execution."
            }
            c if c.contains("shell") => {
                " Disable shell execution or implement command whitelisting."
            }
            _ => "",
        };

        format!("{}{} Consider using dedicated libraries for system interaction instead of direct command execution.", base_advice, context_advice)
    }
}

#[async_trait]
impl OwaspCategoryDetector for CommandInjectionDetector {
    async fn detect(&self, file: &ParsedFile) -> Result<Vec<OwaspVulnerability>, AnalysisError> {
        let patterns = match self.patterns.get(&file.language) {
            Some(patterns) => patterns,
            None => return Ok(Vec::new()),
        };

        let mut vulnerabilities = Vec::new();

        for (line_number, line) in file.source().lines().enumerate() {
            let mut line_vulnerabilities = self.analyze_line(line, line_number + 1, patterns);

            // Update file path in location
            for vuln in &mut line_vulnerabilities {
                vuln.location.file_path = file.file_path.to_path_buf();
            }

            vulnerabilities.extend(line_vulnerabilities);
        }

        Ok(vulnerabilities)
    }
}

impl Default for CommandInjectionDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_command_injection_detection() {
        let detector = CommandInjectionDetector::new();
        let content = r#"
            os.system(f"rm -rf {user_path}")
            exec(`ls -la ${userDir}`);
        "#;

        let file = ParsedFile {
            file_path: Arc::new(PathBuf::from("test.py")),
            language: SourceLanguage::Python,
            content: content.to_string(),
            tree: None,
        };

        let vulnerabilities = detector.detect(&file).await.unwrap();
        assert!(vulnerabilities.len() >= 1);
    }
}
