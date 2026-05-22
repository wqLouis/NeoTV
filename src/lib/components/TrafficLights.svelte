<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { X, Minus, Maximize2, Minimize2 } from '@lucide/svelte';

	let isMaximized = $state(false);

	async function checkMaximized() {
		try {
			isMaximized = await invoke<boolean>('window_is_maximized');
		} catch {
			isMaximized = await getCurrentWindow().isMaximized();
		}
	}

	async function handleMinimize() {
		try {
			await invoke('window_minimize');
		} catch {
			await getCurrentWindow().minimize();
		}
	}

	async function handleMaximize() {
		try {
			await invoke('window_maximize');
		} catch {
			const win = getCurrentWindow();
			if (await win.isMaximized()) {
				await win.unmaximize();
			} else {
				await win.maximize();
			}
		}
		isMaximized = !isMaximized;
	}

	async function handleClose() {
		try {
			await invoke('window_close');
		} catch {
			await getCurrentWindow().close();
		}
	}

	async function startDrag(e: MouseEvent) {
		if ((e.target as HTMLElement).closest('.traffic-btn')) return;
		try {
			await getCurrentWindow().startDragging();
		} catch {
			// Fallback
		}
	}

	getCurrentWindow().onResized(() => {
		checkMaximized();
	});

	$effect(() => {
		checkMaximized();
	});
</script>

<div
	class="group fixed top-2 left-4 z-[9999] flex cursor-grab items-center gap-2 rounded-2xl border bg-background/80 px-3 py-2 backdrop-blur-md opacity-0 transition-opacity duration-300 hover:opacity-100 select-none"
	onmousedown={startDrag}
	role="banner"
>
	<button
		class="traffic-btn group flex size-4 items-center justify-center rounded-full bg-[#ff5f57] transition-all hover:bg-[#ff5f57]/80 active:scale-90"
		onclick={handleClose}
		aria-label="关闭"
	>
		<X class="size-3 text-[#b5301d] opacity-0 transition-opacity group-hover:opacity-100" />
	</button>

	<button
		class="traffic-btn group flex size-4 items-center justify-center rounded-full bg-[#febc2e] transition-all hover:bg-[#febc2e]/80 active:scale-90"
		onclick={handleMinimize}
		aria-label="最小化"
	>
		<Minus class="size-3 text-[#9f5c04] opacity-0 transition-opacity group-hover:opacity-100" />
	</button>

	<button
		class="traffic-btn group flex size-4 items-center justify-center rounded-full bg-[#28c840] transition-all hover:bg-[#28c840]/80 active:scale-90"
		onclick={handleMaximize}
		aria-label={isMaximized ? '还原' : '最大化'}
	>
		{#if isMaximized}
			<Minimize2 class="size-3 text-[#187216] opacity-0 transition-opacity group-hover:opacity-100" />
		{:else}
			<Maximize2 class="size-3 text-[#187216] opacity-0 transition-opacity group-hover:opacity-100" />
		{/if}
	</button>
</div>
