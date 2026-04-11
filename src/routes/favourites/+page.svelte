<script lang="ts">
	import { favouritesStore, type FavouriteItem } from '$lib/stores/favourites.svelte';
	import type { DoubanSubject } from '$lib/api/douban';
	import VideoSourceOverlay from '$lib/components/VideoSourceOverlay.svelte';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Card, CardContent } from '$lib/components/ui/card';
	import CachedImage from '$lib/components/CachedImage.svelte';
	import { Play, Trash2, Heart } from 'lucide-svelte';
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

	function handleRemove(item: FavouriteItem) {
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
		<div class="space-y-3">
			{#each favouritesStore.items as item (item.id + item.source + item.episode)}
				<Card
					class="cursor-pointer transition-colors hover:bg-accent/50"
					onclick={(e) => handleVideoClick(item, e)}
				>
					<CardContent class="p-4">
						<div class="flex gap-4">
							{#if item.cover}
								<CachedImage
									src={item.cover}
									alt={item.title}
									class="h-28 w-20 flex-shrink-0 rounded-md object-cover"
								/>
							{/if}
							<div class="min-w-0 flex-grow">
								<div class="flex items-start justify-between gap-2">
									<div class="min-w-0">
										<h3 class="line-clamp-1 font-semibold">{item.title}</h3>
										<div class="mt-1 flex items-center gap-2">
											<Badge variant="outline" class="text-xs">
												{item.source}
											</Badge>
											{#if item.episode}
												<Badge variant="secondary" class="text-xs">
													{item.episode}
												</Badge>
											{/if}
										</div>
									</div>
									<Button
										variant="ghost"
										size="icon"
										onclick={(e) => {
											e.stopPropagation();
											handleRemove(item);
										}}
									>
										<Heart class="h-4 w-4 fill-primary text-primary" />
									</Button>
								</div>

								<div class="mt-3 flex items-center justify-between">
									<span class="text-xs text-muted-foreground">
										添加于 {formatRelativeTime(item.addedAt)}
									</span>
								</div>
							</div>
						</div>
					</CardContent>
				</Card>
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
