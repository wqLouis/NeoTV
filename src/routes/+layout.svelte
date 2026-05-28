<script lang="ts">
	import { page } from '$app/state';
	import { onMount } from 'svelte';
	import { Home, Search, History, Heart, Settings, LayoutGrid } from '@lucide/svelte';
	import Sonner from '$lib/components/ui/sonner/sonner.svelte';
	import { themeStore } from '$lib/stores/theme.svelte';
	import { modalStore } from '$lib/stores/modal.svelte';
	import { settingsStore } from '$lib/stores/settings.svelte';
	import Modal from '$lib/components/Modal.svelte';
	import FocusRing from '$lib/components/FocusRing.svelte';
	import { getNextFocus } from '@bbc/tv-lrud-spatial';
	import TrafficLights from '$lib/components/TrafficLights.svelte';
	import { platform } from '@tauri-apps/plugin-os';
	import { isTauri } from '@tauri-apps/api/core';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import './layout.css';

	let { children } = $props();

	// Show traffic lights only on Windows and Linux (not macOS)
	let showTrafficLights = $state(false);

	const navLabels = {
		search: '搜索',
		home: '首页',
		browse: '浏览',
		history: '历史',
		favourites: '收藏',
		settings: '设置'
	};

	onMount(async () => {
		if (isTauri()) {
			const os = await platform();
			showTrafficLights = os === 'windows' || os === 'linux';
		}
		themeStore.init();
		modalStore.init();
		await modalStore.checkGstLibav();
	});

	const upperNav = [
		{ href: '/search', key: 'search' as const, icon: Search }
	];
	const lowerNav = [
		{ href: '/', key: 'home' as const, icon: Home },
		{ href: '/browse', key: 'browse' as const, icon: LayoutGrid },
		{ href: '/history', key: 'history' as const, icon: History },
		{ href: '/favourites', key: 'favourites' as const, icon: Heart },
		{ href: '/settings', key: 'settings' as const, icon: Settings }
	];

	function isActive(href: string, pathname: string): boolean {
		if (href === '/') return pathname === '/';
		return pathname.startsWith(href);
	}

	function handleKeydown(e: KeyboardEvent) {
		if (!settingsStore.tvNavModeEnabled) return;

		const dir = e.key;
		if (dir !== 'ArrowUp' && dir !== 'ArrowDown' && dir !== 'ArrowLeft' && dir !== 'ArrowRight')
			return;

		e.preventDefault();
		const next = getNextFocus(document.activeElement, dir as any);
		if (next) {
			next.focus();
		}
	}

	function startNavDrag(e: MouseEvent) {
		// Don't start drag if clicking on interactive elements
		const target = e.target as HTMLElement;
		if (target.closest('a, button, input, [role="toolbar"]')) return;
		if (!isTauri()) return;
		getCurrentWindow().startDragging().catch(() => {});
	}
</script>

<svelte:head>
	<link rel="icon" type="image/png" href="/favicon.png" />
	<link rel="shortcut icon" type="image/png" href="/favicon.png" />
</svelte:head>

<div class="flex h-screen">
	<nav
		class="fixed top-0 left-0 z-50 flex h-full w-20 flex-col border-r bg-card py-4 cursor-grab active:cursor-grabbing"
		onmousedown={startNavDrag}
		role="banner"
		aria-label="导航栏，可拖动窗口"
	>
		{#if showTrafficLights}
			<div class="mb-2">
				<TrafficLights />
			</div>
		{/if}

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
						<span
							class="text-xs transition-all duration-200 {active ? 'opacity-100' : 'h-0 opacity-0'}"
							>{navLabels[item.key]}</span
						>
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
						<span
							class="text-xs transition-all duration-200 {active ? 'opacity-100' : 'h-0 opacity-0'}"
							>{navLabels[item.key]}</span
						>
					</a>
				{/each}
			</div>
		</div>
	</nav>

	<main class="ml-20 flex-1 overflow-y-auto">
		{@render children()}
	</main>
</div>

<svelte:window onkeydown={handleKeydown} />
{#if settingsStore.tvNavModeEnabled}
	<FocusRing />
{/if}

<Sonner />
<Modal />
