# Dockerfile for cross-compiling SimonSaysSeeq Rust core for Norns
# This provides a consistent build environment for ARM cross-compilation

FROM rust:1.75-slim

# Install system dependencies for cross-compilation
RUN apt-get update && apt-get install -y \
    gcc-arm-linux-gnueabihf \
    libc6-dev-armhf-cross \
    pkg-config \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Set up Rust for cross-compilation
RUN rustup target add armv7-unknown-linux-gnueabihf

# Set environment variables for cross-compilation
ENV CC_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-gcc
ENV CXX_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-g++
ENV AR_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-ar
ENV CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc

# Create working directory
WORKDIR /app

# Copy Cargo files first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Create dummy source to build dependencies
RUN mkdir src && echo "fn main() {}" > src/lib.rs
RUN cargo build --release --target armv7-unknown-linux-gnueabihf
RUN rm -rf src

# Copy actual source code
COPY src/ ./src/

# Build the project
RUN cargo build --release --target armv7-unknown-linux-gnueabihf

# Create output directory
RUN mkdir -p /output

# Copy the built library to output
RUN cp target/armv7-unknown-linux-gnueabihf/release/libsimon_says_seeq_core.so /output/simon_says_seeq_core.so

# Default command to show build info
CMD ["sh", "-c", "ls -la /output/ && file /output/simon_says_seeq_core.so"]