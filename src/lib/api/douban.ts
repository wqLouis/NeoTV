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
const IMAGE_CACHE_DURATION_MS = 24 * 60 * 60 * 1000;

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

async function fetchImageAsBase64(url: string): Promise<string | null> {
	const cacheKey = `img_cache_${btoa(url).replace(/[/+=]/g, '_')}`;

	if (typeof sessionStorage !== 'undefined') {
		try {
			const cached = sessionStorage.getItem(cacheKey);
			if (cached) {
				const { data, timestamp } = JSON.parse(cached);
				if (Date.now() - timestamp < IMAGE_CACHE_DURATION_MS) {
					return data;
				}
			}
		} catch {
			sessionStorage.removeItem(cacheKey);
		}
	}

	try {
		const response = await fetch(`/api/proxy?url=${encodeURIComponent(url)}`);
		if (response.ok) {
			const buffer = await response.arrayBuffer();
			const base64 = btoa(String.fromCharCode(...new Uint8Array(buffer)));
			const mimeType = response.headers.get('content-type') || 'image/jpeg';
			const dataUrl = `data:${mimeType};base64,${base64}`;

			if (typeof sessionStorage !== 'undefined') {
				try {
					sessionStorage.setItem(
						cacheKey,
						JSON.stringify({ data: dataUrl, timestamp: Date.now() })
					);
				} catch {
					// Cache full
				}
			}
			return dataUrl;
		}
	} catch {
		// Failed to fetch
	}
	return null;
}

async function fetchDoubanData(url: string): Promise<DoubanResponse> {
	const cacheKey = `douban_api_cache_${url}`;

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

	const maxRetries = 5;
	const initialDelay = 1500;

	for (let attempt = 0; attempt <= maxRetries; attempt++) {
		try {
			const response = await fetch(`/api/proxy?url=${encodeURIComponent(url)}`, {
				headers: {
					'User-Agent': getRandomUserAgent(),
					Referer: 'https://movie.douban.com/',
					Accept: 'application/json, text/plain, */*',
					'Accept-Language': 'zh-CN,zh;q=0.9,en;q=0.8',
					'X-Requested-With': 'XMLHttpRequest'
				}
			});

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
					setTimeout(resolve, Math.min(initialDelay * Math.pow(2, attempt), 15000))
				);
			}
		} catch {
			if (attempt < maxRetries) {
				await new Promise((resolve) =>
					setTimeout(resolve, Math.min(initialDelay * Math.pow(2, attempt), 15000))
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
