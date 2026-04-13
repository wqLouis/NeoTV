import type { Direction, Focusable, MoveResult, NodeId } from './connections';

export class LeafGraph implements Focusable {
	constructor(
		public readonly nodeId: NodeId,
		public readonly element: HTMLElement
	) {}

	move(direction: Direction): MoveResult {
		return { status: 'boundary', nodeId: this.nodeId, direction };
	}

	focus(): void {
		this.element.focus();
	}
}

export class Graph {
	parent: Graph | null = null;
	childGraphs: Map<string, Graph> = new Map();
	currentNodeId: NodeId | null = null;
	nodes: Map<NodeId, Focusable> = new Map();
	connections: Map<NodeId, Partial<Record<Direction, NodeId>>> = new Map();
	defaultEntry: NodeId;
	itemCount = 0;
	cols = 0;

	constructor(
		public readonly name: string,
		defaultEntry: NodeId
	) {
		this.defaultEntry = defaultEntry;
	}

	registerNode(nodeId: NodeId, node: Focusable): void {
		this.nodes.set(nodeId, node);
	}

	registerChildGraph(prefix: string, child: Graph): void {
		child.parent = this;
		this.childGraphs.set(prefix, child);
	}

	addConnection(from: NodeId, direction: Direction, to: NodeId): void {
		const existing = this.connections.get(from) || {};
		existing[direction] = to;
		this.connections.set(from, existing);
	}

	move(direction: Direction): MoveResult {
		if (!this.currentNodeId) {
			return { status: 'none' };
		}

		const conn = this.connections.get(this.currentNodeId);
		const target = conn?.[direction];

		if (target) {
			const node = this.nodes.get(target);
			if (node) {
				node.focus();
				this.currentNodeId = target;
				return { status: 'moved', nodeId: target };
			}

			// Target is in a child graph
			const childGraph = this.getChildGraphForNode(target);
			if (childGraph) {
				childGraph.focusNode(target);
				this.currentNodeId = target;
				return { status: 'moved', nodeId: target };
			}
		}

		if (this.parent) {
			return this.parent.move(direction);
		}

		return { status: 'boundary', nodeId: this.currentNodeId, direction };
	}

	focusEntry(): void {
		const node = this.nodes.get(this.defaultEntry);
		if (node) {
			node.focus();
			this.currentNodeId = this.defaultEntry;
			return;
		}

		// Check child graphs for the default entry
		const childGraph = this.getChildGraphForNode(this.defaultEntry);
		if (childGraph) {
			childGraph.focusEntry();
			this.currentNodeId = this.defaultEntry;
		}
	}

	getChildGraphForNode(nodeId: NodeId): Graph | null {
		for (const [prefix, child] of this.childGraphs) {
			if (nodeId === prefix || nodeId.startsWith(prefix + ':')) {
				return child;
			}
		}
		return null;
	}

	getNode(nodeId: NodeId): Focusable | undefined {
		const node = this.nodes.get(nodeId);
		if (node) return node;

		for (const child of this.childGraphs.values()) {
			const childNode = child.getNode(nodeId);
			if (childNode) return childNode;
		}
		return undefined;
	}

	focusNode(nodeId: NodeId): void {
		const node = this.nodes.get(nodeId);
		if (node) {
			node.focus();
			this.currentNodeId = nodeId;
			return;
		}

		// Check child graphs
		const childGraph = this.getChildGraphForNode(nodeId);
		if (childGraph) {
			childGraph.focusNode(nodeId);
			this.currentNodeId = nodeId;
		}
	}
}
