# Video Player Architecture

## Overview

The VideoPlayer component (`src/lib/components/VideoPlayer.svelte`) handles all video playback in NeoTV. It supports multiple playback strategies with automatic fallback.

## Playback Strategies

| Strategy          | Used When                    | Implementation                |
| ----------------- | ---------------------------- | ----------------------------- |
| Native            | Non-HLS URLs (MP4, WebM)     | HTML5 `<video>` element       |
| HLS.js            | HLS streams with MSE support | hls.js with custom RustLoader |
| FFmpeg Transcoder | HLS fallback (desktop only)  | FFmpeg pipeline via Rust      |

## Architecture Diagram

```
VideoPlayer Component
       │
       ├── Props
       │   ├── src: string              # Video URL
       │   ├── type: 'native' | 'hls'  # Playback type
       │   ├── autoplay: boolean
       │   ├── poster: string
       │   ├── episodes: Episode[]
       │   └── ... (callbacks)
       │
       ├── initHls()                   # HLS playback
       │       │
       │       ├── new Hls({ loader: RustLoader })
       │       ├── RustLoader intercepts all requests
       │       │       ├── manifest/level → invoke('fetch_hls_m3u8')
       │       │       └── segments       → invoke('fetch_hls_segment')
       │       │
       │       └── Error handling with recovery
       │
       ├── initNative()                # Native HTML5 playback
       │       └── videoEl.src = src
       │
       └── initTranscoded()            # FFmpeg (desktop only)
               ├── invoke('check_transcoder')
               ├── invoke('start_transcoded_stream')
               └── videoEl.src = stream_url
```

## Component Props

```typescript
interface Props {
	src: string; // Video URL
	type?: 'native' | 'hls'; // Playback type
	autoplay?: boolean; // Auto-play on load
	poster?: string; // Thumbnail image
	episodes?: Episode[]; // Episode list
	currentEpisodeIndex?: number; // Current episode
	playbackRate?: number; // Playback speed
	availableSources?: Source[]; // Alternative sources
	showFullscreenButton?: boolean; // Show fullscreen button

	// Callbacks
	onTimeUpdate?: (currentTime: number, duration: number) => void;
	onEnded?: () => void;
	onError?: (error: string) => void;
	onReady?: () => void;
	onEpisodeChange?: (episode: Episode, index: number) => void;
	onPlaybackRateChange?: (rate: number) => void;
	onReturn?: () => void;
	onSourceChange?: (source: Source) => void;
}

interface Episode {
	episode: string; // Episode label (e.g., "第1集")
	url: string; // Video URL
}
```

## RustLoader (Custom HLS.js Loader)

The `RustLoader` is a custom hls.js loader that routes all HLS requests through Tauri's Rust backend.

```typescript
class RustLoader implements Loader<LoaderContext> {
    load(context, config, callbacks) {
        const { type, url } = context;

        if (type === 'manifest' || type === 'level' || ...) {
            // Fetch M3U8 via Rust
            invoke<string>('fetch_hls_m3u8', { url })
                .then(content => callbacks.onSuccess(...))
                .catch(e => callbacks.onError(...));
        } else {
            // Fetch segment via Rust
            invoke<number[]>('fetch_hls_segment', { url })
                .then(data => {
                    const buffer = new Uint8Array(data).buffer;
                    callbacks.onSuccess({ code: 200, data: buffer, ... });
                })
                .catch(e => callbacks.onError(...));
        }
    }
}
```

### Request Routing

| Request Type                                  | Rust Command        | Purpose                     |
| --------------------------------------------- | ------------------- | --------------------------- |
| `manifest`, `level`, `audioTrack`, `subtitle` | `fetch_hls_m3u8`    | Fetch M3U8 playlist content |
| `segment` (TS, M4S)                           | `fetch_hls_segment` | Fetch segment bytes         |

## Error Recovery

The player implements a 3-tier error recovery strategy for HLS playback:

```typescript
let mediaErrorRecoveryCount = 0;
const MAX_MEDIA_ERROR_RECOVERIES = 3;
```

### MEDIA_ERROR Handling

```typescript
case Hls.ErrorTypes.MEDIA_ERROR:
    if (data.details === Hls.ErrorDetails.BUFFER_ADD_CODEC_ERROR) {
        mediaErrorRecoveryCount++;
        if (mediaErrorRecoveryCount <= MAX_MEDIA_ERROR_RECOVERIES) {
            hls.recoverMediaError();  // Retry
        } else {
            // Max retries reached - show error, NO FFmpeg fallback on mobile
            error = '视频解码失败，请尝试切换播放源';
        }
        return;
    }

    if (data.details === Hls.ErrorDetails.MANIFEST_INCOMPATIBLE_CODECS_ERROR) {
        // Same pattern - retry with recovery limit
        ...
        return;
    }

    hls.recoverMediaError();  // Generic recovery
    mediaErrorRecoveryCount = 0;  // Reset on success
    return;
```

### NETWORK_ERROR Handling

```typescript
case Hls.ErrorTypes.NETWORK_ERROR:
    // Fatal - FFmpeg fallback (desktop only)
    hls.destroy();
    initTranscoded();  // Attempts FFmpeg pipeline
    return;
```

### Platform-Specific Behavior

| Platform                      | FFmpeg Available | NETWORK_ERROR   | MEDIA_ERROR           |
| ----------------------------- | ---------------- | --------------- | --------------------- |
| Desktop (Windows/macOS/Linux) | Yes              | FFmpeg fallback | Retry (3x) then error |
| Android                       | No               | Error           | Retry (3x) then error |
| iOS                           | N/A              | Native HLS      | Native HLS            |

## FFmpeg Transcoding (Desktop Only)

The transcoder is started via `initTranscoded()` which calls:

1. `invoke('check_transcoder')` - Check FFmpeg availability
2. `invoke('start_transcoded_stream', { id, m3u8_url, referer })` - Start pipeline
3. Set `videoEl.src = streamInfo.url`

### Hardware Acceleration

FFmpeg pipeline prefers hardware acceleration:

```
1. VA-API (Intel GPU) - Linux/Windows
2. NVENC (NVIDIA GPU) - Windows
3. libx264 (Software) - Fallback
```

## Playback Flow

```
1. Component mounts with src + type props
2.
   ├── type='native' → initNative()
   │       └── videoEl.src = src
   │
   └── type='hls' → initHls()
           │
           ├── Hls.isSupported() = true
           │       ├── new Hls({ loader: RustLoader })
           │       ├── RustLoader intercepts requests
           │       ├── hls.loadSource(src)
           │       ├── hls.attachMedia(videoEl)
           │       └── MANIFEST_PARSED → autoplay
           │
           └── Hls.isSupported() = false
                   └── initTranscoded() (desktop only)
```

## Player Controls

The player includes custom controls overlay:

- Play/Pause button
- Volume slider with mute toggle
- Playback speed selector (0.5x - 2x)
- Episode list popup
- Fullscreen toggle
- Return button (for navigation)
- Progress bar with seek

## Key Files

| File                                    | Purpose                     |
| --------------------------------------- | --------------------------- |
| `src/lib/components/VideoPlayer.svelte` | Main player component       |
| `src/routes/player/+page.svelte`        | Player page, URL processing |
| `src-tauri/src/m3u8.rs`                 | M3U8 parsing, ad filtering  |
| `src-tauri/src/transcoder.rs`           | FFmpeg pipeline             |
| `src-tauri/src/commands.rs`             | `fetch_hls_*` commands      |

## Debugging

Enable debug logging by checking console output with `[HLS]` prefix:

```
[HLS] initHls called: { type: 'hls', src: '...' }
[HLS] Hls.isSupported() = true
[RustLoader] Fetching m3u8: https://...
[HLS] MANIFEST_PARSED event fired
[HLS] Error: { type: 'mediaError', details: 'bufferAddCodecError' }
[HLS] Codec error, recovery attempt 1/3
```
