# Build stage
FROM rust:1.70-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libavformat-dev \
    libavcodec-dev \
    libavutil-dev \
    libswscale-dev \
    libavdevice-dev \
    python3 \
    python3-pip \
    && pip3 install torch torchvision --index-url https://download.pytorch.org/whl/cpu \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/imagechain

# Copy the source code
COPY . .

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl1.1 \
    libavformat58 \
    libavcodec58 \
    libavutil56 \
    libswscale5 \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -m imagechain
USER imagechain
WORKDIR /home/imagechain

# Create uploads directory
RUN mkdir -p uploads

# Copy the binary from the builder stage
COPY --from=builder /usr/src/imagechain/target/release/imagechain .

# Expose the port the app runs on
EXPOSE 3000

# Set the startup command
CMD ["./imagechain"]
