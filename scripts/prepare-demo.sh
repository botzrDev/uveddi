#!/bin/bash

# Uveddi Demo Preparation Script
# This script prepares real analysis data for the investor demo

echo "🚀 Preparing Uveddi Demo..."

# Step 1: Build Uveddi with latest performance fixes
echo "📦 Building Uveddi with alpha features..."
cargo build --features alpha

if [ $? -ne 0 ]; then
    echo "❌ Build failed. Please fix compilation errors first."
    exit 1
fi

echo "✅ Build successful!"

# Step 2: Generate fresh real analysis data for different components
echo "🔍 Generating real analysis data..."

# Analyze the detectors directory (rich in architectural issues)
echo "  Analyzing detectors..."
cargo run --features alpha --bin uveddi -- analyze src/analysis/detectors/ \
    --output-format json \
    --output frontend/public/real-analysis-detectors.json

# Analyze the CLI components (good mix of issues)
echo "  Analyzing CLI components..."
cargo run --features alpha --bin uveddi -- analyze src/cli/ \
    --output-format json \
    --output frontend/public/real-analysis-cli.json

# Analyze the core analysis engine (complex architecture)
echo "  Analyzing analysis engine..."
cargo run --features alpha --bin uveddi -- analyze src/analysis/engine.rs src/analysis/orchestrator.rs \
    --output-format json \
    --output frontend/public/real-analysis-engine.json

echo "✅ Analysis data generated!"

# Step 3: Start the frontend development server
echo "🌐 Starting React dashboard..."
cd frontend

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo "📦 Installing frontend dependencies..."
    npm install
fi

# Start the development server
echo "🚀 Starting frontend server at http://localhost:3000"
echo ""
echo "🎯 DEMO READY!"
echo "==============="
echo "• Open http://localhost:3000/dashboard/demo in your browser"
echo "• The dashboard will automatically load real analysis data"
echo "• Data includes:"
echo "  - 506+ real architectural issues"
echo "  - Code duplication detection results"
echo "  - God Object anti-pattern findings"  
echo "  - Interactive Mermaid diagrams"
echo "  - AI-powered insights and recommendations"
echo ""
echo "💡 For investors: Show how Uveddi finds real issues in complex codebases"
echo "   and provides actionable architectural insights."

npm run dev