import { writable } from 'svelte/store';
import type { rootNode } from './Core';
import type { NavNode } from './Core';

export const activeNavNode = writable<rootNode | null>(null);

type BuildNavNodeFn = () => NavNode | null;
const pageNavBuilders = new Map<string, BuildNavNodeFn>();

export function registerPageNav(pathname: string, buildNavNode: BuildNavNodeFn): void {
	pageNavBuilders.set(pathname, buildNavNode);
}

export function unregisterPageNav(pathname: string): void {
	pageNavBuilders.delete(pathname);
}

export function getPageNavBuilder(pathname: string): BuildNavNodeFn | undefined {
	return pageNavBuilders.get(pathname);
}
