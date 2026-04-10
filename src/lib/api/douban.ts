import {
	DOUBAN_NEW_SEARCH_API_BASE,
	DOUBAN_CHART_TOP_LIST_BASE,
	DOUBAN_TAGS_BASE,
	DOUBAN_CHART_GENRE_IDS
} from './constants';

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
}

const API_CACHE_MS = 10 * 60 * 1000;
const IMAGE_CACHE_MS = 7 * 24 * 60 * 60 * 1000;

const imageCache = new Map<string, string>();

export async function getCachedImageUrl(url: string): Promise<string> {
	if (!url) return '';

	if (imageCache.has(url)) {
		return imageCache.get(url)!;
	}

	try {
		const { invoke } = await import('@tauri-apps/api/core');
		const dataUrl: string = await invoke('cache_fetch_image', { url });
		imageCache.set(url, dataUrl);
		return dataUrl;
	} catch (e) {
		console.error('[Image] Failed to fetch via cache:', e);
		return url;
	}
}

export function clearImageCache() {
	imageCache.clear();
	if (typeof localStorage !== 'undefined') {
		const keys = [...localStorage.keys()].filter((k) => k.startsWith('img_'));
		keys.forEach((k) => localStorage.removeItem(k));
	}
}

async function fetchDoubanData(url: string): Promise<DoubanResponse> {
	const cacheKey = `douban_api_cache_${url}`;

	if (typeof sessionStorage !== 'undefined') {
		try {
			const cached = JSON.parse(sessionStorage.getItem(cacheKey) || '{}');
			if (cached?.timestamp && Date.now() - cached.timestamp < API_CACHE_MS) {
				return cached.data;
			}
		} catch {
			sessionStorage.removeItem(cacheKey);
		}
	}

	const headers = {
		'User-Agent':
			'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/122.0.0.0 Safari/537.36',
		Referer: 'https://movie.douban.com/',
		Accept: 'application/json',
		'Accept-Language': 'zh-CN,zh;q=0.9'
	};

	for (let attempt = 0; attempt < 3; attempt++) {
		try {
			const { invoke } = await import('@tauri-apps/api/core');
			const result = await invoke<{ status: number; body: string }>('make_http_request', {
				options: { url, method: 'GET', headers, timeout_secs: 15 }
			});

			if (result.status >= 200 && result.status < 300) {
				const jsonData = JSON.parse(result.body);
				if (url.startsWith(DOUBAN_NEW_SEARCH_API_BASE) && jsonData.data?.length) {
					jsonData.subjects = jsonData.data;
				}
				if (typeof sessionStorage !== 'undefined') {
					try {
						sessionStorage.setItem(
							cacheKey,
							JSON.stringify({ timestamp: Date.now(), data: jsonData })
						);
					} catch {}
				}
				return jsonData;
			}
		} catch {}
		if (attempt < 2) await new Promise((r) => setTimeout(r, Math.pow(2, attempt) * 1000));
	}

	return { subjects: [] };
}

export async function fetchDoubanChart(
	genreName: string,
	{ start = 0, limit = 20 }: { start?: number; limit?: number } = {}
): Promise<DoubanSubject[]> {
	const genreId = DOUBAN_CHART_GENRE_IDS[genreName];
	if (!genreId) return [];

	const url = `${DOUBAN_CHART_TOP_LIST_BASE}?type=${genreId}&interval_id=100:90&action=&start=${start}&limit=${limit}`;
	const data = await fetchDoubanData(url);

	const result = Array.isArray(data) ? data : data.subjects || [];
	for (const item of result) {
		if (item.cover_url && !item.cover) item.cover = item.cover_url;
	}
	return result;
}

export async function fetchDoubanTags(type: 'movie' | 'tv' = 'movie'): Promise<string[]> {
	const data = await fetchDoubanData(`${DOUBAN_TAGS_BASE}?type=${type}`);
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
	const query = new URLSearchParams();
	if (params.sort) query.set('sort', params.sort);
	if (params.range) query.set('range', params.range);
	if (params.tags) query.set('tags', params.tags);
	if (params.start) query.set('start', String(params.start));
	if (params.genres) query.set('genres', params.genres);
	if (params.countries) query.set('countries', params.countries);

	const data = await fetchDoubanData(`${DOUBAN_NEW_SEARCH_API_BASE}?${query}`);
	return data.subjects || [];
}

export async function fetchDoubanTVByTag(
	tag: string,
	{
		sort,
		page_limit = 20,
		page_start = 0
	}: { sort?: string; page_limit?: number; page_start?: number } = {}
): Promise<DoubanSubject[]> {
	const query = new URLSearchParams({
		type: 'tv',
		tag,
		page_limit: String(page_limit),
		page_start: String(page_start)
	});
	if (sort) query.set('sort', sort);

	const data = await fetchDoubanData(`https://movie.douban.com/j/search_subjects?${query}`);
	const subjects: DoubanSubject[] = data?.subjects || [];

	return subjects.map((item) => {
		const raw = item as unknown as Record<string, unknown>;
		return {
			id: item.id || item.url?.split('/subject/')[1]?.replace('/', '') || '',
			title: item.title || '',
			cover: item.cover || item.cover_url || '',
			cover_url: item.cover_url || item.cover || '',
			rate: item.rate || item.score || '',
			score: item.score || item.rate || '',
			region: item.regions || item.region || [],
			regions: item.regions || item.region || [],
			types: item.types?.length ? item.types : [tag],
			director: (raw.directors as string[]) || [],
			actors: (raw.casts as string[]) || [],
			url: item.url || ''
		};
	});
}
