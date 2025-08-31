use anyhow::Result;
use image::DynamicImage;
use std::{fs::File, io::Read, path::Path};

#[cfg(feature = "hashing")]
use sha3::{Digest, Sha3_256};

#[cfg(feature = "hashing")]
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

#[cfg(not(feature = "hashing"))]
/// Computes SHA3-256 hash of a file (placeholder)
pub fn compute_file_hash<P: AsRef<Path>>(_path: P) -> Result<String> {
    Ok("placeholder_hash".to_string())
}

#[cfg(feature = "hashing")]
/// Computes SHA3-256 hash of byte data
pub fn compute_sha3_256(data: &[u8]) -> Result<String> {
    let mut hasher = Sha3_256::new();
    hasher.update(data);
    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(not(feature = "hashing"))]
/// Computes SHA3-256 hash of byte data (placeholder)
pub fn compute_sha3_256(_data: &[u8]) -> Result<String> {
    Ok("placeholder_hash".to_string())
}

/// Alias for compute_file_hash for backward compatibility
pub fn sha3_256<P: AsRef<Path>>(path: P) -> Result<String> {
    compute_file_hash(path)
}

#[cfg(feature = "hashing")]
/// Computes PDQ hash of an image using a simple hash approach
pub fn compute_pdq_hash(image: &DynamicImage) -> Result<String> {
    // Convert to RGB8 and compute a simple hash based on image properties
    let rgb_image = image.to_rgb8();
    let (width, height) = rgb_image.dimensions();
    let pixels = rgb_image.as_raw();
    
    // Simple hash based on image dimensions and pixel data
    let mut hash_value = 0u64;
    hash_value ^= width as u64;
    hash_value ^= (height as u64) << 16;
    
    // Sample some pixels for hash computation
    let step = (pixels.len() / 64).max(1);
    for (i, &pixel) in pixels.iter().step_by(step).enumerate() {
        if i >= 64 { break; }
        hash_value ^= (pixel as u64) << (i % 64);
    }

    // Return hash as a binary string (64 bits → 64 chars "0"/"1")
    Ok(format!("{:064b}", hash_value))
}

#[cfg(not(feature = "hashing"))]
/// Computes PDQ hash of an image (placeholder)
pub fn compute_pdq_hash(_image: &DynamicImage) -> Result<String> {
    Ok("placeholder_pdq_hash".to_string())
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
