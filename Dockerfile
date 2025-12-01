# Build Stage
FROM rust:1.75-slim-bookworm as builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Copy manifest files to cache dependencies
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY modules ./modules

# Build the application (release mode)
# Note: We assume the main binary is in aether_core or a separate cli crate. 
# For now, we'll build the workspace.
RUN cargo build --release

# Runtime Stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
# chromium is for the headless browser fallback
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libsqlite3-0 \
    iproute2 \
    iputils-ping \
    chromium \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
# Adjust the path based on the actual binary name produced
COPY --from=builder /usr/src/app/target/release/aether_core /app/aether

# Create data directory
RUN mkdir -p /app/data

# Environment variables
ENV RUST_LOG=info
ENV AETHER_ENV=production

# Entrypoint
ENTRYPOINT ["/app/aether"]
