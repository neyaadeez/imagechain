//! ImageChain - A Rust library for processing images and videos with cryptographic and perceptual hashing
//!
//! # Features
//! - Image and video processing
//! - Cryptographic hashing (SHA3-256)
//! - Perceptual hashing (PDQ)
//! - Deep learning-based image embeddings
//! - Manifest generation and verification

pub mod api;
pub mod core;
pub mod error;
pub mod models;
pub mod state;
pub mod utils;

// Re-exports for commonly used types
pub use error::Result;
pub use models::manifest::MediaManifest;
pub use state::AppState;
pub use core::embeddings::EmbeddingModel;

/// Initialize the application
pub fn init() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    // Check for required system dependencies
    if let Err(e) = core::video::check_ffmpeg_installed() {
        log::warn!("FFmpeg is not installed or not in PATH: {}", e);
        log::warn!("Video processing will not be available");
    }
    
    Ok(())
}
