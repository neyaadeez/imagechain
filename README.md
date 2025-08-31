# ImageChain

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/yourusername/imagechain/actions/workflows/ci.yml/badge.svg)](https://github.com/yourusername/imagechain/actions)
[![Docker Pulls](https://img.shields.io/docker/pulls/yourusername/imagechain)](https://hub.docker.com/r/yourusername/imagechain)

A high-performance Rust-based service for processing images and videos, generating cryptographic and perceptual hashes, and creating verifiable manifests. ImageChain provides a robust solution for media authentication, content verification, and similarity search.

## Features

- **Image Processing**: Upload and process images to generate hashes
- **Video Processing**: Extract frames from videos at specified intervals
- **Cryptographic Hashing**: SHA3-256 for file integrity verification
- **Perceptual Hashing**: PDQ hashing for content-based image identification
- **Deep Learning Embeddings**: Generate and compare image embeddings using ResNet-50
- **Manifest Generation**: Create verifiable manifests for media files
- **REST API**: Simple HTTP interface for integration

## 🚀 Features

- **Image Processing**
  - Support for multiple image formats (JPEG, PNG, WebP, etc.)
  - Automatic format conversion and optimization
  - Thumbnail generation
  
- **Video Processing**
  - Frame extraction at configurable intervals
  - Support for multiple video formats via FFmpeg
  - Efficient frame processing pipeline
  
- **Hashing & Fingerprinting**
  - Cryptographic hashing (SHA3-256)
  - Perceptual hashing (PDQ)
  - Content-based image identification
  
- **Deep Learning**
  - Image embeddings using ResNet-50
  - Cosine similarity for image comparison
  - Extensible model architecture
  
- **Manifest System**
  - JSON-based manifest format
  - Cryptographic verification
  - Tamper-evident design
  
- **Web API**
  - RESTful endpoints
  - File upload support
  - JSON responses
  - CORS support

## 🛠️ Prerequisites

- Rust (latest stable version)
- FFmpeg (for video processing)
- libtorch (for deep learning features)
- OpenSSL (for cryptographic operations)

## Usage Examples

### Computing Image Embeddings

```rust
use imagechain::{EmbeddingModel, init};
use image::open;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the application
    init()?;
    
    // Create a new embedding model
    let model = EmbeddingModel::new();
    
    // Load an image
    let img = open("path/to/your/image.jpg")?;
    
    // Compute embedding
    let embedding = model.compute_embedding(&img)?;
    
    println!("Generated embedding: {:?}", embedding);
    
    Ok(())
}
```

### Comparing Image Similarity

```rust
use imagechain::{EmbeddingModel, init};
use image::open;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init()?;
    let model = EmbeddingModel::new();
    
    // Load two images
    let img1 = open("path/to/first/image.jpg")?;
    let img2 = open("path/to/second/image.jpg")?;
    
    // Compute embeddings
    let emb1 = model.compute_embedding(&img1)?;
    let emb2 = model.compute_embedding(&img2)?;
    
    // Compare embeddings
    let similarity = EmbeddingModel::cosine_similarity(&emb1, &emb2);
    println!("Similarity: {:.2}%", similarity * 100.0);
    
    Ok(())
}
```

## System Requirements

### Installing FFmpeg

#### Ubuntu/Debian
```bash
sudo apt update
sudo apt install ffmpeg
```

#### macOS (using Homebrew)
```bash
brew install ffmpeg
```

#### Windows (using Chocolatey)
```bash
choco install ffmpeg
```

## 🚀 Quick Start

### Using Docker (Recommended)

```bash
# Start the service
docker-compose up -d

# The API will be available at http://localhost:3000
```

### Building from Source

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/imagechain.git
   cd imagechain
   ```

2. Install system dependencies:
   ```bash
   # Ubuntu/Debian
   sudo apt update
   sudo apt install -y ffmpeg libssl-dev pkg-config
   ```

3. Build the project:
   ```bash
   cargo build --release
   ```

4. Run the server:
   ```bash
   ./target/release/imagechain
   ```

## 📚 API Reference

### Upload and Process Media

```http
POST /api/upload
Content-Type: multipart/form-data

file: <media_file>
```

**Response**
```json
{
  "success": true,
  "data": {
    "media_type": "Image",
    "file_name": "example.jpg",
    "file_size": 12345,
    "sha3_256_hash": "a1b2c3...",
    "pdq_hash": "0000000000000000...",
    "embedding": [0.1, 0.2, ...],
    "created_at": "2023-01-01T00:00:00Z"
  }
}
```

### Verify Media Manifest

```http
POST /api/verify
Content-Type: application/json

{
  "media_type": "Image",
  "file_name": "example.jpg",
  "sha3_256_hash": "a1b2c3..."
}
```

**Response**
```json
{
  "success": true,
  "data": {
    "is_valid": true,
    "verification_time": "2023-01-01T00:00:01Z"
  }
}
```

## 🔍 Examples

### Compare Two Images

```rust
use imagechain::{EmbeddingModel, init};
use image::open;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init()?;
    let model = EmbeddingModel::new();
    
    let img1 = open("images/cat.jpg")?;
    let img2 = open("images/dog.jpg")?;
    
    let emb1 = model.compute_embedding(&img1)?;
    let emb2 = model.compute_embedding(&img2)?;
    
    let similarity = EmbeddingModel::cosine_similarity(&emb1, &emb2);
    println!("Images are {:.2}% similar", similarity * 100.0);
    
    Ok(())
}
```

## 🧪 Running Tests

```bash
# Run all tests
cargo test

# Run tests with detailed output
cargo test -- --nocapture

# Run specific test module
cargo test test_embeddings -- --nocapture
```

## 🐳 Docker Development

Build the development container:
```bash
docker build -t imagechain-dev .
```

Run tests in the container:
```bash
docker run -it --rm -v $(pwd):/app -w /app imagechain-dev cargo test
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- FFmpeg team for the amazing multimedia framework
- PyTorch team for the Rust bindings
- The Rust community for their awesome ecosystem

## Usage

1. Start the server:
   ```bash
   cargo run --release
   ```

2. The server will start on `http://127.0.0.1:3000`

### API Endpoints

#### Upload a File

```http
POST /api/upload
Content-Type: multipart/form-data

file: <file>
```

Example using `curl`:
```bash
curl -X POST -F "file=@/path/to/your/image.jpg" http://localhost:3000/api/upload
```

#### Verify a Manifest

```http
POST /api/verify
Content-Type: application/json

<manifest_json>
```

Example using `curl`:
```bash
curl -X POST -H "Content-Type: application/json" -d @manifest.json http://localhost:3000/api/verify
```

## Manifest Format

The manifest contains metadata about the processed media file, including hashes and other relevant information.

Example manifest for an image:
```json
{
  "media_type": "Image",
  "file_name": "example.jpg",
  "file_size": 12345,
  "created_at": "2023-01-01T00:00:00Z",
  "modified_at": "2023-01-01T00:00:00Z",
  "sha3_256_hash": "a1b2c3...",
  "pdq_hash": "0000000000000000...",
  "frames": null,
  "metadata": {}
}
```

## Configuration

Create a `.env` file in the project root to configure the application:

```env
RUST_LOG=info
UPLOAD_DIR=./uploads
```

## Testing

Run the test suite:

```bash
cargo test
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
