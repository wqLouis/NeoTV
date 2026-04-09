import { browser } from '$app/environment';
import type { ApiSite } from '$lib/api/constants';

export interface Settings {
	selectedApis: string[];
	customApis: ApiSite[];
	doubanEnabled: boolean;
	doubanApiMode: 'all' | 'hot' | 'new';
	yellowFilterEnabled: boolean;
	adFilteringEnabled: boolean;
	autoplayEnabled: boolean;
	episodesReversed: boolean;
}

const DEFAULT_SETTINGS: Settings = {
	selectedApis: ['tyyszy', 'xiaomaomi', 'dyttzy', 'bfzy', 'ruyi'],
	customApis: [],
	doubanEnabled: true,
	doubanApiMode: 'all',
	yellowFilterEnabled: true,
	adFilteringEnabled: true,
	autoplayEnabled: true,
	episodesReversed: false
};

function createSettingsStore() {
	let settings = $state<Settings>(loadSettings());

	function loadSettings(): Settings {
		if (!browser) return DEFAULT_SETTINGS;
		const stored = localStorage.getItem('appSettings');
		if (!stored) return DEFAULT_SETTINGS;
		try {
			return { ...DEFAULT_SETTINGS, ...JSON.parse(stored) };
		} catch {
			return DEFAULT_SETTINGS;
		}
	}

	function save() {
		if (browser) {
			localStorage.setItem('appSettings', JSON.stringify(settings));
		}
	}

	return {
		get selectedApis() {
			return settings.selectedApis;
		},
		get customApis() {
			return settings.customApis;
		},
		get doubanEnabled() {
			return settings.doubanEnabled;
		},
		get doubanApiMode() {
			return settings.doubanApiMode;
		},
		get yellowFilterEnabled() {
			return settings.yellowFilterEnabled;
		},
		get adFilteringEnabled() {
			return settings.adFilteringEnabled;
		},
		get autoplayEnabled() {
			return settings.autoplayEnabled;
		},
		get episodesReversed() {
			return settings.episodesReversed;
		},
		setSelectedApis(apis: string[]) {
			settings.selectedApis = apis;
			save();
		},
		toggleApi(apiKey: string) {
			const idx = settings.selectedApis.indexOf(apiKey);
			if (idx >= 0) {
				settings.selectedApis = settings.selectedApis.filter((k) => k !== apiKey);
			} else {
				settings.selectedApis = [...settings.selectedApis, apiKey];
			}
			save();
		},
		addCustomApi(api: ApiSite) {
			settings.customApis = [...settings.customApis, api];
			save();
		},
		removeCustomApi(index: number) {
			settings.customApis = settings.customApis.filter((_, i) => i !== index);
			save();
		},
		setDoubanEnabled(enabled: boolean) {
			settings.doubanEnabled = enabled;
			save();
		},
		setDoubanApiMode(mode: 'all' | 'hot' | 'new') {
			settings.doubanApiMode = mode;
			save();
		},
		setYellowFilterEnabled(enabled: boolean) {
			settings.yellowFilterEnabled = enabled;
			save();
		},
		setAdFilteringEnabled(enabled: boolean) {
			settings.adFilteringEnabled = enabled;
			save();
		},
		setAutoplayEnabled(enabled: boolean) {
			settings.autoplayEnabled = enabled;
			save();
		},
		setEpisodesReversed(reversed: boolean) {
			settings.episodesReversed = reversed;
			save();
		},
		exportConfig(): string {
			return JSON.stringify(
				{
					selectedApis: settings.selectedApis,
					customApis: settings.customApis,
					doubanEnabled: settings.doubanEnabled,
					doubanApiMode: settings.doubanApiMode,
					yellowFilterEnabled: settings.yellowFilterEnabled,
					adFilteringEnabled: settings.adFilteringEnabled,
					autoplayEnabled: settings.autoplayEnabled,
					episodesReversed: settings.episodesReversed
				},
				null,
				2
			);
		},
		importConfig(jsonStr: string): boolean {
			try {
				const imported = JSON.parse(jsonStr);
				settings = { ...DEFAULT_SETTINGS, ...imported };
				save();
				return true;
			} catch {
				return false;
			}
		},
		reset() {
			settings = { ...DEFAULT_SETTINGS };
			save();
		}
	};
}

export const settingsStore = createSettingsStore();
