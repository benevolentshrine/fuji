#!/bin/bash

# Vanguard Unix Launcher (Linux/macOS)
# This script ensures the build process is isolated to avoid file-system locking 
# and provides a consistent launch experience across Unix-like systems.

# Set the build target directory to an isolated temp path
export CARGO_TARGET_DIR="/tmp/vanguard_build"

# Check if cargo is installed
if ! command -v cargo &> /dev/null
then
    echo "Error: cargo (Rust toolchain) could not be found. Please install it from https://rustup.rs"
    exit 1
fi

echo "--- Starting Vanguard Deployment Pipeline ---"
echo "Target OS: $(uname -s)"
echo "Build Cache: $CARGO_TARGET_DIR"

# Run the application
cargo run --release
