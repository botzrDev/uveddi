//! Community Management Module
//!
//! This module provides a complete community member management system separate from
//! the main analysis database. It's designed for cloud deployment and admin management
//! of community members who sign up for access to the community hub.
//!
//! # Features
//!
//! - **Member Management**: Registration, profile updates, deactivation
//! - **Role-Based Access**: Member, Developer, Admin roles with permissions
//! - **Activity Tracking**: Login history, feature usage, engagement metrics
//! - **Marketing Analytics**: Profile data for marketing insights
//! - **Admin Dashboard Ready**: Designed for easy web interface integration
//!
//! # Database Design
//!
//! Uses a separate SQLite database (`community.db`) with the following tables:
//! - `community_members`: Core member information and profiles
//! - `member_activity`: Activity tracking and engagement metrics
//! - `member_sessions`: Login sessions and authentication tracking
//!
//! # Usage
//!
//! ```rust,no_run
//! use uveddi::community::{CommunityDatabase, MemberRole};
//!
//! // Initialize community database
//! let db = CommunityDatabase::new("community.db")?;
//!
//! // Register a new member
//! let member = db.register_member(
//!     "alice@example.com",
//!     "Alice Johnson",
//!     MemberRole::Member
//! )?;
//!
//! // Track activity
//! db.log_activity(&member.id, "login", Some("Dashboard viewed"))?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

/// Placeholder documentation for public items

pub mod database;
pub mod models;
pub mod analytics;

pub use database::CommunityDatabase;
pub use models::{CommunityMember, MemberRole, MemberActivity, MemberProfile, ActivityType};
pub use analytics::MemberAnalytics;
