import { API_SITES } from '$lib/api/constants';
import { invoke } from '@tauri-apps/api/core';

export interface SpeedTestResult {
	source_id: string;
	source_name: string;
	latency_ms: number;
	download_speed_kbps: number;
	status: string;
	error?: string;
}

interface CachedSpeed {
	result: SpeedTestResult;
	timestamp: number;
}

const SPEED_CACHE_TTL_MS = 30 * 60 * 1000;
const speedCache = new Map<string, CachedSpeed>();

export async function testSourceSpeed(
	sourceId: string,
	customUrl?: string
): Promise<SpeedTestResult> {
	const cacheKey = customUrl ? `custom_${customUrl}` : sourceId;
	const now = Date.now();

	if (speedCache.has(cacheKey)) {
		const cached = speedCache.get(cacheKey)!;
		if (now - cached.timestamp < SPEED_CACHE_TTL_MS) {
			return cached.result;
		}
	}

	try {
		const result = await invoke<SpeedTestResult>('test_source_speed', {
			sourceId,
			customUrl: customUrl || null
		});
		speedCache.set(cacheKey, { result, timestamp: now });
		return result;
	} catch (e) {
		return {
			source_id: sourceId,
			source_name: getSourceName(sourceId),
			latency_ms: 0,
			download_speed_kbps: 0,
			status: 'error',
			error: String(e)
		};
	}
}

export async function testAllSourcesSpeed(
	selectedApis: string[],
	customApis: { api: string; name: string }[]
): Promise<SpeedTestResult[]> {
	const results: SpeedTestResult[] = [];

	for (const apiKey of selectedApis) {
		if (apiKey.startsWith('custom_')) {
			const idx = parseInt(apiKey.replace('custom_', ''), 10);
			const customApi = customApis[idx];
			if (customApi) {
				const result = await testSourceSpeed('custom', customApi.api);
				result.source_name = customApi.name;
				results.push(result);
			}
		} else {
			const result = await testSourceSpeed(apiKey);
			results.push(result);
		}
	}

	return results;
}

function getSourceName(sourceId: string): string {
	if (sourceId === 'custom') return '自定义源';
	const site = API_SITES[sourceId];
	return site?.name || '未知源';
}

export function formatLatency(ms: number): string {
	if (ms === 0) return '-';
	if (ms < 1000) return `${ms}ms`;
	return `${(ms / 1000).toFixed(1)}s`;
}

export function formatSpeed(kbps: number): string {
	if (kbps === 0) return '-';
	if (kbps < 1024) return `${kbps.toFixed(0)} K/s`;
	const mbps = kbps / 1024;
	if (mbps < 1024) return `${mbps.toFixed(1)} M/s`;
	return `${(mbps / 1024).toFixed(1)} G/s`;
}

export function getSpeedLevel(kbps: number): 'fast' | 'medium' | 'slow' | 'unknown' {
	if (kbps === 0) return 'unknown';
	if (kbps > 500000) return 'fast';
	if (kbps > 100000) return 'medium';
	return 'slow';
}
