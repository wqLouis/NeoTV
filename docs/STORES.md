# Svelte Stores

NeoTV uses Svelte 5's `$state` runes for reactive state management. Settings and search history persist to `localStorage`; history and favourites persist via the Rust backend to JSON files.

## Store Index

| Store | File | Storage | Purpose |
|-------|------|---------|---------|
| `settingsStore` | `settings.svelte.ts` | `localStorage` | User preferences |
| `historyStore` | `history.svelte.ts` | Rust JSON file (via Tauri commands) | Watch history |
| `favouritesStore` | `favourites.svelte.ts` | Rust JSON file (via Tauri commands) | Saved favorites |
| `searchHistoryStore` | `search.svelte.ts` | `localStorage` | Search query history |
| `themeStore` | `theme.svelte.ts` | `localStorage` | Theme (light/dark/system) |
| `modalStore` | `modal.svelte.ts` | In-memory | Modal dialog state |

## Quick Usage

```typescript
import { settingsStore } from '$lib/stores/settings.svelte';

// Reactive reads
settingsStore.selectedApis;   // string[]
settingsStore.tvNavModeEnabled; // boolean
settingsStore.autoplayEnabled; // boolean

// Mutations
settingsStore.toggleApi('heimuer');
settingsStore.setAutoplayEnabled(false);
```

---

## Settings Store

**File:** `src/lib/stores/settings.svelte.ts`

### State Shape

```typescript
interface Settings {
	selectedApis: string[];             // Enabled API source IDs
	customApis: ApiSite[];              // Custom API sources (name + url)
	yellowFilterEnabled: boolean;        // Filter adult content
	adFilteringEnabled: boolean;         // Filter ad segments in HLS
	autoplayEnabled: boolean;            // Auto-play on load
	autoplayNextEpisode: boolean;       // Auto-play next episode
	gridDensity: GridDensity;            // 'compact' | 'standard' | 'loose'
	commentaryFilterEnabled: boolean;    // Filter commentary/review videos
	preloaderCacheSizeMB: number;        // Prefetch cache max size in MB
	preloaderWorkerCount: number;        // Number of prefetch workers
	tvNavModeEnabled: boolean;           // TV remote-style focus navigation
}
```

### Grid Density Classes

```typescript
export const GRID_DENSITY_CLASSES: Record<GridDensity, string> = {
	compact: 'grid-cols-8 gap-8',
	standard: 'grid-cols-6 gap-8',
	loose: 'grid-cols-5 gap-8'
};
```

### Default Settings

```typescript
const DEFAULT_SETTINGS = {
	selectedApis: ['tyyszy', 'xiaomaomi', 'dyttzy', 'bfzy', 'ruyi'],
	customApis: [],
	yellowFilterEnabled: true,
	adFilteringEnabled: true,
	autoplayEnabled: true,
	autoplayNextEpisode: true,
	gridDensity: 'standard',
	commentaryFilterEnabled: true,
	preloaderCacheSizeMB: 512,
	preloaderWorkerCount: 6,
	tvNavModeEnabled: true
};
```

### Methods

| Method | Description |
|--------|-------------|
| `selectedApis` (getter) | Get enabled API source IDs |
| `setSelectedApis(apis)` | Replace enabled APIs |
| `toggleApi(apiKey)` | Toggle a single API source |
| `addCustomApi(api)` | Add a custom API source |
| `removeCustomApi(index)` | Remove custom API by index |
| `setYellowFilterEnabled(bool)` | Toggle yellow content filter |
| `setAdFilteringEnabled(bool)` | Toggle ad filtering |
| `setAutoplayEnabled(bool)` | Toggle autoplay |
| `setAutoplayNextEpisode(bool)` | Toggle auto next episode |
| `setGridDensity(density)` | Set grid density |
| `setCommentaryFilterEnabled(bool)` | Toggle commentary filter |
| `setPreloaderCacheSizeMB(size)` | Set prefetch cache size |
| `setPreloaderWorkerCount(count)` | Set prefetch worker count |
| `setTvNavModeEnabled(bool)` | Toggle TV navigation mode |
| `exportConfig()` | Export settings as JSON string |
| `importConfig(jsonStr)` | Import settings from JSON string |
| `reset()` | Reset to defaults |

---

## History Store

**File:** `src/lib/stores/history.svelte.ts`

Persistence is handled by the Rust backend via Tauri commands. The store loads history on initialization and syncs changes through `invoke()`.

### Interface

```typescript
interface HistoryItem {
	id: string;            // Video ID
	title: string;        // Video title
	source: string;        // Source identifier
	cover?: string;        // Thumbnail URL
	episode?: string;      // Episode label
	episodeIndex?: number;  // Episode index
}
```

### Methods

| Method | Description |
|--------|-------------|
| `items` (getter) | All history items (newest first) |
| `loaded` (getter) | Whether initial load from Rust completed |
| `add(item)` | Add or update an item (max 100 items) |
| `remove(id, source, episode?)` | Remove a specific item |
| `clear()` | Clear all history |
| `refresh()` | Re-load from Rust backend |

### Dedup Logic

Items are uniquely identified by `id + source + episode`. Adding an existing item updates it in-place instead of duplicating.

---

## Favourites Store

**File:** `src/lib/stores/favourites.svelte.ts`

Persistence is handled by the Rust backend via Tauri commands. The store communicates using snake_case JSON for the Rust side.

### Interface

```typescript
interface FavouriteItem {
	id: string;            // Video ID
	title: string;         // Video title
	source: string;        // Source identifier
	cover?: string;        // Thumbnail URL
	episode?: string;      // Episode label
	episodeIndex?: number;  // Episode index
	addedAt: number;       // Unix timestamp
}
```

### Methods

| Method | Description |
|--------|-------------|
| `items` (getter) | All favourites (newest first) |
| `loaded` (getter) | Whether initial load from Rust completed |
| `add(item)` | Add to favourites (no-op if already exists) |
| `remove(id, source, episode?)` | Remove from favourites |
| `has(id, source, episode?)` | Check if item is favourited (synchronous, local check) |
| `clear()` | Clear all favourites |
| `refresh()` | Re-load from Rust backend |

---

## Search History Store

**File:** `src/lib/stores/search.svelte.ts`

Persists to `localStorage`. Simple string list.

### Methods

| Method | Description |
|--------|-------------|
| `items` (getter) | Search history (newest first) |
| `add(query)` | Add a search query (dedup, max 20) |
| `remove(query)` | Remove a specific query |
| `clear()` | Clear all search history |

---

## Theme Store

**File:** `src/lib/stores/theme.svelte.ts`

### Theme Types

```typescript
type Theme = 'light' | 'dark' | 'system';
```

### Methods

| Method | Description |
|--------|-------------|
| `current` (getter) | Current theme setting |
| `setTheme(t)` | Set theme |
| `init()` | Apply theme to DOM, listen for system changes |

### Behavior

| Setting | Result |
|---------|--------|
| `light` | Always light mode |
| `dark` | Always dark mode |
| `system` | Follow OS preference via `prefers-color-scheme` |

Applied by toggling `dark` class on `<html>` element.

---

## Modal Store

**File:** `src/lib/stores/modal.svelte.ts`

Unlike other stores, this is a simple module-level state (not a factory pattern).

### Interface

```typescript
interface ModalInfo {
	title: string;
	content: string;
	confirmText?: string;
	cancelText?: string;
	onConfirm?: () => void;
	onCancel?: () => void;
}
```

### Usage

```typescript
import { modalStore } from '$lib/stores/modal.svelte';

modalStore.show({ title: 'Warning', content: 'Are you sure?' });
modalStore.hide();

// On Linux, also checks for missing gst-libav at startup:
await modalStore.checkGstLibav();
```

---

## Creating New Stores

Example using Svelte 5 runes:

```typescript
// src/lib/stores/example.svelte.ts
import { browser } from '$app/environment';

function createExampleStore() {
	let items = $state<string[]>(load());

	function load(): string[] {
		if (!browser) return [];
		const stored = localStorage.getItem('example');
		return stored ? JSON.parse(stored) : [];
	}

	function save() {
		if (browser) {
			localStorage.setItem('example', JSON.stringify(items));
		}
	}

	return {
		get items() { return items; },
		add(item: string) {
			items = [...items, item];
			save();
		},
		remove(item: string) {
			items = items.filter(i => i !== item);
			save();
		}
	};
}

export const exampleStore = createExampleStore();
```

### Key Points

1. Use `$state` for reactive state
2. Check `browser` before accessing `localStorage`
3. Call `save()` after mutations to persist
4. Return getters + methods from the factory function
