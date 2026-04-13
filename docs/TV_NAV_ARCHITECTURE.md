# TV Navigation Focus System Architecture

## Core Classes

### Connections

```typescript
type Direction = 'top' | 'bottom' | 'left' | 'right';

interface Connections {
	top?: GraphNode | 'end';
	bottom?: GraphNode | 'end';
	left?: GraphNode | 'end';
	right?: GraphNode | 'end';
}
```

### Leaf

Leaf nodes wrap actual DOM elements.

```typescript
class Leaf {
	readonly element: HTMLElement;
	readonly nodeId: string;

	focus(): void;
	scrollIntoView(): void;
}
```

### LeafGraph

A LeafGraph is a Graph node that represents a single focusable DOM element with no internal navigation (all directions point to 'end').

```typescript
class LeafGraph extends Graph {
	constructor(element: HTMLElement, nodeId: string);
}
```

### Graph

Graph nodes contain a map of connections and manage navigation between nodes.

```typescript
class Graph {
	readonly name: string;
	readonly connections: Map<GraphNode, Connections>;
	readonly defaultEntry: GraphNode;

	move(direction: Direction): boolean;
	getEntry(direction: Direction): GraphNode | 'end';
	addConnection(node: GraphNode, conn: Connections): void;
	setDefaultEntry(node: GraphNode): void;
	focusCurrent(): void;
}
```

---

## Graph Tree Structure

```
RootGraph (virtual root)
│
├── SidebarGraph (sidebar navigation)
│   └── sidebar:0 ~ sidebar:5 (LeafGraph × 6)
│       └── Each sidebar item connects right → ContentGraph
│
└── ContentGraph (dynamic, switches per page)
    │
    ├── HomeGraph
    │   └── HorizontalSection × 2 (LeafGraph × N cards)
    │
    ├── BrowseGraph
    │   ├── TabGraph × 2 (LeafGraph)
    │   ├── GenreGraph × N (LeafGraph)
    │   └── CardsGraph × N (LeafGraph, grid navigation)
    │
    ├── SearchGraph
    │   ├── HistoryItems × M (LeafGraph)
    │   └── ResultsGrid × N (LeafGraph, 2D grid)
    │
    ├── HistoryGraph
    │   └── CardsGrid × N (LeafGraph, 2D grid)
    │
    ├── FavouritesGraph
    │   └── CardsGrid × N (LeafGraph, 2D grid)
    │
    └── SettingsGraph
        └── SettingControls × N (LeafGraph)
```

---

## Navigator

```typescript
class FocusNavigator {
	root: Graph;
	currentLeaf: Leaf | LeafGraph | null;
	currentNode: GraphNode;
	ringElement: HTMLElement;

	handleKeydown(event: KeyboardEvent): void;
	move(direction: Direction): void;
	navigateTo(target: GraphNode | 'end'): void;
	switchLeafGraph(target: Graph | Leaf): void;
}
```

### Navigation Flow

```
User presses Down
    ↓
currentNode = currentNode.connections.bottom
    ↓
┌─ LeafGraph → update focus ring to element
├─ Graph → switch to graph.defaultEntry, focus it
└─ 'end' → stay at current position, handle boundary
```

### Sidebar ↔ Content Boundary

When navigating left from content area:

1. Find the current page's sidebar index
2. Navigate to corresponding sidebar item
3. Update contentGraph to the new page's graph

---

## Focus Ring

The focus ring is a visual indicator showing the currently focused element.

### Implementation

1. **Ring Element**: A fixed position `<div>` overlaid on the page
2. **Position Tracking**: Polls every 100ms to track `focusNavigator.getCurrentNodeId()`
3. **Visibility**: Only shown when:
   - `settingsStore.focusRingEnabled === true`
   - `focusNavigator.state.overlayActive === false`
   - A valid nodeId exists

### Styling

```css
.tv-focus-ring {
	border: 2px solid var(--primary);
	border-radius: 8px;
	box-shadow: 0 0 12px 2px var(--ring);
	animation: focusPulse 1.5s ease-in-out infinite;
}
```

---

## DOM Integration

### data-tv-node Attribute

Each focusable element should have a unique `data-tv-node` attribute:

```html
<a data-tv-node="sidebar:0" href="/search">搜索</a>
<div data-tv-node="home:section:0:card:5">...</div>
```

### Dynamic Node Registration

Nodes are registered after DOM render using `onMount`:

```typescript
function buildHomeGraph(): Graph | null {
	if (typeof document === 'undefined') return null;

	const cards: LeafGraph[] = [];
	let el = document.querySelector('[data-tv-node="home:section:0:card:0"]');
	let i = 0;
	while (el) {
		cards.push(new LeafGraph(el, `home:section:0:card:${i}`));
		i++;
		el = document.querySelector(`[data-tv-node="home:section:0:card:${i}"]`);
	}

	if (cards.length === 0) return null;

	const graph = new Graph('home', cards[0]);

	// Connect cards in 2D grid
	const cols = 6;
	for (let i = 0; i < cards.length; i++) {
		const row = Math.floor(i / cols);
		const col = i % cols;
		const left = col > 0 ? cards[i - 1] : 'end';
		const right = col < cols - 1 && i + 1 < cards.length ? cards[i + 1] : 'end';
		const top = row === 0 ? 'end' : cards[i - cols];
		const bottom = i + cols < cards.length ? cards[i + cols] : 'end';
		graph.addConnection(cards[i], { left, right, top, bottom });
	}

	return graph;
}
```

---

## Key Implementation Notes

### 1. LeafGraph vs Graph

- **LeafGraph**: Used for single DOM elements with no internal navigation
- **Graph**: Used for containers that manage multiple child nodes

### 2. Boundary Handling

When navigation hits 'end', check if we need to switch between sidebar and content.

### 3. Dynamic Registration

Graphs must be rebuilt when:

- Page content changes (e.g., new search results)
- Grid layout changes (density setting)

### 4. Sidebar Elements

Sidebar items should be registered with their actual DOM elements from `onMount`, not placeholder divs.

### 5. Overlay State

When `overlayActive === true`, navigation should be paused and focus ring hidden.

---

## File Structure

```
src/lib/utils/focus-navigator/
├── connections.ts    # Direction, Connections interface
├── leaf.ts          # Leaf, LeafGraph classes
├── graph.ts         # Graph class
├── navigator.ts     # FocusNavigator class + focusNavigator singleton
└── index.ts         # Re-exports
```

---

## Usage Example

```typescript
import { focusNavigator, Graph, LeafGraph } from '$lib/utils/focus-navigator';

// In +page.svelte onMount:
const graph = buildHomeGraph();
if (graph) {
    focusNavigator.registerPageGraph('home', graph);
}

// In +layout.svelte:
<svelte:window onkeydown={(e) => focusNavigator.handleKeydown(e)} />

// In VideoSourceOverlay:
$effect(() => {
    focusNavigator.setOverlayActive(open);
});
```
