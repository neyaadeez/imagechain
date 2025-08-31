#![forbid(unsafe_code)]
#![warn(
    missing_docs,
    missing_debug_implementations,
    rust_2018_idioms,
    unreachable_pub
)]

//! Main entry point for the ImageChain application

use std::{net::SocketAddr, path::PathBuf};

use axum::{
    http::{header, Method},
    routing::get,
    extract::DefaultBodyLimit,
    Router,
};
use dotenv::dotenv;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    limit::RequestBodyLimitLayer,
    propagate_header::PropagateHeaderLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use imagechain::{create_router, health_check, AppState, Config, Result};

/// Initialize logging and tracing
fn init_logging() {
    // Initialize tracing
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_ansi(true)
        .with_level(true)
        .with_file(true)
        .with_line_number(true);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
}

/// Parse configuration from environment variables
fn parse_config() -> Config {
    let upload_dir = std::env::var("UPLOAD_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("uploads"));

    let max_upload_size = std::env::var("MAX_UPLOAD_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(500 * 1024 * 1024); // Default to 500MB for video files

    Config {
        upload_dir,
        max_upload_size,
        ..Default::default()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file if it exists
    dotenv().ok();
    
    // Initialize logging
    init_logging();
    
    // Parse configuration
    let config = parse_config();
    
    // Create uploads directory if it doesn't exist
    if !config.upload_dir.exists() {
        std::fs::create_dir_all(&config.upload_dir)?;
    }
    
    log::info!("Upload directory: {}", config.upload_dir.display());
    log::info!("Max upload size: {} bytes", config.max_upload_size);
    
    // Store max upload size before moving config
    let max_upload_size = config.max_upload_size;
    
    // Initialize application state
    let state = AppState::with_config(config);
    
    // Set up CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::ACCEPT,
        ])
        .allow_credentials(false);
    
    // Build our application with routes and middleware
    let app = Router::new()
        // Public health check endpoint
        .route("/health", get(health_check))
        // API routes with rate limiting
        .merge(create_router())
        // Add middleware
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(PropagateHeaderLayer::new(header::HeaderName::from_static(
            "x-request-id",
        )))
        .layer(DefaultBodyLimit::max(max_upload_size as usize))
        .layer(RequestBodyLimitLayer::new(max_upload_size as usize))
        .layer(CompressionLayer::new());
    
    // Get the port from environment or use default
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);
    
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    log::info!("Server listening on http://{}", addr);
    
    // Start the server (axum 0.7)
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.with_state(state))
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    
    log::info!("Server shutdown complete");
    Ok(())
}

/// Handle graceful shutdown
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };
    
    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};
        
        let mut sigterm = signal(SignalKind::terminate())
            .expect("Failed to install SIGTERM handler");
        let mut sigint = signal(SignalKind::interrupt())
            .expect("Failed to install SIGINT handler");
        
        tokio::select! {
            _ = sigterm.recv() => {},
            _ = sigint.recv() => {},
        }
    };
    
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    
    tokio::select! {
        _ = ctrl_c => {
            log::info!("Received Ctrl+C, shutting down gracefully...");
        },
        _ = terminate => {
            log::info!("Received termination signal, shutting down gracefully...");
        },
    }
}
