<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import Hls from 'hls.js';
	import type {
		Loader,
		LoaderContext,
		LoaderStats,
		LoaderConfiguration,
		LoaderCallbacks
	} from 'hls.js';
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

	interface Episode {
		episode: string;
		url: string;
	}

	interface Props {
		src: string;
		type?: 'native' | 'hls';
		autoplay?: boolean;
		poster?: string;
		episodes?: Episode[];
		currentEpisodeIndex?: number;
		playbackRate?: number;
		availableSources?: { source_code: string; source_name: string }[];
		showFullscreenButton?: boolean;
		onTimeUpdate?: (currentTime: number, duration: number) => void;
		onEnded?: () => void;
		onError?: (error: string) => void;
		onReady?: () => void;
		onEpisodeChange?: (episode: Episode, index: number) => void;
		onPlaybackRateChange?: (rate: number) => void;
		onReturn?: () => void;
		onSourceChange?: (source: { source_code: string; [key: string]: unknown }) => void;
	}

	let {
		src,
		type = 'native',
		autoplay = true,
		poster = '',
		episodes = [],
		currentEpisodeIndex = 0,
		playbackRate = 1,
		availableSources = [],
		showFullscreenButton = true,
		onTimeUpdate,
		onEnded,
		onError,
		onReady,
		onEpisodeChange,
		onPlaybackRateChange,
		onReturn,
		onSourceChange
	}: Props = $props();

	let videoEl: HTMLVideoElement;
	let containerEl: HTMLDivElement;
	let hls: Hls | null = null;

	async function checkMediaCapabilities(): Promise<{
		video: string[];
		audio: string[];
		supported: boolean;
	}> {
		const result = { video: [] as string[], audio: [] as string[], supported: false };

		if (!('mediaCapabilities' in navigator)) {
			console.warn('[HLS] MediaCapabilities API not available');
			return result;
		}

		const mediaCapabilities = (
			navigator as Navigator & {
				mediaCapabilities: {
					decodingInfo: (config: {
						type: string;
						video?: { contentType: string; width: number; height: number };
						audio?: { contentType: string };
					}) => Promise<{ supported: boolean }>;
				};
			}
		).mediaCapabilities;

		const testConfigs = [
			{
				type: 'media-source',
				video: { contentType: 'video/mp4; codecs="avc1.640028"', width: 1920, height: 1080 },
				audio: undefined
			},
			{
				type: 'media-source',
				video: { contentType: 'video/mp4; codecs="avc1.42E01E"', width: 1280, height: 720 },
				audio: undefined
			},
			{
				type: 'media-source',
				audio: { contentType: 'audio/mp4; codecs="mp4a.40.2"' },
				video: undefined
			},
			{
				type: 'media-source',
				audio: { contentType: 'audio/mp4; codecs="mp4a.40.5"' },
				video: undefined
			},
			{
				type: 'media-source',
				audio: { contentType: 'audio/mp4; codecs="mp4a.40.7"' },
				video: undefined
			},
			{
				type: 'media-source',
				audio: { contentType: 'audio/webm; codecs="opus"' },
				video: undefined
			},
			{ type: 'media-source', audio: { contentType: 'audio/mp3' }, video: undefined }
		];

		for (const config of testConfigs) {
			try {
				const info = await mediaCapabilities.decodingInfo(config);
				if (info.supported) {
					if (config.video) {
						result.video.push(config.video.contentType);
					}
					if (config.audio) {
						result.audio.push(config.audio.contentType);
					}
				}
			} catch (e) {
				// Ignore individual config errors
			}
		}

		result.supported = result.video.length > 0 || result.audio.length > 0;
		console.log('[HLS] MediaCapabilities check:', result);
		return result;
	}

	class SegmentLoader {
		constructor(config: { debug: boolean }) {}
		load(
			context: { type: string; url: string },
			callbacks: {
				onSuccess: (response: { data: ArrayBuffer | string; code: number; text: string }) => void;
				onError: (error: { code: number; text: string }) => void;
			}
		): void {
			const { type, url } = context;

			if (type === 'manifest' || type === 'level' || type === 'audioTrack' || type === 'subtitle') {
				fetch(url)
					.then((r) => r.text())
					.then((text) => callbacks.onSuccess({ data: text, code: 200, text }))
					.catch((e) => callbacks.onError({ code: 0, text: String(e) }));
				return;
			}

			invoke<number[]>('fetch_media_segment', { url })
				.then((data) => {
					const buffer = new Uint8Array(data).buffer;
					callbacks.onSuccess({ data: buffer, code: 200, text: '' });
				})
				.catch((e) => {
					console.error('[HLS] Segment load error:', e);
					callbacks.onError({ code: 0, text: String(e) });
				});
		}
		static destroy() {}
	}

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
	let showPopup = $state(false);
	let localPlaybackRate = $state(1);

	const SPEED_OPTIONS = [0.5, 0.75, 1, 1.25, 1.5, 2] as const;

	let lastClickTime = 0;
	let lastClickTarget: HTMLElement | null = null;

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

	function handleVideoClick(e: MouseEvent) {
		const now = Date.now();
		if (now - lastClickTime < 300 && lastClickTarget === e.target) {
			return;
		}
		lastClickTime = now;
		lastClickTarget = e.target as HTMLElement;
		togglePlay();
	}

	function handleVideoDoubleClick(e: MouseEvent) {
		const rect = containerEl.getBoundingClientRect();
		const x = e.clientX - rect.left;
		const isLeftSide = x < rect.width / 2;

		if (videoEl) {
			if (isLeftSide) {
				videoEl.currentTime = Math.max(0, videoEl.currentTime - 5);
			} else {
				videoEl.currentTime = Math.min(duration, videoEl.currentTime + 5);
			}
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

	function setPlaybackRate(rate: number) {
		localPlaybackRate = rate;
		if (videoEl) {
			videoEl.playbackRate = rate;
		}
		onPlaybackRateChange?.(rate);
	}

	function handleEpisodeSelect(episode: Episode, index: number) {
		showPopup = false;
		onEpisodeChange?.(episode, index);
	}

	class RustLoader implements Loader<LoaderContext> {
		context: LoaderContext | null = null;
		stats: LoaderStats = {
			aborted: false,
			loaded: 0,
			retry: 0,
			total: 0,
			chunkCount: 0,
			bwEstimate: 0,
			loading: { start: 0, end: 0, first: 0 },
			parsing: { start: 0, end: 0 },
			buffering: { start: 0, end: 0, first: 0 }
		};
		private config: any;
		private abortController: AbortController | null = null;

		constructor(config: any) {
			this.config = config;
		}

		destroy(): void {
			this.abortController?.abort();
			this.context = null;
		}

		abort(): void {
			this.abortController?.abort();
			this.stats.aborted = true;
		}

		load(
			context: LoaderContext,
			config: LoaderConfiguration,
			callbacks: LoaderCallbacks<LoaderContext>
		): void {
			this.context = context;
			this.stats.loading.start = performance.now();
			this.stats.aborted = false;
			this.abortController = new AbortController();

			const ctx = context as LoaderContext & { type: string };
			const { type, url } = ctx;

			if (type === 'manifest' || type === 'level' || type === 'audioTrack' || type === 'subtitle') {
				console.log('[RustLoader] Fetching m3u8:', url);
				invoke<string>('fetch_hls_m3u8', { url })
					.then((content) => {
						this.stats.loading.end = performance.now();
						this.stats.loaded = content.length;
						this.stats.total = content.length;
						callbacks.onSuccess(
							{ code: 200, data: content, url: url as string },
							this.stats,
							context,
							null
						);
					})
					.catch((e) => {
						callbacks.onError({ code: 500, text: String(e) }, context, null, this.stats);
					});
			} else {
				console.log('[RustLoader] Fetching segment:', url);
				invoke<number[]>('fetch_hls_segment', { url })
					.then((data) => {
						const buffer = new Uint8Array(data).buffer;
						this.stats.loading.end = performance.now();
						this.stats.loaded = data.length;
						this.stats.total = data.length;
						callbacks.onSuccess(
							{ code: 200, data: buffer, url: url as string },
							this.stats,
							context,
							null
						);
					})
					.catch((e) => {
						callbacks.onError({ code: 500, text: String(e) }, context, null, this.stats);
					});
			}
		}

		getCacheAge?(): number | null {
			return null;
		}

		getResponseHeader?(name: string): string | null {
			return null;
		}
	}

	async function initHls() {
		console.log('[HLS] initHls called:', { type, src });

		if (type !== 'hls') {
			console.log('[HLS] type is not hls, skipping');
			return;
		}

		if (!src) {
			console.log('[HLS] src is empty, skipping');
			return;
		}

		if (Hls.isSupported()) {
			console.log('[HLS] Hls.isSupported() = true, creating custom loader');

			hls = new Hls({
				enableWorker: true,
				lowLatencyMode: false,
				debug: true,
				preferManagedMediaSource: false,
				enableSoftwareAES: true,
				stretchShortVideoTrack: true,
				defaultAudioCodec: 'mp4a.40.2',
				loader: RustLoader
			});

			hls.on(Hls.Events.ERROR, (_, data) => {
				console.error('[HLS] Error:', data);
				if (data.fatal && hls) {
					switch (data.type) {
						case Hls.ErrorTypes.NETWORK_ERROR:
							console.warn('[HLS] Network error, trying transcoder...');
							hls.destroy();
							hls = null;
							initTranscoded();
							return;
						case Hls.ErrorTypes.MEDIA_ERROR:
							if (data.details === Hls.ErrorDetails.BUFFER_ADD_CODEC_ERROR) {
								console.warn('[HLS] Codec not supported by MSE, trying Rust transcoder...');
								hls.destroy();
								hls = null;
								initTranscoded();
								return;
							}
							if (data.details === Hls.ErrorDetails.MANIFEST_INCOMPATIBLE_CODECS_ERROR) {
								console.warn('[HLS] Manifest incompatibility, trying transcoder...');
								hls.destroy();
								hls = null;
								initTranscoded();
								return;
							}
							hls.recoverMediaError();
							return;
						default:
							error = '视频播放失败';
							break;
					}
					onError?.('HLS load error: ' + data.type);
				}
			});

			console.log('[HLS] Loading source via Rust loader:', src);
			hls.loadSource(src);
			hls.attachMedia(videoEl);
			hls.on(Hls.Events.MANIFEST_PARSED, () => {
				console.log('[HLS] MANIFEST_PARSED event fired');
				loading = false;
				if (autoplay) {
					videoEl.play().catch(() => {});
				}
				onReady?.();
			});
		} else {
			console.log('[HLS] Hls.isSupported() = false, trying transcoder');
			initTranscoded();
		}
	}

	function initNative() {
		console.log('[HLS] initNative called:', { src, autoplay });
		if (!src) return;
		loading = false;
		videoEl.src = src;
		if (autoplay) {
			videoEl.play().catch(() => {});
		}
		onReady?.();
	}

	async function initTranscoded() {
		console.log('[HLS] initTranscoded called, src:', src);
		if (!src) return;

		loading = true;
		error = null;

		try {
			console.log('[HLS] Checking transcoder system...');
			const sysInfo = await invoke<{
				vaapi_available: boolean;
				ffmpeg_available: boolean;
			}>('check_transcoder');
			console.log('[HLS] Transcoder system info:', sysInfo);

			if (!sysInfo.ffmpeg_available) {
				error = '系统缺少 ffmpeg，无法转码播放';
				loading = false;
				return;
			}

			const streamId = `stream_${Date.now()}`;
			console.log('[HLS] Starting transcoded stream:', streamId);

			const streamInfo = await invoke<{
				url: string;
				port: number;
				duration: number | null;
				vaapi_available: boolean;
				ffmpeg_available: boolean;
			}>('start_transcoded_stream', {
				id: streamId,
				m3u8Url: src,
				referer: null
			});

			console.log('[HLS] Transcoded stream URL:', streamInfo.url, 'Duration:', streamInfo.duration);
			loading = false;
			videoEl.src = streamInfo.url;

			if (streamInfo.duration) {
				videoEl.addEventListener(
					'loadedmetadata',
					() => {
						if (videoEl.duration === Infinity || isNaN(videoEl.duration)) {
							Object.defineProperty(videoEl, 'duration', {
								value: streamInfo.duration,
								writable: true
							});
							console.log('[HLS] Duration set from M3U8:', streamInfo.duration);
						}
					},
					{ once: true }
				);
			}

			if (autoplay) {
				videoEl.play().catch((e) => {
					console.error('[HLS] Play error:', e);
				});
			}

			onReady?.();
		} catch (e) {
			console.error('[HLS] Transcoder error:', e);
			error = '视频转码失败: ' + String(e);
			loading = false;
		}
	}

	onMount(() => {
		console.log('[HLS] onMount called:', { type, src, hasVideoEl: !!videoEl });
		if (type === 'native' && src) {
			console.log('[HLS] Using native player');
			loading = false;
			if (autoplay) {
				videoEl.play().catch(() => {});
			}
			onReady?.();
		} else if (type === 'hls') {
			console.log('[HLS] Calling initHls...');
			initHls(); // async but don't await - let it run
		} else {
			console.log('[HLS] onMount: no conditions matched, type:', type, 'src:', src);
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
		console.error('[HLS] handleError called, video error:', videoEl?.error);
		error = '视频播放失败';
		onError?.('Video playback error');
	}

	function handleCanPlay() {
		loading = false;
	}

	function closePopup() {
		showPopup = false;
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if showPopup}
	<button
		class="fixed inset-0 z-30 cursor-default bg-black/50"
		onclick={closePopup}
		aria-label="Close sidebar"
	></button>
	<div
		class="fixed top-0 right-0 z-40 h-full w-72 overflow-hidden bg-black/95 backdrop-blur-sm"
		role="dialog"
		onclick={(e) => e.stopPropagation()}
	>
		<div class="flex h-full flex-col p-4">
			<div class="mb-4 flex items-center justify-between">
				<span class="text-sm font-medium text-white">播放设置</span>
				<button class="text-white/60 hover:text-white" onclick={closePopup}>✕</button>
			</div>

			<div class="mb-4 flex-1 overflow-y-auto">
				<div class="mb-4">
					<span class="mb-2 block text-xs text-white/60">倍速</span>
					<div class="flex flex-wrap gap-2">
						{#each SPEED_OPTIONS as speed}
							<button
								class="rounded-md px-3 py-1.5 text-sm transition-colors
									{localPlaybackRate === speed
									? 'bg-primary text-primary-foreground'
									: 'bg-white/10 text-white hover:bg-white/20'}"
								onclick={() => setPlaybackRate(speed)}
							>
								{speed}x
							</button>
						{/each}
					</div>
				</div>

				{#if availableSources && availableSources.length > 1}
					<div class="mb-4">
						<span class="mb-2 block text-xs text-white/60">源选择</span>
						<div class="flex flex-wrap gap-2">
							{#each availableSources as source (source.source_code)}
								<button
									class="rounded-md bg-white/10 px-3 py-1.5 text-sm text-white transition-colors hover:bg-white/20"
									onclick={() => onSourceChange?.(source)}
								>
									{source.source_name}
								</button>
							{/each}
						</div>
					</div>
				{/if}

				{#if episodes.length > 0}
					<div class="overflow-y-auto">
						<span class="mb-2 block text-xs text-white/60">选集 ({episodes.length})</span>
						<div class="grid grid-cols-5 gap-2 sm:grid-cols-6">
							{#each episodes as episode, i (episode.url)}
								<button
									class="rounded-md px-2 py-1.5 text-center text-xs transition-colors
										{i === currentEpisodeIndex
										? 'bg-primary text-primary-foreground'
										: 'bg-white/10 text-white hover:bg-white/20'}"
									onclick={() => handleEpisodeSelect(episode, i)}
								>
									{episode.episode}
								</button>
							{/each}
						</div>
					</div>
				{/if}
			</div>
		</div>
	</div>
{/if}

<div
	class="relative h-full w-full bg-black"
	bind:this={containerEl}
	onmousemove={showControlsTemporarily}
	ondblclick={handleVideoDoubleClick}
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
		onclick={handleVideoClick}
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
				<span class="text-sm">{formatTime(currentTime)} / {formatTime(duration)}</span>
				<button
					class="flex h-8 w-8 cursor-pointer items-center justify-center rounded-full bg-black/40 transition-colors hover:bg-black/60"
					onclick={togglePlay}
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
				value={localCurrentTime}
				oninput={handleSeek}
				class="mb-3 h-1 w-full cursor-pointer appearance-none rounded-lg bg-white/30 [&::-webkit-slider-thumb]:h-3 [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-white"
			/>

			<div class="flex items-center justify-between text-white">
				<div class="flex items-center gap-3">
					<button
						class="flex h-8 w-8 cursor-pointer items-center justify-center rounded-full bg-black/40 transition-colors hover:bg-black/60"
						onclick={toggleMute}
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
						oninput={handleVolumeChange}
						class="h-1 w-20 cursor-pointer appearance-none rounded-lg bg-white/30 [&::-webkit-slider-thumb]:h-3 [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-white"
					/>
				</div>

				<div class="flex items-center gap-2">
					<button
						class="flex h-8 w-8 cursor-pointer items-center justify-center rounded-full bg-black/40 transition-colors hover:bg-black/60"
						onclick={() => {
							showPopup = !showPopup;
						}}
					>
						<List class="h-4 w-4" />
					</button>
					{#if showFullscreenButton}
						<button
							class="flex h-8 w-8 cursor-pointer items-center justify-center rounded-full bg-black/40 transition-colors hover:bg-black/60"
							onclick={toggleFullscreen}
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
</div>
