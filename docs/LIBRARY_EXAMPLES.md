# Svelte Focus Navigator - Examples

## Table of Contents

1. [Vanilla JS / Tauri](#vanilla-js--tauri)
2. [Svelte Components](#svelte-components)
3. [Graph Component with Internal Navigation](#graph-component-with-internal-navigation)
4. [Overlay Lifecycle](#overlay-lifecycle)
5. [Focus Ring Component](#focus-ring-component)
6. [Not in Navigation Tree](#not-in-navigation-tree)

---

## Vanilla JS / Tauri

```typescript
import { focusNavigator, Graph, LeafGraph } from 'svelte-focus-navigator';

document.addEventListener('DOMContentLoaded', () => {
	// Leaf nodes - simple focusable elements
	const card0 = new LeafGraph('home:card:0', document.querySelector('#card0'));
	const card1 = new LeafGraph('home:card:1', document.querySelector('#card1'));

	// Connection graph - simple Map structure
	const connections = new Map();
	connections.set('home:card:0', { right: 'home:card:1' });
	connections.set('home:card:1', { left: 'home:card:0' });

	// Abstracted move function - handles all 6 directions
	const move = (connections, currentNodeId, direction) => {
		const targets = connections.get(currentNodeId);
		const target = targets?.[direction];

		if (direction === 'enter' || direction === 'escape') {
			// Handle special keys - return boundary (or custom logic)
			return { status: 'boundary', nodeId: currentNodeId, direction };
		}

		if (!target) {
			return { status: 'boundary', nodeId: currentNodeId, direction };
		}
		return { status: 'moved', nodeId: target };
	};

	// Create graph with abstracted move function
	const homeGraph = new Graph('home', 'home:card:0', connections, move);

	focusNavigator.registerRoot(homeGraph);
});
```

```html
<svelte:window onkeydown="focusNavigator.handleKeydown(event)" />

<div id="card0" tabindex="0">Card 1</div>
<div id="card1" tabindex="0">Card 2</div>
```

---

## Svelte Components

### Leaf Component

```svelte
<script lang="ts">
	import { focus } from 'svelte-focus-navigator';

	let { item } = $props();
</script>

<!-- Declarative: component declares itself as LEAF -->
<!-- Whole component is one focusable unit, returns boundary for all directions -->
<div
	use:focus={{
		type: 'leaf',
		nodeId: `card:${item.id}`
	}}
	tabindex="0"
	role="button"
>
	<!-- card content -->
</div>
```

### Leaf with Enter/Escape Handlers

```svelte
<script lang="ts">
	import { focus } from 'svelte-focus-navigator';

	let { item } = $props();

	function handleEnter(connections, currentNodeId) {
		// Custom enter logic - open overlay, navigate to detail, etc.
		return { status: 'moved', nodeId: 'overlay:source:0' };
	}

	function handleEscape(connections, currentNodeId) {
		// Custom escape logic - close, go back, etc.
		return { status: 'moved', nodeId: 'parent' };
	}

	const move = (connections, currentNodeId, direction) => {
		switch (direction) {
			case 'enter':
				return handleEnter(connections, currentNodeId);
			case 'escape':
				return handleEscape(connections, currentNodeId);
			default:
				return { status: 'boundary', nodeId: currentNodeId, direction };
		}
	};
</script>

<div
	use:focus={{
		type: 'leaf',
		nodeId: `card:${item.id}`,
		move
	}}
	tabindex="0"
	role="button"
>
	<!-- card content -->
</div>
```

---

## Graph Component with Internal Navigation

### Simple Grid Navigation

```svelte
<script lang="ts">
	import { focus } from 'svelte-focus-navigator';
	import Card from './Card.svelte';

	let { items } = $props();

	// Build 6-column grid connections
	const connections = new Map();
	const cols = 6;

	items.forEach((item, i) => {
		const row = Math.floor(i / cols);
		const col = i % cols;

		const nodeId = `grid:card:${i}`;
		const conn: Record<string, string> = {};

		// Left/Right
		if (col > 0) conn.left = `grid:card:${i - 1}`;
		if (col < cols - 1 && i + 1 < items.length) conn.right = `grid:card:${i + 1}`;

		// Top/Bottom
		if (row > 0) conn.top = `grid:card:${i - cols}`;
		if (i + cols < items.length) conn.bottom = `grid:card:${i + cols}`;

		connections.set(nodeId, conn);
	});

	// Custom move function - handles all 6 directions
	function move(connections, currentNodeId, direction) {
		switch (direction) {
			case 'enter':
				// Navigate to detail view
				return { status: 'moved', nodeId: `detail:${currentNodeId}` };

			case 'escape':
				// Go back to parent
				return { status: 'moved', nodeId: 'parent' };

			case 'top':
			case 'bottom':
			case 'left':
			case 'right':
				const targets = connections.get(currentNodeId);
				const target = targets?.[direction];
				if (!target) {
					return { status: 'boundary', nodeId: currentNodeId, direction };
				}
				return { status: 'moved', nodeId: target };
		}
	}
</script>

<!-- Declarative: component declares itself as GRAPH -->
<!-- move() function handles all navigation logic -->
<div
	use:focus={{
		type: 'graph',
		nodeId: 'home:grid',
		connections,
		move
	}}
>
	<div class="grid grid-cols-6 gap-4">
		{#each items as item, i (item.id)}
			<Card {item} nodeId={`grid:card:${i}`} />
		{/each}
	</div>
</div>
```

### Enter/Escape via Move Function

```svelte
<script lang="ts">
	import { focus } from 'svelte-focus-navigator';

	let { children, onOpenOverlay } = $props();

	const connections = new Map();
	// ... build connections ...

	function move(connections, currentNodeId, direction) {
		switch (direction) {
			case 'enter':
				// Open overlay when enter is pressed
				onOpenOverlay?.();
				return { status: 'moved', nodeId: 'overlay:source:0' };

			case 'escape':
				// Go back / close
				return { status: 'moved', nodeId: 'parent' };

			default:
				const target = connections.get(currentNodeId)?.[direction];
				if (!target) {
					return { status: 'boundary', nodeId: currentNodeId, direction };
				}
				return { status: 'moved', nodeId: target };
		}
	}
</script>

<div
	use:focus={{
		type: 'graph',
		nodeId: 'section',
		connections,
		move
	}}
>
	{@render children()}
</div>
```

---

## Overlay Lifecycle

Overlays are temporary child graphs created when Enter is pressed, destroyed when Escape is pressed.

### Parent Graph Handles Enter → Creates Child Graph

```typescript
const parentMove = (connections, currentNodeId, direction) => {
	if (direction === 'enter' && currentNodeId === 'home:card:5') {
		// Create overlay connections
		const overlayConnections = new Map();
		overlayConnections.set('overlay:source:0', { down: 'overlay:source:1' });
		overlayConnections.set('overlay:source:1', { up: 'overlay:source:0' });

		// Overlay's move function
		const overlayMove = (conn, nodeId, dir) => {
			if (dir === 'escape') {
				// Return focus to parent card
				return { status: 'moved', nodeId: 'home:card:5' };
			}
			const target = conn.get(nodeId)?.[dir];
			if (!target) {
				return { status: 'boundary', nodeId, direction: dir };
			}
			return { status: 'moved', nodeId: target };
		};

		// Create overlay graph
		const overlayGraph = new Graph('overlay', 'overlay:source:0', overlayConnections, overlayMove);

		// Register as child graph
		parentGraph.registerChildGraph('home:card:5', overlayGraph);

		// Return the overlay entry node
		return { status: 'moved', nodeId: 'overlay:source:0' };
	}

	// Normal navigation
	const target = connections.get(currentNodeId)?.[direction];
	if (!target) {
		return { status: 'boundary', nodeId: currentNodeId, direction };
	}
	return { status: 'moved', nodeId: target };
};
```

### Overlay Handles Escape → Returns to Parent

```typescript
const overlayMove = (connections, currentNodeId, direction) => {
	if (direction === 'escape') {
		// Get parent reference and return focus
		const parentNodeId = overlayGraph.parentNodeId;
		overlayGraph.parent.focusNode(parentNodeId);
		overlayGraph.parent.childGraphs.delete(overlayGraph.nodeId);
		return { status: 'moved', nodeId: parentNodeId };
	}

	// Normal overlay navigation
	const target = connections.get(currentNodeId)?.[direction];
	if (!target) {
		return { status: 'boundary', nodeId: currentNodeId, direction };
	}
	return { status: 'moved', nodeId: target };
};
```

### Flow Diagram

```
1. Focus on DoubanCard (leaf)
   └── currentNodeId = 'home:section:0:card:5'

2. Press Enter
   └── Parent graph's move('enter') returns child graph target
       ├── overlayGraph.parent = parentGraph
       ├── overlayGraph.parentNodeId = 'home:section:0:card:5'
       └── Focus moves to overlayGraph entry

3. Navigate within overlay (up/down/enter/escape)
   └── overlayGraph.move(direction) handles all navigation

4. Press Escape
   └── overlayGraph.move('escape')
       ├── Returns { status: 'moved', nodeId: parentNodeId }
       ├── Focus returns to parent node
       └── Destroy overlay component
```

---

## Focus Ring Component

The focus ring is a simple overlay that tracks `document.activeElement`. It has **no knowledge** of the navigation tree:

```svelte
<script lang="ts">
	// Independent - just follows browser focus via focusin events
	let ringEl: HTMLDivElement;
	let isVisible = $state(false);

	function updateRing(e: FocusEvent) {
		const target = e.target as HTMLElement;
		if (!target || !ringEl) return;

		const rect = target.getBoundingClientRect();
		ringEl.style.left = `${rect.left - 4}px`;
		ringEl.style.top = `${rect.top - 4}px`;
		ringEl.style.width = `${rect.width + 8}px`;
		ringEl.style.height = `${rect.height + 8}px`;
		isVisible = true;
	}
</script>

<svelte:document onfocusin={updateRing} />

{#if isVisible}
	<div bind:this={ringEl} class="focus-ring"></div>
{/if}

<style>
	.focus-ring {
		position: fixed;
		pointer-events: none;
		z-index: 9999;
		border: 3px solid white;
		border-radius: 8px;
		box-shadow: 0 0 20px 4px rgba(255, 255, 255, 0.5);
	}
</style>
```

---

## Not in Navigation Tree

```svelte
<!-- No declaration = not in navigation tree -->
<div class="header">
	<Logo />
	<!-- cannot be focused -->
</div>
```

---

## Troubleshooting

### Component not receiving focus

1. Check component has `use:focus={{ type: 'leaf' | 'graph', nodeId: '...' }}` directive
2. Ensure parent graph's move function returns this node as a target
3. Verify `tabindex="0"` is set on the element

### Navigation not entering component

1. Check component declares `type: 'graph'` - only graphs recurse inside
2. Verify parent graph's move function returns the correct target for the direction

### Component wrongly in navigation tree

1. Remove the `use:focus` directive if component should not be navigable
2. Remember: no declaration = not in tree

### Focus not moving visually

1. Ensure DOM elements have `tabindex="0"`
2. Ensure elements are focusable
3. Check `focus()` is being called on element

### Enter/Escape not working

1. Check graph's move function handles `direction === 'enter' | 'escape'`
2. Verify move function returns correct `{ status: 'moved', nodeId }` result
3. Ensure parent graph's move function handles returning from child graph

### Memory leaks

1. Clean up child graphs on component unmount
2. Remove event listeners in cleanup
3. Call `focusNavigator.setOverlayActive(false)` on unmount
