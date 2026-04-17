<script lang="ts">
	import type { Component } from 'svelte';
	import { SvelteMap } from 'svelte/reactivity';
	import { NavNode, LEAF, type NavDir } from '$lib/nav-graph/Core';

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

	export function buildNavNode(): NavNode | null {
		const validButtons = buttonRefs.filter((b): b is HTMLElement => !!b);
		if (validButtons.length < 2) return null;
		if (!containerEl) return null;

		const navGraph = new SvelteMap<NavNode, SvelteMap<NavDir, NavNode>>();
		const tabNodes: NavNode[] = [];

		validButtons.forEach((btn) => {
			const node = new NavNode(btn, LEAF, navGraph);
			navGraph.set(node, new SvelteMap());
			tabNodes.push(node);
		});

		tabNodes.forEach((node, i) => {
			const conn = navGraph.get(node)!;
			if (i > 0) conn.set('LEFT', tabNodes[i - 1]);
			if (i < tabNodes.length - 1) conn.set('RIGHT', tabNodes[i + 1]);
		});

		const parentNode = new NavNode(containerEl, tabNodes[0], navGraph);
		navGraph.set(parentNode, new SvelteMap());

		console.log('[node] ToggleButtonGroup built:', tabNodes.length, 'tabs, parent:', containerEl);

		return parentNode;
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
