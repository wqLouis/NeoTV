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
	import { fade } from 'svelte/transition';
	import PlayerControls from './PlayerControls.svelte';
	import PlayerSettingsPopup from './PlayerSettingsPopup.svelte';

	interface Episode {
		episode: string;
		url: string;
	}

	interface Props {
		src: string;
		type?: 'native' | 'hls';
		autoplay?: boolean;
		poster?: string;
		initialPosition?: number;
		episodes?: Episode[];
		currentEpisodeIndex?: number;
		currentSourceIndex?: number;
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
		initialPosition = 0,
		episodes = [],
		currentEpisodeIndex = 0,
		currentSourceIndex = 0,
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

	let playing = $state(false);
	let currentTime = $state(0);
	let duration = $state(0);
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
	const LONG_PRESS_DURATION = 300;

	let longPressTimer: ReturnType<typeof setTimeout> | null = null;
	let longPressTriggered = false;
	let seekingTime = $state<number | undefined>(undefined);

	let isUnmounting = $state(false);

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

	function toggleFullscreen() {
		if (!containerEl) return;
		if (document.fullscreenElement) {
			document.exitFullscreen();
		} else {
			containerEl.requestFullscreen();
		}
	}

	function startLongPress() {
		if (longPressTimer) clearTimeout(longPressTimer);
		longPressTriggered = false;
		longPressTimer = setTimeout(() => {
			if (videoEl) {
				setPlaybackRate(localPlaybackRate * 2);
				longPressTriggered = true;
			}
		}, LONG_PRESS_DURATION);
	}

	function endLongPress() {
		if (longPressTimer) {
			clearTimeout(longPressTimer);
			longPressTimer = null;
		}
		if (videoEl && localPlaybackRate > 1) {
			setPlaybackRate(localPlaybackRate / 2);
		}
		longPressTriggered = false;
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

	let mediaErrorRecoveryCount = 0;
	const MAX_MEDIA_ERROR_RECOVERIES = 3;
	let networkErrorRetryCount = 0;
	const MAX_NETWORK_ERROR_RETRIES = 2;

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
		mediaErrorRecoveryCount = 0;
		networkErrorRetryCount = 0;

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
				debug: false,
				preferManagedMediaSource: false,
				enableSoftwareAES: true,
				stretchShortVideoTrack: true,
				defaultAudioCodec: 'mp4a.40.2',
				loader: RustLoader,
				autoStartLoad: true,
				startLevel: -1,
				maxBufferLength: 300,
				maxMaxBufferLength: 600,
				maxBufferSize: 2 * 1000 * 1000 * 1000,
				maxBufferHole: 2.0,
				backBufferLength: 120
			});

			hls.on(Hls.Events.ERROR, (_, data) => {
				console.error('[HLS] Error:', data);
				if (data.fatal && hls) {
					switch (data.type) {
						case Hls.ErrorTypes.NETWORK_ERROR:
							console.error('[HLS] Network error');
							networkErrorRetryCount++;
							if (networkErrorRetryCount <= MAX_NETWORK_ERROR_RETRIES) {
								console.warn(
									`[HLS] Network error, retry attempt ${networkErrorRetryCount}/${MAX_NETWORK_ERROR_RETRIES}`
								);
								hls.startLoad();
								return;
							}
							hls.destroy();
							hls = null;
							error = '网络连接失败，请检查网络';
							return;
						case Hls.ErrorTypes.MEDIA_ERROR:
							if (data.details === Hls.ErrorDetails.BUFFER_ADD_CODEC_ERROR) {
								mediaErrorRecoveryCount++;
								if (mediaErrorRecoveryCount <= MAX_MEDIA_ERROR_RECOVERIES) {
									console.warn(
										`[HLS] Codec error, recovery attempt ${mediaErrorRecoveryCount}/${MAX_MEDIA_ERROR_RECOVERIES}`
									);
									hls.recoverMediaError();
								} else {
									console.error('[HLS] Codec error recovery exhausted');
									hls.destroy();
									hls = null;
									error = '视频解码失败，暂不支持该格式';
								}
								return;
							}
							if (data.details === Hls.ErrorDetails.MANIFEST_INCOMPATIBLE_CODECS_ERROR) {
								mediaErrorRecoveryCount++;
								if (mediaErrorRecoveryCount <= MAX_MEDIA_ERROR_RECOVERIES) {
									console.warn(
										`[HLS] Manifest incompatibility, recovery attempt ${mediaErrorRecoveryCount}/${MAX_MEDIA_ERROR_RECOVERIES}`
									);
									hls.recoverMediaError();
								} else {
									console.error('[HLS] Manifest incompatibility recovery exhausted');
									hls.destroy();
									hls = null;
									error = '视频编码不兼容，请尝试其他片源';
								}
								return;
							}
							if (data.details === Hls.ErrorDetails.BUFFER_APPEND_ERROR) {
								mediaErrorRecoveryCount++;
								console.warn(
									`[HLS] Buffer append error, count ${mediaErrorRecoveryCount}/${MAX_MEDIA_ERROR_RECOVERIES}`
								);
								if (mediaErrorRecoveryCount > MAX_MEDIA_ERROR_RECOVERIES) {
									console.error('[HLS] Buffer append error recovery exhausted');
									hls.destroy();
									hls = null;
									error = '视频缓冲失败，请尝试其他片源';
								} else {
									hls.recoverMediaError();
								}
								return;
							}
							hls.recoverMediaError();
							mediaErrorRecoveryCount = 0;
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
			console.error('[HLS] Hls.isSupported() = false, cannot play HLS');
			error = '当前浏览器不支持 HLS 播放';
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
		isUnmounting = true;
		clearTimeout(controlsTimeout);
		if (longPressTimer) clearTimeout(longPressTimer);
		if (hls) {
			hls.destroy();
		}
	});

	function handleTimeUpdate() {
		if (videoEl) {
			currentTime = videoEl.currentTime;
			localCurrentTime = videoEl.currentTime;
			duration = videoEl.duration || 0;
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
		if (isUnmounting) return;
		console.error('[HLS] handleError called, video error:', videoEl?.error);
		error = '视频播放失败';
		onError?.('Video playback error');
	}

	function handleCanPlay() {
		loading = false;
		if (initialPosition > 0 && videoEl && videoEl.duration > initialPosition) {
			videoEl.currentTime = initialPosition;
		}
	}

	function closePopup() {
		showPopup = false;
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<PlayerSettingsPopup
	show={showPopup}
	playbackRate={localPlaybackRate}
	{availableSources}
	{currentSourceIndex}
	{episodes}
	{currentEpisodeIndex}
	speedOptions={SPEED_OPTIONS}
	onClose={closePopup}
	onPlaybackRateChange={setPlaybackRate}
	{onSourceChange}
	onEpisodeSelect={handleEpisodeSelect}
/>

<div
	class="relative h-full w-full bg-black select-none"
	bind:this={containerEl}
	onmousemove={showControlsTemporarily}
	ondblclick={handleVideoDoubleClick}
	onclick={(e) => {
		if ((e.target as HTMLElement).closest('.player-hud')) return;
		if (showControls) {
			showControls = false;
		} else {
			showControlsTemporarily();
		}
	}}
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

	{#if localPlaybackRate > 1}
		<div
			class="pointer-events-none absolute inset-0 z-40 flex items-start justify-center pt-16"
			transition:fade={{ duration: 200 }}
		>
			<div class="flex items-center gap-2 rounded-full bg-black/60 px-4 py-2 backdrop-blur-sm">
				<svg
					class="h-5 w-5 text-yellow-400"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
				>
					<polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2" />
				</svg>
				<span class="text-sm font-medium text-white">{localPlaybackRate}x</span>
			</div>
		</div>
	{/if}

	<video
		bind:this={videoEl}
		src={type === 'native' ? src : undefined}
		{poster}
		class="h-full w-full select-none"
		playsinline
		onplay={handlePlay}
		onpause={handlePause}
		ontimeupdate={handleTimeUpdate}
		onended={handleEnded}
		onerror={handleError}
		oncanplay={handleCanPlay}
		onmousedown={startLongPress}
		onmouseup={endLongPress}
		onmouseleave={endLongPress}
		ontouchstart={(e) => {
			e.preventDefault();
			startLongPress();
		}}
		ontouchend={endLongPress}
		ontouchcancel={endLongPress}
	>
		<track kind="captions" />
	</video>

	<PlayerControls
		{currentTime}
		{duration}
		{playing}
		{muted}
		{volume}
		{fullscreen}
		{showControls}
		{showFullscreenButton}
		showSettings={showPopup}
		seekingTime={seekingTime ?? undefined}
		{onReturn}
		onTogglePlay={togglePlay}
		onSeek={(v) => {
			localCurrentTime = v;
			if (videoEl) videoEl.currentTime = v;
		}}
		onToggleMute={toggleMute}
		onVolumeChange={(v) => {
			volume = v;
			if (videoEl) videoEl.volume = v;
			muted = v === 0;
		}}
		onToggleFullscreen={toggleFullscreen}
		onTogglePopup={() => (showPopup = !showPopup)}
	/>
</div>
