import { invoke } from '@tauri-apps/api/core';

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
	let favourites = $state<FavouriteItem[]>([]);
	let initialized = $state(false);

	async function loadFavourites() {
		try {
			const items = await invoke<any[]>('favourites_get_all');
			// Convert snake_case from Rust to camelCase for frontend
			favourites = items.map((item) => ({
				id: item.id,
				title: item.title,
				source: item.source,
				cover: item.cover,
				episode: item.episode,
				episodeIndex: item.episode_index,
				addedAt: item.added_at
			}));
		} catch (e) {
			console.error('[Favourites] Failed to load from Rust:', e);
			favourites = [];
		}
		initialized = true;
	}

	// Initial load
	loadFavourites();

	return {
		get items() {
			return favourites;
		},
		get loaded() {
			return initialized;
		},
		async add(item: Omit<FavouriteItem, 'addedAt'>) {
			try {
				// Convert camelCase to snake_case for Rust
				const rustItem = {
					id: item.id,
					title: item.title,
					source: item.source,
					cover: item.cover ?? null,
					episode: item.episode ?? null,
					episode_index: item.episodeIndex ?? null,
					added_at: Date.now()
				};
				await invoke('favourites_add', { item: rustItem });
				// Update local state
				const newItem: FavouriteItem = { ...item, addedAt: rustItem.added_at };
				favourites = [newItem, ...favourites];
			} catch (e) {
				console.error('[Favourites] Failed to add:', e);
			}
		},
		async remove(id: string, source: string, episode?: string) {
			try {
				await invoke('favourites_remove', { id, source, episode: episode ?? null });
				favourites = favourites.filter(
					(f) => !(f.id === id && f.source === source && f.episode === episode)
				);
			} catch (e) {
				console.error('[Favourites] Failed to remove:', e);
			}
		},
		has(id: string, source: string, episode?: string): boolean {
			return favourites.some((f) => f.id === id && f.source === source && f.episode === episode);
		},
		async clear() {
			try {
				await invoke('favourites_clear');
				favourites = [];
			} catch (e) {
				console.error('[Favourites] Failed to clear:', e);
			}
		},
		async refresh() {
			await loadFavourites();
		}
	};
}

export const favouritesStore = createFavouritesStore();
