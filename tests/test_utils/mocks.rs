//! Mock implementations for external dependencies
//!
//! This module provides mockall-based mock implementations for traits
//! used by the AnalysisEngine to enable isolated unit testing.

use mockall::predicate::*;
use mockall::mock;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::analysis::detectors::dependency::Dependency;
use crate::analysis::errors::AnalysisError;
use crate::analysis::traits::{AstParserTrait, DependencyExtractorTrait, ResultCacheTrait, CacheStats};
use crate::ast::{ParsedFile, SourceLanguage};

// Mock for AstParserTrait
mock! {
    pub AstParser {}
    
    impl AstParserTrait for AstParser {
        fn parse_file(&self, path: &Path) -> Result<ParsedFile, AnalysisError>;
        fn is_initialized(&self) -> bool;
        fn supported_languages(&self) -> Vec<SourceLanguage>;
    }
    
    // Required for Send + Sync traits
    impl Clone for AstParser {
        fn clone(&self) -> Self;
    }
}

// Mock for DependencyExtractorTrait
mock! {
    pub DependencyExtractor {}
    
    impl DependencyExtractorTrait for DependencyExtractor {
        fn extract_from_ast(&self, parsed_file: &ParsedFile) -> Result<Vec<Dependency>, AnalysisError>;
        fn supports_language(&self, language: &SourceLanguage) -> bool;
    }
    
    impl Clone for DependencyExtractor {
        fn clone(&self) -> Self;
    }
}

// Mock for ResultCacheTrait
mock! {
    pub ResultCache {}
    
    impl ResultCacheTrait for ResultCache {
        fn get_json(&self, key: &str) -> Result<Option<String>, AnalysisError>;
        fn set_json(&self, key: &str, value: &str) -> Result<(), AnalysisError>;
        fn clear(&self);
        fn get_stats(&self) -> CacheStats;
    }
    
    impl Clone for ResultCache {
        fn clone(&self) -> Self;
    }
}

// Mock for AnalysisDetector trait (async version)
mock! {
    pub AnalysisDetector {}
    
    #[async_trait::async_trait]
    impl crate::analysis::AnalysisDetector for AnalysisDetector {
        async fn detect_issues(&self, parsed_file: &ParsedFile) -> Result<Vec<crate::database::models::ArchitecturalIssue>, AnalysisError>;
        fn detector_name(&self) -> &'static str;
        fn detector_type(&self) -> crate::database::models::AntiPatternType;
    }
    
    impl Clone for AnalysisDetector {
        fn clone(&self) -> Self;
    }
}

/// Helper functions for creating common mock setups

impl MockAstParser {
    /// Creates a mock AST parser that successfully parses any file
    pub fn create_successful() -> Self {
        let mut mock = MockAstParser::new();
        
        mock.expect_parse_file()
            .returning(|path| {
                Ok(ParsedFile {
                    path: path.to_path_buf(),
                    content: "mock content".to_string(),
                    language: SourceLanguage::Rust,
                    ast: None, // For testing, we can use None
                })
            });
            
        mock.expect_is_initialized()
            .returning(|| true);
            
        mock.expect_supported_languages()
            .returning(|| vec![SourceLanguage::Rust, SourceLanguage::Python]);
            
        mock
    }
    
    /// Creates a mock AST parser that fails to parse files
    pub fn create_failing() -> Self {
        let mut mock = MockAstParser::new();
        
        mock.expect_parse_file()
            .returning(|_| Err(AnalysisError::ParseError("Mock parse failure".to_string())));
            
        mock.expect_is_initialized()
            .returning(|| false);
            
        mock.expect_supported_languages()
            .returning(|| vec![]);
            
        mock
    }
}

impl MockDependencyExtractor {
    /// Creates a mock dependency extractor that returns sample dependencies
    pub fn create_with_dependencies(deps: Vec<Dependency>) -> Self {
        let mut mock = MockDependencyExtractor::new();
        
        mock.expect_extract_from_ast()
            .returning(move |_| Ok(deps.clone()));
            
        mock.expect_supports_language()
            .returning(|_| true);
            
        mock
    }
    
    /// Creates a mock dependency extractor that returns no dependencies
    pub fn create_empty() -> Self {
        let mut mock = MockDependencyExtractor::new();
        
        mock.expect_extract_from_ast()
            .returning(|_| Ok(vec![]));
            
        mock.expect_supports_language()
            .returning(|_| true);
            
        mock
    }
}

impl MockResultCache {
    /// Creates a mock cache that simulates cache hits and misses
    pub fn create_with_data(data: HashMap<String, String>) -> Self {
        let mut mock = MockResultCache::new();
        
        mock.expect_get_json()
            .returning(move |key| {
                Ok(data.get(key).cloned())
            });
            
        mock.expect_set_json()
            .returning(|_, _| Ok(()));
            
        mock.expect_clear()
            .returning(|| ());
            
        mock.expect_get_stats()
            .returning(|| CacheStats {
                hits: 10,
                misses: 5,
                size: 1024,
                max_size: 10240,
            });
            
        mock
    }
    
    /// Creates a mock cache that always misses
    pub fn create_empty() -> Self {
        let mut mock = MockResultCache::new();
        
        mock.expect_get_json()
            .returning(|_| Ok(None));
            
        mock.expect_set_json()
            .returning(|_, _| Ok(()));
            
        mock.expect_clear()
            .returning(|| ());
            
        mock.expect_get_stats()
            .returning(|| CacheStats {
                hits: 0,
                misses: 10,
                size: 0,
                max_size: 10240,
            });
            
        mock
    }
}

impl MockAnalysisDetector {
    /// Creates a mock detector that finds issues
    pub fn create_with_issues(issues: Vec<crate::database::models::ArchitecturalIssue>) -> Self {
        let mut mock = MockAnalysisDetector::new();
        
        mock.expect_detect_issues()
            .returning(move |_| Ok(issues.clone()));
            
        mock.expect_detector_name()
            .returning(|| "MockDetector");
            
        mock.expect_detector_type()
            .returning(|| crate::database::models::AntiPatternType::DeadCode);
            
        mock
    }
    
    /// Creates a mock detector that finds no issues
    pub fn create_clean() -> Self {
        let mut mock = MockAnalysisDetector::new();
        
        mock.expect_detect_issues()
            .returning(|_| Ok(vec![]));
            
        mock.expect_detector_name()
            .returning(|| "CleanMockDetector");
            
        mock.expect_detector_type()
            .returning(|| crate::database::models::AntiPatternType::DeadCode);
            
        mock
    }
}

/// Helper function to create sample dependency for testing
pub fn create_sample_dependency(name: &str, from: &str, to: &str) -> Dependency {
    Dependency {
        name: name.to_string(),
        from_file: from.to_string(),
        to_file: to.to_string(),
        dependency_type: "use".to_string(),
        line_number: 1,
    }
}

/// Helper function to create sample architectural issue for testing
pub fn create_sample_issue(
    issue_type: crate::database::models::AntiPatternType,
    description: &str,
    file_path: &str,
) -> crate::database::models::ArchitecturalIssue {
    crate::database::models::ArchitecturalIssue {
        id: None,
        issue_type,
        description: description.to_string(),
        file_path: file_path.to_string(),
        line_number: Some(1),
        column_number: Some(1),
        severity: "medium".to_string(),
        confidence: 0.8,
        metadata: None,
    }
}