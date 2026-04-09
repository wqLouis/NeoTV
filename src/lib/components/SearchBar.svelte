<script lang="ts">
	import { Input } from '$lib/components/ui/input';
	import { Button } from '$lib/components/ui/button';
	import { Search, X, History } from 'lucide-svelte';

	interface Props {
		value: string;
		placeholder?: string;
		onSearch: (query: string) => void;
		onClear?: () => void;
	}

	let { value = $bindable(), placeholder = '搜索...', onSearch, onClear }: Props = $props();

	function handleSubmit(e: Event) {
		e.preventDefault();
		if (value.trim()) {
			onSearch(value.trim());
		}
	}

	function handleClear() {
		value = '';
		onClear?.();
	}
</script>

<form onsubmit={handleSubmit} class="relative flex w-full gap-2">
	<div class="relative flex-grow">
		<Search class="absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
		<Input type="text" {placeholder} bind:value class="pr-10 pl-10" />
		{#if value}
			<button
				type="button"
				class="absolute top-1/2 right-3 -translate-y-1/2 text-muted-foreground hover:text-foreground"
				onclick={handleClear}
			>
				<X class="h-4 w-4" />
			</button>
		{/if}
	</div>
	<Button type="submit">搜索</Button>
</form>
