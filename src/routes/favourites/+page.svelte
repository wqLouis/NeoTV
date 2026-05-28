<script lang="ts">
	import { favouritesStore, type FavouriteItem } from '$lib/stores/favourites.svelte';
	import type { DoubanSubject } from '$lib/api/douban';
	import VideoSourceOverlay from '$lib/components/VideoSourceOverlay.svelte';
	import { Button } from '$lib/components/ui/button';
	import { settingsStore, GRID_DENSITY_CLASSES } from '$lib/stores/settings.svelte';
	import { Trash2, Heart } from '@lucide/svelte';
	import { formatRelativeTime } from '$lib/utils/format';
	import PageHeader from '$lib/components/business/PageHeader.svelte';
	import EmptyState from '$lib/components/business/EmptyState.svelte';
	import DoubanCard from '$lib/components/DoubanCard.svelte';
	import { useIntlayer } from 'svelte-intlayer';
	

	const content = useIntlayer('common');

	let selectedVideo: DoubanSubject | null = $state(null);
	let showSourceOverlay = $state(false);
	let selectedCardRect: DOMRect | null = $state(null);
	let selectedItem: FavouriteItem | null = $state(null);

	const titleLabel = $derived(String($content.favouritesTitle.value));
	const noFavouritesLabel = $derived(String($content.noFavourites.value));
	const hintLabel = $derived(String($content.addFavouritesHint.value));
	const clearAllLabel = $derived(String($content.clearHistory.value));

	function favouriteToSubject(item: FavouriteItem): DoubanSubject {
		return {
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
		};
	}

	function handleVideoClick(item: FavouriteItem, e: MouseEvent) {
		selectedItem = item;
		selectedVideo = favouriteToSubject(item);
		selectedCardRect = (e.currentTarget as HTMLElement).getBoundingClientRect();
		showSourceOverlay = true;
	}

	function handleCardClick(item: FavouriteItem) {
		return (_subject: DoubanSubject, e: MouseEvent) => handleVideoClick(item, e);
	}

	function handleRemove(item: FavouriteItem, e: MouseEvent) {
		e.stopPropagation();
		favouritesStore.remove(item.id, item.source, item.episode);
	}

	function handleClearAll() {
		favouritesStore.clear();
	}
</script>

<div class="container mx-auto h-full px-4 py-6">
	<PageHeader title={titleLabel}>
		{#snippet actions()}
			{#if favouritesStore.items.length > 0}
				<Button variant="outline" size="sm" onclick={handleClearAll}>
					<Trash2 class="mr-1 h-4 w-4" />
					{clearAllLabel}
				</Button>
			{/if}
		{/snippet}
	</PageHeader>

	{#if favouritesStore.items.length === 0}
		<EmptyState icon={Heart} message={noFavouritesLabel} description={hintLabel} />
	{:else}
		<div class="grid {GRID_DENSITY_CLASSES[settingsStore.gridDensity]} gap-4">
			{#each favouritesStore.items as item (item.id + item.source + item.episode)}
				<DoubanCard item={favouriteToSubject(item)} onclick={handleCardClick(item)}>
					{#snippet action()}
						<button
							class="rounded-full bg-black/50 p-1.5 transition-colors hover:bg-black/70"
							onclick={(e) => handleRemove(item, e)}
						>
							<Heart class="h-4 w-4 fill-primary text-primary" />
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
							<p class="text-xs text-white/50">
								{item.source} · {formatRelativeTime(item.addedAt)}
							</p>
						</div>
					{/snippet}
				</DoubanCard>
			{/each}
		</div>
	{/if}
</div>

<VideoSourceOverlay
	item={selectedVideo}
	originRect={selectedCardRect}
	open={showSourceOverlay}
	onOpenChange={(open) => (showSourceOverlay = open)}
/>
