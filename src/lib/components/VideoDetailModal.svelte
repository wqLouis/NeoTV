<script lang="ts">
	import { goto } from '$app/navigation';
	import type { SearchResult } from '$lib/api/search';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import EpisodeList from '$lib/components/EpisodeList.svelte';
	import { parsePlayUrl } from '$lib/api/search';
	import type { VideoDetail } from '$lib/api/search';
	import { X } from 'lucide-svelte';

	interface Props {
		detail: VideoDetail | null;
		loading?: boolean;
		onClose: () => void;
		onPlay: (
			episode: { episode: string; url: string },
			videoInfo: { id: string; source: string; title: string; cover?: string }
		) => void;
	}

	let { detail, loading = false, onClose, onPlay }: Props = $props();

	let currentEpisodeIndex = $state(0);
	let episodesReversed = $state(false);

	let video = $derived(detail?.list?.[0]);
	let episodes = $derived(video?.vod_play_url ? parsePlayUrl(video.vod_play_url) : []);

	function handleEpisodeSelect(episode: { episode: string; url: string }, index: number) {
		currentEpisodeIndex = index;
		if (video) {
			onPlay(episode, {
				id: video.vod_id,
				source: '',
				title: video.vod_name,
				cover: video.vod_pic
			});
		}
	}

	function toggleReverse() {
		episodesReversed = !episodesReversed;
	}
</script>

<div
	class="fixed inset-0 z-50 bg-background/80 backdrop-blur-sm"
	onclick={onClose}
	role="button"
	tabindex="-1"
	onkeydown={(e) => e.key === 'Escape' && onClose()}
>
	<div
		class="fixed inset-x-4 top-1/2 mx-auto flex max-h-[90vh] max-w-2xl -translate-y-1/2 flex-col overflow-hidden rounded-lg bg-card shadow-lg"
		onclick={(e) => e.stopPropagation()}
		role="dialog"
	>
		<div class="flex items-center justify-between border-b p-4">
			<h2 class="text-lg font-semibold">视频详情</h2>
			<Button variant="ghost" size="icon" onclick={onClose}>
				<X class="h-5 w-5" />
			</Button>
		</div>

		<div class="flex-grow overflow-y-auto p-4">
			{#if loading}
				<div class="flex items-center justify-center py-12">
					<div class="h-8 w-8 animate-spin rounded-full border-b-2 border-primary"></div>
				</div>
			{:else if video}
				<div class="space-y-4">
					<div class="flex gap-4">
						{#if video.vod_pic}
							<img
								src={video.vod_pic}
								alt={video.vod_name}
								class="h-44 w-32 flex-shrink-0 rounded-md object-cover"
							/>
						{/if}
						<div class="min-w-0 flex-grow">
							<h3 class="mb-2 text-xl font-bold">{video.vod_name}</h3>
							<div class="mb-2 flex flex-wrap gap-2">
								{#if video.type_name}
									<Badge variant="secondary">{video.type_name}</Badge>
								{/if}
								{#if video.vod_year}
									<Badge variant="outline">{video.vod_year}</Badge>
								{/if}
								{#if video.vod_remarks}
									<Badge variant="outline">{video.vod_remarks}</Badge>
								{/if}
							</div>
							{#if video.vod_content}
								<p class="line-clamp-3 text-sm text-muted-foreground">{video.vod_content}</p>
							{/if}
						</div>
					</div>

					{#if episodes.length > 0}
						<div class="space-y-3">
							<div class="flex items-center justify-between">
								<span class="text-sm font-medium">选集 (共{episodes.length}集)</span>
								<Button variant="outline" size="sm" onclick={toggleReverse}>
									{episodesReversed ? '正序' : '倒序'}
								</Button>
							</div>
							<EpisodeList
								{episodes}
								currentIndex={currentEpisodeIndex}
								reversed={episodesReversed}
								onSelect={handleEpisodeSelect}
							/>
						</div>
					{/if}
				</div>
			{:else}
				<div class="py-12 text-center text-muted-foreground">
					<p>暂无详情信息</p>
				</div>
			{/if}
		</div>
	</div>
</div>
