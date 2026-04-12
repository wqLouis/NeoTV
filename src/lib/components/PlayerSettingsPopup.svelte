<script lang="ts">
	import { fly } from 'svelte/transition';

	interface Episode {
		episode: string;
		url: string;
	}

	interface Source {
		source_code: string;
		source_name: string;
		[key: string]: unknown;
	}

	interface Props {
		show: boolean;
		playbackRate: number;
		availableSources: Source[];
		currentSourceIndex?: number;
		episodes: Episode[];
		currentEpisodeIndex: number;
		speedOptions: readonly number[];
		onClose: () => void;
		onPlaybackRateChange: (rate: number) => void;
		onSourceChange?: (source: Source) => void;
		onEpisodeSelect: (episode: Episode, index: number) => void;
	}

	let {
		show,
		playbackRate,
		availableSources,
		currentSourceIndex = 0,
		episodes,
		currentEpisodeIndex,
		speedOptions,
		onClose,
		onPlaybackRateChange,
		onSourceChange,
		onEpisodeSelect
	}: Props = $props();
</script>

{#if show}
	<button
		class="fixed inset-0 z-[60] cursor-default bg-black/50"
		onclick={onClose}
		aria-label="Close settings"
	></button>
	<div
		class="fixed top-0 right-0 z-[60] h-full w-72 overflow-hidden bg-black/95 backdrop-blur-sm"
		role="dialog"
		onclick={(e) => {
			if (e.target === e.currentTarget) onClose();
		}}
		transition:fly={{ x: 288, duration: 300 }}
	>
		<div class="flex h-full flex-col p-4">
			<div class="mb-4 flex items-center justify-between">
				<span class="text-sm font-medium text-white">播放设置</span>
				<button class="text-white/60 hover:text-white" onclick={onClose}>✕</button>
			</div>

			<div class="mb-4 flex-1 overflow-y-auto">
				<div class="mb-4">
					<span class="mb-2 block text-xs text-white/60">倍速</span>
					<div class="flex flex-wrap gap-2">
						{#each speedOptions as speed}
							<button
								class="rounded-md px-3 py-1.5 text-sm transition-colors
									{playbackRate === speed
									? 'bg-primary text-primary-foreground'
									: 'bg-white/10 text-white hover:bg-white/20'}"
								onclick={() => onPlaybackRateChange(speed)}
							>
								{speed}x
							</button>
						{/each}
					</div>
				</div>

				{#if availableSources && availableSources.length > 0}
					<div class="mb-4">
						<span class="mb-2 block text-xs text-white/60">源选择</span>
						<div class="flex flex-wrap gap-2">
							{#each availableSources as source, i (source.source_code)}
								<button
									class="rounded-md px-3 py-1.5 text-sm transition-colors
										{i === currentSourceIndex
										? 'bg-primary text-primary-foreground'
										: 'bg-white/10 text-white hover:bg-white/20'}"
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
						<div class="grid grid-cols-4 gap-2">
							{#each episodes as episode, i (episode.url)}
								<button
									class="min-w-0 rounded-md px-2 py-2 text-center text-xs transition-colors
										{i === currentEpisodeIndex
										? 'bg-primary text-primary-foreground'
										: 'bg-white/10 text-white hover:bg-white/20'}"
									onclick={() => onEpisodeSelect(episode, i)}
								>
									<span class="block truncate">{episode.episode}</span>
								</button>
							{/each}
						</div>
					</div>
				{/if}
			</div>
		</div>
	</div>
{/if}
