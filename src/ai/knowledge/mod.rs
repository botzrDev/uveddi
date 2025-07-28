//! AI Knowledge Library Infrastructure
//! 
//! This module provides the complete infrastructure for Uveddi's AI Knowledge Library,
//! including schema definitions, compression, indexing, loading, performance monitoring,
//! concurrent access patterns, AI integration, error handling, and configuration management.
//!
//! Key Features:
//! - Hierarchical schema optimized for AI consumption and zstd compression
//! - Perfect Hash Function (PHF) indexing for O(1) lookups
//! - Dictionary-trained compression achieving 70-75% size reduction
//! - Production-ready loading with integrity validation and error recovery
//! - High-performance concurrent access with thread-safety guarantees
//! - AI engine integration with smart prompt building and context optimization
//! - Comprehensive monitoring, alerting, and observability
//! - Extensible design supporting language-specific and custom knowledge
//! - Build-time preprocessing for compression and index generation

// Core modules
pub mod schema;
pub mod compression;
pub mod indexing;
pub mod loader;

// Advanced infrastructure modules
pub mod performance;
pub mod concurrent;
pub mod integration;
pub mod errors;
pub mod config;

// Knowledge content modules
pub mod patterns;
pub mod language_integration;
pub mod population;
pub mod validation;
pub mod build_integration;

// Re-export core functionality
pub use schema::*;
pub use compression::*;
pub use indexing::*;
pub use loader::*;

// Re-export infrastructure components
pub use performance::{
    KnowledgePerformanceMonitor, PerformanceMetrics, PerformanceReport,
    OptimizationSuggestion, PerformanceTargets
};
pub use concurrent::{
    ConcurrentKnowledgeAccess, AccessStatistics, ConcurrentHealthStatus,
    UpdateNotification, UpdateReceiver
};
pub use integration::{
    AIKnowledgeIntegration, EnhancedPrompt, PatternContext, ContextRequirements,
    SmartPromptBuilder, RelatedPattern
};
pub use errors::{
    KnowledgeInfrastructureError, ErrorRecoverySystem, RecoveryResult,
    MinimalKnowledgeAccess, CircuitBreaker
};
pub use config::{
    KnowledgeInfrastructureConfig, ConfigurationManager, MonitoringSystem,
    PerformanceConfig, MonitoringConfig
};

// Re-export knowledge content functionality
pub use population::{
    populate_knowledge_library, populate_from_detector_analysis, PopulationError
};
pub use validation::{
    KnowledgeValidator, ValidationReport, ValidationIssue, Severity
};
pub use build_integration::{
    KnowledgeLibraryBuilder, BuildArtifacts, CompressionMetadata,
    generate_knowledge_library_for_build
};
pub use language_integration::{
    LanguageKnowledgeIntegrator, EnhancedPattern, LanguageContext, PatternMetadata,
    LanguageIndices, IntegratedKnowledgeFactory
};