<script lang="ts">
	import {
		fetchDoubanChart,
		fetchDoubanTVByTag,
		fetchDoubanTags,
		type DoubanSubject
	} from '$lib/api/douban';
	import { DOUBAN_CHART_GENRE_IDS } from '$lib/api/constants';
	import { settingsStore, GRID_DENSITY_CLASSES } from '$lib/stores/settings.svelte';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import { Tabs, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
	import { onMount, tick } from 'svelte';
	import VideoSourceOverlay from '$lib/components/VideoSourceOverlay.svelte';
	import DoubanCard from '$lib/components/DoubanCard.svelte';
	import { page } from '$app/state';

	let movieTags = $state<string[]>([]);
	let tvTags = $state<string[]>([]);
	let selectedGenre = $state('剧情');
	let doubanSwitch = $state<'movie' | 'tv'>('movie');
	let loading = $state(false);
	let loadingMore = $state(false);
	let charts = $state<DoubanSubject[]>([]);
	let pageStart = $state(0);
	let loadMoreTrigger: HTMLDivElement | null = $state(null);
	let selectedVideo: DoubanSubject | null = $state(null);
	let showSourceOverlay = $state(false);
	let selectedCardRect: DOMRect | null = $state(null);
	let observer: IntersectionObserver | null = null;

	const PAGE_SIZE = 20;
	const TV_TAGS_FALLBACK = ['热门', '美剧', '英剧', '韩剧', '日剧', '国产剧'];

	onMount(() => {
		loadTags();
		loadCharts();
		tick().then(setupObserver);

		const typeParam = page.url.searchParams.get('type');
		if (typeParam === 'movie' || typeParam === 'tv') {
			doubanSwitch = typeParam;
		}
	});

	async function loadTags() {
		movieTags = Object.keys(DOUBAN_CHART_GENRE_IDS);
		try {
			const tags = await fetchDoubanTags('tv');
			tvTags = tags.length > 0 ? tags : TV_TAGS_FALLBACK;
		} catch {
			tvTags = TV_TAGS_FALLBACK;
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
			const data =
				doubanSwitch === 'tv'
					? await fetchDoubanTVByTag(selectedGenre, {
							page_start: pageStart,
							page_limit: PAGE_SIZE
						})
					: await fetchDoubanChart(selectedGenre, { start: pageStart, limit: PAGE_SIZE });

			charts = reset ? data : [...charts, ...data];
			pageStart += data.length;
		} catch (e) {
			console.error('Failed to load charts:', e);
		} finally {
			loading = false;
			loadingMore = false;
			tick().then(setupObserver);
		}
	}

	function handleVideoClick(item: DoubanSubject, event: MouseEvent) {
		selectedCardRect = (event.currentTarget as HTMLElement).getBoundingClientRect();
		selectedVideo = item;
		showSourceOverlay = true;
	}

	function handleGenreChange(tag: string) {
		selectedGenre = tag;
		loadCharts();
	}

	function handleTabChange(value: string) {
		const newSwitch = value as 'movie' | 'tv';
		if (newSwitch === 'tv' && !tvTags.includes(selectedGenre)) {
			selectedGenre = tvTags[0] || '热门';
		}
		doubanSwitch = newSwitch;
		loadCharts();
	}

	function setupObserver() {
		observer?.disconnect();
		if (!loadMoreTrigger) return;
		observer = new IntersectionObserver(
			(entries) => {
				if (entries[0].isIntersecting && !loading && !loadingMore) {
					loadCharts(false);
				}
			},
			{ rootMargin: '0px', threshold: 0 }
		);
		observer.observe(loadMoreTrigger);
	}
</script>

<div class="mx-auto py-4">
	<div
		class="sticky top-0 z-30 mb-4 border-b bg-background/90 px-4 pt-8 pb-4 shadow-[0_12px_12px] shadow-background/20 backdrop-blur-2xl"
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
				<DoubanCard {item} onclick={handleVideoClick} />
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
