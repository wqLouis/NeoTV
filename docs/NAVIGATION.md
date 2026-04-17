# TV Navigation System

NeoTV uses **lrud-spatial** (BBC) for spatial navigation on TV remotes and directional inputs. This document explains how the navigation system works.

## Overview

lrud-spatial automatically finds the best focusable element based on spatial position. Unlike explicit navigation graphs (where you manually connect nodes), lrud-spatial calculates which element is closest in the direction you're moving.

## How It Works

```
User presses ArrowRight
        │
        ▼
getNextFocus(currentElement, 'ArrowRight')
        │
        ├── Find current element's position
        ├── Get all focusable elements on page
        ├── Filter to elements in the "right" direction
        ├── Sort by distance (closest first)
        │
        ▼
Focus moves to nearest element
```

## Enabling TV Navigation

TV navigation is enabled by default. Toggle it in:

**Settings → 外观 → TV 导航模式**

When disabled:

- Arrow keys scroll the page normally
- Focus ring is hidden
- Standard keyboard tab navigation works

## Focusable Elements

lrud-spatial automatically detects these as focusable:

| Selector                         | Description               |
| -------------------------------- | ------------------------- |
| `[tabindex]` where tabindex >= 0 | Custom focusable elements |
| `a`                              | Links                     |
| `button`                         | Buttons                   |
| `input`                          | Input fields              |

**Note:** `tabindex="-1"` is explicitly excluded (used for elements that should be focusable programmatically but not via tab).

### Making Cards Focusable

Cards in NeoTV (DoubanCard) use `tabindex="0"` to be focusable:

```svelte
<div
    role="button"
    tabindex="0"
    onkeydown={handleKeydown}
>
```

The `onkeydown` handler enables activation with Enter/Space keys.

## Focus Ring

The visual focus indicator is provided by `FocusRing.svelte`:

- White 3px border
- 4px padding around focused element
- Smooth position transitions
- Hidden when TV navigation is disabled

The focus ring automatically tracks `focusin`/`focusout` events on the page.

## Container Awareness

lrud-spatial recognizes these as navigation containers:

| Selector          | Element Type           |
| ----------------- | ---------------------- |
| `nav`             | Navigation elements    |
| `section`         | Section elements       |
| `.lrud-container` | Custom container class |

### Horizontal Scrolling

Horizontal sections (like HorizontalSection) use `overflow-x-auto`. lrud-spatial handles this automatically - when focus reaches the edge and you press the direction again, it moves to the next row.

## Advanced: Container Attributes

For fine-grained control, elements can use these data attributes:

### data-block-exit

Prevent focus from leaving a container in specific directions:

```html
<div class="lrud-container" data-block-exit="up down"></div>
```

When focus is on an element inside this container, pressing Up or Down won't move focus outside.

### data-lrud-consider-container-distance

Include container boundaries in distance calculations:

```html
<section data-lrud-consider-container-distance></section>
```

### data-lrud-overlap-threshold

Adjust overlap tolerance for an element (0.0 to 1.0):

```html
<div class="card" data-lrud-overlap-threshold="0.5"></div>
```

### lrud-ignore

Exclude elements from being focusable:

```html
<div class="lrud-ignore">
	<!-- Nothing inside this will be focusable -->
</div>
```

## Navigation Key Mappings

| Key        | Direction |
| ---------- | --------- |
| ArrowUp    | up        |
| ArrowDown  | down      |
| ArrowLeft  | left      |
| ArrowRight | right     |

## Implementation Details

### Layout Integration

TV navigation is handled in `src/routes/+layout.svelte`:

```typescript
import { getNextFocus } from '@bbc/tv-lrud-spatial';

function handleKeydown(e: KeyboardEvent) {
	if (!settingsStore.tvNavModeEnabled) return;

	const dir = e.key;
	if (dir !== 'ArrowUp' && dir !== 'ArrowDown' && dir !== 'ArrowLeft' && dir !== 'ArrowRight')
		return;

	e.preventDefault();
	const next = getNextFocus(document.activeElement, dir as any);
	if (next) {
		next.focus();
	}
}
```

### FocusRing Component

The FocusRing component (`src/lib/components/FocusRing.svelte`) uses `requestAnimationFrame` for smooth tracking:

```typescript
document.addEventListener('focusin', updateRing);
document.addEventListener('focusout', updateRing);
```

### Settings Integration

TV navigation mode is stored in `settingsStore`:

```typescript
// src/lib/stores/settings.svelte.ts
interface Settings {
	tvNavModeEnabled: boolean;
	// ...
}
```

## Browser Compatibility

lrud-spatial requires:

- `document.activeElement`
- `element.getBoundingClientRect()`
- `requestAnimationFrame`

Supported in all modern browsers. Works on desktop browsers and TV webviews.

## Troubleshooting

### Focus not moving in expected direction

1. Check that target elements are focusable (have `tabindex="0"`, are `button`/`a`/`input`, or have `role="button"`)
2. Check no parent has `lrud-ignore` class
3. Verify TV Nav Mode is enabled in settings

### Focus moves to wrong element

This is expected behavior - lrud-spatial uses spatial sorting, so "closest" may not be visually obvious. The algorithm finds the element whose edge is nearest in the movement direction.

### Focus ring not visible

1. Verify TV Nav Mode is enabled
2. Check that FocusRing component is mounted (in +layout.svelte)
3. Ensure focused element has visible dimensions (not `display: none`)

## Related Files

| File                                                   | Purpose                   |
| ------------------------------------------------------ | ------------------------- |
| `src/routes/+layout.svelte`                            | Navigation event handling |
| `src/lib/components/FocusRing.svelte`                  | Visual focus indicator    |
| `src/lib/stores/settings.svelte.ts`                    | TV Nav Mode setting       |
| `src/lib/components/DoubanCard.svelte`                 | Focusable card example    |
| `src/lib/components/business/ToggleButtonGroup.svelte` | Focusable button group    |
