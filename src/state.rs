use std::path::PathBuf;
use std::sync::Arc;

use crate::core::embeddings::EmbeddingModel;

/// Configuration for the application
#[derive(Clone, Debug)]
pub struct Config {
    /// Base directory for file uploads
    pub upload_dir: PathBuf,
    /// Maximum file size in bytes
    pub max_upload_size: u64,
    /// Allowed file extensions for uploads
    pub allowed_extensions: Vec<String>,
    /// Video processing configuration
    pub video: VideoConfig,
    /// Embedding model configuration
    pub embedding_model: EmbeddingModel,
}

/// Video processing configuration
#[derive(Clone, Debug)]
pub struct VideoConfig {
    /// Interval in seconds between extracted frames
    pub frame_interval: u64,
    /// Target width for resizing frames (maintains aspect ratio)
    pub target_width: u32,
    /// Target height for resizing frames (maintains aspect ratio)
    pub target_height: u32,
    /// Video codec for processed videos
    pub codec: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            upload_dir: PathBuf::from("uploads"),
            max_upload_size: 100 * 1024 * 1024, // 100MB
            allowed_extensions: vec!["jpg", "jpeg", "png", "webp", "mp4", "mov", "avi"]
                .into_iter()
                .map(String::from)
                .collect(),
            video: VideoConfig::default(),
            embedding_model: EmbeddingModel::default(),
        }
    }
}

impl Default for VideoConfig {
    fn default() -> Self {
        Self {
            frame_interval: 5, // 5 seconds
            target_width: 640,
            target_height: 360,
            codec: String::from("libx264"),
        }
    }
}

/// Application state that can be shared across handlers
#[derive(Clone)]
#[derive(Debug)]
pub struct AppState {
    /// Application configuration
    pub config: Config,
    /// Shared embedding model instance
    pub embedding_model: EmbeddingModel,
}

impl AppState {
    /// Create a new application state with default configuration
    pub fn new() -> Arc<Self> {
        let config = Config::default();
        let embedding_model = config.embedding_model.clone();
        
        Arc::new(Self {
            config,
            embedding_model,
        })
    }
    
    /// Create a new application state with custom configuration
    pub fn with_config(config: Config) -> Arc<Self> {
        let embedding_model = config.embedding_model.clone();
        
        Arc::new(Self {
            config,
            embedding_model,
        })
    }
}
