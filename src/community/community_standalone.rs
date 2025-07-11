// Standalone Community Management Demo
// This is a self-contained demo showing the community management system
// It includes all necessary code without depending on the main library

use rusqlite::{Connection, Result, params};
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use std::path::Path;

/// Placeholder documentation for public items

// Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemberRole {
    Member,
    Admin,
    Developer,
}

impl MemberRole {
    pub fn to_string(&self) -> String {
        match self {
            MemberRole::Member => "member".to_string(),
            MemberRole::Admin => "admin".to_string(),
            MemberRole::Developer => "developer".to_string(),
        }
    }

    pub fn from_string(s: &str) -> Result<Self, Box<dyn std::error::Error>> {
        match s {
            "member" => Ok(MemberRole::Member),
            "admin" => Ok(MemberRole::Admin),
            "developer" => Ok(MemberRole::Developer),
            _ => Err(format!("Invalid role: {}", s).into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommunityMember {
    pub id: Option<i64>,
    pub email: String,
    pub username: String,
    pub role: MemberRole,
    pub display_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_active: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub enum ActivityType {
    Login,
    ProfileUpdate,
    ContentView,
    ContentShare,
    Comment,
    Like,
    Download,
    Upload,
    Custom(String),
}

impl ActivityType {
    pub fn to_string(&self) -> String {
        match self {
            ActivityType::Login => "login".to_string(),
            ActivityType::ProfileUpdate => "profile_update".to_string(),
            ActivityType::ContentView => "content_view".to_string(),
            ActivityType::ContentShare => "content_share".to_string(),
            ActivityType::Comment => "comment".to_string(),
            ActivityType::Like => "like".to_string(),
            ActivityType::Download => "download".to_string(),
            ActivityType::Upload => "upload".to_string(),
            ActivityType::Custom(s) => s.clone(),
        }
    }

    pub fn from_string(s: &str) -> Result<Self, Box<dyn std::error::Error>> {
        match s {
            "login" => Ok(ActivityType::Login),
            "profile_update" => Ok(ActivityType::ProfileUpdate),
            "content_view" => Ok(ActivityType::ContentView),
            "content_share" => Ok(ActivityType::ContentShare),
            "comment" => Ok(ActivityType::Comment),
            "like" => Ok(ActivityType::Like),
            "download" => Ok(ActivityType::Download),
            "upload" => Ok(ActivityType::Upload),
            custom => Ok(ActivityType::Custom(custom.to_string())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemberActivity {
    pub id: Option<i64>,
    pub member_id: i64,
    pub activity_type: ActivityType,
    pub description: Option<String>,
    pub metadata: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct MemberProfile {
    pub member_id: i64,
    pub bio: Option<String>,
    pub website: Option<String>,
    pub location: Option<String>,
    pub company: Option<String>,
    pub skills: Option<String>,
    pub interests: Option<String>,
    pub github_username: Option<String>,
    pub twitter_username: Option<String>,
    pub linkedin_url: Option<String>,
    pub avatar_url: Option<String>,
    pub preferences: Option<String>,
    pub updated_at: DateTime<Utc>,
}

/// Placeholder documentation for public items

// Database
pub struct CommunityDatabase {
    connection: Connection,
}

impl CommunityDatabase {
    /// Creates a new instance of `CommunityDatabase`
    /// # Arguments
    ///
    /// * `db_path` - A reference to a `Path` object representing the database file path
    ///
    /// # Returns
    ///
    /// * `Result<Self, Box<dyn std::error::Error>>` - Returns `Ok` with the `CommunityDatabase` instance on success, or an error on failure
    pub fn new(db_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let connection = Connection::open(db_path)?;
        let db = CommunityDatabase { connection };
        db.init_schema()?;
        Ok(db)
    }

    /// Initializes the database schema by creating the necessary tables and indexes
    /// # Returns
    ///
    /// * `Result<(), Box<dyn std::error::Error>>` - Returns `Ok(())` on success, or an error on failure
    fn init_schema(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Members table
        self.connection.execute(
            "CREATE TABLE IF NOT EXISTS community_members (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                email TEXT UNIQUE NOT NULL,
                username TEXT UNIQUE NOT NULL,
                role TEXT NOT NULL DEFAULT 'member',
                display_name TEXT,
                created_at TEXT NOT NULL,
                last_active TEXT,
                is_active BOOLEAN NOT NULL DEFAULT 1
            )",
            [],
        )?;

        // Member profiles table
        self.connection.execute(
            "CREATE TABLE IF NOT EXISTS member_profiles (
                member_id INTEGER PRIMARY KEY,
                bio TEXT,
                website TEXT,
                location TEXT,
                company TEXT,
                skills TEXT,
                interests TEXT,
                github_username TEXT,
                twitter_username TEXT,
                linkedin_url TEXT,
                avatar_url TEXT,
                preferences TEXT,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (member_id) REFERENCES community_members (id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Member activities table
        self.connection.execute(
            "CREATE TABLE IF NOT EXISTS member_activities (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                member_id INTEGER NOT NULL,
                activity_type TEXT NOT NULL,
                description TEXT,
                metadata TEXT,
                timestamp TEXT NOT NULL,
                FOREIGN KEY (member_id) REFERENCES community_members (id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Create indexes for better performance
        self.connection.execute(
            "CREATE INDEX IF NOT EXISTS idx_members_email ON community_members (email)",
            [],
        )?;

        self.connection.execute(
            "CREATE INDEX IF NOT EXISTS idx_members_username ON community_members (username)",
            [],
        )?;

        self.connection.execute(
            "CREATE INDEX IF NOT EXISTS idx_activities_member_id ON member_activities (member_id)",
            [],
        )?;

        self.connection.execute(
            "CREATE INDEX IF NOT EXISTS idx_activities_timestamp ON member_activities (timestamp)",
            [],
        )?;

        Ok(())
    }

    /// Registers a new community member in the database
    /// # Arguments
    ///
    /// * `member` - A reference to a `CommunityMember` object containing the member's information
    ///
    /// # Returns
    ///
    /// * `Result<i64, Box<dyn std::error::Error>>` - Returns `Ok` with the new member's ID on success, or an error on failure
    pub fn register_member(&self, member: &CommunityMember) -> Result<i64, Box<dyn std::error::Error>> {
        let mut stmt = self.connection.prepare(
            "INSERT INTO community_members (email, username, role, display_name, created_at, last_active, is_active)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
        )?;

        let id = stmt.insert(params![
            member.email,
            member.username,
            member.role.to_string(),
            member.display_name,
            member.created_at.to_rfc3339(),
            member.last_active.map(|dt| dt.to_rfc3339()),
            member.is_active
        ])?;

        Ok(id)
    }

    /// Retrieves a community member by their email address
    /// # Arguments
    ///
    /// * `email` - A string slice representing the member's email address
    ///
    /// # Returns
    ///
    /// * `Result<Option<CommunityMember>, Box<dyn std::error::Error>>` - Returns `Ok` with an `Option<CommunityMember>` containing the member's information if found, or `None` if not found
    pub fn get_member_by_email(&self, email: &str) -> Result<Option<CommunityMember>, Box<dyn std::error::Error>> {
        let mut stmt = self.connection.prepare(
            "SELECT id, email, username, role, display_name, created_at, last_active, is_active
             FROM community_members WHERE email = ?1"
        )?;

        let member_iter = stmt.query_map([email], |row| {
            Ok(CommunityMember {
                id: Some(row.get(0)?),
                email: row.get(1)?,
                username: row.get(2)?,
                role: MemberRole::from_string(&row.get::<_, String>(3)?).unwrap(),
                display_name: row.get(4)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                    .unwrap().with_timezone(&Utc),
                last_active: row.get::<_, Option<String>>(6)?
                    .map(|s| DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Utc)),
                is_active: row.get(7)?,
            })
        })?;

        for member in member_iter {
            return Ok(Some(member?));
        }

        Ok(None)
    }

    /// Updates a community member's role
    /// # Arguments
    ///
    /// * `member_id` - The ID of the member whose role is to be updated
    /// * `new_role` - A reference to a `MemberRole` enum representing the new role
    ///
    /// # Returns
    ///
    /// * `Result<(), Box<dyn std::error::Error>>` - Returns `Ok(())` on success, or an error on failure
    pub fn update_member_role(&self, member_id: i64, new_role: &MemberRole) -> Result<(), Box<dyn std::error::Error>> {
        self.connection.execute(
            "UPDATE community_members SET role = ?1 WHERE id = ?2",
            params![new_role.to_string(), member_id],
        )?;
        Ok(())
    }

    /// Logs a member activity in the database
    /// # Arguments
    ///
    /// * `activity` - A reference to a `MemberActivity` object containing the activity's information
    ///
    /// # Returns
    ///
    /// * `Result<i64, Box<dyn std::error::Error>>` - Returns `Ok` with the new activity's ID on success, or an error on failure
    pub fn log_activity(&self, activity: &MemberActivity) -> Result<i64, Box<dyn std::error::Error>> {
        let mut stmt = self.connection.prepare(
            "INSERT INTO member_activities (member_id, activity_type, description, metadata, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5)"
        )?;

        let id = stmt.insert(params![
            activity.member_id,
            activity.activity_type.to_string(),
            activity.description,
            activity.metadata,
            activity.timestamp.to_rfc3339()
        ])?;

        // Update last_active timestamp for the member
        self.connection.execute(
            "UPDATE community_members SET last_active = ?1 WHERE id = ?2",
            params![activity.timestamp.to_rfc3339(), activity.member_id],
        )?;

        Ok(id)
    }

    /// Updates a member's profile information
    /// # Arguments
    ///
    /// * `profile` - A reference to a `MemberProfile` object containing the profile's information
    ///
    /// # Returns
    ///
    /// * `Result<(), Box<dyn std::error::Error>>` - Returns `Ok(())` on success, or an error on failure
    pub fn update_profile(&self, profile: &MemberProfile) -> Result<(), Box<dyn std::error::Error>> {
        self.connection.execute(
            "INSERT OR REPLACE INTO member_profiles 
             (member_id, bio, website, location, company, skills, interests, 
              github_username, twitter_username, linkedin_url, avatar_url, preferences, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                profile.member_id,
                profile.bio,
                profile.website,
                profile.location,
                profile.company,
                profile.skills,
                profile.interests,
                profile.github_username,
                profile.twitter_username,
                profile.linkedin_url,
                profile.avatar_url,
                profile.preferences,
                profile.updated_at.to_rfc3339()
            ],
        )?;
        Ok(())
    }

    /// Retrieves the total count of active members
    /// # Returns
    ///
    /// * `Result<i64, Box<dyn std::error::Error>>` - Returns `Ok` with the count of active members on success, or an error on failure
    pub fn get_member_count(&self) -> Result<i64, Box<dyn std::error::Error>> {
        let count: i64 = self.connection.query_row(
            "SELECT COUNT(*) FROM community_members WHERE is_active = 1",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Retrieves recent activities from members
    /// # Arguments
    ///
    /// * `limit` - The maximum number of activities to retrieve
    ///
    /// # Returns
    ///
    /// * `Result<Vec<MemberActivity>, Box<dyn std::error::Error>>` - Returns `Ok` with a vector of `MemberActivity` objects on success, or an error on failure
    pub fn get_recent_activities(&self, limit: i64) -> Result<Vec<MemberActivity>, Box<dyn std::error::Error>> {
        let mut stmt = self.connection.prepare(
            "SELECT id, member_id, activity_type, description, metadata, timestamp
             FROM member_activities
             ORDER BY timestamp DESC
             LIMIT ?1"
        )?;

        let activity_iter = stmt.query_map([limit], |row| {
            Ok(MemberActivity {
                id: Some(row.get(0)?),
                member_id: row.get(1)?,
                activity_type: ActivityType::from_string(&row.get::<_, String>(2)?).unwrap(),
                description: row.get(3)?,
                metadata: row.get(4)?,
                timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                    .unwrap().with_timezone(&Utc),
            })
        })?;

        let mut activities = Vec::new();
        for activity in activity_iter {
            activities.push(activity?);
        }

        Ok(activities)
    }
}

/// Placeholder documentation for public items

// Demo function
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Community Management System Demo");
    println!("=====================================\n");

    // Create database
    let db = CommunityDatabase::new(Path::new("community_demo.db"))?;
    println!("✅ Database initialized successfully");

    // Register some members
    let member1 = CommunityMember {
        id: None,
        email: "alice@example.com".to_string(),
        username: "alice_dev".to_string(),
        role: MemberRole::Developer,
        display_name: Some("Alice Johnson".to_string()),
        created_at: Utc::now(),
        last_active: None,
        is_active: true,
    };

    let member2 = CommunityMember {
        id: None,
        email: "bob@example.com".to_string(),
        username: "bob_admin".to_string(),
        role: MemberRole::Admin,
        display_name: Some("Bob Smith".to_string()),
        created_at: Utc::now(),
        last_active: None,
        is_active: true,
    };

    let member3 = CommunityMember {
        id: None,
        email: "carol@example.com".to_string(),
        username: "carol_user".to_string(),
        role: MemberRole::Member,
        display_name: Some("Carol Williams".to_string()),
        created_at: Utc::now(),
        last_active: None,
        is_active: true,
    };

    let alice_id = db.register_member(&member1)?;
    let bob_id = db.register_member(&member2)?;
    let carol_id = db.register_member(&member3)?;

    println!("✅ Registered 3 community members:");
    println!("   - Alice (Developer, ID: {})", alice_id);
    println!("   - Bob (Admin, ID: {})", bob_id);
    println!("   - Carol (Member, ID: {})", carol_id);

    // Update some profiles
    let alice_profile = MemberProfile {
        member_id: alice_id,
        bio: Some("Rust developer passionate about clean code and performance".to_string()),
        website: Some("https://alice.dev".to_string()),
        location: Some("San Francisco, CA".to_string()),
        company: Some("TechCorp".to_string()),
        skills: Some("Rust, Python, Docker, Kubernetes".to_string()),
        interests: Some("Open source, Machine learning, Blockchain".to_string()),
        github_username: Some("alice_dev".to_string()),
        twitter_username: Some("alice_codes".to_string()),
        linkedin_url: Some("https://linkedin.com/in/alicejohnson".to_string()),
        avatar_url: Some("https://avatars.example.com/alice".to_string()),
        preferences: Some(r#"{"notifications": true, "theme": "dark"}"#.to_string()),
        updated_at: Utc::now(),
    };

    db.update_profile(&alice_profile)?;
    println!("✅ Updated Alice's profile");

    // Log some activities
    let activities = vec![
        MemberActivity {
            id: None,
            member_id: alice_id,
            activity_type: ActivityType::Login,
            description: Some("User logged in".to_string()),
            metadata: Some(r#"{"ip": "192.168.1.100", "user_agent": "Mozilla/5.0"}"#.to_string()),
            timestamp: Utc::now(),
        },
        MemberActivity {
            id: None,
            member_id: alice_id,
            activity_type: ActivityType::ContentView,
            description: Some("Viewed documentation".to_string()),
            metadata: Some(r#"{"page": "/docs/getting-started", "duration": 300}"#.to_string()),
            timestamp: Utc::now(),
        },
        MemberActivity {
            id: None,
            member_id: bob_id,
            activity_type: ActivityType::Login,
            description: Some("Admin login".to_string()),
            metadata: Some(r#"{"ip": "10.0.0.5", "admin_panel": true}"#.to_string()),
            timestamp: Utc::now(),
        },
        MemberActivity {
            id: None,
            member_id: carol_id,
            activity_type: ActivityType::ProfileUpdate,
            description: Some("Updated profile information".to_string()),
            metadata: Some(r#"{"fields_updated": ["bio", "location"]}"#.to_string()),
            timestamp: Utc::now(),
        },
    ];

    for activity in &activities {
        db.log_activity(activity)?;
    }
    println!("✅ Logged {} activities", activities.len());

    // Change a role
    db.update_member_role(carol_id, &MemberRole::Developer)?;
    println!("✅ Promoted Carol to Developer role");

    // Get some analytics
    let member_count = db.get_member_count()?;
    println!("\n📊 Community Analytics:");
    println!("   - Total active members: {}", member_count);

    let recent_activities = db.get_recent_activities(10)?;
    println!("   - Recent activities ({}):", recent_activities.len());
    for activity in &recent_activities {
        println!("     • {} by member {} at {}", 
                 activity.activity_type.to_string(), 
                 activity.member_id, 
                 activity.timestamp.format("%Y-%m-%d %H:%M:%S"));
    }

    // Test member lookup
    println!("\n🔍 Member Lookup:");
    if let Some(alice) = db.get_member_by_email("alice@example.com")? {
        println!("   - Found Alice: {} ({})", alice.username, alice.role.to_string());
        println!("     Display name: {}", alice.display_name.unwrap_or("None".to_string()));
        println!("     Last active: {}", 
                 alice.last_active
                     .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                     .unwrap_or("Never".to_string()));
    }

    println!("\n✨ Demo completed successfully!");
    println!("   Database file: community_demo.db");
    println!("   You can inspect the database with: sqlite3 community_demo.db");

    Ok(())
}
