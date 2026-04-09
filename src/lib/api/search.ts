import { API_SITES, API_CONFIG, type ApiSite } from './constants';

export interface SearchResult {
	vod_id: string;
	vod_name: string;
	vod_pic: string;
	type_name: string;
	vod_year: string;
	vod_remarks: string;
	vod_content?: string;
	source_name: string;
	source_code: string;
}

export interface SearchResponse {
	list: SearchResult[];
	total: number;
	pagecount?: number;
}

export interface VideoDetail {
	list: {
		vod_id: string;
		vod_name: string;
		vod_pic: string;
		type_name: string;
		vod_year: string;
		vod_remarks: string;
		vod_content: string;
		vod_play_url: string;
	}[];
}

function getHeaders() {
	return {
		'User-Agent':
			'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36',
		Accept: 'application/json'
	};
}

async function fetchFromApi(
	url: string,
	timeout = 8000
): Promise<{ status: number; body: string }> {
	try {
		const controller = new AbortController();
		const timeoutId = setTimeout(() => controller.abort(), timeout);

		const response = await fetch(`/api/proxy?url=${encodeURIComponent(url)}`, {
			signal: controller.signal,
			headers: getHeaders()
		});

		clearTimeout(timeoutId);
		const body = await response.text();
		return { status: response.status, body };
	} catch (error) {
		if (error instanceof Error && error.name === 'AbortError') {
			return { status: 408, body: 'Request timeout' };
		}
		return { status: 500, body: String(error) };
	}
}

async function searchSingleSource(
	query: string,
	apiKey: string,
	customApi?: ApiSite
): Promise<SearchResult[]> {
	let apiBaseUrl: string;
	let apiName: string;

	if (apiKey.startsWith('custom_') && customApi) {
		apiBaseUrl = customApi.api;
		apiName = customApi.name;
	} else {
		const site = API_SITES[apiKey];
		if (!site) return [];
		apiBaseUrl = site.api;
		apiName = site.name;
	}

	const searchUrl = apiBaseUrl + API_CONFIG.search.path + encodeURIComponent(query);
	const result = await fetchFromApi(searchUrl);

	if (result.status < 200 || result.status >= 300) {
		return [];
	}

	try {
		const data = JSON.parse(result.body);
		if (!data || !Array.isArray(data.list)) return [];

		const pageCount = data.pagecount || 1;
		const pagesToFetch = Math.min(pageCount - 1, API_CONFIG.search.maxPages - 1);
		let results = data.list.map((item: Record<string, unknown>) => ({
			...item,
			source_name: apiName,
			source_code: apiKey
		}));

		if (pagesToFetch > 0) {
			const pagePromises: Promise<SearchResult[]>[] = [];
			for (let page = 2; page <= pagesToFetch + 1; page++) {
				const pageUrl =
					apiBaseUrl +
					API_CONFIG.search.pagePath
						.replace('{query}', encodeURIComponent(query))
						.replace('{page}', page.toString());
				pagePromises.push(
					fetchFromApi(pageUrl)
						.then((pageResult) => {
							if (pageResult.status < 200 || pageResult.status >= 300) return [];
							try {
								const pageData = JSON.parse(pageResult.body);
								if (!pageData || !Array.isArray(pageData.list)) return [];
								return pageData.list.map((item: Record<string, unknown>) => ({
									...item,
									source_name: apiName,
									source_code: apiKey
								}));
							} catch {
								return [];
							}
						})
						.catch(() => [])
				);
			}
			const pageResults = await Promise.all(pagePromises);
			for (const pageResult of pageResults) {
				results = results.concat(pageResult);
			}
		}

		return results;
	} catch {
		return [];
	}
}

export async function search(
	query: string,
	selectedApis: string[],
	customApis: ApiSite[],
	yellowFilterEnabled: boolean
): Promise<SearchResult[]> {
	const searchPromises: Promise<SearchResult[]>[] = [];

	for (const apiKey of selectedApis) {
		let customApi: ApiSite | undefined;
		if (apiKey.startsWith('custom_')) {
			const idx = parseInt(apiKey.replace('custom_', ''), 10);
			customApi = customApis[idx];
		}
		searchPromises.push(searchSingleSource(query, apiKey, customApi));
	}

	const resultsArray = await Promise.all(searchPromises);
	let allResults: SearchResult[] = [];

	for (const results of resultsArray) {
		if (Array.isArray(results) && results.length > 0) {
			allResults = allResults.concat(results);
		}
	}

	if (yellowFilterEnabled) {
		const { YELLOW_FILTER_BANNED } = await import('./constants');
		allResults = allResults.filter((item) => {
			const typeName = item.type_name || '';
			return !YELLOW_FILTER_BANNED.some((keyword) => typeName.includes(keyword));
		});
	}

	return allResults;
}

export async function getVideoDetail(
	id: string,
	sourceCode: string,
	customApiUrl?: string
): Promise<VideoDetail | null> {
	let apiBaseUrl: string;

	if (sourceCode === 'custom' && customApiUrl) {
		apiBaseUrl = customApiUrl;
	} else {
		const site = API_SITES[sourceCode];
		if (!site) return null;
		apiBaseUrl = site.detail || site.api;
	}

	const detailUrl = apiBaseUrl + API_CONFIG.detail.path + encodeURIComponent(id);
	const result = await fetchFromApi(detailUrl);

	if (result.status < 200 || result.status >= 300) {
		return null;
	}

	try {
		return JSON.parse(result.body);
	} catch {
		return null;
	}
}

export function parsePlayUrl(vodPlayUrl: string): { episode: string; url: string }[] {
	const episodes: { episode: string; url: string }[] = [];
	if (!vodPlayUrl) return episodes;

	const sources = vodPlayUrl.split('$$$');
	for (const source of sources) {
		const parts = source.split('$');
		if (parts.length >= 2) {
			episodes.push({
				episode: parts[0],
				url: parts.slice(1).join('$')
			});
		}
	}
	return episodes;
}
