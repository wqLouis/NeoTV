<script lang="ts">
	import { searchSubjects, type DoubanSubject } from '$lib/api/douban';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import VideoSourceOverlay from './VideoSourceOverlay.svelte';
	import DoubanCard from './DoubanCard.svelte';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { ArrowRight } from 'lucide-svelte';

	interface Props {
		title: string;
		type: 'movie' | 'tv';
		tag: string;
		sort: 'recommend' | 'time' | 'rank';
		seeMoreLink?: string;
	}

	let { title, type, tag, sort, seeMoreLink }: Props = $props();

	let items = $state<DoubanSubject[]>([]);
	let loading = $state(true);
	let selectedVideo: DoubanSubject | null = $state(null);
	let showOverlay = $state(false);
	let cardRect: DOMRect | null = $state(null);
	let scrollContainer: HTMLDivElement | null = $state(null);
	let showGradient = $state(true);

	onMount(async () => {
		try {
			items = await searchSubjects({ type, tag, sort });
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
		<div class="relative z-0">
			<div
				bind:this={scrollContainer}
				class="scrollbar-hide flex gap-4 overflow-x-auto pb-4"
				onscroll={handleScroll}
			>
				{#each items as item (item.id)}
					<DoubanCard {item} onclick={handleCardClick} class="min-w-48" imgClass="!h-60" />
				{/each}
			</div>

			<div
				class="pointer-events-none absolute top-0 right-0 z-10 h-full w-20 bg-gradient-to-l from-background to-transparent transition-opacity duration-300 {showGradient
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
