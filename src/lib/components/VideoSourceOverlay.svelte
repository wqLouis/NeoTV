<script lang="ts">
	import { goto } from '$app/navigation';
	import { search, type SearchResult } from '$lib/api/search';
	import type { DoubanSubject } from '$lib/api/douban';
	import { settingsStore } from '$lib/stores/settings.svelte';
	import { favouritesStore } from '$lib/stores/favourites.svelte';
	import { testSourceSpeed } from '$lib/utils/speedTest';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import CachedImage from '$lib/components/CachedImage.svelte';
	import { Play, X, Calendar, Film, Info, Users, Heart, AlertCircle } from 'lucide-svelte';
	import { fly, fade, scale } from 'svelte/transition';
	import { toast } from 'svelte-sonner';
	import { onMount } from 'svelte';

	interface Props {
		item: DoubanSubject | null;
		open: boolean;
		originRect?: DOMRect | null;
		onOpenChange: (open: boolean) => void;
	}

	interface AvailableSource {
		source_code: string;
		vod_id: string;
		vod_name: string;
		source_name: string;
		vod_pic?: string;
	}

	let { item, open = $bindable(false), originRect = null, onOpenChange }: Props = $props();

	let loading = $state(false);
	let testingSources = $state(false);
	let searchResults = $state<SearchResult[]>([]);
	let contentVisible = $state(false);
	let selectedDescription = $state('');
	let autoSelectedResult = $state<SearchResult | null>(null);
	let allSourcesFailed = $state(false);
	let overlayDidOpen = $state(false);

	function handlePopState() {
		if (open && overlayDidOpen) {
			handleClose();
		}
	}

	onMount(() => {
		window.addEventListener('popstate', handlePopState);
		return () => {
			window.removeEventListener('popstate', handlePopState);
		};
	});

	async function searchSources(query: string) {
		loading = true;
		searchResults = [];
		selectedDescription = '';
		autoSelectedResult = null;
		allSourcesFailed = false;
		try {
			const results = await search(
				query,
				settingsStore.selectedApis,
				settingsStore.customApis,
				settingsStore.yellowFilterEnabled,
				settingsStore.commentaryFilterEnabled
			);
			searchResults = results;
			const fullResult = results.find(
				(r) => r.vod_content?.trim() && r.vod_content.length > 50 && !r.vod_content.includes('简介')
			);
			if (fullResult?.vod_content) {
				selectedDescription = fullResult.vod_content
					.replace(/<[^>]+>/g, '')
					.replace(/&[^;]+;/g, ' ')
					.trim();
			}

			if (settingsStore.autoIntegrateSources && results.length > 0) {
				testingSources = true;
				const testPromises = results.map((r) =>
					testSourceSpeed(r.source_code).then((speedResult) => ({ speedResult, searchResult: r }))
				);
				const firstPlayable = await Promise.race(testPromises);
				if (firstPlayable?.speedResult.status === 'success') {
					autoSelectedResult = firstPlayable.searchResult;
				}
				const allResults = await Promise.all(testPromises);
				const anySuccess = allResults.some((r) => r.speedResult.status === 'success');
				if (!anySuccess) {
					allSourcesFailed = true;
				}
				testingSources = false;
			}
		} catch (e) {
			console.error('Search failed:', e);
		} finally {
			loading = false;
		}
	}

	function handlePlay(result: SearchResult) {
		const selectedResult = autoSelectedResult || result;
		const availableSources: AvailableSource[] = searchResults.map((r) => ({
			source_code: r.source_code,
			vod_id: r.vod_id,
			vod_name: r.vod_name,
			source_name: r.source_name,
			vod_pic: r.vod_pic
		}));
		sessionStorage.setItem('availableSources', JSON.stringify(availableSources));
		onOpenChange(false);
		goto(
			`/player?id=${selectedResult.vod_id}&source=${selectedResult.source_code}&title=${encodeURIComponent(selectedResult.vod_name)}`
		);
	}

	function handleClose() {
		contentVisible = false;
		autoSelectedResult = null;
		allSourcesFailed = false;
		onOpenChange(false);
	}

	function handleAddToFavorites() {
		if (!item) return;
		if (favouritesStore.has(item.id, 'douban')) {
			favouritesStore.remove(item.id, 'douban');
			toast.success('已取消收藏');
		} else {
			favouritesStore.add({
				id: item.id,
				title: item.title,
				cover: item.cover_url || item.cover,
				source: 'douban'
			});
			toast.success('已添加到收藏');
		}
	}

	$effect(() => {
		if (item && open) {
			searchSources(item.title);
			setTimeout(() => {
				contentVisible = true;
			}, 50);
			document.body.style.overflow = 'hidden';
			overlayDidOpen = true;
			history.pushState(null, '', location.href);
		} else {
			contentVisible = false;
			document.body.style.overflow = '';
			if (overlayDidOpen) {
				overlayDidOpen = false;
				history.back();
			}
		}
	});

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
			{#if contentVisible && item}
				<div class="flex h-full w-full">
					<div class="flex w-1/2 flex-col overflow-hidden border-r">
						<div class="flex items-center justify-between border-b p-4">
							<span class="text-sm text-muted-foreground">详情</span>
							<div class="flex items-center gap-2">
								<Button
									variant="ghost"
									size="icon"
									onclick={handleAddToFavorites}
									title={favouritesStore.has(item.id, 'douban') ? '取消收藏' : '添加到收藏'}
								>
									<Heart
										class="h-5 w-5 {favouritesStore.has(item.id, 'douban')
											? 'fill-primary text-primary'
											: ''}"
									/>
								</Button>
								<Button variant="ghost" size="icon" onclick={handleClose}>
									<X class="h-5 w-5" />
								</Button>
							</div>
						</div>
						<div class="flex-1 overflow-y-auto p-6">
							<div class="flex flex-col items-center gap-4">
								<CachedImage
									src={item.cover_url || item.cover}
									alt={item.title}
									class="aspect-2/3 w-1/2 rounded-lg object-cover shadow-lg"
									referer="https://movie.douban.com/"
								/>
								<div class="flex w-full flex-col gap-3">
									<h2 class="text-center text-xl font-bold">{item.title}</h2>
									<div class="flex flex-wrap items-center justify-center gap-2">
										{#if item.score || item.rate}
											<Badge variant="default" class="bg-yellow-500 px-3 py-1 text-lg text-black">
												{item.score || item.rate}
											</Badge>
										{/if}
										{#if item.types?.length}
											{#each item.types.slice(0, 3) as type}
												<Badge variant="secondary">{type}</Badge>
											{/each}
										{/if}
										{#if item.regions?.[0]}
											<Badge variant="outline">{item.regions[0]}</Badge>
										{/if}
									</div>
								</div>
							</div>

							<div class="mt-6 flex flex-col gap-2">
								{#if item.director?.length}
									<div class="flex items-start gap-2 text-sm">
										<span class="flex items-center gap-1 text-muted-foreground">
											<Users class="h-4 w-4" />
											导演
										</span>
										<span class="truncate">{item.director.join(', ')}</span>
									</div>
								{/if}
								{#if item.actors?.length}
									<div class="flex items-start gap-2 text-sm">
										<span class="flex items-center gap-1 text-muted-foreground">
											<Users class="h-4 w-4" />
											演员
										</span>
										<span class="truncate">
											{item.actors.slice(0, 5).join(', ')}{item.actors.length > 5 ? '...' : ''}
										</span>
									</div>
								{/if}
							</div>

							{#if selectedDescription}
								<div class="mt-4 flex items-start gap-2 rounded-lg bg-muted/50 p-3 text-sm">
									<Info class="mt-0.5 h-4 w-4 flex-shrink-0 text-muted-foreground" />
									<span class="text-muted-foreground">{selectedDescription}</span>
								</div>
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
									{#each Array(5) as _}
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
							{:else if allSourcesFailed}
								<div class="flex flex-col items-center justify-center py-12 text-destructive">
									<AlertCircle class="mb-2 h-12 w-12 opacity-50" />
									<p>所有源均无法播放</p>
									<p class="text-sm text-muted-foreground">请检查网络或稍后重试</p>
								</div>
							{:else}
								<div class="space-y-2 pr-4">
									{#each searchResults as result (result.vod_id)}
										{@const isAutoSelected = autoSelectedResult?.vod_id === result.vod_id}
										<div
											class="flex cursor-pointer items-center gap-3 rounded-lg border p-3 transition-all focus-within:ring-2 focus-within:ring-ring hover:bg-muted/50
												{isAutoSelected ? 'border-primary bg-primary/5' : ''}"
											role="button"
											tabindex="0"
											onclick={() => handlePlay(result)}
											onkeydown={(e) => e.key === 'Enter' && handlePlay(result)}
										>
											{#if result.vod_pic}
												<CachedImage
													src={result.vod_pic}
													alt={result.vod_name}
													class="h-16 w-12 rounded object-cover"
												/>
											{/if}
											<div class="min-w-0 flex-grow">
												<p class="truncate font-medium">
													{result.vod_name}
													{#if isAutoSelected}
														<span class="ml-2 text-xs text-primary">(已自动选择)</span>
													{/if}
												</p>
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
												variant={allSourcesFailed ? 'destructive' : 'default'}
												onclick={(e) => {
													e.stopPropagation();
													handlePlay(result);
												}}
											>
												<Play class="mr-1 h-4 w-4" />
												{allSourcesFailed ? '播放失败' : '播放'}
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
