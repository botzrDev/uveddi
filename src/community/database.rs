//! Community Database Implementation
//!
//! SQLite-based database for community member management, completely separate from
//! the main analysis database. Optimized for cloud deployment and admin management.

use crate::community::models::{
    CommunityMember, MemberActivity, MemberProfile, MemberRole, MemberSession, ActivityType
};
use crate::error::UveddiError;
use chrono::{DateTime, Utc, Duration};
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

            -- Indexes for performance
            CREATE INDEX IF NOT EXISTS idx_member_email ON community_members(email);
            CREATE INDEX IF NOT EXISTS idx_member_role ON community_members(role);
            CREATE INDEX IF NOT EXISTS idx_member_active ON community_members(is_active);
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

        let languages_json = serde_json::to_string(&member.profile.languages)
            .map_err(|e| UveddiError::Database(rusqlite::Error::ToSqlConversionFailure(Box::new(e))))?;
        let custom_fields_json = serde_json::to_string(&member.profile.custom_fields)
            .map_err(|e| UveddiError::Database(rusqlite::Error::ToSqlConversionFailure(Box::new(e))))?;

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
            "#
        )?;

        let member_iter = stmt.query_map([email], |row| {
            self.row_to_member(row)
        })?;

        for member in member_iter {
            return Ok(Some(member?));
        }
        Ok(None)
    }

    /// Get member by ID
    pub fn get_member_by_id(&self, id: &str) -> Result<Option<CommunityMember>, UveddiError> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, email, name, role, created_at, last_active, is_active, email_verified,
                   avatar_url, company, job_title, languages, experience_level, use_case,
                   referral_source, newsletter_subscribed, marketing_consent, custom_fields
            FROM community_members WHERE id = ?1
            "#
        )?;

        let member_iter = stmt.query_map([id], |row| {
            self.row_to_member(row)
        })?;

        for member in member_iter {
            return Ok(Some(member?));
        }
        Ok(None)
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
        "#.to_string();

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
            self.row_to_member(row)
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
        let languages_json = serde_json::to_string(&profile.languages)
            .map_err(|e| UveddiError::Database(rusqlite::Error::ToSqlConversionFailure(Box::new(e))))?;
        let custom_fields_json = serde_json::to_string(&profile.custom_fields)
            .map_err(|e| UveddiError::Database(rusqlite::Error::ToSqlConversionFailure(Box::new(e))))?;

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
    pub fn update_member_role(&self, member_id: &str, new_role: MemberRole) -> Result<bool, UveddiError> {
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
            self.log_activity(member_id, ActivityType::AccountDeactivated, None, None, None)?;
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
        let metadata_json = serde_json::to_string(&metadata)
            .map_err(|e| UveddiError::Database(rusqlite::Error::ToSqlConversionFailure(Box::new(e))))?;

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
            "#.to_string()
        };

        let mut stmt = self.conn.prepare(&sql)?;
        let activity_iter = stmt.query_map([member_id], |row| {
            self.row_to_activity(row)
        })?;

        let mut activities = Vec::new();
        for activity in activity_iter {
            activities.push(activity?);
        }
        Ok(activities)
    }

    /// Helper method to convert database row to CommunityMember
    fn row_to_member(&self, row: &Row) -> SqlResult<CommunityMember> {
        let languages_json: String = row.get(11)?;
        let custom_fields_json: String = row.get(17)?;
        
        let languages: Vec<String> = serde_json::from_str(&languages_json)
            .unwrap_or_default();
        let custom_fields: HashMap<String, String> = serde_json::from_str(&custom_fields_json)
            .unwrap_or_default();

        let last_active_str: Option<String> = row.get(5)?;
        let last_active = last_active_str
            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        Ok(CommunityMember {
            id: row.get(0)?,
            email: row.get(1)?,
            name: row.get(2)?,
            role: MemberRole::from_str(&row.get::<_, String>(3)?),
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                .unwrap()
                .with_timezone(&Utc),
            last_active,
            is_active: row.get(6)?,
            email_verified: row.get(7)?,
            avatar_url: row.get(8)?,
            profile: MemberProfile {
                company: row.get(9)?,
                job_title: row.get(10)?,
                languages,
                experience_level: row.get(12)?,
                use_case: row.get(13)?,
                referral_source: row.get(14)?,
                newsletter_subscribed: row.get(15)?,
                marketing_consent: row.get(16)?,
                custom_fields,
            },
        })
    }

    /// Helper method to convert database row to MemberActivity
    fn row_to_activity(&self, row: &Row) -> SqlResult<MemberActivity> {
        let metadata_json: String = row.get(7)?;
        let metadata: HashMap<String, String> = serde_json::from_str(&metadata_json)
            .unwrap_or_default();

        Ok(MemberActivity {
            id: Some(row.get(0)?),
            member_id: row.get(1)?,
            activity_type: ActivityType::from_str(&row.get::<_, String>(2)?),
            timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                .unwrap()
                .with_timezone(&Utc),
            description: row.get(4)?,
            ip_address: row.get(5)?,
            user_agent: row.get(6)?,
            metadata,
        })
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

        let member = db.register_member("test@example.com", "Test User", MemberRole::Member).unwrap();
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

        let member = db.register_member("test@example.com", "Test User", MemberRole::Member).unwrap();
        
        let updated = db.update_member_role(&member.id, MemberRole::Developer).unwrap();
        assert!(updated);

        let retrieved = db.get_member_by_id(&member.id).unwrap().unwrap();
        assert_eq!(retrieved.role, MemberRole::Developer);
    }

    #[test]
    fn test_activity_logging() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = CommunityDatabase::new(temp_file.path()).unwrap();

        let member = db.register_member("test@example.com", "Test User", MemberRole::Member).unwrap();
        
        // Log some activities
        db.log_activity(&member.id, ActivityType::Login, None, None, None).unwrap();
        db.log_activity(&member.id, ActivityType::DashboardView, Some("Viewed main dashboard"), None, None).unwrap();

        let activities = db.get_member_activity(&member.id, Some(10)).unwrap();
        
        // Should have registration + login + dashboard view = 3 activities
        assert_eq!(activities.len(), 3);
        assert!(activities.iter().any(|a| matches!(a.activity_type, ActivityType::Login)));
        assert!(activities.iter().any(|a| matches!(a.activity_type, ActivityType::Registration)));
    }

    #[test]
    fn test_list_members_with_filters() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = CommunityDatabase::new(temp_file.path()).unwrap();

        db.register_member("user1@example.com", "User 1", MemberRole::Member).unwrap();
        db.register_member("user2@example.com", "User 2", MemberRole::Developer).unwrap();
        db.register_member("admin@example.com", "Admin", MemberRole::Admin).unwrap();

        // Test role filtering
        let developers = db.list_members(Some(MemberRole::Developer), true, None, None).unwrap();
        assert_eq!(developers.len(), 1);
        assert_eq!(developers[0].role, MemberRole::Developer);

        // Test listing all active members
        let all_active = db.list_members(None, true, None, None).unwrap();
        assert_eq!(all_active.len(), 3);

        // Test with limit
        let limited = db.list_members(None, true, Some(2), None).unwrap();
        assert_eq!(limited.len(), 2);
    }
}
