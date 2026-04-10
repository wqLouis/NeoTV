<script lang="ts">
	interface Episode {
		episode: string;
		url: string;
	}

	interface Props {
		episodes: Episode[];
		currentIndex?: number;
		reversed?: boolean;
		onSelect: (episode: Episode, index: number) => void;
	}

	let { episodes, currentIndex = -1, reversed = false, onSelect }: Props = $props();

	let displayEpisodes = $derived(reversed ? [...episodes].reverse() : episodes);
</script>

<div class="grid grid-cols-4 gap-2 sm:grid-cols-6 md:grid-cols-8 lg:grid-cols-10">
	{#each displayEpisodes as episode, i (episode.url)}
		{@const actualIndex = reversed ? episodes.length - 1 - i : i}
		<button
			class="rounded-md px-3 py-2 text-center text-sm transition-colors
				{actualIndex === currentIndex
				? 'bg-primary text-primary-foreground'
				: 'bg-secondary hover:bg-secondary/80'}"
			onclick={() => onSelect(episode, actualIndex)}
		>
			{episode.episode}
		</button>
	{/each}
</div>
