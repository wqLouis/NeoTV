# TODO / Future Features

## Canvas Player (Embedded Video Renderer)

A Rust-based video player that renders directly to frontend canvas, bypassing WebView MSE limitations.

### Motivation

Linux WebView (GTK2/WebKit) has incomplete MSE support. When HLS.js fails with codec/buffer errors on Linux, an alternative is needed that doesn't rely on external players like MPV.

### Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                         Rust Backend                         │
│                                                              │
│  ┌──────────┐   HLS/m3u8   ┌─────────────┐                   │
│  │  FFmpeg  │ ◄───────────► │  Decoder   │                   │
│  └────┬─────┘              └──────┬──────┘                   │
│       │                           │                          │
│       │ Video frames              │ Audio                     │
│       │ (RGB data)                │ (PCM/原生)               │
│       ▼                           ▼                          │
│  ┌─────────────┐            ┌──────────────┐                │
│  │  Tauri IPC  │            │ System Audio │                │
│  │  (binary)   │            │ (ALSA/Pulse) │                │
│  └──────┬──────┘            └──────────────┘                │
└─────────┼───────────────────────────────────────────────────┘
          │ app.emit("canvas-frame", FrameData)
          ▼
┌──────────────────────────────────────────────────────────────┐
│                        Frontend                              │
│                                                              │
│  ┌────────────┐     ┌──────────────┐                        │
│  │  <canvas>  │ ◄── │  Frame data  │                        │
│  │  rendering │     │  (timestamp)  │                        │
│  └────────────┘     └──────────────┘                        │
│                                                              │
│  Audio sync: canvas displays frame matching                  │
│  audio.currentTime                                           │
└──────────────────────────────────────────────────────────────┘
```

### Data Structures

```rust
// Frame data transmitted via Tauri binary event
struct FrameData {
    data: Vec<u8>,      // Raw RGB pixels (no compression)
    width: u32,
    height: u32,
    timestamp_ms: i64,  // For audio sync
    frame_number: u64,
}

// Tauri command responses
enum CanvasPlayerCommand {
    Start { id: String, m3u8_url: String, referer: Option<String> },
    Pause { id: String },
    Resume { id: String },
    Seek { id: String, timestamp_ms: i64 },
    Stop { id: String },
}
```

### Implementation Steps

#### 1. Rust Backend - Core

- [ ] Add `canvas_player.rs` module
- [ ] Implement `start_canvas_player` command
- [ ] FFmpeg pipeline:
  - Open HLS stream via `libav` or `ffmpeg-next` crate
  - Decode video frames to RGB
  - Decode audio and output to system (ALSA/PulseAudio)
- [ ] Implement `canvas_player_control` command (pause/resume/seek)
- [ ] Implement `stop_canvas_player` command
- [ ] Emit frames via `app.emit("canvas-frame", frame_data)`

#### 2. Frontend - Core

- [ ] Add `useCanvasPlayer` setting (toggle in settings page)
- [ ] Create `CanvasPlayer.svelte` component:
  - `onMount`: subscribe to `canvas-frame` events
  - `requestAnimationFrame` loop rendering frames to canvas
  - Audio sync: fetch current audio time via Tauri command or IPC
- [ ] Implement player controls (pause/resume/seek)
- [ ] Connect to settings toggle

#### 3. VideoPlayer Integration

- [ ] Detect `useCanvasPlayer` setting
- [ ] When enabled and HLS fails → use CanvasPlayer instead of error
- [ ] Or always use CanvasPlayer when setting is enabled

#### 4. Audio Sync

- [ ] Option A: Rust controls audio playback (mpv or FFmpeg with audio output)
  - Frontend queries current position via Tauri command
  - Canvas displays frame closest to audio position
- [ ] Option B: Frontend HTML Audio + Rust video frames
  - Audio plays natively via browser
  - Video frames stamped and synced to audio.currentTime

#### 5. Performance Considerations

- [ ] Test with 1080p60fps raw RGB transfer (~375MB/s)
- [ ] If bottleneck: implement shared memory
- [ ] If bottleneck: consider MJPEG compression before transfer

### Shared Memory Option (If Needed)

If Tauri IPC bandwidth is insufficient:

```
┌─────────────────────────────────────────────────────────────┐
│  Shared Memory (mmap)                                        │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │ Ring buffer: [frame1][frame2][frame3]...              │ │
│  │             ▲read              ▲write                  │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
         │ read              │ write
         ▼                   ▼
   ┌────────────┐       ┌────────────┐
   │  Canvas    │       │   FFmpeg   │
   │  (Frontend)│       │  (Rust)   │
   └────────────┘       └────────────┘
```

### Settings UI

```svelte
// Setting toggle in settings page
<Switch
	bind:checked={settings.useCanvasPlayer}
	label="启用 Canvas 播放器"
	description="使用 Rust 后端渲染视频到 Canvas，绕过 WebView 限制"
/>
```

### Dependencies (Rust)

```toml
# Cargo.toml
ffmpeg-next = "7.0"  # or libav
```

### Platform Support

| Platform | Audio Output    | Notes                          |
| -------- | --------------- | ------------------------------ |
| Linux    | ALSA/PulseAudio | FFmpeg `-f alsa` or `-f pulse` |
| Windows  | WASAPI          | FFmpeg `-f wasapi`             |
| Android  | OpenSL ES       | Via FFmpeg or mpv              |

### Comparison with Current Approaches

| Approach                 | Pros                     | Cons                          |
| ------------------------ | ------------------------ | ----------------------------- |
| HLS.js                   | Native, no transcoding   | Fails on incomplete MSE       |
| FFmpeg transcoded stream | Works anywhere           | Opens local HTTP port         |
| MPV external player      | Best codec support       | Not embedded, separate window |
| Canvas Player            | Fully embedded, no ports | Complex, high bandwidth       |

### Status

**Not started** - This is a future enhancement for when standard HLS playback fails on platforms with limited MSE support.
