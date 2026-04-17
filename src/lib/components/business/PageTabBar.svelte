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
		secondary?: Snippet;
	}

	let { options, value, onchange, secondary }: Props = $props();

	let tabGroup: ToggleButtonGroup;

	export function buildNavNode() {
		return tabGroup?.buildNavNode() ?? null;
	}
</script>

<div class="sticky top-0 z-30 border-b bg-background/90 backdrop-blur-2xl">
	<div class="px-4 pt-4 pb-3">
		<ToggleButtonGroup bind:this={tabGroup} {options} {value} {onchange} />
	</div>

	{#if secondary}
		<div class="border-t border-border"></div>
		<div class="px-4 py-3">
			{@render secondary()}
		</div>
	{/if}
</div>
