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
- **Deep Learning Embeddings**: OpenCLIP EVA (default EVA02-L-14) via external Python service (GPU optional)
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
  - Image embeddings via OpenCLIP/EVA models (default EVA02-L-14)
  - Cosine similarity for image comparison
  - External Python FastAPI embedding service; configurable model/device
  
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
- Docker (optional, recommended to run full stack)
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

Notes:
- The stack starts two services from `docker-compose.yml`:
  - `imagechain` (Rust API) on http://localhost:3000
  - `embedding` (Python OpenCLIP service) on http://localhost:8001
- Default embedding model is EVA02-L-14 (pretrained: laion2b_s9b_b144k) on CPU.
- GPU acceleration is available via the optional `embedding-gpu` service. See "Embedding Service" below.

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

Query parameters:
- include_embeddings (bool, default: false) — include image/frame embeddings
- extract_frames (bool, default: true; video only) — enable/disable frame extraction
- frame_interval_secs (f64, default: 1.0; video only) — seconds between frames
- max_frames (usize, optional; video only) — cap number of processed frames. If omitted, the full video is processed.

**Response**
```json
{
  "success": true,
  "data": {
    "media_type": "Image",
    "file_name": "example.jpg",
    "file_size": 12345,
    "created_at": "2023-01-01T00:00:00Z",
    "modified_at": "2023-01-01T00:00:00Z",
    "sha3_256_hash": "a1b2c3...",
    "pdq_hash": "0000000000000000...",
    "frames": null,
    "metadata": { "embedding": [0.1, 0.2, 0.3, "..."] }
  }
}
```

Example (video) with frames and optional embeddings:

```json
{
  "success": true,
  "data": {
    "media_type": "Video",
    "file_name": "video1.mp4",
    "file_size": 13927646,
    "created_at": "2025-08-31T06:59:48Z",
    "modified_at": "2025-08-31T06:59:49Z",
    "sha3_256_hash": "...",
    "pdq_hash": null,
    "frames": [
      { "timestamp_secs": 0.0, "pdq_hash": "0101...", "embedding": [0.12, 0.03, "..."] },
      { "timestamp_secs": 1.0, "pdq_hash": "1110...", "embedding": [0.11, 0.07, "..."] }
    ],
    "metadata": {
      "frame_interval_secs": 1.0,
      "frame_count": 2,
      "max_frames": null,
      "extracted_frames": true,
      "original_extension": "mp4"
    }
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
    "is_valid": true
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

Example with query flags (video, 0.5s interval, cap at 100 frames, include embeddings):
```bash
curl -X POST \
  -F "file=@/path/to/your/video.mp4" \
  "http://localhost:3000/api/upload?extract_frames=true&frame_interval_secs=0.5&max_frames=100&include_embeddings=true"
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
EMBEDDING_SERVICE_URL=http://localhost:8001

# Embedding service (Python, OpenCLIP) defaults:
# MODEL_NAME=EVA02-L-14
# PRETRAINED=laion2b_s9b_b144k
# DEVICE=cpu
```

## Embedding Service (OpenCLIP/EVA)

ImageChain can call an external Python FastAPI service to compute image embeddings using OpenCLIP models.

- Default model: `EVA02-L-14` with `laion2b_s9b_b144k`
- Configure via environment variables on the Python service:
  - `MODEL_NAME` (e.g., EVA02-L-14, ViT-B-32)
  - `PRETRAINED` (e.g., laion2b_s9b_b144k)
  - `DEVICE` (`cpu` or `cuda`)
- Rust connects via `EMBEDDING_SERVICE_URL` (e.g., `http://embedding:8001` in Docker, or `http://localhost:8001` locally).

Docker Compose services:
- `embedding` (CPU): builds from `python_service/Dockerfile` and exposes port 8001.
- `embedding-gpu` (CUDA, optional): builds from `python_service/Dockerfile.cuda`.
  - Requires NVIDIA drivers and NVIDIA Container Toolkit.
  - To enable, switch `imagechain.depends_on` to `embedding-gpu` and set `EMBEDDING_SERVICE_URL=http://embedding-gpu:8001`.

Health check and model info:

```bash
curl http://localhost:8001/health
curl http://localhost:8001/models
```

## Testing

Run the test suite:

```bash
cargo test
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
