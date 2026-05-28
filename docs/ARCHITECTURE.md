# NeoTV Architecture

## Overview

A modern video streaming application built with **Tauri 2.x** (Rust backend) and **Svelte 5** (TypeScript frontend). Supports Windows, macOS, Linux, and Android with **TV remote navigation** via lrud-spatial.

## Tech Stack

| Layer       | Technology                          | Purpose                         |
|-------------|-------------------------------------|---------------------------------|
| Framework   | Tauri 2.x                           | Native desktop/mobile app shell |
| Frontend    | Svelte 5, TypeScript, runes         | Reactive UI                     |
| Styling     | TailwindCSS + shadcn-svelte         | Utility-first CSS + components  |
| Navigation  | lrud-spatial (BBC)                  | Spatial TV remote navigation    |
| Video       | hls.js + RustLoader                 | HLS stream playback             |
| i18n        | Paraglide (inlang)                  | Internationalization            |
| State       | Svelte `$state` stores             | Reactive state management       |
| HTTP        | reqwest (Rust)                      | Proxied HTTP requests           |
| Cache       | LRU in-memory + disk (Rust)         | Image & data caching            |
| Transcoding | FFmpeg                              | Hardware-accelerated transcoding|

## Project Structure

```
NeoTV/
├── src/                              # Frontend (SvelteKit)
│   ├── lib/
│   │   ├── api/                     # API clients
│   │   │   ├── constants.ts         # API endpoints & filter config
│   │   │   ├── douban.ts            # Douban API integration
│   │   │   └── search.ts            # Aggregated video search
│   │   ├── components/
│   │   │   ├── business/            # Domain-specific components
│   │   │   │   ├── ApiSelector.svelte
│   │   │   │   ├── EmptyState.svelte
│   │   │   │   ├── PageHeader.svelte
│   │   │   │   ├── PageTabBar.svelte
│   │   │   │   ├── ThemeSelector.svelte
│   │   │   │   └── ToggleButtonGroup.svelte
│   │   │   ├── CachedImage.svelte        # Lazy image with cache
│   │   │   ├── DoubanCard.svelte         # Card for Douban subjects
│   │   │   ├── FocusRing.svelte          # TV navigation focus ring
│   │   │   ├── HorizontalSection.svelte  # Horizontal scroll section
│   │   │   ├── Modal.svelte              # Generic modal dialog
│   │   │   ├── PlayerControls.svelte     # Video player HUD
│   │   │   ├── PlayerSettingsPopup.svelte # Playback settings sidebar
│   │   │   ├── SearchBar.svelte          # Search input
│   │   │   ├── TrafficLights.svelte      # Window controls (Win/Linux)
│   │   │   ├── VideoCard.svelte          # Search result card
│   │   │   ├── VideoPlayer.svelte        # HLS.js + RustLoader player
│   │   │   └── VideoSourceOverlay.svelte # Source selection overlay
│   │   │   └── ui/                      # shadcn-svelte components
│   │   ├── stores/                      # Reactive stores
│   │   │   ├── settings.svelte.ts
│   │   │   ├── history.svelte.ts
│   │   │   ├── favourites.svelte.ts
│   │   │   ├── search.svelte.ts
│   │   │   ├── theme.svelte.ts
│   │   │   └── modal.svelte.ts
│   │   ├── utils/
│   │   │   ├── format.ts                # Duration & relative time
│   │   │   ├── ranking.ts               # Search result ranking
│   │   │   └── speedTest.ts             # Source speed testing
│   │   ├── cache.ts                     # Image caching
│   │   └── utils.ts                     # cn() + type helpers
│   ├── routes/
│   │   ├── +layout.svelte               # App shell with sidebar nav
│   │   ├── +page.svelte                 # Home (Douban recommendations)
│   │   ├── browse/+page.svelte          # Browse (Douban genres)
│   │   ├── search/+page.svelte          # Search across sources
│   │   ├── player/+page.svelte          # Video player
│   │   ├── history/+page.svelte         # Watch history
│   │   ├── favourites/+page.svelte      # Saved favorites
│   │   └── settings/+page.svelte        # App settings
│   ├── hooks.ts                         # Paraglide reroute
│   ├── hooks.server.ts                  # Paraglide server handle
│   └── service-worker.js                # Basic service worker
├── src-tauri/                           # Rust backend
│   ├── src/
│   │   ├── lib.rs                       # Tauri app setup, plugin registration
│   │   ├── main.rs                      # Entry point (calls lib::run)
│   │   ├── commands.rs                  # Tauri command handlers
│   │   ├── api.rs                       # HTTP request execution
│   │   ├── cache.rs                     # LRU memory + disk cache
│   │   ├── config.rs                    # API source definitions
│   │   ├── error.rs                     # HttpError type
│   │   ├── gst_check.rs                 # Linux gst-libav detection
│   │   ├── http.rs                      # Reusable HTTP client
│   │   ├── m3u8.rs                      # M3U8 parsing, ad filtering, URL rewrite
│   │   ├── preloader.rs                 # HLS segment prefetcher (proactive cache)
│   │   └── storage.rs                   # History & favourites persistence
│   └── tauri.conf.json                  # Tauri configuration
├── scripts/                             # Build scripts
├── static/                              # Static assets (favicon, robots.txt)
└── docs/                                # Documentation
```

## Architecture Layers

### 1. Frontend (Svelte 5)

The frontend is a SvelteKit static app. It handles:

- **UI rendering**: Components, layouts, theming via TailwindCSS
- **State management**: Svelte 5 `$state` stores for settings, history, favorites
- **TV Navigation**: lrud-spatial for arrow key navigation with visual focus ring
- **Video player**: HLS.js integration with a custom `RustLoader` class that proxies segment requests through Rust
- **API calls**: All HTTP requests go through Tauri `invoke()` → Rust backend → remote server (avoids CORS)

#### Key Components

| Component | Purpose |
|-----------|---------|
| `VideoPlayer.svelte` | Main player: HLS.js, RustLoader, keyboard controls, error recovery |
| `VideoSourceOverlay.svelte` | Source selection panel — searches all APIs, ranks by relevance |
| `DoubanCard.svelte` | Reusable card for movie/TV show thumbnails |
| `HorizontalSection.svelte` | Horizontal scrolling section for home page |
| `CachedImage.svelte` | Lazy-loaded image with Rust disk caching |
| `FocusRing.svelte` | Visual indicator for TV navigation focus |
| `TrafficLights.svelte` | macOS-style window controls for Windows/Linux |

#### TV Navigation

TV navigation is powered by [lrud-spatial](https://github.com/bbc/lrud-spatial) from BBC:

- **Focusable elements**: Buttons, links, inputs, and elements with `tabindex >= 0`
- **Spatial sorting**: Automatically finds closest element in movement direction
- **Containers**: Recognizes `nav`, `section`, `.lrud-container` as navigation scopes
- **Visual feedback**: White border ring follows focused element

TV Nav Mode can be toggled in **Settings → 外观 → TV 导航模式** (enabled by default).

#### Routes

| Route | Purpose |
|-------|---------|
| `/` | Home — Douban recommendations (hot/new movies/TV) |
| `/browse` | Browse by genre (Douban categories) |
| `/search` | Search across all API sources or filter Douban |
| `/player?id=...` | Full-screen video player |
| `/history` | Watch history (persisted via Rust) |
| `/favourites` | Saved favorites (persisted via Rust) |
| `/settings` | App settings, API selection, speed test |

### 2. Rust Backend (Tauri)

The Rust backend provides everything the frontend can't do from the browser:

- **HTTP proxying** — fetch remote content with proper headers, avoid CORS
- **M3U8 processing** — parse playlists, filter ads, resolve variant playlists, rewrite relative URLs to absolute
- **Segment prefetching** — proactive HLS segment cache for instant playback
- **Caching** — LRU memory + disk cache for images and API responses
- **Persistence** — history and favorites stored as JSON files
- **Speed testing** — measure API source latency and throughput
- **Platform integration** — detect missing codecs (gst-libav on Linux), identify WiFi network, window controls

#### Key Modules

| Module | Responsibility |
|--------|---------------|
| `commands.rs` | All Tauri commands exposed to the frontend via `invoke()` |
| `api.rs` | HTTP request execution with custom headers, timeout, retries |
| `m3u8.rs` | M3U8 playlist parsing, ad segment filtering, variant selection, URL absolutification |
| `preloader.rs` | HLS segment prefetcher — proactively caches segments by index, only evicts played segments |
| `cache.rs` | LRU in-memory cache with optional disk persistence for images and API responses |
| `config.rs` | API source definitions (URLs, names) used by speed testing |
| `storage.rs` | JSON file-based persistence for history and favorites |
| `http.rs` | Shared HTTP client with connection pooling, used by cache and m3u8 modules |
| `error.rs` | `HttpError` type for fallible Tauri commands |
| `gst_check.rs` | Linux GStreamer libav codec detection (Linux-only) |

### 3. Video Playback Architecture

```
User clicks play on a source
       │
       ▼
VideoSourceOverlay → search all APIs → rank results → group by name
       │
       ▼
Navigate to /player?id=...&source=...
       │
       ▼
player/+page.svelte:
  processVideoUrl() → if M3U8, invoke('fetch_media_url')
       │                                │
       │                    ┌───────────┴───────────┐
       │                    ▼                       ▼
       │               Success                  Error (fallback)
       │                    │                       │
       ▼                    ▼                       ▼
  VideoPlayer.svelte ◄─────┴───────────────────────┘
       │
       ├── type='native' ────► HTML5 <video> element
       │
       └── type='hls' ───────► initHls()
                                    │
                            RustLoader class
                            (custom hls.js loader)
                                    │
                    ┌───────────────┼───────────────┐
                    ▼               ▼               ▼
          fetch_hls_m3u8     fetch_hls_segment   preloader
          (manifest)          (segment data)     (background)
                                                    │
                                              PRELOADER.start()
                                              pre-fetches segments
```

### 4. Segment Prefetcher (Preloader)

The preloader proactively caches HLS segments in the background so they're available instantly when hls.js requests them.

```
PRELOADER.start(content, base_url)
    │
    ├── Parse all segment URLs from M3U8
    ├── Spawn N workers (default: 6)
    └── Workers fetch segments sequentially

CacheData (single mutex)
├── entries: HashMap<url, CachedSegment>
├── lru: VecDeque (insertion order)
├── indices: HashMap<url, segment_index>
├── total_bytes
└── max_bytes

   played_up_to tracks highest segment requested by player
   └── eviction only removes segments with index < played_up_to
```

### 5. Caching Strategy

#### Image Cache
- **LRU memory cache** with disk persistence
- Images are base64-encoded and returned as data URLs
- TTL: 1 hour for cached responses

#### API Response Cache
- HTTP responses cached in memory for 10 minutes
- Used by `make_http_request` for search queries and Douban API calls
- Only caches non-video/audio content under 5MB

#### Speed Cache
- In-memory cache with disk persistence per WiFi network
- Used to speed up source ranking without re-testing
- 30-minute TTL

### 6. State Management (Stores)

| Store | Purpose | Persistence |
|-------|---------|-------------|
| `settings.svelte.ts` | User preferences, API selection, filters | localStorage |
| `history.svelte.ts` | Watch history | Rust JSON file |
| `favourites.svelte.ts` | Saved favorites | Rust JSON file |
| `search.svelte.ts` | Search query history | localStorage |
| `theme.svelte.ts` | Dark/light/system theme | localStorage |
| `modal.svelte.ts` | Modal dialog state | In-memory only |

See [STORES.md](STORES.md) for detailed store documentation.

### 7. Data Flow

#### Video Search Flow

```
User types query → /search page
    │
    ▼
SearchBar.svelte → handleSearch()
    │
    ├── API mode: invoke('make_http_request') for each API → aggregate → filter
    │
    └── Douban mode: invoke('make_http_request') to Douban API → filter
    │
    ▼
Render grid of VideoCard or DoubanCard components
```

#### TV Navigation Flow

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

## CI/CD

The GitHub workflow (`.github/workflows/build.yml`) builds both Linux and Windows binaries on Ubuntu runners:

| Job | Target | Signing |
|-----|--------|---------|
| `build-linux` | `x86_64-unknown-linux-gnu` | N/A |
| `build-windows` | `x86_64-pc-windows-gnu` (MinGW cross-compile) | osslsigncode if certificate configured |

See [SIGNING.md](SIGNING.md) for Windows code signing setup.

## Build Targets

| Platform | Command | Output |
|----------|---------|--------|
| Linux | `./scripts/build-linux.sh` | `dist-linux/neotv` |
| Windows | `./scripts/build-windows.sh` | `dist-windows/neotv.exe` |
| Android | `./scripts/build-android.sh` | APK |

## Key Files Reference

| File | Purpose |
|------|---------|
| `src/lib/components/VideoPlayer.svelte` | Video player with HLS.js and RustLoader |
| `src/lib/components/FocusRing.svelte` | Visual focus indicator for TV nav |
| `src/lib/components/TrafficLights.svelte` | Window controls for Windows/Linux |
| `src/routes/+layout.svelte` | App shell with navigation handling |
| `src/routes/player/+page.svelte` | Player page, URL processing |
| `src-tauri/src/commands.rs` | All Tauri commands |
| `src-tauri/src/m3u8.rs` | M3U8 parsing and ad filtering |
| `src-tauri/src/preloader.rs` | HLS segment prefetcher |
| `src-tauri/src/cache.rs` | Memory and disk cache |
| `src/lib/stores/settings.svelte.ts` | Settings including TV nav mode |
