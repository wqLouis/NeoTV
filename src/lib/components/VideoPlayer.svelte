<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { settingsStore } from '$lib/stores/settings.svelte';
	import Hls from 'hls.js';
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

	let isUnmounting = $state(false);
	let showDebug = $state(false);
	let cacheStats = $state<{ count: number; bytes: number } | null>(null);
	let workerCount = $state(settingsStore.preloaderWorkerCount);
	let debugLogs = $state<{ timestamp: number; message: string; type: 'log' | 'warn' | 'error' }[]>(
		[]
	);

	let currentInstanceId = $state(0);
	let nextInstanceId = $state(1);

	$effect(() => {
		if (!showDebug) return;

		const origLog = console.log;
		const origWarn = console.warn;
		const origError = console.error;

		const filter =
			(prefix: string) =>
			(message: unknown, ...args: unknown[]) => {
				const msg = typeof message === 'string' ? message : String(message);
				origLog(message, ...args);
				debugLogs = [
					...debugLogs,
					{
						timestamp: Date.now(),
						message: args.length > 0 ? `${msg} ${args.map((a) => String(a)).join(' ')}` : msg,
						type: 'log' as const
					}
				].slice(-100);
			};

		const filterWarn =
			(prefix: string) =>
			(message: unknown, ...args: unknown[]) => {
				const msg = typeof message === 'string' ? message : String(message);
				origWarn(message, ...args);
				debugLogs = [
					...debugLogs,
					{
						timestamp: Date.now(),
						message: args.length > 0 ? `${msg} ${args.map((a) => String(a)).join(' ')}` : msg,
						type: 'warn' as const
					}
				].slice(-100);
			};

		const filterError =
			(prefix: string) =>
			(message: unknown, ...args: unknown[]) => {
				const msg = typeof message === 'string' ? message : String(message);
				origError(message, ...args);
				debugLogs = [
					...debugLogs,
					{
						timestamp: Date.now(),
						message: args.length > 0 ? `${msg} ${args.map((a) => String(a)).join(' ')}` : msg,
						type: 'error' as const
					}
				].slice(-100);
			};

		console.log = filter('[HLS]');
		console.warn = filterWarn('[HLS]');
		console.error = filterError('[HLS]');

		return () => {
			console.log = origLog;
			console.warn = origWarn;
			console.error = origError;
		};
	});

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
		if (videoEl) {
			if (videoEl.paused) {
				videoEl.play();
			} else {
				videoEl.pause();
			}
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
		if (videoEl) {
			videoEl.muted = !videoEl.muted;
			muted = videoEl.muted;
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

	let mediaErrorRecoveryCount = 0;
	const MAX_MEDIA_ERROR_RECOVERIES = 3;
	let networkErrorRetryCount = 0;
	const MAX_NETWORK_ERROR_RETRIES = 2;

	async function updateCacheStats() {
		try {
			const [count, bytes] = await invoke<[number, number]>('preloader_stats');
			cacheStats = { count, bytes };
		} catch (e) {
			console.error('[Debug] Failed to get cache stats:', e);
		}
	}

	async function setWorkerCount(count: number) {
		try {
			await invoke('preloader_set_workers', { count });
			workerCount = count;
			console.log(`[Debug] Worker count set to ${count}`);
		} catch (e) {
			console.error('[Debug] Failed to set worker count:', e);
		}
	}

	async function stopPreloader() {
		try {
			await invoke('preloader_stop');
			cacheStats = null;
			console.log('[Debug] Preloader stopped');
		} catch (e) {
			console.error('[Debug] Failed to stop preloader:', e);
		}
	}

	function createRustLoaderClass(instanceId: number) {
		return class RustLoader {
			context: any = null;
			stats: any = {
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
			private loaderInstanceId: number;

			constructor(config: any) {
				this.config = config;
				this.loaderInstanceId = instanceId;
			}

			destroy(): void {
				this.abortController?.abort();
				this.context = null;
			}

			abort(): void {
				this.abortController?.abort();
				this.stats.aborted = true;
			}

			load(context: any, config: any, callbacks: any): void {
				if (this.loaderInstanceId !== currentInstanceId) {
					console.log(
						'[RustLoader] Discarding stale request, instanceId:',
						this.loaderInstanceId,
						'current:',
						currentInstanceId
					);
					return;
				}
				this.context = context;
				this.stats.loading.start = performance.now();
				this.stats.aborted = false;
				this.abortController = new AbortController();

				const ctx = context as any;
				const type = ctx.type as string;
				const url = ctx.url as string;

				if (
					type === 'manifest' ||
					type === 'level' ||
					type === 'audioTrack' ||
					type === 'subtitle'
				) {
					console.log('[RustLoader] Fetching m3u8:', url);
					invoke<string>('fetch_hls_m3u8', { url })
						.then((content) => {
							if (this.loaderInstanceId !== currentInstanceId) {
								console.log('[RustLoader] Discarding stale m3u8 response');
								return;
							}
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
							if (this.loaderInstanceId !== currentInstanceId) {
								return;
							}
							callbacks.onError({ code: 500, text: String(e) }, context, null, this.stats);
						});
				} else {
					console.log('[RustLoader] Fetching segment:', url);
					invoke<number[]>('fetch_hls_segment', { url })
						.then((data) => {
							if (this.loaderInstanceId !== currentInstanceId) {
								console.log('[RustLoader] Discarding stale segment response');
								return;
							}
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
							if (this.loaderInstanceId !== currentInstanceId) {
								return;
							}
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
		};
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

			currentInstanceId = nextInstanceId++;
			hls = new Hls({
				enableWorker: true,
				lowLatencyMode: false,
				debug: false,
				preferManagedMediaSource: false,
				enableSoftwareAES: true,
				stretchShortVideoTrack: true,
				defaultAudioCodec: 'mp4a.40.2',
				loader: createRustLoaderClass(currentInstanceId),
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

		invoke('preloader_set_max_cache_size', {
			bytes: settingsStore.preloaderCacheSizeMB * 1024 * 1024
		});
		invoke('preloader_set_workers', { count: settingsStore.preloaderWorkerCount });

		if (type === 'native' && src) {
			console.log('[HLS] Using native player');
			loading = false;
			if (autoplay) {
				videoEl.play().catch(() => {});
			}
			onReady?.();
		} else if (type === 'hls') {
			console.log('[HLS] Calling initHls...');
			initHls();
		} else {
			console.log('[HLS] onMount: no conditions matched, type:', type, 'src:', src);
		}
	});

	onDestroy(async () => {
		isUnmounting = true;
		clearTimeout(controlsTimeout);
		if (hls) {
			hls.destroy();
		}
		await invoke('preloader_stop');
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
	onkeydown={(e) => {
		if (e.key === 'Enter' || e.key === ' ') {
			e.preventDefault();
			if (showControls) {
				showControls = false;
			} else {
				showControlsTemporarily();
			}
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
	>
		<track kind="captions" />
	</video>

	{#if showDebug && debugLogs.length > 0}
		<div
			class="pointer-events-none absolute inset-x-0 bottom-20 z-40 max-h-32 overflow-y-auto bg-black/30 p-2 font-mono text-xs"
		>
			{#each debugLogs.slice(-20) as log}
				<div
					class="mb-0.5 {log.type === 'error'
						? 'text-red-400'
						: log.type === 'warn'
							? 'text-yellow-400'
							: 'text-white/80'}"
				>
					<span class="text-white/40">[{new Date(log.timestamp).toLocaleTimeString()}]</span>
					<span class="ml-1">{log.message}</span>
				</div>
			{/each}
		</div>
	{/if}

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
		{showDebug}
		{workerCount}
		{cacheStats}
		{onReturn}
		onTogglePlay={togglePlay}
		onSeek={(v) => {
			localCurrentTime = v;
			if (videoEl) {
				videoEl.currentTime = v;
			}
		}}
		onToggleMute={toggleMute}
		onVolumeChange={(v) => {
			volume = v;
			if (videoEl) {
				videoEl.volume = v;
			}
			muted = v === 0;
		}}
		onToggleFullscreen={toggleFullscreen}
		onTogglePopup={() => (showPopup = !showPopup)}
		onToggleDebug={() => (showDebug = !showDebug)}
		onUpdateCacheStats={updateCacheStats}
		onSetWorkerCount={setWorkerCount}
		onStopPreloader={stopPreloader}
	/>
</div>
