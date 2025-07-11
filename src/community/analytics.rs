//! Community Analytics Module
//!
//! Provides analytics and insights for community member data, designed for
//! marketing and administrative purposes. Generates reports on member engagement,
//! growth, and activity patterns.

use crate::error::UveddiError;
use chrono::{Duration, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Placeholder documentation for public items
/// Analytics data for member engagement and growth
#[derive(Debug, Serialize, Deserialize)]
pub struct MemberAnalytics {
    /// Total number of registered members
    pub total_members: usize,
    /// Number of active members (logged in recently)
    pub active_members: usize,
    /// Members by role breakdown
    pub members_by_role: HashMap<String, usize>,
    /// Growth metrics
    pub growth_metrics: GrowthMetrics,
    /// Engagement metrics
    pub engagement_metrics: EngagementMetrics,
    /// Geographic and demographic insights
    pub demographic_insights: DemographicInsights,
}

/// Placeholder documentation for public items
/// Member growth metrics over time
#[derive(Debug, Serialize, Deserialize)]
pub struct GrowthMetrics {
    /// New registrations in the last 7 days
    pub new_members_week: usize,
    /// New registrations in the last 30 days
    pub new_members_month: usize,
    /// Growth rate (percentage) compared to previous month
    pub growth_rate_monthly: f64,
    /// Average registrations per day over last 30 days
    pub avg_daily_registrations: f64,
}

/// Placeholder documentation for public items
/// Member engagement metrics
#[derive(Debug, Serialize, Deserialize)]
pub struct EngagementMetrics {
    /// Daily active users (last 24 hours)
    pub daily_active_users: usize,
    /// Weekly active users (last 7 days)
    pub weekly_active_users: usize,
    /// Monthly active users (last 30 days)
    pub monthly_active_users: usize,
    /// Average session duration (in minutes)
    pub avg_session_duration: f64,
    /// Most popular activities
    pub popular_activities: Vec<ActivityStats>,
}

/// Placeholder documentation for public items
/// Activity statistics for engagement analysis
#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityStats {
    /// Type of activity
    pub activity_type: String,
    /// Number of times this activity occurred
    pub count: usize,
    /// Unique members who performed this activity
    pub unique_members: usize,
}

/// Placeholder documentation for public items
/// Demographic insights for marketing
#[derive(Debug, Serialize, Deserialize)]
pub struct DemographicInsights {
    /// Most common companies/organizations
    pub top_companies: Vec<CompanyStats>,
    /// Programming language interests
    pub language_interests: Vec<LanguageStats>,
    /// Experience level distribution
    pub experience_levels: HashMap<String, usize>,
    /// Referral source breakdown
    pub referral_sources: HashMap<String, usize>,
    /// Newsletter subscription rate
    pub newsletter_subscription_rate: f64,
    /// Marketing consent rate
    pub marketing_consent_rate: f64,
}

/// Placeholder documentation for public items
/// Company statistics for B2B insights
#[derive(Debug, Serialize, Deserialize)]
pub struct CompanyStats {
    /// Company name
    pub company: String,
    /// Number of members from this company
    pub member_count: usize,
    /// Percentage of total members
    pub percentage: f64,
}

/// Placeholder documentation for public items
/// Programming language interest statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct LanguageStats {
    /// Programming language
    pub language: String,
    /// Number of members interested in this language
    pub member_count: usize,
    /// Percentage of total members
    pub percentage: f64,
}

/// Placeholder documentation for public items
/// Community analytics generator
pub struct AnalyticsEngine<'a> {
    conn: &'a Connection,
}

impl<'a> AnalyticsEngine<'a> {
    /// Create a new analytics engine with database connection
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Generate comprehensive member analytics
    pub fn generate_analytics(&self) -> Result<MemberAnalytics, UveddiError> {
        let total_members = self.get_total_members()?;
        let active_members = self.get_active_members(Duration::days(30))?;
        let members_by_role = self.get_members_by_role()?;
        let growth_metrics = self.calculate_growth_metrics()?;
        let engagement_metrics = self.calculate_engagement_metrics()?;
        let demographic_insights = self.generate_demographic_insights()?;

        Ok(MemberAnalytics {
            total_members,
            active_members,
            members_by_role,
            growth_metrics,
            engagement_metrics,
            demographic_insights,
        })
    }

    /// Get total number of registered members
    fn get_total_members(&self) -> Result<usize, UveddiError> {
        let mut stmt = self
            .conn
            .prepare("SELECT COUNT(*) FROM community_members WHERE is_active = 1")?;
        let count: i64 = stmt.query_row([], |row| row.get(0))?;
        Ok(count as usize)
    }

    /// Get number of active members (active within specified duration)
    fn get_active_members(&self, within_duration: Duration) -> Result<usize, UveddiError> {
        let cutoff = Utc::now() - within_duration;
        let mut stmt = self.conn.prepare(
            "SELECT COUNT(*) FROM community_members WHERE is_active = 1 AND last_active >= ?1",
        )?;
        let count: i64 = stmt.query_row([cutoff.to_rfc3339()], |row| row.get(0))?;
        Ok(count as usize)
    }

    /// Get member distribution by role
    fn get_members_by_role(&self) -> Result<HashMap<String, usize>, UveddiError> {
        let mut stmt = self.conn.prepare(
            "SELECT role, COUNT(*) FROM community_members WHERE is_active = 1 GROUP BY role",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
        })?;

        let mut result = HashMap::new();
        for row in rows {
            let (role, count) = row?;
            result.insert(role, count);
        }
        Ok(result)
    }

    /// Calculate growth metrics
    fn calculate_growth_metrics(&self) -> Result<GrowthMetrics, UveddiError> {
        let now = Utc::now();
        let week_ago = now - Duration::days(7);
        let month_ago = now - Duration::days(30);
        let two_months_ago = now - Duration::days(60);

        // New members in last week
        let mut stmt = self
            .conn
            .prepare("SELECT COUNT(*) FROM community_members WHERE created_at >= ?1")?;
        let new_members_week: i64 = stmt.query_row([week_ago.to_rfc3339()], |row| row.get(0))?;

        // New members in last month
        let new_members_month: i64 = stmt.query_row([month_ago.to_rfc3339()], |row| row.get(0))?;

        // New members in previous month (for growth rate calculation)
        let mut stmt_prev = self.conn.prepare(
            "SELECT COUNT(*) FROM community_members WHERE created_at >= ?1 AND created_at < ?2",
        )?;
        let prev_month_members: i64 = stmt_prev.query_row(
            [two_months_ago.to_rfc3339(), month_ago.to_rfc3339()],
            |row| row.get(0),
        )?;

        // Calculate growth rate
        let growth_rate_monthly = if prev_month_members > 0 {
            ((new_members_month - prev_month_members) as f64 / prev_month_members as f64) * 100.0
        } else {
            100.0 // 100% growth if we had no members in previous month
        };

        let avg_daily_registrations = new_members_month as f64 / 30.0;

        Ok(GrowthMetrics {
            new_members_week: new_members_week as usize,
            new_members_month: new_members_month as usize,
            growth_rate_monthly,
            avg_daily_registrations,
        })
    }

    /// Calculate engagement metrics
    fn calculate_engagement_metrics(&self) -> Result<EngagementMetrics, UveddiError> {
        let now = Utc::now();
        let day_ago = now - Duration::days(1);
        let week_ago = now - Duration::days(7);
        let month_ago = now - Duration::days(30);

        // Daily active users
        let daily_active_users = self.get_active_members(Duration::days(1))?;

        // Weekly active users
        let weekly_active_users = self.get_active_members(Duration::days(7))?;

        // Monthly active users
        let monthly_active_users = self.get_active_members(Duration::days(30))?;

        // Popular activities in last 30 days
        let mut stmt = self.conn.prepare(
            r#"
            SELECT activity_type, COUNT(*) as count, COUNT(DISTINCT member_id) as unique_members
            FROM member_activity 
            WHERE timestamp >= ?1 
            GROUP BY activity_type 
            ORDER BY count DESC 
            LIMIT 10
            "#,
        )?;

        let activity_rows = stmt.query_map([month_ago.to_rfc3339()], |row| {
            Ok(ActivityStats {
                activity_type: row.get(0)?,
                count: row.get::<_, i64>(1)? as usize,
                unique_members: row.get::<_, i64>(2)? as usize,
            })
        })?;

        let mut popular_activities = Vec::new();
        for activity in activity_rows {
            popular_activities.push(activity?);
        }

        // TODO: Calculate average session duration when we implement session tracking
        let avg_session_duration = 0.0;

        Ok(EngagementMetrics {
            daily_active_users,
            weekly_active_users,
            monthly_active_users,
            avg_session_duration,
            popular_activities,
        })
    }

    /// Generate demographic insights for marketing
    fn generate_demographic_insights(&self) -> Result<DemographicInsights, UveddiError> {
        let total_members = self.get_total_members()? as f64;

        // Top companies
        let mut stmt = self.conn.prepare(
            r#"
            SELECT company, COUNT(*) as count
            FROM community_members 
            WHERE is_active = 1 AND company IS NOT NULL AND company != ''
            GROUP BY company 
            ORDER BY count DESC 
            LIMIT 10
            "#,
        )?;

        let company_rows = stmt.query_map([], |row| {
            let count = row.get::<_, i64>(1)? as usize;
            Ok(CompanyStats {
                company: row.get(0)?,
                member_count: count,
                percentage: (count as f64 / total_members) * 100.0,
            })
        })?;

        let mut top_companies = Vec::new();
        for company in company_rows {
            top_companies.push(company?);
        }

        // Language interests (need to parse JSON arrays)
        let language_interests = self.calculate_language_interests()?;

        // Experience levels
        let mut stmt = self.conn.prepare(
            r#"
            SELECT experience_level, COUNT(*) 
            FROM community_members 
            WHERE is_active = 1 AND experience_level IS NOT NULL
            GROUP BY experience_level
            "#,
        )?;

        let exp_rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
        })?;

        let mut experience_levels = HashMap::new();
        for row in exp_rows {
            let (level, count) = row?;
            experience_levels.insert(level, count);
        }

        // Referral sources
        let mut stmt = self.conn.prepare(
            r#"
            SELECT referral_source, COUNT(*) 
            FROM community_members 
            WHERE is_active = 1 AND referral_source IS NOT NULL
            GROUP BY referral_source
            "#,
        )?;

        let ref_rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
        })?;

        let mut referral_sources = HashMap::new();
        for row in ref_rows {
            let (source, count) = row?;
            referral_sources.insert(source, count);
        }

        // Newsletter and marketing consent rates
        let mut stmt = self.conn.prepare(
            "SELECT newsletter_subscribed, marketing_consent FROM community_members WHERE is_active = 1"
        )?;

        let consent_rows = stmt.query_map([], |row| {
            Ok((row.get::<_, bool>(0)?, row.get::<_, bool>(1)?))
        })?;

        let mut newsletter_count = 0;
        let mut marketing_count = 0;
        let mut total_count = 0;

        for row in consent_rows {
            let (newsletter, marketing) = row?;
            if newsletter {
                newsletter_count += 1;
            }
            if marketing {
                marketing_count += 1;
            }
            total_count += 1;
        }

        let newsletter_subscription_rate = if total_count > 0 {
            (newsletter_count as f64 / total_count as f64) * 100.0
        } else {
            0.0
        };

        let marketing_consent_rate = if total_count > 0 {
            (marketing_count as f64 / total_count as f64) * 100.0
        } else {
            0.0
        };

        Ok(DemographicInsights {
            top_companies,
            language_interests,
            experience_levels,
            referral_sources,
            newsletter_subscription_rate,
            marketing_consent_rate,
        })
    }

    /// Calculate programming language interest statistics
    fn calculate_language_interests(&self) -> Result<Vec<LanguageStats>, UveddiError> {
        let mut stmt = self.conn.prepare(
            "SELECT languages FROM community_members WHERE is_active = 1 AND languages IS NOT NULL",
        )?;

        let language_rows = stmt.query_map([], |row| row.get::<_, String>(0))?;

        let mut language_counts: HashMap<String, usize> = HashMap::new();
        let mut total_members = 0;

        for row in language_rows {
            let languages_json = row?;
            if let Ok(languages) = serde_json::from_str::<Vec<String>>(&languages_json) {
                total_members += 1;
                for language in languages {
                    *language_counts.entry(language).or_insert(0) += 1;
                }
            }
        }

        let mut language_stats: Vec<LanguageStats> = language_counts
            .into_iter()
            .map(|(language, count)| LanguageStats {
                language,
                member_count: count,
                percentage: if total_members > 0 {
                    (count as f64 / total_members as f64) * 100.0
                } else {
                    0.0
                },
            })
            .collect();

        language_stats.sort_by(|a, b| b.member_count.cmp(&a.member_count));
        language_stats.truncate(10); // Top 10 languages

        Ok(language_stats)
    }
}
