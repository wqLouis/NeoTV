# NeoTV Architecture

## Overview

NeoTV is a modern video streaming application built with **Tauri 2.x** (Rust backend) and **Svelte 5** (TypeScript frontend). It supports Windows, macOS, Linux, and Android with **TV remote navigation** via lrud-spatial.

## Tech Stack

| Layer       | Technology           | Purpose                           |
| ----------- | -------------------- | --------------------------------- |
| Framework   | Tauri 2.x            | Native desktop/mobile app shell   |
| Frontend    | Svelte 5, TypeScript | Reactive UI                       |
| Styling     | TailwindCSS          | Utility-first CSS                 |
| Navigation  | lrud-spatial (BBC)   | Spatial TV remote navigation      |
| Video       | hls.js + FFmpeg      | HLS stream playback + transcoding |
| Transcoding | FFmpeg               | Hardware-accelerated transcoding  |
| State       | Svelte Stores        | Reactive state management         |

## Project Structure

```
NeoTV/
├── src/                          # Frontend (SvelteKit)
│   ├── lib/
│   │   ├── api/                 # API clients
│   │   │   ├── constants.ts     # API endpoints config
│   │   │   ├── douban.ts        # Douban API integration
│   │   │   └── search.ts        # Video search
│   │   ├── components/          # UI components
│   │   │   ├── VideoPlayer.svelte
│   │   │   ├── VideoCard.svelte
│   │   │   ├── VideoDetailModal.svelte
│   │   │   ├── VideoSourceOverlay.svelte
│   │   │   ├── EpisodeList.svelte
│   │   │   ├── SearchBar.svelte
│   │   │   ├── DoubanCard.svelte
│   │   │   ├── FocusRing.svelte  # Visual focus indicator
│   │   │   └── CachedImage.svelte
│   │   ├── stores/             # Svelte stores (reactive state)
│   │   │   ├── settings.svelte.ts
│   │   │   ├── history.svelte.ts
│   │   │   ├── favourites.svelte.ts
│   │   │   ├── search.svelte.ts
│   │   │   └── theme.svelte.ts
│   │   ├── utils/
│   │   │   └── speedTest.ts    # Source speed testing
│   │   ├── cache.ts            # Image caching logic
│   │   └── utils.ts            # General utilities
│   └── routes/                 # SvelteKit pages
│       ├── +layout.svelte      # App shell with sidebar + navigation
│       ├── +page.svelte        # Home (Douban recommendations)
│       ├── search/             # Search page
│       ├── browse/             # Browse page
│       ├── player/              # Video player page
│       ├── history/             # Watch history
│       ├── favourites/          # Favorites
│       └── settings/            # Settings
├── src-tauri/                  # Rust backend
│   ├── src/
│   │   ├── lib.rs              # Tauri app setup, plugin registration
│   │   ├── main.rs             # Entry point (just calls lib::run)
│   │   ├── commands.rs          # Tauri command handlers
│   │   ├── api.rs              # HTTP request handling
│   │   ├── cache.rs            # Memory + disk caching
│   │   ├── config.rs           # API source configuration
│   │   ├── m3u8.rs             # M3U8 parsing, ad filtering, URL rewriting
│   │   ├── transcoder.rs       # FFmpeg streaming transcoder
│   │   └── logging.rs          # Logging utilities
│   └── plugins/
│       └── tauri-plugin-immersive-android/  # Android immersive mode
├── scripts/
│   └── build-android.sh        # Android APK build script
└── docs/                       # Documentation
```

## Architecture Layers

### 1. Frontend (Svelte 5)

The frontend is a SvelteKit application that handles:

- **UI rendering**: Components, layouts, theming
- **State management**: Svelte stores for settings, history, favorites
- **TV Navigation**: lrud-spatial for arrow key navigation with visual focus ring
- **Video player**: HLS.js integration with custom RustLoader
- **API calls**: Communicates with Rust backend via Tauri invoke

#### Key Components

- **VideoPlayer.svelte**: Main player with HLS.js, custom RustLoader, error recovery
- **VideoCard.svelte**: Thumbnail card for video lists
- **DoubanCard.svelte**: Card component for Douban recommendations (focusable with `tabindex="0"`)
- **VideoSourceOverlay.svelte**: Source selection overlay
- **CachedImage.svelte**: Image with disk caching
- **FocusRing.svelte**: Visual indicator for TV navigation focus

#### TV Navigation

TV navigation is powered by [lrud-spatial](https://github.com/bbc/lrud-spatial) from BBC:

- **Focusable elements**: Buttons, links, inputs, and elements with `tabindex >= 0`
- **Spatial sorting**: Automatically finds closest element in movement direction
- **Containers**: Recognizes `nav`, `section`, `.lrud-container` as navigation scopes
- **Visual feedback**: White border ring follows focused element

TV Nav Mode can be toggled in **Settings → 外观 → TV 导航模式** (enabled by default).

#### Routes

| Route            | Purpose                       |
| ---------------- | ----------------------------- |
| `/`              | Home - Douban recommendations |
| `/search`        | Search videos across sources  |
| `/browse`        | Browse all videos             |
| `/player?id=...` | Video player page             |
| `/history`       | Watch history                 |
| `/favourites`    | Saved favorites               |
| `/settings`      | App settings                  |

### 2. Rust Backend (Tauri)

The Rust backend provides:

- **HTTP proxying**: Fetch remote content with proper headers
- **M3U8 processing**: Parse, filter ads, rewrite URLs
- **Transcoding**: FFmpeg pipeline for incompatible streams
- **Caching**: Memory + disk cache for images and data
- **Platform integration**: Android immersive mode

#### Key Modules

| Module          | Responsibility                                      |
| --------------- | --------------------------------------------------- |
| `commands.rs`   | Tauri command handlers exposed to frontend          |
| `api.rs`        | HTTP request execution with headers                 |
| `m3u8.rs`       | M3U8 playlist parsing, ad filtering, URL resolution |
| `transcoder.rs` | FFmpeg streaming pipeline                           |
| `cache.rs`      | Memory and disk cache management                    |
| `config.rs`     | Video API source configuration                      |

### 3. State Management (Svelte Stores)

| Store                | Purpose                                     |
| -------------------- | ------------------------------------------- |
| `settingsStore`      | User preferences, TV nav mode, filters      |
| `historyStore`       | Watch history with localStorage persistence |
| `favouritesStore`    | Saved favorite videos                       |
| `searchHistoryStore` | Search query history                        |
| `themeStore`         | Dark/light theme management                 |

See [STORES.md](STORES.md) for detailed store documentation.

### 4. Video Playback Architecture

```
User Selects Video
       │
       ▼
processVideoUrl()        # src/routes/player/+page.svelte
       │
       ├─── NOT M3U8 ────────► playerType = 'native'
       │
       └─── M3U8 ────────────► invoke('fetch_media_url')
                                   │
                ┌──────────────────┴──────────────────┐
                ▼                                     ▼
          Success                                  Error
                │                                     │
                ▼                                     ▼
       playerType = 'hls'               playerType = 'hls'
       playerSrc = processed URL        playerSrc = original URL
                │                                     │
                ▼                                     ▼
       VideoPlayer Component ◄────────────────────────┘
              │
              ├─── type='native' ────► HTML5 <video>
              │
              └─── type='hls' ───────► initHls()
                                           │
                       ┌───────────────────┼───────────────────┐
                       ▼                   ▼                   ▼
               Hls.isSupported()    Network Error      Media Error
                       │                   │                   │
                       ▼                   ▼                   ▼
               initHls.js with      initTranscoded()   hls.recoverMediaError()
               RustLoader                               │
                                                          ▼
                                                 If fatal ──► Stop (no FFmpeg on mobile)
```

See [VIDEO_PLAYER.md](VIDEO_PLAYER.md) for detailed player documentation.

### 5. Caching Strategy

#### Image Caching

- **Location**: `src-tauri/src/cache.rs`
- **Memory cache**: LRU with configurable size limit
- **Disk cache**: `image_cache/` directory
- **TTL**: 1 hour for cached images
- **Frontend**: `src/lib/cache.ts` provides `cachedFetch()` wrapper

#### Data Caching

- HTTP responses cached in memory for 1 hour
- Disk cache for larger responses

## Data Flow

### Video Search Flow

```
User Input → /search page → search.ts API call
     │
     ▼
invoke('search_videos') ──► Rust commands.rs
     │                           │
     │                           ▼
     │                      config.rs (API source config)
     │                           │
     │                           ▼
     │                      api.rs (HTTP request)
     │                           │
     ▼                           ▼
◄───────────────────────────────┘
     │
     ▼
Svelte store → Render results
```

### Video Playback Flow

```
Episode Select → invoke('fetch_media_url')
     │
     ▼
m3u8.rs: fetch_and_process_m3u8()
     │
     ├── Fetch M3U8 content
     ├── Detect if master or media playlist
     ├── Select best quality variant
     ├── Filter ad segments
     └── Rewrite segment URLs
     │
     ▼
Return MediaInfo { url, headers }
     │
     ▼
VideoPlayer: RustLoader intercepts segment requests
     │
     ├── manifest/level → invoke('fetch_hls_m3u8')
     └── segments       → invoke('fetch_hls_segment')
```

### TV Navigation Flow

```
User presses Arrow Key
        │
        ▼
+layout.svelte: handleKeydown()
        │
        ├── Check settingsStore.tvNavModeEnabled
        │
        ▼
getNextFocus(currentFocus, 'ArrowRight')
        │
        ▼
lrud-spatial: Find all focusable candidates
        │
        ▼
Sort by spatial distance in direction
        │
        ▼
Return closest candidate → .focus()
        │
        ▼
FocusRing component tracks focus changes
```

See [NAVIGATION.md](NAVIGATION.md) for detailed navigation documentation.

## Android Specifics

- **Immersive mode**: Fullscreen without system bars
- **Plugin**: `tauri-plugin-immersive-android`
- **Min SDK**: 24 (Android 7.0)

See [ANDROID.md](ANDROID.md) for details.

## Build Targets

| Platform | Command                      | Output              |
| -------- | ---------------------------- | ------------------- |
| Windows  | `bun tauri build`            | `.exe`, `.msi`      |
| macOS    | `bun tauri build`            | `.app`, `.dmg`      |
| Linux    | `bun tauri build`            | `.deb`, `.AppImage` |
| Android  | `./scripts/build-android.sh` | `.apk`              |

See [BUILDING.md](BUILDING.md) for detailed build instructions.

## Key Files Reference

| File                                                | Purpose                                 |
| --------------------------------------------------- | --------------------------------------- |
| `src/lib/components/VideoPlayer.svelte`             | Video player with HLS.js and RustLoader |
| `src/lib/components/FocusRing.svelte`               | Visual focus indicator for TV nav       |
| `src/routes/+layout.svelte`                         | App shell with navigation handling      |
| `src/routes/player/+page.svelte`                    | Player page, URL processing             |
| `src-tauri/src/commands.rs`                         | All Tauri commands                      |
| `src-tauri/src/m3u8.rs`                             | M3U8 parsing and ad filtering           |
| `src-tauri/src/transcoder.rs`                       | FFmpeg transcoding pipeline             |
| `src-tauri/src/cache.rs`                            | Memory and disk cache                   |
| `src-tauri/plugins/tauri-plugin-immersive-android/` | Android fullscreen                      |
| `src/lib/stores/settings.svelte.ts`                 | Settings including TV nav mode          |
