# Svelte Focus Navigator

A recursive, graph-based navigation system for TV remote controls and D-pad interfaces.

## Documentation

- [API Reference](./LIBRARY_API.md) - Core concepts, types, and interface definitions
- [Examples](./LIBRARY_EXAMPLES.md) - Integration examples for Vanilla JS, Svelte, and overlays

## Quick Start

```bash
# Install
bun add svelte-focus-navigator

# Build
bun run build

# Test
bun test
```

## Key Features

- **Declarative**: Components self-declare their navigation role (leaf/graph/none)
- **Abstraction**: `move()` function is abstracted - any navigation algorithm works
- **Recursive**: Navigation bubbles from child to parent graph when boundaries are hit
- **4-direction**: All `UP/DOWN/LEFT/RIGHT` passed to move()
- **Independent focus ring**: Visual ring is decoupled from navigation tree

## Project Structure

```
svelte-focus-navigator/
├── src/
│   ├── connections.ts      # Core types
│   ├── graph.ts           # Graph class
│   ├── leaf.ts            # LeafGraph class
│   ├── navigator.ts       # FocusNavigator singleton
│   └── index.ts           # Main exports
│
├── svelte/
│   ├── action.ts          # use:focus directive
│   └── focus-ring.svelte  # Independent focus ring
│
├── dist/                  # Built output
├── examples/              # Example apps
└── tests/                 # Unit tests
```

## License

MIT License
