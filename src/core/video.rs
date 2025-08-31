use anyhow::{Context, Result, anyhow};
use ffmpeg_next::{
    format::{input, Pixel},
    media::Type,
    software::scaling,
    util::frame::video::Video,
    Frame,
};
use std::path::Path;
use std::process::Command;

/// Checks if FFmpeg is installed and available in the system path
pub fn check_ffmpeg_installed() -> Result<()> {
    let output = Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map_err(|_| anyhow!("FFmpeg is not installed or not in system PATH"))?;

    if !output.status.success() {
        return Err(anyhow!("FFmpeg command failed"));
    }

    Ok(())
}

/// Extracts frames from a video at specified intervals
pub struct FrameExtractor {
    input_path: String,
    interval_secs: f64,
}

impl FrameExtractor {
    pub fn new<P: AsRef<Path>>(input_path: P, interval_secs: f64) -> Self {
        Self {
            input_path: input_path.as_ref().to_string_lossy().into_owned(),
            interval_secs,
        }
    }

    /// Extracts frames from the video at the specified interval
    pub fn extract_frames<F>(&self, mut callback: F) -> Result<()>
    where
        F: FnMut(Frame, f64) -> Result<()>,
    {
        // Check if FFmpeg is installed
        check_ffmpeg_installed()?;
        
        // Initialize FFmpeg
        ffmpeg_next::init().map_err(|e| anyhow!("Failed to initialize FFmpeg: {}", e))?;
        
        // Check if input file exists
        if !Path::new(&self.input_path).exists() {
            return Err(anyhow!("Input file not found: {}", self.input_path));
        }

        let mut ictx = input(&self.input_path).context("Failed to open input file")?;

        let input = ictx
            .streams()
            .best(Type::Video)
            .ok_or_else(|| anyhow::anyhow!("No video stream found"))?;
        let video_stream_index = input.index();

        let context_decoder = ffmpeg_next::codec::context::Context::from_parameters(input.parameters())?;
        let mut decoder = context_decoder.decoder().video()?;

        let mut frame_index = 0;
        let time_base = input.time_base();
        let frame_rate = input.rate();
        let frame_interval = (frame_rate as f64 * self.interval_secs).round() as i64;

        let mut receive_and_process_decoded_frames = |decoder: &mut ffmpeg_next::decoder::Video| -> Result<()> {
            let mut decoded = Video::empty();
            while decoder.receive_frame(&mut decoded).is_ok() {
                let timestamp_secs = decoded.timestamp().map_or(0.0, |ts| ts as f64 * f64::from(time_base));
                
                // Convert frame to RGB
                let mut rgb_frame = Video::empty();
                let mut scaler = scaling::Context::get(
                    decoded.format(),
                    decoded.width(),
                    decoded.height(),
                    Pixel::RGB24,
                    decoded.width(),
                    decoded.height(),
                    scaling::Flags::BILINEAR,
                )?;
                
                scaler.run(&decoded, &mut rgb_frame)?;
                
                // Call the callback with the frame and timestamp
                callback(rgb_frame, timestamp_secs)?;
                
                frame_index += 1;
            }
            Ok(())
        };

        for (stream, packet) in ictx.packets() {
            if stream.index() == video_stream_index {
                // Only process frames at the specified interval
                if packet.pts().unwrap_or(0) % frame_interval == 0 {
                    decoder.send_packet(&packet)?;
                    receive_and_process_decoded_frames(&mut decoder)?;
                }
            }
        }

        // Flush the decoder
        decoder.send_eof()?;
        receive_and_process_decoded_frames(&mut decoder)?;

        Ok(())
    }
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
