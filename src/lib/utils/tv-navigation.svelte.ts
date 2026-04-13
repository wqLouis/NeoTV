export type FocusRegion = 'none' | 'tabs' | 'genres' | 'grid';

export interface TvnavigationState {
	focusRegion: FocusRegion;
	focusedTabIndex: number;
	focusedGenreIndex: number;
	focusedCardIndex: number;
}

export function createTvnavigation() {
	let state = $state<TvnavigationState>({
		focusRegion: 'tabs',
		focusedTabIndex: 0,
		focusedGenreIndex: -1,
		focusedCardIndex: -1
	});

	function handleKeydown(
		e: KeyboardEvent,
		options: { tabCount: number; genreCount: number; cardCount: number },
		callbacks: {
			onTabChange?: (index: number) => void;
			onGenreChange?: (index: number) => void;
			onCardClick?: (index: number) => void;
		}
	) {
		const { tabCount, genreCount, cardCount } = options;

		switch (e.key) {
			case 'ArrowUp':
				e.preventDefault();
				if (state.focusRegion === 'tabs') {
					break;
				} else if (state.focusRegion === 'genres') {
					state.focusedGenreIndex = -1;
					state.focusRegion = 'tabs';
					state.focusedTabIndex = 0;
				} else if (state.focusRegion === 'grid') {
					state.focusedCardIndex = -1;
					state.focusedGenreIndex = 0;
					state.focusRegion = 'genres';
				}
				break;
			case 'ArrowDown':
				e.preventDefault();
				if (state.focusRegion === 'tabs') {
					state.focusedTabIndex = -1;
					state.focusedGenreIndex = 0;
					state.focusRegion = 'genres';
				} else if (state.focusRegion === 'genres') {
					state.focusedGenreIndex = -1;
					state.focusRegion = 'grid';
					state.focusedCardIndex = 0;
				} else if (state.focusRegion === 'grid') {
					break;
				}
				break;
			case 'ArrowLeft':
				e.preventDefault();
				if (state.focusRegion === 'tabs') {
					state.focusedTabIndex = Math.max(0, state.focusedTabIndex - 1);
				} else if (state.focusRegion === 'genres') {
					state.focusedGenreIndex = Math.max(0, state.focusedGenreIndex - 1);
				} else if (state.focusRegion === 'grid') {
					state.focusedCardIndex = Math.max(0, state.focusedCardIndex - 1);
				}
				break;
			case 'ArrowRight':
				e.preventDefault();
				if (state.focusRegion === 'tabs') {
					state.focusedTabIndex = Math.min(tabCount - 1, state.focusedTabIndex + 1);
				} else if (state.focusRegion === 'genres') {
					state.focusedGenreIndex = Math.min(genreCount - 1, state.focusedGenreIndex + 1);
				} else if (state.focusRegion === 'grid') {
					state.focusedCardIndex = Math.min(cardCount - 1, state.focusedCardIndex + 1);
				}
				break;
			case 'Enter':
			case ' ':
				e.preventDefault();
				if (state.focusRegion === 'tabs') {
					callbacks.onTabChange?.(state.focusedTabIndex);
				} else if (state.focusRegion === 'genres' && state.focusedGenreIndex >= 0) {
					callbacks.onGenreChange?.(state.focusedGenreIndex);
				} else if (state.focusRegion === 'grid' && state.focusedCardIndex >= 0) {
					callbacks.onCardClick?.(state.focusedCardIndex);
				}
				break;
		}
	}

	function reset() {
		state.focusRegion = 'tabs';
		state.focusedTabIndex = 0;
		state.focusedGenreIndex = -1;
		state.focusedCardIndex = -1;
	}

	return {
		get state() {
			return state;
		},
		handleKeydown,
		reset
	};
}

export const tvNav = createTvnavigation();

export type Tvnavigation = ReturnType<typeof createTvnavigation>;
