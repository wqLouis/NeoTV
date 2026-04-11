<script lang="ts">
	import type { Pathname } from '$app/types';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { locales, localizeHref } from '$lib/paraglide/runtime';
	import { onMount } from 'svelte';
	import { Home, Search, History, Heart, Settings } from 'lucide-svelte';
	import Sonner from '$lib/components/ui/sonner/sonner.svelte';
	import { themeStore } from '$lib/stores/theme.svelte';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';
	import { fly } from 'svelte/transition';

	let { children } = $props();

	const upperNav = [{ href: '/search', label: '搜索', icon: Search }];
	const lowerNav = [
		{ href: '/', label: '首页', icon: Home },
		{ href: '/history', label: '历史', icon: History },
		{ href: '/favourites', label: '收藏', icon: Heart },
		{ href: '/settings', label: '设置', icon: Settings }
	];

	function isActive(href: string, pathname: string): boolean {
		if (href === '/') return pathname === '/';
		return pathname.startsWith(href);
	}

	function isMobileDevice(): boolean {
		if (typeof navigator === 'undefined') return false;
		const ua = navigator.userAgent;
		return /android|webos|iphone|ipad|ipod|blackberry|iemobile|opera mini/i.test(ua);
	}

	async function enableFullscreen() {
		if (!isMobileDevice()) return;
		try {
			const appWindow = getCurrentWindow();
			await appWindow.setFullscreen(true);
		} catch {}
	}

	async function enableFullscreenWithRetry(retries = 3) {
		await enableFullscreen();
		if (isMobileDevice()) {
			for (let i = 0; i < retries; i++) {
				await new Promise((r) => setTimeout(r, 100));
				await enableFullscreen();
			}
		}
	}

	onMount(async () => {
		themeStore.init();
		await enableFullscreenWithRetry();
	});
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>

<div class="flex h-screen">
	<nav class="fixed top-0 left-0 z-50 flex h-full w-20 flex-col border-r bg-card">
		<div class="flex flex-1 flex-col items-center justify-center gap-2">
			<div class="flex flex-col items-center gap-2 py-2">
				{#each upperNav as item (item.href)}
					{@const active = isActive(item.href, page.url.pathname)}
					<a
						href={item.href}
						class="flex flex-col items-center justify-center gap-1 transition-all
							{active ? 'text-primary' : 'text-muted-foreground hover:text-foreground'}"
					>
						<div
							class="flex aspect-square w-12 items-center justify-center rounded-lg transition-all
								{active ? 'bg-primary/10' : ''}"
						>
							<item.icon class="h-7 w-7" />
						</div>
						{#if active}
							<span in:fly={{ y: 10, duration: 200 }} class="text-xs">{item.label}</span>
						{/if}
					</a>
				{/each}
			</div>

			<div class="flex-1"></div>

			<div class="flex flex-col items-center gap-2 py-2">
				{#each lowerNav as item (item.href)}
					{@const active = isActive(item.href, page.url.pathname)}
					<a
						href={item.href}
						class="group flex flex-col items-center justify-center gap-1 transition-all
							{active ? 'text-primary' : 'text-muted-foreground hover:text-foreground'}"
					>
						<div
							class="flex aspect-square w-12 items-center justify-center rounded-lg transition-all
								{active ? 'bg-primary/10' : ''}"
						>
							<item.icon
								class="h-7 w-7 transition-transform duration-200 {active
									? 'scale-110'
									: 'scale-100'}"
							/>
						</div>
						{#if active}
							<span in:fly={{ y: 10, duration: 200 }} class="text-xs">{item.label}</span>
						{/if}
					</a>
				{/each}
			</div>
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
