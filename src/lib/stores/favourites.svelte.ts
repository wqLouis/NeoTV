import { browser } from '$app/environment';

export interface FavouriteItem {
	id: string;
	title: string;
	source: string;
	cover?: string;
	episode?: string;
	episodeIndex?: number;
	addedAt: number;
}

function createFavouritesStore() {
	let favourites = $state<FavouriteItem[]>(loadFavourites());

	function loadFavourites(): FavouriteItem[] {
		if (!browser) return [];
		const stored = localStorage.getItem('favourites');
		if (!stored) return [];
		try {
			return JSON.parse(stored);
		} catch {
			return [];
		}
	}

	function save() {
		if (browser) {
			localStorage.setItem('favourites', JSON.stringify(favourites));
		}
	}

	return {
		get items() {
			return favourites;
		},
		add(item: Omit<FavouriteItem, 'addedAt'>) {
			const exists = favourites.some(
				(f) => f.id === item.id && f.source === item.source && f.episode === item.episode
			);
			if (!exists) {
				favourites = [{ ...item, addedAt: Date.now() }, ...favourites];
				save();
			}
		},
		remove(id: string, source: string, episode?: string) {
			favourites = favourites.filter(
				(f) => !(f.id === id && f.source === source && f.episode === episode)
			);
			save();
		},
		has(id: string, source: string, episode?: string): boolean {
			return favourites.some((f) => f.id === id && f.source === source && f.episode === episode);
		},
		clear() {
			favourites = [];
			save();
		}
	};
}

export const favouritesStore = createFavouritesStore();
