<script lang="ts">
	import { tvNav } from '$lib/utils/tv-navigation.svelte';
	import { settingsStore } from '$lib/stores/settings.svelte';
	import { browser } from '$app/environment';
	import { onMount } from 'svelte';

	let rect = $state<DOMRect | null>(null);
	let opacity = $state(0);
	let lastNodeId = $state('');

	function updatePosition() {
		if (!browser) return;

		if (!settingsStore.focusRingEnabled) {
			opacity = 0;
			return;
		}

		const focused = tvNav.state.focusedNodeId;
		const overlay = tvNav.state.overlayActive;

		if (overlay) {
			opacity = 0;
			return;
		}

		if (!focused) {
			opacity = 0;
			return;
		}

		const selector = `[data-tv-node="${focused}"]`;
		const element = document.querySelector(selector) as HTMLElement | null;
		console.log('[TVFocusRing] Looking for:', selector, 'Found:', !!element);
		if (element) {
			const newRect = element.getBoundingClientRect();
			rect = newRect;
			opacity = 1;
			lastNodeId = focused;
		} else {
			opacity = 0;
		}
	}

	onMount(() => {
		updatePosition();

		const interval = setInterval(updatePosition, 100);
		return () => clearInterval(interval);
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
			transition: top 150ms ease-out, left 150ms ease-out, width 150ms ease-out, height 150ms ease-out, opacity 150ms ease-out;
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
