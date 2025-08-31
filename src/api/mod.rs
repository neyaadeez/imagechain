//! API module for handling HTTP requests and responses

#[cfg(feature = "web")]
pub(crate) mod handlers;
#[cfg(feature = "web")]
pub(crate) mod responses;

#[cfg(feature = "web")]
use axum::{
    routing::{get, post},
    Router,
};
#[cfg(feature = "web")]
use std::sync::Arc;
#[cfg(feature = "web")]
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
#[cfg(feature = "web")]
use crate::state::AppState;

#[cfg(feature = "web")]
pub(crate) use handlers::*;

#[cfg(feature = "web")]
/// Create the application router with all routes
pub fn create_router() -> Router<Arc<AppState>> {
    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Public health check (no rate limiting)
        .route("/api/health", get(health_check))
        // Upload endpoint
        .route("/api/upload", post(upload_file))
        // Verification endpoint
        .route("/api/verify", post(verify_manifest))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}

#[cfg(feature = "web")]
/// Health check endpoint
pub async fn health_check() -> &'static str {
    "OK"
}

