//! Example showing how to compute and compare image embeddings

use anyhow::Result;
use image::open;
use imagechain::{EmbeddingModel, init};

fn main() -> Result<()> {
    // Initialize the application
    init()?;
    
    // Create a new embedding model
    let model = EmbeddingModel::new();
    
    // Load two images
    let img1 = open("examples/data/cat.jpg").expect("Failed to load image 1");
    let img2 = open("examples/data/dog.jpg").expect("Failed to load image 2");
    
    // Compute embeddings
    let embedding1 = model.compute_embedding(&img1)?;
    let embedding2 = model.compute_embedding(&img2)?;
    
    // Compute similarity
    let similarity = EmbeddingModel::cosine_similarity(&embedding1, &embedding2);
    
    println!("Similarity between images: {:.2}%", similarity * 100.0);
    
    Ok(())
}
