# NeoTV

A modern, lightweight video streaming application built with Tauri 2.x and Svelte 5.

## Features

- **Multi-Source Search** - Search videos across multiple API sources simultaneously
- **Douban Integration** - Browse movie and TV recommendations from Douban
- **Auto Source Selection** - Automatically test and select the best playable source
- **HLS Video Playback** - Smooth video playback with hls.js
- **Ad Filtering** - Automatically filter out ad segments in videos
- **Commentary Filtering** - Filter out commentary/review videos
- **Episode Selection** - Easy episode navigation for TV series
- **Source Switching** - Switch between different sources while watching
- **Speed Testing** - Test and optimize source selection
- **Dark/Light Theme** - Automatic theme switching based on system preference
- **Search History** - Track your search history
- **Favorites** - Save your favorite movies and TV shows
- **Image Caching** - Fast image loading with local caching

## Tech Stack

- **Frontend**: Svelte 5, TailwindCSS, TypeScript
- **Backend**: Tauri 2.x (Rust)
- **Video**: hls.js for HLS playback
- **Mobile**: Android (APK) support via Tauri

## Development

### Prerequisites

- Node.js 18+
- Rust 1.77+
- Android SDK (for Android builds)

### Setup

```bash
# Install dependencies
npm install

# Run in development mode
npm run dev

# Run Tauri in development mode
npx tauri dev
```

### Building

```bash
# Build web frontend
npm run build

# Build Tauri app for current platform
npx tauri build

# Build Android APK
./scripts/build-android.sh
```

## Project Structure

```
NeoTV/
├── src/                    # Frontend source
│   ├── lib/
│   │   ├── api/           # API clients (search, douban)
│   │   ├── components/    # Reusable UI components
│   │   ├── stores/        # Svelte stores (settings, history, favorites)
│   │   └── utils/         # Utility functions
│   └── routes/            # SvelteKit routes
│       ├── +layout.svelte # App layout with sidebar
│       ├── +page.svelte   # Home page (Douban recommendations)
│       ├── search/        # Search page
│       ├── player/        # Video player page
│       ├── history/       # Watch history
│       ├── favourites/    # Favorites page
│       └── settings/       # App settings
├── src-tauri/             # Rust backend
│   └── src/
│       ├── api.rs         # HTTP request handling
│       ├── cache.rs       # Image caching
│       ├── commands.rs    # Tauri commands
│       ├── m3u8.rs        # M3U8 playlist processing
│       └── transcoder.rs  # Video transcoding server
├── scripts/                # Build scripts
│   └── build-android.sh   # Android APK build script
└── docs/                  # Documentation
```

## Settings

### Content Filters

- **Yellow Filter** - Filter adult content from search results
- **Commentary Filter** - Filter out commentary/review videos
- **Ad Filtering** - Filter ad segments during video playback

### Source Management

- **Auto Integrate Sources** - Automatically test and select playable sources
- **Source Speed Test** - Test all sources and optimize selection

### Playback

- **Autoplay** - Automatically start playback on video load
- **Auto Continue** - Automatically play next episode

## Supported Platforms

- Desktop (Windows, macOS, Linux)
- Android (APK)
