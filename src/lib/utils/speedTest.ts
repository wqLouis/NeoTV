import { invoke } from '@tauri-apps/api/core';

export interface SpeedTestResult {
	source_id: string;
	source_name: string;
	latency_ms: number;
	download_speed_kbps: number;
	status: string;
	error?: string;
	network_id: string;
}

interface CachedSpeed {
	result: SpeedTestResult;
	timestamp: number;
}

const SPEED_CACHE_TTL_MS = 30 * 60 * 1000;
const speedCache = new Map<string, CachedSpeed>();

export async function loadSpeedCacheFromDisk(networkId: string): Promise<void> {
	try {
		const results = await invoke<SpeedTestResult[]>('speed_cache_load', { networkId });
		const now = Date.now();
		for (const result of results) {
			speedCache.set(result.source_id, { result, timestamp: now });
		}
	} catch (e) {
		console.error('[SpeedTest] Failed to load speed cache from disk:', e);
	}
}

export async function saveSpeedCacheToDisk(networkId: string): Promise<void> {
	try {
		await invoke('speed_cache_save', { networkId });
	} catch (e) {
		console.error('[SpeedTest] Failed to save speed cache to disk:', e);
	}
}

export function getSpeedCache(sourceId: string): SpeedTestResult | null {
	const cached = speedCache.get(sourceId);
	if (cached) {
		return cached.result;
	}
	return null;
}

export function getAllSpeedCache(): SpeedTestResult[] {
	return Array.from(speedCache.values()).map((c) => c.result);
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
