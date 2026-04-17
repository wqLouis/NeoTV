<script lang="ts">
	let ringEl: HTMLDivElement = $state()!;
	let isVisible = $state(false);
	let isAnimating = $state(false);

	export function updateFocus(el: HTMLElement | null) {
		if (!el || !ringEl) {
			isVisible = false;
			return;
		}

		const rect = el.getBoundingClientRect();
		const left = rect.left - 4;
		const top = rect.top - 4;
		const width = rect.width + 8;
		const height = rect.height + 8;

		const hasMoved =
			ringEl.style.left !== `${left}px` ||
			ringEl.style.top !== `${top}px` ||
			ringEl.style.width !== `${width}px` ||
			ringEl.style.height !== `${height}px`;

		ringEl.style.left = `${left}px`;
		ringEl.style.top = `${top}px`;
		ringEl.style.width = `${width}px`;
		ringEl.style.height = `${height}px`;
		isVisible = true;

		if (hasMoved) {
			isAnimating = true;
			setTimeout(() => {
				isAnimating = false;
			}, 150);
		}
	}

	export function hide() {
		isVisible = false;
	}
</script>

<div
	bind:this={ringEl}
	class="focus-ring"
	class:hidden={!isVisible}
	class:animating={isAnimating}
></div>

<style>
	.focus-ring {
		position: fixed;
		pointer-events: none;
		z-index: 9999;
		border: 3px solid white;
		border-radius: 12px;
		box-shadow:
			0 0 20px 4px rgba(255, 255, 255, 0.5),
			0 0 40px 8px rgba(255, 255, 255, 0.3);
		transition:
			left 0.15s ease-out,
			top 0.15s ease-out,
			width 0.15s ease-out,
			height 0.15s ease-out,
			opacity 0.15s ease;
	}

	.focus-ring.animating {
		animation: focusPop 0.15s ease-out;
	}

	.focus-ring.hidden {
		opacity: 0;
	}

	@keyframes focusPop {
		0% {
			transform: scale(0.95);
			opacity: 0.5;
		}
		50% {
			transform: scale(1.03);
		}
		100% {
			transform: scale(1);
			opacity: 1;
		}
	}
</style>
