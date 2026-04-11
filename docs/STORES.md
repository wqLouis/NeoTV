# Svelte Stores

NeoTV uses Svelte 5's `$state` runes for reactive state management. All stores persist data to localStorage.

## Store Index

| Store                | File                 | Purpose                       |
| -------------------- | -------------------- | ----------------------------- |
| `settingsStore`      | settings.svelte.ts   | User preferences and settings |
| `historyStore`       | history.svelte.ts    | Watch history                 |
| `favouritesStore`    | favourites.svelte.ts | Saved favorites               |
| `searchHistoryStore` | search.svelte.ts     | Search query history          |
| `themeStore`         | theme.svelte.ts      | Theme (light/dark/system)     |

## Usage Pattern

```typescript
import { settingsStore } from '$lib/stores/settings.svelte';
import { historyStore } from '$lib/stores/history.svelte';

// Accessing state (reactive)
settingsStore.selectedApis;  // string[]
settingsStore.autoplayEnabled;  // boolean

// Calling methods
settingsStore.toggleApi('heimuer');
historyStore.add({ id: '123', title: 'Movie', ... });
```

## Settings Store

**File:** `src/lib/stores/settings.svelte.ts`

### Interface

```typescript
interface Settings {
	selectedApis: string[]; // Enabled API source IDs
	customApis: ApiSite[]; // Custom API sources
	doubanEnabled: boolean; // Show Douban recommendations
	doubanApiMode: 'all' | 'hot' | 'new'; // Douban fetch mode
	yellowFilterEnabled: boolean; // Filter adult content
	adFilteringEnabled: boolean; // Filter ad segments in HLS
	autoplayEnabled: boolean; // Auto-play video
	autoplayNextEpisode: boolean; // Auto-play next episode
	episodesReversed: boolean; // Reverse episode order
	gridDensity: GridDensity; // 'compact' | 'standard' | 'loose'
	commentaryFilterEnabled: boolean; // Filter commentary videos
	autoIntegrateSources: boolean; // Auto test and select sources
}
```

### Grid Density Classes

```typescript
const GRID_DENSITY_CLASSES: Record<GridDensity, string> = {
	compact: 'grid-cols-8 gap-8',
	standard: 'grid-cols-6 gap-8',
	loose: 'grid-cols-5 gap-8'
};
```

### Methods

| Method                             | Description                  |
| ---------------------------------- | ---------------------------- |
| `get selectedApis()`               | Get enabled API source IDs   |
| `setSelectedApis(apis)`            | Set enabled API sources      |
| `toggleApi(apiKey)`                | Toggle single API source     |
| `addCustomApi(api)`                | Add custom API source        |
| `removeCustomApi(index)`           | Remove custom API by index   |
| `setDoubanEnabled(bool)`           | Enable/disable Douban        |
| `setDoubanApiMode(mode)`           | Set Douban mode              |
| `setYellowFilterEnabled(bool)`     | Toggle yellow filter         |
| `setAdFilteringEnabled(bool)`      | Toggle ad filtering          |
| `setAutoplayEnabled(bool)`         | Toggle autoplay              |
| `setAutoplayNextEpisode(bool)`     | Toggle auto next episode     |
| `setEpisodesReversed(bool)`        | Toggle episode order         |
| `setGridDensity(density)`          | Set grid density             |
| `setCommentaryFilterEnabled(bool)` | Toggle commentary filter     |
| `setAutoIntegrateSources(bool)`    | Toggle auto source selection |
| `exportConfig()`                   | Export settings as JSON      |
| `importConfig(json)`               | Import settings from JSON    |
| `reset()`                          | Reset to default settings    |

### Default Settings

```typescript
const DEFAULT_SETTINGS = {
	selectedApis: ['tyyszy', 'xiaomaomi', 'dyttzy', 'bfzy', 'ruyi'],
	customApis: [],
	doubanEnabled: true,
	doubanApiMode: 'all',
	yellowFilterEnabled: true,
	adFilteringEnabled: true,
	autoplayEnabled: true,
	autoplayNextEpisode: true,
	episodesReversed: false,
	gridDensity: 'standard',
	commentaryFilterEnabled: true,
	autoIntegrateSources: true
};
```

### Storage Key

`localStorage.getItem('appSettings')`

---

## History Store

**File:** `src/lib/stores/history.svelte.ts`

### Interface

```typescript
interface HistoryItem {
	id: string; // Video ID
	title: string; // Video title
	source: string; // Source identifier
	cover?: string; // Thumbnail URL
	episode?: string; // Episode label
	episodeIndex?: number; // Episode index
	position: number; // Playback position (seconds)
	duration: number; // Total duration (seconds)
	timestamp: number; // Last updated timestamp
}
```

### Methods

| Method                                                    | Description                |
| --------------------------------------------------------- | -------------------------- |
| `get items()`                                             | Get all history items      |
| `add(item)`                                               | Add or update history item |
| `updatePosition(id, source, episode, position, duration)` | Update playback position   |
| `remove(id, source, episode?)`                            | Remove specific item       |
| `clear()`                                                 | Clear all history          |

### Storage Key

`localStorage.getItem('viewingHistory')`

### Notes

- Maximum 100 items (older items are removed)
- Items are uniquely identified by `id + source + episode`

---

## Favourites Store

**File:** `src/lib/stores/favourites.svelte.ts`

### Interface

```typescript
interface FavouriteItem {
	id: string; // Video ID
	title: string; // Video title
	source: string; // Source identifier
	cover?: string; // Thumbnail URL
	episode?: string; // Episode label
	episodeIndex?: number; // Episode index
	addedAt: number; // Timestamp when added
}
```

### Methods

| Method                         | Description                |
| ------------------------------ | -------------------------- |
| `get items()`                  | Get all favourites         |
| `add(item)`                    | Add to favourites          |
| `remove(id, source, episode?)` | Remove from favourites     |
| `has(id, source, episode?)`    | Check if item is favorited |
| `clear()`                      | Clear all favourites       |

### Storage Key

`localStorage.getItem('favourites')`

---

## Search History Store

**File:** `src/lib/stores/search.svelte.ts`

### Methods

| Method          | Description                       |
| --------------- | --------------------------------- |
| `get items()`   | Get search history (newest first) |
| `add(query)`    | Add search query                  |
| `remove(query)` | Remove specific query             |
| `clear()`       | Clear all search history          |

### Storage Key

`localStorage.getItem('videoSearchHistory')`

### Notes

- Maximum 20 items
- Duplicate queries are moved to top

---

## Theme Store

**File:** `src/lib/stores/theme.svelte.ts`

### Theme Types

```typescript
type Theme = 'light' | 'dark' | 'system';
```

### Methods

| Method          | Description                               |
| --------------- | ----------------------------------------- |
| `get current()` | Get current theme setting                 |
| `setTheme(t)`   | Set theme (light/dark/system)             |
| `init()`        | Apply theme and listen for system changes |

### Storage Key

`localStorage.getItem('theme')`

### Behavior

- `light`: Always use light mode
- `dark`: Always use dark mode
- `system`: Follow system preference (`prefers-color-scheme`)

### Implementation

Theme is applied by toggling `dark` class on `<html>` element.

---

## Creating New Stores

Example of creating a new store using Svelte 5 runes:

```typescript
// src/lib/stores/example.svelte.ts
import { browser } from '$app/environment';

interface ExampleItem {
	id: string;
	name: string;
}

function createExampleStore() {
	let items = $state<ExampleItem[]>(loadItems());

	function loadItems(): ExampleItem[] {
		if (!browser) return [];
		const stored = localStorage.getItem('exampleItems');
		return stored ? JSON.parse(stored) : [];
	}

	function save() {
		if (browser) {
			localStorage.setItem('exampleItems', JSON.stringify(items));
		}
	}

	return {
		get items() {
			return items;
		},
		add(item: ExampleItem) {
			items = [item, ...items];
			save();
		},
		remove(id: string) {
			items = items.filter((i) => i.id !== id);
			save();
		}
	};
}

export const exampleStore = createExampleStore();
```

### Key Points

1. Use `$state` rune for reactive state
2. Check `browser` before accessing localStorage
3. Use `save()` function to persist changes
4. Return getters and methods from the factory function
