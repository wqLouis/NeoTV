<script lang="ts">
	import { goto } from '$app/navigation';
	import { settingsStore } from '$lib/stores/settings.svelte';
	import { historyStore, type HistoryItem } from '$lib/stores/history.svelte';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Card, CardContent } from '$lib/components/ui/card';
	import CachedImage from '$lib/components/CachedImage.svelte';
	import { Play, Trash2, Clock } from 'lucide-svelte';
	import { formatDuration, formatRelativeTime } from '$lib/utils/format';

	function getProgressPercent(item: HistoryItem): number {
		if (!item.duration) return 0;
		return Math.min(100, Math.round((item.position / item.duration) * 100));
	}

	function handlePlay(item: HistoryItem) {
		const params = new URLSearchParams({
			id: item.id,
			source: item.source,
			title: item.title,
			...(item.cover && { cover: item.cover }),
			...(item.episode && { episode: item.episode }),
			...(item.episodeIndex !== undefined && { episodeIndex: item.episodeIndex.toString() }),
			position: item.position.toString()
		});
		goto(`/player?${params.toString()}`);
	}

	function handleRemove(item: HistoryItem) {
		historyStore.remove(item.id, item.source, item.episode);
	}

	function handleClearAll() {
		historyStore.clear();
	}
</script>

<div class="container mx-auto px-4 py-6">
	<div class="mb-6 flex items-center justify-between">
		<h1 class="text-2xl font-bold">历史记录</h1>
		{#if historyStore.items.length > 0}
			<Button variant="outline" size="sm" onclick={handleClearAll}>
				<Trash2 class="mr-1 h-4 w-4" />
				清空全部
			</Button>
		{/if}
	</div>

	{#if historyStore.items.length === 0}
		<div class="py-12 text-center text-muted-foreground">
			<Clock class="mx-auto mb-4 h-12 w-12 opacity-50" />
			<p>暂无观看历史</p>
		</div>
	{:else}
		<div class="space-y-3">
			{#each historyStore.items as item (item.id + item.source + item.episode)}
				<Card class="transition-colors hover:bg-accent/50">
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
									<Button variant="ghost" size="icon" onclick={() => handleRemove(item)}>
										<Trash2 class="h-4 w-4" />
									</Button>
								</div>

								<div class="mt-3 flex items-center gap-4">
									<div class="flex-grow">
										<div class="h-1.5 overflow-hidden rounded-full bg-secondary">
											<div
												class="h-full bg-primary transition-all"
												style="width: {getProgressPercent(item)}%"
											></div>
										</div>
										<div class="mt-1 flex justify-between">
											<span class="text-xs text-muted-foreground">
												{formatDuration(item.position)} / {formatDuration(item.duration)}
											</span>
											<span class="text-xs text-muted-foreground">
												{formatRelativeTime(item.timestamp)}
											</span>
										</div>
									</div>
									<Button size="sm" onclick={() => handlePlay(item)}>
										<Play class="mr-1 h-4 w-4" />
										继续
									</Button>
								</div>
							</div>
						</div>
					</CardContent>
				</Card>
			{/each}
		</div>
	{/if}
</div>
