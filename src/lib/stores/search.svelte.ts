import { browser } from '$app/environment';

const MAX_HISTORY_ITEMS = 20;

function createSearchHistoryStore() {
	let history = $state<string[]>(loadHistory());

	function loadHistory(): string[] {
		if (!browser) return [];
		const stored = localStorage.getItem('videoSearchHistory');
		if (!stored) return [];
		try {
			return JSON.parse(stored);
		} catch {
			return [];
		}
	}

	function save() {
		if (browser) {
			localStorage.setItem('videoSearchHistory', JSON.stringify(history));
		}
	}

	return {
		get items() {
			return history;
		},
		add(query: string) {
			const trimmed = query.trim();
			if (!trimmed) return;
			history = [trimmed, ...history.filter((q) => q !== trimmed)].slice(0, MAX_HISTORY_ITEMS);
			save();
		},
		remove(query: string) {
			history = history.filter((q) => q !== query);
			save();
		},
		clear() {
			history = [];
			save();
		}
	};
}

export const searchHistoryStore = createSearchHistoryStore();
