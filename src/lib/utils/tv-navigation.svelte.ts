export type FocusNodeId = string;

export interface FocusNode {
	left?: FocusNodeId;
	right?: FocusNodeId;
	up?: FocusNodeId;
	down?: FocusNodeId;
}

export interface PageContentGraph {
	defaultNode: FocusNodeId;
	nodes: Record<FocusNodeId, FocusNode>;
}

export interface TvnavigationState {
	focusedNodeId: FocusNodeId;
	overlayActive: boolean;
}

const SIDEBAR_COUNT = 6;

export function createTvnavigation() {
	let state = $state<TvnavigationState>({
		focusedNodeId: 'sidebar:0',
		overlayActive: false
	});

	let pageContentGraphs = $state<Record<string, PageContentGraph>>({});
	let currentPageContentId = $state<string>('');

	function handleKeydown(e: KeyboardEvent) {
		if (state.overlayActive) return;

		const currentNodeId = state.focusedNodeId;
		const isSidebar = currentNodeId.startsWith('sidebar:');

		let nextNodeId: FocusNodeId | undefined;

		switch (e.key) {
			case 'ArrowUp':
				e.preventDefault();
				nextNodeId = findNextNode(currentNodeId, 'up', isSidebar);
				break;
			case 'ArrowDown':
				e.preventDefault();
				nextNodeId = findNextNode(currentNodeId, 'down', isSidebar);
				break;
			case 'ArrowLeft':
				e.preventDefault();
				nextNodeId = findNextNode(currentNodeId, 'left', isSidebar);
				break;
			case 'ArrowRight':
				e.preventDefault();
				nextNodeId = findNextNode(currentNodeId, 'right', isSidebar);
				break;
			case 'Enter':
			case ' ':
				e.preventDefault();
				triggerEnter(currentNodeId);
				break;
		}

		if (nextNodeId && nextNodeId !== currentNodeId) {
			state.focusedNodeId = nextNodeId;
			scrollIntoView(nextNodeId);
		}
	}

	function scrollIntoView(nodeId: FocusNodeId) {
		if (typeof document === 'undefined') return;
		const el = document.querySelector(`[data-tv-node="${nodeId}"]`) as HTMLElement;
		if (!el) return;
		el.scrollIntoView({ behavior: 'smooth', block: 'center', inline: 'center' });
	}

	function findNextNode(
		fromNodeId: FocusNodeId,
		direction: 'up' | 'down' | 'left' | 'right',
		isSidebar: boolean
	): FocusNodeId | undefined {
		if (isSidebar) {
			return findSidebarNext(fromNodeId, direction);
		} else {
			return findContentNext(fromNodeId, direction);
		}
	}

	function findSidebarNext(
		fromNodeId: FocusNodeId,
		direction: 'up' | 'down' | 'left' | 'right'
	): FocusNodeId | undefined {
		const index = parseInt(fromNodeId.split(':')[1], 10);

		if (direction === 'up') {
			return index > 0 ? `sidebar:${index - 1}` : undefined;
		}
		if (direction === 'down') {
			return index < SIDEBAR_COUNT - 1 ? `sidebar:${index + 1}` : undefined;
		}
		if (direction === 'right') {
			if (currentPageContentId && pageContentGraphs[currentPageContentId]) {
				return pageContentGraphs[currentPageContentId].defaultNode;
			}
			return undefined;
		}
		return undefined;
	}

	function findContentNext(
		fromNodeId: FocusNodeId,
		direction: 'up' | 'down' | 'left' | 'right'
	): FocusNodeId | undefined {
		const graph = pageContentGraphs[currentPageContentId];
		if (!graph) return undefined;

		const currentNode = graph.nodes[fromNodeId];
		if (!currentNode) {
			return graph.defaultNode;
		}

		const target = currentNode[direction];
		if (target) {
			return target;
		}

		return undefined;
	}

	function triggerEnter(nodeId: FocusNodeId) {
		const selector = `[data-tv-node="${nodeId}"]`;
		if (typeof document !== 'undefined') {
			const el = document.querySelector(selector) as HTMLElement;
			if (el) {
				el.click();
			}
		}
	}

	function registerPageContentGraph(pageId: string, graph: PageContentGraph) {
		pageContentGraphs[pageId] = graph;
	}

	function setCurrentPageContent(pageId: string) {
		currentPageContentId = pageId;
	}

	function getCurrentPageContent(): string {
		return currentPageContentId;
	}

	function setOverlayActive(active: boolean) {
		state.overlayActive = active;
	}

	function isOverlayActive(): boolean {
		return state.overlayActive;
	}

	function resetFocus() {
		if (currentPageContentId && pageContentGraphs[currentPageContentId]) {
			state.focusedNodeId = pageContentGraphs[currentPageContentId].defaultNode;
		}
	}

	return {
		get state() {
			return state;
		},
		handleKeydown,
		registerPageContentGraph,
		setCurrentPageContent,
		getCurrentPageContent,
		setOverlayActive,
		isOverlayActive,
		resetFocus,
		scrollIntoView
	};
}

export const tvNav = createTvnavigation();

export type Tvnavigation = ReturnType<typeof createTvnavigation>;
