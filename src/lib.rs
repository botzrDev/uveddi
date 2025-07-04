//! Uveddi main library module
//!
//! This crate provides the core functionality for the Uveddi architectural analysis tool.
//! It exposes modules for AI-powered analysis, AST parsing, reporting,
//! database management, and more. See each submodule for details.

pub mod ai;
pub mod analysis;
pub mod application;
pub mod ast;
pub mod cache;
pub mod cli;
pub mod config;
pub mod database;
pub mod error;
pub mod ingestion;
pub mod models;
pub mod report;
