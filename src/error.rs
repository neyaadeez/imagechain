#[cfg(feature = "web")]
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

/// Main error type for the application
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// I/O errors (file operations, etc.)
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Image processing errors
    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),
    
    /// FFmpeg related errors
    #[cfg(feature = "video")]
    #[error("FFmpeg error: {0}")]
    Ffmpeg(#[from] ffmpeg_next::Error),
    
    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// Invalid input parameters
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    /// Resource not found
    #[error("Not found: {0}")]
    NotFound(String),
    
    /// Validation errors
    #[error("Validation error: {0}")]
    Validation(String),
    
    /// Authentication/authorization errors
    #[error("Authentication error: {0}")]
    Auth(String),
    
    /// Rate limiting errors
    #[error("Rate limit exceeded: {message} (retry after {retry_after:?}s)")]
    RateLimit {
        /// The error message.
        message: String,
        /// The number of seconds to wait before retrying.
        retry_after: Option<u64>,
    },
    
    /// Upload errors
    #[error("Upload error: {0}")]
    UploadError(String),
    
    /// Internal server errors
    #[error("Internal server error: {0}")]
    Internal(String),
}

/// Standard error response format
#[derive(Serialize)]
#[derive(Debug)]
pub struct ErrorResponse {
    /// Error code (HTTP status code)
    pub code: u16,
    /// Error message
    pub message: String,
    /// Optional error details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl AppError {
    #[cfg(feature = "web")]
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidInput(_) => StatusCode::BAD_REQUEST,
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Auth(_) => StatusCode::UNAUTHORIZED,
            Self::RateLimit { .. } => StatusCode::TOO_MANY_REQUESTS,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    
    /// Convert the error to a JSON response
    pub fn to_json(&self) -> ErrorResponse {
        #[cfg(feature = "web")]
        let status = self.status_code();
        #[cfg(not(feature = "web"))]
        let status = 500u16;
        
        match self {
            Self::RateLimit { message, retry_after } => {
                let mut response = ErrorResponse {
                    #[cfg(feature = "web")]
                    code: status.as_u16(),
                    #[cfg(not(feature = "web"))]
                    code: status,
                    message: message.clone(),
                    details: None,
                };
                
                // Add retry-after header if available
                if let Some(secs) = retry_after {
                    response.details = Some(format!("Retry after {} seconds", secs));
                }
                
                response
            },
            _ => ErrorResponse {
                #[cfg(feature = "web")]
                code: status.as_u16(),
                #[cfg(not(feature = "web"))]
                code: status,
                message: self.to_string(),
                details: None,
            },
        }
    }
}

#[cfg(feature = "web")]
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let response = self.to_json();
        
        (status, Json(response)).into_response()
    }
}

// Implement From for common error types
#[cfg(feature = "web")]
impl From<axum::BoxError> for AppError {
    fn from(err: axum::BoxError) -> Self {
        AppError::Internal(err.to_string())
    }
}

impl From<tokio::task::JoinError> for AppError {
    fn from(err: tokio::task::JoinError) -> Self {
        AppError::Internal(format!("Task join error: {}", err))
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

#[cfg(feature = "web")]
impl From<axum::extract::multipart::MultipartError> for AppError {
    fn from(err: axum::extract::multipart::MultipartError) -> Self {
        AppError::UploadError(err.to_string())
    }
}

impl From<Box<dyn std::error::Error>> for AppError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        AppError::Internal(err.to_string())
    }
}

#[cfg(feature = "embeddings")]
impl From<tch::TchError> for AppError {
    fn from(err: tch::TchError) -> Self {
        AppError::Internal(format!("PyTorch error: {}", err))
    }
}

/// Result type alias for the application
pub type Result<T> = std::result::Result<T, AppError>;

/// Extension trait for working with Results
pub trait ResultExt<T> {
    /// Add context to an error
    fn context<C>(self, context: C) -> Result<T>
    where
        C: std::fmt::Display + Send + Sync + 'static;
    
    /// Add context to an error if the result is an error
    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: std::fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C;
}

impl<T, E> ResultExt<T> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn context<C>(self, context: C) -> Result<T>
    where
        C: std::fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|e| AppError::Internal(format!("{}: {}", context, e)))
    }
    
    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: std::fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.map_err(|e| {
            let context = f();
            AppError::Internal(format!("{}: {}", context, e))
        })
    }
}
