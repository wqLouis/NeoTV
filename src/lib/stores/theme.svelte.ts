import { browser } from '$app/environment';
import { onMount } from 'svelte';

type Theme = 'light' | 'dark' | 'system';

function createThemeStore() {
	let theme = $state<Theme>(loadTheme());

	function loadTheme(): Theme {
		if (!browser) return 'system';
		const stored = localStorage.getItem('theme');
		if (stored === 'light' || stored === 'dark' || stored === 'system') {
			return stored;
		}
		return 'system';
	}

	function applyTheme(t: Theme) {
		if (!browser) return;
		const root = document.documentElement;
		if (t === 'dark') {
			root.classList.add('dark');
		} else if (t === 'light') {
			root.classList.remove('dark');
		} else {
			if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
				root.classList.add('dark');
			} else {
				root.classList.remove('dark');
			}
		}
	}

	function save() {
		if (browser) {
			localStorage.setItem('theme', theme);
			applyTheme(theme);
		}
	}

	return {
		get current() {
			return theme;
		},
		setTheme(t: Theme) {
			theme = t;
			save();
		},
		init() {
			applyTheme(theme);
			if (browser) {
				const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
				mediaQuery.addEventListener('change', () => {
					if (theme === 'system') {
						applyTheme('system');
					}
				});
			}
		}
	};
}

export const themeStore = createThemeStore();
