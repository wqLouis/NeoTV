import { SvelteMap } from 'svelte/reactivity';

export type NavDir = 'UP' | 'DOWN' | 'RIGHT' | 'LEFT';
export const BOUNDARY = 'BOUNDARY' as const;
export type MoveResult = HTMLElement | typeof BOUNDARY;
export const LEAF = 'LEAF' as const;

export class rootNode {
	focusNode: NavNode;
	navGraph: SvelteMap<NavNode, SvelteMap<NavDir, NavNode>>;

	constructor(focusNode: NavNode, navGraph: SvelteMap<NavNode, SvelteMap<NavDir, NavNode>>) {
		if (navGraph.size <= 0) {
			throw Error('root node should have at least one child');
		}

		this.focusNode = focusNode;
		this.navGraph = navGraph;
	}

	public move(dir: NavDir): MoveResult {
		const result = this.focusNode.move(dir);

		if (result !== BOUNDARY) {
			console.log(`[root] ${dir}:`, result, '✓');
			return result;
		}

		const focusMap = this.navGraph.get(this.focusNode);
		const focus = focusMap?.get(dir);

		if (focus !== undefined) {
			this.focusNode = focus;
			let current: NavNode | typeof LEAF = focus;
			while (typeof current !== 'string' && current.focusNode !== LEAF) {
				current = current.focusNode as NavNode;
			}
			const leafElement =
				typeof current === 'string' ? (current as unknown as NavNode).element : current.element;
			console.log(`[root] ${dir}:`, leafElement, '(via graph)');
			return leafElement;
		}

		console.log(`[root] ${dir}: BOUNDARY`);
		return BOUNDARY;
	}
}

export class NavNode {
	element: HTMLElement;
	focusNode: NavNode | typeof LEAF;
	navGraph: SvelteMap<NavNode, SvelteMap<NavDir, NavNode>>;

	constructor(
		element: HTMLElement,
		focusElement: NavNode | typeof LEAF,
		navGraph: SvelteMap<NavNode, SvelteMap<NavDir, NavNode>>
	) {
		this.element = element;
		this.focusNode = focusElement;
		this.navGraph = navGraph;
	}

	public move(dir: NavDir): MoveResult {
		if (this.focusNode === LEAF) {
			console.log(`[node] ${dir}: LEAF → BOUNDARY`);
			return BOUNDARY;
		}

		const result = this.focusNode.move(dir);

		if (result !== BOUNDARY) {
			console.log(`[node] ${dir}:`, result, '✓');
			return result;
		}

		const focusMap = this.navGraph.get(this.focusNode);
		const focus = focusMap?.get(dir);

		if (focus !== undefined) {
			this.focusNode = focus;
			let current: NavNode | typeof LEAF = focus;
			while (typeof current !== 'string' && current.focusNode !== LEAF) {
				current = current.focusNode as NavNode;
			}
			const leafElement =
				typeof current === 'string' ? (current as unknown as NavNode).element : current.element;
			console.log(`[node] ${dir}:`, leafElement, '(via graph)');
			return leafElement;
		}

		console.log(`[node] ${dir}: BOUNDARY`);
		return BOUNDARY;
	}
}

export type RebuildFn = () => void;

export function createNavController(
	root: rootNode,
	onBoundary: RebuildFn
): { move: (dir: NavDir) => MoveResult } {
	return {
		move(dir: NavDir): MoveResult {
			const result = root.move(dir);
			if (result === BOUNDARY) {
				onBoundary();
			}
			return result;
		}
	};
}
