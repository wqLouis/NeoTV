import { writable } from 'svelte/store';
import type { rootNode } from './navGraph';

export const activeNavNode = writable<rootNode | null>(null);
