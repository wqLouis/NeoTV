<script lang="ts">
	import type { DoubanSubject } from '$lib/api/douban';
	import CachedImage from './CachedImage.svelte';
	import { Badge } from '$lib/components/ui/badge';

	interface Props {
		item: DoubanSubject;
		size?: 'default' | 'small';
		fluid?: boolean;
		focused?: boolean;
		onclick?: (item: DoubanSubject, e: MouseEvent) => void;
	}

	let { item, size = 'default', fluid = false, focused = false, onclick }: Props = $props();

	function handleClick(e: MouseEvent) {
		onclick?.(item, e);
	}
</script>

<div
	class="relative shrink-0 cursor-pointer overflow-hidden rounded-lg bg-card transition-transform hover:scale-105 focus-visible:scale-105 focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none {fluid
		? 'w-full'
		: 'w-40'} {focused ? 'ring-2 ring-primary' : ''}"
	onclick={handleClick}
	role="button"
	tabindex="0"
	onkeydown={(e) => e.key === 'Enter' && handleClick(e as unknown as MouseEvent)}
	style="aspect-ratio: 2/3;"
>
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
	<div class="absolute right-0 bottom-0 left-0 bg-gradient-to-t from-black/60 to-transparent p-2">
		<h3 class="line-clamp-2 text-xs font-medium text-white" title={item.title}>
			{item.title}
		</h3>
		{#if item.types?.length}
			<p class="text-xs text-white/70">{item.types.slice(0, 2).join(' / ')}</p>
		{/if}
	</div>
</div>
