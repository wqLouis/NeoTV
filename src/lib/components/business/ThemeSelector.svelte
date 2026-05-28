<script lang="ts">
	import { themeStore } from '$lib/stores/theme.svelte';
	import { settingsStore } from '$lib/stores/settings.svelte';
	import type { GridDensity } from '$lib/stores/settings.svelte';
	import { useIntlayer } from 'svelte-intlayer';
	

	interface Props {
		class?: string;
	}

	let { class: className = '' }: Props = $props();

	const content = useIntlayer('settings');

	const themes = $derived([
		{ value: 'light' as const, label: String($content.light) },
		{ value: 'dark' as const, label: String($content.dark) },
		{ value: 'system' as const, label: String($content.system) }
	]);

	const densities = $derived([
		{ value: 'compact' as GridDensity, label: String($content.compact), cols: '8' },
		{ value: 'standard' as GridDensity, label: String($content.standard), cols: '6' },
		{ value: 'loose' as GridDensity, label: String($content.loose), cols: '5' }
	]);

	const gridDensityLabel = $derived(String($content.gridDensity));
</script>

<div class="space-y-4 {className}">
	<div class="flex gap-2">
		{#each themes as theme}
			<button
				class="flex flex-1 flex-col items-center gap-2 rounded-lg border p-4 transition-colors
					{themeStore.current === theme.value
					? 'border-primary bg-primary/10'
					: 'border-border hover:bg-accent'}"
				onclick={() => themeStore.setTheme(theme.value)}
			>
				{#if theme.value === 'light'}
					<svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z"
						/>
					</svg>
				{:else if theme.value === 'dark'}
					<svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z"
						/>
					</svg>
				{:else}
					<svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"
						/>
					</svg>
				{/if}
				<span class="text-sm">{theme.label}</span>
			</button>
		{/each}
	</div>

	<div>
		<div class="mb-3 text-sm font-medium">{gridDensityLabel}</div>
		<div class="flex gap-2">
			{#each densities as density}
				<button
					class="flex flex-1 flex-col items-center gap-1 rounded-lg border p-3 transition-colors
						{settingsStore.gridDensity === density.value
						? 'border-primary bg-primary/10'
						: 'border-border hover:bg-accent'}"
					onclick={() => settingsStore.setGridDensity(density.value)}
				>
					<span class="text-sm font-medium">{density.label}</span>
					<span class="text-xs text-muted-foreground">{density.cols}</span>
				</button>
			{/each}
		</div>
	</div>
</div>
