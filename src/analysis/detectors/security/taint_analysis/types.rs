//! Core types and data structures for taint analysis

pub mod core_types;
pub mod extended_types;

// Re-export all core types
pub use core_types::{
    TaintSource, TaintSink, SanitizationPoint, SourceLocation,
    TaintLevel, LanguageTaintPatterns
};

// Re-export all extended types
pub use extended_types::{
    DataFlowNode, DataFlowNodeType, DataFlowGraph, TaintFlow,
    TaintAnalysisContext, ConfidenceMetrics, TaintPropagationRule,
    PropagationType, SanitizationEffectiveness
};