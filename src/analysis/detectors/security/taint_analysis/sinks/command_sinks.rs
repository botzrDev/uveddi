//! Detection of command execution sinks (OS commands, shell injection)

use super::TaintSinkDetector;
use crate::analysis::detectors::security::taint_analysis::types::TaintSink;
use crate::analysis::detectors::security::types::SecurityIssueType;
use crate::ast::{ParsedFile, SourceLanguage};
use crate::analysis::AnalysisError;
use std::collections::HashMap;

/// Detector for command execution sinks
pub struct CommandSinkDetector {
    patterns: HashMap<SourceLanguage, Vec<String>>,
}

impl CommandSinkDetector {
    pub fn new() -> Self {
        let mut detector = Self {
            patterns: HashMap::new(),
        };
        detector.initialize_patterns();
        detector
    }

    fn initialize_patterns(&mut self) {
        // Rust patterns
        self.patterns.insert(
            SourceLanguage::Rust,
            vec![
                "std::process::Command::new".to_string(),
                "std::process::Command::arg".to_string(),
                "std::process::Command::args".to_string(),
                "tokio::process::Command".to_string(),
                "std::process::Stdio".to_string(),
                "shell_escape::".to_string(),
                "shlex::".to_string(),
            ],
        );

        // Python patterns
        self.patterns.insert(
            SourceLanguage::Python,
            vec![
                "os.system".to_string(),
                "os.popen".to_string(),
                "os.execv".to_string(),
                "os.execve".to_string(),
                "os.spawnv".to_string(),
                "subprocess.run".to_string(),
                "subprocess.call".to_string(),
                "subprocess.check_call".to_string(),
                "subprocess.check_output".to_string(),
                "subprocess.Popen".to_string(),
                "eval(".to_string(),
                "exec(".to_string(),
                "compile(".to_string(),
                "__import__".to_string(),
            ],
        );

        // JavaScript/TypeScript patterns
        self.patterns.insert(
            SourceLanguage::JavaScript,
            vec![
                "child_process.exec".to_string(),
                "child_process.execSync".to_string(),
                "child_process.spawn".to_string(),
                "child_process.spawnSync".to_string(),
                "child_process.execFile".to_string(),
                "child_process.fork".to_string(),
                "require('child_process')".to_string(),
                "eval(".to_string(),
                "Function(".to_string(),
                "vm.runInThisContext".to_string(),
                "vm.runInNewContext".to_string(),
            ],
        );

        self.patterns.insert(
            SourceLanguage::TypeScript,
            self.patterns.get(&SourceLanguage::JavaScript).unwrap().clone(),
        );
    }

    /// Create taint sinks for OS command execution
    fn create_command_execution_sinks(&self, language: SourceLanguage) -> Vec<TaintSink> {
        match language {
            SourceLanguage::Rust => vec![
                TaintSink::new(
                    "rust_command_new".to_string(),
                    "std::process::Command::new".to_string(),
                    SecurityIssueType::Injection,
                    "Process command with user-controlled program".to_string(),
                ).with_language(language)
                .with_vulnerable_params(vec![0]), // First parameter is the command
                TaintSink::new(
                    "rust_command_arg".to_string(),
                    "std::process::Command::arg".to_string(),
                    SecurityIssueType::Injection,
                    "Command argument with user input".to_string(),
                ).with_language(language),
                TaintSink::new(
                    "rust_command_args".to_string(),
                    "std::process::Command::args".to_string(),
                    SecurityIssueType::Injection,
                    "Command arguments with user input".to_string(),
                ).with_language(language),
                TaintSink::new(
                    "rust_tokio_command".to_string(),
                    "tokio::process::Command".to_string(),
                    SecurityIssueType::Injection,
                    "Async process command with user input".to_string(),
                ).with_language(language),
            ],
            SourceLanguage::Python => vec![
                TaintSink::new(
                    "python_os_system".to_string(),
                    "os.system".to_string(),
                    SecurityIssueType::Injection,
                    "OS system command with user input".to_string(),
                ).with_language(language),
                TaintSink::new(
                    "python_os_popen".to_string(),
                    "os.popen".to_string(),
                    SecurityIssueType::Injection,
                    "OS popen command with user input".to_string(),
                ).with_language(language),
                TaintSink::new(
                    "python_subprocess_run".to_string(),
                    "subprocess.run".to_string(),
                    SecurityIssueType::Injection,
                    "Subprocess run with user-controlled command".to_string(),
                ).with_language(language),
                TaintSink::new(
                    "python_subprocess_call".to_string(),
                    "subprocess.call".to_string(),
                    SecurityIssueType::Injection,
                    "Subprocess call with user input".to_string(),
                ).with_language(language),
                TaintSink::new(
                    "python_subprocess_popen".to_string(),
                    "subprocess.Popen".to_string(),
                    SecurityIssueType::Injection,
                    "Subprocess Popen with user command".to_string(),
                ).with_language(language),
            ],
            SourceLanguage::JavaScript | SourceLanguage::TypeScript => vec![
                TaintSink::new(
                    "js_child_exec".to_string(),
                    "child_process.exec".to_string(),
                    SecurityIssueType::Injection,
                    "Child process exec with user command".to_string(),
                ).with_language(language),
                TaintSink::new(
                    "js_child_exec_sync".to_string(),
                    "child_process.execSync".to_string(),
                    SecurityIssueType::Injection,
                    "Synchronous process exec with user input".to_string(),
                ).with_language(language),
                TaintSink::new(
                    "js_child_spawn".to_string(),
                    "child_process.spawn".to_string(),
                    SecurityIssueType::Injection,
                    "Child process spawn with user command".to_string(),
                ).with_language(language),
                TaintSink::new(
                    "js_child_exec_file".to_string(),
                    "child_process.execFile".to_string(),
                    SecurityIssueType::Injection,
                    "Child process execFile with user input".to_string(),
                ).with_language(language),
            ],
        }
    }

    /// Create taint sinks for code evaluation and execution
    fn create_code_evaluation_sinks(&self, language: SourceLanguage) -> Vec<TaintSink> {
        match language {
            SourceLanguage::Python => vec![
                TaintSink::new(
                    "python_eval".to_string(),
                    "eval(".to_string(),
                    SecurityIssueType::Injection,
                    "Python eval with user-controlled code".to_string(),
                ).with_language(language),
                TaintSink::new(
                    "python_exec".to_string(),
                    "exec(".to_string(),
                    SecurityIssueType::Injection,
                    "Python exec with user-controlled code".to_string(),
                ).with_language(language),
                TaintSink::new(
                    "python_compile".to_string(),
                    "compile(".to_string(),
                    SecurityIssueType::Injection,
                    "Python compile with user code".to_string(),
                ).with_language(language),
                TaintSink::new(
                    "python_import".to_string(),
                    "__import__".to_string(),
                    SecurityIssueType::Injection,
                    "Dynamic import with user-controlled module".to_string(),
                ).with_language(language),
            ],
            SourceLanguage::JavaScript | SourceLanguage::TypeScript => vec![
                TaintSink::new(
                    "js_eval".to_string(),
                    "eval(".to_string(),
                    SecurityIssueType::Injection,
                    "JavaScript eval with user input".to_string(),
                ).with_language(language),
                TaintSink::new(
                    "js_function_constructor".to_string(),
                    "Function(".to_string(),
                    SecurityIssueType::Injection,
                    "Function constructor with user code".to_string(),
                ).with_language(language),
                TaintSink::new(
                    "js_vm_run_context".to_string(),
                    "vm.runInThisContext".to_string(),
                    SecurityIssueType::Injection,
                    "VM code execution with user input".to_string(),
                ).with_language(language),
                TaintSink::new(
                    "js_vm_run_new_context".to_string(),
                    "vm.runInNewContext".to_string(),
                    SecurityIssueType::Injection,
                    "VM new context execution with user code".to_string(),
                ).with_language(language),
            ],
        }
    }

    /// Create taint sinks for process execution variants
    fn create_process_execution_sinks(&self, language: SourceLanguage) -> Vec<TaintSink> {
        match language {
            SourceLanguage::Python => vec![
                TaintSink::new(
                    "python_os_execv".to_string(),
                    "os.execv".to_string(),
                    SecurityIssueType::Injection,
                    "OS execv with user-controlled arguments".to_string(),
                ).with_language(language),
                TaintSink::new(
                    "python_os_execve".to_string(),
                    "os.execve".to_string(),
                    SecurityIssueType::Injection,
                    "OS execve with user environment".to_string(),
                ).with_language(language),
                TaintSink::new(
                    "python_os_spawnv".to_string(),
                    "os.spawnv".to_string(),
                    SecurityIssueType::Injection,
                    "OS spawnv with user arguments".to_string(),
                ).with_language(language),
            ],
            SourceLanguage::JavaScript | SourceLanguage::TypeScript => vec![
                TaintSink::new(
                    "js_child_fork".to_string(),
                    "child_process.fork".to_string(),
                    SecurityIssueType::Injection,
                    "Child process fork with user module".to_string(),
                ).with_language(language),
            ],
        }
    }
}

impl TaintSinkDetector for CommandSinkDetector {
    fn detect_sinks(&self, file: &ParsedFile) -> Result<Vec<TaintSink>, AnalysisError> {
        let mut sinks = Vec::new();

        // Get different types of command execution sinks
        sinks.extend(self.create_command_execution_sinks(file.language));
        sinks.extend(self.create_code_evaluation_sinks(file.language));
        sinks.extend(self.create_process_execution_sinks(file.language));

        // TODO: Implement actual AST-based detection
        // This would involve:
        // 1. Walking the AST to find function calls
        // 2. Matching against command execution patterns
        // 3. Creating TaintSink objects with proper location info
        // 4. Identifying which parameters are vulnerable
        // 5. Special handling for shell=True in subprocess calls

        Ok(sinks)
    }

    fn get_patterns_for_language(&self, language: SourceLanguage) -> Vec<String> {
        self.patterns
            .get(&language)
            .cloned()
            .unwrap_or_else(Vec::new)
    }
}

impl Default for CommandSinkDetector {
    fn default() -> Self {
        Self::new()
    }
}