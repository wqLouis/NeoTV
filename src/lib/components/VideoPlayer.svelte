<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import Hls from 'hls.js';
	import { Play, Pause, Volume2, VolumeX, Maximize, Minimize } from 'lucide-svelte';

	interface Props {
		src: string;
		type?: 'native' | 'hls';
		autoplay?: boolean;
		poster?: string;
		topControls?: import('svelte').Snippet;
		bottomControls?: import('svelte').Snippet;
		onTimeUpdate?: (currentTime: number, duration: number) => void;
		onEnded?: () => void;
		onError?: (error: string) => void;
		onReady?: () => void;
	}

	let {
		src,
		type = 'native',
		autoplay = true,
		poster = '',
		topControls,
		bottomControls,
		onTimeUpdate,
		onEnded,
		onError,
		onReady
	}: Props = $props();

	let videoEl: HTMLVideoElement;
	let containerEl: HTMLDivElement;
	let hls: Hls | null = null;

	let playing = $state(false);
	let currentTime = $state(0);
	let duration = $state(0);
	let buffered = $state(0);
	let volume = $state(1);
	let muted = $state(false);
	let fullscreen = $state(false);
	let showControls = $state(true);
	let controlsTimeout: ReturnType<typeof setTimeout>;
	let loading = $state(true);
	let error = $state<string | null>(null);
	let localCurrentTime = $state(0);

	function showControlsTemporarily() {
		showControls = true;
		clearTimeout(controlsTimeout);
		controlsTimeout = setTimeout(() => {
			if (!videoEl?.paused) {
				showControls = false;
			}
		}, 3000);
	}

	function togglePlay() {
		if (!videoEl) return;
		if (videoEl.paused) {
			videoEl.play();
		} else {
			videoEl.pause();
		}
		showControlsTemporarily();
	}

	function toggleMute() {
		if (!videoEl) return;
		videoEl.muted = !videoEl.muted;
		muted = videoEl.muted;
	}

	function handleVolumeChange(e: Event) {
		const target = e.target as HTMLInputElement;
		const value = parseFloat(target.value);
		if (videoEl) {
			videoEl.volume = value;
			volume = value;
			muted = value === 0;
		}
	}

	function toggleFullscreen() {
		if (!containerEl) return;
		if (document.fullscreenElement) {
			document.exitFullscreen();
		} else {
			containerEl.requestFullscreen();
		}
	}

	function handleSeek(e: Event) {
		const target = e.target as HTMLInputElement;
		const value = parseFloat(target.value);
		if (videoEl) {
			videoEl.currentTime = value;
			localCurrentTime = value;
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		switch (e.key) {
			case ' ':
			case 'k':
				e.preventDefault();
				togglePlay();
				break;
			case 'm':
				e.preventDefault();
				toggleMute();
				break;
			case 'f':
				e.preventDefault();
				toggleFullscreen();
				break;
			case 'ArrowUp':
				e.preventDefault();
				if (videoEl && volume < 1) {
					videoEl.volume = Math.min(1, volume + 0.1);
					volume = videoEl.volume;
				}
				break;
			case 'ArrowDown':
				e.preventDefault();
				if (videoEl && volume > 0) {
					videoEl.volume = Math.max(0, volume - 0.1);
					volume = videoEl.volume;
				}
				break;
		}
		showControlsTemporarily();
	}

	function initHls() {
		if (type !== 'hls' || !src) return;

		if (Hls.isSupported()) {
			hls = new Hls({
				enableWorker: true,
				lowLatencyMode: true
			});
			hls.loadSource(src);
			hls.attachMedia(videoEl);
			hls.on(Hls.Events.MANIFEST_PARSED, () => {
				loading = false;
				if (autoplay) {
					videoEl.play().catch(() => {});
				}
				onReady?.();
			});
			hls.on(Hls.Events.ERROR, (_, data) => {
				if (data.fatal) {
					error = '视频加载失败';
					onError?.('HLS load error: ' + data.type);
				}
			});
		} else if (videoEl.canPlayType('application/vnd.apple.mpegurl')) {
			videoEl.src = src;
			loading = false;
			if (autoplay) {
				videoEl.play().catch(() => {});
			}
			onReady?.();
		} else {
			error = '您的浏览器不支持此视频格式';
			onError?.('HLS not supported');
		}
	}

	onMount(() => {
		if (type === 'native' && src) {
			loading = false;
			if (autoplay) {
				videoEl.play().catch(() => {});
			}
			onReady?.();
		} else if (type === 'hls') {
			initHls();
		}
	});

	onDestroy(() => {
		clearTimeout(controlsTimeout);
		if (hls) {
			hls.destroy();
		}
	});

	function formatTime(seconds: number): string {
		if (isNaN(seconds)) return '00:00';
		const h = Math.floor(seconds / 3600);
		const m = Math.floor((seconds % 3600) / 60);
		const s = Math.floor(seconds % 60);
		if (h > 0) {
			return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
		}
		return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
	}

	function handleTimeUpdate() {
		if (videoEl) {
			currentTime = videoEl.currentTime;
			localCurrentTime = videoEl.currentTime;
			duration = videoEl.duration || 0;

			if (videoEl.buffered.length > 0) {
				buffered = videoEl.buffered.end(videoEl.buffered.length - 1);
			}

			onTimeUpdate?.(currentTime, duration);
		}
	}

	function handlePlay() {
		playing = true;
		showControlsTemporarily();
	}

	function handlePause() {
		playing = false;
		showControls = true;
	}

	function handleEnded() {
		playing = false;
		onEnded?.();
	}

	function handleError() {
		error = '视频播放失败';
		onError?.('Video playback error');
	}

	function handleCanPlay() {
		loading = false;
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<div
	class="relative h-full w-full bg-black"
	bind:this={containerEl}
	onmousemove={showControlsTemporarily}
	onclick={togglePlay}
	role="button"
	tabindex="0"
>
	{#if loading}
		<div class="absolute inset-0 flex items-center justify-center">
			<div class="h-12 w-12 animate-spin rounded-full border-b-2 border-white"></div>
		</div>
	{/if}

	{#if error}
		<div class="absolute inset-0 flex flex-col items-center justify-center text-white">
			<p class="text-lg">{error}</p>
		</div>
	{/if}

	<video
		bind:this={videoEl}
		src={type === 'native' ? src : undefined}
		{poster}
		class="h-full w-full"
		playsinline
		onplay={handlePlay}
		onpause={handlePause}
		ontimeupdate={handleTimeUpdate}
		onended={handleEnded}
		onerror={handleError}
		oncanplay={handleCanPlay}
	>
		<track kind="captions" />
	</video>

	<div
		class="absolute inset-0 flex flex-col justify-between transition-opacity duration-300 {showControls
			? 'opacity-100'
			: 'pointer-events-none opacity-0'}"
		onclick={(e) => e.stopPropagation()}
	>
		<div class="bg-gradient-to-b from-black/70 to-transparent p-4">
			{@render topControls?.()}
		</div>

		<div class="flex items-center justify-center">
			<button
				class="rounded-full p-4 transition-colors hover:bg-white/20"
				onclick={(e) => {
					e.stopPropagation();
					togglePlay();
				}}
			>
				{#if playing}
					<Pause class="h-12 w-12 text-white" />
				{:else}
					<Play class="h-12 w-12 text-white" />
				{/if}
			</button>
		</div>

		<div class="bg-gradient-to-t from-black/70 to-transparent p-4">
			<div class="mb-2 flex items-center gap-2">
				<span class="text-sm text-white">
					{formatTime(currentTime)} / {formatTime(duration)}
				</span>

				<input
					type="range"
					min="0"
					max={duration || 100}
					value={localCurrentTime}
					oninput={handleSeek}
					class="h-1 flex-1 cursor-pointer appearance-none rounded-lg bg-white/30 [&::-webkit-slider-thumb]:h-3 [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-white"
				/>
			</div>

			<div class="flex items-center justify-between">
				<div class="flex items-center gap-2">
					<button
						class="text-white transition-colors hover:text-white/80"
						onclick={(e) => {
							e.stopPropagation();
							toggleMute();
						}}
					>
						{#if muted || volume === 0}
							<VolumeX class="h-5 w-5" />
						{:else}
							<Volume2 class="h-5 w-5" />
						{/if}
					</button>
					<input
						type="range"
						min="0"
						max="1"
						step="0.01"
						value={muted ? 0 : volume}
						oninput={handleVolumeChange}
						class="h-1 w-20 cursor-pointer appearance-none rounded-lg bg-white/30 [&::-webkit-slider-thumb]:h-3 [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-white"
					/>
				</div>

				<div class="flex items-center gap-2">
					{#if bottomControls}
						{@render bottomControls()}
					{/if}

					<button
						class="text-white transition-colors hover:text-white/80"
						onclick={(e) => {
							e.stopPropagation();
							toggleFullscreen();
						}}
					>
						{#if fullscreen}
							<Minimize class="h-5 w-5" />
						{:else}
							<Maximize class="h-5 w-5" />
						{/if}
					</button>
				</div>
			</div>
		</div>
	</div>
</div>
