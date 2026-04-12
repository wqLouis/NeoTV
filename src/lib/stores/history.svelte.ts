import { invoke } from '@tauri-apps/api/core';

export interface HistoryItem {
	id: string;
	title: string;
	source: string;
	cover?: string;
	episode?: string;
	episodeIndex?: number;
}

function createHistoryStore() {
	let history = $state<HistoryItem[]>([]);
	let initialized = $state(false);

	async function loadHistory() {
		try {
			history = await invoke<HistoryItem[]>('history_get_all');
		} catch (e) {
			console.error('[History] Failed to load from Rust:', e);
			history = [];
		}
		initialized = true;
	}

	loadHistory();

	return {
		get items() {
			return history;
		},
		get loaded() {
			return initialized;
		},
		async add(item: HistoryItem) {
			try {
				await invoke('history_add', { item });
				const existingIdx = history.findIndex(
					(h) => h.id === item.id && h.source === item.source && h.episode === item.episode
				);
				if (existingIdx >= 0) {
					history = history.map((h, i) => (i === existingIdx ? item : h));
				} else {
					history = [item, ...history].slice(0, 100);
				}
			} catch (e) {
				console.error('[History] Failed to add:', e);
			}
		},
		async remove(id: string, source: string, episode?: string) {
			try {
				await invoke('history_remove', { id, source, episode: episode ?? null });
				history = history.filter(
					(h) => !(h.id === id && h.source === source && h.episode === episode)
				);
			} catch (e) {
				console.error('[History] Failed to remove:', e);
			}
		},
		async clear() {
			try {
				await invoke('history_clear');
				history = [];
			} catch (e) {
				console.error('[History] Failed to clear:', e);
			}
		},
		async refresh() {
			await loadHistory();
		}
	};
}

export const historyStore = createHistoryStore();
