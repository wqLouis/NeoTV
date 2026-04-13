# Svelte Focus Navigator - Library Development Guide

## Table of Contents

1. [Overview](#overview)
2. [Why This Library](#why-this-library)
3. [Comparison with Existing Solutions](#comparison-with-existing-solutions)
4. [API Reference](#api-reference)
5. [Overlay Lifecycle](#overlay-lifecycle)
6. [Project Structure](#project-structure)
7. [Development Roadmap](#development-roadmap)
8. [Publishing Guide](#publishing-guide)
9. [Integration Examples](#integration-examples)

---

## Overview

**Svelte Focus Navigator** is a recursive, graph-based navigation system for TV remote controls and D-pad interfaces.

### Core Concepts

| Concept | Description |
|---------|-------------|
| **Graph** | A container of nodes with directional connections |
| **LeafGraph** | A single DOM element wrapper (no internal navigation) |
| **Node** | Any focusable element identified by a unique `NodeId` |
| **Connection** | A directional link between two nodes |
| **Recursive Navigation** | Child graphs delegate to parent when hitting boundaries |

### Key Features

- **Recursive bubble-up**: Navigation bubbles from child to parent graph when boundaries are hit
- **Hierarchical structure**: Graphs can contain child graphs (e.g., overlays, modals)
- **6-direction support**: `top`, `bottom`, `left`, `right`, `enter`, `escape`
- **Framework-agnostic core**: Core logic works with any JS/TS project
- **Svelte integration**: Built-in Svelte support

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

- **Explicit connections**: You define exactly where each direction goes
- **Recursive delegation**: Clean separation between graphs, no central handler
- **Overlay support**: Child graphs for modals/overlays with automatic lifecycle
- **Framework-agnostic**: Core is pure TypeScript, adapters for Svelte/React/Vanilla

---

## Comparison with Existing Solutions

| Feature | Svelte Focus Navigator | Norigin Spatial Nav | CSS-only |
|---------|------------------------|---------------------|----------|
| Navigation model | Explicit graph | Automatic spatial | CSS flex/grid |
| D-pad support | Full | Full | Limited |
| Enter/Escape | Full | Custom | No |
| Overlay/modals | Child graphs | Manual | No |
| Framework | Svelte (+ vanilla) | React only | Any |
| Learning curve | Low | Medium | Low |

---

## API Reference

### Types

```typescript
// Direction of navigation (6 directions)
type Direction = 'top' | 'bottom' | 'left' | 'right' | 'enter' | 'escape';

// Unique identifier for a node (path-like: "home:section:0:card:5")
type NodeId = string;

// Result of a move operation
type MoveResult =
  | { status: 'moved'; nodeId: NodeId }
  | { status: 'boundary'; nodeId: NodeId; direction: Exclude<Direction, 'enter' | 'escape'> }
  | { status: 'escape' }
  | { status: 'none' };

// Anything that can be focused
interface Focusable {
  focus(): void;
}

// Handlers for enter and escape keys
interface DirectionHandlers {
  onEnter?: (registerChild: (child: Graph) => void) => void;
  onEscape?: () => void;
}
```

### LeafGraph

Wraps a single DOM element with optional enter/escape handlers.

```typescript
class LeafGraph implements Focusable {
  constructor(
    public readonly nodeId: NodeId,
    public readonly element: HTMLElement,
    private handlers?: DirectionHandlers
  ) {}

  move(direction: Direction): MoveResult {
    switch (direction) {
      case 'enter':
        this.handlers?.onEnter?.((child) => {
          // Parent registers the child graph
        });
        return { status: 'moved', nodeId: this.nodeId };

      case 'escape':
        // Leaf nodes don't handle escape - bubble up to parent
        return { status: 'escape' };

      default:
        // top/bottom/left/right - leaf has no internal nav
        return { status: 'boundary', nodeId: this.nodeId, direction };
    }
  }

  focus(): void {
    this.element.focus();
  }
}
```

### Graph

Manages a collection of nodes with directional connections.

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

  // Entry point (first focusable node)
  defaultEntry: NodeId;

  constructor(name: string, defaultEntry: NodeId) {}

  // --- Registration ---

  registerNode(nodeId: NodeId, node: Focusable): void;
  registerChildGraph(prefix: NodeId, child: Graph): void;
  addConnection(from: NodeId, direction: Direction, to: NodeId): void;

  // --- Navigation ---

  focus(): void;
  focusEntry(): void;
  focusNode(nodeId: NodeId): void;
  move(direction: Direction): MoveResult;
  getNode(nodeId: NodeId): Focusable | undefined;
}
```

### FocusNavigator

Global singleton that manages the root graph and keyboard input.

```typescript
class FocusNavigator {
  root: Graph | null = null;
  overlayActive = false;

  handleKeydown(event: KeyboardEvent): void;
  move(direction: Direction): void;
  registerRoot(graph: Graph): void;
  setOverlayActive(active: boolean): void;
  getCurrentNodeId(): NodeId | null;
  getRoot(): Graph | null;
}

export const focusNavigator = new FocusNavigator();
```

---

## Overlay Lifecycle

Overlays (modals, video source selectors) are temporary child graphs.

### Flow

```
1. Focus on DoubanCard
   └── currentNodeId = 'home:section:0:card:5'

2. Press Enter
   └── DoubanCard.onEnter(registerChild)
       ├── Create overlayGraph
       ├── registerChild(overlayGraph)
       │   ├── overlayGraph.parent = graph
       │   ├── overlayGraph.parentNodeId = 'home:section:0:card:5'
       │   └── graph.childGraphs.set('home:section:0:card:5', overlayGraph)
       └── overlayGraph.focusEntry()

3. Navigate within overlay (up/down/enter on sources)
   └── move() recurses into overlayGraph

4. Press Escape
   └── overlayGraph.move('escape')
       ├── parent.focusNode(parentNodeId)
       ├── graph.childGraphs.delete('home:section:0:card:5')
       └── Destroy overlay component
```

### Example: VideoSourceOverlay

```typescript
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

## Project Structure

```
svelte-focus-navigator/
├── src/
│   ├── connections.ts      # Core types (Direction, MoveResult, Focusable)
│   ├── graph.ts           # Graph class
│   ├── leaf.ts            # LeafGraph class
│   ├── navigator.ts       # FocusNavigator singleton
│   └── index.ts           # Main exports
│
├── dist/                  # Built output
│   ├── index.js
│   ├── index.d.ts
│   ├── vanilla.js
│   └── svelte.js
│
├── examples/
│   ├── vanilla/
│   │   └── index.html
│   └── svelte/
│       └── App.svelte
│
├── tests/
│   ├── graph.test.ts
│   ├── leaf.test.ts
│   └── navigator.test.ts
│
├── README.md
├── LICENSE (MIT)
├── package.json
├── tsconfig.json
└── vite.config.ts
```

### Package.json

```json
{
  "name": "svelte-focus-navigator",
  "version": "0.1.0",
  "description": "Recursive graph-based TV navigation for Svelte",
  "main": "./dist/index.js",
  "module": "./dist/index.js",
  "types": "./dist/index.d.ts",
  "exports": {
    ".": {
      "import": "./dist/index.js",
      "types": "./dist/index.d.ts"
    }
  },
  "files": ["dist"],
  "scripts": {
    "build": "bun run build.ts",
    "test": "bun test",
    "version:patch": "bun run version.ts patch",
    "version:minor": "bun run version.ts minor",
    "version:major": "bun run version.ts major"
  },
  "keywords": [
    "svelte",
    "tv",
    "smart-tv",
    "navigation",
    "remote-control",
    "d-pad",
    "focus"
  ],
  "peerDependencies": {
    "svelte": ">=4.0.0"
  },
  "devDependencies": {
    "svelte": "^5.0.0",
    "typescript": "^5.0.0",
    "bun": "^1.0.0"
  }
}
```

---

## Development Roadmap

### Phase 1: Core Library (current priority)

1. Create new repository with extracted code
2. Set up TypeScript configuration
3. Add Enter/Escape to Direction type
4. Add `DirectionHandlers` interface
5. Modify `LeafGraph` to accept handlers
6. Update `FocusNavigator.handleKeydown()` to map Enter/Escape keys
7. Implement `registerChild` callback pattern
8. Write unit tests for Graph and LeafGraph

### Phase 2: Svelte Integration

1. Create Svelte adapter (`use:focus` action)
2. Build `FocusRing` component
3. Create `$focusNavigator` store for Svelte 5 runes
4. Write integration tests

### Phase 3: Documentation & Examples

1. Write comprehensive README
2. Create vanilla JS example
3. Create Svelte example app
4. Document API with JSDoc

### Phase 4: Publish

1. Set up GitHub Actions for CI/CD
2. Create CHANGELOG.md
3. Publish to npm
4. Announce on relevant communities

---

## Publishing Guide

### Pre-requisites

- npm account (https://www.npmjs.com)
- Repository hosted on GitHub

### Release Commands

```bash
# Version bump (semantic versioning)
bun run version:patch  # 0.1.0 → 0.1.1
bun run version:minor  # 0.1.0 → 0.2.0
bun run version:major  # 0.1.0 → 1.0.0

# Build
bun run build

# Test
bun test

# Publish
bun publish --access public

# Create GitHub release
gh release create v0.1.0 --title "v0.1.0" --notes "Initial release"
```

> **Note:** This library uses [Bun](https://bun.sh) as the package manager instead of npm.

---

## Integration Examples

### Vanilla JS / Tauri

```typescript
import { focusNavigator, Graph, LeafGraph } from 'svelte-focus-navigator';

document.addEventListener('DOMContentLoaded', () => {
  const card0 = new LeafGraph('home:card:0', document.querySelector('#card0'));
  const card1 = new LeafGraph('home:card:1', document.querySelector('#card1'));

  const homeGraph = new Graph('home', card0.nodeId);
  homeGraph.registerNode(card0.nodeId, card0);
  homeGraph.registerNode(card1.nodeId, card1);
  homeGraph.addConnection(card0.nodeId, 'right', card1.nodeId);
  homeGraph.addConnection(card1.nodeId, 'left', card0.nodeId);

  focusNavigator.registerRoot(homeGraph);
});
```

```html
<svelte:window onkeydown="focusNavigator.handleKeydown(event)" />

<div id="card0" tabindex="0">Card 1</div>
<div id="card1" tabindex="0">Card 2</div>
```

### Svelte Component

```svelte
<script lang="ts">
  import { LeafGraph } from 'svelte-focus-navigator';
  import { onMount } from 'svelte';

  let { item, onEnter } = $props();

  let element: HTMLDivElement;

  onMount(() => {
    const leaf = new LeafGraph(`card:${item.id}`, element, {
      onEnter: (registerChild) => {
        onEnter?.(item, registerChild);
      }
    });
  });
</script>

<div bind:this={element} tabindex="0" role="button">
  <!-- card content -->
</div>
```

### Overlay Example

```typescript
// Creating overlay child graph
const card = new LeafGraph('home:card:5', element, {
  onEnter: (registerChild) => {
    const overlayGraph = new Graph('overlay', 'overlay:source:0');

    // Register overlay nodes
    const source0 = new LeafGraph('overlay:source:0', sourceEl0);
    const source1 = new LeafGraph('overlay:source:1', sourceEl1);
    overlayGraph.registerNode(source0.nodeId, source0);
    overlayGraph.registerNode(source1.nodeId, source1);
    overlayGraph.addConnection(source0.nodeId, 'down', source1.nodeId);
    overlayGraph.addConnection(source1.nodeId, 'up', source0.nodeId);

    // Register as child - escape will cleanup
    registerChild(overlayGraph);

    showOverlay();
  }
});
```

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
// Explicit connections
homeGraph.addConnection('card:0', 'right', 'card:1');
homeGraph.addConnection('card:1', 'down', 'details:back');

// Exact control over where navigation goes
```

### Trade-offs

| Aspect | Norigin | Our Library |
|--------|---------|-------------|
| Setup time | Fast (auto) | Medium (manual) |
| Control | Low | High |
| Edge cases | May choose wrong target | Always predictable |
| Dynamic layouts | Works automatically | Must update connections |
| Overlays/modals | Manual handling | Child graphs built-in |
| Complex flows | Can be confusing | Explicit and clear |

---

## Troubleshooting

### Navigation not working

1. Check `focusNavigator.root` is set (`registerRoot()` called)
2. Check `focusNavigator.overlayActive` is false
3. Verify nodes are registered: `graph.getNode(nodeId)` returns node
4. Check connections exist: `graph.connections` has entries

### Focus not moving visually

1. Ensure DOM elements have `tabindex="0"`
2. Ensure elements are focusable
3. Check `focus()` is being called on element

### Child graph not activating

1. Check prefix matches nodeId exactly or with `:` separator
2. Ensure child graph is registered before navigation
3. Verify `child.parent` is set correctly

### Memory leaks

1. Destroy child graphs on component unmount
2. Remove event listeners in cleanup
3. Call `focusNavigator.setOverlayActive(false)` on unmount

---

## License

MIT License - Free for personal and commercial use
