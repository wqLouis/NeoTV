# Svelte Focus Navigator - Examples

## Table of Contents

1. [Vanilla JS / Tauri](#vanilla-js--tauri)
2. [Svelte Components](#svelte-components)
3. [Graph Component with Internal Navigation](#graph-component-with-internal-navigation)
4. [Focus Ring Component](#focus-ring-component)
5. [Not in Navigation Tree](#not-in-navigation-tree)

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
	connections.set('home:card:0', { RIGHT: 'home:card:1' });
	connections.set('home:card:1', { LEFT: 'home:card:0' });

	// Abstracted move function - handles all 4 directions
	const move = (connections, currentNodeId, direction) => {
		const targets = connections.get(currentNodeId);
		const target = targets?.[direction];

		if (!target) {
			return 'BOUNDARY';
		}
		return 'OK';
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

		// LEFT/RIGHT
		if (col > 0) conn.LEFT = `grid:card:${i - 1}`;
		if (col < cols - 1 && i + 1 < items.length) conn.RIGHT = `grid:card:${i + 1}`;

		// UP/DOWN
		if (row > 0) conn.UP = `grid:card:${i - cols}`;
		if (i + cols < items.length) conn.DOWN = `grid:card:${i + cols}`;

		connections.set(nodeId, conn);
	});

	// Custom move function - handles all 4 directions
	function move(connections, currentNodeId, direction) {
		const targets = connections.get(currentNodeId);
		const target = targets?.[direction];
		if (!target) {
			return 'BOUNDARY';
		}
		return 'OK';
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

### Memory leaks

1. Clean up event listeners in cleanup
