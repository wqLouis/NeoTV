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
	import { onMount, tick } from 'svelte';
	import VideoSourceOverlay from '$lib/components/VideoSourceOverlay.svelte';
	import DoubanCard from '$lib/components/DoubanCard.svelte';
	import { page } from '$app/state';
	import PageTabBar from '$lib/components/business/PageTabBar.svelte';
	import EmptyState from '$lib/components/business/EmptyState.svelte';
	import { Film, Tv } from '@lucide/svelte';

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

	// ── TV navigation state ──────────────────────────────────────────
	let focusedIndex = $state(0);
	// Track if we're currently scrolling to prevent focus conflicts
	let isScrolling = $state(false);

	const PAGE_SIZE = 20;
	const TV_TAGS_FALLBACK = ['热门', '美剧', '英剧', '韩剧', '日剧', '国产剧'];

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

	// Larger buffer in TV nav mode so off-screen targets are still in the DOM
	const visibleRange = $derived.by(() => {
		if (!scrollContainer || containerHeight === 0 || charts.length === 0) {
			return { start: 0, end: Math.min(charts.length, columns * 4) };
		}

		const bufferRows = settingsStore.tvNavModeEnabled ? 6 : 2;
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

	const currentTags = $derived(doubanSwitch === 'movie' ? movieTags : tvTags);

	const typeOptions = [
		{ value: 'movie', label: '电影', icon: Film },
		{ value: 'tv', label: '电视剧', icon: Tv }
	];

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
			tick().then(() => {
				setupObserver();
				if (settingsStore.tvNavModeEnabled && reset && charts.length > 0) {
					focusCard(0);
				}
			});
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

	// ── TV / keyboard grid navigation ────────────────────────────────
	function handleContainerKeydown(e: KeyboardEvent) {
		if (!settingsStore.tvNavModeEnabled) return;
		if (charts.length === 0) return;

		const dir = e.key;
		if (dir !== 'ArrowUp' && dir !== 'ArrowDown' && dir !== 'ArrowLeft' && dir !== 'ArrowRight')
			return;

		// Don't interfere when the user is focused on an input/button inside the container
		const tag = (e.target as HTMLElement).tagName;
		if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT' || tag === 'BUTTON') return;

		e.preventDefault();
		e.stopPropagation();

		// Clamp focusedIndex to valid range
		if (focusedIndex >= charts.length) focusedIndex = 0;

		let targetIndex = focusedIndex;

		switch (dir) {
			case 'ArrowRight':
				if (targetIndex % columns < columns - 1 && targetIndex < charts.length - 1) {
					targetIndex++;
				}
				break;
			case 'ArrowLeft':
				if (targetIndex % columns > 0 && targetIndex > 0) {
					targetIndex--;
				}
				break;
			case 'ArrowDown':
				targetIndex = Math.min(targetIndex + columns, charts.length - 1);
				break;
			case 'ArrowUp':
				if (targetIndex >= columns) {
					targetIndex -= columns;
				}
				break;
		}

		if (targetIndex === focusedIndex) return;

		focusedIndex = targetIndex;
		void scrollToAndFocus(targetIndex);
	}

	function focusCard(index: number) {
		focusedIndex = index;
		// Find the actual DOM element for this index in the current visible items
		const el = document.querySelector(`[data-card-index="${index}"]`) as HTMLElement | null;
		if (el) {
			el.focus({ preventScroll: true });
		}
	}

	async function scrollToAndFocus(index: number) {
		const row = Math.floor(index / columns);
		const targetTop = row * rowHeight;
		const targetBottom = targetTop + cardHeight;

		const viewTop = scrollTop;
		const viewBottom = scrollTop + containerHeight;

		const needsScroll = targetTop < viewTop || targetBottom > viewBottom;
		let scrollTarget = 0;

		if (targetTop < viewTop) {
			scrollTarget = targetTop - rowHeight;
		} else if (targetBottom > viewBottom) {
			scrollTarget = targetBottom - containerHeight + rowHeight;
		}

		if (needsScroll) {
			isScrolling = true;
			scrollContainer?.scrollTo({ top: scrollTarget, behavior: 'smooth' });
			// Wait for scroll + re-render (increase timeout for smoother experience)
			await tick();
			await new Promise((r) => setTimeout(r, 400));
			isScrolling = false;
		}

		// Find the element by data attribute (more reliable than array index)
		let el = document.querySelector(`[data-card-index="${index}"]`) as HTMLElement | null;
		
		// Retry a few times if element not found (DOM might need more time to update)
		if (!el) {
			for (let retry = 0; retry < 3; retry++) {
				await new Promise((r) => setTimeout(r, 100));
				el = document.querySelector(`[data-card-index="${index}"]`) as HTMLElement | null;
				if (el) break;
			}
		}
		
		if (el) {
			el.focus({ preventScroll: true });
		}
	}

	function handleCardFocus(thisIndex: number) {
		focusedIndex = thisIndex;
	}

	function handleCardActivate(item: DoubanSubject, e: KeyboardEvent) {
		if (e.key === 'Enter' || e.key === ' ') {
			e.preventDefault();
			e.stopPropagation();
			handleVideoClick(item, e as unknown as MouseEvent);
		}
	}
</script>

<svelte:window
	onresize={() => {
		if (scrollContainer) {
			containerWidth = scrollContainer.clientWidth;
			containerHeight = scrollContainer.clientHeight;
		}
	}}
/>

<div class="flex h-full flex-col">
	<PageTabBar options={typeOptions} value={doubanSwitch} onchange={handleTabChange}>
		{#snippet secondary()}
			<div class="scrollbar-hide flex gap-2 overflow-x-auto">
				<span class="mr-2 self-center text-sm whitespace-nowrap text-muted-foreground">类型:</span>
				{#each currentTags as tag}
					<button
						class="rounded-lg px-3 py-1.5 text-sm whitespace-nowrap transition-colors
							{selectedGenre === tag
							? 'bg-primary text-primary-foreground'
							: 'bg-secondary text-muted-foreground hover:bg-secondary/80 hover:text-foreground'}"
						onclick={() => handleGenreChange(tag)}
					>
						{tag}
					</button>
				{/each}
			</div>
		{/snippet}
	</PageTabBar>

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
				onkeydown={handleContainerKeydown}
				class="relative w-full flex-1 overflow-y-auto py-12"
				role="grid"
				tabindex="0"
			>
				<div class="isolation-isolate relative min-h-full w-full px-8 pt-4">
					{#each visibleItems as { item, top, left, width, height, index } (item.id)}
						<div
							class="absolute z-0 overflow-hidden rounded-lg"
							class:focused={index === focusedIndex}
							data-card-index={index}
							style="top: {top}px; left: {left}px; width: {width}px; height: {height}px;"
							role="button"
							tabindex="-1"
							onclick={(e) => handleVideoClick(item, e)}
							onkeydown={(e) => handleCardActivate(item, e)}
							onfocus={() => { if (!isScrolling) focusedIndex = index; }}
						>
							<DoubanCard {item} fluid={true} onclick={handleVideoClick} />
						</div>
					{/each}
				</div>

				<div
					bind:this={loadMoreTrigger}
					class="absolute right-0 left-0 z-10 py-4 text-center"
					style="top: {totalHeight + 48}px"
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
				class="pointer-events-none absolute inset-x-0 bottom-0 h-16 bg-linear-to-t from-background to-transparent"
			></div>
		</div>
	{:else}
		<EmptyState message="暂无数据" />
	{/if}
</div>

<VideoSourceOverlay
	item={selectedVideo}
	originRect={selectedCardRect}
	open={showSourceOverlay}
	onOpenChange={(open) => (showSourceOverlay = open)}
/>

<style>
	.focused {
		outline: 2px solid var(--color-primary);
		outline-offset: 2px;
	}
</style>
