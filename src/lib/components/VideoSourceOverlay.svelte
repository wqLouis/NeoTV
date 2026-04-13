<script lang="ts">
	import { goto, beforeNavigate } from '$app/navigation';
	import { invoke } from '@tauri-apps/api/core';
	import { search, type SearchResult } from '$lib/api/search';
	import type { DoubanSubject } from '$lib/api/douban';
	import { settingsStore } from '$lib/stores/settings.svelte';
	import { favouritesStore } from '$lib/stores/favourites.svelte';
	import {
		getSpeedCache,
		loadSpeedCacheFromDisk,
		saveSpeedCacheToDisk,
		type SpeedTestResult
	} from '$lib/utils/speedTest';
	import type { SearchGroup, ScoredSource } from '$lib/utils/ranking';
	import { groupByNameAndTags, sortGroupsByScore } from '$lib/utils/ranking';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import CachedImage from '$lib/components/CachedImage.svelte';
	import { Play, X, Film, Info, Users, Heart } from '@lucide/svelte';
	import { fly, fade, scale } from 'svelte/transition';
	import { cubicOut } from 'svelte/easing';
	import { toast } from 'svelte-sonner';

	let currentNetworkId = $state('default');

	async function getNetworkId(): Promise<string> {
		try {
			const networkId = await invoke<string>('get_network_id');
			return networkId;
		} catch {
			return 'default';
		}
	}

	async function loadCache() {
		currentNetworkId = await getNetworkId();
		await loadSpeedCacheFromDisk(currentNetworkId);
	}

	async function saveCache() {
		await saveSpeedCacheToDisk(currentNetworkId);
	}

	function scaleFly(node: Element, { delay = 0, duration = 300, y = 100, startScale = 0.5 }) {
		return {
			delay,
			duration,
			easing: cubicOut,
			css: (t: number) => {
				const scaleVal = startScale + (1 - startScale) * t;
				const yOffset = y * (1 - t);
				return `
					transform: scale(${scaleVal}) translateY(${yOffset}px);
					opacity: ${t};
				`;
			}
		};
	}

	interface Props {
		item: DoubanSubject | null;
		open: boolean;
		originRect?: DOMRect | null;
		onOpenChange: (open: boolean) => void;
		onPlay?: (params: {
			id: string;
			source: string;
			title: string;
			sources: AvailableSource[];
		}) => void;
	}

	interface AvailableSource {
		source_code: string;
		vod_id: string;
		vod_name: string;
		source_name: string;
		vod_pic?: string;
	}

	let { item, open = $bindable(false), originRect = null, onOpenChange, onPlay }: Props = $props();

	let loading = $state(false);
	let testingSources = $state(false);
	let groupedResults = $state<SearchGroup[]>([]);
	let speedCache = $state<Map<string, SpeedTestResult>>(new Map());
	let selectedDescription = $state('');

	let isFavorited = $derived(
		item ? favouritesStore.items.some((f) => f.id === item.id && f.source === 'douban') : false
	);

	$effect(() => {
		if (open && item) {
			loadCache();
			searchSources(item.title);
		} else if (!open) {
			saveCache();
			groupedResults = [];
			loading = false;
			testingSources = false;
		}
	});

	beforeNavigate((navigation) => {
		if (open) {
			navigation.cancel();
			handleClose();
		}
	});

	async function searchSources(query: string) {
		loading = true;
		groupedResults = [];
		selectedDescription = '';

		const isMovieSearch =
			item?.types?.some((t) => t.includes('电影') || t.includes('film')) ?? false;
		const searchTags = item?.types ?? [];

		try {
			const results = await search(
				query,
				settingsStore.selectedApis,
				settingsStore.customApis,
				settingsStore.yellowFilterEnabled,
				settingsStore.commentaryFilterEnabled
			);

			const fullResult = results.find(
				(r) => r.vod_content?.trim() && r.vod_content.length > 50 && !r.vod_content.includes('简介')
			);
			if (fullResult?.vod_content) {
				selectedDescription = fullResult.vod_content
					.replace(/<[^>]+>/g, '')
					.replace(/&[^;]+;/g, ' ')
					.trim();
			}

			const groups = groupByNameAndTags(results);
			groupedResults = sortGroupsByScore(groups, speedCache, query, isMovieSearch, searchTags);
		} catch (e) {
			console.error('Search failed:', e);
		} finally {
			loading = false;
			testingSources = false;
		}
	}

	function handlePlay(group: SearchGroup) {
		const bestSource = group.sources[0];
		const availableSources: AvailableSource[] = group.sources.map((s) => ({
			source_code: s.result.source_code,
			vod_id: s.result.vod_id,
			vod_name: s.result.vod_name,
			source_name: s.result.source_name,
			vod_pic: s.result.vod_pic
		}));

		sessionStorage.setItem('availableSources', JSON.stringify(availableSources));
		onOpenChange(false);

		if (onPlay) {
			onPlay({
				id: bestSource.result.vod_id,
				source: bestSource.result.source_code,
				title: group.name,
				sources: availableSources
			});
		} else {
			goto(
				`/player?id=${bestSource.result.vod_id}&source=${bestSource.result.source_code}&title=${encodeURIComponent(group.name)}`
			);
		}
	}

	function handleClose() {
		onOpenChange(false);
	}

	function handleToggleFavorite() {
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

	function getSourceStatusIcon(scored: ScoredSource): 'fast' | 'slow' | 'pending' {
		if (scored.speedMs === undefined) return 'pending';
		return scored.speedMs < 2000 ? 'fast' : 'slow';
	}

	function formatSpeed(ms: number | undefined): string {
		if (ms === undefined) return '-';
		return ms < 1000 ? `${ms}ms` : `${(ms / 1000).toFixed(1)}s`;
	}
</script>

{#if open}
	<div class="fixed inset-0 z-50 overflow-hidden">
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
			in:scaleFly={{ y: 100, duration: 300, startScale: 0.5 }}
			out:fly={{ y: 50, duration: 200 }}
		>
			{#if item}
				<div class="flex h-full w-full">
					<div class="flex w-1/2 flex-col overflow-hidden border-r">
						<div class="flex items-center justify-between border-b p-4">
							<span class="text-sm text-muted-foreground">详情</span>
							<div class="flex items-center gap-2">
								<Button
									variant="ghost"
									size="icon"
									onclick={handleToggleFavorite}
									title={isFavorited ? '取消收藏' : '添加到收藏'}
								>
									<Heart class="h-5 w-5 {isFavorited ? 'fill-primary text-primary' : ''}" />
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
							<span class="text-sm text-muted-foreground">
								{groupedResults.length} 个结果
							</span>
						</div>

						<div class="flex-1 overflow-y-auto">
							{#if loading}
								<div class="space-y-3 pr-4">
									{#each Array(5) as _, i (i)}
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
							{:else if groupedResults.length === 0}
								<div class="flex flex-col items-center justify-center py-12 text-muted-foreground">
									<Film class="mb-2 h-12 w-12 opacity-50" />
									<p>未找到播放源</p>
								</div>
							{:else}
								<div class="space-y-2 pr-4">
									{#each groupedResults as group (group.id)}
										<div
											class="rounded-lg border bg-card transition-all hover:bg-muted/30"
											role="button"
											tabindex="0"
											onclick={() => handlePlay(group)}
											onkeydown={(e) => e.key === 'Enter' && handlePlay(group)}
										>
											<div class="flex items-center gap-3 p-3">
												{#if group.cover}
													<CachedImage
														src={group.cover}
														alt={group.name}
														class="h-16 w-12 rounded object-cover"
													/>
												{/if}
												<div class="min-w-0 flex-grow">
													<div class="flex items-center gap-2">
														<p class="truncate font-medium">{group.name}</p>
														{#if group.year}
															<span class="text-xs text-muted-foreground">
																{group.year}
															</span>
														{/if}
													</div>
													<div
														class="flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-muted-foreground"
													>
														<Badge variant="outline" class="text-xs">
															{group.sources.length} 个源
														</Badge>
														{#if group.typeName}
															<span class="flex items-center gap-1">
																<Film class="h-3 w-3" />
																{group.typeName}
															</span>
														{/if}
													</div>
												</div>
												<div class="flex items-center gap-2">
													{#if group.sources[0].speedMs !== undefined}
														<span class="text-xs text-muted-foreground">
															{formatSpeed(group.sources[0].speedMs)}
														</span>
													{/if}
													<Button size="sm" variant="default">
														<Play class="mr-1 h-4 w-4" />
														播放
													</Button>
												</div>
											</div>
											{#if group.sources.length > 1}
												<div class="flex flex-wrap gap-1 border-t bg-muted/20 px-3 py-2">
													{#each group.sources.slice(1) as scored}
														{@const status = getSourceStatusIcon(scored)}
														<span
															class="rounded bg-muted px-2 py-0.5 text-xs text-muted-foreground"
														>
															{scored.result.source_name}
															{#if scored.speedMs !== undefined}
																<span
																	class="ml-1 {status === 'fast'
																		? 'text-green-500'
																		: status === 'slow'
																			? 'text-orange-500'
																			: ''}"
																>
																	{formatSpeed(scored.speedMs)}
																</span>
															{/if}
														</span>
													{/each}
												</div>
											{/if}
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

<style>
	@keyframes overlayIn {
		from {
			opacity: 0;
			transform: scale(0.5) translateY(100%);
		}
		to {
			opacity: 1;
			transform: scale(1) translateY(0);
		}
	}
</style>
