# 1️⃣ Build Stage (Use Rust Official Image with Build Tools)
FROM rust:latest AS builder

# Install sqlx-cli so that migrations can be run in the start.sh script
RUN cargo install sqlx-cli --no-default-features --features postgres
# Install build dependencies
RUN apt-get update && apt-get install -y protobuf-compiler

WORKDIR /app

# Cache cargo dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo fetch

# Copy source files
COPY build.rs ./
COPY .sqlx ./.sqlx
COPY proto ./proto
COPY src ./src

# Build the release binary
RUN cargo build --release

# 2️⃣ Runtime Stage (Minimal, Secure Image)
FROM debian:bookworm-slim

# Install additional dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy start script dependencies
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx
COPY start.sh /app/start.sh
COPY migrations /app/migrations
COPY --from=builder /app/target/release/axum-grpc-example /app/axum-grpc-example

# Expose gRPC and HTTP ports
EXPOSE 50051 8080

# Call the start script
ENTRYPOINT ["/bin/sh", "/app/start.sh"]
