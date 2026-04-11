# Cache System

## Overview

NeoTV implements a two-tier caching system for optimal performance:

1. **Memory Cache** - Fast LRU cache for in-memory storage
2. **Disk Cache** - Persistent storage for larger files

## Architecture

```
Frontend Request
       │
       ▼
┌──────────────────┐
│  Memory Cache    │ ◄── LRU, max 1000 entries
│  (Rust static)   │
└────────┬─────────┘
         │ miss
         ▼
┌──────────────────┐
│   Disk Cache     │ ◄── Local file storage
│  (JSON files)   │
└────────┬─────────┘
         │ miss
         ▼
    HTTP Request
         │
         ▼
    Store Result
```

## Rust Backend (`src-tauri/src/cache.rs`)

### CacheEntry

```rust
struct CacheEntry {
    data: Vec<u8>,         // Raw bytes
    content_type: String,  // MIME type
    cached_at: u64,        // Timestamp
}
```

### Memory Cache

- **Max entries**: 1000 (oldest evicted when exceeded)
- **TTL**: Configurable per request (default 1 hour)
- **Thread-safe**: Uses `Mutex` for concurrent access

```rust
static MEM_CACHE: OnceLock<Mutex<MemCache>> = OnceLock::new();

struct MemCache {
    entries: HashMap<String, CacheEntry>,
}
```

### Disk Cache

- **Location**: `{data_local_dir}/libretv_cache/`
- **Format**: JSON files named by URL hash
- **TTL**: Same as memory cache

### Key Functions

| Function                              | Description                     |
| ------------------------------------- | ------------------------------- |
| `get_cached(url, ttl_secs)`           | Get cached entry if not expired |
| `set_cached(url, data, content_type)` | Store data in memory + disk     |
| `clear_cache()`                       | Clear both caches               |
| `get_cache_stats()`                   | Get cache statistics            |

### Cache Commands

```typescript
// Clear all cache
invoke('cache_clear'): Promise<void>

// Get cache stats
invoke('cache_stats'): Promise<CacheStats>

interface CacheStats {
    mem_count: number;
    mem_size: number;    // bytes
    disk_count: number;
    disk_size: number;   // bytes
}
```

## Frontend Cache (`src/lib/cache.ts`)

### Image Caching

The frontend maintains an additional in-memory cache for images:

```typescript
const imageCache = new Map<string, string>();

async function fetchImage(url: string, referer?: string): Promise<string> {
	if (imageCache.has(url)) {
		return imageCache.get(url)!;
	}
	const dataUrl: string = await invoke('fetch_url', { url, referer });
	imageCache.set(url, dataUrl);
	return dataUrl;
}
```

### Cache Functions

| Function                    | Description                   |
| --------------------------- | ----------------------------- |
| `fetchImage(url, referer?)` | Fetch image with caching      |
| `clearImageCache()`         | Clear frontend image cache    |
| `clearJsImageCache()`       | Clear JS Map only             |
| `clearCache()`              | Clear both JS and Rust cache  |
| `getCacheStats()`           | Get combined cache statistics |

## Usage Examples

### Using cached fetch

```typescript
import { invoke } from '@tauri-apps/api/core';

// Images are automatically cached for 1 hour
const imageData: string = await invoke('fetch_url', {
	url: 'https://example.com/image.jpg',
	referer: 'https://example.com/'
});
```

### Manual cache clear

```typescript
import { clearCache, getCacheStats } from '$lib/cache';

// Clear all caches
await clearCache();

// Check stats
const stats = await getCacheStats();
console.log(`Memory: ${stats.mem_count} items, ${stats.mem_size} bytes`);
console.log(`Disk: ${stats.disk_count} items, ${stats.disk_size} bytes`);
```

## TTL (Time To Live)

| Cache Type | Default TTL | Configurable |
| ---------- | ----------- | ------------ |
| Memory     | 1 hour      | Per-request  |
| Disk       | 1 hour      | Per-request  |

The TTL is passed to `get_cached()`:

```rust
// Check if entry exists and not expired
if let Some(entry) = cache::get_cached(&url, 3600) {  // 3600 seconds
    // Cache hit
}
```

## Cache Key

URLs are hashed using a simple hash function:

```rust
fn hash_key(url: &str) -> String {
    let mut s = DefaultHasher::new();
    url.hash(&mut s);
    format!("{:x}", s.finish())
}
```

## Storage Location

| Platform | Cache Directory                                |
| -------- | ---------------------------------------------- |
| Linux    | `~/.local/share/libretv_cache/`                |
| macOS    | `~/Library/Application Support/libretv_cache/` |
| Windows  | `%LOCALAPPDATA%\libretv_cache\`                |
| Android  | `/data/data/com.neotv.app/cache/`              |

## Performance Notes

- **Memory cache**: Extremely fast, no I/O
- **Disk cache**: JSON serialization overhead
- **Image cache**: Stores as base64 data URLs
- **Cache eviction**: LRU for memory, manual clear for disk

## Troubleshooting

### Cache not working

1. Check disk permissions
2. Verify cache directory exists
3. Check TTL is not 0

### Large cache size

```bash
# Clear cache manually
rm -rf ~/.local/share/libretv_cache/

# Or from app: Settings > Clear Cache
```
