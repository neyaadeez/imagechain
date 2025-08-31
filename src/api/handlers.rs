use axum::{
    extract::{Multipart, State},
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

use super::responses::ApiResponse;

/// Handles file uploads, processing them based on media type.
///
/// This endpoint accepts multipart form data with a "file" field.
/// It computes cryptographic and perceptual hashes for images and videos,
/// and returns a `MediaManifest` upon success.
pub async fn upload_file(
    State(_state): State<Arc<AppState>>,
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

    let manifest = match media_type {
        MediaType::Image => {
            // Process image
            let img = image::open(&temp_path)?;
            let pdq_hash = hash::compute_pdq_hash(&img)?;

            MediaManifest::new(
                new_file_name,
                &temp_path,
                MediaType::Image,
                file_hash,
                Some(pdq_hash),
                None,
                None,
            )?
        }
        MediaType::Video => {
            // For now, just create a basic video manifest without frame extraction
            // In a full implementation, you'd extract frames here
            MediaManifest::new(
                new_file_name,
                &temp_path,
                MediaType::Video,
                file_hash,
                None, // No single PDQ hash for video
                None, // No frames for now
                None,
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

