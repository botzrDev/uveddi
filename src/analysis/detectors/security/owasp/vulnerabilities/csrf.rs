//! Cross-Site Request Forgery (CSRF) Vulnerability Detector
//!
//! Specialized detector for CSRF vulnerabilities that allow attackers to perform
//! unauthorized actions on behalf of authenticated users.

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

/// CSRF pattern definition
#[derive(Debug, Clone)]
pub struct CsrfPattern {
    pub pattern: String,
    pub description: String,
    pub confidence: f64,
    pub severity: SecuritySeverity,
    pub csrf_type: CsrfType,
    pub framework_context: String,
}

#[derive(Debug, Clone)]
pub enum CsrfType {
    MissingTokenValidation,
    WeakTokenGeneration,
    TokenExposure,
    SameSiteConfiguration,
    RefererValidation,
    CorsConfiguration,
    StateChangingGetRequest,
}

impl CsrfType {
    fn description(&self) -> &'static str {
        match self {
            CsrfType::MissingTokenValidation => "Missing CSRF token validation",
            CsrfType::WeakTokenGeneration => "Weak CSRF token generation",
            CsrfType::TokenExposure => "CSRF token exposure vulnerability",
            CsrfType::SameSiteConfiguration => "Insecure SameSite cookie configuration",
            CsrfType::RefererValidation => "Missing or weak Referer validation",
            CsrfType::CorsConfiguration => "Insecure CORS configuration",
            CsrfType::StateChangingGetRequest => "State-changing operation via GET request",
        }
    }
}

/// Specialized CSRF detector
pub struct CsrfDetector {
    patterns: HashMap<SourceLanguage, Vec<CsrfPattern>>,
}

impl CsrfDetector {
    pub fn new() -> Self {
        Self {
            patterns: Self::init_patterns(),
        }
    }

    fn init_patterns() -> HashMap<SourceLanguage, Vec<CsrfPattern>> {
        let mut patterns = HashMap::new();

        patterns.insert(SourceLanguage::Rust, Self::rust_patterns());
        patterns.insert(SourceLanguage::Python, Self::python_patterns());
        patterns.insert(SourceLanguage::JavaScript, Self::javascript_patterns());
        patterns.insert(SourceLanguage::TypeScript, Self::javascript_patterns());

        patterns
    }

    fn rust_patterns() -> Vec<CsrfPattern> {
        vec![
            CsrfPattern {
                pattern: r#"\.route\(.*post.*\).*without.*csrf"#.to_string(),
                description: "POST route without CSRF protection".to_string(),
                confidence: 0.8,
                severity: SecuritySeverity::High,
                csrf_type: CsrfType::MissingTokenValidation,
                framework_context: "Web framework routing".to_string(),
            },
            CsrfPattern {
                pattern: r#"Cookie::build.*same_site\(SameSite::None\)"#.to_string(),
                description: "Cookie with SameSite=None without Secure flag".to_string(),
                confidence: 0.7,
                severity: SecuritySeverity::Medium,
                csrf_type: CsrfType::SameSiteConfiguration,
                framework_context: "Cookie configuration".to_string(),
            },
            CsrfPattern {
                pattern: r#"cors\(\)\.allow_any_origin\(\)"#.to_string(),
                description: "CORS configured to allow any origin".to_string(),
                confidence: 0.85,
                severity: SecuritySeverity::High,
                csrf_type: CsrfType::CorsConfiguration,
                framework_context: "CORS middleware".to_string(),
            },
            CsrfPattern {
                pattern: r#"#\[get.*\].*fn.*delete\|update\|create"#.to_string(),
                description: "State-changing operation using GET method".to_string(),
                confidence: 0.8,
                severity: SecuritySeverity::Medium,
                csrf_type: CsrfType::StateChangingGetRequest,
                framework_context: "HTTP method annotation".to_string(),
            },
        ]
    }

    fn python_patterns() -> Vec<CsrfPattern> {
        vec![
            CsrfPattern {
                // Simplified pattern - matches POST routes (cannot verify CSRF absence without lookahead)
                pattern: r#"@app\.route.*methods=\[.*POST.*\]"#
                    .to_string(),
                description: "POST route without CSRF token validation".to_string(),
                confidence: 0.5, // Lowered - may have false positives if CSRF is present
                severity: SecuritySeverity::High,
                csrf_type: CsrfType::MissingTokenValidation,
                framework_context: "Flask route".to_string(),
            },
            CsrfPattern {
                pattern: r#"WTF_CSRF_ENABLED\s*=\s*False"#.to_string(),
                description: "CSRF protection explicitly disabled".to_string(),
                confidence: 0.95,
                severity: SecuritySeverity::Critical,
                csrf_type: CsrfType::MissingTokenValidation,
                framework_context: "Flask-WTF configuration".to_string(),
            },
            CsrfPattern {
                pattern: r#"SECRET_KEY\s*=\s*['""].*['""].*\n.*CSRF_COOKIE_SECURE\s*=\s*False"#
                    .to_string(),
                description: "CSRF cookie not marked as secure".to_string(),
                confidence: 0.8,
                severity: SecuritySeverity::Medium,
                csrf_type: CsrfType::TokenExposure,
                framework_context: "Django CSRF configuration".to_string(),
            },
            CsrfPattern {
                pattern: r#"@csrf_exempt"#.to_string(),
                description: "View explicitly exempted from CSRF protection".to_string(),
                confidence: 0.9,
                severity: SecuritySeverity::High,
                csrf_type: CsrfType::MissingTokenValidation,
                framework_context: "Django CSRF exemption".to_string(),
            },
            CsrfPattern {
                pattern: r#"cors_allow_origins\s*=\s*\[\s*['"]\*['"]\s*\]"#.to_string(),
                description: "CORS configured to allow all origins".to_string(),
                confidence: 0.9,
                severity: SecuritySeverity::High,
                csrf_type: CsrfType::CorsConfiguration,
                framework_context: "CORS configuration".to_string(),
            },
        ]
    }

    fn javascript_patterns() -> Vec<CsrfPattern> {
        vec![
            CsrfPattern {
                // Simplified pattern - matches POST endpoints (cannot verify CSRF absence without lookahead)
                pattern: r#"app\.post\("#.to_string(),
                description: "POST endpoint without CSRF protection".to_string(),
                confidence: 0.45, // Lowered - may have false positives if CSRF is present
                severity: SecuritySeverity::High,
                csrf_type: CsrfType::MissingTokenValidation,
                framework_context: "Express.js route".to_string(),
            },
            CsrfPattern {
                pattern: r#"app\.use\(cors\(\{.*origin:\s*true.*\}\)\)"#.to_string(),
                description: "CORS configured to reflect request origin".to_string(),
                confidence: 0.8,
                severity: SecuritySeverity::High,
                csrf_type: CsrfType::CorsConfiguration,
                framework_context: "Express CORS middleware".to_string(),
            },
            CsrfPattern {
                pattern: r#"sameSite:\s*['"]none['"].*secure:\s*false"#.to_string(),
                description: "Cookie with SameSite=None but not secure".to_string(),
                confidence: 0.9,
                severity: SecuritySeverity::Medium,
                csrf_type: CsrfType::SameSiteConfiguration,
                framework_context: "Cookie options".to_string(),
            },
            CsrfPattern {
                pattern: r#"app\.get\(.*\)\s*{[\s\S]*(?:delete|update|create|modify)[\s\S]*}"#
                    .to_string(),
                description: "State-changing operation via GET request".to_string(),
                confidence: 0.75,
                severity: SecuritySeverity::Medium,
                csrf_type: CsrfType::StateChangingGetRequest,
                framework_context: "Express GET route".to_string(),
            },
            CsrfPattern {
                pattern: r#"Math\.random\(\)\.toString\(\).*token"#.to_string(),
                description: "Weak CSRF token generation using Math.random()".to_string(),
                confidence: 0.85,
                severity: SecuritySeverity::Medium,
                csrf_type: CsrfType::WeakTokenGeneration,
                framework_context: "Token generation".to_string(),
            },
            CsrfPattern {
                pattern: r#"credentials:\s*['"]include['"].*mode:\s*['"]cors['"]"#.to_string(),
                description: "Cross-origin requests with credentials enabled".to_string(),
                confidence: 0.7,
                severity: SecuritySeverity::Medium,
                csrf_type: CsrfType::CorsConfiguration,
                framework_context: "Fetch API options".to_string(),
            },
        ]
    }

    fn analyze_line(
        &self,
        line: &str,
        line_number: usize,
        patterns: &[CsrfPattern],
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
                        "csrf_type".to_string(),
                        pattern.csrf_type.description().to_string(),
                    );
                    metadata.add_metadata(
                        "framework_context".to_string(),
                        pattern.framework_context.clone(),
                    );
                    metadata.add_metadata("pattern_matched".to_string(), pattern.pattern.clone());
                    metadata.add_metadata("line_content".to_string(), line.trim().to_string());

                    let remediation =
                        Self::generate_remediation(&pattern.csrf_type, &pattern.framework_context);

                    let vulnerability = OwaspVulnerability::new(
                        OwaspCategory::InsecureDesign, // CSRF often falls under insecure design
                        SecurityIssueType::CrossSiteRequestForgery,
                        format!("CSRF: {}", pattern.csrf_type.description()),
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

    fn generate_remediation(csrf_type: &CsrfType, framework_context: &str) -> String {
        let base_advice = match csrf_type {
            CsrfType::MissingTokenValidation => {
                "Implement CSRF token validation for all state-changing operations."
            }
            CsrfType::WeakTokenGeneration => {
                "Use cryptographically secure random number generation for CSRF tokens."
            }
            CsrfType::TokenExposure => {
                "Ensure CSRF tokens are transmitted securely and not exposed in logs or URLs."
            }
            CsrfType::SameSiteConfiguration => {
                "Configure SameSite cookie attribute properly: use 'Strict' or 'Lax' instead of 'None'."
            }
            CsrfType::RefererValidation => {
                "Implement proper Referer header validation as an additional CSRF protection layer."
            }
            CsrfType::CorsConfiguration => {
                "Configure CORS restrictively: specify allowed origins instead of using wildcards."
            }
            CsrfType::StateChangingGetRequest => {
                "Use POST, PUT, or DELETE methods for state-changing operations, never GET."
            }
        };

        let framework_advice = match framework_context {
            c if c.contains("Express") => {
                " Consider using csurf middleware for Express.js applications."
            }
            c if c.contains("Flask") => " Use Flask-WTF for CSRF protection in Flask applications.",
            c if c.contains("Django") => {
                " Ensure Django's built-in CSRF middleware is enabled and properly configured."
            }
            c if c.contains("CORS") => {
                " Review and restrict CORS policies to trusted domains only."
            }
            _ => "",
        };

        format!(
            "{}{} Also consider implementing double-submit cookie pattern for additional security.",
            base_advice, framework_advice
        )
    }
}

#[async_trait]
impl OwaspCategoryDetector for CsrfDetector {
    async fn detect(&self, file: &ParsedFile) -> Result<Vec<OwaspVulnerability>, AnalysisError> {
        let patterns = match self.patterns.get(&file.language) {
            Some(patterns) => patterns,
            None => return Ok(Vec::new()),
        };

        let mut vulnerabilities = Vec::new();

        for (line_number, line) in file.source.lines().enumerate() {
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

impl Default for CsrfDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_csrf_detection_javascript() {
        let detector = CsrfDetector::new();
        let content = r#"
            app.post('/transfer', (req, res) => {
                // No CSRF protection
                transferMoney(req.body.amount);
            });

            app.use(cors({ origin: true, credentials: true }));

            const token = Math.random().toString(36);
        "#;

        let file = ParsedFile {
            file_path: Arc::new(PathBuf::from("test.js")),
            language: SourceLanguage::JavaScript,
            source: content.to_string().into(),
            tree: None,
            custom_ast: std::sync::Arc::new(None),
            modified_at: std::time::SystemTime::now().into(),
        };

        let vulnerabilities = detector.detect(&file).await.unwrap();
        assert!(vulnerabilities.len() >= 2);
        assert!(vulnerabilities
            .iter()
            .any(|v| v.description.contains("CORS")));
    }

    #[tokio::test]
    async fn test_csrf_detection_python() {
        let detector = CsrfDetector::new();
        let content = r#"
            WTF_CSRF_ENABLED = False

            @csrf_exempt
            @app.route('/api/data', methods=['POST'])
            def update_data():
                # Vulnerable endpoint
                pass
        "#;

        let file = ParsedFile {
            file_path: Arc::new(PathBuf::from("test.py")),
            language: SourceLanguage::Python,
            source: content.to_string().into(),
            tree: None,
            custom_ast: std::sync::Arc::new(None),
            modified_at: std::time::SystemTime::now().into(),
        };

        let vulnerabilities = detector.detect(&file).await.unwrap();
        assert!(vulnerabilities.len() >= 2);
        assert!(vulnerabilities
            .iter()
            .any(|v| v.severity == SecuritySeverity::Critical));
    }

    #[tokio::test]
    async fn test_csrf_detection_rust() {
        let detector = CsrfDetector::new();
        let content = r#"
            app.service(
                web::resource("/api/data")
                    .route(web::post().to(handler)) // No CSRF protection
            );

            cors().allow_any_origin().allow_any_method()
        "#;

        let file = ParsedFile {
            file_path: Arc::new(PathBuf::from("test.rs")),
            language: SourceLanguage::Rust,
            source: content.to_string().into(),
            tree: None,
            custom_ast: std::sync::Arc::new(None),
            modified_at: std::time::SystemTime::now().into(),
        };

        let vulnerabilities = detector.detect(&file).await.unwrap();
        assert!(vulnerabilities.len() >= 1);
        assert!(vulnerabilities
            .iter()
            .any(|v| v.description.contains("CORS")));
    }

    #[test]
    fn test_remediation_generation() {
        let remediation = CsrfDetector::generate_remediation(
            &CsrfType::MissingTokenValidation,
            "Express.js route",
        );
        assert!(remediation.contains("CSRF token validation"));
        assert!(remediation.contains("csurf middleware"));
    }
}
