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
