# Dockerfile for Uveddi + Ollama + DeepSeek-Coder
FROM ubuntu:22.04

# Install dependencies
RUN apt-get update && \
    apt-get install -y curl git build-essential pkg-config libssl-dev ca-certificates libsqlite3-dev && \
    rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install Ollama
RUN curl -fsSL https://ollama.com/install.sh | sh

# Pre-fetch all dependencies (including dev-dependencies)
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch

# Copy the rest of the source
COPY . .

# Build and test to cache all dependencies and dev-dependencies
RUN cargo build --release
RUN cargo test --release --no-run

# Expose Ollama API port
EXPOSE 11434

COPY docker-entrypoint.sh /docker-entrypoint.sh
RUN chmod +x /docker-entrypoint.sh

ENTRYPOINT ["/docker-entrypoint.sh"]
