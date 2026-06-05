#!/bin/bash
set -e

echo "=== Build Android ARM64 ==="

# 1. Install Rust Android target
echo "[1/4] Installing Rust Android target..."
rustup target add aarch64-linux-android

# 2. Install cargo-ndk if not present
if ! command -v cargo-ndk &> /dev/null; then
    echo "[2/4] Installing cargo-ndk..."
    cargo install cargo-ndk
else
    echo "[2/4] cargo-ndk already installed"
fi

# 3. Build frontend
echo "[3/4] Building frontend..."
cd frontend
npm install --silent
npm run build
cd ..

# 4. Cross-compile Rust library for Android ARM64
echo "[4/4] Cross-compiling Rust library..."
cd backend
cargo ndk \
    --target aarch64-linux-android \
    --platform 24 \
    --output-dir ../android/app/src/main/jniLibs \
    build --release

echo ""
echo "=== Done ==="
echo "APK can be built by opening android/ in Android Studio"
echo "or running: cd android && ./gradlew assembleDebug"
echo ""
echo "The .so file is at: android/app/src/main/jniLibs/arm64-v8a/libbackend.so"
