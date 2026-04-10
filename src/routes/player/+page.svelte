<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { onMount, onDestroy } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { settingsStore } from '$lib/stores/settings.svelte';
	import { historyStore } from '$lib/stores/history.svelte';
	import { parsePlayUrl, getVideoDetail, type VideoDetail } from '$lib/api/search';
	import VideoPlayer from '$lib/components/VideoPlayer.svelte';
	import { Button } from '$lib/components/ui/button';
	import { ArrowLeft, Lock, Unlock, AlertCircle } from 'lucide-svelte';

	let title = $state('');
	let cover = $state('');
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

	interface AvailableSource {
		source_code: string;
		vod_id: string;
		vod_name: string;
		source_name: string;
		vod_pic?: string;
	}
	let availableSources = $state<AvailableSource[]>([]);
	let currentSourceIndex = $state(0);

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

		title = params.get('title') || '视频播放';
		cover = params.get('cover') || '';

		if (epIdx) currentEpisodeIndex = parseInt(epIdx, 10);

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
		const storedSources = sessionStorage.getItem('availableSources');
		if (storedSources) {
			try {
				availableSources = JSON.parse(storedSources);
			} catch {
				availableSources = [];
			}
		}
		await loadVideoDetail();
		try {
			await invoke('set_immersive_android', { enabled: true });
		} catch (e) {
			console.log('[Player] Immersive mode not available:', e);
		}
	});

	onDestroy(async () => {
		try {
			await invoke('set_immersive_android', { enabled: false });
		} catch (e) {
			console.log('[Player] Failed to disable immersive mode:', e);
		}
	});

	async function handleEpisodeSelect(episode: { episode: string; url: string }, index: number) {
		currentEpisodeIndex = index;
		loading = true;

		const processed = await processVideoUrl(episode.url);
		playerSrc = processed.url;
		playerType = processed.type;

		historyStore.add({
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
				historyStore.add({
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

	async function goBack() {
		if (window.history.length > 1) {
			window.history.back();
		} else {
			goto('/');
		}
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
		console.error('[Player] handlePlayerError called:', errorMsg);
		error = '视频播放失败';
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
			{episodes}
			{currentEpisodeIndex}
			showFullscreenButton={false}
			onEpisodeChange={handleEpisodeSelect}
			onTimeUpdate={handleTimeUpdate}
			onEnded={handleEnded}
			onError={handlePlayerError}
			onReturn={goBack}
			{availableSources}
			onSourceChange={handleSourceChange}
		/>
	{:else}
		<div class="absolute inset-0 flex flex-col items-center justify-center text-white">
			<p>暂无视频源</p>
		</div>
	{/if}
</div>
