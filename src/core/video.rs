use anyhow::Result;
use image::DynamicImage;
use std::path::Path;
use std::process::Command;


/// Checks if FFmpeg is installed and available in the system path
pub fn check_ffmpeg_installed() -> Result<()> {
    let output = Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map_err(|_| anyhow::anyhow!("FFmpeg is not installed or not in system PATH"))?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("FFmpeg command failed"));
    }

    Ok(())
}

#[cfg(feature = "video")]
/// Initializes FFmpeg
pub fn init_ffmpeg() -> Result<()> {
    ffmpeg_next::init().map_err(|e| anyhow::anyhow!("Failed to initialize FFmpeg: {}", e))
}

#[cfg(not(feature = "video"))]
/// Initializes FFmpeg (placeholder)
pub fn init_ffmpeg() -> Result<()> {
    Ok(())
}

/// Extracts frames from a video at specified intervals
#[derive(Debug)]
#[allow(dead_code)]
pub struct FrameExtractor {
    input_path: String,
    interval_secs: f64,
}

impl FrameExtractor {
        /// Creates a new `FrameExtractor`.
    ///
    /// # Arguments
    ///
    /// * `input_path` - The path to the video file.
    /// * `interval_secs` - The interval in seconds at which to extract frames.
    pub fn new<P: AsRef<Path>>(input_path: P, interval_secs: f64) -> Self {
        Self {
            input_path: input_path.as_ref().to_string_lossy().into_owned(),
            interval_secs,
        }
    }

    #[cfg(feature = "video")]
    /// Extracts frames from the video at the specified interval
    pub fn extract_frames<F>(&self, mut callback: F) -> Result<()>
    where
        F: FnMut(DynamicImage, f64) -> Result<()>,
    {
        // Check if FFmpeg is installed
        check_ffmpeg_installed()?;
        init_ffmpeg()?;

        // Check if input file exists
        if !Path::new(&self.input_path).exists() {
            return Err(anyhow::anyhow!("Input file not found: {}", self.input_path));
        }

        // Create a temporary directory for extracted frames
        let tmpdir = tempfile::tempdir()?;
        let out_pattern = tmpdir.path().join("frame_%05d.png");

        // Compute fps filter string (frames per second)
        let interval = if self.interval_secs > 0.0 { self.interval_secs } else { 1.0 };
        let fps = 1.0 / interval;
        let vf_filter = format!("fps={}", fps);

        // Run ffmpeg to extract frames
        let status = Command::new("ffmpeg")
            .arg("-hide_banner")
            .arg("-loglevel").arg("error")
            .arg("-i").arg(&self.input_path)
            .arg("-vf").arg(vf_filter)
            .arg("-vsync").arg("vfr")
            .arg(out_pattern.to_string_lossy().to_string())
            .status()
            .map_err(|e| anyhow::anyhow!("Failed to spawn ffmpeg: {}", e))?;

        if !status.success() {
            return Err(anyhow::anyhow!("ffmpeg failed to extract frames"));
        }

        // Read extracted frames, sorted by name
        let mut entries: Vec<_> = std::fs::read_dir(tmpdir.path())?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();
        entries.sort();

        for (i, frame_path) in entries.iter().enumerate() {
            let img = image::open(frame_path)?;
            let timestamp = (i as f64) * interval;
            callback(img, timestamp)?;
        }

        Ok(())
    }

    #[cfg(not(feature = "video"))]
    /// Extracts frames from the video at the specified interval (placeholder)
    pub fn extract_frames<F>(&self, mut _callback: F) -> Result<()>
    where
        F: FnMut(DynamicImage, f64) -> Result<()>,
    {
        Err(anyhow::anyhow!("Video processing not available - enable 'video' feature"))
    }
}

#[cfg(feature = "video")]
/// Process video and extract frames
pub fn process_video<P: AsRef<Path>>(path: P, interval_secs: f64) -> Result<Vec<DynamicImage>> {
    let extractor = FrameExtractor::new(path, interval_secs);
    let mut frames = Vec::new();
    
    extractor.extract_frames(|frame, _timestamp| {
        frames.push(frame);
        Ok(())
    })?;
    
    Ok(frames)
}

#[cfg(not(feature = "video"))]
/// Process video and extract frames (placeholder)
pub fn process_video<P: AsRef<Path>>(_path: P, _interval_secs: f64) -> Result<Vec<DynamicImage>> {
    Err(anyhow::anyhow!("Video processing not available - enable 'video' feature"))
}

#[cfg(feature = "video")]
/// Extract frames from video
pub fn extract_frames<P: AsRef<Path>>(path: P, interval_secs: f64) -> Result<Vec<DynamicImage>> {
    process_video(path, interval_secs)
}

#[cfg(not(feature = "video"))]
/// Extract frames from video (placeholder)
pub fn extract_frames<P: AsRef<Path>>(_path: P, _interval_secs: f64) -> Result<Vec<DynamicImage>> {
    Err(anyhow::anyhow!("Video processing not available - enable 'video' feature"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_frame_extraction() {
        // This is a placeholder test - in a real scenario, you'd need a test video file
        // and proper assertions about the extracted frames
        let test_video = PathBuf::from("test_data/test_video.mp4");
        
        if test_video.exists() {
            let extractor = FrameExtractor::new(&test_video, 1.0);
            let mut frame_count = 0;
            
            let result = extractor.extract_frames(|_frame, _timestamp| {
                frame_count += 1;
                Ok(())
            });
            
            assert!(result.is_ok());
            assert!(frame_count > 0);
        }
    }
}
