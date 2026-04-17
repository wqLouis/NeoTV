# Svelte Focus Navigator - API Reference

## Table of Contents

1. [Overview](#overview)
2. [Core Concepts](#core-concepts)
3. [Declarative Navigation](#declarative-navigation)
4. [Focus Ring Design](#focus-ring-design)
5. [Why This Library](#why-this-library)
6. [API Reference](#api-reference)
7. [Comparison with Existing Solutions](#comparison-with-existing-solutions)

---

## Overview

**Svelte Focus Navigator** is a recursive, graph-based navigation system for TV remote controls and D-pad interfaces.

### Key Principles

- **Declarative**: Components self-declare their navigation role (leaf/graph/none)
- **Abstraction**: `move()` function is abstracted - any navigation algorithm works
- **Recursive**: Navigation bubbles from child to parent graph when boundaries are hit
- **4-direction**: All `UP/DOWN/LEFT/RIGHT` passed to move()
- **Independent focus ring**: Visual ring is decoupled from navigation tree

---

## Core Concepts

| Concept                  | Description                                                                                                                             |
| ------------------------ | --------------------------------------------------------------------------------------------------------------------------------------- |
| **Leaf**                 | A component that declares itself as a single focusable unit. No internal navigation - the whole component receives focus as one entity. |
| **Graph**                | A component that contains internal navigation between child nodes. The navigation system recurses inside to manage child connections.   |
| **Not in tree**          | A component that doesn't declare itself. Not added to navigation tree, cannot receive focus.                                            |
| **Node**                 | A registered Leaf node identified by a unique `NodeId`                                                                                  |
| **Connection**           | A directional link between two nodes                                                                                                    |
| **Recursive Navigation** | Child graphs delegate to parent when hitting boundaries                                                                                 |
| **move()**               | Abstracted function that implements navigation logic                                                                                    |

---

## Declarative Navigation

Components **declare their navigation type** via directive:

```svelte
<!-- Leaf: whole component is one focusable unit -->
<div use:focus={{ type: 'leaf', nodeId: 'home:card:0' }}>
	<!-- children are NOT focusable -->
</div>

<!-- Graph: navigation recurses INSIDE to children -->
<div use:focus={{ type: 'graph', nodeId: 'home:section:0', connections, move }}>
	<!-- children are registered as nodes inside -->
</div>

<!-- Not declared: not in navigation tree -->
<div class="header">
	<!-- cannot be focused -->
</div>
```

### Component Declaration Types

| Declaration     | Receives Focus        | Internal Nav | Use Case                        |
| --------------- | --------------------- | ------------ | ------------------------------- |
| `type: 'leaf'`  | Yes (whole component) | None         | Buttons, cards, simple elements |
| `type: 'graph'` | Yes (enters children) | Yes          | Sections, grids, lists          |
| Not declared    | No                    | N/A          | Layout divs, static UI          |

---

## Focus Ring Design

The focus ring is **completely independent** from the navigation tree. It simply tracks which element has focus and positions itself accordingly.

### Architecture

```
┌─────────────────────────────────────┐
│           Focus Ring                │
│  - Track document.activeElement    │
│  - Listen to focus events           │
│  - Move to element's rect          │
└─────────────────────────────────────┘
              ▲
              │ focus events
              │
┌─────────────────────────────────────┐
│      Browser / DOM                  │
│  - element.focus() fires focus     │
└─────────────────────────────────────┘
              ▲
              │ focus()
              │
┌─────────────────────────────────────┐
│    Navigation Tree                  │
│  - Calls element.focus()            │
│  - Has no knowledge of ring          │
└─────────────────────────────────────┘
```

### Why This Design

| Aspect           | Benefit                                                          |
| ---------------- | ---------------------------------------------------------------- |
| **Decoupled**    | Ring works with any focus mechanism, not just this library       |
| **Event-driven** | No polling, efficient - only updates on actual focus changes     |
| **Simple**       | Ring just follows `document.activeElement`, nothing more         |
| **Robust**       | Works even if navigation tree has bugs - always reflects reality |

---

## Why This Library

### Problems with existing solutions

1. **Norigin-Spatial-Navigation** (445 stars, most popular):
   - Uses automatic spatial calculation (finds nearest element in direction)
   - Doesn't support explicit graph-based navigation
   - React-specific, not framework-agnostic
   - Can't handle complex overlay hierarchies cleanly

2. **CSS-based approaches**:
   - Fragile - breaks when layouts change
   - Can't express complex navigation flows
   - Poor performance on low-end TVs

### Our Solution

- **Abstracted move function**: Any navigation algorithm works (spatial, grid, explicit connections, etc.)
- **Recursive delegation**: Clean separation between graphs, no central handler
- **Enter/Escape via move()**: All 6 directions handled uniformly
- **Overlay support**: Child graphs for modals/overlays with automatic lifecycle
- **Framework-agnostic**: Core is pure TypeScript, adapters for Svelte/React/Vanilla

---

## API Reference

### Valid Return Values

All `move()` functions must return one of:

| Return                                                         | Description                           |
| -------------------------------------------------------------- | ------------------------------------- |
| `{ status: 'moved'; nodeId: NodeId }`                          | Successfully moved to target node     |
| `{ status: 'boundary'; nodeId: NodeId; direction: Direction }` | Cannot move further in that direction |
| `{ status: 'none' }`                                           | No current node / not initialized     |

### Types

```typescript
// Direction of navigation (4 directions)
type NavDir = 'UP' | 'DOWN' | 'LEFT' | 'RIGHT';

// Unique identifier for a node (path-like: "home:section:0:card:5")
type NodeId = string;

// Result of a move operation
type MoveResult = 'BOUNDARY' | 'OK';

// Anything that can be focused
interface Focusable {
	focus(): void;
}

// Abstracted move function signature
// connections: customizable data structure (any format the component chooses)
// currentNodeId: the current focused node
// direction: the direction to move in (UP/DOWN/LEFT/RIGHT)
type MoveFn = (connections: unknown, currentNodeId: NodeId | null, direction: NavDir) => MoveResult;

// Svelte action directive options
interface FocusOptions {
	type: 'leaf' | 'graph';
	nodeId: NodeId;
	connections?: unknown; // Custom data structure (any format)
	move?: MoveFn; // Required for graph, unused for leaf
}
```

### Leaf vs Graph

| Type      | Has move()     | Connection Graph   | Boundary handling              |
| --------- | -------------- | ------------------ | ------------------------------ |
| **Leaf**  | No (none)      | No                 | All directions return BOUNDARY |
| **Graph** | Yes (required) | Yes (customizable) | Passed to move() as direction  |

### Leaf

When a component declares `type: 'leaf'`, it is one focusable unit. All directions return boundary.

```typescript
// Leaf has no move() function
// It's registered as a simple Focusable
const leaf: Focusable = {
	focus: () => element.focus()
};
```

### Graph (Abstraction)

When a component declares `type: 'graph'`, it provides:

1. A **connection graph** (customizable data structure)
2. A **move function** (implements navigation logic, receives all 6 directions)

```typescript
// Example: Simple Map-based connection graph
const connections = new Map<NodeId, Partial<Record<NavDir, NodeId>>>();

// Example: Grid-based connection graph
const connections = {
  nodes: [...],
  cols: 6,
  // ... any structure
};

// Example: Spatial/calculated connection graph
const connections = {
  getTarget: (current, dir) => calculateSpatialNav(current, dir)
};

// The move function receives direction
const move: MoveFn = (connections, currentNodeId, direction) => {
  switch (direction) {
    case 'UP':
    case 'DOWN':
    case 'LEFT':
    case 'RIGHT':
      // Custom directional navigation
      return navigateDirection(connections, currentNodeId, direction);
  }
};

// Graph is created with abstracted move function
const graph = new Graph('home:section:0', 'home:section:0:card:0', connections, move);
```

### Graph Class (Conceptual)

```typescript
class Graph implements Focusable {
	currentNodeId: NodeId | null = null;

	constructor(
		public readonly nodeId: NodeId,
		private entryNodeId: NodeId,
		private connections: unknown, // Customizable structure
		private move: MoveFn // Required - the abstraction
	) {}

	focus(): void;
	focusEntry(): void;
	focusNode(nodeId: NodeId): void;

	move(direction: NavDir): MoveResult {
		// Delegates to component's abstracted move function
		return this.move(this.connections, this.currentNodeId, direction);
	}
}
```

### FocusNavigator

Global singleton that manages the root graph and keyboard input.

```typescript
class FocusNavigator {
	root: Graph | null = null;
	overlayActive = false;

	handleKeydown(event: KeyboardEvent): void;
	registerRoot(graph: Graph): void;
	setOverlayActive(active: boolean): void;
	getCurrentNodeId(): NodeId | null;
	getRoot(): Graph | null;
}

export const focusNavigator = new FocusNavigator();
```

---

## Comparison with Existing Solutions

| Feature          | Svelte Focus Navigator   | Norigin Spatial Nav | CSS-only      |
| ---------------- | ------------------------ | ------------------- | ------------- |
| Navigation model | Abstracted move function | Automatic spatial   | CSS flex/grid |
| D-pad support    | Full (4 directions)      | Full                | Limited       |
| Enter/Escape     | Not supported            | Custom              | No            |
| Overlay/modals   | Manual                   | Manual              | No            |
| Framework        | Svelte (+ vanilla)       | React only          | Any           |
| Customization    | Full - any algorithm     | None                | N/A           |
| Learning curve   | Low (simple API)         | Medium              | Low           |

---

## Comparison with Norigin Spatial Navigation

### Norigin Approach

```tsx
// Uses automatic spatial calculation
<Focusable id="card1">
	<div onClick={() => goToDetails()} />
</Focusable>

// Navigation calculated based on DOM position
// "Press right" → find nearest element to the right
```

### Our Approach

```typescript
// Custom move function - any navigation algorithm
const move = (connections, currentNodeId, direction) => {
	// Can use: spatial calculation, explicit connections, AI, anything
	const target = calculateTarget(connections, currentNodeId, direction);
	if (!target) return { status: 'boundary', nodeId: currentNodeId, direction };
	return { status: 'moved', nodeId: target };
};

const graph = new Graph('home', 'home:card:0', connections, move);
```

### Trade-offs

| Aspect          | Norigin                 | Our Library          |
| --------------- | ----------------------- | -------------------- |
| Setup time      | Fast (auto)             | Medium (manual)      |
| Control         | Low                     | Full - any algorithm |
| Edge cases      | May choose wrong target | Always predictable   |
| Dynamic layouts | Works automatically     | Customizable         |
| Overlays/modals | Manual handling         | Manual               |
| Enter/Escape    | Custom                  | Not supported        |
