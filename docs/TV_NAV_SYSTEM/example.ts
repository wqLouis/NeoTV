type NavDir = 'UP' | 'DOWN' | 'RIGHT' | 'LEFT';
type MoveResult = 'BOUNDARY' | 'OK';
type Leaf = 'LEAF';

class rootNode {
	focusNode: NavNode;
	navGraph: Map<NavNode, Map<NavDir, NavNode>>;

	constructor(focusNode: NavNode, navGraph: Map<NavNode, Map<NavDir, NavNode>>) {
		if (navGraph.size <= 0) {
			throw Error('root node should have at least one child');
		}

		this.focusNode = focusNode;
		this.navGraph = navGraph;
	}

	// this is the only entry point for all user input
	public move(dir: NavDir): MoveResult {
		const result = this.focusNode.move(dir);

		if (result === 'OK') {
			return 'OK';
		}

		const focus = this.navGraph.get(this.focusNode)?.get(dir);

		if (focus !== undefined) {
			this.focusNode = focus;
			return 'OK';
		}

		return 'BOUNDARY';
	}
}

class NavNode {
	public element: HTMLElement;
	focusNode: NavNode | 'LEAF';
	navGraph: Map<NavNode, Map<NavDir, NavNode>>; // Child node -> (direction, node connected)

	constructor(
		element: HTMLElement,
		focusElement: NavNode | 'LEAF',
		navGraph: Map<NavNode, Map<NavDir, NavNode>>
	) {
		this.element = element;
		this.focusNode = focusElement;
		this.navGraph = navGraph;
	}

	public move(dir: NavDir): MoveResult {
		if (this.focusNode === 'LEAF') {
			return 'BOUNDARY';
		}

		const result = this.focusNode.move(dir);

		if (result === 'OK') {
			return 'OK';
		}

		const focus = this.navGraph.get(this.focusNode)?.get(dir);

		if (focus !== undefined) {
			this.focusNode = focus;
			return 'OK';
		}

		return 'BOUNDARY';
	}
}
