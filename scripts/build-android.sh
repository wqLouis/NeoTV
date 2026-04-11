#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_DIR"

USER_HOME="${HOME:-/home/$USER}"
ANDROID_SDK="${ANDROID_HOME:-$USER_HOME/Android/Sdk}"
NDK_HOME="${NDK_HOME:-$ANDROID_SDK/ndk/25.2.4759257}"
JAVA_HOME="${JAVA_HOME:-/usr/lib/jvm/java-17-openjdk}"

export ANDROID_HOME
export ANDROID_SDK_ROOT="$ANDROID_SDK"
export NDK_HOME
export JAVA_HOME
export PATH="$ANDROID_SDK/cmdline-tools/latest/bin:$ANDROID_SDK/platform-tools:$PATH"

SDK_MANAGER="$ANDROID_SDK/cmdline-tools/latest/bin/sdkmanager"
APKSIGNER="$ANDROID_SDK/build-tools/34.0.0/apksigner"
KEYSTORE="$SCRIPT_DIR/release-key.jks"
ALIAS="libretv-key"

echo "[Build] Android SDK: $ANDROID_SDK"
echo "[Build] NDK: $NDK_HOME"
echo "[Build] Java: $JAVA_HOME"

if [ ! -d "$ANDROID_SDK/licenses" ]; then
    echo "[Build] Creating Android SDK licenses..."
    mkdir -p "$ANDROID_SDK/licenses"
fi

echo -e "\n24333f8a63b6825ea9c5514f83c2829b004d1fee" > "$ANDROID_SDK/licenses/android-sdk-license"
echo -e "\n84831b9409646a918e30573bab4c9c91346d8abd" > "$ANDROID_SDK/licenses/android-sdk-preview-license"
echo -e "\nd975f751698a77b662f1254ddbeed3901e976f5a" > "$ANDROID_SDK/licenses/intel-android-extra-license"

if [ -f "$SDK_MANAGER" ]; then
    echo "[Build] Installing required SDK components..."
    $SDK_MANAGER --sdk_root="$ANDROID_SDK" "platforms;android-34" "build-tools;34.0.0" || true
fi

echo "[Build] Building frontend..."
bun run build

echo "[Build] Building Android APK..."
bunx tauri android build --target aarch64

echo "[Build] Signing APK..."
UNSIGNED_APK="src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk"
SIGNED_APK="NeoTV.apk"

if [ ! -f "$KEYSTORE" ]; then
    echo "[Build] Creating keystore..."
    keytool -genkey -v -keystore "$KEYSTORE" -keyalg RSA -keysize 2048 -validity 10000 -alias "$ALIAS" -storepass android -keypass android -dname "CN=LibreTV, OU=LibreTV, O=LibreTV, L=Unknown, ST=Unknown, C=US"
fi

$APKSIGNER sign --ks "$KEYSTORE" --ks-pass pass:android --key-pass pass:android --out "$SIGNED_APK" "$UNSIGNED_APK"

echo "[Build] Done! Signed APK: $SIGNED_APK"
ls -lh "$SIGNED_APK"
