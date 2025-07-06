//! AI module for Uveddi
//!
//! This module provides AI-powered analysis capabilities, including integration with local and API-based LLMs.
//! Submodules implement providers, prompt templates, and the main AI analysis engine.

//! The `ai` module provides interfaces and implementations for integrating AI-powered
//! analysis into the system. It includes providers for different AI models and services,
//! as well as logic for generating prompts and parsing AI-generated responses.

/// The `api` submodule defines the traits and data structures for interacting with AI providers.
pub mod api;
/// The `engine` submodule contains the core logic for the AI analysis engine.
pub mod engine;
/// The `prompts` submodule provides tools for building and managing prompts for the AI models.
pub mod prompts;

pub use engine::AiAnalysisEngine;
pub use prompts::{IssueContext, PromptGenerator};
