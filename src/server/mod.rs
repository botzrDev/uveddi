//! HTTP server module for Uveddi service endpoints
//!
//! This module implements the HTTP API using Axum framework

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
