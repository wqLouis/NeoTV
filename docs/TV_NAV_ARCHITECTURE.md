# TV Navigation Architecture

## Overview

A **recursive focus navigation system** for TV remote controls. Navigation is organized as a tree of graphs where child graphs can be dynamically added (e.g., when entering an overlay) and removed (when escaping).

---

## Core Types

### Direction

```typescript
type Direction = 'top' | 'bottom' | 'left' | 'right' | 'enter' | 'escape';
```

### MoveResult

```typescript
type MoveResult =
	| { status: 'moved'; nodeId: NodeId }
	| { status: 'boundary'; nodeId: NodeId; direction: Exclude<Direction, 'enter' | 'escape'> }
	| { status: 'none' };
```

### Focusable

```typescript
interface Focusable {
	focus(): void;
}
```

### DirectionHandlers

```typescript
interface DirectionHandlers {
	onEnter?: (registerChild: (child: Graph) => void) => void;
	onEscape?: () => void;
}
```

### NodeId Convention

```
"home"                              // Root content graph
"home:section:0"                   // Section container
"home:section:0:card:5"            // Card in section
"overlay:source:0"                 // Overlay node (e.g., video source)
```

---

## Classes

### LeafGraph

Wraps a single DOM element. Has no internal navigation - all direction moves return boundary except enter/escape which execute handlers.

```typescript
class LeafGraph implements Focusable {
	constructor(
		public readonly nodeId: NodeId,
		public readonly element: HTMLElement,
		private handlers?: DirectionHandlers
	) {}

	focus(): void {
		this.element.focus();
	}

	move(direction: Direction): MoveResult {
		switch (direction) {
			case 'enter':
				this.handlers?.onEnter?.((child) => {
					// Parent registers the child graph
				});
				return { status: 'moved', nodeId: this.nodeId };

			case 'escape':
				// Leaf nodes don't handle escape - bubble up to parent
				return { status: 'boundary', nodeId: this.nodeId, direction: 'escape' };

			default:
				// top/bottom/left/right - leaf has no internal nav
				return { status: 'boundary', nodeId: this.nodeId, direction };
		}
	}
}
```

### Graph

Manages a collection of nodes with directional connections. Supports child graphs attached to specific nodes.

```typescript
class Graph implements Focusable {
	// Hierarchy
	parent: Graph | null = null;
	parentNodeId: NodeId | null = null;
	childGraphs: Map<NodeId, Graph> = new Map();

	// Navigation state
	currentNodeId: NodeId | null = null;

	// Node registry
	private nodes: Map<NodeId, Focusable> = new Map();
	private connections: Map<NodeId, Partial<Record<Direction, NodeId>>> = new Map();

	// Entry point
	defaultEntry: NodeId;

	constructor(name: string, defaultEntry: NodeId) {
		this.name = name;
		this.defaultEntry = defaultEntry;
	}

	// --- Registration ---

	registerNode(nodeId: NodeId, node: Focusable): void {
		this.nodes.set(nodeId, node);
	}

	addConnection(from: NodeId, direction: Direction, to: NodeId): void {
		const existing = this.connections.get(from) || {};
		existing[direction] = to;
		this.connections.set(from, existing);
	}

	// --- Navigation ---

	focus(): void {
		this.focusEntry();
	}

	focusEntry(): void {
		const node = this.nodes.get(this.defaultEntry);
		if (node) {
			node.focus();
			this.currentNodeId = this.defaultEntry;
		}
	}

	focusNode(nodeId: NodeId): void {
		const node = this.nodes.get(nodeId);
		if (node) {
			node.focus();
			this.currentNodeId = nodeId;
		}
	}

	move(direction: Direction): MoveResult {
		// No current node - can't navigate
		if (!this.currentNodeId) {
			return { status: 'none' };
		}

		// Escape: return to parent graph
		if (direction === 'escape') {
			if (this.parent && this.parentNodeId) {
				this.parent.focusNode(this.parentNodeId);
				return { status: 'moved', nodeId: this.parentNodeId };
			}
			return { status: 'boundary', nodeId: this.currentNodeId, direction: 'escape' };
		}

		// Enter: check if current node has a child graph
		if (direction === 'enter') {
			const child = this.childGraphs.get(this.currentNodeId);
			if (child) {
				child.focusEntry();
				return { status: 'moved', nodeId: child.defaultEntry };
			}
			return { status: 'boundary', nodeId: this.currentNodeId, direction: 'enter' };
		}

		// top/bottom/left/right: check internal connections
		const conn = this.connections.get(this.currentNodeId);
		const target = conn?.[direction];

		if (target) {
			const node = this.nodes.get(target);
			if (node) {
				if ('move' in node && typeof node.move === 'function') {
					// It's a Graph - recurse into it
					return node.move(direction);
				} else {
					// It's a LeafGraph - focus it
					node.focus();
					this.currentNodeId = target;
					return { status: 'moved', nodeId: target };
				}
			}
		}

		// Hit boundary - bubble up to parent
		if (this.parent) {
			return this.parent.move(direction);
		}

		// At root - can't navigate further
		return { status: 'boundary', nodeId: this.currentNodeId, direction };
	}
}
```

---

## FocusNavigator

Singleton that manages the root graph and global navigation state.

```typescript
class FocusNavigator {
	root: Graph | null = null;
	overlayActive = false;

	handleKeydown(event: KeyboardEvent): void {
		if (this.overlayActive) return;
		if (!this.root) return;

		const dir = this.keyToDirection(event.key);
		if (!dir) return;

		event.preventDefault();
		this.move(dir);
	}

	move(direction: Direction): void {
		if (!this.root) return;

		const result = this.root.move(direction);

		// boundary at root means navigation failed - stay where we are
		if (result.status === 'boundary') {
			console.log('boundary:', result.direction, 'at', result.nodeId);
		}
	}

	registerRoot(graph: Graph): void {
		this.root = graph;
		this.root.focusEntry();
	}

	setOverlayActive(active: boolean): void {
		this.overlayActive = active;
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
			case 'Enter':
				return 'enter';
			case 'Escape':
				return 'escape';
			default:
				return null;
		}
	}
}

export const focusNavigator = new FocusNavigator();
```

---

## Overlay Lifecycle

Overlays (e.g., VideoSourceOverlay) are temporary child graphs created on Enter and destroyed on Escape.

### Lifecycle Flow

```
┌─────────────────────────────────────────────────────────────┐
│ 1. Focus on DoubanCard (home:section:0:card:5)              │
│    currentNodeId = 'home:section:0:card:5'                  │
├─────────────────────────────────────────────────────────────┤
│ 2. User presses Enter                                       │
│    DoubanCard.onEnter(registerChild)                        │
│         ├── Create overlayGraph                             │
│         ├── Create VideoSourceOverlay component             │
│         └── registerChild(overlayGraph)                     │
│              └── graph.childGraphs.set('home:section:0:card:5', overlayGraph) │
│              └── overlayGraph.parent = graph                │
│              └── overlayGraph.parentNodeId = 'home:section:0:card:5' │
│    overlayGraph.focusEntry()                                │
├─────────────────────────────────────────────────────────────┤
│ 3. User navigates within overlay (up/down/enter)            │
│    move() recurses into overlayGraph                        │
├─────────────────────────────────────────────────────────────┤
│ 4. User presses Escape                                      │
│    overlayGraph.move('escape')                              │
│         ├── parent.focusNode(parentNodeId)                  │
│         ├── Destroy overlayGraph                            │
│         └── Destroy VideoSourceOverlay component            │
│    graph.childGraphs.delete('home:section:0:card:5')        │
├─────────────────────────────────────────────────────────────┤
│ 5. Focus returns to DoubanCard                              │
│    currentNodeId = 'home:section:0:card:5'                  │
└─────────────────────────────────────────────────────────────┘
```

### Example: VideoSourceOverlay

```typescript
// DoubanCard creation
const card = new LeafGraph('home:section:0:card:5', element, {
	onEnter: (registerChild) => {
		// Create overlay component
		const overlay = new VideoSourceOverlay(videoData);

		// Build overlay graph
		const overlayGraph = new Graph('overlay', 'overlay:source:0');
		// ... register source nodes ...

		// Register as child - parent handles the relationship
		registerChild(overlayGraph);

		// Show overlay
		overlay.show();
	}
});
```

---

## Complete Example: Home Page

```typescript
// +page.svelte
import { onMount } from 'svelte';
import { focusNavigator, Graph, LeafGraph, type Direction } from '$lib/utils/focus-navigator';

let section0Cards: LeafGraph[] = [];
let section1Cards: LeafGraph[] = [];

onMount(() => {
	// Create cards for section 0
	const section0Els = document.querySelectorAll('[data-section="0"] .card');
	section0Els.forEach((el, i) => {
		const nodeId = `home:section:0:card:${i}`;
		const card = new LeafGraph(nodeId, el as HTMLElement, {
			onEnter: (registerChild) => showVideoOverlay(item, registerChild),
			onEscape: () => {} // Won't be called - parent handles escape
		});
		section0Cards.push(card);
	});

	// Create cards for section 1
	// ...

	// Build section 0 graph
	const section0Graph = buildSectionGraph('home:section:0', section0Cards, 6);

	// Build section 1 graph
	const section1Graph = buildSectionGraph('home:section:1', section1Cards, 6);

	// Link sections vertically (section 0 bottom → section 1 top)
	const homeGraph = new Graph('home', section0Graph.defaultEntry);

	// Copy all connections from section graphs
	// (simplified - actual implementation copies node registrations and connections)

	// Register root
	focusNavigator.registerRoot(homeGraph);
});

function buildSectionGraph(prefix: string, cards: LeafGraph[], cols: number): Graph {
	const graph = new Graph(prefix, cards[0].nodeId);

	for (let i = 0; i < cards.length; i++) {
		const row = Math.floor(i / cols);
		const col = i % cols;

		graph.registerNode(cards[i].nodeId, cards[i]);

		if (col > 0) graph.addConnection(cards[i].nodeId, 'left', cards[i - 1].nodeId);
		if (col < cols - 1 && i + 1 < cards.length) {
			graph.addConnection(cards[i].nodeId, 'right', cards[i + 1].nodeId);
		}
		if (row > 0) graph.addConnection(cards[i].nodeId, 'top', cards[i - cols].nodeId);
		if (i + cols < cards.length) {
			graph.addConnection(cards[i].nodeId, 'bottom', cards[i + cols].nodeId);
		}
	}

	return graph;
}
```

---

## Integration

### Global Layout (+layout.svelte)

```svelte
<svelte:window onkeydown={(e) => focusNavigator.handleKeydown(e)} />
```

### Overlay Components

```svelte
<script lang="ts">
	import { focusNavigator } from '$lib/utils/focus-navigator';

	let { open, item } = $props();

	$effect(() => {
		focusNavigator.setOverlayActive(open);
	});
</script>
```

### Debugging

Enable devtools in `tauri.conf.json`:

```json
{
	"build": {
		"devtools": true
	}
}
```

Console logs in navigator:

```typescript
// navigator.ts
console.log('move:', direction, 'from', currentNodeId);
console.log('boundary:', direction, 'at', nodeId);
```

---

## File Structure

```
src/lib/utils/focus-navigator/
├── connections.ts    # Direction, MoveResult, Focusable, DirectionHandlers
├── graph.ts         # Graph, LeafGraph classes
├── navigator.ts     # FocusNavigator singleton
└── index.ts         # Re-exports

src/lib/components/
├── TVFocusRing.svelte         # Visual focus indicator
├── VideoSourceOverlay.svelte  # Overlay component
└── DoubanCard.svelte          # Leaf node with onEnter handler

src/routes/
├── +layout.svelte             # Global keydown handler
└── +page.svelte               # Page graph building
```
