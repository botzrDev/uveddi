use chrono::{DateTime, Utc};
use rusqlite::{Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// Minimal community database implementation for testing

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MemberRole {
    Member,
    Developer,
    Admin,
}

impl MemberRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            MemberRole::Member => "member",
            MemberRole::Developer => "developer",
            MemberRole::Admin => "admin",
        }
    }

    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "admin" => MemberRole::Admin,
            "developer" => MemberRole::Developer,
            _ => MemberRole::Member,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemberProfile {
    pub company: Option<String>,
    pub job_title: Option<String>,
    pub languages: Vec<String>,
    pub experience_level: Option<String>,
    pub use_case: Option<String>,
    pub referral_source: Option<String>,
    pub newsletter_subscribed: bool,
    pub marketing_consent: bool,
    pub custom_fields: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityMember {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: MemberRole,
    pub created_at: DateTime<Utc>,
    pub last_active: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub profile: MemberProfile,
    pub email_verified: bool,
    pub avatar_url: Option<String>,
}

pub struct CommunityDatabase {
    conn: Connection,
}

impl CommunityDatabase {
    pub fn new(db_path: &str) -> SqlResult<Self> {
        let conn = Connection::open(db_path)?;
        let db = Self { conn };
        db.initialize_schema()?;
        Ok(db)
    }

    fn initialize_schema(&self) -> SqlResult<()> {
        self.conn.execute_batch(
            r#"
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
                company TEXT,
                job_title TEXT,
                languages TEXT,
                experience_level TEXT,
                use_case TEXT,
                referral_source TEXT,
                newsletter_subscribed BOOLEAN NOT NULL DEFAULT 0,
                marketing_consent BOOLEAN NOT NULL DEFAULT 0,
                custom_fields TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_member_email ON community_members(email);
            CREATE INDEX IF NOT EXISTS idx_member_role ON community_members(role);
            CREATE INDEX IF NOT EXISTS idx_member_active ON community_members(is_active);
            "#,
        )?;
        Ok(())
    }

    pub fn register_member(
        &self,
        email: &str,
        name: &str,
        role: MemberRole,
    ) -> SqlResult<CommunityMember> {
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

        let languages_json = serde_json::to_string(&member.profile.languages).unwrap();
        let custom_fields_json = serde_json::to_string(&member.profile.custom_fields).unwrap();

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

        Ok(member)
    }

    pub fn get_member_by_email(&self, email: &str) -> SqlResult<Option<CommunityMember>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, email, name, role, created_at, last_active, is_active, email_verified,
                   avatar_url, company, job_title, languages, experience_level, use_case,
                   referral_source, newsletter_subscribed, marketing_consent, custom_fields
            FROM community_members WHERE email = ?1
            "#,
        )?;

        let mut member_iter = stmt.query_map([email], |row| {
            let languages_json: String = row.get(11)?;
            let custom_fields_json: String = row.get(17)?;

            let languages: Vec<String> = serde_json::from_str(&languages_json).unwrap_or_default();
            let custom_fields: HashMap<String, String> =
                serde_json::from_str(&custom_fields_json).unwrap_or_default();

            let last_active_str: Option<String> = row.get(5)?;
            let last_active = last_active_str
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc));

            Ok(CommunityMember {
                id: row.get(0)?,
                email: row.get(1)?,
                name: row.get(2)?,
                role: MemberRole::from_string(&row.get::<_, String>(3)?),
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
        })?;

        if let Some(member) = member_iter.next() {
            return Ok(Some(member?));
        }
        Ok(None)
    }

    pub fn list_members(&self) -> SqlResult<Vec<CommunityMember>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, email, name, role, created_at, last_active, is_active, email_verified,
                   avatar_url, company, job_title, languages, experience_level, use_case,
                   referral_source, newsletter_subscribed, marketing_consent, custom_fields
            FROM community_members WHERE is_active = 1 ORDER BY created_at DESC
            "#,
        )?;

        let member_iter = stmt.query_map([], |row| {
            let languages_json: String = row.get(11)?;
            let custom_fields_json: String = row.get(17)?;

            let languages: Vec<String> = serde_json::from_str(&languages_json).unwrap_or_default();
            let custom_fields: HashMap<String, String> =
                serde_json::from_str(&custom_fields_json).unwrap_or_default();

            let last_active_str: Option<String> = row.get(5)?;
            let last_active = last_active_str
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc));

            Ok(CommunityMember {
                id: row.get(0)?,
                email: row.get(1)?,
                name: row.get(2)?,
                role: MemberRole::from_string(&row.get::<_, String>(3)?),
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
        })?;

        let mut members = Vec::new();
        for member in member_iter {
            members.push(member?);
        }
        Ok(members)
    }

    pub fn update_member_profile(
        &self,
        member_id: &str,
        profile: &MemberProfile,
    ) -> SqlResult<bool> {
        let languages_json = serde_json::to_string(&profile.languages).unwrap();
        let custom_fields_json = serde_json::to_string(&profile.custom_fields).unwrap();

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

        Ok(affected > 0)
    }

    pub fn update_member_role(&self, member_id: &str, new_role: MemberRole) -> SqlResult<bool> {
        let affected = self.conn.execute(
            "UPDATE community_members SET role = ?1 WHERE id = ?2",
            (new_role.as_str(), member_id),
        )?;
        Ok(affected > 0)
    }

    pub fn list_members_by_role(&self, role: MemberRole) -> SqlResult<Vec<CommunityMember>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, email, name, role, created_at, last_active, is_active, email_verified,
                   avatar_url, company, job_title, languages, experience_level, use_case,
                   referral_source, newsletter_subscribed, marketing_consent, custom_fields
            FROM community_members WHERE is_active = 1 AND role = ?1 ORDER BY created_at DESC
            "#,
        )?;

        let member_iter = stmt.query_map([role.as_str()], |row| {
            let languages_json: String = row.get(11)?;
            let custom_fields_json: String = row.get(17)?;

            let languages: Vec<String> = serde_json::from_str(&languages_json).unwrap_or_default();
            let custom_fields: HashMap<String, String> =
                serde_json::from_str(&custom_fields_json).unwrap_or_default();

            let last_active_str: Option<String> = row.get(5)?;
            let last_active = last_active_str
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc));

            Ok(CommunityMember {
                id: row.get(0)?,
                email: row.get(1)?,
                name: row.get(2)?,
                role: MemberRole::from_string(&row.get::<_, String>(3)?),
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
        })?;

        let mut members = Vec::new();
        for member in member_iter {
            members.push(member?);
        }
        Ok(members)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Community Database Demo");
    println!("==========================\n");

    // Initialize the community database
    let db = CommunityDatabase::new("working_community_demo.db")?;
    println!("✅ Community database initialized");

    // Register some demo members
    println!("\n📝 Registering demo members...");

    let alice = db.register_member("alice@techcorp.com", "Alice Johnson", MemberRole::Developer)?;
    println!("   Registered: {} ({})", alice.name, alice.email);

    let bob = db.register_member("bob@startup.io", "Bob Smith", MemberRole::Member)?;
    println!("   Registered: {} ({})", bob.name, bob.email);

    let admin = db.register_member("admin@uveddi.com", "Phillip Green", MemberRole::Admin)?;
    println!("   Registered: {} ({})", admin.name, admin.email);

    // Update Alice's profile with marketing information
    println!("\n📋 Updating member profiles...");
    let mut alice_profile = MemberProfile {
        company: Some("TechCorp Inc.".to_string()),
        job_title: Some("Senior Software Engineer".to_string()),
        languages: vec![
            "Rust".to_string(),
            "Python".to_string(),
            "TypeScript".to_string(),
        ],
        experience_level: Some("advanced".to_string()),
        use_case: Some("Code quality analysis for large codebase".to_string()),
        referral_source: Some("GitHub".to_string()),
        newsletter_subscribed: true,
        marketing_consent: true,
        custom_fields: HashMap::new(),
    };
    alice_profile
        .custom_fields
        .insert("team_size".to_string(), "15".to_string());
    alice_profile
        .custom_fields
        .insert("deployment_frequency".to_string(), "weekly".to_string());

    db.update_member_profile(&alice.id, &alice_profile)?;
    println!("   Updated profile for {}", alice.name);

    // Update Bob's profile
    let bob_profile = MemberProfile {
        company: Some("StartupIO".to_string()),
        job_title: Some("CTO".to_string()),
        languages: vec!["JavaScript".to_string(), "Go".to_string()],
        experience_level: Some("expert".to_string()),
        use_case: Some("Architectural review for scaling".to_string()),
        referral_source: Some("Twitter".to_string()),
        newsletter_subscribed: true,
        marketing_consent: false,
        custom_fields: HashMap::new(),
    };

    db.update_member_profile(&bob.id, &bob_profile)?;
    println!("   Updated profile for {}", bob.name);

    // Promote Bob to Developer role
    println!("\n👑 Updating member roles...");
    db.update_member_role(&bob.id, MemberRole::Developer)?;
    println!("   Promoted {} to Developer role", bob.name);

    // Test retrieval
    println!("\n🔍 Testing member retrieval...");
    if let Some(retrieved_alice) = db.get_member_by_email("alice@techcorp.com")? {
        println!(
            "   Found Alice: {} with company {}",
            retrieved_alice.name,
            retrieved_alice
                .profile
                .company
                .as_ref()
                .unwrap_or(&"None".to_string())
        );
        println!("   Languages: {:?}", retrieved_alice.profile.languages);
        println!(
            "   Custom fields: {:?}",
            retrieved_alice.profile.custom_fields
        );
    }

    // Display member list
    println!("\n👥 Current Community Members:");
    let members = db.list_members()?;
    for member in &members {
        let role_emoji = match member.role {
            MemberRole::Admin => "👑",
            MemberRole::Developer => "💻",
            MemberRole::Member => "👤",
        };
        println!(
            "   {} {} ({}) - {} - Company: {}",
            role_emoji,
            member.name,
            member.email,
            member.role.as_str(),
            member
                .profile
                .company
                .as_ref()
                .unwrap_or(&"None".to_string())
        );
    }

    // Generate analytics
    println!("\n📊 Community Analytics:");
    println!("   Total Members: {}", members.len());

    let developers = db.list_members_by_role(MemberRole::Developer)?;
    let admins = db.list_members_by_role(MemberRole::Admin)?;
    let regular_members = db.list_members_by_role(MemberRole::Member)?;

    println!("   Members by Role:");
    println!("     member: {}", regular_members.len());
    println!("     developer: {}", developers.len());
    println!("     admin: {}", admins.len());

    // Marketing insights
    let mut newsletter_subscribers = 0;
    let mut marketing_consents = 0;
    let mut companies = HashMap::new();
    let mut all_languages = HashMap::new();

    for member in &members {
        if member.profile.newsletter_subscribed {
            newsletter_subscribers += 1;
        }
        if member.profile.marketing_consent {
            marketing_consents += 1;
        }
        if let Some(company) = &member.profile.company {
            *companies.entry(company.clone()).or_insert(0) += 1;
        }
        for language in &member.profile.languages {
            *all_languages.entry(language.clone()).or_insert(0) += 1;
        }
    }

    println!("\n📈 Marketing Insights:");
    println!(
        "   Newsletter subscription rate: {:.1}%",
        (newsletter_subscribers as f64 / members.len() as f64) * 100.0
    );
    println!(
        "   Marketing consent rate: {:.1}%",
        (marketing_consents as f64 / members.len() as f64) * 100.0
    );

    if !companies.is_empty() {
        println!("   Top Companies:");
        for (company, count) in &companies {
            println!("     {company}: {count} members");
        }
    }

    if !all_languages.is_empty() {
        println!("   Programming Language Interests:");
        for (language, count) in &all_languages {
            println!("     {language}: {count} members");
        }
    }

    println!("\n✅ Demo completed successfully!");
    println!("💾 Database saved to: working_community_demo.db");
    println!("🔍 You can inspect the database using any SQLite browser");

    // Verify data integrity
    assert_eq!(members.len(), 3);
    assert!(members.iter().any(|m| m.role == MemberRole::Admin));
    assert!(members.iter().any(|m| m.role == MemberRole::Developer));
    assert_eq!(developers.len(), 2); // Alice and promoted Bob

    println!("🎉 All assertions passed! The community database is working perfectly.");

    Ok(())
}
