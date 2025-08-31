use axum::{
    extract::{Multipart, State, Query},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::{
    core::hash,
    error::{AppError, Result},
    models::manifest::{MediaManifest, MediaType},
    AppState,
};
use serde::Deserialize;

use super::responses::ApiResponse;

/// Handles file uploads, processing them based on media type.
///
/// This endpoint accepts multipart form data with a "file" field.
/// It computes cryptographic and perceptual hashes for images and videos,
/// and returns a `MediaManifest` upon success.
#[derive(Debug, Deserialize, Default)]
pub struct UploadParams {
    pub include_embeddings: Option<bool>,
    pub frame_interval_secs: Option<f64>,
    pub max_frames: Option<usize>,
    pub extract_frames: Option<bool>,
}

/// Upload endpoint: accepts multipart form with a `file` field and optional query params.
///
/// Query parameters:
/// - `include_embeddings` (bool, default: false) — include image/frame embeddings.
/// - `extract_frames` (bool, default: true; video only) — enable/disable frame extraction.
/// - `frame_interval_secs` (f64, default: 1.0; video only) — seconds between frames.
/// - `max_frames` (usize, optional; video only) — cap number of processed frames; if omitted, the full video is processed.
///
/// Returns a `MediaManifest` JSON with hashes, and optional embeddings
/// (for videos, embeddings are per frame; for images, embedding is in `metadata.embedding`).
pub async fn upload_file(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<UploadParams>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse> {
    let mut file_name = None;
    let mut temp_path = None;

    // Process the multipart form data
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::UploadError(format!("Failed to read multipart field: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();
        
        if name == "file" {
            let file_name_field = field.file_name()
                .ok_or_else(|| AppError::UploadError("No filename provided".to_string()))?
                .to_string();
            
            let extension = std::path::Path::new(&file_name_field)
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();
            
            let temp_file_name = format!("{}.{}", Uuid::new_v4(), extension);
            let temp_file_path = std::env::temp_dir().join(&temp_file_name);
            
            let mut temp_file = File::create(&temp_file_path).await?;
            // Stream the field content to disk to avoid buffering the whole file in memory
            let mut field_stream = field;
            while let Some(chunk) = field_stream
                .chunk()
                .await
                .map_err(|e| AppError::UploadError(format!("Failed to read file content: {}", e)))?
            {
                temp_file.write_all(&chunk).await?;
            }
            temp_file.flush().await?;
            
            file_name = Some(file_name_field);
            temp_path = Some(temp_file_path);
        }
    }

    let temp_path = temp_path.ok_or_else(|| AppError::UploadError("No file provided".to_string()))?;
    let file_name = file_name.unwrap_or_else(|| "unknown".to_string());
    
    // Process the file based on its type
    let extension = std::path::Path::new(&file_name)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    let media_type = if ["jpg", "jpeg", "png", "gif", "bmp", "webp"].contains(&extension.as_str()) {
        MediaType::Image
    } else if ["mp4", "avi", "mov", "mkv", "webm"].contains(&extension.as_str()) {
        MediaType::Video
    } else {
        MediaType::Other
    };
    
    // Compute file hash
    let file_hash = hash::compute_file_hash(&temp_path)?;
    
    // Process based on media type
    let new_file_name = temp_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Resolve flags with defaults
    let include_embeddings = params.include_embeddings.unwrap_or(false);
    let frame_interval = params.frame_interval_secs.unwrap_or(1.0);
    let frame_interval = if frame_interval > 0.0 { frame_interval } else { 1.0 };
    let max_frames = params.max_frames;
    let extract_frames_flag = params.extract_frames.unwrap_or(true);

    let manifest = match media_type {
        MediaType::Image => {
            // Process image
            let img = image::open(&temp_path)?;
            let pdq_hash = hash::compute_pdq_hash(&img)?;

            // Optional embedding for image stored in metadata
            let mut metadata: Option<serde_json::Value> = None;
            if include_embeddings {
                let embedding_opt = crate::core::embeddings::compute_image_embedding(&img).await?;
                if let Some(embedding) = embedding_opt {
                    metadata = Some(serde_json::json!({ "embedding": embedding }));
                }
            }

            MediaManifest::new(
                new_file_name,
                &temp_path,
                MediaType::Image,
                file_hash,
                Some(pdq_hash),
                None,
                metadata,
            )?
        }
        MediaType::Video => {
            // Extract frames and compute PDQ per frame, with optional embeddings
            let mut frames_info: Vec<crate::models::manifest::FrameInfo> = Vec::new();
            if extract_frames_flag {
                let mut frames_images = crate::core::video::extract_frames(&temp_path, frame_interval)?;
                if let Some(limit) = max_frames {
                    if frames_images.len() > limit {
                        frames_images.truncate(limit);
                    }
                }
                for (i, img) in frames_images.iter().enumerate() {
                    let pdq = hash::compute_pdq_hash(img)?;
                    let embedding = if include_embeddings {
                        crate::core::embeddings::compute_image_embedding(img).await?
                    } else {
                        None
                    };
                    frames_info.push(crate::models::manifest::FrameInfo {
                        timestamp_secs: (i as f64) * frame_interval,
                        pdq_hash: pdq,
                        embedding,
                    });
                }
            }

            // Include basic metadata
            let metadata = serde_json::json!({
                "frame_interval_secs": frame_interval,
                "frame_count": frames_info.len(),
                "max_frames": max_frames,
                "extracted_frames": extract_frames_flag,
                "original_extension": std::path::Path::new(&file_name)
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
            });

            MediaManifest::new(
                new_file_name,
                &temp_path,
                MediaType::Video,
                file_hash,
                None, // No single PDQ hash for video
                Some(frames_info),
                Some(metadata),
            )?
        }
        MediaType::Other => {
            // Create a basic manifest for other file types
            MediaManifest::new(
                new_file_name,
                &temp_path,
                MediaType::Other,
                file_hash,
                None,
                None,
                None,
            )?
        }
    };
    
    // Create uploads directory if it doesn't exist
    let uploads_dir = std::env::current_dir()?.join("uploads");
    tokio::fs::create_dir_all(&uploads_dir).await?;

    // Move the file to the uploads directory
    let new_file_name = manifest.file_name.clone();
    let dest_path = uploads_dir.join(&new_file_name);
    tokio::fs::rename(&temp_path, dest_path).await?;
    
    // In a real application, you'd save the manifest to a database
    
    Ok(Json(ApiResponse::success(manifest)))
}

/// Verifies the integrity of a file against a provided `MediaManifest`.
///
/// This endpoint checks if a file on disk matches the metadata and hashes
/// stored in the manifest.
pub async fn verify_manifest(
    State(_state): State<Arc<AppState>>,
    Json(manifest): Json<MediaManifest>,
) -> Result<impl IntoResponse> {
    // In a real application, you'd look up the file path based on the manifest
    // For this example, we'll assume the file is in the uploads directory
    let uploads_dir = std::env::current_dir()?.join("uploads");
    let file_path = uploads_dir.join(&manifest.file_name);
    
    let is_valid = manifest.verify(&file_path)?;
    
    Ok(Json(ApiResponse::success(serde_json::json!({ "is_valid": is_valid }))))
}

