<script lang="ts">
	import HorizontalSection from '$lib/components/HorizontalSection.svelte';
	import { Film, Tv } from '@lucide/svelte';

	let selectedType = $state<'movie' | 'tv'>('movie');
</script>

<div class="h-full overflow-y-auto">
	<div class="sticky top-0 z-30 border-b bg-background/90 px-4 pt-8 pb-3 backdrop-blur-2xl">
		<div class="flex gap-4">
			<button
				class="flex items-center gap-2 rounded-lg px-4 py-2 transition-colors
					{selectedType === 'movie'
					? 'bg-primary text-primary-foreground'
					: 'text-muted-foreground hover:bg-secondary hover:text-foreground'}"
				onclick={() => (selectedType = 'movie')}
			>
				<Film class="h-5 w-5" />
				<span class="font-medium">电影</span>
			</button>

			<button
				class="flex items-center gap-2 rounded-lg px-4 py-2 transition-colors
					{selectedType === 'tv'
					? 'bg-primary text-primary-foreground'
					: 'text-muted-foreground hover:bg-secondary hover:text-foreground'}"
				onclick={() => (selectedType = 'tv')}
			>
				<Tv class="h-5 w-5" />
				<span class="font-medium">电视剧</span>
			</button>
		</div>
	</div>

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
