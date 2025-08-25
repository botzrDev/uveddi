//! Event System for Decoupled Communication
//!
//! This module implements an event-driven communication system to break
//! circular dependencies between AST and Analysis components (UV-105, Phase 1.2).
//!
//! The event bus allows components to communicate without direct dependencies,
//! implementing the Observer pattern for loose coupling.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, warn, error, debug};

/// Central event bus for component communication
///
/// The event bus enables loose coupling between components by allowing
/// them to communicate through events rather than direct method calls.
/// This breaks circular dependencies and enables better testability.
pub struct EventBus {
    sender: broadcast::Sender<DomainEvent>,
    _receiver: broadcast::Receiver<DomainEvent>,
}

impl EventBus {
    /// Create a new event bus with default capacity
    pub fn new() -> Self {
        Self::with_capacity(1000)
    }

    /// Create a new event bus with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        let (sender, receiver) = broadcast::channel(capacity);
        Self {
            sender,
            _receiver: receiver,
        }
    }

    /// Publish an event to all subscribers
    pub fn publish(&self, event: DomainEvent) -> Result<usize, EventError> {
        self.sender
            .send(event)
            .map_err(|_| EventError::NoSubscribers)
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<DomainEvent> {
        self.sender.subscribe()
    }

    /// Get the number of active subscribers
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Domain events for inter-component communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainEvent {
    /// AST-related events
    Ast(AstEvent),

    /// Analysis-related events
    Analysis(AnalysisEvent),

    /// Security-related events
    Security(SecurityEvent),

    /// Performance monitoring events
    Performance(PerformanceEvent),

    /// Cache-related events
    Cache(CacheEvent),
}

/// AST parsing and processing events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AstEvent {
    /// File successfully parsed
    FileParsed {
        file_path: String,
        ast_data: AstData,
        parse_duration: std::time::Duration,
        timestamp: DateTime<Utc>,
    },

    /// Batch parsing started
    BatchParsingStarted {
        total_files: usize,
        timestamp: DateTime<Utc>,
    },

    /// Batch parsing completed
    BatchParsingCompleted {
        successful: usize,
        failed: usize,
        total_duration: std::time::Duration,
        timestamp: DateTime<Utc>,
    },

    /// Parse error occurred
    ParseError {
        file_path: String,
        error: String,
        timestamp: DateTime<Utc>,
    },

    /// AST cache hit
    CacheHit {
        file_path: String,
        timestamp: DateTime<Utc>,
    },

    /// AST cache miss
    CacheMiss {
        file_path: String,
        timestamp: DateTime<Utc>,
    },
}

/// Analysis engine and detector events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisEvent {
    /// Analysis started
    AnalysisStarted {
        run_id: i64,
        target_path: String,
        timestamp: DateTime<Utc>,
    },

    /// Analysis completed
    AnalysisCompleted {
        run_id: i64,
        issues_found: usize,
        files_analyzed: usize,
        duration: std::time::Duration,
        timestamp: DateTime<Utc>,
    },

    /// Detector completed analysis
    DetectorCompleted {
        detector_name: String,
        file_path: String,
        issues_found: usize,
        duration: std::time::Duration,
        timestamp: DateTime<Utc>,
    },

    /// Analysis error occurred
    AnalysisError {
        run_id: Option<i64>,
        error: String,
        context: String,
        timestamp: DateTime<Utc>,
    },

    /// Issue detected
    IssueDetected {
        run_id: i64,
        detector_name: String,
        severity: String,
        file_path: String,
        timestamp: DateTime<Utc>,
    },
}

/// Security audit and validation events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEvent {
    /// Security audit started
    AuditStarted {
        run_id: i64,
        audit_type: String,
        timestamp: DateTime<Utc>,
    },

    /// Security audit completed
    AuditCompleted {
        run_id: i64,
        vulnerabilities_found: usize,
        timestamp: DateTime<Utc>,
    },

    /// Security violation detected
    ViolationDetected {
        run_id: i64,
        violation_type: String,
        severity: String,
        file_path: String,
        timestamp: DateTime<Utc>,
    },

    /// Access validation performed
    AccessValidated {
        file_path: String,
        allowed: bool,
        timestamp: DateTime<Utc>,
    },
}

/// Performance monitoring events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceEvent {
    /// Memory usage update
    MemoryUsageUpdate {
        used_mb: usize,
        available_mb: usize,
        timestamp: DateTime<Utc>,
    },

    /// Performance metric recorded
    MetricRecorded {
        metric_name: String,
        value: f64,
        unit: String,
        timestamp: DateTime<Utc>,
    },

    /// Performance threshold exceeded
    ThresholdExceeded {
        metric_name: String,
        value: f64,
        threshold: f64,
        timestamp: DateTime<Utc>,
    },
}

/// Cache operation events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheEvent {
    /// Cache hit occurred
    CacheHit {
        cache_type: String,
        key: String,
        timestamp: DateTime<Utc>,
    },

    /// Cache miss occurred
    CacheMiss {
        cache_type: String,
        key: String,
        timestamp: DateTime<Utc>,
    },

    /// Cache invalidation
    CacheInvalidated {
        cache_type: String,
        key: String,
        reason: String,
        timestamp: DateTime<Utc>,
    },

    /// Cache statistics update
    StatsUpdate {
        cache_type: String,
        hit_rate: f64,
        total_entries: usize,
        timestamp: DateTime<Utc>,
    },
}

/// Generic AST data representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstData {
    /// Serialized syntax tree (JSON representation)
    pub syntax_tree: serde_json::Value,

    /// AST metadata
    pub metadata: AstMetadata,
}

/// AST metadata information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstMetadata {
    /// Programming language
    pub language: String,

    /// Original file size in bytes
    pub file_size: u64,

    /// Number of AST nodes
    pub node_count: usize,

    /// Maximum tree depth
    pub depth: usize,

    /// Parse success indicator
    pub is_valid: bool,
}

/// Event system errors
#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("No subscribers to receive the event")]
    NoSubscribers,

    #[error("Event channel closed")]
    ChannelClosed,

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Event processing error: {0}")]
    ProcessingError(String),
}

/// Event publisher trait for components that emit events
pub trait EventPublisher {
    /// Publish a domain event
    fn publish_event(&self, event: DomainEvent) -> Result<(), EventError>;
}

/// Event subscriber trait for components that consume events
#[async_trait::async_trait]
pub trait EventSubscriber: Send + Sync {
    /// Handle a received domain event
    async fn handle_event(&self, event: DomainEvent) -> Result<(), EventError>;

    /// Get the event types this subscriber is interested in
    fn interested_events(&self) -> Vec<&'static str>;
}

impl AstEvent {
    /// Create a file parsed event
    pub fn file_parsed(
        file_path: String,
        ast_data: AstData,
        parse_duration: std::time::Duration,
    ) -> Self {
        Self::FileParsed {
            file_path,
            ast_data,
            parse_duration,
            timestamp: Utc::now(),
        }
    }

    /// Create a parse error event
    pub fn parse_error(file_path: String, error: String) -> Self {
        Self::ParseError {
            file_path,
            error,
            timestamp: Utc::now(),
        }
    }
}

impl AnalysisEvent {
    /// Create an analysis started event
    pub fn analysis_started(run_id: i64, target_path: String) -> Self {
        Self::AnalysisStarted {
            run_id,
            target_path,
            timestamp: Utc::now(),
        }
    }

    /// Create an analysis completed event
    pub fn analysis_completed(
        run_id: i64,
        issues_found: usize,
        files_analyzed: usize,
        duration: std::time::Duration,
    ) -> Self {
        Self::AnalysisCompleted {
            run_id,
            issues_found,
            files_analyzed,
            duration,
            timestamp: Utc::now(),
        }
    }

    /// Create an issue detected event
    pub fn issue_detected(
        run_id: i64,
        detector_name: String,
        severity: String,
        file_path: String,
    ) -> Self {
        Self::IssueDetected {
            run_id,
            detector_name,
            severity,
            file_path,
            timestamp: Utc::now(),
        }
    }
}

/// Async event processor for handling events
pub struct EventProcessor {
    event_bus: Arc<EventBus>,
    handlers: Vec<Box<dyn EventSubscriber>>,
}

impl EventProcessor {
    /// Create a new event processor
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self {
            event_bus,
            handlers: Vec::new(),
        }
    }

    /// Add an event handler
    pub fn add_handler(&mut self, handler: Box<dyn EventSubscriber>) {
        self.handlers.push(handler);
    }

    /// Start processing events
    pub async fn start_processing(&mut self) -> Result<(), EventError> {
        let mut receiver = self.event_bus.subscribe();

        loop {
            match receiver.recv().await {
                Ok(event) => {
                    for handler in &self.handlers {
                        if let Err(e) = handler.handle_event(event.clone()).await {
                            error!("Event handling error: {}", e);
                        }
                    }
                }
                Err(broadcast::error::RecvError::Closed) => break,
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    error!("Event processing lagged, some events may be lost");
                    continue;
                }
            }
        }

        Ok(())
    }
}
