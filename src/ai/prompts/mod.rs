//! This module provides tools for building and managing prompts for the AI models.

/// Builds the context for a given issue, which is used to generate a prompt.
pub mod context_builder;
/// Contains the templates for the prompts that are sent to the AI models.
pub mod prompt_templates;

pub use context_builder::IssueContext;
pub mod smart_prompting;
