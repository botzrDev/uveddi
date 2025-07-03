//! Database module for Uveddi
//!
//! This module provides database access, models, and CRUD operations for analysis runs,
//! anti-pattern types, architectural issues, and projects.

pub mod crud;
pub mod models;

pub use self::crud::Database;
