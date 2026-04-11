<script lang="ts">
	import HorizontalSection from '$lib/components/HorizontalSection.svelte';
	import { Film, Tv } from 'lucide-svelte';

	let selectedType = $state<'movie' | 'tv'>('movie');
</script>

<div class="flex h-full">
	<aside
		class="sticky top-0 z-30 flex h-fit w-40 flex-col gap-2 border-r bg-background/90 p-4 pt-8 backdrop-blur-2xl"
	>
		<button
			class="flex items-center gap-3 rounded-lg px-3 py-2.5 text-left transition-colors
				{selectedType === 'movie'
				? 'bg-primary/10 text-primary'
				: 'text-muted-foreground hover:bg-secondary hover:text-foreground'}"
			onclick={() => (selectedType = 'movie')}
		>
			<Film class="h-5 w-5" />
			<span class="font-medium">电影</span>
		</button>

		<button
			class="flex items-center gap-3 rounded-lg px-3 py-2.5 text-left transition-colors
				{selectedType === 'tv'
				? 'bg-primary/10 text-primary'
				: 'text-muted-foreground hover:bg-secondary hover:text-foreground'}"
			onclick={() => (selectedType = 'tv')}
		>
			<Tv class="h-5 w-5" />
			<span class="font-medium">电视剧</span>
		</button>
	</aside>

	<main class="flex-1 overflow-y-auto p-4">
		{#if selectedType === 'movie'}
			<div class="space-y-10">
				<HorizontalSection title="热门电影" type="movie" sort="T" />
				<HorizontalSection title="最新电影" type="movie" sort="R" />
			</div>
		{:else}
			<div class="space-y-10">
				<HorizontalSection title="热门电视剧" type="tv" sort="T" />
				<HorizontalSection title="最新电视剧" type="tv" sort="R" />
			</div>
		{/if}
	</main>
</div>
