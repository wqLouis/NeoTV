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
| Framework  | Tauri 2.x            | Native desktop/mobile app shell |
| Frontend   | Svelte 5 + TypeScript | Reactive UI                     |
| Styling    | TailwindCSS           | Utility-first CSS               |
| Navigation | lrud-spatial (BBC)   | Spatial TV remote navigation    |
| Video      | hls.js + FFmpeg      | HLS playback with transcoding   |
| State      | Svelte Stores        | Reactive state management       |

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

| Document | Description |
|----------|-------------|
| [ARCHITECTURE.md](ARCHITECTURE.md) | System architecture and design overview |
| [STORES.md](STORES.md) | State management with Svelte stores |
| [SIGNING.md](SIGNING.md) | Windows code signing setup for CI/CD builds |

## Build Scripts

| Platform | Command | Output |
|----------|---------|--------|
| Linux | `./scripts/build-linux.sh` | `dist-linux/neotv` |
| Windows | `./scripts/build-windows.sh` | `dist-windows/neotv.exe` |
| Android | `./scripts/build-android.sh` | APK |

For cross-compiling from Linux to Windows:
```bash
./scripts/build-windows.sh
```

## Navigation

TV navigation is enabled by default. Use arrow keys to navigate:

- **Arrow keys** - Move focus between elements
- **Enter** - Activate focused element (click)
- **TV Nav Mode** - Can be toggled in Settings → 外观 (Appearance)

## Project Structure

```
LibreTV/
├── src/                      # Frontend (SvelteKit)
│   ├── lib/
│   │   ├── api/             # API clients
│   │   ├── components/      # UI components
│   │   │   ├── business/    # Business logic components
│   │   │   └── ui/         # shadcn-svelte UI components
│   │   └── stores/          # Svelte stores
│   └── routes/              # SvelteKit pages
├── src-tauri/               # Rust backend
└── docs/                    # Documentation
```

## License

Apache-2.0
