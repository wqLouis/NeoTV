#!/bin/bash
set -e

echo "========================================"
echo "Building Linux binary for NeoTV"
echo "========================================"

cd "$(dirname "$0")"

echo "[1/3] Building frontend..."
npm run build

echo "[2/3] Building Tauri app (Linux)..."
cd src-tauri
cargo build --release --target x86_64-unknown-linux-gnu

echo "[3/3] Copying binary..."
mkdir -p ../dist-linux
cp target/x86_64-unknown-linux-gnu/release/app ../dist-linux/neotv

echo ""
echo "========================================"
echo "Build complete!"
echo "Binary: dist-linux/neotv"
echo ""
echo "To run with X11 (for MPV embedding):"
echo "  GDK_BACKEND=x11 GTK_BACKEND=x11 ./dist-linux/neotv"
echo "========================================"
