<script lang="ts">
	import HorizontalSection from '$lib/components/HorizontalSection.svelte';
	import PageTabBar from '$lib/components/business/PageTabBar.svelte';
	import { Film, Tv } from '@lucide/svelte';

	let selectedType = $state<'movie' | 'tv'>('movie');

	let typeOptions = $derived([
		{ value: 'movie', label: '电影', icon: Film },
		{ value: 'tv', label: '电视剧', icon: Tv }
	]);

	let hotMovieTitle = '热门电影';
	let latestMovieTitle = '最新电影';
	let hotTvTitle = '热门电视剧';
	let latestTvTitle = '最新电视剧';

	function handleTypeChange(v: string) {
		selectedType = v as typeof selectedType;
	}
</script>

<div class="h-full overflow-y-auto">
	<PageTabBar options={typeOptions} value={selectedType} onchange={handleTypeChange} />

	<main class="grid grid-rows-2 gap-10 overflow-hidden p-4">
		{#if selectedType === 'movie'}
			<HorizontalSection title={hotMovieTitle} type="movie" tag="热门" sort="recommend" />
			<HorizontalSection title={latestMovieTitle} type="movie" tag="最新" sort="time" />
		{:else}
			<HorizontalSection title={hotTvTitle} type="tv" tag="热门" sort="recommend" />
			<HorizontalSection title={latestTvTitle} type="tv" tag="最新" sort="time" />
		{/if}
	</main>
</div>
