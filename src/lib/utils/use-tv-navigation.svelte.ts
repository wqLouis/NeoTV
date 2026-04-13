import { onMount, onDestroy } from 'svelte';
import { browser } from '$app/environment';
import { tvNav } from './tv-navigation.svelte';
import { buildPageGraph, type PageContentGraph } from './tv-graph-builder';

export interface TVNavOptions {
	pageId: string;
	rootSelector?: string;
	autoRegister?: boolean;
	watchForChanges?: boolean;
}

export function useTVNavigation(options: TVNavOptions) {
	const {
		pageId,
		rootSelector = '[data-tv-page]',
		autoRegister = true,
		watchForChanges = false
	} = options;

	let containerRef = $state<HTMLElement | null>(null);
	let isRegistered = $state(false);
	let observer: MutationObserver | null = null;

	function registerGraph() {
		if (!browser || !containerRef || isRegistered) return;

		const root =
			rootSelector === '[data-tv-page]'
				? containerRef.closest(rootSelector) || containerRef
				: containerRef.querySelector(rootSelector) || containerRef;

		const pageAttr = rootSelector === '[data-tv-page]' ? root.getAttribute('data-tv-page') : null;

		const effectivePageId = pageAttr || pageId;

		const graph = buildPageGraph({
			pageId: effectivePageId,
			rootSelector: undefined,
			defaultNode: undefined
		});

		if (Object.keys(graph.nodes).length === 0) {
			return;
		}

		tvNav.registerPageContentGraph(effectivePageId, graph);
		tvNav.setCurrentPageContent(effectivePageId);
		isRegistered = true;
	}

	function unregisterGraph() {
		if (!browser) return;
		tvNav.registerPageContentGraph(pageId, { defaultNode: '', nodes: {} });
		isRegistered = false;
	}

	function handleRouteChange() {
		if (!autoRegister) return;

		isRegistered = false;
		registerGraph();
	}

	$effect(() => {
		if (containerRef && autoRegister && !isRegistered) {
			registerGraph();
		}
	});

	$effect(() => {
		if (!browser || !watchForChanges || !containerRef) return;

		observer = new MutationObserver(() => {
			if (!isRegistered) {
				registerGraph();
			}
		});

		const root = containerRef.querySelector(rootSelector) || containerRef;
		observer.observe(root, {
			childList: true,
			subtree: true,
			attributes: true,
			attributeFilter: [
				'data-tv-card',
				'data-tv-tab',
				'data-tv-genre',
				'data-tv-node',
				'data-tv-default'
			]
		});

		return () => {
			observer?.disconnect();
			observer = null;
		};
	});

	onDestroy(() => {
		if (watchForChanges) {
			observer?.disconnect();
			observer = null;
		}
	});

	return {
		get containerRef() {
			return containerRef;
		},
		set containerRef(el: HTMLElement | null) {
			containerRef = el;
		},
		registerGraph,
		unregisterGraph,
		handleRouteChange
	};
}

export function registerPageGraph(pageId: string, graph: PageContentGraph) {
	if (!browser) return;
	tvNav.registerPageContentGraph(pageId, graph);
}

export function setActivePage(pageId: string) {
	if (!browser) return;
	tvNav.setCurrentPageContent(pageId);
}

export function resetFocus() {
	if (!browser) return;
	tvNav.resetFocus();
}

export { tvNav };
export type { PageContentGraph, FocusNode } from './tv-graph-builder';
