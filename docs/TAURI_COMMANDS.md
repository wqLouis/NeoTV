# Tauri Commands API Reference

All Tauri commands are exposed to the frontend via `invoke()` from `@tauri-apps/api/core`.

## Command Index

| Command                   | File        | Purpose                   |
| ------------------------- | ----------- | ------------------------- |
| `make_http_request`       | commands.rs | Generic HTTP request      |
| `fetch_url`               | commands.rs | Fetch URL with caching    |
| `search_videos`           | commands.rs | Search videos by source   |
| `get_video_detail`        | commands.rs | Get video detail          |
| `fetch_media_url`         | commands.rs | Fetch and process M3U8    |
| `fetch_media_segment`     | commands.rs | Fetch media segment bytes |
| `fetch_hls_m3u8`          | commands.rs | Fetch M3U8 content        |
| `fetch_hls_segment`       | commands.rs | Fetch HLS segment         |
| `cache_clear`             | commands.rs | Clear cache               |
| `cache_stats`             | commands.rs | Get cache statistics      |
| `test_source_speed`       | commands.rs | Test source speed         |
| `check_transcoder`        | commands.rs | Check FFmpeg availability |
| `start_transcoded_stream` | commands.rs | Start FFmpeg transcoding  |
| `stop_transcoded_stream`  | commands.rs | Stop transcoding stream   |

## Video Search & Detail

### search_videos

Search for videos across configured API sources.

```typescript
invoke<string>('search_videos', {
    query: string,
    source_id: string,
    custom_api_url?: string
}): Promise<string>
```

**Parameters:**

- `query`: Search keyword
- `source_id`: Source identifier (e.g., `"heimuer"`, `"dyttzy"`) or `"custom"`
- `custom_api_url`: Required if `source_id` is `"custom"`

**Returns:** JSON string with search results

**Example:**

```typescript
const result = await invoke<string>('search_videos', {
	query: '流浪地球',
	source_id: 'heimuer'
});
const data = JSON.parse(result);
```

### get_video_detail

Get detailed information about a video including episodes.

```typescript
invoke<string>('get_video_detail', {
    video_id: string,
    source_id: string
}): Promise<string>
```

**Parameters:**

- `video_id`: Video identifier from search results
- `source_id`: Source identifier

**Returns:** JSON string with video details and episode list

## Media Fetching

### fetch_media_url

Fetch and process an M3U8 URL, returning processed media info.

```typescript
invoke<MediaInfo>('fetch_media_url', {
    url: string,
    ad_filtering?: boolean
}): Promise<MediaInfo>
```

**Parameters:**

- `url`: M3U8 playlist URL
- `ad_filtering`: Enable ad segment filtering (default: true)

**Returns:**

```typescript
interface MediaInfo {
	url: string;
	content_type: string;
	is_m3u8: boolean;
	processed_content: string | null;
	duration: number | null;
}
```

### fetch_hls_m3u8

Fetch M3U8 content directly (used by RustLoader).

```typescript
invoke<string>('fetch_hls_m3u8', {
    url: string,
    ad_filtering?: boolean
}): Promise<string>
```

**Parameters:**

- `url`: M3U8 URL
- `ad_filtering`: Enable ad filtering (default: true)

**Returns:** M3U8 playlist content as string

### fetch_hls_segment

Fetch a media segment (used by RustLoader).

```typescript
invoke<number[]>('fetch_hls_segment', {
    url: string
}): Promise<number[]>
```

**Parameters:**

- `url`: Segment URL (supports `app-media://` prefix)

**Returns:** Raw segment bytes as `number[]` (Uint8Array)

### fetch_media_segment

Fetch raw media bytes (image, video, etc.).

```typescript
invoke<number[]>('fetch_media_segment', {
    url: string
}): Promise<number[]>
```

**Parameters:**

- `url`: Media URL (supports `app-media://` prefix)

**Returns:** Raw bytes as `number[]`

## HTTP Requests

### make_http_request

Generic HTTP request with custom headers.

```typescript
invoke<HttpResponse>('make_http_request', {
    options: HttpRequestOptions
}): Promise<HttpResponse>
```

```typescript
interface HttpRequestOptions {
	url: string;
	method?: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';
	headers?: Record<string, string>;
	body?: unknown;
	timeout_secs?: number;
}

interface HttpResponse {
	status: number;
	headers: Record<string, string>;
	body: string;
}
```

### fetch_url

Fetch URL with automatic caching (images cached for 1 hour).

```typescript
invoke<string>('fetch_url', {
    url: string,
    referer?: string
}): Promise<string>
```

**Returns:**

- Images: `data:{content_type};base64,{data}`
- Other content: Raw string

## Caching

### cache_clear

Clear all memory and disk cache.

```typescript
invoke<void>('cache_clear'): Promise<void>
```

### cache_stats

Get cache statistics.

```typescript
invoke<CacheStats>('cache_stats'): Promise<CacheStats>
```

```typescript
interface CacheStats {
	mem_count: number;
	mem_size: number;
	disk_count: number;
	disk_size: number;
}
```

## Source Speed Testing

### test_source_speed

Test API source responsiveness.

```typescript
invoke<SpeedTestResult>('test_source_speed', {
    source_id: string,
    custom_url?: string
}): Promise<SpeedTestResult>
```

```typescript
interface SpeedTestResult {
	source_id: string;
	source_name: string;
	latency_ms: number;
	download_speed_kbps: number;
	status: 'success' | 'error';
	error?: string;
}
```

## FFmpeg Transcoder

### check_transcoder

Check FFmpeg and hardware acceleration availability.

```typescript
invoke<TranscoderInfo>('check_transcoder'): Promise<TranscoderInfo>
```

```typescript
interface TranscoderInfo {
	url: string;
	port: number;
	duration: number | null;
	vaapi_available: boolean;
	ffmpeg_available: boolean;
}
```

### start_transcoded_stream

Start FFmpeg transcoding pipeline for HLS streams.

```typescript
invoke<TranscoderInfo>('start_transcoded_stream', {
    id: string,
    m3u8_url: string,
    referer?: string
}): Promise<TranscoderInfo>
```

**Parameters:**

- `id`: Unique stream identifier
- `m3u8_url`: Source M3U8 URL
- `referer`: Optional referer header

**Returns:**

```typescript
interface TranscoderInfo {
	url: string; // app-media://stream/{id}
	port: number; // Internal port (0 if not HTTP)
	duration: number | null;
	vaapi_available: boolean;
	ffmpeg_available: boolean;
}
```

### stop_transcoded_stream

Stop a running transcoded stream.

```typescript
invoke<void>('stop_transcoded_stream', {
    id: string
}): Promise<void>
```

## Video Sources Configuration

Sources are configured in `src-tauri/src/config.rs`:

| Source ID   | Name         | Type |
| ----------- | ------------ | ---- |
| `dyttzy`    | 电影天堂资源 | JSON |
| `ruyi`      | 如意资源     | JSON |
| `bfzy`      | 暴风资源     | JSON |
| `tyyszy`    | 天涯资源     | JSON |
| `xiaomaomi` | 小猫咪资源   | JSON |
| `ffzy`      | 非凡影视     | HTML |
| `heimuer`   | 黑木耳       | HTML |
| `zy360`     | 360资源      | JSON |
| `wolong`    | 卧龙资源     | JSON |
| `hwba`      | 华为吧资源   | JSON |
| `jisu`      | 极速资源     | JSON |
| `dbzy`      | 豆瓣资源     | JSON |
| `mozhua`    | 魔爪资源     | JSON |
| `mdzy`      | 魔都资源     | JSON |
| `zuid`      | 最大资源     | JSON |
| `yinghua`   | 樱花资源     | JSON |
| `baidu`     | 百度云资源   | JSON |
| `wujin`     | 无尽资源     | JSON |
| `wwzy`      | 旺旺短剧     | JSON |
| `ikun`      | iKun资源     | JSON |

## Usage Examples

### Search and Play Video

```typescript
import { invoke } from '@tauri-apps/api/core';

// 1. Search for videos
const searchResult = await invoke<string>('search_videos', {
	query: '流浪地球',
	source_id: 'heimuer'
});
const searchData = JSON.parse(searchResult);

// 2. Get video detail
const detailResult = await invoke<string>('get_video_detail', {
	video_id: searchData.list[0].vod_id,
	source_id: 'heimuer'
});
const videoData = JSON.parse(detailResult);

// 3. Fetch and process M3U8
const mediaInfo = await invoke<MediaInfo>('fetch_media_url', {
	url: videoData.list[0].vod_play_url,
	ad_filtering: true
});

// 4. Play with VideoPlayer component
playerSrc = mediaInfo.url;
playerType = 'hls';
```

### Custom HTTP Request

```typescript
const response = await invoke<HttpResponse>('make_http_request', {
	options: {
		url: 'https://api.example.com/data',
		method: 'POST',
		headers: {
			Authorization: 'Bearer token',
			'Content-Type': 'application/json'
		},
		body: { key: 'value' },
		timeout_secs: 30
	}
});
```

### Check FFmpeg Availability

```typescript
const info = await invoke<TranscoderInfo>('check_transcoder');
if (info.ffmpeg_available) {
	console.log(`FFmpeg available: VA-API=${info.vaapi_available}`);
} else {
	console.log('FFmpeg not available on this system');
}
```
