# app-media:// Protocol

## Overview

The `app-media://` protocol is a custom URI scheme used by NeoTV for handling media streams through Tauri's IPC mechanism. This avoids the need for local HTTP servers.

## URI Format

```
app-media://{type}/{encoded_path}
```

## Types

| Type      | Example                                                      | Description             |
| --------- | ------------------------------------------------------------ | ----------------------- |
| `segment` | `app-media://segment/https%3A%2F%2Fexample.com%2Fseg.ts`     | Media segment (TS, M4S) |
| `m3u8`    | `app-media://m3u8/https%3A%2F%2Fexample.com%2Fplaylist.m3u8` | M3U8 playlist           |
| `stream`  | `app-media://stream/stream_1234567890`                       | Transcoded stream       |

## Implementation

### Rust Handler

The protocol is registered via Tauri's custom protocol handler in `src-tauri/src/lib.rs`:

```rust
.register_uri_scheme_protocol("app-media", |_app, request| {
    let uri = request.uri();
    // Parse type and path from URI
    // Route to appropriate handler
})
```

### Segment/M3U8 Flow

```
1. M3U8 processed by Rust (ad filtering, URL rewriting)
2. Segment URLs rewritten to app-media://segment/{encoded_url}
3. HLS.js requests segment via protocol handler
4. Handler fetches segment bytes, returns to HLS.js
```

### Transcoded Stream Flow

```
1. Frontend requests stream via start_transcoded_stream command
2. Rust starts FFmpeg pipeline, generates unique stream_id
3. Frontend receives app-media://stream/{stream_id} URL
4. Video element loads URL via protocol handler
5. Handler returns bytes from FFmpeg output pipe
6. Frontend plays stream in real-time
```

## Commands

### start_transcoded_stream

Creates a new transcoded stream.

```typescript
invoke<TranscoderInfo>('start_transcoded_stream', {
    id: string,
    m3u8_url: string,
    referer?: string
}): Promise<TranscoderInfo>
```

**Returns:**

```json
{
	"url": "app-media://stream/stream_1234567890",
	"port": 0,
	"duration": 7554.31,
	"vaapi_available": true,
	"ffmpeg_available": true
}
```

### fetch_hls_segment

Fetches a media segment.

```typescript
invoke<number[]>('fetch_hls_segment', {
    url: string  // May include app-media:// prefix
}): Promise<number[]>
```

### fetch_hls_m3u8

Fetches and processes an M3U8 playlist.

```typescript
invoke<string>('fetch_hls_m3u8', {
    url: string,
    ad_filtering?: boolean
}): Promise<string>
```

## Response Types

| Content          | MIME Type                       |
| ---------------- | ------------------------------- |
| M3U8 playlist    | `application/vnd.apple.mpegurl` |
| TS segment       | `video/mp2t`                    |
| M4S segment      | `video/mp4`                     |
| MP4 (transcoded) | `video/mp4`                     |

## URL Encoding

Special characters in URLs must be percent-encoded:

```javascript
const encoded = encodeURIComponent('https://example.com/seg.ts');
// Result: 'https%3A%2F%2Fexample.com%2Fseg.ts'

const uri = `app-media://segment/${encoded}`;
// Result: 'app-media://segment/https%3A%2F%2Fexample.com%2Fseg.ts'
```

## Decoding in Rust

```rust
fn decode_media_url(url: &str) -> String {
    if url.starts_with("app-media://") {
        url.strip_prefix("app-media://")
            .and_then(|e| urlencoding::decode(e).ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| url.to_string())
    } else {
        url.to_string()
    }
}
```

## Comparison with HTTP Streaming

| Aspect           | HTTP Server (:port) | app-media:// Protocol |
| ---------------- | ------------------- | --------------------- |
| Port binding     | Required            | Not needed            |
| Network exposure | localhost only      | None (in-process)     |
| Mobile support   | Problematic         | Native                |
| Permissions      | None                | Tauri permissions     |
| CORS issues      | Possible            | None                  |

## Security

- All `app-media://` requests are intercepted by Tauri
- No network exposure (unlike HTTP servers on ports)
- Participates in Tauri permission system
- URL-encoded paths prevent injection

## Usage in VideoPlayer

```typescript
// VideoPlayer.svelte uses fetch_hls_segment for segments

invoke<number[]>('fetch_hls_segment', { url: segmentUrl })
    .then(data => {
        const buffer = new Uint8Array(data).buffer;
        callbacks.onSuccess({ code: 200, data: buffer, ... });
    });
```

## Files

| File                                    | Role                  |
| --------------------------------------- | --------------------- |
| `src-tauri/src/lib.rs`                  | Protocol registration |
| `src-tauri/src/transcoder.rs`           | Stream pipeline       |
| `src-tauri/src/m3u8.rs`                 | M3U8 processing       |
| `src-tauri/src/commands.rs`             | Tauri commands        |
| `src/lib/components/VideoPlayer.svelte` | Frontend integration  |

## Platform Notes

### Android

The `app-media://` protocol works natively on Android through Tauri's WebView integration.

### Desktop

Works on all desktop platforms (Windows, macOS, Linux) through Tauri's custom protocol handler.

### Limitations

- FFmpeg transcoding is **not available** on Android (no `ffmpeg` binary)
- On Android, if HLS playback fails, the player shows an error instead of attempting FFmpeg fallback
