# ========================
# Build stage
# ========================
FROM rustlang/rust:nightly-slim AS builder

# Install build dependencies (Rust, FFmpeg build tools, Python, Torch)
RUN apt-get update && apt-get install -y \
    build-essential \
    clang \
    libclang-dev \
    pkg-config \
    libssl-dev \
    cmake \
    git \
    perl \
    nasm \
    yasm \
    python3 \
    python3-pip \
    && pip3 install --break-system-packages torch torchvision --index-url https://download.pytorch.org/whl/cpu \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /usr/src/imagechain

# Copy the source code
COPY . .

# Build the application (release mode)
RUN cargo build --release

# ========================
# Runtime stage
# ========================
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl1.1 \
    python3 \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -m imagechain
USER imagechain
WORKDIR /home/imagechain

# Create uploads directory
RUN mkdir -p uploads

# Copy the compiled binary from builder
COPY --from=builder /usr/src/imagechain/target/release/imagechain .

# Expose the port
EXPOSE 3000

# Start application
CMD ["./imagechain"]
