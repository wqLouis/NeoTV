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

async function fetchFromApi(
	url: string,
	timeout = 8000
): Promise<{ status: number; body: string }> {
	const headers = {
		'User-Agent': 'Mozilla/5.0 Chrome/122.0.0.0 Safari/537.36',
		Accept: 'application/json'
	};

	try {
		const { invoke } = await import('@tauri-apps/api/core');
		const result = await invoke<{ status: number; body: string }>('make_http_request', {
			options: { url, method: 'GET', headers, timeout_secs: timeout / 1000 }
		});
		return result;
	} catch {
		return { status: 500, body: 'Tauri invoke failed' };
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
	if (result.status < 200 || result.status >= 300) return [];

	try {
		const data = JSON.parse(result.body);
		if (!data?.list?.length) return [];

		const pageCount = data.pagecount || 1;
		const maxPages = Math.min(pageCount - 1, API_CONFIG.search.maxPages - 1);

		let results: SearchResult[] = data.list.map((item: Record<string, unknown>) => ({
			...item,
			source_name: apiName,
			source_code: apiKey
		}));

		for (let page = 2; page <= maxPages + 1; page++) {
			const pageUrl =
				apiBaseUrl +
				API_CONFIG.search.pagePath
					.replace('{query}', encodeURIComponent(query))
					.replace('{page}', String(page));
			const pageResult = await fetchFromApi(pageUrl);
			if (pageResult.status >= 200 && pageResult.status < 300) {
				try {
					const pageData = JSON.parse(pageResult.body);
					if (pageData?.list?.length) {
						results = results.concat(
							pageData.list.map((item: Record<string, unknown>) => ({
								...item,
								source_name: apiName,
								source_code: apiKey
							}))
						);
					}
				} catch {}
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
	yellowFilterEnabled: boolean,
	commentaryFilterEnabled: boolean = false
): Promise<SearchResult[]> {
	if (!query.trim() && selectedApis.length === 0) return [];

	const results = await Promise.all(
		selectedApis.map((apiKey) => {
			const customApi = apiKey.startsWith('custom_')
				? customApis[parseInt(apiKey.replace('custom_', ''), 10)]
				: undefined;
			return searchSingleSource(query, apiKey, customApi);
		})
	);

	let allResults = results.flat();

	if (yellowFilterEnabled) {
		const { YELLOW_FILTER_BANNED } = await import('./constants');
		allResults = allResults.filter((item) => {
			const typeName = item.type_name || '';
			return !YELLOW_FILTER_BANNED.some((keyword) => typeName.includes(keyword));
		});
	}

	if (commentaryFilterEnabled) {
		allResults = allResults.filter((item) => {
			const remarks = item.vod_remarks || '';
			const typeName = item.type_name || '';
			return !remarks.includes('解说') && !typeName.includes('解说');
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
	if (result.status < 200 || result.status >= 300) return null;

	try {
		return JSON.parse(result.body);
	} catch {
		return null;
	}
}

export function parsePlayUrl(vodPlayUrl: string): { episode: string; url: string }[] {
	if (!vodPlayUrl) return [];
	return vodPlayUrl.split('#').reduce<{ episode: string; url: string }[]>((eps, source) => {
		const parts = source.split('$');
		if (parts.length >= 2) {
			eps.push({ episode: parts[0], url: parts.slice(1).join('$') });
		}
		return eps;
	}, []);
}
