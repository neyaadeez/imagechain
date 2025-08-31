use serde::{Deserialize, Serialize};
use std::path::Path;
use log::{info, warn};
use crate::error::Result;

/// Represents the type of media file.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)] // <-- add PartialEq and Eq
pub enum MediaType {
    /// Represents an image file.
    Image,
    /// Represents a video file.
    Video,
    /// Represents any other file type.
    Other,
}


/// Contains information about a single frame extracted from a video.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FrameInfo {
    /// The timestamp of the frame in seconds from the start of the video.
    pub timestamp_secs: f64,
    /// The PDQ perceptual hash of the frame.
    pub pdq_hash: String,
    /// The deep learning embedding of the frame, if available.
    pub embedding: Option<Vec<f32>>,
}

/// A manifest containing metadata and hashes for a media file.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MediaManifest {
    /// The type of media (e.g., Image or Video).
    pub media_type: MediaType,
    /// The original name of the file.
    pub file_name: String,
    /// The size of the file in bytes.
    pub file_size: u64,
    /// The creation timestamp of the file (RFC 3339 format).
    pub created_at: String,
    /// The last modification timestamp of the file (RFC 3339 format).
    pub modified_at: String,
    /// The SHA3-256 hash of the file content.
    pub sha3_256_hash: String,
    /// The PDQ perceptual hash of the image (for images only).
    pub pdq_hash: Option<String>,
    /// Information about extracted frames (for videos only).
    pub frames: Option<Vec<FrameInfo>>,
    /// Arbitrary JSON metadata associated with the file.
    pub metadata: serde_json::Value,
}

impl MediaManifest {
    /// Creates a new `MediaManifest` from a file path and associated data.
    pub fn new<P: AsRef<Path>>(
        file_name: String,
        file_path: P,
        media_type: MediaType,
        sha3_256_hash: String,
        pdq_hash: Option<String>,
        frames: Option<Vec<FrameInfo>>,
        metadata: Option<serde_json::Value>,
    ) -> Result<Self> {
        let file_metadata = std::fs::metadata(file_path)?;
        
        // Handle creation time - fallback to modified time if creation time is not available
        let created_at: chrono::DateTime<chrono::Utc> = file_metadata
            .created()
            .or_else(|_| file_metadata.modified())
            .unwrap_or_else(|_| std::time::SystemTime::now())
            .into();
        
        let modified_at: chrono::DateTime<chrono::Utc> = file_metadata.modified()?.into();
        let file_size = file_metadata.len();

        Ok(Self {
            media_type,
            file_name,
            file_size,
            created_at: created_at.to_rfc3339(),
            modified_at: modified_at.to_rfc3339(),
            sha3_256_hash,
            pdq_hash,
            frames,
            metadata: metadata.unwrap_or(serde_json::Value::Null),
        })
    }

    /// Serializes the manifest to a pretty-printed JSON string.
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Deserializes a `MediaManifest` from a JSON string.
    pub fn from_json(json_str: &str) -> Result<Self> {
        Ok(serde_json::from_str(json_str)?)
    }

    /// Verifies the integrity of a file against the manifest.
    ///
    /// This checks the file size and SHA3-256 hash.
    pub fn verify<P: AsRef<Path>>(&self, file_path: P) -> Result<bool> {
        let path = file_path.as_ref();
        info!("Verifying file at path: {:?}", path);

        if !path.exists() {
            warn!("Verification failed: path does not exist.");
            return Ok(false);
        }

        if !path.is_file() {
            warn!("Verification failed: path is not a file.");
            return Ok(false);
        }

        let metadata = std::fs::metadata(path)?;
        if metadata.len() != self.file_size {
            warn!(
                "Verification failed: size mismatch. Expected: {}, Found: {}",
                self.file_size,
                metadata.len()
            );
            return Ok(false);
        }

        let file_hash = crate::core::hash::compute_file_hash(path)?;
        if file_hash != self.sha3_256_hash {
            warn!(
                "Verification failed: SHA3 hash mismatch. Expected: {}, Found: {}",
                self.sha3_256_hash,
                file_hash
            );
            return Ok(false);
        }

        if self.media_type == MediaType::Image {
            if let Some(pdq_hash) = &self.pdq_hash {
                let img = image::open(path)?;
                let computed_pdq_hash = crate::core::hash::compute_pdq_hash(&img)?;
                if pdq_hash != &computed_pdq_hash {
                    warn!(
                        "Verification failed: PDQ hash mismatch. Expected: {}, Found: {}",
                        pdq_hash,
                        computed_pdq_hash
                    );
                    return Ok(false);
                }
            }
        }

        info!("Verification successful.");
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_manifest_verification() {
        // Create a temporary image file
        let file = tempfile::Builder::new().suffix(".png").tempfile().unwrap();
        let img = image::RgbImage::new(10, 10);
        img.save(file.path()).unwrap();

        let file_hash = crate::core::hash::compute_file_hash(file.path()).unwrap();
        let pdq_hash = crate::core::hash::compute_pdq_hash(&image::open(file.path()).unwrap()).unwrap();

        let manifest = MediaManifest::new(
            "test_image.png".to_string(),
            file.path(),
            MediaType::Image,
            file_hash,
            Some(pdq_hash),
            None,
            None,
        ).unwrap();

        // Verification should pass
        assert!(manifest.verify(file.path()).unwrap());
    }
    
    #[test]
    fn test_manifest_serialization() {
        let manifest = MediaManifest {
            media_type: MediaType::Image,
            file_name: "test.jpg".to_string(),
            file_size: 1024,
            created_at: Utc::now().to_rfc3339(),
            modified_at: Utc::now().to_rfc3339(),
            sha3_256_hash: "test_hash".to_string(),
            pdq_hash: Some("pdq_hash".to_string()),
            frames: None,
            metadata: serde_json::json!({}),
        };
        
        let json = manifest.to_json().unwrap();
        let deserialized: MediaManifest = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.file_name, "test.jpg");
        assert_eq!(deserialized.sha3_256_hash, "test_hash");
    }
}
