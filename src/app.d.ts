// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
declare global {
	namespace App {
		// interface Error {}
		// interface Locals {}
		// interface PageData {}
		// interface PageState {}
		// interface Platform {}
	}
}

interface TauriWindow {
	__TAURI__?: {
		tauri?: {
			invoke: <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>;
		};
	};
}

declare global {
	interface Window extends TauriWindow {}
}

export {};
