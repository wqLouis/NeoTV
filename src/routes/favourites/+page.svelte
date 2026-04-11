<script lang="ts">
	import { favouritesStore, type FavouriteItem } from '$lib/stores/favourites.svelte';
	import type { DoubanSubject } from '$lib/api/douban';
	import VideoSourceOverlay from '$lib/components/VideoSourceOverlay.svelte';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import CachedImage from '$lib/components/CachedImage.svelte';
	import { settingsStore, GRID_DENSITY_CLASSES } from '$lib/stores/settings.svelte';
	import { Trash2, Heart } from 'lucide-svelte';
	import { formatRelativeTime } from '$lib/utils/format';

	let selectedVideo: DoubanSubject | null = $state(null);
	let showSourceOverlay = $state(false);
	let selectedCardRect: DOMRect | null = $state(null);

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

	function handleVideoClick(item: FavouriteItem, event: MouseEvent) {
		const target = event.currentTarget as HTMLDivElement;
		selectedCardRect = target.getBoundingClientRect();
		selectedVideo = favouriteToSubject(item);
		showSourceOverlay = true;
	}

	function handleRemove(item: FavouriteItem, e: MouseEvent) {
		e.stopPropagation();
		favouritesStore.remove(item.id, item.source, item.episode);
	}

	function handleClearAll() {
		favouritesStore.clear();
	}
</script>

<div class="container mx-auto px-4 py-6">
	<div class="mb-6 flex items-center justify-between">
		<h1 class="text-2xl font-bold">我的收藏</h1>
		{#if favouritesStore.items.length > 0}
			<Button variant="outline" size="sm" onclick={handleClearAll}>
				<Trash2 class="mr-1 h-4 w-4" />
				清空全部
			</Button>
		{/if}
	</div>

	{#if favouritesStore.items.length === 0}
		<div class="py-12 text-center text-muted-foreground">
			<Heart class="mx-auto mb-4 h-12 w-12 opacity-50" />
			<p>暂无收藏内容</p>
			<p class="mt-1 text-sm">在播放器中点击收藏按钮添加内容</p>
		</div>
	{:else}
		<div class="grid {GRID_DENSITY_CLASSES[settingsStore.gridDensity]} gap-4">
			{#each favouritesStore.items as item (item.id + item.source + item.episode)}
				<div
					class="group relative cursor-pointer overflow-hidden rounded-lg bg-card transition-all hover:scale-[1.02] hover:shadow-md"
					onclick={(e) => handleVideoClick(item, e)}
					role="button"
					tabindex="0"
					onkeydown={(e) => e.key === 'Enter' && handleVideoClick(item, e as unknown as MouseEvent)}
				>
					<div class="relative aspect-[2/3] w-full overflow-hidden">
						{#if item.cover}
							<CachedImage
								src={item.cover}
								alt={item.title}
								class="h-full w-full object-cover transition-transform group-hover:scale-110"
							/>
						{:else}
							<div class="flex h-full w-full items-center justify-center bg-secondary">
								<span class="text-muted-foreground">无封面</span>
							</div>
						{/if}
						<button
							class="absolute top-1.5 right-1.5 rounded-full bg-black/50 p-1.5 opacity-0 transition-opacity group-hover:opacity-100 hover:bg-black/70"
							onclick={(e) => handleRemove(item, e)}
						>
							<Heart class="h-4 w-4 fill-primary text-primary" />
						</button>
						{#if item.episode}
							<Badge class="absolute bottom-1.5 left-1.5 bg-black/50 text-xs text-white">
								{item.episode}
							</Badge>
						{/if}
					</div>
					<div class="p-2">
						<h3 class="line-clamp-2 text-xs font-medium" title={item.title}>
							{item.title}
						</h3>
						<div class="mt-1 flex items-center justify-between">
							<span class="text-xs text-muted-foreground">{item.source}</span>
							<span class="text-xs text-muted-foreground">
								{formatRelativeTime(item.addedAt)}
							</span>
						</div>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

<VideoSourceOverlay
	item={selectedVideo}
	originRect={selectedCardRect}
	bind:open={showSourceOverlay}
	onOpenChange={(open) => (showSourceOverlay = open)}
/>
