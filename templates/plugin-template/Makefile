# Uveddi Plugin Development Makefile
# Provides convenient commands for plugin development workflow

# Plugin information (from Cargo.toml)
PLUGIN_NAME ?= $(shell grep '^name = ' Cargo.toml | cut -d'"' -f2)
PLUGIN_VERSION ?= $(shell grep '^version = ' Cargo.toml | cut -d'"' -f2)

# Directories
DIST_DIR = dist
WIT_DIR = wit

# Build configuration
WASM_TARGET = wasm32-wasi
RELEASE_WASM = target/$(WASM_TARGET)/release/$(PLUGIN_NAME).wasm
DEBUG_WASM = target/$(WASM_TARGET)/debug/$(PLUGIN_NAME).wasm

# Colors for pretty output
GREEN = \033[0;32m
YELLOW = \033[1;33m
RED = \033[0;31m
NC = \033[0m # No Color

.PHONY: help build release debug test clean install dev check lint fmt package

# Default target
all: check release

help: ## Show this help message
	@echo "Uveddi Plugin Development Commands"
	@echo "================================="
	@echo
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# Development commands
dev: ## Start development mode with file watching
	@echo "$(GREEN)[INFO]$(NC) Starting development mode..."
	@cargo watch -x 'build --target $(WASM_TARGET)'

check: ## Run all checks (clippy, fmt, etc.)
	@echo "$(GREEN)[INFO]$(NC) Running code checks..."
	@cargo check --target $(WASM_TARGET)
	@cargo clippy --target $(WASM_TARGET) -- -D warnings
	@cargo fmt -- --check

lint: ## Run linting checks
	@echo "$(GREEN)[INFO]$(NC) Running linting..."
	@cargo clippy --target $(WASM_TARGET) -- -D warnings

fmt: ## Format code
	@echo "$(GREEN)[INFO]$(NC) Formatting code..."
	@cargo fmt

# Build commands
build: debug ## Build the plugin (debug mode)

debug: ## Build debug WASM plugin
	@echo "$(GREEN)[INFO]$(NC) Building debug WASM plugin..."
	@rustup target add $(WASM_TARGET) 2>/dev/null || true
	@cargo build --target $(WASM_TARGET)
	@echo "$(GREEN)[INFO]$(NC) Debug build: $(DEBUG_WASM)"

release: ## Build optimized release WASM plugin
	@echo "$(GREEN)[INFO]$(NC) Building release WASM plugin..."
	@rustup target add $(WASM_TARGET) 2>/dev/null || true
	@cargo build --release --target $(WASM_TARGET)
	@echo "$(GREEN)[INFO]$(NC) Release build: $(RELEASE_WASM)"
	@echo "$(GREEN)[INFO]$(NC) WASM size: $$(wc -c < $(RELEASE_WASM) | xargs) bytes"

optimize: release ## Build and further optimize WASM
	@echo "$(GREEN)[INFO]$(NC) Optimizing WASM binary..."
	@if command -v wasm-tools >/dev/null 2>&1; then \
		wasm-tools strip $(RELEASE_WASM) -o $(RELEASE_WASM).stripped; \
		mv $(RELEASE_WASM).stripped $(RELEASE_WASM); \
		echo "$(GREEN)[INFO]$(NC) Optimized size: $$(wc -c < $(RELEASE_WASM) | xargs) bytes"; \
	else \
		echo "$(YELLOW)[WARN]$(NC) wasm-tools not found, skipping strip optimization"; \
		echo "$(YELLOW)[WARN]$(NC) Install with: cargo install wasm-tools"; \
	fi

# Testing commands
test: ## Run all tests
	@echo "$(GREEN)[INFO]$(NC) Running tests..."
	@cargo test

test-watch: ## Run tests in watch mode
	@echo "$(GREEN)[INFO]$(NC) Running tests in watch mode..."
	@cargo watch -x test

# Packaging commands
package: release ## Create distribution package
	@echo "$(GREEN)[INFO]$(NC) Creating distribution package..."
	@mkdir -p $(DIST_DIR)
	@rm -rf $(DIST_DIR)/$(PLUGIN_NAME)
	@mkdir -p $(DIST_DIR)/$(PLUGIN_NAME)

	# Copy WASM binary
	@cp $(RELEASE_WASM) $(DIST_DIR)/$(PLUGIN_NAME)/plugin.wasm

	# Copy manifest and docs
	@cp plugin.toml $(DIST_DIR)/$(PLUGIN_NAME)/
	@if [ -f README.md ]; then cp README.md $(DIST_DIR)/$(PLUGIN_NAME)/; fi
	@if [ -f LICENSE ]; then cp LICENSE $(DIST_DIR)/$(PLUGIN_NAME)/; fi

	# Create build info
	@echo "Built: $$(date)" > $(DIST_DIR)/$(PLUGIN_NAME)/BUILD_INFO.txt
	@echo "Plugin: $(PLUGIN_NAME)" >> $(DIST_DIR)/$(PLUGIN_NAME)/BUILD_INFO.txt
	@echo "Version: $(PLUGIN_VERSION)" >> $(DIST_DIR)/$(PLUGIN_NAME)/BUILD_INFO.txt
	@echo "Rust: $$(rustc --version)" >> $(DIST_DIR)/$(PLUGIN_NAME)/BUILD_INFO.txt

	# Create ZIP archive
	@cd $(DIST_DIR) && zip -r $(PLUGIN_NAME).zip $(PLUGIN_NAME)/

	@echo "$(GREEN)[INFO]$(NC) Package created: $(DIST_DIR)/$(PLUGIN_NAME).zip"

# Documentation commands
docs: ## Generate documentation
	@echo "$(GREEN)[INFO]$(NC) Generating documentation..."
	@cargo doc --target $(WASM_TARGET) --no-deps --open

# Utility commands
clean: ## Clean build artifacts
	@echo "$(GREEN)[INFO]$(NC) Cleaning build artifacts..."
	@cargo clean
	@rm -rf $(DIST_DIR)
	@echo "$(GREEN)[INFO]$(NC) Clean complete"

install-deps: ## Install development dependencies
	@echo "$(GREEN)[INFO]$(NC) Installing dependencies..."
	@rustup target add $(WASM_TARGET)
	@if ! command -v cargo-watch >/dev/null 2>&1; then \
		cargo install cargo-watch; \
	fi
	@if ! command -v wasm-tools >/dev/null 2>&1; then \
		cargo install wasm-tools; \
	fi
	@echo "$(GREEN)[INFO]$(NC) Dependencies installed"

validate: ## Validate plugin configuration
	@echo "$(GREEN)[INFO]$(NC) Validating plugin..."
	@if [ ! -f "plugin.toml" ]; then \
		echo "$(RED)[ERROR]$(NC) plugin.toml not found"; \
		exit 1; \
	fi
	@if [ ! -f "wit/core-analysis.wit" ]; then \
		echo "$(RED)[ERROR]$(NC) wit/core-analysis.wit not found"; \
		exit 1; \
	fi
	@if ! grep -q 'crate-type.*=.*\["cdylib"\]' Cargo.toml; then \
		echo "$(RED)[ERROR]$(NC) Cargo.toml must specify crate-type = [\"cdylib\"]"; \
		exit 1; \
	fi
	@echo "$(GREEN)[INFO]$(NC) Plugin configuration is valid"

# Integration commands
install: release ## Install plugin locally with Uveddi
	@echo "$(GREEN)[INFO]$(NC) Installing plugin locally..."
	@if command -v uveddi >/dev/null 2>&1; then \
		uveddi plugin install $(RELEASE_WASM); \
		echo "$(GREEN)[INFO]$(NC) Plugin installed successfully"; \
	else \
		echo "$(YELLOW)[WARN]$(NC) Uveddi CLI not found in PATH"; \
		echo "$(YELLOW)[WARN]$(NC) WASM available at: $(RELEASE_WASM)"; \
	fi

test-plugin: install ## Test plugin with Uveddi
	@echo "$(GREEN)[INFO]$(NC) Testing plugin with Uveddi..."
	@if command -v uveddi >/dev/null 2>&1; then \
		uveddi plugin test $(PLUGIN_NAME) --verbose; \
	else \
		echo "$(RED)[ERROR]$(NC) Uveddi CLI not found"; \
		exit 1; \
	fi

# CI/CD helpers
ci-build: ## CI build (non-interactive)
	@echo "$(GREEN)[INFO]$(NC) CI build starting..."
	@rustup target add $(WASM_TARGET)
	@cargo check --target $(WASM_TARGET)
	@cargo test
	@cargo build --release --target $(WASM_TARGET)
	@echo "$(GREEN)[INFO]$(NC) CI build complete"

ci-test: ## CI test suite
	@echo "$(GREEN)[INFO]$(NC) CI test suite..."
	@cargo test -- --nocapture
	@cargo clippy --target $(WASM_TARGET) -- -D warnings
	@cargo fmt -- --check

# Show current status
status: ## Show plugin status
	@echo "Plugin Status"
	@echo "============="
	@echo "Name: $(PLUGIN_NAME)"
	@echo "Version: $(PLUGIN_VERSION)"
	@echo "Target: $(WASM_TARGET)"
	@echo "Debug built: $$(if [ -f "$(DEBUG_WASM)" ]; then echo "Yes ($$(wc -c < $(DEBUG_WASM) | xargs) bytes)"; else echo "No"; fi)"
	@echo "Release built: $$(if [ -f "$(RELEASE_WASM)" ]; then echo "Yes ($$(wc -c < $(RELEASE_WASM) | xargs) bytes)"; else echo "No"; fi)"
	@echo "Package: $$(if [ -f "$(DIST_DIR)/$(PLUGIN_NAME).zip" ]; then echo "Yes"; else echo "No"; fi)"
