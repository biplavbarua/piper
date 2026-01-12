#!/bin/bash

TEST_DIR=~/piper_test_ground
mkdir -p "$TEST_DIR"

echo "Creating test environment in $TEST_DIR..."

# 1. Create a "Heavy" Node Modules folder (fake size)
mkdir -p "$TEST_DIR/my_project/node_modules"
# Use /dev/zero to create a 300MB COMPRESSIBLE file (Happy Path)
dd if=/dev/zero of="$TEST_DIR/my_project/node_modules/massive_dependency.bin" bs=1024 count=300000 2>/dev/null
echo "Created 300MB compressible node_modules artifact."

# 2. Create a "Stale" Log File
# Create it, then touch it to be old (60 days ago)
echo "This is a massive log file..." > "$TEST_DIR/old_server.log"
# Append junk to make it 5MB
for i in {1..5000}; do echo "ERROR: Something went wrong line $i" >> "$TEST_DIR/old_server.log"; done
# Set modification time to 2 months ago
touch -d "2 months ago" "$TEST_DIR/old_server.log"
echo "Created stale old_server.log (simulated 2 months old)."

# 3. Create a "Target" folder (Rust build artifact) - Guaranteed Compressible
mkdir -p "$TEST_DIR/rust_app/target/debug"
# Use /dev/zero for high compressibility (mimics unoptimized binaries/debug symbols which compress well)
dd if=/dev/zero of="$TEST_DIR/rust_app/target/debug/app_binary" bs=1024 count=50000 2>/dev/null # 50MB
printf "Log entry repeating...\n" | perl -ne 'print $_ x 1000000' > "$TEST_DIR/rust_app/target/debug/build.log" # Text logs
echo "Created highly compressible build artifacts."

echo ""
echo "Test Environment Ready!"
echo "Run: cargo run --release -- --scan $TEST_DIR"
