//! Utility functions and helpers for the ImageChain application

use std::path::Path;
use anyhow::Result;

/// Normalize a path to use forward slashes
#[allow(dead_code)]
pub(crate) fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

/// Ensure a directory exists, creating it if necessary
#[allow(dead_code)]
pub(crate) fn ensure_dir_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Generate a unique filename with a timestamp
#[allow(dead_code)]
pub(crate) fn generate_filename(prefix: &str, extension: &str) -> String {
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S_%3f");
    format!("{}_{}.{}", prefix, timestamp, extension.trim_start_matches('.'))
}

/// Validate that a file has an allowed extension
#[allow(dead_code)]
pub(crate) fn validate_file_extension(filename: &str, allowed_extensions: &[&str]) -> bool {
    if let Some(ext) = Path::new(filename).extension() {
        if let Some(ext_str) = ext.to_str() {
            return allowed_extensions
                .iter()
                .any(|&e| e.eq_ignore_ascii_case(ext_str));
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path() {
        assert_eq!(
            normalize_path("C:\\path\\to\\file"),
            "C:/path/to/file"
        );
        assert_eq!(
            normalize_path("/already/unix/path"),
            "/already/unix/path"
        );
    }

    #[test]
    fn test_generate_filename() {
        let filename = generate_filename("test", "jpg");
        assert!(filename.starts_with("test_"));
        assert!(filename.ends_with(".jpg"));
    }

    #[test]
    fn test_validate_file_extension() {
        let allowed = vec!["jpg", "jpeg", "png"];
        assert!(validate_file_extension("test.jpg", &allowed));
        assert!(validate_file_extension("test.JPEG", &allowed));
        assert!(!validate_file_extension("test.txt", &allowed));
        assert!(!validate_file_extension("test", &allowed));
    }
}
