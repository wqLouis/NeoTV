<script lang="ts">
	import HorizontalSection from '$lib/components/HorizontalSection.svelte';
	import PageTabBar from '$lib/components/business/PageTabBar.svelte';
	import { Film, Tv } from '@lucide/svelte';

	let selectedType = $state<'movie' | 'tv'>('movie');

	const typeOptions = [
		{ value: 'movie', label: '电影', icon: Film },
		{ value: 'tv', label: '电视剧', icon: Tv }
	];
</script>

<div class="h-full overflow-y-auto">
	<PageTabBar
		options={typeOptions}
		value={selectedType}
		onchange={(v) => (selectedType = v as typeof selectedType)}
	/>

	<main class="grid grid-rows-2 gap-10 overflow-hidden p-4">
		{#if selectedType === 'movie'}
			<HorizontalSection title="热门电影" type="movie" tag="热门" sort="recommend" />
			<HorizontalSection title="最新电影" type="movie" tag="最新" sort="time" />
		{:else}
			<HorizontalSection title="热门电视剧" type="tv" tag="热门" sort="recommend" />
			<HorizontalSection title="最新电视剧" type="tv" tag="热门" sort="time" />
		{/if}
	</main>
</div>
