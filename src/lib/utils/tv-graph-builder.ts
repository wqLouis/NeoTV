import { browser } from '$app/environment';

export interface FocusNode {
	left?: string;
	right?: string;
	up?: string;
	down?: string;
}

export interface PageContentGraph {
	defaultNode: string;
	nodes: Record<string, FocusNode>;
}

export interface DiscoveredNode {
	nodeId: string;
	group: string;
	index: number;
	element: HTMLElement;
}

interface GraphBuilderOptions {
	pageId: string;
	rootSelector?: string;
	defaultNode?: string;
}

const KNOWN_PATTERNS = [
	{ attr: 'data-tv-card', group: 'grid' },
	{ attr: 'data-tv-tab', group: 'tabs' },
	{ attr: 'data-tv-genre', group: 'genres' },
	{ attr: 'data-tv-node', group: 'custom' }
];

export function buildPageGraph(options: GraphBuilderOptions): PageContentGraph {
	if (!browser) {
		return { defaultNode: '', nodes: {} };
	}

	const root = options.rootSelector ? document.querySelector(options.rootSelector) : document.body;

	if (!root) {
		return { defaultNode: '', nodes: {} };
	}

	const nodes = discoverNodes(root);
	const groups = groupNodes(nodes);

	const graph = buildGraphFromGroups(groups, options.pageId);

	if (options.defaultNode) {
		graph.defaultNode = options.defaultNode;
	} else if (!graph.defaultNode) {
		graph.defaultNode = findDefaultNode(groups) || '';
	}

	return graph;
}

function discoverNodes(root: Element): DiscoveredNode[] {
	const discovered: DiscoveredNode[] = [];

	for (const pattern of KNOWN_PATTERNS) {
		const elements = root.querySelectorAll(`[${pattern.attr}]`);

		elements.forEach((el) => {
			const attrValue = el.getAttribute(pattern.attr);
			if (attrValue === null) return;

			const group = pattern.group;
			let index: number;
			let nodeId: string;

			if (pattern.attr === 'data-tv-node') {
				nodeId = attrValue;
				index = parseInt((el as HTMLElement).dataset.tvIndex || '0', 10);
			} else {
				const parsed = parseInt(attrValue, 10);
				index = isNaN(parsed) ? discovered.filter((n) => n.group === group).length : parsed;
				nodeId = `${group === 'grid' ? getPageId(root) : getPageId(root)}:${pattern.group.replace('s', '').replace('grid', 'card')}:${index}`;

				if (pattern.group === 'grid') {
					nodeId = `${getPageId(root)}:card:${index}`;
				} else if (pattern.group === 'tabs') {
					nodeId = `${getPageId(root)}:tab:${index}`;
				} else if (pattern.group === 'genres') {
					nodeId = `${getPageId(root)}:genre:${index}`;
				}
			}

			if (!discovered.some((n) => n.nodeId === nodeId)) {
				discovered.push({ nodeId, group: pattern.group, index, element: el as HTMLElement });
			}
		});
	}

	return discovered;
}

function getPageId(root: Element): string {
	const pageEl = root.closest('[data-tv-page]');
	return pageEl?.getAttribute('data-tv-page') || 'page';
}

function groupNodes(nodes: DiscoveredNode[]): Map<string, DiscoveredNode[]> {
	const groups = new Map<string, DiscoveredNode[]>();

	for (const node of nodes) {
		const existing = groups.get(node.group) || [];
		existing.push(node);
		groups.set(node.group, existing);
	}

	for (const [_, groupNodes] of groups) {
		groupNodes.sort((a, b) => a.index - b.index);
	}

	return groups;
}

function findDefaultNode(groups: Map<string, DiscoveredNode[]>): string | undefined {
	const defaultNode = [...groups.values()]
		.flat()
		.find((n) => n.element.hasAttribute('data-tv-default'));

	if (defaultNode) {
		return defaultNode.nodeId;
	}

	const gridGroup = groups.get('grid');
	if (gridGroup && gridGroup.length > 0) {
		return gridGroup[0].nodeId;
	}

	const tabsGroup = groups.get('tabs');
	if (tabsGroup && tabsGroup.length > 0) {
		return tabsGroup[0].nodeId;
	}

	const genresGroup = groups.get('genres');
	if (genresGroup && genresGroup.length > 0) {
		return genresGroup[0].nodeId;
	}

	return undefined;
}

function buildGraphFromGroups(
	groups: Map<string, DiscoveredNode[]>,
	pageId: string
): PageContentGraph {
	const graph: PageContentGraph = {
		defaultNode: '',
		nodes: {}
	};

	const gridNodes = groups.get('grid') || [];
	const tabNodes = groups.get('tabs') || [];
	const genreNodes = groups.get('genres') || [];
	const customNodes = groups.get('custom') || [];

	const tabCount = tabNodes.length;
	const genreCount = genreNodes.length;
	const cols = inferGridCols(groups);

	for (let i = 0; i < tabNodes.length; i++) {
		const node = tabNodes[i];
		const isLast = i === tabCount - 1;
		const isFirst = i === 0;

		graph.nodes[node.nodeId] = {
			left: isFirst ? 'sidebar:2' : tabNodes[i - 1].nodeId,
			right: isLast
				? genreCount > 0
					? genreNodes[0].nodeId
					: gridNodes.length > 0
						? gridNodes[0].nodeId
						: undefined
				: tabNodes[i + 1].nodeId,
			down:
				genreCount > 0
					? genreNodes[0].nodeId
					: gridNodes.length > 0
						? gridNodes[0].nodeId
						: undefined
		};
	}

	for (let i = 0; i < genreNodes.length; i++) {
		const node = genreNodes[i];
		const isLast = i === genreCount - 1;
		const isFirst = i === 0;

		const leftNode = isFirst
			? tabCount > 0
				? tabNodes[Math.min(i, tabCount - 1)].nodeId
				: 'sidebar:2'
			: genreNodes[i - 1].nodeId;

		const rightNode = isLast
			? gridNodes.length > 0
				? gridNodes[0].nodeId
				: undefined
			: genreNodes[i + 1].nodeId;

		const downIndex = i * cols;
		const downNode = gridNodes[Math.min(downIndex, gridNodes.length - 1)]?.nodeId;

		graph.nodes[node.nodeId] = {
			left: leftNode,
			right: rightNode,
			down: downNode
		};
	}

	for (let i = 0; i < gridNodes.length; i++) {
		const node = gridNodes[i];
		const row = Math.floor(i / cols);
		const col = i % cols;
		const isLastInRow = col === cols - 1 || i === gridNodes.length - 1;
		const isFirstInRow = col === 0 || i === 0;

		const leftNodeId = isFirstInRow
			? genreCount > 0
				? genreNodes[Math.min(col, genreCount - 1)].nodeId
				: undefined
			: gridNodes[i - 1].nodeId;

		const rightNodeId = isLastInRow ? undefined : gridNodes[i + 1].nodeId;

		const upRow = row - 1;
		const upNodeId =
			upRow < 0
				? genreCount > 0
					? genreNodes[Math.min(col, genreCount - 1)].nodeId
					: undefined
				: gridNodes[Math.min(upRow * cols + col, gridNodes.length - 1)]?.nodeId;

		const downRow = row + 1;
		const downNodeId =
			downRow * cols + col < gridNodes.length ? gridNodes[downRow * cols + col].nodeId : undefined;

		graph.nodes[node.nodeId] = {
			left: leftNodeId,
			right: rightNodeId,
			up: upNodeId,
			down: downNodeId
		};
	}

	for (const node of customNodes) {
		if (!graph.nodes[node.nodeId]) {
			graph.nodes[node.nodeId] = {};
		}
	}

	if (tabCount > 0 && !graph.defaultNode) {
		graph.defaultNode = tabNodes[0].nodeId;
	} else if (genreCount > 0 && !graph.defaultNode) {
		graph.defaultNode = genreNodes[0].nodeId;
	} else if (gridNodes.length > 0 && !graph.defaultNode) {
		graph.defaultNode = gridNodes[0].nodeId;
	}

	return graph;
}

function inferGridCols(groups: Map<string, DiscoveredNode[]>): number {
	const gridNodes = groups.get('grid');
	if (!gridNodes || gridNodes.length < 2) {
		return 6;
	}

	const indices = gridNodes.map((n) => n.index).sort((a, b) => a - b);
	const diffs = new Set<number>();

	for (let i = 1; i < indices.length; i++) {
		const diff = indices[i] - indices[i - 1];
		if (diff > 0 && diff < 100) {
			diffs.add(diff);
		}
	}

	if (diffs.size > 0) {
		const sortedDiffs = [...diffs].sort((a, b) => a - b);
		return sortedDiffs[Math.floor(sortedDiffs.length / 2)];
	}

	return 6;
}

export function buildSimpleGridGraph(
	pageId: string,
	itemCount: number,
	cols: number,
	options?: {
		startIndex?: number;
		itemPrefix?: string;
		defaultNode?: string;
	}
): PageContentGraph {
	const graph: PageContentGraph = {
		defaultNode: options?.defaultNode || `${pageId}:card:${options?.startIndex || 0}`,
		nodes: {}
	};

	const start = options?.startIndex || 0;
	const prefix = options?.itemPrefix || `${pageId}:card`;

	for (let i = 0; i < itemCount; i++) {
		const nodeId = `${prefix}:${i + start}`;
		const row = Math.floor(i / cols);
		const col = i % cols;

		const isFirstRow = row === 0;
		const isLastRow = row === Math.floor((itemCount - 1) / cols);
		const isFirstCol = col === 0;
		const isLastCol = col === cols - 1 || i === itemCount - 1;

		graph.nodes[nodeId] = {
			left: isFirstCol ? undefined : `${prefix}:${i - 1}`,
			right: isLastCol ? undefined : `${prefix}:${i + 1}`,
			up: isFirstRow ? undefined : `${prefix}:${i - cols}`,
			down: isLastRow ? undefined : `${prefix}:${i + cols}`
		};
	}

	return graph;
}

export function buildListGraph(
	pageId: string,
	itemCount: number,
	options?: {
		startIndex?: number;
		itemPrefix?: string;
		defaultNode?: string;
	}
): PageContentGraph {
	const graph: PageContentGraph = {
		defaultNode: options?.defaultNode || `${pageId}:item:${options?.startIndex || 0}`,
		nodes: {}
	};

	const start = options?.startIndex || 0;
	const prefix = options?.itemPrefix || `${pageId}:item`;

	for (let i = 0; i < itemCount; i++) {
		const nodeId = `${prefix}:${i + start}`;
		const isFirst = i === 0;
		const isLast = i === itemCount - 1;

		graph.nodes[nodeId] = {
			up: isFirst ? undefined : `${prefix}:${i - 1 + start}`,
			down: isLast ? undefined : `${prefix}:${i + 1 + start}`
		};
	}

	return graph;
}
