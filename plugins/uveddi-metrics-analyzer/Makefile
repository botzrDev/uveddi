# Uveddi Plugin Development Makefile
# Provides convenient commands for plugin development workflow

# Plugin information (can be overridden)
PLUGIN_NAME ?= $(shell grep '^name = ' Cargo.toml | cut -d'"' -f2)
PLUGIN_VERSION ?= $(shell grep '^version = ' Cargo.toml | cut -d'"' -f2)

# Directories
PKG_DIR = pkg
DIST_DIR = dist
WIT_DIR = wit
TESTS_DIR = tests

# Build configuration
WASM_TARGET = wasm32-unknown-unknown
WASM_PACK_ARGS = --target web --out-dir $(PKG_DIR)
OPTIMIZE_LEVEL = -Oz

# Colors for pretty output
GREEN = \033[0;32m
YELLOW = \033[1;33m
RED = \033[0;31m
NC = \033[0m # No Color

.PHONY: help build test clean install dev check lint fmt optimize package docs serve

# Default target
all: check build test package

help: ## Show this help message
	@echo "Uveddi Plugin Development Commands"
	@echo "================================="
	@echo
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# Development commands
dev: ## Start development mode with file watching
	@echo "$(GREEN)[INFO]$(NC) Starting development mode..."
	@cargo watch -x 'test' -x 'build --target $(WASM_TARGET)' -s 'make build-wasm'

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
build: check build-wasm ## Build the plugin (with checks)

build-wasm: ## Build WASM module only
	@echo "$(GREEN)[INFO]$(NC) Building WASM plugin..."
	@rustup target add $(WASM_TARGET) 2>/dev/null || true
	@wasm-pack build $(WASM_PACK_ARGS)
	@echo "$(GREEN)[INFO]$(NC) WASM build complete"

build-debug: ## Build in debug mode
	@echo "$(GREEN)[INFO]$(NC) Building debug WASM plugin..."
	@wasm-pack build $(WASM_PACK_ARGS) --debug

optimize: build-wasm ## Build and optimize WASM
	@echo "$(GREEN)[INFO]$(NC) Optimizing WASM binary..."
	@if command -v wasm-opt >/dev/null 2>&1; then \
		find $(PKG_DIR) -name "*.wasm" -exec wasm-opt $(OPTIMIZE_LEVEL) {} -o {} \; ; \
		echo "$(GREEN)[INFO]$(NC) Optimization complete"; \
	else \
		echo "$(YELLOW)[WARN]$(NC) wasm-opt not found, skipping optimization"; \
		echo "$(YELLOW)[WARN]$(NC) Install binaryen: brew install binaryen / apt-get install binaryen"; \
	fi

# Testing commands
test: ## Run all tests
	@echo "$(GREEN)[INFO]$(NC) Running tests..."
	@cargo test
	@cargo test --target $(WASM_TARGET)

test-watch: ## Run tests in watch mode
	@echo "$(GREEN)[INFO]$(NC) Running tests in watch mode..."
	@cargo watch -x test

test-coverage: ## Generate test coverage report
	@echo "$(GREEN)[INFO]$(NC) Generating coverage report..."
	@if command -v cargo-tarpaulin >/dev/null 2>&1; then \
		cargo tarpaulin --out Html --output-dir coverage; \
		echo "$(GREEN)[INFO]$(NC) Coverage report generated in coverage/"; \
	else \
		echo "$(YELLOW)[WARN]$(NC) cargo-tarpaulin not found"; \
		echo "$(YELLOW)[WARN]$(NC) Install with: cargo install cargo-tarpaulin"; \
	fi

# Packaging commands
package: optimize ## Create distribution package
	@echo "$(GREEN)[INFO]$(NC) Creating distribution package..."
	@mkdir -p $(DIST_DIR)
	@rm -rf $(DIST_DIR)/$(PLUGIN_NAME)
	@mkdir -p $(DIST_DIR)/$(PLUGIN_NAME)
	
	# Copy WASM binary
	@cp $(PKG_DIR)/*.wasm $(DIST_DIR)/$(PLUGIN_NAME)/plugin.wasm
	
	# Copy manifest
	@cp plugin.toml $(DIST_DIR)/$(PLUGIN_NAME)/
	
	# Copy documentation
	@if [ -f README.md ]; then cp README.md $(DIST_DIR)/$(PLUGIN_NAME)/; fi
	@if [ -f CHANGELOG.md ]; then cp CHANGELOG.md $(DIST_DIR)/$(PLUGIN_NAME)/; fi
	@if [ -f LICENSE ]; then cp LICENSE $(DIST_DIR)/$(PLUGIN_NAME)/; fi
	
	# Create build info
	@echo "Built: $$(date)" > $(DIST_DIR)/$(PLUGIN_NAME)/BUILD_INFO.txt
	@echo "Plugin: $(PLUGIN_NAME)" >> $(DIST_DIR)/$(PLUGIN_NAME)/BUILD_INFO.txt
	@echo "Version: $(PLUGIN_VERSION)" >> $(DIST_DIR)/$(PLUGIN_NAME)/BUILD_INFO.txt
	@echo "Rust: $$(rustc --version)" >> $(DIST_DIR)/$(PLUGIN_NAME)/BUILD_INFO.txt
	@echo "wasm-pack: $$(wasm-pack --version | head -n1)" >> $(DIST_DIR)/$(PLUGIN_NAME)/BUILD_INFO.txt
	
	# Create ZIP archive
	@cd $(DIST_DIR) && zip -r $(PLUGIN_NAME).zip $(PLUGIN_NAME)/
	
	@echo "$(GREEN)[INFO]$(NC) Package created: $(DIST_DIR)/$(PLUGIN_NAME).zip"
	@echo "$(GREEN)[INFO]$(NC) WASM size: $$(wc -c < $(DIST_DIR)/$(PLUGIN_NAME)/plugin.wasm) bytes"

# Documentation commands
docs: ## Generate documentation
	@echo "$(GREEN)[INFO]$(NC) Generating documentation..."
	@cargo doc --target $(WASM_TARGET) --no-deps --open

docs-serve: ## Serve documentation locally
	@echo "$(GREEN)[INFO]$(NC) Serving documentation..."
	@cd target/$(WASM_TARGET)/doc && python3 -m http.server 8080

# Development server for testing
serve: build-wasm ## Serve plugin for testing
	@echo "$(GREEN)[INFO]$(NC) Starting development server..."
	@if [ -d "$(PKG_DIR)" ]; then \
		cd $(PKG_DIR) && python3 -m http.server 3000; \
	else \
		echo "$(RED)[ERROR]$(NC) No pkg directory found. Run 'make build' first."; \
		exit 1; \
	fi

# Utility commands
clean: ## Clean build artifacts
	@echo "$(GREEN)[INFO]$(NC) Cleaning build artifacts..."
	@cargo clean
	@rm -rf $(PKG_DIR)
	@rm -rf $(DIST_DIR)
	@rm -rf target
	@rm -rf coverage
	@echo "$(GREEN)[INFO]$(NC) Clean complete"

install-deps: ## Install development dependencies
	@echo "$(GREEN)[INFO]$(NC) Installing dependencies..."
	@rustup target add $(WASM_TARGET)
	@if ! command -v wasm-pack >/dev/null 2>&1; then \
		curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh; \
	fi
	@if ! command -v cargo-watch >/dev/null 2>&1; then \
		cargo install cargo-watch; \
	fi
	@echo "$(GREEN)[INFO]$(NC) Dependencies installed"

validate: ## Validate plugin configuration
	@echo "$(GREEN)[INFO]$(NC) Validating plugin..."
	@if [ ! -f "plugin.toml" ]; then \
		echo "$(RED)[ERROR]$(NC) plugin.toml not found"; \
		exit 1; \
	fi
	@if ! grep -q 'crate-type.*=.*\["cdylib"\]' Cargo.toml; then \
		echo "$(RED)[ERROR]$(NC) Cargo.toml must specify crate-type = [\"cdylib\"]"; \
		exit 1; \
	fi
	@echo "$(GREEN)[INFO]$(NC) Plugin configuration is valid"

# Release commands
release: test package ## Create a release build
	@echo "$(GREEN)[INFO]$(NC) Creating release..."
	@echo "$(GREEN)[INFO]$(NC) Version: $(PLUGIN_VERSION)"
	@echo "$(GREEN)[INFO]$(NC) Package: $(DIST_DIR)/$(PLUGIN_NAME).zip"

bump-version: ## Bump version (patch)
	@echo "$(GREEN)[INFO]$(NC) Bumping version..."
	@cargo install cargo-edit 2>/dev/null || true
	@cargo set-version --bump patch
	@echo "$(GREEN)[INFO]$(NC) Version bumped to: $$(grep '^version = ' Cargo.toml | cut -d'"' -f2)"

# Integration commands
install-plugin: package ## Install plugin locally
	@echo "$(GREEN)[INFO]$(NC) Installing plugin locally..."
	@if command -v uveddi >/dev/null 2>&1; then \
		uveddi plugin install $(DIST_DIR)/$(PLUGIN_NAME).zip; \
	else \
		echo "$(YELLOW)[WARN]$(NC) Uveddi CLI not found in PATH"; \
		echo "$(YELLOW)[WARN]$(NC) Plugin package available at: $(DIST_DIR)/$(PLUGIN_NAME).zip"; \
	fi

test-with-uveddi: install-plugin ## Test plugin with Uveddi
	@echo "$(GREEN)[INFO]$(NC) Testing plugin with Uveddi..."
	@if command -v uveddi >/dev/null 2>&1; then \
		uveddi plugin test $(PLUGIN_NAME) --verbose; \
	else \
		echo "$(RED)[ERROR]$(NC) Uveddi CLI not found"; \
		exit 1; \
	fi

# CI/CD helpers
ci-build: ## CI build (no interactive components)
	@echo "$(GREEN)[INFO]$(NC) CI build starting..."
	@rustup target add $(WASM_TARGET)
	@cargo check --target $(WASM_TARGET)
	@cargo test
	@wasm-pack build $(WASM_PACK_ARGS)
	@echo "$(GREEN)[INFO]$(NC) CI build complete"

ci-test: ## CI test suite
	@echo "$(GREEN)[INFO]$(NC) CI test suite..."
	@cargo test --target $(WASM_TARGET) -- --nocapture
	@cargo clippy --target $(WASM_TARGET) -- -D warnings
	@cargo fmt -- --check

# Performance and size analysis
analyze-size: build-wasm ## Analyze WASM binary size
	@echo "$(GREEN)[INFO]$(NC) Analyzing WASM binary size..."
	@find $(PKG_DIR) -name "*.wasm" -exec ls -lh {} \;
	@if command -v twiggy >/dev/null 2>&1; then \
		find $(PKG_DIR) -name "*.wasm" -exec twiggy top {} \; ; \
	else \
		echo "$(YELLOW)[WARN]$(NC) Install twiggy for detailed size analysis: cargo install twiggy"; \
	fi

benchmark: ## Run benchmarks
	@echo "$(GREEN)[INFO]$(NC) Running benchmarks..."
	@if [ -d "benches" ]; then \
		cargo bench; \
	else \
		echo "$(YELLOW)[WARN]$(NC) No benchmarks found"; \
	fi

# Security analysis
security-audit: ## Run security audit
	@echo "$(GREEN)[INFO]$(NC) Running security audit..."
	@if command -v cargo-audit >/dev/null 2>&1; then \
		cargo audit; \
	else \
		echo "$(YELLOW)[WARN]$(NC) Install cargo-audit: cargo install cargo-audit"; \
	fi

# Show current status
status: ## Show plugin status
	@echo "Plugin Status"
	@echo "============="
	@echo "Name: $(PLUGIN_NAME)"
	@echo "Version: $(PLUGIN_VERSION)"
	@echo "Built: $$(if [ -d "$(PKG_DIR)" ]; then echo "Yes"; else echo "No"; fi)"
	@echo "Package: $$(if [ -f "$(DIST_DIR)/$(PLUGIN_NAME).zip" ]; then echo "Yes"; else echo "No"; fi)"
	@echo "WASM size: $$(if [ -f "$(PKG_DIR)/"*.wasm ]; then wc -c < $$(find $(PKG_DIR) -name "*.wasm" | head -n1) | xargs echo; else echo "Not built"; fi) bytes"