<script lang="ts">
	import { goto } from '$app/navigation';
	import type { SearchResult } from '$lib/api/search';
	import { Badge } from '$lib/components/ui/badge';

	interface Props {
		item: SearchResult;
	}

	let { item }: Props = $props();

	function handleClick() {
		const params = new URLSearchParams({
			id: item.vod_id,
			source: item.source_code,
			title: item.vod_name
		});
		goto(`/player?${params.toString()}`);
	}

	function hasCover(): boolean {
		return !!(item.vod_pic && item.vod_pic.startsWith('http'));
	}
</script>

<div
	class="card-hover h-full cursor-pointer overflow-hidden rounded-lg bg-card shadow-sm transition-all hover:scale-[1.02] hover:shadow-md"
	onclick={handleClick}
	role="button"
	tabindex="0"
	onkeydown={(e) => e.key === 'Enter' && handleClick()}
>
	{#if hasCover()}
		<div class="relative aspect-[2/3] w-full overflow-hidden">
			<img
				src={item.vod_pic}
				alt={item.vod_name}
				class="h-full w-full object-cover transition-transform hover:scale-110"
				loading="lazy"
				onerror={(e) => {
					const img = e.currentTarget as HTMLImageElement;
					img.src = 'https://via.placeholder.com/300x450?text=无封面';
					img.classList.add('object-contain');
				}}
			/>
		</div>
	{/if}

	<div class="p-3">
		<h3 class="mb-2 line-clamp-2 font-semibold" title={item.vod_name}>
			{item.vod_name}
		</h3>

		<div class="mb-2 flex flex-wrap gap-1">
			{#if item.type_name}
				<Badge variant="secondary" class="text-xs">
					{item.type_name}
				</Badge>
			{/if}
			{#if item.vod_year}
				<Badge variant="outline" class="text-xs">
					{item.vod_year}
				</Badge>
			{/if}
		</div>

		<p class="line-clamp-2 text-sm text-muted-foreground">
			{item.vod_remarks || '暂无介绍'}
		</p>

		<div class="mt-2 flex items-center justify-between border-t pt-2">
			<Badge variant="outline" class="bg-secondary text-xs">
				{item.source_name}
			</Badge>
		</div>
	</div>
</div>
