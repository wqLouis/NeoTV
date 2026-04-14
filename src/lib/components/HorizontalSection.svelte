<script lang="ts">
	import { searchSubjects, type DoubanSubject } from '$lib/api/douban';
	import VideoSourceOverlay from './VideoSourceOverlay.svelte';
	import DoubanCard from '$lib/components/DoubanCard.svelte';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { ArrowRight } from '@lucide/svelte';

	interface Props {
		title: string;
		type: 'movie' | 'tv';
		tag: string;
		sort: 'recommend' | 'time' | 'rank';
		seeMoreLink?: string;
		nodePrefix?: string;
	}

	let { title, type, tag, sort, seeMoreLink, nodePrefix = '' }: Props = $props();

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

<div class="relative flex flex-1 flex-col overflow-hidden">
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
		<div class="flex gap-4">
			{#each Array(8) as _, i (i)}
				<div class="h-64 w-40 shrink-0 rounded-lg bg-muted"></div>
			{/each}
		</div>
	{:else}
		<div class="relative">
			<div
				bind:this={scrollContainer}
				class="scrollbar-hide flex gap-4 overflow-x-auto py-2"
				onscroll={handleScroll}
			>
				{#each items as item (item.id)}
					<DoubanCard {item} onclick={handleCardClick} />
				{/each}
			</div>

			<div
				class="pointer-events-none absolute top-0 right-0 h-full w-16 bg-gradient-to-l from-background to-transparent transition-opacity duration-300 {showGradient
					? 'opacity-100'
					: 'opacity-0'}"
			></div>
		</div>
	{/if}
</div>

<VideoSourceOverlay
	item={selectedVideo}
	originRect={cardRect}
	open={showOverlay}
	onOpenChange={(open) => (showOverlay = open)}
/>
