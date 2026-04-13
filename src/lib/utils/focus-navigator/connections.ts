export type Direction = 'top' | 'bottom' | 'left' | 'right';

export interface Focusable {
	focus(): void;
}

export type NodeId = string;

export type MoveResult =
	| { status: 'moved'; nodeId: NodeId }
	| { status: 'boundary'; nodeId: NodeId; direction: Direction }
	| { status: 'none' };

export const END = 'END' as const;
