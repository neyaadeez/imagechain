#![doc(html_root_url = "https://docs.rs/imagechain/0.1.0")]
#![forbid(unsafe_code)]
#![warn(
    missing_docs,
    missing_debug_implementations,
    rust_2018_idioms,
    unreachable_pub
)]

//! # ImageChain
//! 
//! A high-performance Rust library and service for processing, hashing, and comparing images and videos
//! with cryptographic and perceptual hashing, plus deep learning-based embeddings.
//!
//! ## Features
//!
//! - **Image Processing**: Load, transform, and process images in various formats
//! - **Video Processing**: Extract frames, process videos, and generate previews
//! - **Cryptographic Hashing**: SHA-3/Keccak for file integrity verification
//! - **Perceptual Hashing**: PDQ (Pretty Damn Quick) for similar image detection
//! - **Deep Learning**: Generate and compare image embeddings using pre-trained models
//! - **Manifests**: Generate and verify manifests containing all hashes and metadata
//! - **Web API**: HTTP server with endpoints for processing and verification
//!
//! ## Quick Start
//!
//! Add to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! imagechain = { version = "0.1", features = ["full"] }
//! ```
//!
//! Basic usage:
//! ```rust,no_run
//! use imagechain::{process_image, MediaManifest, Result};
//! use std::path::Path;
//!
//! fn main() -> Result<()> {
//!     let manifest = process_image(Path::new("path/to/image.jpg"))?;
//!     println!("Generated manifest: {:?}", manifest);
//!     Ok(())
//! }
//! ```

// Internal modules
pub mod api;
pub mod core;
/// Defines the application's error types and result aliases.
pub mod error;
pub mod models;
mod state;
mod utils;

// Public API exports
pub use crate::{
    error::{AppError, Result, ResultExt},
    models::manifest::{MediaManifest, MediaType},
};

#[cfg(feature = "web")]
pub use crate::{
    api::{create_router, health_check, handlers::{upload_file, verify_manifest}},
    state::{AppState, Config},
};

#[cfg(feature = "hashing")]
pub use crate::core::hash::{self, compute_sha3_256, compute_pdq_hash, compute_file_hash, sha3_256};

#[cfg(feature = "embeddings")]
pub use crate::core::embeddings::{self, EmbeddingModel};

#[cfg(feature = "video")]
pub use crate::{
    core::video::{self, extract_frames, process_video, check_ffmpeg_installed},
    state::VideoConfig,
};

/// Initialize the application with default settings
///
/// This function sets up logging and checks for required system dependencies.
/// It should be called early in the application startup process.
///
/// # Errors
///
/// Returns an error if logging initialization fails.
///
/// # Example
///
/// ```no_run
/// use imagechain::init;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     init()?;
///     // Application code here
///     Ok(())
/// }
/// ```
pub fn init() -> Result<()> {
    // Initialize logging with sensible defaults
    let env = env_logger::Env::default()
        .default_filter_or("info")
        .default_write_style_or("auto");
    
    env_logger::Builder::from_env(env)
        .format_timestamp_millis()
        .format_module_path(false)
        .format_target(false)
        .init();
    
    log::info!("Initializing ImageChain");
    
    // Check for required system dependencies
    #[cfg(feature = "video")]
    if let Err(e) = video::check_ffmpeg_installed() {
        log::warn!("FFmpeg is not installed or not in PATH: {}", e);
        log::warn!("Video processing will not be available");
    }
    
    // Initialize the embedding model if needed
    #[cfg(feature = "embeddings")]
    {
        use std::sync::Once;
        static INIT: Once = Once::new();
        
        INIT.call_once(|| {
            if let Err(e) = EmbeddingModel::global() {
                log::error!("Failed to initialize embedding model: {}", e);
            }
        });
    }
    
    log::info!("ImageChain initialized successfully");
    Ok(())
}

/// Process an image file and generate a manifest
///
/// This is a convenience function that handles the entire processing pipeline
/// for a single image file.
///
/// # Arguments
///
/// * `path` - Path to the image file
///
/// # Errors
///
/// Returns an error if the file cannot be read, processed, or if any hashing operation fails.
pub fn process_image<P: AsRef<std::path::Path>>(path: P) -> Result<MediaManifest> {
    let path = path.as_ref();
    log::debug!("Processing image: {}", path.display());
    
    // Read the file
    let data = std::fs::read(path)
        .map_err(|e| error::AppError::Io(e))?;
    
    // Compute hashes
    let sha3_hash = crate::core::hash::compute_sha3_256(&data)?;
    // Load image for PDQ hash computation
    let image = image::load_from_memory(&data)?;
    let pdq_hash = crate::core::hash::compute_pdq_hash(&image)?;
    
    // Generate embeddings if feature is enabled
    #[cfg(feature = "embeddings")]
    let _embedding = {
        let model = EmbeddingModel::global()?;
        model.compute_embedding(&image)?
    };
    
    #[cfg(not(feature = "embeddings"))]
    let embedding: Option<Vec<f32>> = None;
    
    // Create and return
    let file_name = path.file_name()
        .and_then(|n| n.to_str())
        .map(String::from)
        .unwrap_or_else(|| "unknown".to_string());
    
    let file_size = data.len() as u64;
    let now = chrono::Utc::now().to_rfc3339();
    
    Ok(MediaManifest {
        media_type: MediaType::Image,
        file_name,
        file_size,
        created_at: now.clone(),
        modified_at: now,
        sha3_256_hash: sha3_hash,
        pdq_hash: Some(pdq_hash),
        frames: None,
        metadata: serde_json::Value::Null,
    })
}

/// Process a video file and generate a manifest with frame information
///
/// # Arguments
///
/// * `path` - Path to the video file
///
/// # Errors
///
/// Returns an error if the file cannot be read, processed, or if any hashing operation fails.
#[cfg(feature = "video")]
pub fn process_video_file<P: AsRef<std::path::Path>>(path: P) -> Result<MediaManifest> {
    
    let path = path.as_ref();
    log::debug!("Processing video: {}", path.display());
    
    // Read the file
    let data = std::fs::read(path)
        .map_err(|e| error::AppError::Io(e))?;
    
    // Compute hashes
    let sha3_hash = hash::compute_sha3_256(&data)?;
    
    // Extract frames and process them
    let _frames_dir = tempfile::tempdir()?;
    let frames = video::extract_frames(path, 1.0)?; // 1 frame per second
    
    // Process each frame
    let mut frame_manifests = Vec::new();
    for (i, frame) in frames.iter().enumerate() {
        // Convert DynamicImage to FrameInfo
        let pdq_hash = hash::compute_pdq_hash(frame)?;
        let frame_info = crate::models::manifest::FrameInfo {
            timestamp_secs: i as f64, // Simple timestamp based on frame index
            pdq_hash,
            embedding: None,
        };
        frame_manifests.push(frame_info);
    }
    
    // Create and return the manifest
    let file_name = path.file_name()
        .and_then(|n| n.to_str())
        .map(String::from)
        .unwrap_or_else(|| "unknown".to_string());
    
    let now = chrono::Utc::now().to_rfc3339();
    
    let manifest = MediaManifest {
        media_type: MediaType::Video,
        file_name,
        file_size: data.len() as u64,
        created_at: now.clone(),
        modified_at: now,
        sha3_256_hash: sha3_hash,
        pdq_hash: None, // Videos don't have a single PDQ hash
        frames: Some(frame_manifests),
        metadata: serde_json::Value::Null,
    };
    
    Ok(manifest)
}
