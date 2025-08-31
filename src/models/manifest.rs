use serde::{Deserialize, Serialize};
use chrono::Utc;
use std::path::Path;

/// Represents the type of media file.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MediaType {
    /// An image file.
    Image,
    /// A video file.
    Video,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FrameInfo {
    pub timestamp_secs: f64,
    pub pdq_hash: String,
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
        file_path: P,
        media_type: MediaType,
        sha3_hash: String,
        pdq_hash: Option<String>,
        frames: Option<Vec<FrameInfo>>,
        _metadata: Option<serde_json::Value>,
    ) -> Result<Self, std::io::Error> {
        let metadata = std::fs::metadata(&file_path)?;
        let file_name = file_path
            .as_ref()
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(Self {
            media_type,
            file_name,
            file_size: metadata.len(),
            created_at: metadata.created()
                .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339())
                .unwrap_or_else(|_| chrono::Utc::now().to_rfc3339()),
            modified_at: metadata.modified()
                .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339())
                .unwrap_or_else(|_| chrono::Utc::now().to_rfc3339()),
            sha3_256_hash: sha3_hash,
            pdq_hash,
            frames,
            metadata: serde_json::Value::Null,
        })
    }

    /// Serializes the manifest to a pretty-printed JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserializes a `MediaManifest` from a JSON string.
    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }

    /// Verifies the integrity of a file against the manifest.
    ///
    /// This checks the file size and SHA3-256 hash.
    pub fn verify(&self, file_path: &Path) -> Result<bool, Box<dyn std::error::Error>> {
        // Verify file exists and has the same size
        let metadata = std::fs::metadata(file_path)?;
        if metadata.len() != self.file_size {
            return Ok(false);
        }

        // Verify hashes
        let current_sha3 = crate::core::hash::compute_file_hash(file_path)?;
        if current_sha3 != self.sha3_256_hash {
            return Ok(false);
        }

        // For videos, we might want to verify frame hashes as well
        // This is a simplified version - in a real implementation, you'd want to
        // extract frames and verify their hashes

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_manifest_creation() {
        // Create a temporary file
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "test content").unwrap();
        
        let manifest = MediaManifest::new(
            file.path(),
            MediaType::Image,
            "test_hash".to_string(),
            Some("pdq_hash".to_string()),
            None,
            None,
        ).unwrap();
        
        assert_eq!(manifest.media_type, MediaType::Image);
        assert_eq!(manifest.sha3_256_hash, "test_hash");
        assert_eq!(manifest.pdq_hash, Some("pdq_hash".to_string()));
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
