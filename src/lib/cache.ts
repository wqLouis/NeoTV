import { invoke } from '@tauri-apps/api/core';

const MAX_CACHE_SIZE = 500;
const imageCache = new Map<string, string>();
const pendingRequests = new Map<string, Promise<string>>();

function evictOldest() {
	if (imageCache.size >= MAX_CACHE_SIZE) {
		const firstKey = imageCache.keys().next().value;
		if (firstKey) {
			imageCache.delete(firstKey);
		}
	}
}

export async function fetchImage(url: string, referer?: string): Promise<string> {
	if (!url) return '';

	if (imageCache.has(url)) {
		return imageCache.get(url)!;
	}

	if (pendingRequests.has(url)) {
		return pendingRequests.get(url)!;
	}

	const requestPromise = (async () => {
		try {
			evictOldest();
			const dataUrl: string = await invoke('fetch_url', { url, referer });
			imageCache.set(url, dataUrl);
			return dataUrl;
		} catch (e) {
			console.error('[Cache] Image fetch failed:', e);
			return url;
		} finally {
			pendingRequests.delete(url);
		}
	})();

	pendingRequests.set(url, requestPromise);
	return requestPromise;
}
