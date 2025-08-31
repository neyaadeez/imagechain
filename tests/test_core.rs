use std::io::Write;
use tempfile::NamedTempFile;

use imagechain::core::hash::{compute_file_hash, compute_pdq_hash};
use image::RgbImage;

#[test]
fn test_compute_file_hash() {
    // Create a temporary file with some content
    let mut file = NamedTempFile::new().unwrap();
    write!(file, "test content").unwrap();
    
    // Compute hash
    let hash = compute_file_hash(file.path()).unwrap();
    
    // Verify the hash is not empty and has correct length (64 chars for SHA3-256)
    assert_eq!(hash.len(), 64);
    assert!(!hash.is_empty());
    
    // Compute hash again and verify it's the same
    let hash2 = compute_file_hash(file.path()).unwrap();
    assert_eq!(hash, hash2);
}

#[test]
fn test_compute_pdq_hash() {
    // Create a simple test image (2x2 pixels)
    let width = 2;
    let height = 2;
    let mut imgbuf = RgbImage::new(width, height);
    
    // Set some pixel values
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        *pixel = image::Rgb([
            (x as f32 * 255.0 / width as f32) as u8,
            (y as f32 * 255.0 / height as f32) as u8,
            128,
        ]);
    }
    
    // Convert to dynamic image
    let dynamic_img = image::DynamicImage::ImageRgb8(imgbuf);
    
    // Compute PDQ hash
    let hash = compute_pdq_hash(&dynamic_img).unwrap();
    
    // In a real test, we would verify the hash format
    // For now, just check it's not empty
    assert!(!hash.is_empty());
}

#[test]
fn test_manifest_creation() {
    use imagechain::models::manifest::{MediaManifest, MediaType};
    use std::io::Write;
    
    // Create a temporary file
    let mut file = NamedTempFile::new().unwrap();
    write!(file, "test content").unwrap();
    
    // Create a manifest
    let manifest = MediaManifest::new(
        file.path(),
        MediaType::Image,
        "test_hash".to_string(),
        Some("pdq_hash".to_string()),
        None,
        None,
    ).unwrap();
    
    // Verify manifest fields
    assert_eq!(manifest.media_type, MediaType::Image);
    assert_eq!(manifest.sha3_256_hash, "test_hash");
    assert_eq!(manifest.pdq_hash, Some("pdq_hash".to_string()));
    
    // Test serialization/deserialization
    let json = manifest.to_json().unwrap();
    let deserialized: MediaManifest = serde_json::from_str(&json).unwrap();
    
    assert_eq!(deserialized.media_type, MediaType::Image);
    assert_eq!(deserialized.sha3_256_hash, "test_hash");
}
