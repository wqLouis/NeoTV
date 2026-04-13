<script lang="ts">
	import type { Snippet } from 'svelte';
	import ToggleButtonGroup from './ToggleButtonGroup.svelte';

	interface ToggleOption {
		value: string;
		label: string;
		icon?: any;
	}

	interface Props {
		options: ToggleOption[];
		value: string;
		onchange: (value: string) => void;
		showRing?: boolean;
		focusedIndex?: number;
		focusRegion?: string;
		secondary?: Snippet;
	}

	let {
		options,
		value,
		onchange,
		showRing = false,
		focusedIndex = 0,
		focusRegion = '',
		secondary
	}: Props = $props();
</script>

<div class="sticky top-0 z-30 border-b bg-background/90 backdrop-blur-2xl">
	<div class="px-4 pt-4 pb-3">
		<ToggleButtonGroup {options} {value} {onchange} {showRing} {focusedIndex} {focusRegion} />
	</div>

	{#if secondary}
		<div class="border-t border-border"></div>
		<div class="px-4 py-3">
			{@render secondary()}
		</div>
	{/if}
</div>
