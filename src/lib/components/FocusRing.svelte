<script lang="ts">
	import { onMount } from 'svelte';

	let ringEl: HTMLDivElement | undefined = $state();
	let currentFocus: HTMLElement | null = $state(null);

	function updateRing() {
		const focused = document.activeElement as HTMLElement;
		if (!focused || !ringEl) {
			currentFocus = null;
			return;
		}

		if (focused === currentFocus) return;

		const isFocusable =
			focused.matches('[tabindex], a, button, input') ||
			focused.closest('[tabindex], a, button, input');

		if (!isFocusable) {
			currentFocus = null;
			return;
		}

		currentFocus = focused;
	}

	function positionRing(el: HTMLElement) {
		if (!ringEl) return;
		const rect = el.getBoundingClientRect();
		ringEl.style.left = `${rect.left - 4}px`;
		ringEl.style.top = `${rect.top - 4}px`;
		ringEl.style.width = `${rect.width + 8}px`;
		ringEl.style.height = `${rect.height + 8}px`;

		const style = window.getComputedStyle(el);
		const br = parseFloat(style.borderRadius) || 0;
		ringEl.style.borderRadius = `${br + 4}px`;
	}

	let animationFrame: number;
	function loop() {
		if (currentFocus && ringEl) {
			positionRing(currentFocus);
		}
		animationFrame = requestAnimationFrame(loop);
	}

	onMount(() => {
		document.addEventListener('focusin', updateRing);
		document.addEventListener('focusout', updateRing);
		animationFrame = requestAnimationFrame(loop);

		return () => {
			document.removeEventListener('focusin', updateRing);
			document.removeEventListener('focusout', updateRing);
			cancelAnimationFrame(animationFrame);
		};
	});
</script>

<div bind:this={ringEl} class="focus-ring"></div>

<style>
	.focus-ring {
		position: fixed;
		pointer-events: none;
		z-index: 9999;
		border: 3px solid white;
		box-shadow: 0 0 0 2px rgba(0, 0, 0, 0.3);
		transition:
			left 0.1s ease-out,
			top 0.1s ease-out,
			width 0.1s ease-out,
			height 0.1s ease-out,
			border-radius 0.1s ease-out;
	}
</style>
