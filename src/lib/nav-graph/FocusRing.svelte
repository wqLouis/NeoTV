<script lang="ts">
	let ringEl: HTMLDivElement = $state()!;
	let isVisible = $state(false);

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

		const style = window.getComputedStyle(el);
		const br = parseFloat(style.borderRadius) + 4;
		ringEl.style.borderRadius = `${br}px`;

		ringEl.style.left = `${left}px`;
		ringEl.style.top = `${top}px`;
		ringEl.style.width = `${width}px`;
		ringEl.style.height = `${height}px`;
		isVisible = true;
	}

	export function hide() {
		isVisible = false;
	}
</script>

<div bind:this={ringEl} class="focus-ring" class:hidden={!isVisible}></div>

<style>
	.focus-ring {
		position: fixed;
		pointer-events: none;
		z-index: 9999;
		border: 3px solid white;
		transition:
			left 0.1s ease-in-out,
			top 0.1s ease-in-out,
			width 0.1s ease-in-out,
			height 0.1s ease-in-out,
			border-radius 0.1s ease-in-out;
	}

	.focus-ring.hidden {
		display: none;
	}
</style>
