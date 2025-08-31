use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MediaType {
    Image,
    Video,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FrameInfo {
    pub timestamp_secs: f64,
    pub pdq_hash: String,
    pub embedding: Option<Vec<f32>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MediaManifest {
    pub media_type: MediaType,
    pub file_name: String,
    pub file_size: u64,
    pub created_at: String,
    pub modified_at: String,
    pub sha3_256_hash: String,
    pub pdq_hash: Option<String>,
    pub frames: Option<Vec<FrameInfo>>,
    pub metadata: serde_json::Value,
}

impl MediaManifest {
    pub fn new<P: AsRef<Path>>(
        file_path: P,
        media_type: MediaType,
        sha3_hash: String,
        pdq_hash: Option<String>,
        frames: Option<Vec<FrameInfo>>,
        metadata: Option<serde_json::Value>,
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
            created_at: metadata.created()?.into(),
            modified_at: metadata.modified()?.into(),
            sha3_256_hash: sha3_hash,
            pdq_hash,
            frames,
            metadata: metadata.unwrap_or_default(),
        })
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }

    pub fn verify(&self, file_path: &Path) -> Result<bool, Box<dyn std::error::Error>> {
        // Verify file exists and has the same size
        let metadata = std::fs::metadata(file_path)?;
        if metadata.len() != self.file_size {
            return Ok(false);
        }

        // Verify hashes
        let current_sha3 = super::hash::compute_file_hash(file_path)?;
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
