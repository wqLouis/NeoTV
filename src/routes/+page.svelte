<script lang="ts">
	import { goto } from '$app/navigation';
	import {
		fetchDoubanChart,
		fetchDoubanTVByTag,
		fetchDoubanTags,
		getCachedImageUrl
	} from '$lib/api/douban';
	import { DOUBAN_CHART_GENRE_IDS } from '$lib/api/constants';
	import { settingsStore, GRID_DENSITY_CLASSES } from '$lib/stores/settings.svelte';
	import { Badge } from '$lib/components/ui/badge';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import { Tabs, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
	import { onMount, onDestroy, tick } from 'svelte';
	import VideoSourceOverlay from '$lib/components/VideoSourceOverlay.svelte';

	let movieTags = $state<string[]>([]);
	let tvTags = $state<string[]>([]);
	let selectedGenre = $state('剧情');
	let doubanSwitch = $state<'movie' | 'tv'>('movie');
	let loading = $state(false);
	let loadingMore = $state(false);
	let charts = $state<
		Array<{
			id: string;
			title: string;
			cover: string;
			cover_url?: string;
			rate: string;
			score?: string;
			region?: string[];
			regions?: string[];
			types?: string[];
			director?: string[];
			actors?: string[];
		}>
	>([]);
	let pageStart = $state(0);
	const PAGE_SIZE = 20;
	let loadMoreTrigger: HTMLDivElement | null = $state(null);
	let initialized = $state(false);
	let selectedVideo: (typeof charts)[0] | null = $state(null);
	let showSourceOverlay = $state(false);
	let selectedCardRect: DOMRect | null = $state(null);
	let cardRefs: Map<string, HTMLDivElement> = $state(new Map());
	let observer: IntersectionObserver | null = null;
	let imageUrlCache: Map<string, string> = $state(new Map());

	const genres = Object.keys(DOUBAN_CHART_GENRE_IDS);
	const movieGenres = genres;
	const tvGenres = [
		'剧情',
		'喜剧',
		'动作',
		'爱情',
		'科幻',
		'动画',
		'悬疑',
		'惊悚',
		'恐怖',
		'古装',
		'家庭',
		'犯罪'
	];

	async function loadTags() {
		const genreKeys = Object.keys(DOUBAN_CHART_GENRE_IDS);
		movieTags = genreKeys;
		try {
			const tvTagsFromApi = await fetchDoubanTags('tv');
			tvTags =
				tvTagsFromApi.length > 0
					? tvTagsFromApi
					: ['热门', '美剧', '英剧', '韩剧', '日剧', '国产剧'];
		} catch {
			tvTags = ['热门', '美剧', '英剧', '韩剧', '日剧', '国产剧'];
		}
	}

	async function loadCharts(reset = true) {
		if (reset) {
			loading = true;
			pageStart = 0;
		} else {
			loadingMore = true;
		}

		try {
			let data;
			if (doubanSwitch === 'tv') {
				data = await fetchDoubanTVByTag(selectedGenre, {
					page_start: pageStart,
					page_limit: PAGE_SIZE
				});
			} else {
				data = await fetchDoubanChart(selectedGenre, {
					start: pageStart,
					limit: PAGE_SIZE
				});
			}

			if (reset) {
				charts = data;
			} else {
				charts = [...charts, ...data];
			}
			pageStart += data.length;
		} catch (e) {
			console.error('Failed to load charts:', e);
		} finally {
			loading = false;
			loadingMore = false;
			tick().then(() => {
				setupObserver();
			});
		}
	}

	function handleVideoClick(item: (typeof charts)[0], event: MouseEvent | KeyboardEvent) {
		const target = event.currentTarget as HTMLDivElement;
		selectedCardRect = target.getBoundingClientRect();
		selectedVideo = item;
		showSourceOverlay = true;
	}

	function handleGenreChange(tag: string) {
		selectedGenre = tag;
		loadCharts(true);
	}

	function handleTabChange(value: string) {
		const newSwitch = value as 'movie' | 'tv';
		if (newSwitch === 'tv' && !tvTags.includes(selectedGenre)) {
			selectedGenre = tvTags[0] || '热门';
		}
		doubanSwitch = newSwitch;
		loadCharts(true);
	}

	function setupObserver() {
		if (observer) {
			observer.disconnect();
		}
		observer = new IntersectionObserver(
			(entries) => {
				if (entries[0].isIntersecting && !loading && !loadingMore) {
					loadCharts(false);
				}
			},
			{ rootMargin: '0px', threshold: 0 }
		);
		if (loadMoreTrigger) {
			observer.observe(loadMoreTrigger);
		}
	}

	onMount(() => {
		loadTags();
		loadCharts(true);
		initialized = true;
		tick().then(() => {
			setupObserver();
		});
	});

	onDestroy(() => {
		if (observer) {
			observer.disconnect();
		}
	});

	function getCoverUrl(item: (typeof charts)[0]): string {
		const url = item.cover_url || item.cover || '';
		if (!url) return '';
		if (url.startsWith('data:') || url.startsWith('blob:')) return url;
		if (url.startsWith('/')) return url;
		return `/api/proxy?url=${encodeURIComponent(url)}`;
	}
</script>

<div class="mx-auto py-4">
	<div
		class="sticky top-0 z-30 mb-4 w-full border-b bg-background/90 px-4 pt-2 pb-4 shadow-[0_12px_12px] shadow-background/20 backdrop-blur-2xl"
	>
		<Tabs value={doubanSwitch} onValueChange={handleTabChange} class="mb-4">
			<TabsList>
				<TabsTrigger value="movie">电影</TabsTrigger>
				<TabsTrigger value="tv">电视剧</TabsTrigger>
			</TabsList>
		</Tabs>

		<div class="scrollbar-hide flex gap-2 overflow-x-auto pb-2">
			<span class="mr-2 self-center text-sm whitespace-nowrap text-muted-foreground">类型:</span>
			{#each doubanSwitch === 'movie' ? movieTags : tvTags as tag}
				<button
					class="rounded-full px-3 py-1.5 text-sm whitespace-nowrap transition-colors
						{selectedGenre === tag
						? 'bg-primary text-primary-foreground'
						: 'bg-secondary hover:bg-secondary/80'}"
					onclick={() => handleGenreChange(tag)}
				>
					{tag}
				</button>
			{/each}
		</div>
	</div>

	{#if loading}
		<div class="grid px-8 {GRID_DENSITY_CLASSES[settingsStore.gridDensity]} gap-4">
			{#each Array(settingsStore.gridDensity === 'compact' ? 30 : settingsStore.gridDensity === 'loose' ? 12 : 20) as _}
				<div class="space-y-2">
					<Skeleton class="aspect-2/3 w-full rounded-lg" />
					<Skeleton class="h-4 w-3/4" />
					<Skeleton class="h-3 w-1/2" />
				</div>
			{/each}
		</div>
	{:else}
		<div class="grid px-8 {GRID_DENSITY_CLASSES[settingsStore.gridDensity]} gap-4">
			{#each charts as item (item.id)}
				<div
					class="cursor-pointer overflow-hidden rounded-lg bg-card transition-all hover:scale-[1.02] hover:shadow-md focus-visible:scale-[1.02] focus-visible:shadow-lg focus-visible:ring-2 focus-visible:ring-ring"
					onclick={(e) => handleVideoClick(item, e)}
					role="button"
					tabindex="0"
					onkeydown={(e) => e.key === 'Enter' && handleVideoClick(item, e)}
				>
					<div class="relative aspect-2/3 w-full overflow-hidden">
						<img
							src={getCoverUrl(item)}
							alt={item.title}
							class="h-full w-full object-cover"
							loading="lazy"
							onerror={(e) => {
								const img = e.currentTarget as HTMLImageElement;
								img.src = 'https://via.placeholder.com/300x450?text=无封面';
								img.classList.add('object-contain');
							}}
						/>
						{#if item.score}
							<Badge class="absolute top-1.5 right-1.5 bg-yellow-500 text-xs text-black">
								{item.score}
							</Badge>
						{:else if item.rate}
							<Badge class="absolute top-1.5 right-1.5 bg-yellow-500 text-xs text-black">
								{item.rate}
							</Badge>
						{/if}
					</div>
					<div class="p-2">
						<h3 class="line-clamp-2 text-xs font-medium" title={item.title}>
							{item.title}
						</h3>
						{#if item.types && item.types.length > 0}
							<p class="text-xs text-muted-foreground">
								{item.types.slice(0, 2).join(' / ')}
							</p>
						{/if}
					</div>
				</div>
			{/each}
		</div>

		{#if charts.length === 0}
			<div class="py-12 text-center text-muted-foreground">
				<p>暂无数据</p>
			</div>
		{/if}

		<div bind:this={loadMoreTrigger} class="py-4 text-center">
			{#if loadingMore}
				<div class="flex justify-center gap-2">
					<div
						class="h-4 w-4 animate-spin rounded-full border-2 border-primary border-t-transparent"
					></div>
					<span class="text-sm text-muted-foreground">加载更多...</span>
				</div>
			{/if}
		</div>
	{/if}
</div>

<VideoSourceOverlay
	item={selectedVideo}
	originRect={selectedCardRect}
	bind:open={showSourceOverlay}
	onOpenChange={(open) => (showSourceOverlay = open)}
/>

<style>
	.scrollbar-hide::-webkit-scrollbar {
		display: none;
	}
	.scrollbar-hide {
		-ms-overflow-style: none;
		scrollbar-width: none;
	}
</style>
