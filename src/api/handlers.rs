use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::{
    core::{hash, video::FrameExtractor},
    error::{AppError, Result},
    models::manifest::{MediaManifest, MediaType},
    AppState,
};

use super::responses::ApiResponse;

pub async fn upload_file(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse> {
    let mut file_name = None;
    let mut temp_path = None;

    // Process the multipart form data
    while let Some(field) = multipart.next_field().await? {
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
            let content = field.bytes().await?;
            temp_file.write_all(&content).await?;
            
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
        return Err(AppError::UploadError("Unsupported file type".to_string()));
    };
    
    // Compute file hash
    let file_hash = hash::compute_file_hash(&temp_path)?;
    
    // Process based on media type
    let manifest = match media_type {
        MediaType::Image => {
            // Process image
            let img = image::open(&temp_path)?;
            let pdq_hash = hash::compute_pdq_hash(&img)?;
            
            MediaManifest::new(
                &temp_path,
                MediaType::Image,
                file_hash,
                Some(pdq_hash),
                None,
                None,
            )?
        }
        MediaType::Video => {
            // Process video
            let extractor = FrameExtractor::new(&temp_path, 5.0); // Extract frame every 5 seconds
            let mut frames = Vec::new();
            
            extractor.extract_frames(|frame, timestamp_secs| {
                // Convert frame to image
                let img = image::RgbImage::from_raw(
                    frame.width(),
                    frame.height(),
                    frame.data(0).unwrap().to_vec(),
                )
                .ok_or_else(|| anyhow::anyhow!("Failed to create image from frame"))?;
                
                let dynamic_img = image::DynamicImage::ImageRgb8(img);
                let pdq_hash = hash::compute_pdq_hash(&dynamic_img)?;
                
                frames.push(crate::models::manifest::FrameInfo {
                    timestamp_secs,
                    pdq_hash,
                    embedding: None, // In a real implementation, you'd compute embeddings here
                });
                
                Ok(())
            })?;
            
            MediaManifest::new(
                &temp_path,
                MediaType::Video,
                file_hash,
                None, // No single PDQ hash for video
                Some(frames),
                None,
            )?
        }
    };
    
    // Clean up temp file
    let _ = std::fs::remove_file(&temp_path);
    
    // In a real application, you'd save the manifest to a database
    
    Ok(Json(ApiResponse::success(manifest)))
}

pub async fn verify_manifest(
    State(_state): State<Arc<AppState>>,
    Json(manifest): Json<MediaManifest>,
) -> Result<impl IntoResponse> {
    // In a real application, you'd look up the file path based on the manifest
    // For this example, we'll assume the file is in the uploads directory
    let uploads_dir = std::env::current_dir()?.join("uploads");
    let file_path = uploads_dir.join(&manifest.file_name);
    
    let is_valid = manifest.verify(&file_path)?;
    
    Ok(Json(ApiResponse::success(json!({ "is_valid": is_valid }))))
}
