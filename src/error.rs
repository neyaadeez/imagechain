use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("File upload error: {0}")]
    UploadError(String),
    
    #[error("File processing error: {0}")]
    ProcessingError(String),
    
    #[error("Hashing error: {0}")]
    HashingError(String),
    
    #[error("Video processing error: {0}")]
    VideoError(String),
    
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Image processing error: {0}")]
    ImageError(#[from] image::ImageError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::UploadError(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::ProcessingError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::HashingError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::VideoError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::IoError(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            AppError::ImageError(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}

// Helper type for handler results
type Result<T> = std::result::Result<T, AppError>;
