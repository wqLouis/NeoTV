import type { Direction, NodeId } from './connections';
import { Graph } from './graph';

export class FocusNavigator {
	root: Graph | null = null;
	overlayActive = false;

	handleKeydown(event: KeyboardEvent): void {
		console.log('handleKeydown', event.key);
		if (this.overlayActive) return;
		if (!this.root) {
			console.log('handleKeydown: no root');
			return;
		}

		const dir = this.keyToDirection(event.key);
		if (!dir) return;

		event.preventDefault();
		this.move(dir);
	}

	move(direction: Direction): void {
		if (!this.root) return;

		console.log('move', direction, 'from', this.root.currentNodeId);
		const result = this.root.move(direction);
		console.log('move result', result);

		if (result.status === 'boundary') {
			console.log('boundary at:', result.nodeId, 'direction:', result.direction);
		}
	}

	setOverlayActive(active: boolean): void {
		this.overlayActive = active;
	}

	registerRoot(graph: Graph): void {
		this.root = graph;
		this.root.focusEntry();
	}

	getCurrentNodeId(): NodeId | null {
		return this.root?.currentNodeId ?? null;
	}

	getRoot(): Graph | null {
		return this.root;
	}

	private keyToDirection(key: string): Direction | null {
		switch (key) {
			case 'ArrowUp':
				return 'top';
			case 'ArrowDown':
				return 'bottom';
			case 'ArrowLeft':
				return 'left';
			case 'ArrowRight':
				return 'right';
			default:
				return null;
		}
	}
}

export const focusNavigator = new FocusNavigator();
