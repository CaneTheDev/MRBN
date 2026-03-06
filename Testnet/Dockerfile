# Multi-stage build for MRBN Node with Relay Bridge
# Stage 1: Build
FROM rustlang/rust:nightly-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY tests ./tests

# Build both binaries
RUN cargo build --release --bin mrbn-node
RUN cargo build --release --bin relay-bridge

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 mrbn

WORKDIR /home/mrbn

# Copy binaries from builder
COPY --from=builder /app/target/release/mrbn-node /usr/local/bin/
COPY --from=builder /app/target/release/relay-bridge /usr/local/bin/

# Copy startup script
COPY start_with_bridge.sh /home/mrbn/
RUN chmod +x /home/mrbn/start_with_bridge.sh

# Create data directory
RUN mkdir -p /home/mrbn/data && chown -R mrbn:mrbn /home/mrbn

USER mrbn

# Expose ports
EXPOSE 8333
EXPOSE 8334

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD pgrep mrbn-node || exit 1

# Run with relay bridge
CMD ["/home/mrbn/start_with_bridge.sh"]
