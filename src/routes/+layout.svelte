<script lang="ts">
	import type { Pathname } from '$app/types';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { locales, localizeHref } from '$lib/paraglide/runtime';
	import { onMount } from 'svelte';
	import { Home, Search, History, Heart, Settings, LayoutGrid } from '@lucide/svelte';
	import Sonner from '$lib/components/ui/sonner/sonner.svelte';
	import { themeStore } from '$lib/stores/theme.svelte';
	import { modalStore } from '$lib/stores/modal.svelte';
	import Modal from '$lib/components/Modal.svelte';
	import FocusRing from '$lib/nav-graph/FocusRing.svelte';
	import { activeNavNode } from '$lib/nav-graph/store';
	import { BOUNDARY, rootNode, type NavDir, type NavNode } from '$lib/nav-graph/navGraph';
	import './layout.css';

	import HomePage from './+page.svelte';
	import SearchPage from './search/+page.svelte';
	import BrowsePage from './browse/+page.svelte';
	import HistoryPage from './history/+page.svelte';
	import FavouritesPage from './favourites/+page.svelte';
	import SettingsPage from './settings/+page.svelte';

	interface NavPageInstance {
		buildNavNode?: () => NavNode | null;
	}

	let { children } = $props();
	let focusRing: FocusRing;

	let homePage = $state<NavPageInstance | undefined>(undefined);
	let searchPage = $state<NavPageInstance | undefined>(undefined);
	let browsePage = $state<NavPageInstance | undefined>(undefined);
	let historyPage = $state<NavPageInstance | undefined>(undefined);
	let favouritesPage = $state<NavPageInstance | undefined>(undefined);
	let settingsPage = $state<NavPageInstance | undefined>(undefined);

	const upperNav = [{ href: '/search', label: '搜索', icon: Search }];
	const lowerNav = [
		{ href: '/', label: '首页', icon: Home },
		{ href: '/browse', label: '浏览', icon: LayoutGrid },
		{ href: '/history', label: '历史', icon: History },
		{ href: '/favourites', label: '收藏', icon: Heart },
		{ href: '/settings', label: '设置', icon: Settings }
	];

	function isActive(href: string, pathname: string): boolean {
		if (href === '/') return pathname === '/';
		return pathname.startsWith(href);
	}

	function getPageName(pathname: string): string {
		switch (pathname) {
			case '/':
				return 'home';
			case '/browse':
				return 'browse';
			case '/history':
				return 'history';
			case '/favourites':
				return 'favourites';
			case '/settings':
				return 'settings';
			case '/search':
				return 'search';
			default:
				return 'unknown';
		}
	}

	function initNavNode() {
		let pageInstance: NavPageInstance | undefined;
		let pathname = page.url.pathname;

		if (pathname === '/') pageInstance = homePage;
		else if (pathname === '/search') pageInstance = searchPage;
		else if (pathname === '/browse') pageInstance = browsePage;
		else if (pathname === '/history') pageInstance = historyPage;
		else if (pathname === '/favourites') pageInstance = favouritesPage;
		else if (pathname === '/settings') pageInstance = settingsPage;

		if (pageInstance?.buildNavNode) {
			const pageNavNode = pageInstance.buildNavNode();
			if (pageNavNode) {
				const root = new rootNode(pageNavNode, pageNavNode.navGraph);
				activeNavNode.set(root);
			}
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		const nav = $activeNavNode;
		if (!nav || !focusRing) return;

		const dirMap: Record<string, NavDir> = {
			ArrowUp: 'UP',
			ArrowDown: 'DOWN',
			ArrowLeft: 'LEFT',
			ArrowRight: 'RIGHT'
		};
		const dir = dirMap[e.key];
		if (!dir) return;

		e.preventDefault();
		const result = nav.move(dir);
		if (result !== BOUNDARY) {
			focusRing.updateFocus(result);
		}
	}

	onMount(async () => {
		themeStore.init();
		modalStore.init();
		await modalStore.checkGstLibav();
		initNavNode();
	});
</script>

<svelte:head><link rel="icon" type="image/png" href="/favicon.png" /></svelte:head>

<div class="flex h-screen">
	<nav class="fixed top-0 left-0 z-50 flex h-full w-20 flex-col border-r bg-card py-4">
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
							>{item.label}</span
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
							>{item.label}</span
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

<div class="hidden">
	{#each locales as locale (locale)}
		<a href={resolve(localizeHref(page.url.pathname, { locale }) as Pathname)}>{locale}</a>
	{/each}
</div>

<svelte:window onkeydown={handleKeydown} />
<FocusRing bind:this={focusRing} />

<Sonner />
<Modal />
