# --- Configuration Variables ---

# The name of the resulting binary/executable
TARGET_NAME := my_rust_program

# The directory where the source code is located
SRC_DIR := src

# The source file(s) that trigger a rebuild
# Includes the main.rs and any other Rust files in src/
RUST_SOURCES := $(wildcard $(SRC_DIR)/*.rs) $(wildcard $(SRC_DIR)/*/*.rs)

# The default build command
CARGO_BUILD_CMD := cargo build --release

# The default run command
CARGO_RUN_CMD := cargo run --release

# --- Build Targets ---

# Default target: builds the release version
.PHONY: all
all: build

.PHONY: build
build: $(TARGET_NAME)
	@echo "✨ Build complete. Release executable is in target/release/$(TARGET_NAME)"

# Rule to depend on source files and run the cargo build command
# This is a 'phony' target because we rely on cargo to handle dependencies and output
$(TARGET_NAME): $(RUST_SOURCES)
	$(CARGO_BUILD_CMD)

# --- Execution Targets ---

.PHONY: run
run: $(TARGET_NAME)
	@echo "🚀 Running $(TARGET_NAME)..."
	# Find the release executable path
	# Note: On Windows, the executable might be $(TARGET_NAME).exe
	@target/release/$(TARGET_NAME)

# --- Utility Targets ---

.PHONY: debug
debug:
	@echo "🛠️ Building debug version..."
	cargo build

.PHONY: run-debug
run-debug: debug
	@echo "🏃 Running debug version..."
	cargo run

.PHONY: test
test:
	@echo "✅ Running tests..."
	cargo test

.PHONY: clean
clean:
	@echo "🧹 Cleaning up target directory..."
	cargo clean


# Docker commands
.PHONY: clean
services:
	@echo "🧹 Starting up docker services..."
	docker compose up

# --- Help Target ---

.PHONY: help
help:
	@echo "Available targets:"
	@echo "  all      - Default target. Builds the release version (same as 'make build')."
	@echo "  build    - Builds the release executable."
	@echo "  run      - Builds (if necessary) and runs the release executable."
	@echo "  debug    - Builds the debug executable (in target/debug)."
	@echo "  run-debug- Builds and runs the debug executable."
	@echo "  test     - Runs all tests."
	@echo "  clean    - Removes the target directory."