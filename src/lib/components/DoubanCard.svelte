<script lang="ts">
	import type { DoubanSubject } from '$lib/api/douban';
	import CachedImage from './CachedImage.svelte';
	import { Badge } from '$lib/components/ui/badge';

	interface Props {
		item: DoubanSubject;
		size?: 'default' | 'small';
		class?: string;
		imgClass?: string;
		onclick?: (item: DoubanSubject, e: MouseEvent) => void;
	}

	let { item, size = 'default', class: className = '', imgClass = '', onclick }: Props = $props();

	function handleClick(e: MouseEvent) {
		onclick?.(item, e);
	}
</script>

<div
	class="shrink-0 cursor-pointer overflow-hidden rounded-lg bg-card transition-all hover:scale-[1.02] hover:shadow-md focus-visible:scale-[1.02] focus-visible:shadow-lg focus-visible:ring-2 focus-visible:ring-ring {className}"
	onclick={handleClick}
	role="button"
	tabindex="0"
	onkeydown={(e) => e.key === 'Enter' && handleClick(e as unknown as MouseEvent)}
>
	<div class="relative aspect-[2/3] w-full overflow-hidden {imgClass}">
		<CachedImage
			src={item.cover_url || item.cover}
			alt={item.title}
			class="h-full w-full object-cover"
			referer="https://movie.douban.com/"
		/>
		{#if item.score || item.rate}
			<Badge class="absolute top-1.5 right-1.5 bg-yellow-500 text-xs text-black">
				{item.score || item.rate}
			</Badge>
		{/if}
	</div>
	<div class="p-2">
		<h3 class="line-clamp-2 text-xs font-medium" title={item.title}>
			{item.title}
		</h3>
		{#if item.types?.length}
			<p class="text-xs text-muted-foreground">
				{item.types.slice(0, 2).join(' / ')}
			</p>
		{/if}
	</div>
</div>
