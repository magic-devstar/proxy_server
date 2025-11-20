#!/bin/bash

echo "🔨 Building Riptide Rust Proxy..."
echo ""

# Check if running on Linux
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "✅ Linux detected - building with io_uring support"
    cargo build --release
else
    echo "⚠️  Non-Linux system - building without io_uring"
    cargo build --release --no-default-features
fi

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ Build successful!"
    echo ""
    echo "📦 Binaries:"
    echo "  - target/release/riptide       (main proxy server)"
    echo "  - target/release/test-client   (test client)"
    echo ""
    echo "🚀 Quick start:"
    echo "  ./target/release/riptide --config config.json"
    echo ""
    echo "🧪 Test:"
    echo "  ./target/release/test-client -u user -P pass -c 10"
else
    echo ""
    echo "❌ Build failed"
    exit 1
fi

