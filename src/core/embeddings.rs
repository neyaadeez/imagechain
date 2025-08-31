use anyhow::{Context, Result};
use image::{DynamicImage, GenericImageView};
use ndarray::Array1;
use tch::{Device, Kind, Tensor, Vision};

/// A struct to handle image embeddings using a pre-trained model
pub struct EmbeddingModel {
    model: Vision,
    device: Device,
}

impl Default for EmbeddingModel {
    fn default() -> Self {
        Self::new()
    }
}

impl EmbeddingModel {
    /// Create a new instance of the embedding model
    pub fn new() -> Self {
        let device = Device::cuda_if_available();
        // Load a pre-trained ResNet-50 model
        let model = tch::vision::resnet::resnet50();
        model.set_eval();
        model.to(device);
        
        Self { model, device }
    }

    /// Compute an embedding for an image
    pub fn compute_embedding(&self, img: &DynamicImage) -> Result<Array1<f32>> {
        // Preprocess the image
        let input_tensor = self.preprocess_image(img)?;
        
        // Move tensor to the same device as the model
        let input_tensor = input_tensor.to(self.device);
        
        // Forward pass
        let output = self.model.forward_ts(&[&input_tensor])?;
        
        // Convert to ndarray
        let embedding = Vec::<f32>::try_from(output)?;
        
        Ok(Array1::from(embedding))
    }
    
    /// Preprocess an image for the model
    fn preprocess_image(&self, img: &DynamicImage) -> Result<Tensor> {
        // Resize to 224x224 (standard size for ResNet)
        let img = img.resize_exact(224, 224, image::imageops::FilterType::Triangle);
        
        // Convert to RGB if needed
        let rgb_img = img.to_rgb8();
        
        // Get image dimensions
        let (width, height) = rgb_img.dimensions();
        
        // Convert to a flat vector of f32 values in [0, 1] range
        let mut data = Vec::with_capacity((width * height * 3) as usize);
        
        for y in 0..height {
            for x in 0..width {
                let pixel = rgb_img.get_pixel(x, y);
                data.push(pixel[0] as f32 / 255.0); // R
                data.push(pixel[1] as f32 / 255.0); // G
                data.push(pixel[2] as f32 / 255.0); // B
            }
        }
        
        // Create a tensor from the data [3, 224, 224]
        let tensor = Tensor::of_slice(&data)
            .reshape(&[3, 224, 224])
            .to_kind(Kind::Float);
        
        // Add batch dimension and normalize
        let mean = Tensor::of_slice(&[0.485, 0.456, 0.406])
            .view([3, 1, 1])
            .to_kind(Kind::Float);
        let std = Tensor::of_slice(&[0.229, 0.224, 0.225])
            .view([3, 1, 1])
            .to_kind(Kind::Float);
        
        let normalized = (tensor - &mean) / &std;
        
        // Add batch dimension [1, 3, 224, 224]
        Ok(normalized.unsqueeze(0))
    }
    
    /// Compute cosine similarity between two embeddings
    pub fn cosine_similarity(a: &Array1<f32>, b: &Array1<f32>) -> f32 {
        let dot_product = a.dot(b);
        let norm_a = a.dot(a).sqrt();
        let norm_b = b.dot(b).sqrt();
        
        if norm_a > 0.0 && norm_b > 0.0 {
            (dot_product / (norm_a * norm_b)).min(1.0).max(-1.0)
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbImage;
    use std::path::Path;
    
    #[test]
    fn test_embedding_computation() {
        // Skip this test in CI since it requires downloading the model
        if std::env::var("CI").is_ok() {
            return;
        }
        
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
        
        let img = DynamicImage::ImageRgb8(imgbuf);
        
        // Compute embedding
        let model = EmbeddingModel::new();
        let embedding = model.compute_embedding(&img).unwrap();
        
        // Check embedding dimensions (ResNet-50 outputs 1000-dimensional vectors)
        assert_eq!(embedding.len(), 1000);
    }
    
    #[test]
    fn test_cosine_similarity() {
        // Test with identical vectors
        let a = Array1::from(vec![1.0, 0.0, 0.0]);
        let b = Array1::from(vec![1.0, 0.0, 0.0]);
        assert!((EmbeddingModel::cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);
        
        // Test with orthogonal vectors
        let a = Array1::from(vec![1.0, 0.0]);
        let b = Array1::from(vec![0.0, 1.0]);
        assert!((EmbeddingModel::cosine_similarity(&a, &b) - 0.0).abs() < 1e-6);
        
        // Test with opposite vectors
        let a = Array1::from(vec![1.0, 0.0]);
        let b = Array1::from(vec![-1.0, 0.0]);
        assert!((EmbeddingModel::cosine_similarity(&a, &b) - (-1.0)).abs() < 1e-6);
    }
}
