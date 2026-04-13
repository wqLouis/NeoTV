<script lang="ts">
	import { focusNavigator } from '$lib/utils/focus-navigator';
	import { onMount } from 'svelte';

	let ringEl = $state<HTMLDivElement | null>(null);
	let isVisible = $state(false);
	let isMoving = $state(false);

	onMount(() => {
		let animationId: number;

		const updateRing = () => {
			const nodeId = focusNavigator.getCurrentNodeId();
			const root = focusNavigator.getRoot();

			if (!nodeId || !root) {
				if (isVisible) isVisible = false;
				animationId = requestAnimationFrame(updateRing);
				return;
			}

			const node = root.getNode(nodeId);
			if (node && 'element' in node) {
				const el = (node as unknown as { element: HTMLElement }).element;
				if (el && ringEl) {
					const rect = el.getBoundingClientRect();
					ringEl.style.left = `${rect.left - 4}px`;
					ringEl.style.top = `${rect.top - 4}px`;
					ringEl.style.width = `${rect.width + 8}px`;
					ringEl.style.height = `${rect.height + 8}px`;
					isVisible = true;
					isMoving = true;
					setTimeout(() => (isMoving = false), 150);
				}
			}
			animationId = requestAnimationFrame(updateRing);
		};

		animationId = requestAnimationFrame(updateRing);

		return () => {
			cancelAnimationFrame(animationId);
		};
	});
</script>

{#if isVisible}
	<div bind:this={ringEl} class="tv-focus-ring" class:moving={isMoving}></div>
{/if}

<style>
	.tv-focus-ring {
		position: absolute;
		pointer-events: none;
		z-index: 9999;
		border: 3px solid oklch(100% 0 0);
		border-radius: 8px;
		box-shadow:
			0 0 0 2px oklch(100% 0 0 / 0.3),
			0 0 30px 4px oklch(100% 0 0 / 0.5),
			0 0 60px 8px oklch(100% 0 0 / 0.2);
		transition-duration: 200ms;
		transition:
			left 200ms cubic-bezier(0, 0, 0.2, 1),
			top 200ms cubic-bezier(0, 0, 0.2, 1),
			width 200ms cubic-bezier(0, 0, 0.2, 1),
			height 200ms cubic-bezier(0, 0, 0.2, 1);
	}

	.tv-focus-ring.moving {
		transform: scale(1.03);
		transition:
			left 150ms cubic-bezier(0, 0, 0.2, 1),
			top 150ms cubic-bezier(0, 0, 0.2, 1),
			width 150ms cubic-bezier(0, 0, 0.2, 1),
			height 150ms cubic-bezier(0, 0, 0.2, 1),
			transform 150ms cubic-bezier(0, 0, 0.2, 1);
	}
</style>
