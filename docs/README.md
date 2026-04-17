# LibreTV

A modern video streaming application for desktop and TV devices with TV remote navigation support.

## Features

- **Multi-source video search** across multiple free video APIs
- **TV remote navigation** - Navigate with arrow keys using spatial navigation (lrud-spatial)
- **HLS video playback** with ad filtering via FFmpeg
- **Douban recommendations** on the home page
- **History & Favorites** with local storage persistence
- **Cross-platform** - Windows, macOS, Linux, and Android support

## Tech Stack

| Layer      | Technology            | Purpose                         |
| ---------- | --------------------- | ------------------------------- |
| Framework  | Tauri 2.x             | Native desktop/mobile app shell |
| Frontend   | Svelte 5 + TypeScript | Reactive UI                     |
| Styling    | TailwindCSS           | Utility-first CSS               |
| Navigation | lrud-spatial (BBC)    | Spatial TV remote navigation    |
| Video      | hls.js + FFmpeg       | HLS playback with transcoding   |
| State      | Svelte Stores         | Reactive state management       |

## Quick Start

```bash
# Install dependencies
bun install

# Start development server
bun run dev

# Build for production
bun run build
```

## Documentation

### Getting Started

| Document                           | Description                             |
| ---------------------------------- | --------------------------------------- |
| [ARCHITECTURE.md](ARCHITECTURE.md) | System architecture and design overview |
| [NAVIGATION.md](NAVIGATION.md)     | TV remote navigation system guide       |

### Features

| Document                           | Description                         |
| ---------------------------------- | ----------------------------------- |
| [STORES.md](STORES.md)             | State management with Svelte stores |
| [VIDEO_PLAYER.md](VIDEO_PLAYER.md) | HLS video playback architecture     |
| [CACHE.md](CACHE.md)               | Image and data caching strategy     |

### Development

| Document                               | Description                          |
| -------------------------------------- | ------------------------------------ |
| [BUILDING.md](BUILDING.md)             | Build instructions for all platforms |
| [TAURI_COMMANDS.md](TAURI_COMMANDS.md) | Rust backend command reference       |
| [STORES.md](STORES.md)                 | Store interfaces and methods         |

### Reference

| Document                   | Description                    |
| -------------------------- | ------------------------------ |
| [ANDROID.md](ANDROID.md)   | Android-specific configuration |
| [I18N.md](I18N.md)         | Internationalization setup     |
| [PROTOCOL.md](PROTOCOL.md) | Network protocol details       |
| [TODO.md](TODO.md)         | Planned features and tasks     |

## Navigation

TV navigation is enabled by default. Use arrow keys to navigate:

- **Arrow keys** - Move focus between elements
- **Enter** - Activate focused element (click)
- **TV Nav Mode** - Can be toggled in Settings → 外观 (Appearance)

See [NAVIGATION.md](NAVIGATION.md) for detailed documentation.

## Project Structure

```
LibreTV/
├── src/                      # Frontend (SvelteKit)
│   ├── lib/
│   │   ├── api/             # API clients
│   │   ├── components/      # UI components
│   │   │   ├── business/     # Business logic components
│   │   │   └── ui/          # shadcn-svelte UI components
│   │   └── stores/          # Svelte stores
│   └── routes/              # SvelteKit pages
├── src-tauri/               # Rust backend
└── docs/                    # Documentation
```

## License

Apache-2.0
