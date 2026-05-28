<script lang="ts">
	import HorizontalSection from '$lib/components/HorizontalSection.svelte';
	import PageTabBar from '$lib/components/business/PageTabBar.svelte';
	import { Film, Tv } from '@lucide/svelte';
	import { useIntlayer } from 'svelte-intlayer';

	const content = useIntlayer('home');

	let selectedType = $state<'movie' | 'tv'>('movie');

	let typeOptions = $derived([
		{ value: 'movie', label: String($content.movie.value), icon: Film },
		{ value: 'tv', label: String($content.tv.value), icon: Tv }
	]);

	let hotMovieTitle = $derived(String($content.hotMovies.value));
	let latestMovieTitle = $derived(String($content.latestMovies.value));
	let hotTvTitle = $derived(String($content.hotTv.value));
	let latestTvTitle = $derived(String($content.latestTv.value));

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
