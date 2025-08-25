//! HTTP server module for Uveddi service endpoints
//!
//! This module implements the HTTP API using Axum framework, providing
//! health monitoring and analysis service endpoints for external integrations.
//!
//! # Architecture
//!
//! The server uses a shared state pattern with Arc<Mutex<T>> for thread-safe
//! access to monitoring components. All endpoints are async and return
//! appropriate HTTP status codes and JSON responses.
//!
//! ## Shared State Pattern
//!
//! - **AppState**: Contains shared resources accessible across all handlers
//! - **Arc<Mutex<T>>**: Provides thread-safe access to mutable state
//! - **Clone**: State is cloned efficiently using Arc for each request
//!
//! # Available Endpoints
//!
//! ## Health Monitoring
//!
//! - `GET /health` - Basic health check (returns 200/503)
//!   - Returns simple "OK"/"DEGRADED" status
//!   - Used by load balancers and monitoring systems
//!   - Low overhead, minimal processing
//!
//! - `GET /health/detailed` - Detailed health status with metrics
//!   - Returns complete HealthStatus JSON object
//!   - Includes component-level health information
//!   - Used for debugging and detailed monitoring
//!
//! - `GET /health/alerts` - Current active alerts and warnings
//!   - Returns array of Alert objects
//!   - Includes severity levels and timestamps
//!   - Used for alerting systems and dashboards
//!
//! # Error Handling
//!
//! All endpoints handle errors gracefully and return appropriate HTTP status
//! codes. Internal errors are logged but not exposed to clients.
//!
//! ## Status Code Mapping
//!
//! - `200 OK` - Healthy service with no issues
//! - `503 Service Unavailable` - Degraded service or critical issues
//! - `500 Internal Server Error` - Unexpected errors (with logging)
//!
//! # Usage Example
//!
//! ```rust,no_run
//! use std::sync::Arc;
//! use tokio::sync::Mutex;
//! use uveddi::server::run_server;
//! use uveddi::resilience::health::HealthMonitor;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let health_monitor = Arc::new(Mutex::new(HealthMonitor::new()));
//!     run_server(health_monitor).await?;
//!     Ok(())
//! }
//! ```
//!
//! # Integration Examples
//!
//! ## Kubernetes Health Checks
//!
//! ```yaml
//! livenessProbe:
//!   httpGet:
//!     path: /health
//!     port: 3000
//!   periodSeconds: 30
//!
//! readinessProbe:
//!   httpGet:
//!     path: /health/detailed
//!     port: 3000
//!   periodSeconds: 10
//! ```
//!
//! ## Prometheus Monitoring
//!
//! ```bash
//! # Monitor detailed health status
//! curl http://localhost:3000/health/detailed | jq .score
//!
//! # Check for active alerts
//! curl http://localhost:3000/health/alerts | jq 'length'
//! ```

use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use std::sync::Arc;
use tokio::sync::Mutex;
use uveddi::resilience::health::{Alert, HealthMonitor, HealthStatus};

/// Placeholder documentation for public items
/// Shared state for HTTP handlers
#[derive(Clone)]
#[allow(dead_code)]
pub struct AppState {
    pub health_monitor: Arc<Mutex<HealthMonitor>>,
}

/// Initialize the HTTP server
#[allow(dead_code)]
pub async fn run_server(
    health_monitor: Arc<Mutex<HealthMonitor>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let app_state = AppState { health_monitor };

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/health/detailed", get(detailed_health_handler))
        .route("/health/alerts", get(alerts_handler))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    println!("Health monitoring server running on http://localhost:3000");
    axum::serve(listener, app).await?;
    Ok(())
}

/// Basic health endpoint
#[allow(dead_code)]
async fn health_handler(State(state): State<AppState>) -> (StatusCode, &'static str) {
    let monitor = state.health_monitor.lock().await;
    match monitor.get_status().await.score {
        100 => (StatusCode::OK, "OK"),
        _ => (StatusCode::SERVICE_UNAVAILABLE, "DEGRADED"),
    }
}

/// Detailed health status endpoint
#[allow(dead_code)]
async fn detailed_health_handler(State(state): State<AppState>) -> Json<HealthStatus> {
    let monitor = state.health_monitor.lock().await;
    Json(monitor.get_detailed_status().await)
}

/// Active alerts endpoint
#[allow(dead_code)]
async fn alerts_handler(State(state): State<AppState>) -> Json<Vec<Alert>> {
    let monitor = state.health_monitor.lock().await;
    Json(monitor.get_alerts().await)
}
