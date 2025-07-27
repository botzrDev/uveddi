//! AI module for Uveddi
//!
//! This module provides AI-powered analysis capabilities, including integration with local and API-based LLMs.
//! Submodules implement providers, prompt templates, and the main AI analysis engine.

//! The `ai` module provides interfaces and implementations for integrating AI-powered
//! analysis into the system. It includes providers for different AI models and services,
//! as well as logic for generating prompts and parsing AI-generated responses.

/// The `api` submodule defines the traits and data structures for interacting with AI providers.
#[cfg(feature = "ai")]
pub mod api;
/// The `engine` submodule contains the core logic for the AI analysis engine.
#[cfg(feature = "ai")]
pub mod engine;
/// The `ollama_provider` submodule provides Ollama LLM integration.
#[cfg(feature = "ai")]
pub mod ollama_provider;
/// The `prompts` submodule provides tools for building and managing prompts for the AI models.
#[cfg(feature = "ai")]
pub mod prompts;
/// The `knowledge` submodule provides the AI Knowledge Library with compressed pattern data.
#[cfg(feature = "ai")]
pub mod knowledge;

#[cfg(feature = "ai")]
pub use engine::AiAnalysisEngine;
