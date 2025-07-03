//! AI module for Uveddi
//!
//! This module provides AI-powered analysis capabilities, including integration with local and API-based LLMs.
//! Submodules implement providers, prompt templates, and the main AI analysis engine.

pub mod api;
pub mod engine;
pub mod ollama_provider;
pub mod prompts;
pub mod types;

pub use self::engine::AiAnalysisEngine;
