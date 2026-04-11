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
