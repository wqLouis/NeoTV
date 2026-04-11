import { browser } from '$app/environment';
import type { ApiSite } from '$lib/api/constants';

export type GridDensity = 'compact' | 'standard' | 'loose';

export interface Settings {
	selectedApis: string[];
	customApis: ApiSite[];
	doubanEnabled: boolean;
	doubanApiMode: 'all' | 'hot' | 'new';
	yellowFilterEnabled: boolean;
	adFilteringEnabled: boolean;
	autoplayEnabled: boolean;
	autoplayNextEpisode: boolean;
	episodesReversed: boolean;
	gridDensity: GridDensity;
	commentaryFilterEnabled: boolean;
	autoIntegrateSources: boolean;
}

const DEFAULT_SETTINGS: Settings = {
	selectedApis: ['tyyszy', 'xiaomaomi', 'dyttzy', 'bfzy', 'ruyi'],
	customApis: [],
	doubanEnabled: true,
	doubanApiMode: 'all',
	yellowFilterEnabled: true,
	adFilteringEnabled: true,
	autoplayEnabled: true,
	autoplayNextEpisode: true,
	episodesReversed: false,
	gridDensity: 'standard',
	commentaryFilterEnabled: true,
	autoIntegrateSources: true
};

export const GRID_DENSITY_CLASSES: Record<GridDensity, string> = {
	compact: 'grid-cols-8 gap-8',
	standard: 'grid-cols-6 gap-8',
	loose: 'grid-cols-5 gap-8'
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
		get autoplayNextEpisode() {
			return settings.autoplayNextEpisode;
		},
		get episodesReversed() {
			return settings.episodesReversed;
		},
		get gridDensity() {
			return settings.gridDensity;
		},
		get commentaryFilterEnabled() {
			return settings.commentaryFilterEnabled;
		},
		get autoIntegrateSources() {
			return settings.autoIntegrateSources;
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
		setAutoplayNextEpisode(enabled: boolean) {
			settings.autoplayNextEpisode = enabled;
			save();
		},
		setEpisodesReversed(reversed: boolean) {
			settings.episodesReversed = reversed;
			save();
		},
		setGridDensity(density: GridDensity) {
			settings.gridDensity = density;
			save();
		},
		setCommentaryFilterEnabled(enabled: boolean) {
			settings.commentaryFilterEnabled = enabled;
			save();
		},
		setAutoIntegrateSources(enabled: boolean) {
			settings.autoIntegrateSources = enabled;
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
					autoplayNextEpisode: settings.autoplayNextEpisode,
					episodesReversed: settings.episodesReversed,
					gridDensity: settings.gridDensity,
					commentaryFilterEnabled: settings.commentaryFilterEnabled,
					autoIntegrateSources: settings.autoIntegrateSources
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
