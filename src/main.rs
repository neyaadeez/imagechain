use imagechain::{init, AppState, Result};

use std::{net::SocketAddr, sync::Arc};

use axum::{
    routing::post,
    Router,
};
use tower_http::cors::{Any, CorsLayer};

use crate::api::handlers::{upload_file, verify_manifest};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the application
    init()?;
    
    // Create uploads directory if it doesn't exist
    let uploads_dir = std::env::current_dir()?.join("uploads");
    if !uploads_dir.exists() {
        std::fs::create_dir_all(&uploads_dir)?;
    }

    // Initialize application state
    let state = AppState::new();

    // Set up CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build our application with routes
    let app = Router::new()
        .route("/api/upload", post(upload_file))
        .route("/api/verify", post(verify_manifest))
        .layer(cors)
        .with_state(state);

    // Set up the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    log::info!("Server listening on {}", addr);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
