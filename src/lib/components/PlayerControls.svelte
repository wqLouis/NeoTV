<script lang="ts">
	import {
		Play,
		Pause,
		Volume2,
		VolumeX,
		Maximize,
		Minimize,
		List,
		ArrowLeft,
		Bug
	} from '@lucide/svelte';
	import { formatDuration } from '$lib/utils/format';

	interface CacheStats {
		count: number;
		bytes: number;
	}

	interface Props {
		currentTime: number;
		duration: number;
		playing: boolean;
		muted: boolean;
		volume: number;
		fullscreen: boolean;
		showControls: boolean;
		showFullscreenButton: boolean;
		showSettings?: boolean;
		seekingTime?: number;
		showDebug?: boolean;
		workerCount?: number;
		cacheStats?: CacheStats | null;
		onReturn?: () => void;
		onTogglePlay: () => void;
		onSeek: (value: number) => void;
		onToggleMute: () => void;
		onVolumeChange: (value: number) => void;
		onToggleFullscreen: () => void;
		onTogglePopup: () => void;
		onToggleDebug?: () => void;
		onUpdateCacheStats?: () => void;
		onSetWorkerCount?: (count: number) => void;
		onStopPreloader?: () => void;
	}

	let {
		currentTime,
		duration,
		playing,
		muted,
		volume,
		fullscreen,
		showControls,
		showFullscreenButton,
		showSettings = false,
		seekingTime,
		showDebug = false,
		workerCount = 6,
		cacheStats = null,
		onReturn,
		onTogglePlay,
		onSeek,
		onToggleMute,
		onVolumeChange,
		onToggleFullscreen,
		onTogglePopup,
		onToggleDebug,
		onUpdateCacheStats,
		onSetWorkerCount,
		onStopPreloader
	}: Props = $props();

	function formatBytes(bytes: number): string {
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
		if (bytes < 1024 * 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
		return `${(bytes / 1024 / 1024 / 1024).toFixed(2)} GB`;
	}
</script>

<div
	class="player-hud absolute inset-0 z-50 flex flex-col justify-between transition-opacity duration-300 {showControls
		? 'opacity-100'
		: 'pointer-events-none opacity-0'}"
	style={showSettings ? 'pointer-events: none' : ''}
	onclick={(e) => e.stopPropagation()}
	onkeydown={(e) => e.key === 'Escape' && e.stopPropagation()}
	role="presentation"
>
	<div class="bg-gradient-to-b from-black/70 to-transparent p-4">
		<div class="flex items-center justify-between text-white">
			<button
				class="flex h-8 w-8 cursor-pointer items-center justify-center rounded-full bg-black/40 transition-colors hover:bg-black/60"
				onclick={() => onReturn?.()}
			>
				<ArrowLeft class="h-4 w-4" />
			</button>
			<div class="flex items-center gap-2">
				<button
					class="flex h-12 w-12 cursor-pointer items-center justify-center rounded-full bg-black/40 transition-colors hover:bg-black/60"
					onclick={onTogglePlay}
				>
					{#if playing}
						<Pause class="h-6 w-6" />
					{:else}
						<Play class="h-6 w-6" />
					{/if}
				</button>
			</div>
		</div>
	</div>

	<div class="bg-gradient-to-t from-black/70 to-transparent p-4">
		<div class="mb-3 flex items-center gap-2">
			<input
				type="range"
				min="0"
				max={duration || 100}
				value={seekingTime ?? currentTime}
				oninput={(e) => {
					seekingTime = parseFloat((e.target as HTMLInputElement).value);
					onSeek(parseFloat((e.target as HTMLInputElement).value));
				}}
				onpointerdown={() => {
					seekingTime = currentTime;
				}}
				onpointerup={() => {
					seekingTime = undefined;
				}}
				onpointercancel={() => {
					seekingTime = undefined;
				}}
				class="h-2 flex-1 cursor-pointer appearance-none rounded-full bg-white/30 [&::-webkit-slider-thumb]:h-3 [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:cursor-grab [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-white"
			/>
			<span class="min-w-20 text-right text-sm text-white"
				>{formatDuration(seekingTime ?? currentTime)} / {formatDuration(duration)}</span
			>
		</div>

		<div class="flex items-center justify-between text-white">
			<div class="flex items-center gap-3">
				<button
					class="flex h-8 w-8 cursor-pointer items-center justify-center rounded-full bg-black/40 transition-colors hover:bg-black/60"
					onclick={onToggleMute}
				>
					{#if muted || volume === 0}
						<VolumeX class="h-4 w-4" />
					{:else}
						<Volume2 class="h-4 w-4" />
					{/if}
				</button>
				<input
					type="range"
					min="0"
					max="1"
					step="0.01"
					value={muted ? 0 : volume}
					oninput={(e) => onVolumeChange(parseFloat((e.target as HTMLInputElement).value))}
					class="h-1 w-20 cursor-pointer appearance-none rounded-lg bg-white/30 [&::-webkit-slider-thumb]:h-3 [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-white"
				/>
			</div>

			<div class="flex items-center gap-2">
				{#if showDebug}
					<div
						class="absolute right-4 bottom-16 z-50 w-64 rounded-lg bg-black/90 p-3 text-xs text-white"
					>
						<div class="mb-2 flex items-center justify-between">
							<span class="font-medium">Preloader Debug</span>
							<button class="text-white/60 hover:text-white" onclick={onToggleDebug}>✕</button>
						</div>

						<div class="mb-3 space-y-1">
							<div class="flex justify-between">
								<span class="text-white/60">Workers:</span>
								<span>{workerCount}</span>
							</div>
							<div class="flex justify-between">
								<span class="text-white/60">Cached Segments:</span>
								<span>{cacheStats?.count ?? 0}</span>
							</div>
							<div class="flex justify-between">
								<span class="text-white/60">Cache Size:</span>
								<span>{cacheStats ? formatBytes(cacheStats.bytes) : '0 B'}</span>
							</div>
						</div>

						<div class="mb-3 flex gap-1">
							<button
								class="flex-1 rounded bg-white/10 px-2 py-1 hover:bg-white/20"
								onclick={onUpdateCacheStats}
							>
								Refresh
							</button>
							<button
								class="flex-1 rounded bg-white/10 px-2 py-1 hover:bg-white/20"
								onclick={onStopPreloader}
							>
								Stop
							</button>
						</div>

						<div class="mb-2 flex flex-wrap gap-1">
							{#each [4, 6, 8, 12] as count}
								<button
									class="rounded px-2 py-1 text-xs {workerCount === count
										? 'bg-primary text-primary-foreground'
										: 'bg-white/10 hover:bg-white/20'}"
									onclick={() => onSetWorkerCount?.(count)}
								>
									{count}
								</button>
							{/each}
						</div>
					</div>
				{:else if onToggleDebug}
					<button
						class="flex h-8 w-8 cursor-pointer items-center justify-center rounded-full bg-black/40 transition-colors hover:bg-black/60"
						onclick={onToggleDebug}
					>
						<Bug class="h-4 w-4" />
					</button>
				{/if}
				<button
					class="flex h-8 w-8 cursor-pointer items-center justify-center rounded-full bg-black/40 transition-colors hover:bg-black/60"
					onclick={onTogglePopup}
				>
					<List class="h-4 w-4" />
				</button>
				{#if showFullscreenButton}
					<button
						class="flex h-8 w-8 cursor-pointer items-center justify-center rounded-full bg-black/40 transition-colors hover:bg-black/60"
						onclick={onToggleFullscreen}
					>
						{#if fullscreen}
							<Minimize class="h-4 w-4" />
						{:else}
							<Maximize class="h-4 w-4" />
						{/if}
					</button>
				{/if}
			</div>
		</div>
	</div>
</div>
