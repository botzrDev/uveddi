# Dockerfile for CodeAtlas + Ollama + DeepSeek-Coder
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

# Pull DeepSeek-Coder model (change version if needed)
# RUN ollama pull deepseek-coder:6.7b-instruct-q4_0

# Copy CodeAtlas source code into the container
WORKDIR /app
COPY . .

# Build CodeAtlas
RUN cargo build --release

# Expose Ollama API port
EXPOSE 11434

COPY docker-entrypoint.sh /docker-entrypoint.sh
RUN chmod +x /docker-entrypoint.sh

ENTRYPOINT ["/docker-entrypoint.sh"]
