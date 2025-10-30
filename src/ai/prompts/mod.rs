//! This module provides tools for building and managing prompts for the AI models.

/// Builds the context for a given issue, which is used to generate a prompt.
pub mod context_builder;
/// Contains the templates for the prompts that are sent to the AI models.
pub mod prompt_templates;
/// Enhanced prompt templates with advanced formatting (disabled for alpha)
// pub mod enhanced_templates;
/// Knowledge integration for AI prompts (disabled for alpha)
// pub mod knowledge_integration;
/// Core templates module
pub mod templates;

pub use context_builder::IssueContext;
pub mod smart_prompting;
