# 1️⃣ Build Stage (Use Rust Official Image with Build Tools)
FROM rust:latest AS builder

# Set working directory inside the container
WORKDIR /app

# Install protobuf compiler
RUN apt-get update && apt-get install -y protobuf-compiler

# Copy only Cargo files to leverage Docker cache
COPY Cargo.toml Cargo.lock ./
COPY proto ./proto

# Create a dummy lib.rs to prevent dependency invalidation
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

# Fetch dependencies
RUN cargo build --release && cargo clean

# Now copy the actual source files
COPY src ./src
COPY build.rs ./

# Explicitly run build.rs before main compilation (sets OUT_DIR)
RUN cargo check

# Build the actual binary
RUN cargo build --release

# 2️⃣ Runtime Stage (Minimal, Secure Image)
FROM debian:bookworm-slim

# Install required dependencies for gRPC
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/axum-grpc-example /app/axum-grpc-example

# Expose gRPC and HTTP ports
EXPOSE 50051 8080

# Run the service
CMD ["/app/axum-grpc-example"]
