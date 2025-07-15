#!/bin/bash
set -e

# Function to check if Ollama is responsive
wait_for_ollama() {
  local max_attempts=30
  local attempt=1
  
  echo "Waiting for Ollama server to be ready..."
  while ! curl -s --fail http://localhost:11434/api/tags > /dev/null 2>&1; do
    if [ $attempt -ge $max_attempts ]; then
      echo "ERROR: Ollama server did not become available after $max_attempts attempts."
      return 1
    fi
    echo "Attempt $attempt/$max_attempts: Ollama server not ready yet. Waiting..."
    sleep 2
    ((attempt++))
  done
  
  echo "✅ Ollama server is up and running."
  return 0
}

# Start Ollama in the background
echo "Starting Ollama server..."
ollama serve &
OLLAMA_PID=$!

# Wait for Ollama to be ready
if ! wait_for_ollama; then
  echo "ERROR: Failed to start Ollama server. Exiting."
  exit 1
fi

# Check if the deepseek model is already available
if ollama list | grep -q "deepseek-coder:6.7b-instruct-q4_0"; then
  echo "deepseek-coder model is already available."
else
  echo "Pulling DeepSeek model (this may take a few minutes)..."
  ollama pull deepseek-coder:6.7b-instruct-q4_0
  
  if [ $? -ne 0 ]; then
    echo "WARNING: Failed to pull the DeepSeek model. AI features may not work correctly."
  else
    echo "✅ DeepSeek model successfully pulled."
  fi
fi

# Verify the model works
echo "Testing the model with a simple query..."
RESPONSE=$(ollama run deepseek-coder:6.7b-instruct-q4_0 "Say hello" 2>/dev/null || echo "ERROR")

if [[ "$RESPONSE" == *"ERROR"* ]]; then
  echo "WARNING: The model test failed. AI features may not work correctly."
else
  echo "✅ Model test successful."
fi

# Run the test suite
echo "Running Uveddi tests (pre-built, should be fast)..."
cargo test --release --no-run && cargo test --release

# Generate test data for benchmarking
echo "Generating benchmark data for testing..."
cargo run --release --bin generate_benchmark_data

# Start a bash shell to allow the user to interact with the container
echo "All setup complete. Dropping to shell."
echo "You can now run 'cargo run' to start Uveddi."
exec /bin/bash
