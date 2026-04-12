<script lang="ts">
	import { search, type SearchResult } from '$lib/api/search';
	import { searchDouban, type DoubanSubject } from '$lib/api/douban';
	import { settingsStore, GRID_DENSITY_CLASSES } from '$lib/stores/settings.svelte';
	import { searchHistoryStore } from '$lib/stores/search.svelte';
	import { DOUBAN_CHART_GENRE_IDS } from '$lib/api/constants';
	import SearchBar from '$lib/components/SearchBar.svelte';
	import VideoCard from '$lib/components/VideoCard.svelte';
	import VideoSourceOverlay from '$lib/components/VideoSourceOverlay.svelte';
	import CachedImage from '$lib/components/CachedImage.svelte';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import { Select, SelectTrigger, SelectContent, SelectItem } from '$lib/components/ui/select';
	import { X, Clock, Trash2 } from '@lucide/svelte';

	type SearchMode = 'api' | 'douban';
	type DoubanQuickFilter = 'hot' | 'new' | 'top';
	type DoubanType = 'movie' | 'tv';

	let query = $state('');
	let searchMode = $state<SearchMode>('api');
	let loading = $state(false);
	let hasSearched = $state(false);
	let apiResults = $state<SearchResult[]>([]);
	let doubanResults = $state<DoubanSubject[]>([]);
	let selectedVideo: DoubanSubject | null = $state(null);
	let showSourceOverlay = $state(false);
	let selectedCardRect: DOMRect | null = $state(null);

	let range = $state('0,10');
	let selectedGenre = $state('');
	let selectedCountry = $state('');
	let selectedType = $state<DoubanType>('movie');
	let doubanQuickFilter = $state<DoubanQuickFilter>('hot');

	const genres = Object.keys(DOUBAN_CHART_GENRE_IDS);
	const countries = [
		'中国大陆',
		'美国',
		'日本',
		'韩国',
		'香港',
		'台湾',
		'英国',
		'法国',
		'德国',
		'意大利',
		'西班牙',
		'印度',
		'泰国',
		'其他'
	];
	const ratingOptions = [
		{ value: '0,10', label: '全部' },
		{ value: '9,10', label: '9分以上' },
		{ value: '8,10', label: '8分以上' },
		{ value: '7,10', label: '7分以上' }
	];
	const quickFilters: { value: DoubanQuickFilter; label: string }[] = [
		{ value: 'hot', label: '热门' },
		{ value: 'new', label: '最新' },
		{ value: 'top', label: '高分' }
	];

	async function handleSearch(q: string) {
		if (!q.trim() && !selectedGenre && !selectedCountry && searchMode === 'api') return;

		loading = true;
		hasSearched = true;

		if (searchMode === 'api') {
			if (q.trim() && settingsStore.selectedApis.length === 0) {
				alert('请至少选择一个API源');
				loading = false;
				return;
			}
			if (q.trim()) {
				searchHistoryStore.add(q);
			}
			try {
				apiResults = await search(
					q,
					settingsStore.selectedApis,
					settingsStore.customApis,
					settingsStore.yellowFilterEnabled,
					settingsStore.commentaryFilterEnabled
				);
			} catch (e) {
				console.error('Search failed:', e);
				apiResults = [];
			}
		} else {
			try {
				const sortMap: Record<DoubanQuickFilter, string> = {
					hot: 'T',
					new: 'R',
					top: 'S'
				};
				doubanResults = await searchDouban({
					sort: sortMap[doubanQuickFilter],
					range: range,
					genres: selectedGenre || undefined,
					countries: selectedCountry || undefined,
					type: selectedType
				});
			} catch (e) {
				console.error('Search failed:', e);
				doubanResults = [];
			}
		}
		loading = false;
	}

	function handleHistoryClick(q: string) {
		query = q;
		searchMode = 'api';
		handleSearch(q);
	}

	function removeHistoryItem(q: string, e: Event) {
		e.stopPropagation();
		searchHistoryStore.remove(q);
	}

	function clearHistory() {
		searchHistoryStore.clear();
	}

	function setQuickFilter(filter: DoubanQuickFilter) {
		doubanQuickFilter = filter;
		if (hasSearched && searchMode === 'douban') {
			handleSearch('');
		}
	}

	$effect(() => {
		if (searchMode === 'douban' && hasSearched) {
			handleSearch('');
		}
	});

	function handleVideoClick(item: DoubanSubject, event: MouseEvent) {
		const target = event.currentTarget as HTMLDivElement;
		selectedCardRect = target.getBoundingClientRect();
		selectedVideo = item;
		showSourceOverlay = true;
	}
</script>

<div class="container mx-auto px-4 py-4">
	<div class="mb-4">
		<div class="mb-4 flex gap-2">
			<button
				class="flex-1 rounded-lg border py-2 text-sm font-medium transition-colors
					{searchMode === 'api'
					? 'border-primary bg-primary/10 text-primary'
					: 'border-border hover:bg-accent'}"
				onclick={() => (searchMode = 'api')}
			>
				全网搜索
			</button>
			<button
				class="flex-1 rounded-lg border py-2 text-sm font-medium transition-colors
					{searchMode === 'douban'
					? 'border-primary bg-primary/10 text-primary'
					: 'border-border hover:bg-accent'}"
				onclick={() => (searchMode = 'douban')}
			>
				豆瓣筛选
			</button>
		</div>

		{#if searchMode === 'api'}
			<SearchBar bind:value={query} placeholder="搜索视频..." onSearch={handleSearch} />
		{:else}
			<div class="sticky top-14 z-30 bg-background pt-8 pb-4">
				<div class="mb-3 flex gap-2">
					<button
						class="rounded-md px-3 py-1.5 text-sm font-medium transition-colors
							{selectedType === 'movie'
							? 'bg-primary text-primary-foreground'
							: 'bg-secondary hover:bg-secondary/80'}"
						onclick={() => (selectedType = 'movie')}
					>
						电影
					</button>
					<button
						class="rounded-md px-3 py-1.5 text-sm font-medium transition-colors
							{selectedType === 'tv'
							? 'bg-primary text-primary-foreground'
							: 'bg-secondary hover:bg-secondary/80'}"
						onclick={() => (selectedType = 'tv')}
					>
						电视剧
					</button>
				</div>

				<div class="scrollbar-hide mb-3 flex gap-2 overflow-x-auto pb-2">
					{#each quickFilters as filter}
						<button
							class="rounded-full px-3 py-1.5 text-sm whitespace-nowrap transition-colors
								{doubanQuickFilter === filter.value
								? 'bg-primary text-primary-foreground'
								: 'bg-secondary hover:bg-secondary/80'}"
							onclick={() => setQuickFilter(filter.value)}
						>
							{filter.label}
						</button>
					{/each}
				</div>

				<div class="scrollbar-hide flex gap-2 overflow-x-auto pb-2">
					<Select type="single" bind:value={selectedGenre}>
						<SelectTrigger
							class="h-9 shrink-0 rounded-md border border-input bg-transparent px-3 text-sm"
						>
							{selectedGenre || '类型'}
						</SelectTrigger>
						<SelectContent>
							<SelectItem value="">全部</SelectItem>
							{#each genres as genre}
								<SelectItem value={genre}>{genre}</SelectItem>
							{/each}
						</SelectContent>
					</Select>

					<Select type="single" bind:value={selectedCountry}>
						<SelectTrigger
							class="h-9 shrink-0 rounded-md border border-input bg-transparent px-3 text-sm"
						>
							{selectedCountry || '地区'}
						</SelectTrigger>
						<SelectContent>
							<SelectItem value="">全部</SelectItem>
							{#each countries as country}
								<SelectItem value={country}>{country}</SelectItem>
							{/each}
						</SelectContent>
					</Select>

					<Select type="single" bind:value={range}>
						<SelectTrigger
							class="h-9 shrink-0 rounded-md border border-input bg-transparent px-3 text-sm"
						>
							{ratingOptions.find((o) => o.value === range)?.label || '评分'}
						</SelectTrigger>
						<SelectContent>
							{#each ratingOptions as option}
								<SelectItem value={option.value}>{option.label}</SelectItem>
							{/each}
						</SelectContent>
					</Select>

					<Button size="sm" class="h-9 shrink-0" onclick={() => handleSearch('')}>筛选</Button>
				</div>
			</div>
		{/if}
	</div>

	{#if !hasSearched && searchMode === 'api'}
		{#if searchHistoryStore.items.length > 0}
			<div class="mb-6">
				<div class="mb-3 flex items-center justify-between">
					<div class="flex items-center gap-2">
						<Clock class="h-4 w-4 text-muted-foreground" />
						<span class="text-sm font-medium">搜索历史</span>
					</div>
					<Button variant="ghost" size="sm" onclick={clearHistory}>
						<Trash2 class="mr-1 h-4 w-4" />
						清空
					</Button>
				</div>
				<div class="flex flex-wrap gap-2">
					{#each searchHistoryStore.items as item}
						<div
							class="flex cursor-pointer items-center gap-2 rounded-full bg-secondary px-3 py-1.5 hover:bg-secondary/80"
							onclick={() => handleHistoryClick(item)}
							role="button"
							tabindex="0"
							onkeydown={(e) => e.key === 'Enter' && handleHistoryClick(item)}
						>
							<span class="text-sm">{item}</span>
							<button
								class="text-muted-foreground hover:text-foreground"
								onclick={(e) => removeHistoryItem(item, e)}
							>
								<X class="h-3 w-3" />
							</button>
						</div>
					{/each}
				</div>
			</div>
		{/if}

		<div class="py-12 text-center text-muted-foreground">
			<p>输入关键词搜索视频</p>
		</div>
	{:else if loading}
		<div class="grid {GRID_DENSITY_CLASSES[settingsStore.gridDensity]} gap-4">
			{#each Array(20) as _, i (i)}
				<div class="space-y-2">
					<Skeleton class="aspect-[2/3] w-full rounded-lg" />
					<Skeleton class="h-4 w-3/4" />
					<Skeleton class="h-3 w-1/2" />
				</div>
			{/each}
		</div>
	{:else if searchMode === 'api' && apiResults.length > 0}
		<div class="mb-4 flex items-center gap-2">
			<span class="text-sm text-muted-foreground">找到</span>
			<Badge variant="secondary">{apiResults.length}</Badge>
			<span class="text-sm text-muted-foreground">个结果</span>
		</div>
		<div class="grid {GRID_DENSITY_CLASSES[settingsStore.gridDensity]} gap-4">
			{#each apiResults as item (item.vod_id + item.source_code)}
				<VideoCard {item} />
			{/each}
		</div>
	{:else if searchMode === 'douban' && doubanResults.length > 0}
		<div class="mb-4 flex items-center gap-2">
			<span class="text-sm text-muted-foreground">找到</span>
			<Badge variant="secondary">{doubanResults.length}</Badge>
			<span class="text-sm text-muted-foreground">个结果</span>
		</div>
		<div class="grid {GRID_DENSITY_CLASSES[settingsStore.gridDensity]} gap-4">
			{#each doubanResults as item (item.id)}
				<div
					class="cursor-pointer overflow-hidden rounded-lg bg-card transition-all hover:scale-[1.02] hover:shadow-md"
					onclick={(e) => handleVideoClick(item, e)}
					role="button"
					tabindex="0"
					onkeydown={(e) => e.key === 'Enter' && handleVideoClick(item, e as unknown as MouseEvent)}
				>
					<div class="relative aspect-[2/3] w-full overflow-hidden">
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
	{:else if hasSearched}
		<div class="py-12 text-center text-muted-foreground">
			<p>没有找到匹配的结果</p>
			<p class="mt-2 text-sm">请尝试其他关键词或更换数据源</p>
		</div>
	{/if}
</div>

<VideoSourceOverlay
	item={selectedVideo}
	originRect={selectedCardRect}
	open={showSourceOverlay}
	onOpenChange={(open) => (showSourceOverlay = open)}
/>
