<script lang="ts">
	import {
		fetchDoubanChart,
		fetchDoubanTVByTag,
		fetchDoubanTags,
		type DoubanSubject
	} from '$lib/api/douban';
	import { DOUBAN_CHART_GENRE_IDS } from '$lib/api/constants';
	import { settingsStore, GRID_DENSITY_CLASSES } from '$lib/stores/settings.svelte';
	import { Film, Tv } from '@lucide/svelte';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import { onMount, tick } from 'svelte';
	import VideoSourceOverlay from '$lib/components/VideoSourceOverlay.svelte';
	import DoubanCard from '$lib/components/DoubanCard.svelte';
	import { page } from '$app/state';
	import { createTvnavigation } from '$lib/utils/tv-navigation.svelte';

	const GRID_COLS: Record<string, number> = {
		compact: 8,
		standard: 6,
		loose: 5
	};

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
	let observer: IntersectionObserver | null = $state(null);

	let scrollContainer: HTMLDivElement | null = $state(null);
	let scrollTop = $state(0);
	let containerHeight = $state(600);
	let containerWidth = $state(800);

	const PAGE_SIZE = 20;
	const TV_TAGS_FALLBACK = ['热门', '美剧', '英剧', '韩剧', '日剧', '国产剧'];

	const tvNav = createTvnavigation();

	const columns = $derived(GRID_COLS[settingsStore.gridDensity] || 6);
	const gap = 16;
	const padding = 64;
	const itemWidth = $derived(
		containerWidth > padding ? (containerWidth - padding - gap * (columns - 1)) / columns : 160
	);
	const cardHeight = $derived(itemWidth / (2 / 3));
	const rowHeight = $derived(cardHeight + gap);
	const totalRows = $derived(Math.ceil(charts.length / columns));
	const totalHeight = $derived(totalRows * rowHeight);

	const rowWidth = $derived(columns * itemWidth + (columns - 1) * gap);

	const visibleRange = $derived.by(() => {
		if (!scrollContainer || containerHeight === 0 || charts.length === 0) {
			return { start: 0, end: Math.min(charts.length, columns * 4) };
		}

		const bufferRows = 2;
		const startRow = Math.max(0, Math.floor(scrollTop / rowHeight) - bufferRows);
		const endRow = Math.min(
			totalRows - 1,
			Math.floor((scrollTop + containerHeight) / rowHeight) + bufferRows
		);

		const start = startRow * columns;
		const end = Math.min(charts.length, (endRow + 1) * columns);

		return { start, end };
	});

	const visibleItems = $derived.by(() => {
		const result: {
			item: DoubanSubject;
			index: number;
			top: number;
			left: number;
			width: number;
			height: number;
		}[] = [];
		const { start, end } = visibleRange;

		const currentRowWidth = columns * itemWidth + (columns - 1) * gap;
		const currentRowStartOffset = (containerWidth - currentRowWidth) / 2;

		for (let i = start; i < end; i++) {
			const row = Math.floor(i / columns);
			const col = i % columns;
			result.push({
				item: charts[i],
				index: i,
				top: row * rowHeight,
				left: currentRowStartOffset + col * (itemWidth + gap),
				width: itemWidth,
				height: cardHeight
			});
		}

		return result;
	});

	$effect(() => {
		void settingsStore.gridDensity;
		if (scrollContainer) {
			containerWidth = scrollContainer.clientWidth;
			containerHeight = scrollContainer.clientHeight;
		}
	});

	onMount(() => {
		loadTags();
		loadCharts();
		tick().then(setupObserver);

		if (scrollContainer) {
			containerHeight = scrollContainer.clientHeight;
			containerWidth = scrollContainer.clientWidth;
		}

		const resizeObserver = new ResizeObserver((entries) => {
			for (const entry of entries) {
				containerWidth = entry.contentRect.width;
				containerHeight = entry.contentRect.height;
			}
		});

		if (scrollContainer) {
			resizeObserver.observe(scrollContainer);
		}

		const typeParam = page.url.searchParams.get('type');
		if (typeParam === 'movie' || typeParam === 'tv') {
			doubanSwitch = typeParam;
		}

		return () => resizeObserver.disconnect();
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
		if (!loadMoreTrigger || !scrollContainer) return;
		observer = new IntersectionObserver(
			(entries) => {
				if (entries[0].isIntersecting && !loading && !loadingMore) {
					loadCharts(false);
				}
			},
			{ root: scrollContainer, rootMargin: '0px', threshold: 0 }
		);
		observer.observe(loadMoreTrigger);
	}

	function handleScroll(e: Event) {
		const target = e.target as HTMLDivElement;
		scrollTop = target.scrollTop;
	}

	function updateContainerSize() {
		if (scrollContainer) {
			containerHeight = scrollContainer.clientHeight;
			containerWidth = scrollContainer.clientWidth;
		}
	}

	function initContainer(el: HTMLDivElement) {
		scrollContainer = el;
		updateContainerSize();
	}

	const currentTags = $derived(doubanSwitch === 'movie' ? movieTags : tvTags);

	function handleKeydown(e: KeyboardEvent) {
		tvNav.handleKeydown(
			e,
			{
				tabCount: 2,
				genreCount: currentTags.length,
				cardCount: charts.length
			},
			{
				onTabChange: (index) => handleTabChange(index === 0 ? 'movie' : 'tv'),
				onGenreChange: (index) => handleGenreChange(currentTags[index]),
				onCardClick: (index) => handleVideoClick(charts[index], {} as MouseEvent)
			}
		);
	}
</script>

<svelte:window
	onkeydown={handleKeydown}
	onresize={() => {
		if (scrollContainer) {
			containerWidth = scrollContainer.clientWidth;
			containerHeight = scrollContainer.clientHeight;
		}
	}}
/>

<div class="flex h-full flex-col">
	<div
		class="shrink-0 border-b bg-background/90 shadow-[0_12px_12px] shadow-background/70 backdrop-blur-2xl"
	>
		<div class="mx-4 mb-4 flex gap-4 pt-4">
			<button
				class="flex items-center gap-2 rounded-lg px-4 py-2 transition-colors
					{tvNav.state.focusedTabIndex === 0 && tvNav.state.focusRegion === 'tabs'
					? 'ring-2 ring-primary ring-offset-2 ring-offset-background'
					: ''}
					{doubanSwitch === 'movie'
					? 'bg-primary text-primary-foreground'
					: 'text-muted-foreground hover:bg-secondary hover:text-foreground'}"
				onclick={() => handleTabChange('movie')}
				tabindex={tvNav.state.focusRegion === 'tabs' ? 0 : -1}
			>
				<Film class="h-5 w-5" />
				<span class="font-medium">电影</span>
			</button>

			<button
				class="flex items-center gap-2 rounded-lg px-4 py-2 transition-colors
					{tvNav.state.focusedTabIndex === 1 && tvNav.state.focusRegion === 'tabs'
					? 'ring-2 ring-primary ring-offset-2 ring-offset-background'
					: ''}
					{doubanSwitch === 'tv'
					? 'bg-primary text-primary-foreground'
					: 'text-muted-foreground hover:bg-secondary hover:text-foreground'}"
				onclick={() => handleTabChange('tv')}
				tabindex={tvNav.state.focusRegion === 'tabs' ? 0 : -1}
			>
				<Tv class="h-5 w-5" />
				<span class="font-medium">电视剧</span>
			</button>
		</div>

		<div class="">
			<hr class="border-border" />
		</div>

		<div class="scrollbar-hide m-4 flex gap-2 overflow-x-auto">
			<span class="mr-2 self-center text-sm whitespace-nowrap text-muted-foreground">类型:</span>
			{#each currentTags as tag, i}
				<button
					class="rounded-lg px-3 py-1.5 text-sm whitespace-nowrap transition-colors
						{tvNav.state.focusedGenreIndex === i && tvNav.state.focusRegion === 'genres'
						? 'ring-2 ring-primary ring-offset-2 ring-offset-background'
						: ''}
						{selectedGenre === tag
						? 'bg-primary text-primary-foreground'
						: 'bg-secondary text-muted-foreground hover:bg-secondary/80 hover:text-foreground'}"
					onclick={() => handleGenreChange(tag)}
					tabindex={tvNav.state.focusRegion === 'genres' ? 0 : -1}
				>
					{tag}
				</button>
			{/each}
		</div>
	</div>

	{#if loading}
		<div class="grid px-8 pt-8 {GRID_DENSITY_CLASSES[settingsStore.gridDensity]} gap-4">
			{#each Array(settingsStore.gridDensity === 'compact' ? 30 : settingsStore.gridDensity === 'loose' ? 12 : 20) as _, i (i)}
				<div class="space-y-2">
					<Skeleton class="aspect-2/3 w-full rounded-lg" />
					<Skeleton class="h-4 w-3/4" />
					<Skeleton class="h-3 w-1/2" />
				</div>
			{/each}
		</div>
	{:else if charts.length > 0}
		<div class="flex flex-1 flex-col">
			<div
				bind:this={scrollContainer}
				onscroll={handleScroll}
				class="relative w-full flex-1 overflow-y-auto py-12"
			>
				<div class="isolation-isolate relative min-h-full w-full px-8 pt-4">
					{#each visibleItems as { item, index, top, left, width, height } (item.id)}
						<div
							class="absolute z-0 overflow-hidden rounded-lg"
							style="top: {top}px; left: {left}px; width: {width}px; height: {height}px;"
						>
							<DoubanCard
								{item}
								fluid={true}
								onclick={handleVideoClick}
								focused={tvNav.state.focusRegion === 'grid' &&
									tvNav.state.focusedCardIndex === index}
							/>
						</div>
					{/each}
				</div>

				<div
					bind:this={loadMoreTrigger}
					class="absolute right-0 left-0 py-4 text-center"
					style="top: {totalHeight + 80}px"
				>
					{#if loadingMore}
						<div class="flex justify-center gap-2">
							<div
								class="h-4 w-4 animate-spin rounded-full border-2 border-primary border-t-transparent"
							></div>
							<span class="text-sm text-muted-foreground">加载更多...</span>
						</div>
					{/if}
				</div>
			</div>

			<div
				class="pointer-events-none absolute inset-x-0 bottom-0 h-32 bg-linear-to-t from-background to-transparent"
			></div>

			{#if charts.length === 0 && !loading}
				<div class="py-12 text-center text-muted-foreground">
					<p>暂无数据</p>
				</div>
			{/if}
		</div>
	{:else}
		<div class="py-12 text-center text-muted-foreground">
			<p>暂无数据</p>
		</div>
	{/if}
</div>

<VideoSourceOverlay
	item={selectedVideo}
	originRect={selectedCardRect}
	open={showSourceOverlay}
	onOpenChange={(open) => (showSourceOverlay = open)}
/>
