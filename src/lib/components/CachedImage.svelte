<script lang="ts">
	import { fetchImage } from '$lib/cache';
	import { onMount } from 'svelte';

	interface Props {
		src: string;
		alt: string;
		class?: string;
		fallback?: string;
		referer?: string;
		aspectRatio?: string;
		lazy?: boolean;
	}

	let {
		src,
		alt,
		class: className = '',
		fallback = 'https://via.placeholder.com/300x450?text=无封面',
		referer,
		aspectRatio = '2/3',
		lazy = true
	}: Props = $props();

	let loaded = $state(false);
	let error = $state(false);
	let cachedSrc = $state('');
	let currentSrc = $state('');
	let shouldLoad = $state(!lazy);
	let imgElement: HTMLImageElement | null = $state(null);
	let containerElement: HTMLDivElement | null = $state(null);

	onMount(() => {
		if (!lazy || !containerElement) return;

		const observer = new IntersectionObserver(
			(entries) => {
				if (entries[0].isIntersecting) {
					shouldLoad = true;
					observer.disconnect();
				}
			},
			{ rootMargin: '200px' }
		);

		observer.observe(containerElement);

		return () => observer.disconnect();
	});

	$effect(() => {
		if (!shouldLoad || !src) {
			loaded = false;
			cachedSrc = '';
			currentSrc = '';
			return;
		}

		loaded = false;
		error = false;
		cachedSrc = '';
		currentSrc = src;

		if (src.startsWith('data:') || src.startsWith('blob:') || src.startsWith('/')) {
			cachedSrc = src;
			loaded = true;
			return;
		}

		fetchImage(src, referer)
			.then((result) => {
				if (currentSrc === src) {
					cachedSrc = result;
					loaded = true;
				}
			})
			.catch(() => {
				if (currentSrc === src) {
					error = true;
					loaded = true;
				}
			});
	});
</script>

<div bind:this={containerElement} class="relative {className}" style="aspect-ratio: {aspectRatio};">
	{#if !loaded}
		<div class="absolute inset-0 animate-pulse bg-muted"></div>
	{:else if error || !cachedSrc}
		<img src={fallback} {alt} class="absolute inset-0 h-full w-full object-cover" />
	{:else}
		<img
			bind:this={imgElement}
			src={cachedSrc}
			{alt}
			class="absolute inset-0 h-full w-full object-cover"
		/>
	{/if}
</div>
