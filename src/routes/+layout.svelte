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

<div class="flex h-screen">
	<nav class="fixed top-0 left-0 z-50 flex h-full w-20 flex-col border-r bg-card">
		<div class="flex flex-1 flex-col items-center justify-center gap-2 py-4">
			{#each navItems as item (item.href)}
				{@const active = isActive(item.href, page.url.pathname)}
				<a
					href={item.href}
					tabindex="0"
					class="flex flex-col items-center justify-center gap-1 rounded-lg px-3 py-4 transition-all
						{active
						? 'bg-primary/10 text-primary ring-2 ring-primary'
						: 'text-muted-foreground hover:bg-muted hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2'}"
				>
					<item.icon class="h-7 w-7" />
					<span class="text-sm">{item.label}</span>
				</a>
			{/each}
		</div>
	</nav>

	<main class="ml-20 flex-1 overflow-y-auto">
		{@render children()}
	</main>
</div>

<div style="display:none">
	{#each locales as locale (locale)}
		<a href={resolve(localizeHref(page.url.pathname, { locale }) as Pathname)}>{locale}</a>
	{/each}
</div>

<Sonner />
