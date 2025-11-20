#!/bin/bash

# Quick run script for Riptide Proxy

CONFIG="${1:-config.json}"

if [ ! -f "$CONFIG" ]; then
    echo "❌ Config file not found: $CONFIG"
    echo ""
    echo "Usage: ./run.sh [config-file]"
    echo ""
    echo "Example:"
    echo "  ./run.sh config.json"
    echo ""
    echo "Create config.json from config.example.json first:"
    echo "  cp config.example.json config.json"
    echo "  # Edit config.json with your settings"
    exit 1
fi

if [ ! -f "target/release/riptide" ]; then
    echo "❌ Binary not found. Building..."
    ./build.sh
fi

echo "🚀 Starting Riptide Proxy..."
echo "📋 Config: $CONFIG"
echo ""

# Set log level
export RUST_LOG="${RUST_LOG:-info}"

# Run the proxy
exec ./target/release/riptide --config "$CONFIG"

