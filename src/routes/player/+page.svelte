<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { onMount, onDestroy } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { settingsStore } from '$lib/stores/settings.svelte';
	import { historyStore } from '$lib/stores/history.svelte';
	import { parsePlayUrl, getVideoDetail, type VideoDetail } from '$lib/api/search';
	import VideoPlayer from '$lib/components/VideoPlayer.svelte';
	import VideoSourceOverlay from '$lib/components/VideoSourceOverlay.svelte';
	import { Button } from '$lib/components/ui/button';
	import { AlertCircle } from 'lucide-svelte';
	import { toast } from 'svelte-sonner';
	import type { DoubanSubject } from '$lib/api/douban';

	let title = $state('');
	let cover = $state('');
	let initialPosition = $state(0);
	let episodes = $state<{ episode: string; url: string }[]>([]);
	let currentEpisodeIndex = $state(0);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let controlsLocked = $state(false);
	let autoplayEnabled = $state(true);
	let videoDetail = $state<VideoDetail | null>(null);
	let containerEl: HTMLDivElement;

	let playerType: 'native' | 'hls' = $state('native');
	let playerSrc: string = $state('');
	let isDesktop = $state(true);
	let showSourceOverlay = $state(false);
	let overlaySubject = $state<DoubanSubject | null>(null);

	function isMobileDevice(): boolean {
		if (typeof navigator === 'undefined') return true;
		const ua = navigator.userAgent.toLowerCase();
		return /android|webos|iphone|ipad|ipod|blackberry|iemobile|opera mini/i.test(ua);
	}

	interface AvailableSource {
		source_code: string;
		vod_id: string;
		vod_name: string;
		source_name: string;
		vod_pic?: string;
	}
	let availableSources = $state<AvailableSource[]>([]);
	let currentSourceIndex = $state(0);
	let failedSources = $state<Set<string>>(new Set());

	async function processVideoUrl(url: string): Promise<{ type: 'native' | 'hls'; url: string }> {
		const urlClean = url.split('$$$')[0].split('#')[0].trim();
		const isM3U8 = urlClean.toLowerCase().includes('.m3u8') || urlClean.includes('m3u8');
		console.log('[Player] processVideoUrl called:', { originalUrl: url, url: urlClean, isM3U8 });

		if (!isM3U8) {
			console.log('[Player] Not M3U8, using native player');
			return { type: 'native', url: urlClean };
		}

		try {
			console.log('[Player] Calling Rust fetch_media_url...');
			const result = await invoke<{
				url: string;
				content_type: string;
				is_m3u8: boolean;
				processed_content: string | null;
			}>('fetch_media_url', { url: urlClean, adFiltering: settingsStore.adFilteringEnabled });

			console.log('[Player] Rust response:', {
				url: result.url,
				content_type: result.content_type,
				is_m3u8: result.is_m3u8,
				hasProcessedContent: !!result.processed_content
			});

			if (result.processed_content) {
				return { type: 'hls', url: result.url };
			}

			return { type: 'hls', url: result.url };
		} catch (e) {
			console.warn('[Player] Rust failed to process M3U8, using hls.js fallback:', e);
			return { type: 'hls', url: urlClean };
		}
	}

	async function loadVideoDetail() {
		const params = $page.url.searchParams;
		const id = params.get('id');
		const source = params.get('source');
		const directUrl = params.get('url');
		const epIdx = params.get('episodeIndex');
		const searchQuery = params.get('search');
		const positionParam = params.get('position');

		title = params.get('title') || '视频播放';
		cover = params.get('cover') || '';

		if (epIdx) currentEpisodeIndex = parseInt(epIdx, 10);
		if (positionParam) initialPosition = parseFloat(positionParam);

		if (searchQuery) {
			overlaySubject = {
				id: id || searchQuery,
				title: decodeURIComponent(searchQuery),
				cover: cover || '',
				cover_url: cover || '',
				rate: '',
				score: '',
				types: [],
				regions: []
			};
			showSourceOverlay = true;
			loading = false;
			return;
		}

		if (directUrl) {
			const processed = await processVideoUrl(decodeURIComponent(directUrl));
			playerSrc = processed.url;
			playerType = processed.type;
			loading = false;
			return;
		}

		if (id && source) {
			try {
				const detail = await getVideoDetail(id, source);
				if (detail && detail.list && detail.list.length > 0) {
					videoDetail = detail;
					const video = detail.list[0];
					title = video.vod_name || title;
					cover = video.vod_pic || cover;

					const parsedEpisodes = parsePlayUrl(video.vod_play_url);
					if (parsedEpisodes.length > 0) {
						episodes = parsedEpisodes;
						if (currentEpisodeIndex >= episodes.length) {
							currentEpisodeIndex = 0;
						}
						const processed = await processVideoUrl(episodes[currentEpisodeIndex].url);
						playerSrc = processed.url;
						playerType = processed.type;
					} else {
						error = '该视频暂无播放地址';
					}
				} else {
					error = '无法获取视频详情';
				}
			} catch (e) {
				console.error('Failed to load video detail:', e);
				error = '加载视频详情失败';
			}
		} else {
			error = '缺少视频ID或来源信息';
		}

		loading = false;
	}

	onMount(async () => {
		autoplayEnabled = settingsStore.autoplayEnabled;
		isDesktop = !isMobileDevice();
		const storedSources = sessionStorage.getItem('availableSources');
		if (storedSources) {
			try {
				availableSources = JSON.parse(storedSources);
			} catch {
				availableSources = [];
			}
		}
		await loadVideoDetail();
	});

	async function handleEpisodeSelect(episode: { episode: string; url: string }, index: number) {
		currentEpisodeIndex = index;
		loading = true;

		const processed = await processVideoUrl(episode.url);
		playerSrc = processed.url;
		playerType = processed.type;

		await historyStore.add({
			id: $page.url.searchParams.get('id') || '',
			title,
			source: $page.url.searchParams.get('source') || '',
			cover,
			episode: episode.episode,
			episodeIndex: index,
			position: 0,
			duration: 0
		});

		loading = false;
	}

	async function handleSourceChange(source: {
		source_code: string;
		vod_id?: string;
		vod_name?: string;
	}) {
		const vodId = source.vod_id;
		const sourceCode = source.source_code;
		if (!vodId) return;

		const idx = availableSources.findIndex((s) => s.source_code === sourceCode);
		if (idx === -1) return;
		currentSourceIndex = idx;
		loading = true;

		try {
			const detail = await getVideoDetail(vodId, sourceCode);
			if (detail?.list?.[0]) {
				videoDetail = detail;
				const video = detail.list[0];
				const parsedEpisodes = parsePlayUrl(video.vod_play_url);
				episodes = parsedEpisodes;
				currentEpisodeIndex = 0;
				if (episodes.length > 0) {
					const processed = await processVideoUrl(episodes[0].url);
					playerSrc = processed.url;
					playerType = processed.type;
				} else {
					error = '该视频暂无播放地址';
				}
				await historyStore.add({
					id: vodId,
					title: video.vod_name,
					source: sourceCode,
					cover: video.vod_pic,
					episode: episodes[0]?.episode || '第1集',
					episodeIndex: 0,
					position: 0,
					duration: 0
				});
			} else {
				error = '无法获取视频详情';
			}
		} catch (e) {
			console.error('Failed to load source:', e);
			error = '切换源失败';
		}

		loading = false;
	}

	function toggleControlsLock() {
		controlsLocked = !controlsLocked;
	}

	function handleTimeUpdate(currentTime: number, dur: number) {
		historyStore.updatePosition(
			$page.url.searchParams.get('id') || '',
			$page.url.searchParams.get('source') || '',
			episodes[currentEpisodeIndex]?.episode,
			currentTime,
			dur
		);
	}

	function handleEnded() {
		if (autoplayEnabled && currentEpisodeIndex < episodes.length - 1) {
			const nextIndex = currentEpisodeIndex + 1;
			handleEpisodeSelect(episodes[nextIndex], nextIndex);
		}
	}

	function handlePlayerError(errorMsg: string) {
		console.error('[Player] Playback error:', errorMsg);

		const currentSource = availableSources[currentSourceIndex];
		if (currentSource) {
			failedSources = new Set(failedSources);
			failedSources.add(currentSource.source_code);
		}

		const remainingSources = availableSources.filter((s) => !failedSources.has(s.source_code));

		if (remainingSources.length > 0) {
			const nextSource = remainingSources[0];
			const nextIndex = availableSources.findIndex((s) => s.source_code === nextSource.source_code);
			toast.error('播放失败，尝试切换源...');
			handleSourceChange({ source_code: nextSource.source_code, vod_id: nextSource.vod_id });
		} else {
			toast.error('所有源均无法播放');
			error = '所有源均无法播放';
		}
	}

	async function goBack() {
		if (error && !loading) {
			toast.error('无法播放，请尝试其他源');
		}
		if (window.history.length > 1) {
			window.history.back();
		} else {
			goto('/');
		}
	}
</script>

<div class="fixed inset-0 z-[100] bg-black" bind:this={containerEl}>
	{#if loading}
		<div class="absolute inset-0 flex items-center justify-center">
			<div class="h-12 w-12 animate-spin rounded-full border-b-2 border-white"></div>
		</div>
	{:else if error}
		<div class="absolute inset-0 flex flex-col items-center justify-center text-white">
			<AlertCircle class="mb-4 h-16 w-16 opacity-50" />
			<p class="text-lg">{error}</p>
			<Button variant="outline" class="mt-4" onclick={goBack}>返回首页</Button>
		</div>
	{:else if playerSrc}
		<VideoPlayer
			src={playerSrc}
			type={playerType}
			autoplay={autoplayEnabled}
			poster={cover}
			{initialPosition}
			{episodes}
			{currentEpisodeIndex}
			showFullscreenButton={isDesktop}
			onEpisodeChange={handleEpisodeSelect}
			onTimeUpdate={handleTimeUpdate}
			onEnded={handleEnded}
			onError={handlePlayerError}
			onReturn={goBack}
			{availableSources}
			onSourceChange={handleSourceChange}
		/>
	{:else if showSourceOverlay && overlaySubject}
		<div class="h-full w-full"></div>
	{:else}
		<div class="absolute inset-0 flex flex-col items-center justify-center text-white">
			<p>暂无视频源</p>
		</div>
	{/if}
</div>

<VideoSourceOverlay
	item={overlaySubject}
	open={showSourceOverlay}
	onOpenChange={(open) => {
		showSourceOverlay = open;
		if (!open) {
			goBack();
		}
	}}
/>
