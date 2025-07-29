//! Community Database Implementation
//!
//! SQLite-based database for community member management, completely separate from
//! the main analysis database. Optimized for cloud deployment and admin management.

use crate::community::models::{
    ActivityType, AdminLevel, AdminPermissions, AdminProfile, ApiAccessLevel, BadgeType,
    CommunityMember, DeveloperBadge, DeveloperProfile, DeveloperType, MemberActivity,
    MemberProfile, MemberRole, RolePermissions,
};
use crate::error::{DeserializationError, RusqliteError, UveddiError};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, Result as SqlResult, Row};
use serde_json;
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;

/// Community database manager
pub struct CommunityDatabase {
    pub(crate) conn: Connection,
}

impl CommunityDatabase {
    /// Create a new community database connection and initialize schema
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self, UveddiError> {
        let conn = Connection::open(db_path)?;
        let db = Self { conn };
        db.initialize_schema()?;
        Ok(db)
    }

    /// Create an in-memory database for testing
    pub fn new_in_memory() -> Result<Self, UveddiError> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.initialize_schema()?;
        Ok(db)
    }

    /// Initialize database schema with all required tables
    fn initialize_schema(&self) -> Result<(), UveddiError> {
        self.conn.execute_batch(
            r#"
            -- Community members table
            CREATE TABLE IF NOT EXISTS community_members (
                id TEXT PRIMARY KEY,
                email TEXT UNIQUE NOT NULL,
                name TEXT NOT NULL,
                role TEXT NOT NULL DEFAULT 'member',
                created_at TEXT NOT NULL,
                last_active TEXT,
                is_active BOOLEAN NOT NULL DEFAULT 1,
                email_verified BOOLEAN NOT NULL DEFAULT 0,
                avatar_url TEXT,
                -- Profile information (JSON)
                company TEXT,
                job_title TEXT,
                languages TEXT, -- JSON array
                experience_level TEXT,
                use_case TEXT,
                referral_source TEXT,
                newsletter_subscribed BOOLEAN NOT NULL DEFAULT 0,
                marketing_consent BOOLEAN NOT NULL DEFAULT 0,
                custom_fields TEXT -- JSON object
            );

            -- Admin profiles and permissions
            CREATE TABLE IF NOT EXISTS admin_profiles (
                member_id TEXT PRIMARY KEY,
                admin_level TEXT NOT NULL DEFAULT 'moderator',
                permissions TEXT NOT NULL, -- JSON object with AdminPermissions
                assigned_regions TEXT, -- JSON array
                assigned_teams TEXT, -- JSON array  
                admin_notes TEXT,
                appointed_by TEXT,
                appointed_at TEXT NOT NULL,
                last_admin_action TEXT,
                is_super_admin BOOLEAN NOT NULL DEFAULT 0,
                FOREIGN KEY (member_id) REFERENCES community_members (id) ON DELETE CASCADE,
                FOREIGN KEY (appointed_by) REFERENCES community_members (id)
            );

            -- Developer profiles and access
            CREATE TABLE IF NOT EXISTS developer_profiles (
                member_id TEXT PRIMARY KEY,
                developer_type TEXT NOT NULL DEFAULT 'opensource',
                specializations TEXT, -- JSON array
                github_verified BOOLEAN NOT NULL DEFAULT 0,
                contribution_score INTEGER NOT NULL DEFAULT 0,
                api_access_level TEXT NOT NULL DEFAULT 'basic',
                repositories TEXT, -- JSON array
                verified_at TEXT,
                verification_method TEXT,
                mentor_status BOOLEAN NOT NULL DEFAULT 0,
                beta_tester BOOLEAN NOT NULL DEFAULT 0,
                FOREIGN KEY (member_id) REFERENCES community_members (id) ON DELETE CASCADE
            );

            -- Developer badges and achievements
            CREATE TABLE IF NOT EXISTS developer_badges (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                member_id TEXT NOT NULL,
                badge_type TEXT NOT NULL,
                earned_at TEXT NOT NULL,
                description TEXT NOT NULL,
                metadata TEXT, -- JSON object
                FOREIGN KEY (member_id) REFERENCES developer_profiles (member_id) ON DELETE CASCADE
            );

            -- Role-specific permissions and settings
            CREATE TABLE IF NOT EXISTS role_permissions (
                role TEXT PRIMARY KEY,
                can_access_api BOOLEAN NOT NULL DEFAULT 0,
                can_view_source BOOLEAN NOT NULL DEFAULT 0,
                can_download_reports BOOLEAN NOT NULL DEFAULT 0,
                can_create_projects BOOLEAN NOT NULL DEFAULT 0,
                can_share_publicly BOOLEAN NOT NULL DEFAULT 0,
                can_invite_members BOOLEAN NOT NULL DEFAULT 0,
                rate_limit_tier INTEGER NOT NULL DEFAULT 1,
                storage_quota_mb INTEGER NOT NULL DEFAULT 100,
                features TEXT -- JSON array of enabled features
            );

            -- Member activity tracking
            CREATE TABLE IF NOT EXISTS member_activity (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                member_id TEXT NOT NULL,
                activity_type TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                description TEXT,
                ip_address TEXT,
                user_agent TEXT,
                metadata TEXT, -- JSON object
                FOREIGN KEY (member_id) REFERENCES community_members (id)
            );

            -- Member sessions for authentication tracking
            CREATE TABLE IF NOT EXISTS member_sessions (
                id TEXT PRIMARY KEY,
                member_id TEXT NOT NULL,
                created_at TEXT NOT NULL,
                expires_at TEXT NOT NULL,
                last_activity TEXT NOT NULL,
                ip_address TEXT,
                user_agent TEXT,
                is_active BOOLEAN NOT NULL DEFAULT 1,
                FOREIGN KEY (member_id) REFERENCES community_members (id)
            );

            -- Insert default role permissions
            INSERT OR REPLACE INTO role_permissions (
                role, can_access_api, can_view_source, can_download_reports,
                can_create_projects, can_share_publicly, can_invite_members,
                rate_limit_tier, storage_quota_mb, features
            ) VALUES
            ('member', 0, 0, 1, 1, 0, 0, 1, 100, '["basic_analysis"]'),
            ('developer', 1, 1, 1, 1, 1, 1, 2, 500, '["basic_analysis", "api_access", "source_view", "advanced_reports"]'),
            ('admin', 1, 1, 1, 1, 1, 1, 3, 1000, '["basic_analysis", "api_access", "source_view", "advanced_reports", "admin_panel", "user_management"]');

            -- Indexes for performance
            CREATE INDEX IF NOT EXISTS idx_member_email ON community_members(email);
            CREATE INDEX IF NOT EXISTS idx_member_role ON community_members(role);
            CREATE INDEX IF NOT EXISTS idx_member_active ON community_members(is_active);
            CREATE INDEX IF NOT EXISTS idx_admin_level ON admin_profiles(admin_level);
            CREATE INDEX IF NOT EXISTS idx_admin_super ON admin_profiles(is_super_admin);
            CREATE INDEX IF NOT EXISTS idx_dev_type ON developer_profiles(developer_type);
            CREATE INDEX IF NOT EXISTS idx_dev_verified ON developer_profiles(github_verified);
            CREATE INDEX IF NOT EXISTS idx_dev_api_level ON developer_profiles(api_access_level);
            CREATE INDEX IF NOT EXISTS idx_badges_member ON developer_badges(member_id);
            CREATE INDEX IF NOT EXISTS idx_badges_type ON developer_badges(badge_type);
            CREATE INDEX IF NOT EXISTS idx_activity_member ON member_activity(member_id);
            CREATE INDEX IF NOT EXISTS idx_activity_timestamp ON member_activity(timestamp);
            CREATE INDEX IF NOT EXISTS idx_activity_type ON member_activity(activity_type);
            CREATE INDEX IF NOT EXISTS idx_session_member ON member_sessions(member_id);
            CREATE INDEX IF NOT EXISTS idx_session_active ON member_sessions(is_active);
            "#
        )?;
        Ok(())
    }

    /// Register a new community member
    pub fn register_member(
        &self,
        email: &str,
        name: &str,
        role: MemberRole,
    ) -> Result<CommunityMember, UveddiError> {
        let member = CommunityMember {
            id: Uuid::new_v4().to_string(),
            email: email.to_string(),
            name: name.to_string(),
            role,
            created_at: Utc::now(),
            last_active: None,
            is_active: true,
            profile: MemberProfile::default(),
            email_verified: false,
            avatar_url: None,
        };

        let languages_json = serde_json::to_string(&member.profile.languages).map_err(|e| {
            UveddiError::database_error(
                "serialization",
                "community database",
                rusqlite::Error::ToSqlConversionFailure(Box::new(e)),
            )
        })?;
        let custom_fields_json =
            serde_json::to_string(&member.profile.custom_fields).map_err(|e| {
                UveddiError::database_error(
                    "serialization",
                    "community database",
                    rusqlite::Error::ToSqlConversionFailure(Box::new(e)),
                )
            })?;

        self.conn.execute(
            r#"
            INSERT INTO community_members (
                id, email, name, role, created_at, is_active, email_verified,
                company, job_title, languages, experience_level, use_case,
                referral_source, newsletter_subscribed, marketing_consent, custom_fields
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
            "#,
            (
                &member.id,
                &member.email,
                &member.name,
                member.role.as_str(),
                member.created_at.to_rfc3339(),
                member.is_active,
                member.email_verified,
                &member.profile.company,
                &member.profile.job_title,
                &languages_json,
                &member.profile.experience_level,
                &member.profile.use_case,
                &member.profile.referral_source,
                member.profile.newsletter_subscribed,
                member.profile.marketing_consent,
                &custom_fields_json,
            ),
        )?;

        // Log registration activity
        self.log_activity(&member.id, ActivityType::Registration, None, None, None)?;

        Ok(member)
    }

    /// Get member by email address
    pub fn get_member_by_email(&self, email: &str) -> Result<Option<CommunityMember>, UveddiError> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, email, name, role, created_at, last_active, is_active, email_verified,
                   avatar_url, company, job_title, languages, experience_level, use_case,
                   referral_source, newsletter_subscribed, marketing_consent, custom_fields
            FROM community_members WHERE email = ?1
            "#,
        )?;

        let mut member_iter = stmt.query_map([email], |row| {
            self.row_to_member(row).map_err(|e| match e {
                UveddiError::DatabaseError {
                    source: Some(sql_err),
                    ..
                } => sql_err,
                _ => rusqlite::Error::ToSqlConversionFailure(Box::new(e)),
            })
        })?;

        // Clippy fix UV-151: Replace for loop with if-let for single result
        if let Some(member) = member_iter.next() {
            Ok(Some(member?))
        } else {
            Ok(None)
        }
    }

    /// Get member by ID
    pub fn get_member_by_id(&self, id: &str) -> Result<Option<CommunityMember>, UveddiError> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, email, name, role, created_at, last_active, is_active, email_verified,
                   avatar_url, company, job_title, languages, experience_level, use_case,
                   referral_source, newsletter_subscribed, marketing_consent, custom_fields
            FROM community_members WHERE id = ?1
            "#,
        )?;

        let mut member_iter = stmt.query_map([id], |row| {
            self.row_to_member(row).map_err(|e| match e {
                UveddiError::DatabaseError {
                    source: Some(sql_err),
                    ..
                } => sql_err,
                _ => rusqlite::Error::ToSqlConversionFailure(Box::new(e)),
            })
        })?;

        // Clippy fix UV-151: Replace for loop with if-let for single result
        if let Some(member) = member_iter.next() {
            Ok(Some(member?))
        } else {
            Ok(None)
        }
    }

    /// List all members with optional filtering
    pub fn list_members(
        &self,
        role_filter: Option<MemberRole>,
        active_only: bool,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<CommunityMember>, UveddiError> {
        let mut sql = r#"
            SELECT id, email, name, role, created_at, last_active, is_active, email_verified,
                   avatar_url, company, job_title, languages, experience_level, use_case,
                   referral_source, newsletter_subscribed, marketing_consent, custom_fields
            FROM community_members WHERE 1=1
        "#
        .to_string();

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        let mut param_count = 0;

        if let Some(role) = role_filter {
            param_count += 1;
            sql.push_str(&format!(" AND role = ?{}", param_count));
            params.push(Box::new(role.as_str().to_string()));
        }

        if active_only {
            sql.push_str(" AND is_active = 1");
        }

        sql.push_str(" ORDER BY created_at DESC");

        if let Some(limit_val) = limit {
            param_count += 1;
            sql.push_str(&format!(" LIMIT ?{}", param_count));
            params.push(Box::new(limit_val as i64));
        }

        if let Some(offset_val) = offset {
            param_count += 1;
            sql.push_str(&format!(" OFFSET ?{}", param_count));
            params.push(Box::new(offset_val as i64));
        }

        let mut stmt = self.conn.prepare(&sql)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let member_iter = stmt.query_map(&param_refs[..], |row| {
            self.row_to_member(row).map_err(|e| match e {
                UveddiError::DatabaseError {
                    source: Some(sql_err),
                    ..
                } => sql_err,
                _ => rusqlite::Error::ToSqlConversionFailure(Box::new(e)),
            })
        })?;

        let mut members = Vec::new();
        for member in member_iter {
            members.push(member?);
        }
        Ok(members)
    }

    /// Update member profile
    pub fn update_member_profile(
        &self,
        member_id: &str,
        profile: &MemberProfile,
    ) -> Result<bool, UveddiError> {
        let languages_json = serde_json::to_string(&profile.languages).map_err(|e| {
            UveddiError::database_error(
                "serialization",
                "community database",
                rusqlite::Error::ToSqlConversionFailure(Box::new(e)),
            )
        })?;
        let custom_fields_json = serde_json::to_string(&profile.custom_fields).map_err(|e| {
            UveddiError::database_error(
                "serialization",
                "community database",
                rusqlite::Error::ToSqlConversionFailure(Box::new(e)),
            )
        })?;

        let affected = self.conn.execute(
            r#"
            UPDATE community_members SET
                company = ?1, job_title = ?2, languages = ?3, experience_level = ?4,
                use_case = ?5, referral_source = ?6, newsletter_subscribed = ?7,
                marketing_consent = ?8, custom_fields = ?9
            WHERE id = ?10
            "#,
            (
                &profile.company,
                &profile.job_title,
                &languages_json,
                &profile.experience_level,
                &profile.use_case,
                &profile.referral_source,
                profile.newsletter_subscribed,
                profile.marketing_consent,
                &custom_fields_json,
                member_id,
            ),
        )?;

        if affected > 0 {
            self.log_activity(member_id, ActivityType::ProfileUpdate, None, None, None)?;
        }

        Ok(affected > 0)
    }

    /// Update member role (admin function)
    pub fn update_member_role(
        &self,
        member_id: &str,
        new_role: MemberRole,
    ) -> Result<bool, UveddiError> {
        let affected = self.conn.execute(
            "UPDATE community_members SET role = ?1 WHERE id = ?2",
            (new_role.as_str(), member_id),
        )?;

        if affected > 0 {
            self.log_activity(
                member_id,
                ActivityType::RoleChanged,
                Some(&format!("Role changed to {}", new_role.as_str())),
                None,
                None,
            )?;
        }

        Ok(affected > 0)
    }

    /// Update member's last active timestamp
    pub fn update_last_active(&self, member_id: &str) -> Result<bool, UveddiError> {
        let affected = self.conn.execute(
            "UPDATE community_members SET last_active = ?1 WHERE id = ?2",
            (Utc::now().to_rfc3339(), member_id),
        )?;
        Ok(affected > 0)
    }

    /// Deactivate member account
    pub fn deactivate_member(&self, member_id: &str) -> Result<bool, UveddiError> {
        let affected = self.conn.execute(
            "UPDATE community_members SET is_active = 0 WHERE id = ?1",
            [member_id],
        )?;

        if affected > 0 {
            self.log_activity(
                member_id,
                ActivityType::AccountDeactivated,
                None,
                None,
                None,
            )?;
        }

        Ok(affected > 0)
    }

    /// Log member activity for analytics
    pub fn log_activity(
        &self,
        member_id: &str,
        activity_type: ActivityType,
        description: Option<&str>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<i64, UveddiError> {
        let metadata: HashMap<String, String> = HashMap::new();
        let metadata_json = serde_json::to_string(&metadata).map_err(|e| {
            UveddiError::database_error(
                "serialization",
                "community database",
                rusqlite::Error::ToSqlConversionFailure(Box::new(e)),
            )
        })?;

        self.conn.execute(
            r#"
            INSERT INTO member_activity (
                member_id, activity_type, timestamp, description, 
                ip_address, user_agent, metadata
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            (
                member_id,
                activity_type.as_str(),
                Utc::now().to_rfc3339(),
                description,
                ip_address,
                user_agent,
                &metadata_json,
            ),
        )?;

        // Update member's last active timestamp
        self.update_last_active(member_id)?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Get recent activity for a member
    pub fn get_member_activity(
        &self,
        member_id: &str,
        limit: Option<usize>,
    ) -> Result<Vec<MemberActivity>, UveddiError> {
        let sql = if let Some(limit_val) = limit {
            format!(
                r#"
                SELECT id, member_id, activity_type, timestamp, description, 
                       ip_address, user_agent, metadata
                FROM member_activity 
                WHERE member_id = ?1 
                ORDER BY timestamp DESC 
                LIMIT {}
                "#,
                limit_val
            )
        } else {
            r#"
            SELECT id, member_id, activity_type, timestamp, description, 
                   ip_address, user_agent, metadata
            FROM member_activity 
            WHERE member_id = ?1 
            ORDER BY timestamp DESC
            "#
            .to_string()
        };

        let mut stmt = self.conn.prepare(&sql)?;
        let activity_iter = stmt.query_map([member_id], |row| {
            self.row_to_activity(row).map_err(|e| match e {
                UveddiError::DatabaseError {
                    source: Some(sql_err),
                    ..
                } => sql_err,
                _ => rusqlite::Error::ToSqlConversionFailure(Box::new(e)),
            })
        })?;

        let mut activities = Vec::new();
        for activity in activity_iter {
            activities.push(activity?);
        }
        Ok(activities)
    }

    /// Helper method to convert database row to CommunityMember
    fn row_to_member(&self, row: &Row) -> Result<CommunityMember, UveddiError> {
        let languages_json: String = row.get(11).map_err(|e| {
            UveddiError::database_error("get languages_json", "community database", e)
        })?;
        let custom_fields_json: String = row.get(17).map_err(|e| {
            UveddiError::database_error("get custom_fields_json", "community database", e)
        })?;

        let languages = self.deserialize_languages(&languages_json).map_err(|e| {
            log::warn!("Failed to deserialize languages for member: {}", e);
            e
        })?;

        let custom_fields = self
            .deserialize_custom_fields(&custom_fields_json)
            .map_err(|e| {
                log::warn!("Failed to deserialize custom fields for member: {}", e);
                e
            })?;

        let last_active_str: Option<String> = row
            .get(5)
            .map_err(|e| UveddiError::database_error("get last_active", "community database", e))?;
        let last_active = last_active_str
            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        Ok(CommunityMember {
            id: row
                .get(0)
                .map_err(|e| UveddiError::database_error("get id", "community database", e))?,
            email: row
                .get(1)
                .map_err(|e| UveddiError::database_error("get email", "community database", e))?,
            name: row
                .get(2)
                .map_err(|e| UveddiError::database_error("get name", "community database", e))?,
            role: MemberRole::from_str(
                &row.get::<_, String>(3).map_err(|e| {
                    UveddiError::database_error("get role", "community database", e)
                })?,
            ),
            created_at: {
                let timestamp_str = row.get::<_, String>(4).map_err(|e| {
                    UveddiError::database_error("get created_at", "community database", e)
                })?;
                DateTime::parse_from_rfc3339(&timestamp_str)
                    .map_err(|_e| {
                        UveddiError::database_error(
                            "parse created_at timestamp",
                            "community database",
                            rusqlite::Error::InvalidColumnType(
                                4,
                                timestamp_str.clone(),
                                rusqlite::types::Type::Text,
                            ),
                        )
                    })?
                    .with_timezone(&Utc)
            },
            last_active,
            is_active: row.get(6).map_err(|e| {
                UveddiError::database_error("get is_active", "community database", e)
            })?,
            email_verified: row.get(7).map_err(|e| {
                UveddiError::database_error("get email_verified", "community database", e)
            })?,
            avatar_url: row.get(8).map_err(|e| {
                UveddiError::database_error("get avatar_url", "community database", e)
            })?,
            profile: MemberProfile {
                company: row.get(9).map_err(|e| {
                    UveddiError::database_error("get company", "community database", e)
                })?,
                job_title: row.get(10).map_err(|e| {
                    UveddiError::database_error("get job_title", "community database", e)
                })?,
                languages,
                experience_level: row.get(12).map_err(|e| {
                    UveddiError::database_error("get experience_level", "community database", e)
                })?,
                use_case: row.get(13).map_err(|e| {
                    UveddiError::database_error("get use_case", "community database", e)
                })?,
                referral_source: row.get(14).map_err(|e| {
                    UveddiError::database_error("get referral_source", "community database", e)
                })?,
                newsletter_subscribed: row.get(15).map_err(|e| {
                    UveddiError::database_error(
                        "get newsletter_subscribed",
                        "community database",
                        e,
                    )
                })?,
                marketing_consent: row.get(16).map_err(|e| {
                    UveddiError::database_error("get marketing_consent", "community database", e)
                })?,
                custom_fields,
            },
        })
    }

    /// Helper method to convert database row to MemberActivity
    fn row_to_activity(&self, row: &Row) -> Result<MemberActivity, UveddiError> {
        let metadata_json: String = row.get(7).map_err(|e| {
            UveddiError::database_error("get metadata_json", "community database", e)
        })?;

        let metadata = self.deserialize_metadata(&metadata_json).map_err(|e| {
            log::warn!("Failed to deserialize activity metadata: {}", e);
            e
        })?;

        Ok(MemberActivity {
            id: Some(row.get(0).map_err(|e| {
                UveddiError::database_error("get activity id", "community database", e)
            })?),
            member_id: row.get(1).map_err(|e| {
                UveddiError::database_error("get member_id", "community database", e)
            })?,
            activity_type: ActivityType::from_str(&row.get::<_, String>(2).map_err(|e| {
                UveddiError::database_error("get activity_type", "community database", e)
            })?),
            timestamp: {
                let timestamp_str = row.get::<_, String>(3).map_err(|e| {
                    UveddiError::database_error("get timestamp", "community database", e)
                })?;
                DateTime::parse_from_rfc3339(&timestamp_str)
                    .map_err(|_e| {
                        UveddiError::database_error(
                            "parse timestamp",
                            "community database",
                            rusqlite::Error::InvalidColumnType(
                                3,
                                timestamp_str.clone(),
                                rusqlite::types::Type::Text,
                            ),
                        )
                    })?
                    .with_timezone(&Utc)
            },
            description: row.get(4).map_err(|e| {
                UveddiError::database_error("get description", "community database", e)
            })?,
            ip_address: row.get(5).map_err(|e| {
                UveddiError::database_error("get ip_address", "community database", e)
            })?,
            user_agent: row.get(6).map_err(|e| {
                UveddiError::database_error("get user_agent", "community database", e)
            })?,
            metadata,
        })
    }

    // ================================
    // Admin Management Methods
    // ================================

    /// Create an admin profile for a member
    pub fn create_admin_profile(
        &self,
        member_id: &str,
        permissions: AdminPermissions,
        admin_level: AdminLevel,
        appointed_by: &str,
    ) -> Result<AdminProfile, UveddiError> {
        let admin_profile = AdminProfile {
            member_id: member_id.to_string(),
            permissions: permissions.clone(),
            admin_level: admin_level.clone(),
            assigned_regions: Vec::new(),
            assigned_teams: Vec::new(),
            admin_notes: None,
            appointed_by: Some(appointed_by.to_string()),
            appointed_at: Utc::now(),
            last_admin_action: None,
            is_super_admin: matches!(admin_level, AdminLevel::SuperAdmin),
        };

        let permissions_json = serde_json::to_string(&permissions).map_err(|e| {
            UveddiError::database_error(
                "serialization",
                "community database",
                rusqlite::Error::ToSqlConversionFailure(Box::new(e)),
            )
        })?;
        let regions_json = serde_json::to_string(&admin_profile.assigned_regions).map_err(|e| {
            UveddiError::database_error(
                "serialization",
                "community database",
                rusqlite::Error::ToSqlConversionFailure(Box::new(e)),
            )
        })?;
        let teams_json = serde_json::to_string(&admin_profile.assigned_teams).map_err(|e| {
            UveddiError::database_error(
                "serialization",
                "community database",
                rusqlite::Error::ToSqlConversionFailure(Box::new(e)),
            )
        })?;

        let level_str = match admin_level {
            AdminLevel::Moderator => "moderator",
            AdminLevel::Administrator => "administrator",
            AdminLevel::SuperAdmin => "super_admin",
        };

        self.conn.execute(
            r#"
            INSERT INTO admin_profiles (
                member_id, admin_level, permissions, assigned_regions, assigned_teams,
                appointed_by, appointed_at, is_super_admin
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            (
                member_id,
                level_str,
                &permissions_json,
                &regions_json,
                &teams_json,
                appointed_by,
                admin_profile.appointed_at.to_rfc3339(),
                admin_profile.is_super_admin,
            ),
        )?;

        Ok(admin_profile)
    }

    /// Get admin profile for a member
    pub fn get_admin_profile(&self, member_id: &str) -> Result<Option<AdminProfile>, UveddiError> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT member_id, admin_level, permissions, assigned_regions, assigned_teams,
                   admin_notes, appointed_by, appointed_at, last_admin_action, is_super_admin
            FROM admin_profiles WHERE member_id = ?1
            "#,
        )?;

        match stmt.query_row([member_id], |row| {
            let permissions_json: String = row.get(2)?;
            let regions_json: String = row.get(3)?;
            let teams_json: String = row.get(4)?;
            let appointed_at_str: String = row.get(7)?;
            let last_action_str: Option<String> = row.get(8)?;

            let permissions: AdminPermissions = serde_json::from_str(&permissions_json)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
            let assigned_regions: Vec<String> = serde_json::from_str(&regions_json)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
            let assigned_teams: Vec<String> = serde_json::from_str(&teams_json)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

            let admin_level = match row.get::<_, String>(1)?.as_str() {
                "moderator" => AdminLevel::Moderator,
                "administrator" => AdminLevel::Administrator,
                "super_admin" => AdminLevel::SuperAdmin,
                _ => AdminLevel::Moderator,
            };

            Ok(AdminProfile {
                member_id: row.get(0)?,
                admin_level,
                permissions,
                assigned_regions,
                assigned_teams,
                admin_notes: row.get(5)?,
                appointed_by: row.get(6)?,
                appointed_at: DateTime::parse_from_rfc3339(&appointed_at_str)
                    .unwrap()
                    .with_timezone(&Utc),
                last_admin_action: last_action_str.map(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .unwrap()
                        .with_timezone(&Utc)
                }),
                is_super_admin: row.get(9)?,
            })
        }) {
            Ok(profile) => Ok(Some(profile)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(UveddiError::database_error(
                "database operation",
                "community database",
                e,
            )),
        }
    }

    /// Update admin permissions
    pub fn update_admin_permissions(
        &self,
        member_id: &str,
        permissions: AdminPermissions,
    ) -> Result<(), UveddiError> {
        let permissions_json = serde_json::to_string(&permissions).map_err(|e| {
            UveddiError::database_error(
                "serialization",
                "community database",
                rusqlite::Error::ToSqlConversionFailure(Box::new(e)),
            )
        })?;

        self.conn.execute(
            "UPDATE admin_profiles SET permissions = ?1, last_admin_action = ?2 WHERE member_id = ?3",
            (&permissions_json, Utc::now().to_rfc3339(), member_id),
        )?;

        Ok(())
    }

    /// Get all admins
    pub fn list_admins(&self) -> Result<Vec<(CommunityMember, AdminProfile)>, UveddiError> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT cm.id, cm.email, cm.name, cm.role, cm.created_at, cm.last_active, cm.is_active,
                   ap.admin_level, ap.permissions, ap.assigned_regions, ap.assigned_teams,
                   ap.admin_notes, ap.appointed_by, ap.appointed_at, ap.last_admin_action, ap.is_super_admin
            FROM community_members cm
            JOIN admin_profiles ap ON cm.id = ap.member_id
            WHERE cm.role = 'admin' AND cm.is_active = 1
            ORDER BY ap.appointed_at DESC
            "#,
        )?;

        let results = stmt.query_map([], |row| {
            // Build member
            let member = CommunityMember {
                id: row.get(0)?,
                email: row.get(1)?,
                name: row.get(2)?,
                role: MemberRole::from_str(&row.get::<_, String>(3)?),
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                    .unwrap()
                    .with_timezone(&Utc),
                last_active: row.get::<_, Option<String>>(5)?.map(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .unwrap()
                        .with_timezone(&Utc)
                }),
                is_active: row.get(6)?,
                profile: MemberProfile::default(), // We'll fetch this separately if needed
                email_verified: false,
                avatar_url: None,
            };

            // Build admin profile
            let permissions_json: String = row.get(8)?;
            let regions_json: String = row.get(9)?;
            let teams_json: String = row.get(10)?;

            let permissions: AdminPermissions = serde_json::from_str(&permissions_json)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
            let assigned_regions: Vec<String> = serde_json::from_str(&regions_json)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
            let assigned_teams: Vec<String> = serde_json::from_str(&teams_json)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

            let admin_level = match row.get::<_, String>(7)?.as_str() {
                "moderator" => AdminLevel::Moderator,
                "administrator" => AdminLevel::Administrator,
                "super_admin" => AdminLevel::SuperAdmin,
                _ => AdminLevel::Moderator,
            };

            let admin_profile = AdminProfile {
                member_id: row.get(0)?,
                admin_level,
                permissions,
                assigned_regions,
                assigned_teams,
                admin_notes: row.get(11)?,
                appointed_by: row.get(12)?,
                appointed_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(13)?)
                    .unwrap()
                    .with_timezone(&Utc),
                last_admin_action: row.get::<_, Option<String>>(14)?.map(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .unwrap()
                        .with_timezone(&Utc)
                }),
                is_super_admin: row.get(15)?,
            };

            Ok((member, admin_profile))
        })?;

        let mut admins = Vec::new();
        for result in results {
            admins.push(result?);
        }

        Ok(admins)
    }

    // ================================
    // Developer Management Methods
    // ================================

    /// Create a developer profile for a member
    pub fn create_developer_profile(
        &self,
        member_id: &str,
        developer_type: DeveloperType,
        specializations: Vec<String>,
    ) -> Result<DeveloperProfile, UveddiError> {
        let dev_profile = DeveloperProfile {
            member_id: member_id.to_string(),
            developer_type: developer_type.clone(),
            specializations: specializations.clone(),
            github_verified: false,
            contribution_score: 0,
            api_access_level: ApiAccessLevel::Basic,
            repositories: Vec::new(),
            badges: Vec::new(),
            verified_at: None,
            verification_method: None,
            mentor_status: false,
            beta_tester: false,
        };

        let specializations_json = serde_json::to_string(&specializations).map_err(|e| {
            UveddiError::database_error(
                "serialization",
                "community database",
                rusqlite::Error::ToSqlConversionFailure(Box::new(e)),
            )
        })?;
        let repositories_json = serde_json::to_string(&dev_profile.repositories).map_err(|e| {
            UveddiError::database_error(
                "serialization",
                "community database",
                rusqlite::Error::ToSqlConversionFailure(Box::new(e)),
            )
        })?;

        let dev_type_str = match developer_type {
            DeveloperType::OpenSource => "opensource",
            DeveloperType::Enterprise => "enterprise",
            DeveloperType::Student => "student",
            DeveloperType::Freelancer => "freelancer",
            DeveloperType::Startup => "startup",
            DeveloperType::Corporate => "corporate",
        };

        self.conn.execute(
            r#"
            INSERT INTO developer_profiles (
                member_id, developer_type, specializations, github_verified, contribution_score,
                api_access_level, repositories, mentor_status, beta_tester
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            (
                member_id,
                dev_type_str,
                &specializations_json,
                dev_profile.github_verified,
                dev_profile.contribution_score,
                "basic",
                &repositories_json,
                dev_profile.mentor_status,
                dev_profile.beta_tester,
            ),
        )?;

        Ok(dev_profile)
    }

    /// Get developer profile for a member
    pub fn get_developer_profile(
        &self,
        member_id: &str,
    ) -> Result<Option<DeveloperProfile>, UveddiError> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT member_id, developer_type, specializations, github_verified, contribution_score,
                   api_access_level, repositories, verified_at, verification_method, 
                   mentor_status, beta_tester
            FROM developer_profiles WHERE member_id = ?1
            "#,
        )?;

        match stmt.query_row([member_id], |row| {
            let specializations_json: String = row.get(2)?;
            let repositories_json: String = row.get(6)?;

            let specializations: Vec<String> = serde_json::from_str(&specializations_json)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
            let repositories: Vec<String> = serde_json::from_str(&repositories_json)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

            let developer_type = match row.get::<_, String>(1)?.as_str() {
                "opensource" => DeveloperType::OpenSource,
                "enterprise" => DeveloperType::Enterprise,
                "student" => DeveloperType::Student,
                "freelancer" => DeveloperType::Freelancer,
                "startup" => DeveloperType::Startup,
                "corporate" => DeveloperType::Corporate,
                _ => DeveloperType::OpenSource,
            };

            let api_access_level = match row.get::<_, String>(5)?.as_str() {
                "basic" => ApiAccessLevel::Basic,
                "premium" => ApiAccessLevel::Premium,
                "enterprise" => ApiAccessLevel::Enterprise,
                _ => ApiAccessLevel::Basic,
            };

            Ok(DeveloperProfile {
                member_id: row.get(0)?,
                developer_type,
                specializations,
                github_verified: row.get(3)?,
                contribution_score: row.get(4)?,
                api_access_level,
                repositories,
                badges: Vec::new(), // We'll fetch these separately
                verified_at: row.get::<_, Option<String>>(7)?.map(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .unwrap()
                        .with_timezone(&Utc)
                }),
                verification_method: row.get(8)?,
                mentor_status: row.get(9)?,
                beta_tester: row.get(10)?,
            })
        }) {
            Ok(profile) => Ok(Some(profile)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(UveddiError::database_error(
                "database operation",
                "community database",
                e,
            )),
        }
    }

    /// Award a badge to a developer
    pub fn award_developer_badge(
        &self,
        member_id: &str,
        badge_type: BadgeType,
        description: &str,
        metadata: Option<&str>,
    ) -> Result<DeveloperBadge, UveddiError> {
        let badge = DeveloperBadge {
            badge_type: badge_type.clone(),
            earned_at: Utc::now(),
            description: description.to_string(),
            metadata: metadata.map(|s| s.to_string()),
        };

        let badge_type_str = match badge_type {
            BadgeType::EarlyAdopter => "early_adopter",
            BadgeType::Contributor => "contributor",
            BadgeType::Mentor => "mentor",
            BadgeType::BugFinder => "bug_finder",
            BadgeType::FeatureRequester => "feature_requester",
            BadgeType::BetaTester => "beta_tester",
            BadgeType::CommunityHelper => "community_helper",
            BadgeType::CodeReviewer => "code_reviewer",
            BadgeType::Documentation => "documentation",
            BadgeType::Custom(ref name) => name,
        };

        self.conn.execute(
            r#"
            INSERT INTO developer_badges (member_id, badge_type, earned_at, description, metadata)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            (
                member_id,
                badge_type_str,
                badge.earned_at.to_rfc3339(),
                &badge.description,
                &badge.metadata,
            ),
        )?;

        Ok(badge)
    }

    /// Get role permissions
    pub fn get_role_permissions(
        &self,
        role: &MemberRole,
    ) -> Result<Option<RolePermissions>, UveddiError> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT role, can_access_api, can_view_source, can_download_reports,
                   can_create_projects, can_share_publicly, can_invite_members,
                   rate_limit_tier, storage_quota_mb, features
            FROM role_permissions WHERE role = ?1
            "#,
        )?;

        match stmt.query_row([role.as_str()], |row| {
            let features_json: String = row.get(9)?;
            let features: Vec<String> = serde_json::from_str(&features_json)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

            Ok(RolePermissions {
                role: MemberRole::from_str(&row.get::<_, String>(0)?),
                can_access_api: row.get(1)?,
                can_view_source: row.get(2)?,
                can_download_reports: row.get(3)?,
                can_create_projects: row.get(4)?,
                can_share_publicly: row.get(5)?,
                can_invite_members: row.get(6)?,
                rate_limit_tier: row.get(7)?,
                storage_quota_mb: row.get(8)?,
                features,
            })
        }) {
            Ok(permissions) => Ok(Some(permissions)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(UveddiError::database_error(
                "database operation",
                "community database",
                e,
            )),
        }
    }

    // ================================
    // Secure JSON Deserialization Methods
    // ================================

    /// Maximum JSON payload size (1MB)
    const MAX_JSON_SIZE: usize = 1024 * 1024;

    /// Maximum array length for security
    const MAX_ARRAY_LENGTH: usize = 1000;

    /// Securely deserialize languages array with validation
    pub(crate) fn deserialize_languages(&self, json_str: &str) -> Result<Vec<String>, UveddiError> {
        // Size validation
        if json_str.len() > Self::MAX_JSON_SIZE {
            return Err(UveddiError::Deserialization {
                message: format!("JSON payload too large: {} bytes", json_str.len()),
                context: "languages deserialization".to_string(),
                suggestion: "Reduce payload size or increase limits".to_string(),
                source: Some(DeserializationError::PayloadTooLarge {
                    size: json_str.len(),
                    max_size: Self::MAX_JSON_SIZE,
                }),
            });
        }

        // Parse with proper error handling
        let languages: Vec<String> =
            serde_json::from_str(json_str).map_err(|e| UveddiError::Deserialization {
                message: format!("Invalid JSON format: {}", e),
                context: "languages deserialization".to_string(),
                suggestion: "Check JSON syntax and structure".to_string(),
                source: Some(DeserializationError::InvalidFormat(e.to_string())),
            })?;

        // Validate array length
        if languages.len() > Self::MAX_ARRAY_LENGTH {
            return Err(UveddiError::Deserialization {
                message: format!("Languages array too large: {} items", languages.len()),
                context: "languages deserialization".to_string(),
                suggestion: "Reduce number of languages or increase limits".to_string(),
                source: Some(DeserializationError::SecurityValidation(format!(
                    "Languages array too large: {} items",
                    languages.len()
                ))),
            });
        }

        // Validate each language string
        for lang in &languages {
            if lang.len() > 50
                || !lang
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
            {
                return Err(UveddiError::Deserialization {
                    message: format!("Invalid language format: {}", lang),
                    context: "languages deserialization".to_string(),
                    suggestion:
                        "Use valid language codes (alphanumeric, hyphens, underscores only)"
                            .to_string(),
                    source: Some(DeserializationError::SecurityValidation(format!(
                        "Invalid language format: {}",
                        lang
                    ))),
                });
            }
        }

        Ok(languages)
    }

    /// Securely deserialize custom fields with validation
    pub(crate) fn deserialize_custom_fields(
        &self,
        json_str: &str,
    ) -> Result<HashMap<String, String>, UveddiError> {
        // Size validation
        if json_str.len() > Self::MAX_JSON_SIZE {
            return Err(UveddiError::Deserialization {
                message: format!("JSON payload too large: {} bytes", json_str.len()),
                context: "custom fields deserialization".to_string(),
                suggestion: "Reduce payload size or increase limits".to_string(),
                source: Some(DeserializationError::PayloadTooLarge {
                    size: json_str.len(),
                    max_size: Self::MAX_JSON_SIZE,
                }),
            });
        }

        // Parse with proper error handling
        let custom_fields: HashMap<String, String> =
            serde_json::from_str(json_str).map_err(|e| UveddiError::Deserialization {
                message: format!("Invalid JSON format: {}", e),
                context: "custom fields deserialization".to_string(),
                suggestion: "Check JSON syntax and structure".to_string(),
                source: Some(DeserializationError::InvalidFormat(e.to_string())),
            })?;

        // Validate field count
        if custom_fields.len() > 50 {
            return Err(UveddiError::Deserialization {
                message: format!("Too many custom fields: {}", custom_fields.len()),
                context: "custom fields deserialization".to_string(),
                suggestion: "Reduce number of custom fields".to_string(),
                source: Some(DeserializationError::SecurityValidation(format!(
                    "Too many custom fields: {}",
                    custom_fields.len()
                ))),
            });
        }

        // Validate keys and values
        for (key, value) in &custom_fields {
            if key.len() > 100 || value.len() > 1000 {
                return Err(UveddiError::Deserialization {
                    message: "Custom field key or value too large".to_string(),
                    context: "custom fields deserialization".to_string(),
                    suggestion: "Reduce field key/value length (max 100/1000 chars)".to_string(),
                    source: Some(DeserializationError::SecurityValidation(
                        "Custom field key or value too large".to_string(),
                    )),
                });
            }
        }

        Ok(custom_fields)
    }

    /// Securely deserialize metadata with validation
    pub(crate) fn deserialize_metadata(
        &self,
        json_str: &str,
    ) -> Result<HashMap<String, String>, UveddiError> {
        // Reuse custom_fields validation logic
        self.deserialize_custom_fields(json_str)
    }

    /// Generate community analytics
    pub fn generate_analytics(
        &self,
    ) -> Result<crate::community::analytics::MemberAnalytics, UveddiError> {
        let analytics_engine = crate::community::analytics::AnalyticsEngine::new(&self.conn);
        analytics_engine.generate_analytics()
    }

    /// Create an analytics engine for this database
    pub fn analytics_engine(&self) -> crate::community::analytics::AnalyticsEngine {
        crate::community::analytics::AnalyticsEngine::new(&self.conn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_register_and_get_member() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = CommunityDatabase::new(temp_file.path()).unwrap();

        let member = db
            .register_member("test@example.com", "Test User", MemberRole::Member)
            .unwrap();
        assert_eq!(member.email, "test@example.com");
        assert_eq!(member.name, "Test User");
        assert_eq!(member.role, MemberRole::Member);
        assert!(member.is_active);

        let retrieved = db.get_member_by_email("test@example.com").unwrap();
        assert!(retrieved.is_some());
        let retrieved_member = retrieved.unwrap();
        assert_eq!(retrieved_member.email, "test@example.com");
        assert_eq!(retrieved_member.id, member.id);
    }

    #[test]
    fn test_update_member_role() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = CommunityDatabase::new(temp_file.path()).unwrap();

        let member = db
            .register_member("test@example.com", "Test User", MemberRole::Member)
            .unwrap();

        let updated = db
            .update_member_role(&member.id, MemberRole::Developer)
            .unwrap();
        assert!(updated);

        let retrieved = db.get_member_by_id(&member.id).unwrap().unwrap();
        assert_eq!(retrieved.role, MemberRole::Developer);
    }

    #[test]
    fn test_activity_logging() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = CommunityDatabase::new(temp_file.path()).unwrap();

        let member = db
            .register_member("test@example.com", "Test User", MemberRole::Member)
            .unwrap();

        // Log some activities
        db.log_activity(&member.id, ActivityType::Login, None, None, None)
            .unwrap();
        db.log_activity(
            &member.id,
            ActivityType::DashboardView,
            Some("Viewed main dashboard"),
            None,
            None,
        )
        .unwrap();

        let activities = db.get_member_activity(&member.id, Some(10)).unwrap();

        // Should have registration + login + dashboard view = 3 activities
        assert_eq!(activities.len(), 3);
        assert!(activities
            .iter()
            .any(|a| matches!(a.activity_type, ActivityType::Login)));
        assert!(activities
            .iter()
            .any(|a| matches!(a.activity_type, ActivityType::Registration)));
    }

    #[test]
    fn test_list_members_with_filters() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = CommunityDatabase::new(temp_file.path()).unwrap();

        db.register_member("user1@example.com", "User 1", MemberRole::Member)
            .unwrap();
        db.register_member("user2@example.com", "User 2", MemberRole::Developer)
            .unwrap();
        db.register_member("admin@example.com", "Admin", MemberRole::Admin)
            .unwrap();

        // Test role filtering
        let developers = db
            .list_members(Some(MemberRole::Developer), true, None, None)
            .unwrap();
        assert_eq!(developers.len(), 1);
        assert_eq!(developers[0].role, MemberRole::Developer);

        // Test listing all active members
        let all_active = db.list_members(None, true, None, None).unwrap();
        assert_eq!(all_active.len(), 3);

        // Test with limit
        let limited = db.list_members(None, true, Some(2), None).unwrap();
        assert_eq!(limited.len(), 2);
    }

    // ================================
    // Security Tests (UV-275)
    // ================================

    #[test]
    fn test_malicious_json_payloads() {
        let db = CommunityDatabase::new_in_memory().unwrap();

        // Test oversized payload (simulate large JSON)
        let large_languages: Vec<String> = (0..100000).map(|i| format!("lang{}", i)).collect();
        let large_json = serde_json::to_string(&large_languages).unwrap();
        let result = db.deserialize_languages(&large_json);
        assert!(result.is_err());
        if let Err(UveddiError::Deserialization {
            source: Some(DeserializationError::PayloadTooLarge { .. }),
            ..
        }) = result
        {
            // Expected error type
        } else {
            panic!("Expected PayloadTooLarge error for oversized payload");
        }

        // Test malformed JSON
        let malformed = "{ invalid json }";
        let result = db.deserialize_custom_fields(malformed);
        assert!(result.is_err());
        if let Err(UveddiError::Deserialization {
            source: Some(DeserializationError::InvalidFormat(_)),
            ..
        }) = result
        {
            // Expected error type
        } else {
            panic!("Expected InvalidFormat error for malformed JSON");
        }
    }

    #[test]
    fn test_size_limits() {
        let db = CommunityDatabase::new_in_memory().unwrap();

        // Test array length limits for languages
        let too_many_languages: Vec<String> = (0..2000).map(|i| format!("lang{}", i)).collect();
        let json = serde_json::to_string(&too_many_languages).unwrap();
        let result = db.deserialize_languages(&json);
        assert!(result.is_err());
        if let Err(UveddiError::Deserialization {
            source: Some(DeserializationError::SecurityValidation(_)),
            ..
        }) = result
        {
            // Expected error type
        } else {
            panic!("Expected SecurityValidation error for too many languages");
        }

        // Test field count limits for custom fields
        let too_many_fields: HashMap<String, String> = (0..100)
            .map(|i| (format!("key{}", i), format!("value{}", i)))
            .collect();
        let json = serde_json::to_string(&too_many_fields).unwrap();
        let result = db.deserialize_custom_fields(&json);
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_data_acceptance() {
        let db = CommunityDatabase::new_in_memory().unwrap();

        // Test valid languages
        let valid_languages = r#"["rust", "python", "javascript"]"#;
        let result = db.deserialize_languages(valid_languages);
        assert!(result.is_ok());
        let languages = result.unwrap();
        assert_eq!(languages.len(), 3);
        assert!(languages.contains(&"rust".to_string()));

        // Test valid custom fields
        let valid_fields = r#"{"company": "TechCorp", "role": "developer"}"#;
        let result = db.deserialize_custom_fields(valid_fields);
        assert!(result.is_ok());
        let fields = result.unwrap();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields.get("company"), Some(&"TechCorp".to_string()));
    }

    #[test]
    fn test_language_validation() {
        let db = CommunityDatabase::new_in_memory().unwrap();

        // Test valid language codes
        let valid_languages = r#"["rust", "python3", "c-sharp", "objective_c"]"#;
        let result = db.deserialize_languages(valid_languages);
        assert!(result.is_ok());

        // Test invalid language codes with special characters
        let invalid_languages = r#"["rust", "python/3", "c#"]"#;
        let result = db.deserialize_languages(invalid_languages);
        assert!(result.is_err()); // Should fail due to invalid characters

        // Test language codes that are too long
        let long_language = "x".repeat(100);
        let long_languages = format!(r#"["rust", "{}"]"#, long_language);
        let result = db.deserialize_languages(&long_languages);
        assert!(result.is_err()); // Should fail due to length limit
    }
}
