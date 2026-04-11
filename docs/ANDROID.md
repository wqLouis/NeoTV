# Android Platform

## Overview

NeoTV supports Android via Tauri's mobile build system. The app targets Android 7.0+ (API 24+).

## Requirements

- Android SDK (API 34 recommended)
- Android NDK (r25b or compatible)
- Java 17+
- Rust

## Build Configuration

### tauri.conf.json

```json
{
	"bundle": {
		"android": {
			"minSdkVersion": 24
		}
	}
}
```

### Environment Variables

```bash
export ANDROID_HOME="$HOME/Android/Sdk"
export ANDROID_SDK_ROOT="$ANDROID_HOME"
export NDK_HOME="$ANDROID_SDK/ndk/25.2.4759257"
export JAVA_HOME="/usr/lib/jvm/java-17-openjdk"
```

## Building

### Quick Build

```bash
# Development build
npx tauri android dev

# Production build
./scripts/build-android.sh
```

### Manual Build

```bash
# Initialize Android project (first time only)
npx tauri android init

# Build APK
npx tauri android build

# Sign APK
apksigner sign --ks release-key.jks --out NeoTV.apk app-universal-release-unsigned.apk
```

## Android-Specific Features

### Immersive Mode

The app uses fullscreen immersive mode for video playback, hiding system bars.

**Implementation:**

- Rust plugin: `src-tauri/plugins/tauri-plugin-immersive-android/`
- Kotlin class: `com.libretv.app.immersive.ImmersiveAndroidPlugin`

#### Usage

```typescript
import { invoke } from '@tauri-apps/api/core';

// Enter fullscreen
await invoke('set_immersive_android', { enabled: true });

// Exit fullscreen
await invoke('set_immersive_android', { enabled: false });
```

#### Kotlin Implementation

```kotlin
// com.libretv.app.immersive.ImmersiveAndroidPlugin
class ImmersiveAndroidPlugin {
    companion object {
        @JvmStatic
        fun setImmersive(activity: Activity, enabled: Boolean) {
            activity.runOnUiThread {
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
                    val controller = WindowCompat.getInsetsController(window, decorView)
                    if (enabled) {
                        controller.hide(WindowInsetsCompat.Type.systemBars())
                        controller.systemBarsBehavior = BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
                    } else {
                        controller.show(WindowInsetsCompat.Type.systemBars())
                    }
                } else {
                    // Legacy immersive mode for older Android
                    var options = decorView.systemUiVisibility
                    options = options or View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY
                    // ... flags setup
                }
            }
        }
    }
}
```

### Permissions

The following permissions are configured:

```xml
<!-- AndroidManifest.xml -->
<uses-permission android:name="android.permission.INTERNET" />
<uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />
```

### ABI Configuration

By default, the build creates a universal APK supporting all architectures. To reduce APK size, configure specific ABIs in `tauri.conf.json`:

```json
{
	"bundle": {
		"android": {
			"abi": ["arm64-v8a", "x86_64"]
		}
	}
}
```

Supported ABIs:

- `arm64-v8a` - 64-bit ARM (recommended)
- `armeabi-v7a` - 32-bit ARM
- `x86_64` - 64-bit x86
- `x86` - 32-bit x86

## APK Output

After build, the APK is located at:

```
src-tauri/gen/android/app/build/outputs/apk/universal/release/
└── app-universal-release-unsigned.apk
```

The `build-android.sh` script automatically signs the APK to produce `NeoTV.apk`.

## Troubleshooting

### Build fails with NDK not found

Ensure `NDK_HOME` environment variable points to a valid NDK directory:

```bash
export NDK_HOME="$ANDROID_SDK/ndk/25.2.4759257"
```

### App crashes on launch

Check logcat for errors:

```bash
adb logcat | grep -i "libretv\|neotv\|tauri"
```

### Video playback fails

- Verify internet permission in manifest
- Check `usesCleartextTraffic` for HTTP sources
- FFmpeg transcoding is NOT available on Android (no fallback)

## Key Files

| File                                                          | Purpose                      |
| ------------------------------------------------------------- | ---------------------------- |
| `scripts/build-android.sh`                                    | Android build script         |
| `src-tauri/plugins/tauri-plugin-immersive-android/`           | Immersive mode plugin        |
| `src-tauri/plugins/tauri-plugin-immersive-android/src/lib.rs` | Rust plugin entry            |
| `src-tauri/plugins/.../ImmersiveAndroidPlugin.kt`             | Kotlin implementation        |
| `src/routes/player/+page.svelte`                              | Uses `set_immersive_android` |
