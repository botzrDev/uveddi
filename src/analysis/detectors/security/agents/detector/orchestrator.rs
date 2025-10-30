//! SecurityOrchestrator - Main coordinator for multi-agent security analysis
//!
//! This module implements the SecurityOrchestrator that coordinates security analysis
//! through a multi-agent architecture with task decomposition and result synthesis.

use super::super::analysis::{
    AgentDetector, AnalysisConfig, AnalysisModule, BehaviorAnalyzer,
    PatternMatcher as AnalysisPatternMatcher,
};
use super::super::config::{AgentConfig, MultiAgentConfig};
use super::super::language_support::{get_language_analyzer, LanguageAgentAnalyzer};
use super::super::patterns::{
    AgentPatternDatabase, MaliciousPatternDatabase, PatternConfig, PatternMatcher,
};
use super::super::types::*;
use super::agents::{ConfigAnalysisAgent, DependencyAgent, TaintAnalysisAgent, ValidationAgent};
use crate::analysis::detectors::security::core::{
    SecurityAnalysisResult, SecurityContext, VulnerabilityDatabase,
};
use crate::analysis::detectors::security::knowledge_graph::SecurityKnowledgeGraph;
use crate::analysis::detectors::security::owasp::OwaspVulnerability;
use crate::analysis::AnalysisError;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{mpsc, Mutex, RwLock};
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

/// Main orchestrator agent that coordinates the multi-agent security analysis
pub struct SecurityOrchestrator {
    config: MultiAgentConfig,
    agents: HashMap<String, Box<dyn SecurityAgent>>,
    message_tx: mpsc::UnboundedSender<AgentMessage>,
    message_rx: Arc<Mutex<mpsc::UnboundedReceiver<AgentMessage>>>,
    knowledge_graph: Arc<SecurityKnowledgeGraph>,
    vulnerability_db: Arc<VulnerabilityDatabase>,
    active_tasks: Arc<RwLock<HashMap<String, TaskMetadata>>>,
    agent_detector: AgentDetector,
    behavior_analyzer: BehaviorAnalyzer,
    pattern_matcher: AnalysisPatternMatcher,
    agent_patterns: AgentPatternDatabase,
    malicious_patterns: MaliciousPatternDatabase,
}

impl SecurityOrchestrator {
    pub fn new(
        config: MultiAgentConfig,
        knowledge_graph: Arc<SecurityKnowledgeGraph>,
        vulnerability_db: Arc<VulnerabilityDatabase>,
    ) -> Result<Self, AnalysisError> {
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        let message_rx = Arc::new(Mutex::new(message_rx));

        // Initialize analysis modules
        let analysis_config = AnalysisConfig::default();
        let pattern_config = PatternConfig::default();

        let agent_detector = AgentDetector::new(analysis_config.clone());
        let behavior_analyzer = BehaviorAnalyzer::new(analysis_config.clone());
        let pattern_matcher = AnalysisPatternMatcher::new(analysis_config.clone())?;
        let agent_patterns = AgentPatternDatabase::new(pattern_config.clone());
        let malicious_patterns = MaliciousPatternDatabase::new(pattern_config);

        let mut orchestrator = Self {
            config,
            agents: HashMap::new(),
            message_tx,
            message_rx,
            knowledge_graph,
            vulnerability_db,
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
            agent_detector,
            behavior_analyzer,
            pattern_matcher,
            agent_patterns,
            malicious_patterns,
        };

        orchestrator.initialize_agents()?;
        Ok(orchestrator)
    }

    /// Initialize all configured agents
    fn initialize_agents(&mut self) -> Result<(), AnalysisError> {
        info!("Initializing security analysis agents");

        if self.config.enable_taint_agent {
            let agent = TaintAnalysisAgent::new(
                AgentConfig::new("TaintAgent".to_string()),
                self.message_tx.clone(),
                self.knowledge_graph.clone(),
            )?;
            self.agents
                .insert("TaintAgent".to_string(), Box::new(agent));
        }

        if self.config.enable_config_agent {
            let agent = ConfigAnalysisAgent::new(
                AgentConfig::new("ConfigAgent".to_string()),
                self.message_tx.clone(),
                self.knowledge_graph.clone(),
            )?;
            self.agents
                .insert("ConfigAgent".to_string(), Box::new(agent));
        }

        if self.config.enable_dependency_agent {
            let agent = DependencyAgent::new(
                AgentConfig::new("DependencyAgent".to_string()),
                self.message_tx.clone(),
                self.vulnerability_db.clone(),
            )?;
            self.agents
                .insert("DependencyAgent".to_string(), Box::new(agent));
        }

        if self.config.enable_validation_agent {
            let agent = ValidationAgent::new(
                AgentConfig::new("ValidationAgent".to_string()),
                self.message_tx.clone(),
            )?;
            self.agents
                .insert("ValidationAgent".to_string(), Box::new(agent));
        }

        info!("Initialized {} agents", self.agents.len());
        Ok(())
    }

    /// Analyze a security context using the multi-agent system
    #[instrument(skip(self, context))]
    pub async fn analyze_file(
        &self,
        context: &SecurityContext,
    ) -> Result<SecurityAnalysisResult, AnalysisError> {
        info!(
            "Starting multi-agent security analysis for file: {:?}",
            context.file_path
        );

        let _analysis_id = Uuid::new_v4().to_string();

        // Run core agent detection analysis
        let mut all_issues = Vec::new();

        // Agent detection
        let agent_issues = self.agent_detector.analyze(context).await?;
        all_issues.extend(agent_issues);

        // Behavior analysis
        let behavior_issues = self.behavior_analyzer.analyze(context).await?;
        all_issues.extend(behavior_issues);

        // Pattern matching
        let pattern_issues = self.pattern_matcher.analyze(context).await?;
        all_issues.extend(pattern_issues);

        // Agent pattern matching
        let agent_pattern_issues = self.agent_patterns.match_patterns(context).await?;
        all_issues.extend(agent_pattern_issues);

        // Malicious pattern matching
        let malicious_pattern_issues = self.malicious_patterns.match_patterns(context).await?;
        all_issues.extend(malicious_pattern_issues);

        // Language-specific analysis
        if let Some(lang_analyzer) = get_language_analyzer(context.language) {
            let lang_issues = lang_analyzer.analyze(context).await?;
            all_issues.extend(lang_issues);
        }

        // Decompose the analysis into subtasks for specialized agents
        let subtasks = self.decompose_analysis_tasks(context).await?;
        info!("Decomposed analysis into {} subtasks", subtasks.len());

        // Execute subtasks concurrently
        let mut task_handles = Vec::new();
        for subtask in subtasks {
            let handle = self.execute_subtask(subtask).await?;
            task_handles.push(handle);
        }

        // Collect and synthesize results
        let mut agent_results = Vec::new();
        for handle in task_handles {
            match handle.await {
                Ok(Ok(result)) => agent_results.push(result),
                Ok(Err(e)) => {
                    warn!("Subtask failed: {}", e);
                }
                Err(e) => {
                    warn!("Task handle join failed: {}", e);
                }
            }
        }

        // Convert issues to OWASP vulnerabilities and synthesize results
        let vulnerabilities = self.convert_issues_to_vulnerabilities(all_issues);
        let mut final_result = SecurityAnalysisResult::new();
        final_result.vulnerabilities.extend(vulnerabilities);

        // Synthesize agent results
        let agent_result = self.synthesize_results(agent_results, context).await?;
        final_result
            .vulnerabilities
            .extend(agent_result.vulnerabilities);

        // Apply deduplication and final filtering
        final_result = self.deduplicate_and_filter(final_result).await?;

        info!(
            "Multi-agent analysis completed: {} vulnerabilities found",
            final_result.vulnerabilities.len()
        );
        Ok(final_result)
    }

    fn convert_issues_to_vulnerabilities(
        &self,
        issues: Vec<crate::analysis::detectors::security::types::SecurityIssue>,
    ) -> Vec<OwaspVulnerability> {
        issues
            .into_iter()
            .map(|issue| OwaspVulnerability {
                category: crate::analysis::detectors::security::owasp::OwaspCategory::Injection,
                issue_type: issue.issue_type,
                title: issue.title,
                description: issue.description,
                severity: issue.severity,
                confidence_score: issue.confidence_score,
                location: crate::analysis::detectors::security::types::SecurityLocation {
                    file_path: issue.location.file_path,
                    start_line: issue.location.start_line,
                    end_line: issue.location.end_line,
                    start_column: issue.location.start_column,
                    end_column: issue.location.end_column,
                    function_name: issue.location.function_name,
                    class_name: issue.location.class_name,
                    module_name: issue.location.module_name,
                },
                remediation: issue.remediation,
                metadata: issue.metadata,
                architectural_correlation: issue
                    .correlation_id
                    .map(|id| vec![id])
                    .unwrap_or_default(),
            })
            .collect()
    }

    async fn decompose_analysis_tasks(
        &self,
        context: &SecurityContext,
    ) -> Result<Vec<SubTask>, AnalysisError> {
        let mut subtasks = Vec::new();

        // Always include OWASP analysis as it's comprehensive
        subtasks.push(SubTask {
            task_id: Uuid::new_v4().to_string(),
            task_type: TaskType::OwaspAnalysis,
            agent_id: "OwaspAgent".to_string(),
            context: context.clone(),
            priority: 1,
        });

        if self.config.enable_taint_agent {
            subtasks.push(SubTask {
                task_id: Uuid::new_v4().to_string(),
                task_type: TaskType::TaintAnalysis,
                agent_id: "TaintAgent".to_string(),
                context: context.clone(),
                priority: 2,
            });
        }

        if self.config.enable_config_agent && self.is_configuration_file(context) {
            subtasks.push(SubTask {
                task_id: Uuid::new_v4().to_string(),
                task_type: TaskType::ConfigurationAnalysis,
                agent_id: "ConfigAgent".to_string(),
                context: context.clone(),
                priority: 3,
            });
        }

        if self.config.enable_dependency_agent && self.is_dependency_file(context) {
            subtasks.push(SubTask {
                task_id: Uuid::new_v4().to_string(),
                task_type: TaskType::DependencyAnalysis,
                agent_id: "DependencyAgent".to_string(),
                context: context.clone(),
                priority: 4,
            });
        }

        Ok(subtasks)
    }

    fn is_configuration_file(&self, context: &SecurityContext) -> bool {
        let file_name = context
            .file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        matches!(
            file_name,
            "config.toml"
                | "Cargo.toml"
                | "package.json"
                | "requirements.txt"
                | "settings.py"
                | "Dockerfile"
        ) || file_name.ends_with(".toml")
            || file_name.ends_with(".json")
            || file_name.ends_with(".yml")
            || file_name.ends_with(".yaml")
    }

    fn is_dependency_file(&self, context: &SecurityContext) -> bool {
        let file_name = context
            .file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        matches!(
            file_name,
            "Cargo.toml"
                | "Cargo.lock"
                | "package.json"
                | "package-lock.json"
                | "requirements.txt"
                | "Pipfile"
                | "Pipfile.lock"
        )
    }

    async fn execute_subtask(
        &self,
        subtask: SubTask,
    ) -> Result<tokio::task::JoinHandle<Result<AgentResult, AnalysisError>>, AnalysisError> {
        let task_metadata = TaskMetadata {
            task_id: subtask.task_id.clone(),
            task_type: subtask.task_type.clone(),
            assigned_agent: subtask.agent_id.clone(),
            start_time: Instant::now(),
            deadline: None,
            status: TaskStatus::Pending,
        };

        {
            let mut tasks = self.active_tasks.write().await;
            tasks.insert(subtask.task_id.clone(), task_metadata);
        }

        let task_id = subtask.task_id.clone();

        let handle = match subtask.task_type {
            TaskType::TaintAnalysis => {
                if let Some(agent) = self.agents.get("TaintAgent") {
                    let agent = agent.clone_box();
                    let context = subtask.context.clone();
                    tokio::spawn(async move { agent.execute_task(task_id, context).await })
                } else {
                    return Err(AnalysisError::DetectionError(
                        "TaintAgent not available".to_string(),
                    ));
                }
            }
            TaskType::ConfigurationAnalysis => {
                if let Some(agent) = self.agents.get("ConfigAgent") {
                    let agent = agent.clone_box();
                    let context = subtask.context.clone();
                    tokio::spawn(async move { agent.execute_task(task_id, context).await })
                } else {
                    return Err(AnalysisError::DetectionError(
                        "ConfigAgent not available".to_string(),
                    ));
                }
            }
            _ => {
                return Err(AnalysisError::DetectionError(format!(
                    "Unsupported task type: {:?}",
                    subtask.task_type
                )));
            }
        };

        Ok(handle)
    }

    async fn synthesize_results(
        &self,
        agent_results: Vec<AgentResult>,
        _context: &SecurityContext,
    ) -> Result<SecurityAnalysisResult, AnalysisError> {
        let mut final_result = SecurityAnalysisResult::new();

        for result in agent_results {
            match result {
                AgentResult::TaintAnalysis(issues) => {
                    final_result
                        .vulnerabilities
                        .extend(issues.into_iter().map(|issue| {
                            OwaspVulnerability {
                        category:
                            crate::analysis::detectors::security::owasp::OwaspCategory::Injection,
                        issue_type: issue.issue_type,
                        title: issue.title,
                        description: issue.description,
                        severity: issue.severity,
                        confidence_score: issue.confidence_score,
                        location: crate::analysis::detectors::security::types::SecurityLocation {
                            file_path: issue.location.file_path,
                            start_line: issue.location.start_line,
                            end_line: issue.location.end_line,
                            start_column: issue.location.start_column,
                            end_column: issue.location.end_column,
                            function_name: issue.location.function_name,
                            class_name: issue.location.class_name,
                            module_name: issue.location.module_name,
                        },
                        remediation: issue.remediation,
                        metadata: issue.metadata,
                        architectural_correlation:
                            issue.correlation_id.map(|id| vec![id]).unwrap_or_default(),
                    }
                        }));
                }
                AgentResult::OwaspAnalysis(vulnerabilities) => {
                    final_result.vulnerabilities.extend(vulnerabilities);
                }
                AgentResult::ValidationResult(_validation_report) => {
                    // Apply validation results to adjust confidence scores
                }
                _ => {
                    debug!("Unhandled agent result type in synthesis");
                }
            }
        }

        Ok(final_result)
    }

    async fn deduplicate_and_filter(
        &self,
        mut result: SecurityAnalysisResult,
    ) -> Result<SecurityAnalysisResult, AnalysisError> {
        // Simple deduplication based on location and issue type
        result.vulnerabilities.sort_by(|a, b| {
            a.location
                .file_path
                .cmp(&b.location.file_path)
                .then(a.location.start_line.cmp(&b.location.start_line))
                .then(a.issue_type.to_string().cmp(&b.issue_type.to_string()))
        });

        result.vulnerabilities.dedup_by(|a, b| {
            a.location.file_path == b.location.file_path
                && a.location.start_line == b.location.start_line
                && a.issue_type == b.issue_type
        });

        // Filter by confidence threshold
        let min_confidence = 0.5;
        result
            .vulnerabilities
            .retain(|v| v.confidence_score >= min_confidence);

        info!(
            "After deduplication and filtering: {} vulnerabilities remain",
            result.vulnerabilities.len()
        );
        Ok(result)
    }
}
