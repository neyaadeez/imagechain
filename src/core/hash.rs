use anyhow::Result;
use image::DynamicImage;
use sha3::{Digest, Sha3_256};
use std::{fs::File, io::Read, path::Path};

/// Computes SHA3-256 hash of a file
pub fn compute_file_hash<P: AsRef<Path>>(path: P) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha3_256::new();
    let mut buffer = [0; 1024];

    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Computes PDQ hash of an image
pub fn compute_pdq_hash(image: &DynamicImage) -> Result<String> {
    // Convert to grayscale and resize for PDQ hashing
    let gray_image = image.to_luma8();
    
    // In a real implementation, we would use the pdq-rs crate here
    // For now, we'll use a placeholder that returns a fixed hash
    // Replace this with actual PDQ hash computation
    Ok("pdq_hash_placeholder".to_string())
    
    // Actual implementation would look like:
    // let hash = pdq_rs::pdq_hash(&gray_image);
    // Ok(hash.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_compute_file_hash() {
        // Create a temporary file
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "test content").unwrap();
        
        // Compute hash
        let hash = compute_file_hash(file.path()).unwrap();
        
        // Verify the hash is not empty and has correct length (64 chars for SHA3-256)
        assert_eq!(hash.len(), 64);
        assert!(!hash.is_empty());
    }
}
