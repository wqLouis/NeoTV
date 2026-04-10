import { invoke } from '@tauri-apps/api/core';

const imageCache = new Map<string, string>();

export async function fetchImage(url: string, referer?: string): Promise<string> {
	if (!url) return '';

	if (imageCache.has(url)) {
		return imageCache.get(url)!;
	}

	try {
		const dataUrl: string = await invoke('fetch_url', { url, referer });
		imageCache.set(url, dataUrl);
		return dataUrl;
	} catch (e) {
		console.error('[Cache] Image fetch failed:', e);
		return url;
	}
}

export async function clearImageCache(): Promise<void> {
	imageCache.clear();
	await invoke('cache_clear');
}

export async function getCacheStats() {
	return invoke<{
		mem_count: number;
		mem_size: number;
		disk_count: number;
		disk_size: number;
	}>('cache_stats');
}

export function clearJsImageCache() {
	imageCache.clear();
}

export async function clearCache(): Promise<void> {
	clearJsImageCache();
	await invoke('cache_clear');
}
