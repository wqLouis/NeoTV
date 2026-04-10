<script lang="ts">
	import { goto } from '$app/navigation';
	import { search, type SearchResult } from '$lib/api/search';
	import { settingsStore } from '$lib/stores/settings.svelte';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import type { DoubanSubject } from '$lib/api/douban';
	import { Play, ExternalLink, X, Calendar, Film, Info, Users } from 'lucide-svelte';
	import { fly, fade, scale } from 'svelte/transition';

	interface Props {
		item: DoubanSubject | null;
		open: boolean;
		originRect?: DOMRect | null;
		onOpenChange: (open: boolean) => void;
	}

	let { item, open = $bindable(false), originRect = null, onOpenChange }: Props = $props();

	let loading = $state(false);
	let searchResults = $state<SearchResult[]>([]);
	let contentVisible = $state(false);
	let selectedDescription = $state('');

	async function searchSources(query: string) {
		loading = true;
		searchResults = [];
		selectedDescription = '';
		try {
			const results = await search(
				query,
				settingsStore.selectedApis,
				settingsStore.customApis,
				settingsStore.yellowFilterEnabled
			);
			searchResults = results;
			const fullResult = results.find(
				(r) =>
					r.vod_content &&
					r.vod_content.trim() &&
					r.vod_content.length > 50 &&
					!r.vod_content.includes('简介')
			);
			if (fullResult?.vod_content) {
				selectedDescription = fullResult.vod_content
					.replace(/<[^>]+>/g, '')
					.replace(/&[^;]+;/g, ' ')
					.trim();
			}
		} catch (e) {
			console.error('Search failed:', e);
		} finally {
			loading = false;
		}
	}

	function handlePlay(result: SearchResult) {
		const params = new URLSearchParams({
			id: result.vod_id,
			source: result.source_code,
			title: result.vod_name
		});
		onOpenChange(false);
		goto(`/player?${params.toString()}`);
	}

	function handleClose() {
		contentVisible = false;
		onOpenChange(false);
	}

	$effect(() => {
		if (item && open) {
			searchSources(item.title);
			setTimeout(() => {
				contentVisible = true;
			}, 50);
			document.body.style.overflow = 'hidden';
		} else {
			contentVisible = false;
			document.body.style.overflow = '';
		}
	});

	function getCoverUrl(item: DoubanSubject): string {
		const url = item.cover_url || item.cover || '';
		if (!url) return '';
		if (url.startsWith('data:') || url.startsWith('blob:')) return url;
		if (url.startsWith('/')) return url;
		return `/api/proxy?url=${encodeURIComponent(url)}`;
	}

	function getProxyUrl(url: string): string {
		if (!url) return '';
		if (url.startsWith('data:') || url.startsWith('blob:')) return url;
		if (url.startsWith('/')) return url;
		return `/api/proxy?url=${encodeURIComponent(url)}`;
	}

	function getTransformOrigin() {
		if (!originRect) return 'center center';
		const { top, left, width, height } = originRect;
		return `${left + width / 2}px ${top + height / 2}px`;
	}
</script>

{#if open}
	<div
		class="fixed inset-0 z-50 overflow-hidden"
		style="transform-origin: {getTransformOrigin()};"
		transition:scale={{ duration: 300, start: 0.9, opacity: 0 }}
	>
		<div
			class="absolute inset-0 bg-background/80 backdrop-blur-sm"
			onclick={handleClose}
			role="button"
			tabindex="-1"
			onkeydown={(e) => e.key === 'Escape' && handleClose()}
			transition:fade={{ duration: 200 }}
		></div>

		<div
			class="relative z-10 flex h-screen w-full overflow-hidden bg-card shadow-2xl"
			transition:fly={{ duration: 300, y: 30, opacity: 0 }}
		>
			<button
				class="absolute top-4 left-4 z-20 rounded-full p-2 transition-colors hover:bg-muted"
				onclick={handleClose}
			>
				<X class="h-5 w-5" />
			</button>

			{#if contentVisible && item}
				<div class="flex h-full w-full">
					<div class="flex w-1/2 flex-col border-r p-6">
						<div class="flex flex-col items-center gap-4">
							<img
								src={getCoverUrl(item)}
								alt={item.title}
								class="aspect-2/3 w-1/2 rounded-lg object-cover shadow-lg"
								onerror={(e) => {
									const img = e.currentTarget as HTMLImageElement;
									img.src = 'https://via.placeholder.com/300x450?text=无封面';
									img.classList.add('object-contain');
								}}
							/>
							<div class="flex w-full flex-col gap-3">
								<h2 class="text-center text-xl font-bold">{item.title}</h2>
								<div class="flex flex-wrap items-center justify-center gap-2">
									{#if item.score || item.rate}
										<Badge variant="default" class="bg-yellow-500 px-3 py-1 text-lg text-black">
											{item.score || item.rate}
										</Badge>
									{/if}
									{#if item.types && item.types.length > 0}
										{#each item.types.slice(0, 3) as type, i}
											<Badge variant="secondary">{type}</Badge>
										{/each}
									{/if}
									{#if item.regions && item.regions.length > 0}
										<Badge variant="outline">{item.regions[0]}</Badge>
									{/if}
								</div>
							</div>
						</div>

						<div class="mt-6 flex flex-col gap-2">
							{#if item.director && item.director.length > 0}
								<div class="flex items-start gap-2 text-sm">
									<span class="flex items-center gap-1 text-muted-foreground">
										<Users class="h-4 w-4" />
										导演
									</span>
									<span class="truncate">{item.director.join(', ')}</span>
								</div>
							{/if}
							{#if item.actors && item.actors.length > 0}
								<div class="flex items-start gap-2 text-sm">
									<span class="flex items-center gap-1 text-muted-foreground">
										<Users class="h-4 w-4" />
										演员
									</span>
									<span class="truncate"
										>{item.actors.slice(0, 5).join(', ')}{item.actors.length > 5 ? '...' : ''}</span
									>
								</div>
							{/if}
						</div>

						{#if selectedDescription}
							<div class="mt-4 flex items-start gap-2 rounded-lg bg-muted/50 p-3 text-sm">
								<Info class="mt-0.5 h-4 w-4 flex-shrink-0 text-muted-foreground" />
								<span class="text-muted-foreground">{selectedDescription}</span>
							</div>
						{/if}

						<div class="mt-auto flex-shrink-0 pt-4">
							{#if item.url}
								<a
									href={item.url}
									target="_blank"
									rel="noopener noreferrer"
									class="inline-flex items-center gap-2 text-sm text-blue-500 hover:underline"
								>
									<ExternalLink class="h-4 w-4" />
									查看豆瓣详情
								</a>
							{/if}
						</div>
					</div>

					<div class="flex w-1/2 flex-col overflow-hidden p-6">
						<div class="mb-4 flex items-center justify-between">
							<h3 class="text-lg font-semibold">播放源</h3>
							<span class="text-sm text-muted-foreground">{searchResults.length} 个结果</span>
						</div>

						<div class="flex-1 overflow-y-auto">
							{#if loading}
								<div class="space-y-3 pr-4">
									{#each Array(5) as _, i}
										<div class="flex items-center gap-3 rounded-lg border p-3">
											<Skeleton class="h-16 w-12 rounded" />
											<div class="flex-grow space-y-2">
												<Skeleton class="h-4 w-3/4" />
												<Skeleton class="h-3 w-1/2" />
											</div>
											<Skeleton class="h-8 w-16 rounded" />
										</div>
									{/each}
								</div>
							{:else if searchResults.length === 0}
								<div class="flex flex-col items-center justify-center py-12 text-muted-foreground">
									<Film class="mb-2 h-12 w-12 opacity-50" />
									<p>未找到播放源</p>
								</div>
							{:else}
								<div class="space-y-2 pr-4">
									{#each searchResults as result (result.vod_id)}
										<div
											class="flex cursor-pointer items-center gap-3 rounded-lg border p-3 transition-all focus-within:ring-2 focus-within:ring-ring hover:bg-muted/50"
											role="button"
											tabindex="0"
											onclick={() => handlePlay(result)}
											onkeydown={(e) => e.key === 'Enter' && handlePlay(result)}
										>
											{#if result.vod_pic}
												<img
													src={getProxyUrl(result.vod_pic)}
													alt={result.vod_name}
													class="h-16 w-12 rounded object-cover"
													onerror={(e) => {
														const img = e.currentTarget as HTMLImageElement;
														img.src = 'https://via.placeholder.com/300x450?text=无封面';
														img.classList.add('object-contain');
													}}
												/>
											{/if}
											<div class="min-w-0 flex-grow">
												<p class="truncate font-medium">{result.vod_name}</p>
												<div
													class="flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-muted-foreground"
												>
													<Badge variant="outline" class="text-xs">{result.source_name}</Badge>
													{#if result.type_name}
														<span class="flex items-center gap-1">
															<Film class="h-3 w-3" />
															{result.type_name}
														</span>
													{/if}
													{#if result.vod_year}
														<span class="flex items-center gap-1">
															<Calendar class="h-3 w-3" />
															{result.vod_year}
														</span>
													{/if}
													{#if result.vod_remarks}
														<span class="text-primary">{result.vod_remarks}</span>
													{/if}
												</div>
											</div>
											<Button
												size="sm"
												onclick={(e) => {
													e.stopPropagation();
													handlePlay(result);
												}}
											>
												<Play class="mr-1 h-4 w-4" />
												播放
											</Button>
										</div>
									{/each}
								</div>
							{/if}
						</div>
					</div>
				</div>
			{/if}
		</div>
	</div>
{/if}
