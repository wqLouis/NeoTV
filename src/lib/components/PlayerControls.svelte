<script lang="ts">
	import {
		Play,
		Pause,
		Volume2,
		VolumeX,
		Maximize,
		Minimize,
		List,
		ArrowLeft
	} from 'lucide-svelte';
	import { formatDuration } from '$lib/utils/format';

	interface Props {
		currentTime: number;
		duration: number;
		playing: boolean;
		muted: boolean;
		volume: number;
		fullscreen: boolean;
		showControls: boolean;
		showFullscreenButton: boolean;
		onReturn?: () => void;
		onTogglePlay: () => void;
		onSeek: (value: number) => void;
		onToggleMute: () => void;
		onVolumeChange: (value: number) => void;
		onToggleFullscreen: () => void;
		onTogglePopup: () => void;
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
		onReturn,
		onTogglePlay,
		onSeek,
		onToggleMute,
		onVolumeChange,
		onToggleFullscreen,
		onTogglePopup
	}: Props = $props();
</script>

<div
	class="absolute inset-0 z-50 flex flex-col justify-between transition-opacity duration-300 {showControls
		? 'opacity-100'
		: 'pointer-events-none opacity-0'}"
	onclick={(e) => e.stopPropagation()}
>
	<div class="bg-gradient-to-b from-black/70 to-transparent p-4">
		<div class="flex items-center justify-between text-white">
			<button
				class="flex h-8 w-8 cursor-pointer items-center justify-center rounded-full bg-black/40 transition-colors hover:bg-black/60"
				onclick={() => onReturn?.()}
			>
				<ArrowLeft class="h-4 w-4" />
			</button>
			<span class="text-sm">{formatDuration(currentTime)} / {formatDuration(duration)}</span>
			<button
				class="flex h-8 w-8 cursor-pointer items-center justify-center rounded-full bg-black/40 transition-colors hover:bg-black/60"
				onclick={onTogglePlay}
			>
				{#if playing}
					<Pause class="h-4 w-4" />
				{:else}
					<Play class="h-4 w-4" />
				{/if}
			</button>
		</div>
	</div>

	<div class="bg-gradient-to-t from-black/70 to-transparent p-4">
		<input
			type="range"
			min="0"
			max={duration || 100}
			value={currentTime}
			oninput={(e) => onSeek(parseFloat((e.target as HTMLInputElement).value))}
			class="mb-3 h-1 w-full cursor-pointer appearance-none rounded-lg bg-white/30 [&::-webkit-slider-thumb]:h-3 [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-white"
		/>

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
