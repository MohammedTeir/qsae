#!/bin/bash
set -e

echo "=== QSAE Phase 1 Build Verification ==="
echo ""

# Check Rust installation
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Install Rust: https://rustup.rs"
    exit 1
fi

echo "✓ Rust toolchain found: $(cargo --version)"
echo "✓ rustc: $(rustc --version)"
echo ""

# Check project structure
echo "Checking project structure..."
for file in     "crates/qsae-core/src/lib.rs"     "crates/qsae-core/src/compressor.rs"     "crates/qsae-core/src/decompressor.rs"     "crates/qsae-cli/src/main.rs"     "Cargo.toml"; do
    if [ -f "$file" ]; then
        echo "  ✓ $file"
    else
        echo "  ❌ Missing: $file"
        exit 1
    fi
done
echo ""

# Run cargo check
echo "Running cargo check..."
if cargo check 2>&1; then
    echo "✓ cargo check passed"
else
    echo "❌ cargo check failed"
    exit 1
fi
echo ""

# Run tests
echo "Running cargo test..."
if cargo test 2>&1; then
    echo "✓ All tests passed"
else
    echo "❌ Tests failed"
    exit 1
fi
echo ""

# Build release binary
echo "Building release binary..."
if cargo build --release 2>&1; then
    echo "✓ Release build successful"
    echo ""
    echo "Binary location: ./target/release/qsae"
    echo ""
    echo "Quick test:"
    echo "  echo 'Hello QSAE' > /tmp/test.txt"
    echo "  ./target/release/qsae compress /tmp/test.txt /tmp/test.qsae"
    echo "  ./target/release/qsae decompress /tmp/test.qsae /tmp/test2.txt"
    echo "  ./target/release/qsae inspect /tmp/test.qsae"
else
    echo "❌ Release build failed"
    exit 1
fi
