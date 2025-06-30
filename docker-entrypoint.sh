#!/bin/bash
set -e

ollama serve &
# Wait for Ollama server to be ready
until curl -s http://localhost:11434/api/tags > /dev/null; do
  echo "Waiting for Ollama server..."
  sleep 2
done

echo "Ollama server is up. Pulling DeepSeek model..."
ollama pull deepseek-coder:6.7b-instruct-q4_0

echo "Running CodeAtlas tests (pre-built, should be fast)..."
cargo test --release --no-run && cargo test --release

echo "All done. Dropping to shell."
exec /bin/bash
