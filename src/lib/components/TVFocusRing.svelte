<script lang="ts">
	import { tvNav, type FocusRegion } from '$lib/utils/tv-navigation.svelte';
	import { browser } from '$app/environment';
	import { onMount } from 'svelte';

	interface FocusTarget {
		region: FocusRegion;
		index: number;
		element: HTMLElement;
	}

	let targetElement = $state<HTMLElement | null>(null);
	let rect = $state<DOMRect | null>(null);
	let scale = $state(1);
	let opacity = $state(0);

	function getTarget(region: FocusRegion, index: number): HTMLElement | null {
		if (!browser) return null;

		if (region === 'tabs') {
			return document.querySelector(`[data-tv-tab="${index}"]`);
		} else if (region === 'genres') {
			return document.querySelector(`[data-tv-genre="${index}"]`);
		} else if (region === 'grid') {
			return document.querySelector(`[data-tv-card="${index}"]`);
		}
		return null;
	}

	function updatePosition() {
		const { focusRegion, focusedTabIndex, focusedGenreIndex, focusedCardIndex } = tvNav.state;

		if (focusRegion === 'none') {
			opacity = 0;
			return;
		}

		let index: number;
		switch (focusRegion) {
			case 'tabs':
				index = focusedTabIndex;
				break;
			case 'genres':
				index = focusedGenreIndex;
				break;
			case 'grid':
				index = focusedCardIndex;
				break;
			default:
				opacity = 0;
				return;
		}

		if (index < 0) {
			opacity = 0;
			return;
		}

		const element = getTarget(focusRegion, index);
		if (element) {
			targetElement = element;
			const newRect = element.getBoundingClientRect();
			rect = newRect;
			scale = 1.02;
			opacity = 1;
		} else {
			opacity = 0;
		}
	}

	$effect(() => {
		if (browser) {
			tvNav.state;
			requestAnimationFrame(updatePosition);
		}
	});
</script>

{#if browser && opacity > 0 && rect}
	<div
		class="tv-focus-ring"
		style="
			position: fixed;
			top: {rect.top - 4}px;
			left: {rect.left - 4}px;
			width: {rect.width + 8}px;
			height: {rect.height + 8}px;
			pointer-events: none;
			z-index: 9999;
			border: 2px solid var(--primary, oklch(0.922 0 0));
			border-radius: 8px;
			box-shadow: 0 0 12px 2px var(--ring, oklch(0.556 0 0) / 0.5);
			transition: top 150ms ease-out, left 150ms ease-out, width 150ms ease-out, height 150ms ease-out, transform 200ms ease-out, opacity 150ms ease-out;
			transform: scale({scale});
		"
	></div>
{/if}

<style>
	.tv-focus-ring {
		animation: focusPulse 1.5s ease-in-out infinite;
	}

	@keyframes focusPulse {
		0%,
		100% {
			box-shadow: 0 0 12px 2px var(--ring, oklch(0.556 0 0) / 0.3);
		}
		50% {
			box-shadow: 0 0 20px 4px var(--ring, oklch(0.556 0 0) / 0.6);
		}
	}
</style>
