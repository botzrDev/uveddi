//! Automated report scheduling system
//!
//! Provides cron-like scheduling for automated report generation and distribution

use anyhow::{Context, Result};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{interval, sleep, Duration as TokioDuration, Instant};
use uuid::Uuid;

use super::reporting::{ReportingEngine, ReportConfiguration, ReportType, GeneratedReport};

/// Schedule configuration for automated reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleConfig {
    pub enabled: bool,
    pub schedule_type: ScheduleType,
    pub timezone: String,
    pub retry_config: RetryConfig,
}

/// Different types of scheduling supported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScheduleType {
    /// Run at specified intervals
    Interval {
        duration_minutes: u64,
    },
    /// Run at specific times daily
    Daily {
        times: Vec<String>, // Format: "HH:MM"
    },
    /// Run on specific days of the week
    Weekly {
        days: Vec<String>, // Format: "Monday", "Tuesday", etc.
        time: String,      // Format: "HH:MM"
    },
    /// Run on specific days of the month
    Monthly {
        days: Vec<u8>, // 1-31
        time: String,  // Format: "HH:MM"
    },
    /// Full cron expression support
    Cron {
        expression: String,
    },
}

/// Retry configuration for failed report generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_retries: u8,
    pub initial_delay_seconds: u64,
    pub backoff_multiplier: f64,
    pub max_delay_seconds: u64,
}

/// Scheduled job information
#[derive(Debug, Clone)]
pub struct ScheduledJob {
    pub id: Uuid,
    pub name: String,
    pub config: ReportConfiguration,
    pub schedule: ScheduleConfig,
    pub next_run: DateTime<Utc>,
    pub last_run: Option<DateTime<Utc>>,
    pub run_count: u64,
    pub failure_count: u64,
    pub status: JobStatus,
}

/// Job execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Disabled,
}

/// Job execution result
#[derive(Debug, Clone)]
pub struct JobExecutionResult {
    pub job_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: JobStatus,
    pub report: Option<GeneratedReport>,
    pub error_message: Option<String>,
    pub retry_count: u8,
}

/// Report scheduler manages automated report generation
pub struct ReportScheduler {
    jobs: HashMap<Uuid, ScheduledJob>,
    execution_history: Vec<JobExecutionResult>,
    is_running: bool,
}

impl ReportScheduler {
    /// Create a new report scheduler
    pub fn new() -> Self {
        Self {
            jobs: HashMap::new(),
            execution_history: Vec::new(),
            is_running: false,
        }
    }

    /// Add a new scheduled job
    pub fn add_job(
        &mut self,
        name: String,
        config: ReportConfiguration,
        schedule: ScheduleConfig,
    ) -> Result<Uuid> {
        let job_id = Uuid::new_v4();
        let next_run = self.calculate_next_run(&schedule)?;

        let job = ScheduledJob {
            id: job_id,
            name,
            config,
            schedule,
            next_run,
            last_run: None,
            run_count: 0,
            failure_count: 0,
            status: JobStatus::Pending,
        };

        self.jobs.insert(job_id, job);
        log::info!("Added scheduled job {} with next run at {}", job_id, next_run);

        Ok(job_id)
    }

    /// Remove a scheduled job
    pub fn remove_job(&mut self, job_id: Uuid) -> Result<()> {
        self.jobs.remove(&job_id)
            .context("Job not found")?;
        log::info!("Removed scheduled job {}", job_id);
        Ok(())
    }

    /// Update job schedule
    pub fn update_job_schedule(&mut self, job_id: Uuid, schedule: ScheduleConfig) -> Result<()> {
        // Calculate next run time first
        let next_run = self.calculate_next_run(&schedule)?;
        
        // Now update the job
        let job = self.jobs.get_mut(&job_id)
            .context("Job not found")?;

        job.schedule = schedule;
        job.next_run = next_run;
        
        log::info!("Updated schedule for job {} with next run at {}", job_id, job.next_run);
        Ok(())
    }

    /// Enable or disable a job
    pub fn set_job_enabled(&mut self, job_id: Uuid, enabled: bool) -> Result<()> {
        let job = self.jobs.get_mut(&job_id)
            .context("Job not found")?;

        job.schedule.enabled = enabled;
        job.status = if enabled { JobStatus::Pending } else { JobStatus::Disabled };

        log::info!("Job {} is now {}", job_id, if enabled { "enabled" } else { "disabled" });
        Ok(())
    }

    /// Start the scheduler
    pub async fn start(&mut self, mut reporting_engine: ReportingEngine) -> Result<()> {
        if self.is_running {
            return Ok(());
        }

        self.is_running = true;
        log::info!("Starting report scheduler with {} jobs", self.jobs.len());

        let mut check_interval = interval(TokioDuration::from_secs(60)); // Check every minute

        while self.is_running {
            check_interval.tick().await;

            let now = Utc::now();
            let mut jobs_to_run = Vec::new();

            // Find jobs that are ready to run
            for job in self.jobs.values_mut() {
                if job.schedule.enabled 
                    && matches!(job.status, JobStatus::Pending) 
                    && job.next_run <= now {
                    jobs_to_run.push(job.id);
                }
            }

            // Execute ready jobs
            for job_id in jobs_to_run {
                if let Some(job) = self.jobs.get_mut(&job_id) {
                    job.status = JobStatus::Running;
                    
                    // Clone job config for execution
                    let job_config = job.config.clone();
                    let job_name = job.name.clone();
                    
                    log::info!("Executing scheduled job: {} ({})", job_name, job_id);
                    
                    let execution_result = self.execute_job(
                        job_id,
                        &job_config,
                        &mut reporting_engine,
                    ).await;

                    // Update job status based on execution result
                    if let Some(job) = self.jobs.get(&job_id) {
                        let (next_run, log_message) = match &execution_result.status {
                            JobStatus::Completed => {
                                let schedule_clone = job.schedule.clone();
                                let next_run = self.calculate_next_run(&schedule_clone)?;
                                let log_message = format!("Job {} completed successfully. Next run: {}", job_id, next_run);
                                (next_run, log_message)
                            }
                            JobStatus::Failed => {
                                let retry_config = job.schedule.retry_config.clone();
                                let failure_count = job.failure_count + 1;
                                let next_run = self.calculate_retry_time(&retry_config, failure_count);
                                let log_message = format!("Job {} failed. Next retry: {}", job_id, next_run);
                                (next_run, log_message)
                            }
                            _ => {
                                continue; // Skip status updates for other statuses
                            }
                        };
                        
                        // Now update the job with computed values
                        if let Some(job) = self.jobs.get_mut(&job_id) {
                            match &execution_result.status {
                                JobStatus::Completed => {
                                    job.status = JobStatus::Pending;
                                    job.last_run = Some(now);
                                    job.run_count += 1;
                                    job.next_run = next_run;
                                    log::info!("{}", log_message);
                                }
                                JobStatus::Failed => {
                                    job.status = JobStatus::Pending;
                                    job.failure_count += 1;
                                    job.next_run = next_run;
                                    log::error!("{}", log_message);
                                }
                                _ => {}
                            }
                        }
                    }

                    self.execution_history.push(execution_result);
                }
            }

            // Clean up old execution history (keep last 1000 entries)
            if self.execution_history.len() > 1000 {
                self.execution_history.drain(0..self.execution_history.len() - 1000);
            }
        }

        Ok(())
    }

    /// Stop the scheduler
    pub fn stop(&mut self) {
        self.is_running = false;
        log::info!("Report scheduler stopped");
    }

    /// Get all scheduled jobs
    pub fn get_jobs(&self) -> Vec<&ScheduledJob> {
        self.jobs.values().collect()
    }

    /// Get job by ID
    pub fn get_job(&self, job_id: Uuid) -> Option<&ScheduledJob> {
        self.jobs.get(&job_id)
    }

    /// Get execution history for a job
    pub fn get_job_history(&self, job_id: Uuid) -> Vec<&JobExecutionResult> {
        self.execution_history
            .iter()
            .filter(|result| result.job_id == job_id)
            .collect()
    }

    /// Get recent execution history
    pub fn get_recent_history(&self, limit: usize) -> Vec<&JobExecutionResult> {
        self.execution_history
            .iter()
            .rev()
            .take(limit)
            .collect()
    }

    /// Execute a single job
    async fn execute_job(
        &self,
        job_id: Uuid,
        config: &ReportConfiguration,
        reporting_engine: &mut ReportingEngine,
    ) -> JobExecutionResult {
        let started_at = Utc::now();
        let mut result = JobExecutionResult {
            job_id,
            started_at,
            completed_at: None,
            status: JobStatus::Running,
            report: None,
            error_message: None,
            retry_count: 0,
        };

        // Generate the report
        match reporting_engine.generate_report(config).await {
            Ok(report) => {
                // Distribute the report
                match reporting_engine.distribute_report(&report, config).await {
                    Ok(_) => {
                        result.status = JobStatus::Completed;
                        result.report = Some(report);
                        result.completed_at = Some(Utc::now());
                        log::info!("Successfully executed job {}", job_id);
                    }
                    Err(e) => {
                        result.status = JobStatus::Failed;
                        result.error_message = Some(format!("Distribution failed: {}", e));
                        result.completed_at = Some(Utc::now());
                        log::error!("Job {} distribution failed: {}", job_id, e);
                    }
                }
            }
            Err(e) => {
                result.status = JobStatus::Failed;
                result.error_message = Some(format!("Report generation failed: {}", e));
                result.completed_at = Some(Utc::now());
                log::error!("Job {} generation failed: {}", job_id, e);
            }
        }

        result
    }

    /// Calculate the next run time for a schedule
    fn calculate_next_run(&self, schedule: &ScheduleConfig) -> Result<DateTime<Utc>> {
        let now = Utc::now();
        
        match &schedule.schedule_type {
            ScheduleType::Interval { duration_minutes } => {
                Ok(now + Duration::minutes(*duration_minutes as i64))
            }
            ScheduleType::Daily { times } => {
                // For simplicity, use the first time for now
                if let Some(time_str) = times.first() {
                    let (hour, minute) = self.parse_time(time_str)?;
                    let mut next_run = now.date_naive().and_hms_opt(hour, minute, 0)
                        .context("Invalid time")?
                        .and_utc();
                    
                    if next_run <= now {
                        next_run = next_run + Duration::days(1);
                    }
                    
                    Ok(next_run)
                } else {
                    Ok(now + Duration::days(1)) // Default to tomorrow
                }
            }
            ScheduleType::Weekly { days: _, time } => {
                // For simplicity, calculate next week for now
                let (hour, minute) = self.parse_time(time)?;
                let next_run = (now + Duration::weeks(1))
                    .date_naive()
                    .and_hms_opt(hour, minute, 0)
                    .context("Invalid time")?
                    .and_utc();
                
                Ok(next_run)
            }
            ScheduleType::Monthly { days: _, time } => {
                // For simplicity, calculate next month for now
                let (hour, minute) = self.parse_time(time)?;
                let next_run = (now + Duration::days(30))
                    .date_naive()
                    .and_hms_opt(hour, minute, 0)
                    .context("Invalid time")?
                    .and_utc();
                
                Ok(next_run)
            }
            ScheduleType::Cron { expression: _ } => {
                // For now, default to next hour
                // TODO: Implement proper cron parsing
                Ok(now + Duration::hours(1))
            }
        }
    }

    /// Calculate retry time with exponential backoff
    fn calculate_retry_time(&self, retry_config: &RetryConfig, failure_count: u64) -> DateTime<Utc> {
        if failure_count > retry_config.max_retries as u64 {
            // If we've exceeded max retries, schedule for next regular run
            return Utc::now() + Duration::hours(24);
        }

        let delay_seconds = (retry_config.initial_delay_seconds as f64 
            * retry_config.backoff_multiplier.powi((failure_count - 1) as i32))
            .min(retry_config.max_delay_seconds as f64) as i64;

        Utc::now() + Duration::seconds(delay_seconds)
    }

    /// Parse time string in HH:MM format
    fn parse_time(&self, time_str: &str) -> Result<(u32, u32)> {
        let parts: Vec<&str> = time_str.split(':').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid time format: {}", time_str);
        }

        let hour: u32 = parts[0].parse()
            .context("Invalid hour")?;
        let minute: u32 = parts[1].parse()
            .context("Invalid minute")?;

        if hour > 23 || minute > 59 {
            anyhow::bail!("Invalid time values: {}:{}", hour, minute);
        }

        Ok((hour, minute))
    }
}

impl Default for ReportScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_seconds: 300, // 5 minutes
            backoff_multiplier: 2.0,
            max_delay_seconds: 3600, // 1 hour
        }
    }
}

/// Builder for creating schedule configurations
pub struct ScheduleBuilder {
    schedule_type: Option<ScheduleType>,
    timezone: String,
    retry_config: RetryConfig,
    enabled: bool,
}

impl ScheduleBuilder {
    /// Create a new schedule builder
    pub fn new() -> Self {
        Self {
            schedule_type: None,
            timezone: "UTC".to_string(),
            retry_config: RetryConfig::default(),
            enabled: true,
        }
    }

    /// Set interval-based scheduling
    pub fn every_minutes(mut self, minutes: u64) -> Self {
        self.schedule_type = Some(ScheduleType::Interval {
            duration_minutes: minutes,
        });
        self
    }

    /// Set daily scheduling
    pub fn daily_at(mut self, time: &str) -> Self {
        self.schedule_type = Some(ScheduleType::Daily {
            times: vec![time.to_string()],
        });
        self
    }

    /// Set weekly scheduling
    pub fn weekly_on(mut self, day: &str, time: &str) -> Self {
        self.schedule_type = Some(ScheduleType::Weekly {
            days: vec![day.to_string()],
            time: time.to_string(),
        });
        self
    }

    /// Set monthly scheduling
    pub fn monthly_on(mut self, day: u8, time: &str) -> Self {
        self.schedule_type = Some(ScheduleType::Monthly {
            days: vec![day],
            time: time.to_string(),
        });
        self
    }

    /// Set cron expression
    pub fn cron(mut self, expression: &str) -> Self {
        self.schedule_type = Some(ScheduleType::Cron {
            expression: expression.to_string(),
        });
        self
    }

    /// Set timezone
    pub fn timezone(mut self, tz: &str) -> Self {
        self.timezone = tz.to_string();
        self
    }

    /// Configure retry behavior
    pub fn with_retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }

    /// Set enabled status
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Build the schedule configuration
    pub fn build(self) -> Result<ScheduleConfig> {
        let schedule_type = self.schedule_type
            .context("Schedule type must be specified")?;

        Ok(ScheduleConfig {
            enabled: self.enabled,
            schedule_type,
            timezone: self.timezone,
            retry_config: self.retry_config,
        })
    }
}

impl Default for ScheduleBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience functions for common schedules
pub mod schedules {
    use super::*;

    /// Create a daily schedule at specified time
    pub fn daily(time: &str) -> Result<ScheduleConfig> {
        ScheduleBuilder::new().daily_at(time).build()
    }

    /// Create a weekly schedule
    pub fn weekly(day: &str, time: &str) -> Result<ScheduleConfig> {
        ScheduleBuilder::new().weekly_on(day, time).build()
    }

    /// Create an interval-based schedule
    pub fn every_hours(hours: u64) -> Result<ScheduleConfig> {
        ScheduleBuilder::new().every_minutes(hours * 60).build()
    }

    /// Create a monthly schedule
    pub fn monthly(day: u8, time: &str) -> Result<ScheduleConfig> {
        ScheduleBuilder::new().monthly_on(day, time).build()
    }
}