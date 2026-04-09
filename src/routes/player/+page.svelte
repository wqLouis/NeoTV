<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { settingsStore } from '$lib/stores/settings.svelte';
	import { historyStore } from '$lib/stores/history.svelte';
	import { parsePlayUrl, getVideoDetail, type VideoDetail } from '$lib/api/search';
	import EpisodeList from '$lib/components/EpisodeList.svelte';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import {
		ArrowLeft,
		Play,
		Pause,
		Volume2,
		VolumeX,
		Maximize,
		Copy,
		Lock,
		Unlock
	} from 'lucide-svelte';

	let videoUrl = $state('');
	let title = $state('');
	let cover = $state('');
	let episodes = $state<{ episode: string; url: string }[]>([]);
	let currentEpisodeIndex = $state(0);
	let loading = $state(true);
	let controlsLocked = $state(false);
	let adFilteringEnabled = $state(true);
	let autoplayEnabled = $state(true);

	let player: HTMLVideoElement;
	let containerEl: HTMLDivElement;

	onMount(() => {
		const params = $page.url.searchParams;
		const url = params.get('url');
		const epIdx = params.get('episodeIndex');
		const pos = params.get('position');

		title = params.get('title') || '视频播放';
		cover = params.get('cover') || '';

		if (epIdx) currentEpisodeIndex = parseInt(epIdx, 10);

		if (url) {
			videoUrl = decodeURIComponent(url);
		}

		adFilteringEnabled = settingsStore.adFilteringEnabled;
		autoplayEnabled = settingsStore.autoplayEnabled;

		if (pos && player) {
			player.currentTime = parseFloat(pos);
		}

		loading = false;
	});

	function handleEpisodeSelect(episode: { episode: string; url: string }, index: number) {
		currentEpisodeIndex = index;
		videoUrl = episode.url;
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
	}

	function toggleControlsLock() {
		controlsLocked = !controlsLocked;
	}

	function goBack() {
		if (window.history.length > 1) {
			window.history.back();
		} else {
			goto('/');
		}
	}

	function copyUrl() {
		navigator.clipboard.writeText(videoUrl);
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

	function handleKeydown(e: KeyboardEvent) {
		if (controlsLocked) return;
		switch (e.key) {
			case ' ':
			case 'k':
				e.preventDefault();
				if (player.paused) player.play();
				else player.pause();
				break;
			case 'ArrowLeft':
				player.currentTime -= 10;
				break;
			case 'ArrowRight':
				player.currentTime += 10;
				break;
			case 'm':
				player.muted = !player.muted;
				break;
			case 'f':
				toggleFullscreen();
				break;
		}
	}

	function handleTimeUpdate() {
		if (player && player.duration) {
			historyStore.updatePosition(
				$page.url.searchParams.get('id') || '',
				$page.url.searchParams.get('source') || '',
				episodes[currentEpisodeIndex]?.episode,
				player.currentTime,
				player.duration
			);
		}
	}

	function handleEnded() {
		if (autoplayEnabled && currentEpisodeIndex < episodes.length - 1) {
			const nextIndex = currentEpisodeIndex + 1;
			handleEpisodeSelect(episodes[nextIndex], nextIndex);
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="min-h-screen bg-black" bind:this={containerEl}>
	<div class="container mx-auto px-4 py-4">
		<div class="mb-4 flex items-center gap-4">
			<Button variant="ghost" size="icon" onclick={goBack}>
				<ArrowLeft class="h-5 w-5" />
			</Button>
			<h1 class="flex-grow truncate text-lg font-semibold text-white">{title}</h1>
			<Button variant="ghost" size="icon" onclick={toggleControlsLock}>
				{#if controlsLocked}
					<Lock class="h-5 w-5" />
				{:else}
					<Unlock class="h-5 w-5" />
				{/if}
			</Button>
		</div>

		<div
			class="relative aspect-video overflow-hidden rounded-lg bg-black {controlsLocked
				? 'cursor-none'
				: ''}"
		>
			{#if loading}
				<div class="absolute inset-0 flex items-center justify-center">
					<div class="h-12 w-12 animate-spin rounded-full border-b-2 border-white"></div>
				</div>
			{:else if videoUrl}
				<video
					bind:this={player}
					src={videoUrl}
					class="h-full w-full"
					controls={!controlsLocked}
					autoplay={autoplayEnabled}
					onclick={() => (player.paused ? player.play() : player.pause())}
					ontimeupdate={handleTimeUpdate}
					onended={handleEnded}
				>
					<track kind="captions" />
				</video>
			{:else}
				<div class="absolute inset-0 flex flex-col items-center justify-center text-white">
					<Play class="mb-4 h-16 w-16 opacity-50" />
					<p>暂无视频源</p>
				</div>
			{/if}
		</div>

		<div class="mt-4 flex gap-2">
			<Button variant="outline" size="sm" onclick={copyUrl}>
				<Copy class="mr-1 h-4 w-4" />
				复制链接
			</Button>
			<Button variant="outline" size="sm" onclick={toggleFullscreen}>
				<Maximize class="mr-1 h-4 w-4" />
				全屏
			</Button>
		</div>

		{#if episodes.length > 0}
			<div class="mt-6">
				<div class="mb-3 flex items-center justify-between">
					<span class="text-sm font-medium text-white">选集 (共{episodes.length}集)</span>
					<Badge variant="secondary">当前: {currentEpisodeIndex + 1}</Badge>
				</div>
				<EpisodeList
					{episodes}
					currentIndex={currentEpisodeIndex}
					reversed={settingsStore.episodesReversed}
					onSelect={handleEpisodeSelect}
				/>
			</div>
		{/if}
	</div>
</div>
