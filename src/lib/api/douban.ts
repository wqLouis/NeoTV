import {
	DOUBAN_NEW_SEARCH_API_BASE,
	DOUBAN_CHART_TOP_LIST_BASE,
	DOUBAN_TAGS_BASE,
	DOUBAN_CHART_GENRE_IDS
} from './constants';

const COVER_BASE_URL = 'https://movie.douban.com';

export interface DoubanSubject {
	id: string;
	title: string;
	cover: string;
	cover_url?: string;
	rate: string;
	score?: string;
	region?: string[];
	regions?: string[];
	types?: string[];
	director?: string[];
	actors?: string[];
	url?: string;
}

interface DoubanResponse {
	subjects?: DoubanSubject[];
	tags?: string[];
	data?: DoubanSubject[];
	msg?: string;
}

const DOUBAN_API_CACHE_DURATION_MS = 10 * 60 * 1000;
const IMAGE_CACHE_DURATION_MS = 7 * 24 * 60 * 60 * 1000;

const USER_AGENTS = [
	'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36',
	'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15',
	'Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/121.0',
	'Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1',
	'Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Mobile Safari/537.36'
];

function getRandomUserAgent() {
	return USER_AGENTS[Math.floor(Math.random() * USER_AGENTS.length)];
}

const imageMemoryCache = new Map<string, string>();

export function clearImageCache() {
	imageMemoryCache.clear();
	if (typeof localStorage !== 'undefined') {
		const keysToRemove: string[] = [];
		for (let i = 0; i < localStorage.length; i++) {
			const key = localStorage.key(i);
			if (key && key.startsWith('img_')) {
				keysToRemove.push(key);
			}
		}
		keysToRemove.forEach((key) => localStorage.removeItem(key));
	}
}

export function getImageCacheSize(): number {
	return imageMemoryCache.size;
}

export async function getCachedImageUrl(url: string): Promise<string> {
	if (!url) return '';

	const cacheKey = `img_${btoa(url).replace(/[/+=]/g, '_')}`;

	if (imageMemoryCache.has(cacheKey)) {
		return imageMemoryCache.get(cacheKey)!;
	}

	if (typeof localStorage !== 'undefined') {
		try {
			const cached = localStorage.getItem(cacheKey);
			if (cached) {
				const { data, timestamp } = JSON.parse(cached);
				if (Date.now() - timestamp < IMAGE_CACHE_DURATION_MS) {
					imageMemoryCache.set(cacheKey, data);
					return data;
				}
				localStorage.removeItem(cacheKey);
			}
		} catch {
			localStorage.removeItem(cacheKey);
		}
	}

	const isTauri = typeof window !== 'undefined' && '__TAURI__' in window;

	if (isTauri) {
		try {
			const { invoke } = await import('@tauri-apps/api/core');
			console.log('[getCachedImageUrl] Calling Rust cache_fetch_image for:', url.slice(0, 50));
			const dataUrl = await invoke<string>('cache_fetch_image', { url });
			console.log('[getCachedImageUrl] Rust returned:', dataUrl.length, 'chars');
			imageMemoryCache.set(cacheKey, dataUrl);
			return dataUrl;
		} catch (e) {
			console.warn('[getCachedImageUrl] Tauri invoke failed:', e);
			console.log('[getCachedImageUrl] Falling back to proxy');
		}
	}

	try {
		const proxyUrl = `/api/proxy?url=${encodeURIComponent(url)}`;
		const response = await fetch(proxyUrl);
		if (response.ok) {
			const buffer = await response.arrayBuffer();
			const base64 = btoa(String.fromCharCode(...new Uint8Array(buffer)));
			const mimeType = response.headers.get('content-type') || 'image/jpeg';
			const dataUrl = `data:${mimeType};base64,${base64}`;

			imageMemoryCache.set(cacheKey, dataUrl);

			if (typeof localStorage !== 'undefined') {
				try {
					localStorage.setItem(cacheKey, JSON.stringify({ data: dataUrl, timestamp: Date.now() }));
				} catch {
					// localStorage full
				}
			}
			return dataUrl;
		}
	} catch {
		// Failed
	}
	return url;
}

async function fetchDoubanData(url: string): Promise<DoubanResponse> {
	// Log to Rust file at function entry
	console.log('[fetchDoubanData] Function called with URL:', url);
	if (typeof window !== 'undefined' && '__TAURI__' in window) {
		try {
			const { invoke } = await import('@tauri-apps/api/core');
			await invoke('tauri_write_log', {
				level: 'INFO',
				tag: 'JS',
				msg: `fetchDoubanData called: ${url}`
			});
		} catch (e) {
			console.warn('Failed to write log:', e);
		}
	}

	const cacheKey = `douban_api_cache_${url}`;
	console.log('[fetchDoubanData] Cache key:', cacheKey);

	if (typeof sessionStorage !== 'undefined') {
		try {
			const cachedItemRaw = sessionStorage.getItem(cacheKey);
			if (cachedItemRaw) {
				const cachedItem = JSON.parse(cachedItemRaw);
				if (cachedItem && cachedItem.timestamp && cachedItem.data) {
					if (Date.now() - cachedItem.timestamp < DOUBAN_API_CACHE_DURATION_MS) {
						return cachedItem.data;
					}
				}
			}
		} catch {
			sessionStorage.removeItem(cacheKey);
		}
	}

	const maxRetries = 3;
	const initialDelay = 1000;

	console.log('[fetchDoubanData] Starting fetch loop, maxRetries:', maxRetries);

	const headers = {
		'User-Agent': getRandomUserAgent(),
		Referer: 'https://movie.douban.com/',
		Accept: 'application/json, text/plain, */*',
		'Accept-Language': 'zh-CN,zh;q=0.9,en;q=0.8'
	};

	for (let attempt = 0; attempt <= maxRetries; attempt++) {
		console.log('[fetchDoubanData] Attempt:', attempt);
		try {
			const isTauri = typeof window !== 'undefined' && '__TAURI__' in window;
			console.log('[fetchDoubanData] isTauri:', isTauri);

			let response: Response;

			if (isTauri) {
				try {
					const { invoke } = await import('@tauri-apps/api/core');
					await invoke('tauri_write_log', {
						level: 'DEBUG',
						tag: 'JS',
						msg: `Calling make_http_request: ${url}`
					});
					console.log('[fetchDoubanData] Calling Rust make_http_request for:', url);
					const result = await invoke<{
						status: number;
						headers: Record<string, string>;
						body: string;
					}>('make_http_request', {
						options: { url, method: 'GET', headers, timeout_secs: 15 }
					});
					await invoke('tauri_write_log', {
						level: 'DEBUG',
						tag: 'JS',
						msg: `Rust returned status: ${result.status}, body_len: ${result.body.length}`
					});
					console.log(
						'[fetchDoubanData] Rust returned:',
						result.status,
						result.body.length,
						'bytes'
					);
					response = new Response(result.body, { status: result.status });
				} catch (e) {
					await invoke('tauri_write_log', {
						level: 'WARN',
						tag: 'JS',
						msg: `Tauri invoke failed: ${e}`
					});
					console.warn('[fetchDoubanData] Tauri invoke failed:', e);
					console.log('[fetchDoubanData] Falling back to direct fetch');
					response = await fetch(url, { headers });
				}
			} else {
				response = await fetch(`/api/proxy?url=${encodeURIComponent(url)}`, { headers });
			}

			if (response.status >= 200 && response.status < 300) {
				const jsonData = await response.json();

				if (
					url.startsWith(DOUBAN_NEW_SEARCH_API_BASE) &&
					jsonData.data &&
					Array.isArray(jsonData.data)
				) {
					jsonData.subjects = jsonData.data;
				}

				if (typeof sessionStorage !== 'undefined') {
					try {
						sessionStorage.setItem(
							cacheKey,
							JSON.stringify({ timestamp: Date.now(), data: jsonData })
						);
					} catch {
						// Cache full or unavailable
					}
				}

				return jsonData;
			}

			if (attempt < maxRetries) {
				await new Promise((resolve) =>
					setTimeout(resolve, Math.min(initialDelay * Math.pow(2, attempt), 5000))
				);
			}
		} catch (e) {
			console.warn(`[fetchDoubanData] Attempt ${attempt} failed:`, e);
			if (attempt < maxRetries) {
				await new Promise((resolve) =>
					setTimeout(resolve, Math.min(initialDelay * Math.pow(2, attempt), 5000))
				);
			}
		}
	}

	return { subjects: [] };
}

export async function fetchDoubanChart(
	genreName: string,
	params: { start?: number; limit?: number; interval_id?: string } = {}
): Promise<DoubanSubject[]> {
	const genreId = DOUBAN_CHART_GENRE_IDS[genreName];
	if (typeof genreId === 'undefined') {
		return [];
	}

	const queryParams = new URLSearchParams();
	queryParams.append('type', genreId.toString());
	queryParams.append('interval_id', params.interval_id || '100:90');
	queryParams.append('action', '');
	queryParams.append('start', (params.start || 0).toString());
	queryParams.append('limit', (params.limit || 20).toString());

	const url = `${DOUBAN_CHART_TOP_LIST_BASE}?${queryParams.toString()}`;
	const data = await fetchDoubanData(url);

	let result: DoubanSubject[] = [];
	if (Array.isArray(data)) {
		result = data;
	} else {
		result = data.subjects || [];
	}

	for (const item of result) {
		if (item.cover_url && !item.cover) {
			item.cover = item.cover_url;
		}
	}

	return result;
}

export async function fetchDoubanTags(type: 'movie' | 'tv' = 'movie'): Promise<string[]> {
	const url = `${DOUBAN_TAGS_BASE}?type=${type}`;
	const data = await fetchDoubanData(url);
	return data.tags || [];
}

export async function searchDouban(params: {
	sort?: string;
	range?: string;
	tags?: string;
	start?: number;
	genres?: string;
	countries?: string;
}): Promise<DoubanSubject[]> {
	const queryParams = new URLSearchParams();
	if (params.sort) queryParams.append('sort', params.sort);
	if (params.range) queryParams.append('range', params.range);
	if (params.tags) queryParams.append('tags', params.tags);
	if (params.start) queryParams.append('start', params.start.toString());
	if (params.genres) queryParams.append('genres', params.genres);
	if (params.countries) queryParams.append('countries', params.countries);

	const url = `${DOUBAN_NEW_SEARCH_API_BASE}?${queryParams.toString()}`;
	const data = await fetchDoubanData(url);
	return data.subjects || [];
}

export async function fetchDoubanTVByTag(
	tag: string,
	params: { sort?: string; page_limit?: number; page_start?: number } = {}
): Promise<DoubanSubject[]> {
	const queryParams = new URLSearchParams();
	queryParams.append('type', 'tv');
	queryParams.append('tag', tag);
	if (params.sort) queryParams.append('sort', params.sort);
	queryParams.append('page_limit', (params.page_limit || 20).toString());
	queryParams.append('page_start', (params.page_start || 0).toString());

	const url = `https://movie.douban.com/j/search_subjects?${queryParams.toString()}`;
	const data = await fetchDoubanData(url);

	let subjects = [];
	if (data && typeof data === 'object' && 'subjects' in data) {
		subjects = (data as { subjects: DoubanSubject[] }).subjects;
	} else if (Array.isArray(data)) {
		subjects = data;
	}

	return subjects.map((item) => ({
		id: item.id || item.url?.split('/subject/')[1]?.replace('/', '') || '',
		title: item.title || '',
		cover: item.cover || item.cover_url || '',
		cover_url: item.cover_url || item.cover || '',
		rate: item.rate || item.score || '',
		score: item.score || item.rate || '',
		region: item.regions || item.region || [],
		regions: item.regions || item.region || [],
		types: item.types && item.types.length > 0 ? item.types : [tag],
		director: item.directors || [],
		actors: item.casts || [],
		url: item.url || ''
	}));
}
