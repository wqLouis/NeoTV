<script lang="ts">
	import { fetchImage } from '$lib/cache';

	interface Props {
		src: string;
		alt: string;
		class?: string;
		fallback?: string;
		referer?: string;
	}

	let {
		src,
		alt,
		class: className = '',
		fallback = 'https://via.placeholder.com/300x450?text=无封面',
		referer
	}: Props = $props();

	let loaded = $state(false);
	let error = $state(false);
	let cachedSrc = $state('');

	$effect(() => {
		if (!src) return;
		loaded = false;
		error = false;
		cachedSrc = '';

		if (src.startsWith('data:') || src.startsWith('blob:') || src.startsWith('/')) {
			cachedSrc = src;
			loaded = true;
			return;
		}

		fetchImage(src, referer)
			.then((result) => {
				cachedSrc = result;
				loaded = true;
			})
			.catch(() => {
				error = true;
				loaded = true;
			});
	});
</script>

{#if !loaded}
	<div class="animate-pulse bg-muted {className}"></div>
{:else if error || !cachedSrc}
	<img src={fallback} {alt} class={className} />
{:else}
	<img src={cachedSrc} {alt} class={className} />
{/if}
