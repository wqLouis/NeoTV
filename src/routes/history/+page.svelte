<script lang="ts">
	import { goto } from '$app/navigation';
	import { historyStore, type HistoryItem } from '$lib/stores/history.svelte';
	import { Button } from '$lib/components/ui/button';
	import { settingsStore, GRID_DENSITY_CLASSES } from '$lib/stores/settings.svelte';
	import { Play, Trash2, Clock } from '@lucide/svelte';
	import PageHeader from '$lib/components/business/PageHeader.svelte';
	import EmptyState from '$lib/components/business/EmptyState.svelte';
	import DoubanCard from '$lib/components/DoubanCard.svelte';

	function handlePlay(item: HistoryItem) {
		const params = new URLSearchParams({
			id: item.id,
			source: item.source,
			title: item.title,
			...(item.cover && { cover: item.cover }),
			...(item.episode && { episode: item.episode }),
			...(item.episodeIndex !== undefined && { episodeIndex: item.episodeIndex.toString() })
		});
		goto(`/player?${params.toString()}`);
	}

	function handleRemove(item: HistoryItem, e: MouseEvent) {
		e.stopPropagation();
		historyStore.remove(item.id, item.source, item.episode);
	}

	function handleClearAll() {
		historyStore.clear();
	}
</script>

<div class="container mx-auto h-full px-4 py-6">
	<PageHeader title="历史记录">
		{#snippet actions()}
			{#if historyStore.items.length > 0}
				<Button variant="outline" size="sm" onclick={handleClearAll}>
					<Trash2 class="mr-1 h-4 w-4" />
					清空全部
				</Button>
			{/if}
		{/snippet}
	</PageHeader>

	{#if historyStore.items.length === 0}
		<EmptyState icon={Clock} message="暂无观看历史" />
	{:else}
		<div class="grid {GRID_DENSITY_CLASSES[settingsStore.gridDensity]} gap-4">
			{#each historyStore.items as item (item.id + item.source + item.episode)}
				<DoubanCard
					item={{
						id: item.id,
						title: item.title,
						cover: item.cover || '',
						cover_url: item.cover || '',
						rate: '',
						score: '',
						region: [],
						regions: [],
						types: item.episode ? [item.episode] : [],
						director: [],
						actors: []
					}}
				>
					{#snippet action()}
						<button
							class="rounded-full bg-black/50 p-1.5 transition-colors hover:bg-black/70"
							onclick={(e) => handleRemove(item, e)}
						>
							<Trash2 class="h-4 w-4 text-white" />
						</button>
					{/snippet}

					{#snippet overlay(args)}
						<div
							class="absolute right-0 bottom-0 left-0 bg-gradient-to-t from-black/60 to-transparent p-2"
						>
							<h3 class="line-clamp-2 text-xs font-medium text-white" title={args.title}>
								{args.title}
							</h3>
							{#if item.episode}
								<p class="text-xs text-white/70">{item.episode}</p>
							{/if}
							<div class="mt-1 flex items-center gap-2">
								<Button
									size="sm"
									onclick={(e) => {
										e.stopPropagation();
										handlePlay(item);
									}}
								>
									<Play class="mr-1 h-3 w-3" />
									播放
								</Button>
								<span class="text-xs text-white/50">{item.source}</span>
							</div>
						</div>
					{/snippet}
				</DoubanCard>
			{/each}
		</div>
	{/if}
</div>
