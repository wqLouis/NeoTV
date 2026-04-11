<script lang="ts">
	import { searchDouban, type DoubanSubject } from '$lib/api/douban';
	import CachedImage from './CachedImage.svelte';
	import { Badge } from '$lib/components/ui/badge';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import VideoSourceOverlay from './VideoSourceOverlay.svelte';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { ArrowRight } from 'lucide-svelte';

	interface Props {
		title: string;
		type: 'movie' | 'tv';
		sort: 'T' | 'R';
		seeMoreLink?: string;
	}

	let { title, type, sort, seeMoreLink }: Props = $props();

	let items = $state<DoubanSubject[]>([]);
	let loading = $state(true);
	let selectedVideo: DoubanSubject | null = $state(null);
	let showOverlay = $state(false);
	let cardRect: DOMRect | null = $state(null);
	let scrollContainer: HTMLDivElement | null = $state(null);
	let showGradient = $state(true);

	const CARD_WIDTH = 160;
	const CARD_HEIGHT = 240;
	const GAP = 16;

	onMount(async () => {
		try {
			items = await searchDouban({ sort, type });
		} catch (e) {
			console.error('Failed to load section:', title, e);
		} finally {
			loading = false;
		}
	});

	function handleCardClick(item: DoubanSubject, e: MouseEvent) {
		cardRect = (e.currentTarget as HTMLElement).getBoundingClientRect();
		selectedVideo = item;
		showOverlay = true;
	}

	function handleSeeMore() {
		if (seeMoreLink) {
			goto(seeMoreLink);
		} else {
			goto(`/browse?type=${type}`);
		}
	}

	function handleScroll() {
		if (!scrollContainer) return;
		const { scrollLeft, scrollWidth, clientWidth } = scrollContainer;
		showGradient = scrollLeft < scrollWidth - clientWidth - 10;
	}
</script>

<div class="group/section relative">
	<div class="mb-3 flex items-center justify-between">
		<h2 class="text-lg font-semibold">{title}</h2>
		<button
			class="flex items-center gap-1 text-sm text-muted-foreground transition-colors hover:text-foreground"
			onclick={handleSeeMore}
		>
			<span>浏览全部</span>
			<ArrowRight class="h-4 w-4" />
		</button>
	</div>

	{#if loading}
		<div class="flex gap-4 overflow-hidden">
			{#each Array(8) as _}
				<Skeleton class="h-60 w-40 shrink-0 rounded-lg" />
			{/each}
		</div>
	{:else}
		<div class="relative">
			<div
				bind:this={scrollContainer}
				class="scrollbar-hide flex gap-4 overflow-x-auto pb-4"
				onscroll={handleScroll}
			>
				{#each items as item (item.id)}
					<div
						class="shrink-0 cursor-pointer overflow-hidden rounded-lg bg-card transition-all hover:scale-[1.02] hover:shadow-md"
						style="width: {CARD_WIDTH}px; height: {CARD_HEIGHT}px;"
						onclick={(e) => handleCardClick(item, e)}
						role="button"
						tabindex="0"
						onkeydown={(e) =>
							e.key === 'Enter' && handleCardClick(item, e as unknown as MouseEvent)}
					>
						<div class="relative h-full w-full overflow-hidden">
							<CachedImage
								src={item.cover_url || item.cover}
								alt={item.title}
								class="h-full w-full object-cover"
								referer="https://movie.douban.com/"
							/>
							{#if item.score}
								<Badge class="absolute top-1.5 right-1.5 bg-yellow-500 text-xs text-black">
									{item.score}
								</Badge>
							{/if}
						</div>
					</div>
				{/each}
			</div>

			<div
				class="pointer-events-none absolute top-0 right-0 h-full w-20 bg-gradient-to-l from-background to-transparent transition-opacity duration-300 {showGradient
					? 'opacity-100'
					: 'opacity-0'}"
			></div>
		</div>
	{/if}
</div>

<VideoSourceOverlay
	item={selectedVideo}
	originRect={cardRect}
	bind:open={showOverlay}
	onOpenChange={(open) => (showOverlay = open)}
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
