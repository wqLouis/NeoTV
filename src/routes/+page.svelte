<script lang="ts">
	import { goto } from '$app/navigation';
	import { fetchDoubanChart, fetchDoubanTags } from '$lib/api/douban';
	import { DOUBAN_CHART_GENRE_IDS } from '$lib/api/constants';
	import { Badge } from '$lib/components/ui/badge';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import { Tabs, TabsList, TabsTrigger } from '$lib/components/ui/tabs';

	let movieTags = $state<string[]>([]);
	let tvTags = $state<string[]>([]);
	let selectedGenre = $state('剧情');
	let doubanSwitch = $state<'movie' | 'tv'>('movie');
	let loading = $state(false);
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

	const genres = Object.keys(DOUBAN_CHART_GENRE_IDS);

	async function loadTags() {
		const movie = await fetchDoubanTags('movie');
		const tv = await fetchDoubanTags('tv');
		const genreIds = Object.keys(DOUBAN_CHART_GENRE_IDS);
		movieTags = movie.filter((t) => genreIds.includes(t));
		if (movieTags.length === 0) {
			movieTags = ['剧情', '喜剧', '动作', '爱情', '科幻', '动画', '悬疑', '惊悚', '恐怖'];
		}
		tvTags = tv.filter((t) => genreIds.includes(t));
		if (tvTags.length === 0) {
			tvTags = ['剧情', '喜剧', '动作', '爱情', '科幻', '动画', '悬疑', '惊悚', '恐怖'];
		}
	}

	async function loadCharts() {
		loading = true;
		try {
			const data = await fetchDoubanChart(selectedGenre);
			charts = data;
		} catch (e) {
			console.error('Failed to load charts:', e);
		} finally {
			loading = false;
		}
	}

	function handleVideoClick(item: (typeof charts)[0]) {
		const coverUrl = item.cover_url || item.cover;
		const params = new URLSearchParams({
			id: item.id,
			source: 'douban',
			title: item.title,
			cover: coverUrl || ''
		});
		goto(`/player?${params.toString()}`);
	}

	$effect(() => {
		loadTags();
	});

	$effect(() => {
		loadCharts();
	});

	function getCoverUrl(item: (typeof charts)[0]): string {
		if (item.cover_url) {
			return `/api/proxy?url=${encodeURIComponent(item.cover_url)}`;
		}
		return item.cover || '';
	}
</script>

<div class="container mx-auto px-4 py-4">
	<div class="sticky top-14 z-30 mb-4 bg-background pt-2 pb-4">
		<Tabs bind:value={doubanSwitch} class="mb-4">
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
					onclick={() => (selectedGenre = tag)}
				>
					{tag}
				</button>
			{/each}
		</div>
	</div>

	{#if loading}
		<div class="grid grid-cols-3 gap-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-8">
			{#each Array(16) as _}
				<div class="space-y-2">
					<Skeleton class="aspect-[2/3] w-full rounded-lg" />
					<Skeleton class="h-4 w-3/4" />
					<Skeleton class="h-3 w-1/2" />
				</div>
			{/each}
		</div>
	{:else}
		<div class="grid grid-cols-3 gap-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-8">
			{#each charts as item (item.id)}
				<div
					class="cursor-pointer overflow-hidden rounded-lg bg-card transition-all hover:scale-[1.02] hover:shadow-md"
					onclick={() => handleVideoClick(item)}
					role="button"
					tabindex="0"
					onkeydown={(e) => e.key === 'Enter' && handleVideoClick(item)}
				>
					<div class="relative aspect-[2/3] w-full overflow-hidden">
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
	{/if}
</div>

<style>
	.scrollbar-hide::-webkit-scrollbar {
		display: none;
	}
	.scrollbar-hide {
		-ms-overflow-style: none;
		scrollbar-width: none;
	}
</style>
