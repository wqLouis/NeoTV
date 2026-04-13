<script lang="ts">
	import type { Snippet } from 'svelte';
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

	function handleClick(opt: ToggleOption) {
		onchange(opt.value);
	}
</script>

<div class="flex gap-2 {className}">
	{#each options as opt (opt.value)}
		<button
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
