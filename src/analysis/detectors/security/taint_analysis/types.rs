//! Core types and data structures for taint analysis

pub mod core_types;
pub mod extended_types;

// Re-export all core types
pub use core_types::{
    LanguageTaintPatterns, SanitizationPoint, SourceLocation, TaintLevel, TaintSink, TaintSource,
};

// Re-export all extended types
pub use extended_types::{
    ConfidenceMetrics, DataFlowGraph, DataFlowNode, DataFlowNodeType, PropagationType,
    SanitizationEffectiveness, TaintAnalysisContext, TaintFlow, TaintPropagationRule,
};
