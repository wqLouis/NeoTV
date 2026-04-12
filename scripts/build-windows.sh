#!/bin/bash
set -e

echo "========================================"
echo "Building Windows binary on Linux"
echo "========================================"

cd "$(dirname "$0")"

echo "[1/3] Building frontend..."
npm run build

echo "[2/3] Building Tauri app (Windows cross-compile)..."
cd src-tauri
rustup target add x86_64-pc-windows-gnu 2>/dev/null || true
cargo build --release --target x86_64-pc-windows-gnu

echo "[3/3] Copying binary..."
mkdir -p ../dist-windows
cp target/x86_64-pc-windows-gnu/release/app.exe ../dist-windows/neotv.exe

echo ""
echo "========================================"
echo "Build complete!"
echo "Binary: dist-windows/neotv.exe"
echo "========================================"
