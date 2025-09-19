//! Security pattern detection within knowledge graph structures
//!
//! This module identifies security-relevant patterns by analyzing the knowledge graph
//! structure, relationships, and entity characteristics to detect potential security issues.

use crate::analysis::detectors::security::knowledge_graph::types::{
    CodeEntity, CodeRelationship, SecurityQuery, AntiPatternInfo,
    StructuralSemanticGraph, GraphNode, GraphEdge,
};
use crate::analysis::detectors::security::types::{SecurityIssue, SecurityIssueType, SecuritySeverity};
use crate::analysis::AnalysisError;
use std::collections::{HashMap, HashSet};
use tracing::{debug, info};

/// Security pattern detector using graph analysis
pub struct SecurityPatternDetector {
    patterns: Vec<SecurityPattern>,
    detection_config: DetectionConfig,
}

impl SecurityPatternDetector {
    pub fn new() -> Self {
        Self {
            patterns: Self::default_security_patterns(),
            detection_config: DetectionConfig::default(),
        }
    }

    /// Detect security patterns in the knowledge graph
    pub async fn detect_patterns(
        &self,
        graph: &StructuralSemanticGraph,
        entities: &[CodeEntity],
    ) -> Result<SecurityPatternResult, AnalysisError> {
        info!("Detecting security patterns in graph with {} nodes", graph.nodes.len());

        let mut detected_issues = Vec::new();
        let mut pattern_matches = Vec::new();

        // Apply each security pattern
        for pattern in &self.patterns {
            let pattern_result = self.apply_security_pattern(pattern, graph, entities).await?;
            detected_issues.extend(pattern_result.security_issues);
            pattern_matches.extend(pattern_result.pattern_matches);
        }

        // Analyze pattern interactions
        let interaction_issues = self.analyze_pattern_interactions(&pattern_matches, graph).await?;
        detected_issues.extend(interaction_issues);

        Ok(SecurityPatternResult {
            detected_issues,
            pattern_matches,
            confidence_score: self.calculate_overall_confidence(&detected_issues),
        })
    }

    /// Apply a specific security pattern to the graph
    async fn apply_security_pattern(
        &self,
        pattern: &SecurityPattern,
        graph: &StructuralSemanticGraph,
        entities: &[CodeEntity],
    ) -> Result<PatternApplicationResult, AnalysisError> {
        match pattern {
            SecurityPattern::PrivilegeEscalation => self.detect_privilege_escalation(graph, entities).await,
            SecurityPattern::TrustBoundaryViolation => self.detect_trust_boundary_violations(graph, entities).await,
            SecurityPattern::UnsafeDataFlow => self.detect_unsafe_data_flows(graph, entities).await,
            SecurityPattern::OverPrivilegedComponents => self.detect_overprivileged_components(graph, entities).await,
            SecurityPattern::WeakAuthenticationPaths => self.detect_weak_authentication_paths(graph, entities).await,
            SecurityPattern::InsecureDefaults => self.detect_insecure_defaults(graph, entities).await,
            SecurityPattern::UnvalidatedInputPaths => self.detect_unvalidated_input_paths(graph, entities).await,
            SecurityPattern::ExposedInternalAPIs => self.detect_exposed_internal_apis(graph, entities).await,
        }
    }

    /// Detect privilege escalation patterns
    async fn detect_privilege_escalation(
        &self,
        graph: &StructuralSemanticGraph,
        entities: &[CodeEntity],
    ) -> Result<PatternApplicationResult, AnalysisError> {
        let mut issues = Vec::new();
        let mut matches = Vec::new();

        // Identify privilege levels
        let (low_privilege, high_privilege) = self.categorize_entities_by_privilege(entities);

        // Find paths from low to high privilege
        for low_priv_id in &low_privilege {
            for high_priv_id in &high_privilege {
                if let Some(path) = self.find_graph_path(low_priv_id, high_priv_id, graph) {
                    let vulnerability_score = self.assess_escalation_risk(&path, entities);

                    if vulnerability_score > self.detection_config.escalation_threshold {
                        issues.push(SecurityIssue {
                            id: Some(format!("privilege_escalation_{}_{}", low_priv_id, high_priv_id)),
                            title: "Potential Privilege Escalation Path".to_string(),
                            description: format!(
                                "Found path from low-privilege entity '{}' to high-privilege entity '{}'",
                                self.get_entity_name(low_priv_id, entities),
                                self.get_entity_name(high_priv_id, entities)
                            ),
                            issue_type: SecurityIssueType::PrivilegeEscalation,
                            severity: SecuritySeverity::High,
                            location: self.get_entity_location(low_priv_id, entities)
                                .unwrap_or_else(|| self.default_location()),
                            file_path: None,
                            line_number: None,
                            code_snippet: None,
                            recommendation: "Review access controls and implement proper privilege boundaries".to_string(),
                            cwe_id: Some("CWE-269".to_string()),
                            confidence: vulnerability_score,
                        });

                        matches.push(PatternMatch {
                            pattern_type: "Privilege Escalation".to_string(),
                            affected_entities: vec![low_priv_id.clone(), high_priv_id.clone()],
                            severity: vulnerability_score,
                            description: "Privilege escalation path detected".to_string(),
                        });
                    }
                }
            }
        }

        Ok(PatternApplicationResult {
            security_issues: issues,
            pattern_matches: matches,
        })
    }

    /// Detect trust boundary violations
    async fn detect_trust_boundary_violations(
        &self,
        graph: &StructuralSemanticGraph,
        entities: &[CodeEntity],
    ) -> Result<PatternApplicationResult, AnalysisError> {
        let mut issues = Vec::new();
        let mut matches = Vec::new();

        // Identify trust boundaries based on file paths and naming
        let (trusted_entities, untrusted_entities) = self.categorize_entities_by_trust(entities);

        // Check for direct connections from untrusted to trusted without validation
        for untrusted_id in &untrusted_entities {
            for trusted_id in &trusted_entities {
                if self.has_direct_connection(untrusted_id, trusted_id, graph) {
                    // Check if there's validation in between
                    if !self.has_validation_barrier(untrusted_id, trusted_id, graph, entities) {
                        issues.push(SecurityIssue {
                            id: Some(format!("trust_boundary_{}_{}", untrusted_id, trusted_id)),
                            title: "Trust Boundary Violation".to_string(),
                            description: format!(
                                "Direct connection from untrusted '{}' to trusted '{}' without validation",
                                self.get_entity_name(untrusted_id, entities),
                                self.get_entity_name(trusted_id, entities)
                            ),
                            issue_type: SecurityIssueType::TrustBoundaryViolation,
                            severity: SecuritySeverity::High,
                            location: self.get_entity_location(untrusted_id, entities)
                                .unwrap_or_else(|| self.default_location()),
                            file_path: None,
                            line_number: None,
                            code_snippet: None,
                            recommendation: "Add input validation and sanitization at trust boundaries".to_string(),
                            cwe_id: Some("CWE-20".to_string()),
                            confidence: 0.8,
                        });

                        matches.push(PatternMatch {
                            pattern_type: "Trust Boundary Violation".to_string(),
                            affected_entities: vec![untrusted_id.clone(), trusted_id.clone()],
                            severity: 0.8,
                            description: "Trust boundary crossed without validation".to_string(),
                        });
                    }
                }
            }
        }

        Ok(PatternApplicationResult {
            security_issues: issues,
            pattern_matches: matches,
        })
    }

    /// Detect unsafe data flows
    async fn detect_unsafe_data_flows(
        &self,
        graph: &StructuralSemanticGraph,
        entities: &[CodeEntity],
    ) -> Result<PatternApplicationResult, AnalysisError> {
        let mut issues = Vec::new();
        let mut matches = Vec::new();

        // Identify data sources and sinks
        let data_sources = self.identify_data_sources(entities);
        let dangerous_sinks = self.identify_dangerous_sinks(entities);

        // Check for unsafe flows from sources to sinks
        for source_id in &data_sources {
            for sink_id in &dangerous_sinks {
                if let Some(flow_path) = self.find_data_flow_path(source_id, sink_id, graph) {
                    let risk_score = self.assess_data_flow_risk(&flow_path, entities);

                    if risk_score > self.detection_config.data_flow_threshold {
                        issues.push(SecurityIssue {
                            id: Some(format!("unsafe_data_flow_{}_{}", source_id, sink_id)),
                            title: "Unsafe Data Flow".to_string(),
                            description: format!(
                                "Unsafe data flow from '{}' to dangerous sink '{}'",
                                self.get_entity_name(source_id, entities),
                                self.get_entity_name(sink_id, entities)
                            ),
                            issue_type: SecurityIssueType::Injection,
                            severity: SecuritySeverity::High,
                            location: self.get_entity_location(source_id, entities)
                                .unwrap_or_else(|| self.default_location()),
                            file_path: None,
                            line_number: None,
                            code_snippet: None,
                            recommendation: "Implement proper input validation and output encoding".to_string(),
                            cwe_id: Some("CWE-79".to_string()),
                            confidence: risk_score,
                        });

                        matches.push(PatternMatch {
                            pattern_type: "Unsafe Data Flow".to_string(),
                            affected_entities: vec![source_id.clone(), sink_id.clone()],
                            severity: risk_score,
                            description: "Potentially unsafe data flow detected".to_string(),
                        });
                    }
                }
            }
        }

        Ok(PatternApplicationResult {
            security_issues: issues,
            pattern_matches: matches,
        })
    }

    /// Detect overprivileged components
    async fn detect_overprivileged_components(
        &self,
        graph: &StructuralSemanticGraph,
        entities: &[CodeEntity],
    ) -> Result<PatternApplicationResult, AnalysisError> {
        let mut issues = Vec::new();
        let mut matches = Vec::new();

        // Analyze each entity's connections and privilege requirements
        for entity in entities {
            let connections = self.count_entity_connections(&entity.id, graph);
            let privilege_indicators = self.count_privilege_indicators(entity);

            // Heuristic: high connections with privilege indicators suggest overprivilege
            if connections > 15 && privilege_indicators > 0 {
                let overprivilege_score = (connections as f64 / 20.0).min(1.0);

                issues.push(SecurityIssue {
                    id: Some(format!("overprivileged_{}", entity.id)),
                    title: "Overprivileged Component".to_string(),
                    description: format!(
                        "Component '{}' has extensive connections ({}) and privilege indicators, suggesting overprivilege",
                        entity.name, connections
                    ),
                    issue_type: SecurityIssueType::PrivilegeEscalation,
                    severity: SecuritySeverity::Medium,
                    location: self.get_entity_location(&entity.id, entities)
                        .unwrap_or_else(|| self.default_location()),
                    file_path: None,
                    line_number: None,
                    code_snippet: None,
                    recommendation: "Apply principle of least privilege and reduce component responsibilities".to_string(),
                    cwe_id: Some("CWE-250".to_string()),
                    confidence: overprivilege_score,
                });

                matches.push(PatternMatch {
                    pattern_type: "Overprivileged Component".to_string(),
                    affected_entities: vec![entity.id.clone()],
                    severity: overprivilege_score,
                    description: "Component has excessive privileges".to_string(),
                });
            }
        }

        Ok(PatternApplicationResult {
            security_issues: issues,
            pattern_matches: matches,
        })
    }

    /// Detect weak authentication paths
    async fn detect_weak_authentication_paths(
        &self,
        graph: &StructuralSemanticGraph,
        entities: &[CodeEntity],
    ) -> Result<PatternApplicationResult, AnalysisError> {
        let mut issues = Vec::new();
        let mut matches = Vec::new();

        // Identify authentication-related entities
        let auth_entities = self.identify_authentication_entities(entities);
        let protected_resources = self.identify_protected_resources(entities);

        // Check for paths to protected resources that bypass authentication
        for resource_id in &protected_resources {
            let auth_required = auth_entities.iter()
                .any(|auth_id| self.is_on_path_to_resource(auth_id, resource_id, graph));

            if !auth_required {
                issues.push(SecurityIssue {
                    id: Some(format!("weak_auth_path_{}", resource_id)),
                    title: "Weak Authentication Path".to_string(),
                    description: format!(
                        "Protected resource '{}' may be accessible without proper authentication",
                        self.get_entity_name(resource_id, entities)
                    ),
                    issue_type: SecurityIssueType::WeakAuthentication,
                    severity: SecuritySeverity::High,
                    location: self.get_entity_location(resource_id, entities)
                        .unwrap_or_else(|| self.default_location()),
                    file_path: None,
                    line_number: None,
                    code_snippet: None,
                    recommendation: "Ensure all paths to protected resources require authentication".to_string(),
                    cwe_id: Some("CWE-287".to_string()),
                    confidence: 0.7,
                });

                matches.push(PatternMatch {
                    pattern_type: "Weak Authentication Path".to_string(),
                    affected_entities: vec![resource_id.clone()],
                    severity: 0.7,
                    description: "Resource accessible without authentication".to_string(),
                });
            }
        }

        Ok(PatternApplicationResult {
            security_issues: issues,
            pattern_matches: matches,
        })
    }

    /// Detect insecure defaults
    async fn detect_insecure_defaults(
        &self,
        _graph: &StructuralSemanticGraph,
        entities: &[CodeEntity],
    ) -> Result<PatternApplicationResult, AnalysisError> {
        let mut issues = Vec::new();
        let mut matches = Vec::new();

        // Look for configuration-related entities with insecure patterns
        for entity in entities {
            if self.is_configuration_entity(entity) {
                let insecurity_indicators = self.check_insecure_defaults(entity);

                if !insecurity_indicators.is_empty() {
                    issues.push(SecurityIssue {
                        id: Some(format!("insecure_defaults_{}", entity.id)),
                        title: "Insecure Default Configuration".to_string(),
                        description: format!(
                            "Configuration entity '{}' may have insecure defaults: {}",
                            entity.name,
                            insecurity_indicators.join(", ")
                        ),
                        issue_type: SecurityIssueType::InsecureDefaults,
                        severity: SecuritySeverity::Medium,
                        location: self.get_entity_location(&entity.id, entities)
                            .unwrap_or_else(|| self.default_location()),
                        file_path: None,
                        line_number: None,
                        code_snippet: None,
                        recommendation: "Review and secure default configurations".to_string(),
                        cwe_id: Some("CWE-1188".to_string()),
                        confidence: 0.6,
                    });

                    matches.push(PatternMatch {
                        pattern_type: "Insecure Defaults".to_string(),
                        affected_entities: vec![entity.id.clone()],
                        severity: 0.6,
                        description: "Configuration has insecure defaults".to_string(),
                    });
                }
            }
        }

        Ok(PatternApplicationResult {
            security_issues: issues,
            pattern_matches: matches,
        })
    }

    /// Detect unvalidated input paths
    async fn detect_unvalidated_input_paths(
        &self,
        graph: &StructuralSemanticGraph,
        entities: &[CodeEntity],
    ) -> Result<PatternApplicationResult, AnalysisError> {
        let mut issues = Vec::new();
        let mut matches = Vec::new();

        // Find input entry points
        let input_sources = self.identify_input_sources(entities);
        let processing_entities = self.identify_processing_entities(entities);

        // Check for direct paths from input to processing without validation
        for input_id in &input_sources {
            for processor_id in &processing_entities {
                if self.has_direct_connection(input_id, processor_id, graph) {
                    if !self.has_validation_on_path(input_id, processor_id, graph, entities) {
                        issues.push(SecurityIssue {
                            id: Some(format!("unvalidated_input_{}_{}", input_id, processor_id)),
                            title: "Unvalidated Input Path".to_string(),
                            description: format!(
                                "Input from '{}' reaches processor '{}' without validation",
                                self.get_entity_name(input_id, entities),
                                self.get_entity_name(processor_id, entities)
                            ),
                            issue_type: SecurityIssueType::Injection,
                            severity: SecuritySeverity::High,
                            location: self.get_entity_location(input_id, entities)
                                .unwrap_or_else(|| self.default_location()),
                            file_path: None,
                            line_number: None,
                            code_snippet: None,
                            recommendation: "Add input validation before processing user data".to_string(),
                            cwe_id: Some("CWE-20".to_string()),
                            confidence: 0.8,
                        });

                        matches.push(PatternMatch {
                            pattern_type: "Unvalidated Input".to_string(),
                            affected_entities: vec![input_id.clone(), processor_id.clone()],
                            severity: 0.8,
                            description: "Input reaches processor without validation".to_string(),
                        });
                    }
                }
            }
        }

        Ok(PatternApplicationResult {
            security_issues: issues,
            pattern_matches: matches,
        })
    }

    /// Detect exposed internal APIs
    async fn detect_exposed_internal_apis(
        &self,
        _graph: &StructuralSemanticGraph,
        entities: &[CodeEntity],
    ) -> Result<PatternApplicationResult, AnalysisError> {
        let mut issues = Vec::new();
        let mut matches = Vec::new();

        // Look for internal APIs that might be exposed
        for entity in entities {
            if self.is_internal_api(entity) && self.appears_exposed(entity) {
                issues.push(SecurityIssue {
                    id: Some(format!("exposed_internal_api_{}", entity.id)),
                    title: "Exposed Internal API".to_string(),
                    description: format!(
                        "Internal API '{}' appears to be externally accessible",
                        entity.name
                    ),
                    issue_type: SecurityIssueType::ExposureOfSensitiveInformation,
                    severity: SecuritySeverity::Medium,
                    location: self.get_entity_location(&entity.id, entities)
                        .unwrap_or_else(|| self.default_location()),
                    file_path: None,
                    line_number: None,
                    code_snippet: None,
                    recommendation: "Restrict access to internal APIs and add proper authentication".to_string(),
                    cwe_id: Some("CWE-200".to_string()),
                    confidence: 0.6,
                });

                matches.push(PatternMatch {
                    pattern_type: "Exposed Internal API".to_string(),
                    affected_entities: vec![entity.id.clone()],
                    severity: 0.6,
                    description: "Internal API is externally accessible".to_string(),
                });
            }
        }

        Ok(PatternApplicationResult {
            security_issues: issues,
            pattern_matches: matches,
        })
    }

    /// Analyze interactions between detected patterns
    async fn analyze_pattern_interactions(
        &self,
        pattern_matches: &[PatternMatch],
        _graph: &StructuralSemanticGraph,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut interaction_issues = Vec::new();

        // Group patterns by affected entities
        let mut entity_patterns: HashMap<String, Vec<&PatternMatch>> = HashMap::new();
        for pattern_match in pattern_matches {
            for entity_id in &pattern_match.affected_entities {
                entity_patterns.entry(entity_id.clone())
                    .or_default()
                    .push(pattern_match);
            }
        }

        // Look for entities with multiple security patterns (amplification)
        for (entity_id, patterns) in entity_patterns {
            if patterns.len() > 1 {
                let combined_severity = patterns.iter()
                    .map(|p| p.severity)
                    .fold(0.0, |acc, s| acc + s * 0.7); // Amplification factor

                if combined_severity > 1.0 {
                    interaction_issues.push(SecurityIssue {
                        id: Some(format!("pattern_amplification_{}", entity_id)),
                        title: "Security Pattern Amplification".to_string(),
                        description: format!(
                            "Entity '{}' affected by multiple security patterns, amplifying risk",
                            entity_id
                        ),
                        issue_type: SecurityIssueType::MultipleVulnerabilities,
                        severity: SecuritySeverity::Critical,
                        location: self.default_location(),
                        file_path: None,
                        line_number: None,
                        code_snippet: None,
                        recommendation: "Address all security patterns affecting this entity as a high priority".to_string(),
                        cwe_id: None,
                        confidence: combined_severity.min(1.0),
                    });
                }
            }
        }

        Ok(interaction_issues)
    }

    // Helper methods for pattern detection

    fn categorize_entities_by_privilege(&self, entities: &[CodeEntity]) -> (Vec<String>, Vec<String>) {
        let mut low_privilege = Vec::new();
        let mut high_privilege = Vec::new();

        for entity in entities {
            let name_lower = entity.name.to_lowercase();
            if name_lower.contains("admin") || name_lower.contains("root") || name_lower.contains("system") {
                high_privilege.push(entity.id.clone());
            } else if name_lower.contains("user") || name_lower.contains("guest") || name_lower.contains("public") {
                low_privilege.push(entity.id.clone());
            }
        }

        (low_privilege, high_privilege)
    }

    fn categorize_entities_by_trust(&self, entities: &[CodeEntity]) -> (Vec<String>, Vec<String>) {
        let mut trusted = Vec::new();
        let mut untrusted = Vec::new();

        for entity in entities {
            let path_str = entity.location.file_path.display().to_string().to_lowercase();
            if path_str.contains("internal") || path_str.contains("core") || path_str.contains("private") {
                trusted.push(entity.id.clone());
            } else if path_str.contains("api") || path_str.contains("public") || path_str.contains("external") {
                untrusted.push(entity.id.clone());
            }
        }

        (trusted, untrusted)
    }

    fn identify_data_sources(&self, entities: &[CodeEntity]) -> Vec<String> {
        entities.iter()
            .filter(|e| {
                let name_lower = e.name.to_lowercase();
                name_lower.contains("input") || name_lower.contains("request") ||
                name_lower.contains("form") || name_lower.contains("param")
            })
            .map(|e| e.id.clone())
            .collect()
    }

    fn identify_dangerous_sinks(&self, entities: &[CodeEntity]) -> Vec<String> {
        entities.iter()
            .filter(|e| {
                let name_lower = e.name.to_lowercase();
                name_lower.contains("exec") || name_lower.contains("eval") ||
                name_lower.contains("sql") || name_lower.contains("command")
            })
            .map(|e| e.id.clone())
            .collect()
    }

    fn find_graph_path(&self, from: &str, to: &str, graph: &StructuralSemanticGraph) -> Option<Vec<String>> {
        // Simple BFS pathfinding
        use std::collections::VecDeque;

        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent = HashMap::new();

        queue.push_back(from.to_string());
        visited.insert(from.to_string());

        while let Some(current) = queue.pop_front() {
            if current == to {
                // Reconstruct path
                let mut path = Vec::new();
                let mut node = to.to_string();
                while node != from {
                    path.push(node.clone());
                    if let Some(p) = parent.get(&node) {
                        node = p.clone();
                    } else {
                        break;
                    }
                }
                path.push(from.to_string());
                path.reverse();
                return Some(path);
            }

            // Find neighbors
            for edge in &graph.edges {
                if edge.from == current && !visited.contains(&edge.to) {
                    visited.insert(edge.to.clone());
                    parent.insert(edge.to.clone(), current.clone());
                    queue.push_back(edge.to.clone());
                }
            }
        }

        None
    }

    fn has_direct_connection(&self, from: &str, to: &str, graph: &StructuralSemanticGraph) -> bool {
        graph.edges.iter().any(|edge| edge.from == from && edge.to == to)
    }

    fn has_validation_barrier(&self, _from: &str, _to: &str, _graph: &StructuralSemanticGraph, _entities: &[CodeEntity]) -> bool {
        // Placeholder: would check for validation entities on the path
        false
    }

    fn assess_escalation_risk(&self, _path: &[String], _entities: &[CodeEntity]) -> f64 {
        // Placeholder: would analyze path for escalation risk factors
        0.7
    }

    fn get_entity_name(&self, entity_id: &str, entities: &[CodeEntity]) -> String {
        entities.iter()
            .find(|e| e.id == entity_id)
            .map(|e| e.name.clone())
            .unwrap_or_else(|| entity_id.to_string())
    }

    fn get_entity_location(&self, entity_id: &str, entities: &[CodeEntity]) -> Option<crate::analysis::detectors::security::types::SecurityLocation> {
        entities.iter()
            .find(|e| e.id == entity_id)
            .map(|e| crate::analysis::detectors::security::types::SecurityLocation::new(
                e.location.file_path.clone(),
                e.location.start_line,
                e.location.end_line,
            ))
    }

    fn default_location(&self) -> crate::analysis::detectors::security::types::SecurityLocation {
        crate::analysis::detectors::security::types::SecurityLocation::new(
            std::path::PathBuf::from("unknown"),
            0,
            0,
        )
    }

    fn find_data_flow_path(&self, from: &str, to: &str, graph: &StructuralSemanticGraph) -> Option<Vec<String>> {
        // Similar to find_graph_path but filtered for data flow edges
        self.find_graph_path(from, to, graph)
    }

    fn assess_data_flow_risk(&self, _path: &[String], _entities: &[CodeEntity]) -> f64 {
        // Placeholder: would analyze path for data flow risk factors
        0.8
    }

    fn count_entity_connections(&self, entity_id: &str, graph: &StructuralSemanticGraph) -> usize {
        graph.edges.iter()
            .filter(|edge| edge.from == entity_id || edge.to == entity_id)
            .count()
    }

    fn count_privilege_indicators(&self, entity: &CodeEntity) -> usize {
        let name_lower = entity.name.to_lowercase();
        let mut count = 0;

        if name_lower.contains("admin") { count += 1; }
        if name_lower.contains("priv") { count += 1; }
        if name_lower.contains("auth") { count += 1; }
        if name_lower.contains("secure") { count += 1; }

        count
    }

    fn identify_authentication_entities(&self, entities: &[CodeEntity]) -> Vec<String> {
        entities.iter()
            .filter(|e| {
                let name_lower = e.name.to_lowercase();
                name_lower.contains("auth") || name_lower.contains("login") ||
                name_lower.contains("verify") || name_lower.contains("token")
            })
            .map(|e| e.id.clone())
            .collect()
    }

    fn identify_protected_resources(&self, entities: &[CodeEntity]) -> Vec<String> {
        entities.iter()
            .filter(|e| {
                let name_lower = e.name.to_lowercase();
                name_lower.contains("admin") || name_lower.contains("private") ||
                name_lower.contains("secure") || name_lower.contains("protected")
            })
            .map(|e| e.id.clone())
            .collect()
    }

    fn is_on_path_to_resource(&self, _auth_id: &str, _resource_id: &str, _graph: &StructuralSemanticGraph) -> bool {
        // Placeholder: would check if auth entity is on path to resource
        false
    }

    fn is_configuration_entity(&self, entity: &CodeEntity) -> bool {
        let name_lower = entity.name.to_lowercase();
        name_lower.contains("config") || name_lower.contains("setting") || name_lower.contains("default")
    }

    fn check_insecure_defaults(&self, entity: &CodeEntity) -> Vec<String> {
        let mut indicators = Vec::new();
        let name_lower = entity.name.to_lowercase();

        if name_lower.contains("password") && name_lower.contains("default") {
            indicators.push("Default password".to_string());
        }
        if name_lower.contains("debug") && name_lower.contains("true") {
            indicators.push("Debug mode enabled".to_string());
        }
        if name_lower.contains("ssl") && name_lower.contains("false") {
            indicators.push("SSL disabled".to_string());
        }

        indicators
    }

    fn identify_input_sources(&self, entities: &[CodeEntity]) -> Vec<String> {
        entities.iter()
            .filter(|e| {
                let name_lower = e.name.to_lowercase();
                name_lower.contains("input") || name_lower.contains("request") ||
                name_lower.contains("param") || name_lower.contains("form")
            })
            .map(|e| e.id.clone())
            .collect()
    }

    fn identify_processing_entities(&self, entities: &[CodeEntity]) -> Vec<String> {
        entities.iter()
            .filter(|e| {
                let name_lower = e.name.to_lowercase();
                name_lower.contains("process") || name_lower.contains("handle") ||
                name_lower.contains("parse") || name_lower.contains("execute")
            })
            .map(|e| e.id.clone())
            .collect()
    }

    fn has_validation_on_path(&self, _from: &str, _to: &str, _graph: &StructuralSemanticGraph, _entities: &[CodeEntity]) -> bool {
        // Placeholder: would check for validation entities on the path
        false
    }

    fn is_internal_api(&self, entity: &CodeEntity) -> bool {
        let name_lower = entity.name.to_lowercase();
        let path_str = entity.location.file_path.display().to_string().to_lowercase();

        (name_lower.contains("api") || name_lower.contains("endpoint")) &&
        (path_str.contains("internal") || path_str.contains("private"))
    }

    fn appears_exposed(&self, entity: &CodeEntity) -> bool {
        // Heuristic: check for exposure indicators
        let name_lower = entity.name.to_lowercase();
        name_lower.contains("public") || name_lower.contains("expose") ||
        entity.metadata.get("visibility").map(|v| v.as_str()) == Some(Some("public"))
    }

    fn calculate_overall_confidence(&self, issues: &[SecurityIssue]) -> f64 {
        if issues.is_empty() {
            1.0
        } else {
            issues.iter().map(|i| i.confidence).sum::<f64>() / issues.len() as f64
        }
    }

    fn default_security_patterns() -> Vec<SecurityPattern> {
        vec![
            SecurityPattern::PrivilegeEscalation,
            SecurityPattern::TrustBoundaryViolation,
            SecurityPattern::UnsafeDataFlow,
            SecurityPattern::OverPrivilegedComponents,
            SecurityPattern::WeakAuthenticationPaths,
            SecurityPattern::InsecureDefaults,
            SecurityPattern::UnvalidatedInputPaths,
            SecurityPattern::ExposedInternalAPIs,
        ]
    }
}

/// Security patterns to detect in the knowledge graph
#[derive(Debug, Clone)]
pub enum SecurityPattern {
    PrivilegeEscalation,
    TrustBoundaryViolation,
    UnsafeDataFlow,
    OverPrivilegedComponents,
    WeakAuthenticationPaths,
    InsecureDefaults,
    UnvalidatedInputPaths,
    ExposedInternalAPIs,
}

/// Configuration for pattern detection
#[derive(Debug, Clone)]
pub struct DetectionConfig {
    pub escalation_threshold: f64,
    pub data_flow_threshold: f64,
    pub trust_boundary_threshold: f64,
}

impl Default for DetectionConfig {
    fn default() -> Self {
        Self {
            escalation_threshold: 0.6,
            data_flow_threshold: 0.7,
            trust_boundary_threshold: 0.8,
        }
    }
}

/// Result of security pattern detection
#[derive(Debug, Clone)]
pub struct SecurityPatternResult {
    pub detected_issues: Vec<SecurityIssue>,
    pub pattern_matches: Vec<PatternMatch>,
    pub confidence_score: f64,
}

/// Result of applying a specific pattern
#[derive(Debug, Clone)]
pub struct PatternApplicationResult {
    pub security_issues: Vec<SecurityIssue>,
    pub pattern_matches: Vec<PatternMatch>,
}

/// A detected pattern match
#[derive(Debug, Clone)]
pub struct PatternMatch {
    pub pattern_type: String,
    pub affected_entities: Vec<String>,
    pub severity: f64,
    pub description: String,
}