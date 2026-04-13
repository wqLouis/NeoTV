<script lang="ts">
	import type { Component } from 'svelte';

	interface Props {
		icon?: Component<{ class?: string }>;
		title?: string;
		message: string;
		description?: string;
		class?: string;
	}

	let { icon: Icon, title, message, description, class: className = '' }: Props = $props();
</script>

<div class="py-12 text-center text-muted-foreground {className}">
	{#if Icon}
		<Icon class="mx-auto mb-4 h-12 w-12 opacity-50" />
	{/if}
	{#if title}
		<p class="text-lg font-medium">{title}</p>
	{/if}
	<p>{message}</p>
	{#if description}
		<p class="mt-2 text-sm">{description}</p>
	{/if}
</div>
