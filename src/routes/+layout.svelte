<script lang="ts">
	import type { Pathname } from '$app/types';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { locales, localizeHref } from '$lib/paraglide/runtime';
	import { onMount } from 'svelte';
	import { Home, Search, History, Settings } from 'lucide-svelte';
	import Sonner from '$lib/components/ui/sonner/sonner.svelte';
	import { themeStore } from '$lib/stores/theme.svelte';
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';

	let { children } = $props();

	const navItems = [
		{ href: '/', label: '首页', icon: Home },
		{ href: '/search', label: '搜索', icon: Search },
		{ href: '/history', label: '历史', icon: History },
		{ href: '/settings', label: '设置', icon: Settings }
	];

	function isActive(href: string, pathname: string): boolean {
		if (href === '/') return pathname === '/';
		return pathname.startsWith(href);
	}

	onMount(() => {
		themeStore.init();
	});
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>

<div class="flex min-h-screen flex-col">
	<main class="flex-grow pb-20">
		{@render children()}
	</main>

	<nav class="fixed right-0 bottom-0 left-0 z-50 border-t bg-background">
		<div class="container mx-auto px-4">
			<div class="flex h-16 items-center justify-around">
				{#each navItems as item (item.href)}
					{@const active = isActive(item.href, page.url.pathname)}
					<a
						href={item.href}
						class="flex flex-col items-center justify-center gap-1 px-3 py-2 transition-colors
							{active ? 'text-primary' : 'text-muted-foreground hover:text-foreground'}"
					>
						<item.icon class="h-5 w-5" />
						<span class="text-xs">{item.label}</span>
					</a>
				{/each}
			</div>
		</div>
	</nav>
</div>

<div style="display:none">
	{#each locales as locale (locale)}
		<a href={resolve(localizeHref(page.url.pathname, { locale }) as Pathname)}>{locale}</a>
	{/each}
</div>

<Sonner />
