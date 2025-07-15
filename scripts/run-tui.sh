#!/bin/bash
# Script to run the TUI in a Docker container with proper TTY support

set -e

echo "Building Docker image for TUI..."
docker build -t uveddi-tui -f - . << 'EOF'
FROM rust:1.70

WORKDIR /app
COPY . .

# Install dependencies and build
RUN cargo build --bin tui_test --features tui

# Set up proper terminal environment
ENV TERM=xterm-256color

CMD ["cargo", "run", "--bin", "tui_test", "--features", "tui"]
EOF

echo "Running TUI in Docker with TTY support..."
docker run -it --rm uveddi-tui