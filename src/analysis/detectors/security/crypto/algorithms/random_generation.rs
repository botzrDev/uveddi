//! Random Number Generation Analysis
//!
//! This module analyzes random number generation for cryptographic security issues,
//! including weak PRNGs, insufficient entropy, and improper seeding.

use crate::analysis::detectors::security::crypto::types::{CryptoFinding, CryptoFindingType};
use crate::analysis::detectors::security::types::SecuritySeverity;
use crate::analysis::AnalysisError;
use crate::ast::SourceLanguage;
use regex::Regex;
use std::collections::HashMap;

/// Analyzer for random number generation
pub struct RandomGenerationAnalyzer {
    weak_prngs: HashMap<SourceLanguage, Vec<WeakPrngPattern>>,
    entropy_patterns: HashMap<SourceLanguage, Vec<EntropyPattern>>,
    seeding_patterns: HashMap<SourceLanguage, Vec<SeedingPattern>>,
}

/// Pattern for detecting weak pseudo-random number generators
#[derive(Debug, Clone)]
pub struct WeakPrngPattern {
    pub name: String,
    pub pattern: String,
    pub description: String,
    pub severity: SecuritySeverity,
    pub confidence: f64,
    pub replacement: String,
    pub prng_type: PrngType,
}

/// Pattern for detecting entropy-related issues
#[derive(Debug, Clone)]
pub struct EntropyPattern {
    pub name: String,
    pub pattern: String,
    pub description: String,
    pub severity: SecuritySeverity,
    pub confidence: f64,
    pub recommendation: String,
}

/// Pattern for detecting improper seeding
#[derive(Debug, Clone)]
pub struct SeedingPattern {
    pub name: String,
    pub pattern: String,
    pub description: String,
    pub severity: SecuritySeverity,
    pub confidence: f64,
    pub recommendation: String,
}

/// Types of pseudo-random number generators
#[derive(Debug, Clone, PartialEq)]
pub enum PrngType {
    SystemRandom,      // OS-provided CSPRNG
    CryptographicPrng, // Cryptographically secure PRNG
    WeakPrng,          // Non-cryptographic PRNG
    DeterministicPrng, // Deterministic/predictable PRNG
}

impl RandomGenerationAnalyzer {
    /// Create a new random generation analyzer
    pub fn new() -> Self {
        Self {
            weak_prngs: Self::initialize_weak_prngs(),
            entropy_patterns: Self::initialize_entropy_patterns(),
            seeding_patterns: Self::initialize_seeding_patterns(),
        }
    }

    /// Analyze content for random generation issues
    pub fn analyze(&self, content: &str, language: &SourceLanguage) -> Result<Vec<CryptoFinding>, AnalysisError> {
        let mut findings = Vec::new();

        // Check for weak PRNGs
        if let Some(weak_patterns) = self.weak_prngs.get(language) {
            findings.extend(self.detect_weak_prngs(content, weak_patterns)?);
        }

        // Check for entropy issues
        if let Some(entropy_patterns) = self.entropy_patterns.get(language) {
            findings.extend(self.detect_entropy_issues(content, entropy_patterns)?);
        }

        // Check for seeding issues
        if let Some(seeding_patterns) = self.seeding_patterns.get(language) {
            findings.extend(self.detect_seeding_issues(content, seeding_patterns)?);
        }

        Ok(findings)
    }

    /// Initialize weak PRNG patterns for each language
    fn initialize_weak_prngs() -> HashMap<SourceLanguage, Vec<WeakPrngPattern>> {
        let mut patterns = HashMap::new();

        // Rust patterns
        patterns.insert(SourceLanguage::Rust, vec![
            WeakPrngPattern {
                name: "Standard Random".to_string(),
                pattern: r"rand::random\s*\(\s*\)|thread_rng\(\)\.gen".to_string(),
                description: "Standard random generators may not be cryptographically secure".to_string(),
                severity: SecuritySeverity::Medium,
                confidence: 0.7,
                replacement: "Use rand::rngs::OsRng or ring::rand".to_string(),
                prng_type: PrngType::WeakPrng,
            },
            WeakPrngPattern {
                name: "LCG Random".to_string(),
                pattern: r"(?i)(lcg|linear_congruential)".to_string(),
                description: "Linear congruential generators are predictable".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.9,
                replacement: "Use cryptographically secure random number generator".to_string(),
                prng_type: PrngType::DeterministicPrng,
            },
            WeakPrngPattern {
                name: "Fixed Seed PRNG".to_string(),
                pattern: r"(?i)seed\s*\(\s*\d+\s*\)".to_string(),
                description: "Using fixed seed makes random numbers predictable".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.8,
                replacement: "Use system entropy for seeding or avoid seeding".to_string(),
                prng_type: PrngType::DeterministicPrng,
            },
        ]);

        // Python patterns
        patterns.insert(SourceLanguage::Python, vec![
            WeakPrngPattern {
                name: "Python random module".to_string(),
                pattern: r"random\.(random|randint|choice|shuffle|sample)\s*\(".to_string(),
                description: "Python's random module is not cryptographically secure".to_string(),
                severity: SecuritySeverity::Medium,
                confidence: 0.8,
                replacement: "Use secrets module or os.urandom()".to_string(),
                prng_type: PrngType::WeakPrng,
            },
            WeakPrngPattern {
                name: "NumPy random".to_string(),
                pattern: r"np\.random\.|numpy\.random\.".to_string(),
                description: "NumPy random functions are not cryptographically secure".to_string(),
                severity: SecuritySeverity::Medium,
                confidence: 0.8,
                replacement: "Use secrets module for cryptographic purposes".to_string(),
                prng_type: PrngType::WeakPrng,
            },
            WeakPrngPattern {
                name: "Fixed seed".to_string(),
                pattern: r"random\.seed\s*\(\s*\d+\s*\)".to_string(),
                description: "Fixed seed makes random sequence predictable".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.9,
                replacement: "Remove fixed seeding or use secrets module".to_string(),
                prng_type: PrngType::DeterministicPrng,
            },
        ]);

        // JavaScript patterns
        let js_patterns = vec![
            WeakPrngPattern {
                name: "Math.random()".to_string(),
                pattern: r"Math\.random\s*\(\s*\)".to_string(),
                description: "Math.random() is not cryptographically secure".to_string(),
                severity: SecuritySeverity::Medium,
                confidence: 0.9,
                replacement: "Use crypto.getRandomValues() or crypto.randomBytes()".to_string(),
                prng_type: PrngType::WeakPrng,
            },
            WeakPrngPattern {
                name: "Predictable random".to_string(),
                pattern: r"new\s+Date\(\)\.getTime\(\).*random".to_string(),
                description: "Using timestamp as random seed is predictable".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.8,
                replacement: "Use proper cryptographic random number generator".to_string(),
                prng_type: PrngType::DeterministicPrng,
            },
        ];

        patterns.insert(SourceLanguage::JavaScript, js_patterns.clone());
        patterns.insert(SourceLanguage::TypeScript, js_patterns);

        patterns
    }

    /// Initialize entropy patterns
    fn initialize_entropy_patterns() -> HashMap<SourceLanguage, Vec<EntropyPattern>> {
        let mut patterns = HashMap::new();

        // Common patterns for insufficient entropy
        let common_patterns = vec![
            EntropyPattern {
                name: "Low entropy random".to_string(),
                pattern: r"(?i)random.*[^0-9][0-9]{1,2}[^0-9]".to_string(), // Small ranges
                description: "Random number generation with low entropy".to_string(),
                severity: SecuritySeverity::Medium,
                confidence: 0.6,
                recommendation: "Use larger random space and cryptographic generators".to_string(),
            },
            EntropyPattern {
                name: "Time-based randomness".to_string(),
                pattern: r"(?i)(time|date|timestamp).*random".to_string(),
                description: "Using time as source of randomness provides low entropy".to_string(),
                severity: SecuritySeverity::Medium,
                confidence: 0.7,
                recommendation: "Use system entropy sources".to_string(),
            },
        ];

        patterns.insert(SourceLanguage::Rust, common_patterns.clone());
        patterns.insert(SourceLanguage::Python, common_patterns.clone());
        patterns.insert(SourceLanguage::JavaScript, common_patterns.clone());
        patterns.insert(SourceLanguage::TypeScript, common_patterns);

        patterns
    }

    /// Initialize seeding patterns
    fn initialize_seeding_patterns() -> HashMap<SourceLanguage, Vec<SeedingPattern>> {
        let mut patterns = HashMap::new();

        // Python-specific seeding patterns
        patterns.insert(SourceLanguage::Python, vec![
            SeedingPattern {
                name: "Fixed random seed".to_string(),
                pattern: r"random\.seed\s*\(\s*['\"]?\w+['\"]?\s*\)".to_string(),
                description: "Fixed seed makes random sequence deterministic".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.9,
                recommendation: "Remove seeding or use system entropy".to_string(),
            },
            SeedingPattern {
                name: "Simple seed value".to_string(),
                pattern: r"random\.seed\s*\(\s*[01]\s*\)".to_string(),
                description: "Simple seed values provide no security".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.95,
                recommendation: "Use cryptographically secure random generator".to_string(),
            },
        ]);

        // JavaScript seeding patterns
        let js_seeding_patterns = vec![
            SeedingPattern {
                name: "Predictable seed".to_string(),
                pattern: r"seed\s*[:=]\s*\d{1,6}".to_string(),
                description: "Simple numeric seeds are easily guessable".to_string(),
                severity: SecuritySeverity::High,
                confidence: 0.8,
                recommendation: "Use crypto.getRandomValues() instead".to_string(),
            },
        ];

        patterns.insert(SourceLanguage::JavaScript, js_seeding_patterns.clone());
        patterns.insert(SourceLanguage::TypeScript, js_seeding_patterns);

        patterns
    }

    /// Detect weak pseudo-random number generators
    fn detect_weak_prngs(&self, content: &str, patterns: &[WeakPrngPattern]) -> Result<Vec<CryptoFinding>, AnalysisError> {
        let mut findings = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            for pattern in patterns {
                if let Ok(regex) = Regex::new(&pattern.pattern) {
                    if regex.is_match(line) && !self.is_likely_safe_context(line) {
                        let cwe_id = match pattern.prng_type {
                            PrngType::WeakPrng => Some(338), // Use of Cryptographically Weak PRNG
                            PrngType::DeterministicPrng => Some(330), // Use of Insufficiently Random Values
                            _ => Some(338),
                        };

                        findings.push(CryptoFinding {
                            finding_type: CryptoFindingType::WeakRandom,
                            line_number: line_num + 1,
                            column: 0,
                            description: pattern.description.clone(),
                            severity: pattern.severity,
                            confidence: pattern.confidence,
                            algorithm: Some(pattern.name.clone()),
                            recommendation: format!("Replace with {}", pattern.replacement),
                            cwe_id,
                        });
                    }
                }
            }
        }

        Ok(findings)
    }

    /// Detect entropy-related issues
    fn detect_entropy_issues(&self, content: &str, patterns: &[EntropyPattern]) -> Result<Vec<CryptoFinding>, AnalysisError> {
        let mut findings = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            for pattern in patterns {
                if let Ok(regex) = Regex::new(&pattern.pattern) {
                    if regex.is_match(line) {
                        findings.push(CryptoFinding {
                            finding_type: CryptoFindingType::EntropyIssue,
                            line_number: line_num + 1,
                            column: 0,
                            description: pattern.description.clone(),
                            severity: pattern.severity,
                            confidence: pattern.confidence,
                            algorithm: Some(pattern.name.clone()),
                            recommendation: pattern.recommendation.clone(),
                            cwe_id: Some(330), // Use of Insufficiently Random Values
                        });
                    }
                }
            }
        }

        Ok(findings)
    }

    /// Detect seeding-related issues
    fn detect_seeding_issues(&self, content: &str, patterns: &[SeedingPattern]) -> Result<Vec<CryptoFinding>, AnalysisError> {
        let mut findings = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            for pattern in patterns {
                if let Ok(regex) = Regex::new(&pattern.pattern) {
                    if regex.is_match(line) {
                        findings.push(CryptoFinding {
                            finding_type: CryptoFindingType::WeakRandom,
                            line_number: line_num + 1,
                            column: 0,
                            description: pattern.description.clone(),
                            severity: pattern.severity,
                            confidence: pattern.confidence,
                            algorithm: Some(pattern.name.clone()),
                            recommendation: pattern.recommendation.clone(),
                            cwe_id: Some(330), // Use of Insufficiently Random Values
                        });
                    }
                }
            }
        }

        Ok(findings)
    }

    /// Check if random usage appears to be in a safe context
    fn is_likely_safe_context(&self, line: &str) -> bool {
        let safe_indicators = [
            "test", "demo", "example", "simulation", "game",
            "shuffle", "sample", "non-crypto", "mock"
        ];

        let line_lower = line.to_lowercase();
        safe_indicators.iter().any(|&indicator|
            line_lower.contains(indicator)
        )
    }

    /// Check if random generation appears secure
    pub fn is_secure_usage(&self, line: &str) -> bool {
        let secure_indicators = [
            "osrng", "crypto", "secure", "urandom", "getrandomvalues",
            "randombytes", "secrets", "system_random", "csprng"
        ];

        secure_indicators.iter().any(|&indicator|
            line.to_lowercase().contains(indicator)
        )
    }

    /// Get recommended secure random number generators by language
    pub fn get_secure_recommendations() -> HashMap<&'static str, Vec<&'static str>> {
        let mut recommendations = HashMap::new();

        recommendations.insert("rust", vec![
            "rand::rngs::OsRng",
            "ring::rand::SystemRandom",
            "getrandom::getrandom()",
            "rustls::crypto::ring::default_provider()",
        ]);

        recommendations.insert("python", vec![
            "secrets module (secrets.randbits, secrets.token_bytes)",
            "os.urandom()",
            "cryptography.hazmat.primitives.random",
            "hashlib.pbkdf2_hmac with os.urandom salt",
        ]);

        recommendations.insert("javascript", vec![
            "crypto.getRandomValues() (browser)",
            "crypto.randomBytes() (Node.js)",
            "crypto.randomUUID() (Node.js 14.17+)",
            "webcrypto.getRandomValues() (modern environments)",
        ]);

        recommendations.insert("general_guidelines", vec![
            "Use OS-provided entropy sources",
            "Avoid seeding cryptographic generators",
            "Use at least 128 bits of entropy for keys",
            "Test random output for statistical quality",
        ]);

        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weak_prng_detection() {
        let analyzer = RandomGenerationAnalyzer::new();
        let content = "Math.random() * 1000";
        let findings = analyzer.analyze(content, &SourceLanguage::JavaScript).unwrap();

        assert!(!findings.is_empty());
        assert_eq!(findings[0].finding_type, CryptoFindingType::WeakRandom);
    }

    #[test]
    fn test_fixed_seed_detection() {
        let analyzer = RandomGenerationAnalyzer::new();
        let content = "random.seed(12345)";
        let findings = analyzer.analyze(content, &SourceLanguage::Python).unwrap();

        assert!(!findings.is_empty());
        assert!(findings[0].description.contains("deterministic"));
    }

    #[test]
    fn test_safe_context_detection() {
        let analyzer = RandomGenerationAnalyzer::new();
        assert!(analyzer.is_likely_safe_context("// test random shuffle"));
        assert!(!analyzer.is_likely_safe_context("token = random()"));
    }

    #[test]
    fn test_secure_usage_detection() {
        let analyzer = RandomGenerationAnalyzer::new();
        assert!(analyzer.is_secure_usage("crypto.getRandomValues()"));
        assert!(!analyzer.is_secure_usage("Math.random()"));
    }
}