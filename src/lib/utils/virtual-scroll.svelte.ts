import { onMount } from 'svelte';

interface VirtualItem<T> {
	item: T;
	index: number;
	offsetTop: number;
	offsetLeft: number;
}

interface VirtualOptions<T> {
	items: T[];
	itemHeight: number;
	itemWidth: number;
	containerHeight: number;
	columns: number;
	buffer?: number;
}

export function createVirtualScroller<T>(options: VirtualOptions<T>) {
	const { items, itemHeight, itemWidth, containerHeight, columns, buffer = 3 } = options;

	const rowHeight = itemHeight;
	const totalRows = Math.ceil(items.length / columns);
	const totalHeight = totalRows * rowHeight;

	const startIndex = Math.max(
		0,
		Math.floor(containerHeight / rowHeight) * columns - buffer * columns
	);
	const endIndex = Math.min(
		items.length - 1,
		startIndex + Math.ceil(containerHeight / rowHeight) * columns + buffer * columns
	);

	const visibleItems: VirtualItem<T>[] = [];

	for (let i = startIndex; i <= endIndex; i++) {
		const row = Math.floor(i / columns);
		const col = i % columns;
		visibleItems.push({
			item: items[i],
			index: i,
			offsetTop: row * rowHeight,
			offsetLeft: col * itemWidth
		});
	}

	return {
		visibleItems,
		totalHeight,
		startIndex,
		endIndex,
		itemWidth,
		itemHeight
	};
}

export class VirtualList<T> {
	private items: T[];
	private itemHeight: number;
	private itemWidth: number;
	private containerHeight: number;
	private columns: number;
	private buffer: number;
	private scrollTop = 0;
	private containerRef: HTMLDivElement | null = null;

	constructor(
		items: T[],
		itemHeight: number,
		itemWidth: number,
		columns: number = 1,
		buffer: number = 3
	) {
		this.items = items;
		this.itemHeight = itemHeight;
		this.itemWidth = itemWidth;
		this.containerHeight = 0;
		this.columns = columns;
		this.buffer = buffer;
	}

	setContainer(el: HTMLDivElement) {
		this.containerRef = el;
		this.containerHeight = el.clientHeight;
	}

	updateItems(items: T[]) {
		this.items = items;
	}

	getVisibleRange(): { start: number; end: number } {
		if (!this.containerRef) {
			return { start: 0, end: this.items.length };
		}

		this.scrollTop = this.containerRef.scrollTop;
		const viewHeight = this.containerRef.clientHeight;

		const startRow = Math.max(0, Math.floor(this.scrollTop / this.itemHeight) - this.buffer);
		const endRow = Math.min(
			Math.ceil(this.items.length / this.columns) - 1,
			Math.floor((this.scrollTop + viewHeight) / this.itemHeight) + this.buffer
		);

		const start = startRow * this.columns;
		const end = Math.min(this.items.length, (endRow + 1) * this.columns);

		return { start, end };
	}

	getTotalHeight(): number {
		return Math.ceil(this.items.length / this.columns) * this.itemHeight;
	}

	getItems(): T[] {
		return this.items;
	}
}

export function useVirtualScroll<T>(
	items: () => T[],
	itemHeight: number,
	itemWidth: number,
	columns: number = 1,
	bufferSize: number = 3
) {
	let containerRef: HTMLDivElement | null = $state(null);
	let scrollTop = $state(0);
	let containerHeight = $state(0);

	const totalRows = $derived(Math.ceil(items().length / columns));
	const totalHeight = $derived(totalRows * itemHeight);

	const visibleRange = $derived.by(() => {
		if (!containerRef) return { start: 0, end: items().length };

		const startRow = Math.max(0, Math.floor(scrollTop / itemHeight) - bufferSize);
		const endRow = Math.min(
			totalRows - 1,
			Math.floor((scrollTop + containerHeight) / itemHeight) + bufferSize
		);

		const start = startRow * columns;
		const end = Math.min(items().length, (endRow + 1) * columns);

		return { start, end };
	});

	const visibleItems = $derived.by(() => {
		const result: { item: T; index: number; style: string }[] = [];
		const { start, end } = visibleRange;

		for (let i = start; i < end; i++) {
			const row = Math.floor(i / columns);
			const col = i % columns;

			result.push({
				item: items()[i],
				index: i,
				style: `position: absolute; top: ${row * itemHeight}px; left: ${col * itemWidth}px; width: ${itemWidth}px; height: ${itemHeight}px;`
			});
		}

		return result;
	});

	function handleScroll(e: Event) {
		const target = e.target as HTMLDivElement;
		scrollTop = target.scrollTop;
	}

	function setContainer(el: HTMLDivElement) {
		containerRef = el;
		containerHeight = el.clientHeight;
	}

	return {
		visibleItems,
		totalHeight,
		handleScroll,
		setContainer
	};
}
