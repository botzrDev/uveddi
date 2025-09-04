//! Health check endpoints for the REST API

use axum::response::IntoResponse;
use axum::{http::StatusCode, Json};
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    timestamp: String,
    service: String,
}

/// Health check endpoint
pub async fn health_check() -> impl IntoResponse {
    let response = HealthResponse {
        status: "healthy".to_string(),
        timestamp: Utc::now().to_rfc3339(),
        service: "uveddi-api".to_string(),
    };
    (StatusCode::OK, Json(response))
}