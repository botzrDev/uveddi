#!/usr/bin/env bash

# Test Uveddi with a real-world Rust codebase
# This script runs Uveddi on the Tokio project and verifies both CLI and dashboard

set -e

# Define colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}📊 Uveddi Real-World Test${NC}"
echo

# Check if the test codebase exists, if not clone it
if [ ! -d "/tmp/tokio-test" ]; then
  echo -e "${YELLOW}Cloning Tokio repository...${NC}"
  cd /tmp && git clone https://github.com/tokio-rs/tokio.git tokio-test
fi

echo -e "${GREEN}✓${NC} Test codebase ready at /tmp/tokio-test"

# Run Uveddi analysis with dashboard launch
echo -e "${YELLOW}Running Uveddi analysis on Tokio...${NC}"
echo -e "${YELLOW}This may take a while depending on the size of the codebase...${NC}"

# Build and run analysis
cd "$(dirname "$0")/.." # Navigate to project root
cargo build --release
RUST_BACKTRACE=1 ./target/release/uveddi analyze /tmp/tokio-test --output-format markdown --output /tmp/tokio-analysis-report.md --open-dashboard

echo
echo -e "${GREEN}✓${NC} Analysis complete!"
echo -e "${GREEN}✓${NC} Markdown report: /tmp/tokio-analysis-report.md"
echo -e "${GREEN}✓${NC} Dashboard should be open in your browser"
echo
echo -e "${YELLOW}Testing complete - Dashboard is now showing analysis results${NC}"
