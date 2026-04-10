<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { onMount, onDestroy } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { settingsStore } from '$lib/stores/settings.svelte';
	import { historyStore } from '$lib/stores/history.svelte';
	import { parsePlayUrl, getVideoDetail, type VideoDetail } from '$lib/api/search';
	import VideoPlayer from '$lib/components/VideoPlayer.svelte';
	import EpisodeList from '$lib/components/EpisodeList.svelte';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { ArrowLeft, Maximize, Copy, Lock, Unlock, AlertCircle, ChevronDown } from 'lucide-svelte';

	let videoUrl = $state('');
	let title = $state('');
	let cover = $state('');
	let episodes = $state<{ episode: string; url: string }[]>([]);
	let currentEpisodeIndex = $state(0);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let controlsLocked = $state(false);
	let autoplayEnabled = $state(true);
	let videoDetail = $state<VideoDetail | null>(null);
	let showControls = $state(true);
	let showEpisodes = $state(false);
	let containerEl: HTMLDivElement;

	let playerType: 'native' | 'hls' = $state('native');
	let playerSrc: string = $state('');

	async function setImmersive(enabled: boolean) {
		try {
			await invoke('plugin:immersiveandroid|set_immersive_android', { enabled });
		} catch (e) {
			console.warn('Failed to set immersive mode:', e);
		}
	}

	async function processVideoUrl(url: string): Promise<{ type: 'native' | 'hls'; url: string }> {
		const isM3U8 = url.toLowerCase().includes('.m3u8') || url.includes('m3u8');

		if (!isM3U8) {
			return { type: 'native', url };
		}

		try {
			console.log('[Player] Processing M3U8 URL:', url);
			const result = await invoke<{
				url: string;
				content_type: string;
				is_m3u8: boolean;
				processed_content: string | null;
			}>('fetch_media_url', { url });

			if (result.processed_content) {
				console.log('[Player] M3U8 content processed, creating Blob URL');
				const blob = new Blob([result.processed_content], {
					type: 'application/vnd.apple.mpegurl'
				});
				const blobUrl = URL.createObjectURL(blob);
				return { type: 'native', url: blobUrl };
			}

			return { type: 'native', url: result.url };
		} catch (e) {
			console.warn('[Player] Rust failed to process M3U8, using hls.js fallback:', e);
			return { type: 'hls', url };
		}
	}

	async function loadVideoDetail() {
		const params = $page.url.searchParams;
		const id = params.get('id');
		const source = params.get('source');
		const directUrl = params.get('url');
		const epIdx = params.get('episodeIndex');
		const pos = params.get('position');

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
		await setImmersive(true);
		autoplayEnabled = settingsStore.autoplayEnabled;
		await loadVideoDetail();
	});

	onDestroy(() => {
		setImmersive(false);
	});

	async function handleEpisodeSelect(episode: { episode: string; url: string }, index: number) {
		currentEpisodeIndex = index;
		showEpisodes = false;
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

	function toggleControlsLock() {
		controlsLocked = !controlsLocked;
	}

	async function goBack() {
		await setImmersive(false);
		if (window.history.length > 1) {
			window.history.back();
		} else {
			goto('/');
		}
	}

	function copyUrl() {
		navigator.clipboard.writeText(playerSrc || videoUrl);
	}

	function toggleFullscreen() {
		if (containerEl) {
			if (document.fullscreenElement) {
				document.exitFullscreen();
			} else {
				containerEl.requestFullscreen();
			}
		}
	}

	function handleTimeUpdate(currentTime: number, duration: number) {
		historyStore.updatePosition(
			$page.url.searchParams.get('id') || '',
			$page.url.searchParams.get('source') || '',
			episodes[currentEpisodeIndex]?.episode,
			currentTime,
			duration
		);
	}

	function handleEnded() {
		if (autoplayEnabled && currentEpisodeIndex < episodes.length - 1) {
			const nextIndex = currentEpisodeIndex + 1;
			handleEpisodeSelect(episodes[nextIndex], nextIndex);
		}
	}

	function handlePlayerError(errorMsg: string) {
		console.error('[Player] Video player error:', errorMsg);
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
			onTimeUpdate={handleTimeUpdate}
			onEnded={handleEnded}
			onError={handlePlayerError}
		>
			{#snippet topControls()}
				<div class="flex items-start justify-between">
					<div class="flex items-center gap-3">
						<Button
							variant="ghost"
							size="icon"
							onclick={goBack}
							class="text-white hover:bg-white/20"
						>
							<ArrowLeft class="h-6 w-6" />
						</Button>
						<div class="flex flex-col">
							<h1 class="text-lg font-semibold text-white">{title}</h1>
							{#if episodes.length > 0}
								<span class="text-sm text-white/70"
									>第 {currentEpisodeIndex + 1} / {episodes.length} 集</span
								>
							{/if}
						</div>
					</div>
					<div class="flex items-center gap-2">
						<Button
							variant="ghost"
							size="icon"
							onclick={toggleControlsLock}
							class="text-white hover:bg-white/20"
						>
							{#if controlsLocked}
								<Lock class="h-5 w-5" />
							{:else}
								<Unlock class="h-5 w-5" />
							{/if}
						</Button>
					</div>
				</div>
			{/snippet}

			{#snippet bottomControls()}
				{#if episodes.length > 0 && !controlsLocked}
					<Button
						variant="outline"
						size="sm"
						onclick={(e) => {
							e.stopPropagation();
							showEpisodes = !showEpisodes;
						}}
						class="border-white/50 text-white hover:bg-white/20"
					>
						选集
						<ChevronDown class="ml-1 h-4 w-4 {showEpisodes ? 'rotate-180' : ''}" />
					</Button>
				{/if}
				<Button
					variant="outline"
					size="sm"
					onclick={(e) => {
						e.stopPropagation();
						copyUrl();
					}}
					class="border-white/50 text-white hover:bg-white/20"
				>
					<Copy class="mr-1 h-4 w-4" />
					复制链接
				</Button>
			{/snippet}
		</VideoPlayer>

		{#if showEpisodes && episodes.length > 0 && !controlsLocked}
			<div
				class="absolute top-1/2 right-4 left-4 max-h-80 -translate-y-1/2 overflow-y-auto rounded-lg bg-black/90 p-4"
				role="dialog"
				onclick={(e) => e.stopPropagation()}
			>
				<div class="mb-3 flex items-center justify-between">
					<span class="text-sm font-medium text-white">选集</span>
					<Badge variant="secondary" class="bg-white/20 text-white"
						>当前: {currentEpisodeIndex + 1}</Badge
					>
				</div>
				<EpisodeList
					{episodes}
					currentIndex={currentEpisodeIndex}
					reversed={settingsStore.episodesReversed}
					onSelect={handleEpisodeSelect}
				/>
			</div>
		{/if}
	{:else}
		<div class="absolute inset-0 flex flex-col items-center justify-center text-white">
			<p>暂无视频源</p>
		</div>
	{/if}
</div>
