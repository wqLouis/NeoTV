<script lang="ts">
	import HorizontalSection from '$lib/components/HorizontalSection.svelte';
	import PageTabBar from '$lib/components/business/PageTabBar.svelte';
	import { Film, Tv } from '@lucide/svelte';
	import { focusNavigator, Graph, END } from '$lib/utils/focus-navigator';

	let selectedType = $state<'movie' | 'tv'>('movie');

	const typeOptions = [
		{ value: 'movie', label: '电影', icon: Film },
		{ value: 'tv', label: '电视剧', icon: Tv }
	];

	let section0Graph = $state<{ value: Graph | null }>({ value: null });
	let section1Graph = $state<{ value: Graph | null }>({ value: null });

	function buildHomeGraph() {
		if (!section0Graph.value || !section1Graph.value) return;

		const firstNode = section0Graph.value.defaultEntry;
		const homeGraph = new Graph('home', firstNode);

		// Register child graphs
		homeGraph.registerChildGraph('home:section:0', section0Graph.value);
		homeGraph.registerChildGraph('home:section:1', section1Graph.value);

		// Add cross-section connections
		const sec0Count = section0Graph.value.itemCount;
		const sec1Count = section1Graph.value.itemCount;

		for (let i = 0; i < Math.min(sec0Count, sec1Count); i++) {
			const sec0NodeId = `home:section:0:card:${i}`;
			const sec1NodeId = `home:section:1:card:${i}`;

			homeGraph.addConnection(sec0NodeId, 'bottom', sec1NodeId);
			homeGraph.addConnection(sec1NodeId, 'top', sec0NodeId);
		}

		focusNavigator.registerRoot(homeGraph);
	}

	let graphsReady = 0;

	function onSection0Ready() {
		graphsReady++;
		if (graphsReady >= 2) {
			buildHomeGraph();
		}
	}

	function onSection1Ready() {
		graphsReady++;
		if (graphsReady >= 2) {
			buildHomeGraph();
		}
	}

	function handleTypeChange(v: string) {
		selectedType = v as typeof selectedType;
		section0Graph.value = null;
		section1Graph.value = null;
		graphsReady = 0;
		focusNavigator.root = null;
	}
</script>

<div class="h-full overflow-y-auto">
	<PageTabBar options={typeOptions} value={selectedType} onchange={handleTypeChange} />

	<main class="grid grid-rows-2 gap-10 overflow-hidden p-4">
		{#if selectedType === 'movie'}
			<HorizontalSection
				nodePrefix="home:section:0"
				title="热门电影"
				type="movie"
				tag="热门"
				sort="recommend"
				bind:graph={section0Graph}
				onGraphReady={onSection0Ready}
			/>
			<HorizontalSection
				nodePrefix="home:section:1"
				title="最新电影"
				type="movie"
				tag="最新"
				sort="time"
				bind:graph={section1Graph}
				onGraphReady={onSection1Ready}
			/>
		{:else}
			<HorizontalSection
				nodePrefix="home:section:0"
				title="热门电视剧"
				type="tv"
				tag="热门"
				sort="recommend"
				bind:graph={section0Graph}
				onGraphReady={onSection0Ready}
			/>
			<HorizontalSection
				nodePrefix="home:section:1"
				title="最新电视剧"
				type="tv"
				tag="热门"
				sort="time"
				bind:graph={section1Graph}
				onGraphReady={onSection1Ready}
			/>
		{/if}
	</main>
</div>
