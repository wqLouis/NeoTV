<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { DoubanSubject } from '$lib/api/douban';
	import CachedImage from './CachedImage.svelte';
	import { Badge } from '$lib/components/ui/badge';

	interface Props {
		item: DoubanSubject;
		fluid?: boolean;
		onclick?: (item: DoubanSubject, e: MouseEvent) => void;
		action?: Snippet;
		overlay?: Snippet<[Pick<DoubanSubject, 'title' | 'types'>]>;
	}

	let { item, fluid = false, onclick, action, overlay }: Props = $props();

	function handleClick(e: MouseEvent) {
		onclick?.(item, e);
	}
</script>

<div
	class="relative aspect-[2/3] shrink-0 cursor-pointer overflow-hidden rounded-lg bg-card transition-transform hover:scale-105 {fluid
		? 'w-full'
		: 'w-40'}"
	onclick={handleClick}
	role="button"
	tabindex="-1"
>
	<CachedImage
		src={item.cover_url || item.cover}
		alt={item.title}
		class="h-full w-full object-cover"
		referer="https://movie.douban.com/"
	/>

	<div
		class="absolute inset-x-0 bottom-0 h-1/3 bg-gradient-to-t from-black/40 to-transparent"
	></div>

	{#if item.score || item.rate}
		<Badge class="absolute top-1.5 right-1.5 bg-yellow-500 text-xs text-black">
			{item.score || item.rate}
		</Badge>
	{/if}

	{#if action}
		<div class="absolute top-1.5 left-1.5 z-10">
			{@render action()}
		</div>
	{/if}

	{#if overlay}
		{@render overlay({ title: item.title, types: item.types })}
	{:else}
		<div class="absolute right-0 bottom-0 left-0 bg-gradient-to-t from-black/60 to-transparent p-2">
			<h3 class="line-clamp-2 text-xs font-medium text-white" title={item.title}>
				{item.title}
			</h3>
			{#if item.types?.length}
				<p class="text-xs text-white/70">{item.types.slice(0, 2).join(' / ')}</p>
			{/if}
		</div>
	{/if}
</div>
