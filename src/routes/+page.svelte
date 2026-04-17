<script lang="ts">
	import HorizontalSection from '$lib/components/HorizontalSection.svelte';
	import PageTabBar from '$lib/components/business/PageTabBar.svelte';
	import { Film, Tv } from '@lucide/svelte';
	import { onMount, onDestroy } from 'svelte';
	import { SvelteMap } from 'svelte/reactivity';
	import { NavNode, type NavDir } from '$lib/nav-graph/Core';
	import { registerPageNav, unregisterPageNav } from '$lib/nav-graph/store';

	let selectedType = $state<'movie' | 'tv'>('movie');

	const typeOptions = [
		{ value: 'movie', label: '电影', icon: Film },
		{ value: 'tv', label: '电视剧', icon: Tv }
	];

	function handleTypeChange(v: string) {
		selectedType = v as typeof selectedType;
	}

	let tabBarEl: PageTabBar;
	let section0El: HorizontalSection | undefined = $state();
	let section1El: HorizontalSection | undefined = $state();
	let pageEl: HTMLDivElement;

	function buildNavNode(): NavNode | null {
		if (!pageEl || !tabBarEl || !section0El || !section1El) return null;

		const tabSection = tabBarEl.buildNavNode();
		const section0 = section0El.buildNavNode();
		const section1 = section1El.buildNavNode();

		console.log(
			'[node] Page building - tabSection:',
			tabSection,
			'section0:',
			section0,
			'section1:',
			section1
		);

		if (!tabSection || !section0 || !section1) return null;

		const pageNavNode = new NavNode(pageEl, tabSection, new SvelteMap());

		pageNavNode.navGraph.set(tabSection, tabSection.navGraph.get(tabSection) || new SvelteMap());
		pageNavNode.navGraph.set(section0, section0.navGraph.get(section0) || new SvelteMap());
		pageNavNode.navGraph.set(section1, section1.navGraph.get(section1) || new SvelteMap());

		pageNavNode.navGraph.get(tabSection)!.set('DOWN', section0);
		pageNavNode.navGraph.get(section0)!.set('UP', tabSection);
		pageNavNode.navGraph.get(section0)!.set('DOWN', section1);
		pageNavNode.navGraph.get(section1)!.set('UP', section0);

		console.log(
			'[node] Page linking: tabSection→section0 (DOWN), section0→section1 (DOWN), section1→section0 (UP)'
		);

		console.log('[node] pageNavNode.navGraph entries:');
		for (const [node, dirs] of pageNavNode.navGraph) {
			const dirObj: Record<string, NavNode | undefined> = {};
			dirs.forEach((target, dir) => {
				dirObj[dir] = target;
			});
			console.log(`  ${node.element.tagName}.${node.element.className.split(' ')[0]} ->`, dirObj);
		}

		// Log the tabSection DOWN connection specifically
		const tabSectionEntry = pageNavNode.navGraph.get(tabSection);
		console.log(
			'[node] tabSection entry:',
			tabSectionEntry,
			'DOWN target:',
			tabSectionEntry?.get('DOWN')
		);

		return pageNavNode;
	}

	export { buildNavNode };

	onMount(() => {
		registerPageNav('/', buildNavNode);
	});

	onDestroy(() => {
		unregisterPageNav('/');
	});
</script>

<div bind:this={pageEl} class="h-full overflow-y-auto">
	<PageTabBar
		bind:this={tabBarEl}
		options={typeOptions}
		value={selectedType}
		onchange={handleTypeChange}
	/>

	<main class="grid grid-rows-2 gap-10 overflow-hidden p-4">
		{#if selectedType === 'movie'}
			<HorizontalSection
				bind:this={section0El}
				title="热门电影"
				type="movie"
				tag="热门"
				sort="recommend"
			/>
			<HorizontalSection
				bind:this={section1El}
				title="最新电影"
				type="movie"
				tag="最新"
				sort="time"
			/>
		{:else}
			<HorizontalSection
				bind:this={section0El}
				title="热门电视剧"
				type="tv"
				tag="热门"
				sort="recommend"
			/>
			<HorizontalSection
				bind:this={section1El}
				title="最新电视剧"
				type="tv"
				tag="热门"
				sort="time"
			/>
		{/if}
	</main>
</div>
