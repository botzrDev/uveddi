# Multi-stage Dockerfile for Uveddi with security hardening
# Stage 1: Builder
FROM rust:1.70-slim AS builder

# Install only necessary build dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev \
        ca-certificates \
        libsqlite3-dev \
        git && \
    rm -rf /var/lib/apt/lists/* && \
    apt-get clean

# Create app directory
WORKDIR /app

# Copy dependency files first for better layer caching
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch --target x86_64-unknown-linux-gnu

# Copy source code
COPY . .

# Build the application in release mode
RUN cargo build --release --target x86_64-unknown-linux-gnu

# Stage 2: Runtime (minimal image)
FROM debian:bookworm-slim AS runtime

# Install only runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        libssl3 \
        libsqlite3-0 \
        curl && \
    rm -rf /var/lib/apt/lists/* && \
    apt-get clean

# Create non-root user
RUN groupadd -r -g 1001 uveddi && \
    useradd -r -g uveddi -u 1001 -s /bin/false -M uveddi

# Create necessary directories with proper ownership
RUN mkdir -p /app/data /app/cache /app/logs && \
    chown -R uveddi:uveddi /app

# Copy application binary from builder stage
COPY --from=builder /app/target/x86_64-unknown-linux-gnu/release/uveddi /usr/local/bin/uveddi
COPY --chown=uveddi:uveddi deploy/docker-entrypoint.sh /docker-entrypoint.sh

# Set proper permissions
RUN chmod +x /docker-entrypoint.sh && \
    chmod +x /usr/local/bin/uveddi

# Set working directory
WORKDIR /app

# Switch to non-root user
USER uveddi

# Configure environment
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

# Expose application port (not running as root, so can't use privileged ports)
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Use the entrypoint script
ENTRYPOINT ["/docker-entrypoint.sh"]
