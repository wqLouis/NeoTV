import { browser } from '$app/environment';

export interface HistoryItem {
	id: string;
	title: string;
	source: string;
	cover?: string;
	episode?: string;
	episodeIndex?: number;
	position: number;
	duration: number;
	timestamp: number;
}

function createHistoryStore() {
	let history = $state<HistoryItem[]>(loadHistory());

	function loadHistory(): HistoryItem[] {
		if (!browser) return [];
		const stored = localStorage.getItem('viewingHistory');
		if (!stored) return [];
		try {
			return JSON.parse(stored);
		} catch {
			return [];
		}
	}

	function save() {
		if (browser) {
			localStorage.setItem('viewingHistory', JSON.stringify(history));
		}
	}

	return {
		get items() {
			return history;
		},
		add(item: Omit<HistoryItem, 'timestamp'>) {
			const existing = history.findIndex(
				(h) => h.id === item.id && h.source === item.source && h.episode === item.episode
			);
			const newItem: HistoryItem = { ...item, timestamp: Date.now() };
			if (existing >= 0) {
				history = history.map((h, i) => (i === existing ? newItem : h));
			} else {
				history = [newItem, ...history].slice(0, 100);
			}
			save();
		},
		updatePosition(
			id: string,
			source: string,
			episode: string | undefined,
			position: number,
			duration: number
		) {
			const idx = history.findIndex(
				(h) => h.id === id && h.source === source && h.episode === episode
			);
			if (idx >= 0) {
				history = history.map((h, i) =>
					i === idx ? { ...h, position, duration, timestamp: Date.now() } : h
				);
				save();
			}
		},
		remove(id: string, source: string, episode?: string) {
			history = history.filter(
				(h) => !(h.id === id && h.source === source && h.episode === episode)
			);
			save();
		},
		clear() {
			history = [];
			save();
		}
	};
}

export const historyStore = createHistoryStore();
