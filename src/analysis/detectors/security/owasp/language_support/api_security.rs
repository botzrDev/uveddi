//! API security pattern analysis
//!
//! This module provides security analysis for REST APIs, GraphQL APIs,
//! and other API architectures.

use std::collections::HashMap;

/// Analyzer for API security patterns and vulnerabilities
pub struct ApiSecurityAnalyzer {
    api_patterns: HashMap<String, ApiSecurityPattern>,
}

#[derive(Debug, Clone)]
pub struct ApiSecurityPattern {
    pub pattern_type: ApiType,
    pub security_concerns: Vec<String>,
    pub detection_patterns: Vec<String>,
    pub remediation_advice: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ApiType {
    RestApi,
    GraphQLApi,
    GrpcApi,
    WebSocket,
}

impl ApiSecurityAnalyzer {
    pub fn new() -> Self {
        let mut api_patterns = HashMap::new();

        // REST API security patterns
        api_patterns.insert(
            "rest_api".to_string(),
            ApiSecurityPattern {
                pattern_type: ApiType::RestApi,
                security_concerns: vec![
                    "Missing authentication on endpoints".to_string(),
                    "Improper HTTP method usage".to_string(),
                    "Missing rate limiting".to_string(),
                    "Insufficient input validation".to_string(),
                    "Information disclosure in responses".to_string(),
                ],
                detection_patterns: vec![
                    "app.get(".to_string(),
                    "app.post(".to_string(),
                    "@app.route".to_string(),
                    "actix_web::web::".to_string(),
                ],
                remediation_advice: vec![
                    "Implement OAuth 2.0 or JWT authentication".to_string(),
                    "Use appropriate HTTP methods (GET for read, POST for create)".to_string(),
                    "Implement rate limiting per endpoint".to_string(),
                    "Validate all input parameters and request bodies".to_string(),
                    "Minimize data exposure in API responses".to_string(),
                ],
            },
        );

        // GraphQL API security patterns
        api_patterns.insert(
            "graphql_api".to_string(),
            ApiSecurityPattern {
                pattern_type: ApiType::GraphQLApi,
                security_concerns: vec![
                    "Query depth attacks".to_string(),
                    "Query complexity attacks".to_string(),
                    "Missing field-level authorization".to_string(),
                    "Information disclosure through introspection".to_string(),
                ],
                detection_patterns: vec![
                    "graphql".to_string(),
                    "GraphQLSchema".to_string(),
                    "resolver".to_string(),
                ],
                remediation_advice: vec![
                    "Implement query depth limiting".to_string(),
                    "Set query complexity analysis".to_string(),
                    "Add field-level authorization".to_string(),
                    "Disable introspection in production".to_string(),
                ],
            },
        );

        // gRPC API security patterns
        api_patterns.insert(
            "grpc_api".to_string(),
            ApiSecurityPattern {
                pattern_type: ApiType::GrpcApi,
                security_concerns: vec![
                    "Missing TLS encryption".to_string(),
                    "Insufficient authentication".to_string(),
                    "Unvalidated protobuf messages".to_string(),
                ],
                detection_patterns: vec![
                    "grpc".to_string(),
                    "tonic::".to_string(),
                    ".proto".to_string(),
                ],
                remediation_advice: vec![
                    "Enable TLS encryption for all gRPC communication".to_string(),
                    "Implement proper authentication mechanisms".to_string(),
                    "Validate all protobuf message fields".to_string(),
                ],
            },
        );

        // WebSocket security patterns
        api_patterns.insert(
            "websocket".to_string(),
            ApiSecurityPattern {
                pattern_type: ApiType::WebSocket,
                security_concerns: vec![
                    "Missing origin validation".to_string(),
                    "Insufficient authentication".to_string(),
                    "Message injection vulnerabilities".to_string(),
                ],
                detection_patterns: vec![
                    "WebSocket".to_string(),
                    "ws://".to_string(),
                    "wss://".to_string(),
                    "socket.io".to_string(),
                ],
                remediation_advice: vec![
                    "Validate WebSocket origin headers".to_string(),
                    "Implement authentication for WebSocket connections".to_string(),
                    "Validate and sanitize all WebSocket messages".to_string(),
                ],
            },
        );

        Self { api_patterns }
    }

    /// Detect API patterns in code content
    pub fn detect_api_patterns(&self, content: &str) -> Vec<String> {
        let mut detected_patterns = Vec::new();

        for (pattern_name, pattern) in &self.api_patterns {
            for detection_pattern in &pattern.detection_patterns {
                if content.contains(detection_pattern) {
                    detected_patterns.push(pattern_name.clone());
                    break;
                }
            }
        }

        detected_patterns
    }

    /// Get security concerns for a specific API pattern
    pub fn get_security_concerns(&self, pattern_name: &str) -> Vec<String> {
        self.api_patterns
            .get(pattern_name)
            .map(|pattern| pattern.security_concerns.clone())
            .unwrap_or_default()
    }

    /// Get remediation advice for a specific API pattern
    pub fn get_remediation_advice(&self, pattern_name: &str) -> Vec<String> {
        self.api_patterns
            .get(pattern_name)
            .map(|pattern| pattern.remediation_advice.clone())
            .unwrap_or_default()
    }

    /// Check if content contains potentially insecure API patterns
    pub fn analyze_api_security(&self, content: &str) -> Vec<ApiSecurityIssue> {
        let mut issues = Vec::new();

        // Check for common API security issues
        if content.contains("app.get") && !content.contains("auth") {
            issues.push(ApiSecurityIssue {
                issue_type: "Missing Authentication".to_string(),
                description: "API endpoint without authentication middleware".to_string(),
                remediation: "Add authentication middleware to protect API endpoints".to_string(),
            });
        }

        if content.contains("*") && content.contains("cors") {
            issues.push(ApiSecurityIssue {
                issue_type: "Permissive CORS".to_string(),
                description: "CORS policy allows all origins".to_string(),
                remediation: "Restrict CORS to specific allowed origins".to_string(),
            });
        }

        if content.contains("eval(") || content.contains("exec(") {
            issues.push(ApiSecurityIssue {
                issue_type: "Code Injection".to_string(),
                description: "Dangerous eval/exec usage in API handler".to_string(),
                remediation: "Remove eval/exec calls and use safe alternatives".to_string(),
            });
        }

        issues
    }
}

#[derive(Debug, Clone)]
pub struct ApiSecurityIssue {
    pub issue_type: String,
    pub description: String,
    pub remediation: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_pattern_detection() {
        let analyzer = ApiSecurityAnalyzer::new();

        let rest_content = "app.get('/api/users', handler)";
        let patterns = analyzer.detect_api_patterns(rest_content);
        assert!(patterns.contains(&"rest_api".to_string()));

        let graphql_content = "const schema = new GraphQLSchema({";
        let patterns = analyzer.detect_api_patterns(graphql_content);
        assert!(patterns.contains(&"graphql_api".to_string()));
    }

    #[test]
    fn test_security_analysis() {
        let analyzer = ApiSecurityAnalyzer::new();

        let insecure_content = "app.get('/api/data', (req, res) => { eval(req.body.code); });";
        let issues = analyzer.analyze_api_security(insecure_content);
        assert!(!issues.is_empty());
        assert!(issues
            .iter()
            .any(|i| i.issue_type.contains("Code Injection")));
    }
}
