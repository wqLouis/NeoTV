<script lang="ts">
	import type { Component } from 'svelte';

	interface ToggleOption {
		value: string;
		label: string;
		icon?: Component<{ class?: string }>;
	}

	interface Props {
		options: ToggleOption[];
		value: string;
		onchange: (value: string) => void;
		class?: string;
	}

	let { options, value, onchange, class: className = '' }: Props = $props();

	let buttonRefs = $state<HTMLElement[]>([]);
	let containerEl: HTMLDivElement | null = null;

	function handleClick(opt: ToggleOption) {
		onchange(opt.value);
	}
</script>

<div bind:this={containerEl} class="flex gap-2 {className}">
	{#each options as opt, i (opt.value)}
		<button
			bind:this={buttonRefs[i]}
			class="flex items-center gap-2 rounded-lg px-4 py-2 transition-colors
				{value === opt.value
				? 'bg-primary text-primary-foreground'
				: 'text-muted-foreground hover:bg-secondary hover:text-foreground'}"
			onclick={() => handleClick(opt)}
		>
			{#if opt.icon}
				<opt.icon class="h-5 w-5" />
			{/if}
			<span class="font-medium">{opt.label}</span>
		</button>
	{/each}
</div>
