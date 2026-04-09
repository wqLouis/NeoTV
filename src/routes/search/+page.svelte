<script lang="ts">
	import { search, type SearchResult } from '$lib/api/search';
	import { searchDouban, type DoubanSubject } from '$lib/api/douban';
	import { settingsStore } from '$lib/stores/settings.svelte';
	import { searchHistoryStore } from '$lib/stores/search.svelte';
	import { DOUBAN_CHART_GENRE_IDS } from '$lib/api/constants';
	import SearchBar from '$lib/components/SearchBar.svelte';
	import VideoCard from '$lib/components/VideoCard.svelte';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import { X, Clock, Trash2 } from 'lucide-svelte';

	type SearchMode = 'api' | 'douban';
	type DoubanQuickFilter = 'hot' | 'new' | 'top';

	let query = $state('');
	let searchMode = $state<SearchMode>('api');
	let loading = $state(false);
	let hasSearched = $state(false);
	let apiResults = $state<SearchResult[]>([]);
	let doubanResults = $state<DoubanSubject[]>([]);

	let sortBy = $state('U');
	let range = $state('0,10');
	let selectedGenre = $state('');
	let selectedCountry = $state('');
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
	const sortOptions = [
		{ value: 'U', label: '综合' },
		{ value: 'T', label: '评分最高' },
		{ value: 'S', label: '评价最多' }
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
		if (!q.trim() && !selectedGenre && !selectedCountry) return;

		loading = true;
		hasSearched = true;

		if (searchMode === 'api') {
			if (settingsStore.selectedApis.length === 0) {
				alert('请至少选择一个API源');
				loading = false;
				return;
			}
			searchHistoryStore.add(q);
			try {
				apiResults = await search(
					q,
					settingsStore.selectedApis,
					settingsStore.customApis,
					settingsStore.yellowFilterEnabled
				);
			} catch (e) {
				console.error('Search failed:', e);
				apiResults = [];
			}
		} else {
			try {
				const sortMap: Record<DoubanQuickFilter, string> = {
					hot: 'U',
					new: 'S',
					top: 'T'
				};
				doubanResults = await searchDouban({
					sort: sortMap[doubanQuickFilter],
					range: range,
					tags: selectedGenre || undefined,
					countries: selectedCountry || undefined
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

	function getCoverUrl(item: DoubanSubject): string {
		if (item.cover_url) {
			return `/api/proxy?url=${encodeURIComponent(item.cover_url)}`;
		}
		return item.cover || '';
	}

	$effect(() => {
		if (searchMode === 'douban' && hasSearched) {
			handleSearch('');
		}
	});
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
			<div class="sticky top-14 z-30 bg-background pb-4">
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
					<select
						bind:value={selectedGenre}
						class="h-9 shrink-0 rounded-md border border-input bg-transparent px-3 text-sm"
					>
						<option value="">类型</option>
						{#each genres as genre}
							<option value={genre}>{genre}</option>
						{/each}
					</select>

					<select
						bind:value={selectedCountry}
						class="h-9 shrink-0 rounded-md border border-input bg-transparent px-3 text-sm"
					>
						<option value="">地区</option>
						{#each countries as country}
							<option value={country}>{country}</option>
						{/each}
					</select>

					<select
						bind:value={range}
						class="h-9 shrink-0 rounded-md border border-input bg-transparent px-3 text-sm"
					>
						{#each ratingOptions as option}
							<option value={option.value}>{option.label}</option>
						{/each}
					</select>

					<Button size="sm" class="h-9 shrink-0" onclick={() => handleSearch('')}>筛选</Button>
				</div>
			</div>
		{/if}
	</div>

	{#if searchMode === 'api' && !hasSearched}
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
		<div class="grid grid-cols-3 gap-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-8">
			{#each Array(16) as _}
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
		<div class="grid grid-cols-3 gap-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-8">
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
		<div class="grid grid-cols-3 gap-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-8">
			{#each doubanResults as item (item.id)}
				<div
					class="cursor-pointer overflow-hidden rounded-lg bg-card transition-all hover:scale-[1.02] hover:shadow-md"
					onclick={() => {
						const params = new URLSearchParams({
							id: item.id,
							source: 'douban',
							title: item.title,
							cover: item.cover_url || ''
						});
						window.location.href = `/player?${params.toString()}`;
					}}
					role="button"
					tabindex="0"
					onkeydown={(e) => {
						if (e.key === 'Enter') {
							const params = new URLSearchParams({
								id: item.id,
								source: 'douban',
								title: item.title,
								cover: item.cover_url || ''
							});
							window.location.href = `/player?${params.toString()}`;
						}
					}}
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
	{:else}
		<div class="py-12 text-center text-muted-foreground">
			<p>没有找到匹配的结果</p>
			<p class="mt-2 text-sm">请尝试其他关键词或更换数据源</p>
		</div>
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
