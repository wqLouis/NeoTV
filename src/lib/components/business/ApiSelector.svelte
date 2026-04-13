<script lang="ts">
	import { settingsStore } from '$lib/stores/settings.svelte';
	import { API_SITES } from '$lib/api/constants';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Separator } from '$lib/components/ui/separator';

	interface BuiltinApiEntry {
		key: string;
		api: string;
		name: string;
		detail?: string;
		adult?: boolean;
	}

	let newCustomName = $state('');
	let newCustomUrl = $state('');

	const builtinApis: BuiltinApiEntry[] = Object.entries(API_SITES).map(([key, site]) => ({
		key,
		...site
	}));

	function isApiSelected(key: string): boolean {
		return settingsStore.selectedApis.includes(key);
	}

	function toggleApi(key: string) {
		settingsStore.toggleApi(key);
	}

	function selectAllApis() {
		const allKeys = builtinApis.map((e) => e.key);
		settingsStore.setSelectedApis(allKeys);
	}

	function reverseSelectApis() {
		const allKeys = builtinApis.map((e) => e.key);
		const currentSelected = settingsStore.selectedApis;
		const newSelection = allKeys.filter((key) => !currentSelected.includes(key));
		settingsStore.setSelectedApis(newSelection);
	}

	function addCustomApi() {
		if (!newCustomName.trim() || !newCustomUrl.trim()) return;

		settingsStore.addCustomApi({
			name: newCustomName.trim(),
			api: newCustomUrl.trim()
		});
		newCustomName = '';
		newCustomUrl = '';
	}

	function removeCustomApi(index: number) {
		settingsStore.removeCustomApi(index);
	}
</script>

<div class="space-y-4">
	<div class="flex items-center justify-between">
		<p class="text-sm text-muted-foreground">选择要使用的视频源，至少选择一个</p>
		<div class="flex gap-2">
			<Button variant="ghost" size="sm" onclick={selectAllApis}>全选</Button>
			<Button variant="ghost" size="sm" onclick={reverseSelectApis}>反选</Button>
		</div>
	</div>
	<div class="grid grid-cols-2 gap-2 md:grid-cols-3 lg:grid-cols-4">
		{#each builtinApis as entry (entry.key)}
			<button
				class="flex items-center gap-2 rounded-md border px-3 py-2 text-left transition-colors
					{isApiSelected(entry.key) ? 'border-primary bg-primary/10' : 'bg-transparent hover:bg-accent'}"
				onclick={() => toggleApi(entry.key)}
			>
				<div
					class="flex h-4 w-4 shrink-0 items-center justify-center rounded border
						{isApiSelected(entry.key) ? 'border-primary bg-primary' : 'border-muted-foreground'}"
				>
					{#if isApiSelected(entry.key)}
						<svg class="h-3 w-3 text-primary-foreground" viewBox="0 0 12 12" fill="none">
							<path
								d="M2 6l3 3 5-5"
								stroke="currentColor"
								stroke-width="2"
								stroke-linecap="round"
								stroke-linejoin="round"
							/>
						</svg>
					{/if}
				</div>
				<span class="truncate text-sm">{entry.name}</span>
			</button>
		{/each}
	</div>

	<Separator />

	<div class="space-y-3">
		<div class="flex items-center justify-between">
			<Label>自定义 API 源</Label>
			<span class="text-xs text-muted-foreground">
				已添加 {settingsStore.customApis.length} 个
			</span>
		</div>
		<div class="flex gap-2">
			<Input placeholder="名称" bind:value={newCustomName} class="w-32" />
			<Input placeholder="API 地址" bind:value={newCustomUrl} class="grow" />
			<Button onclick={addCustomApi}>添加</Button>
		</div>
		{#if settingsStore.customApis.length > 0}
			<div class="space-y-2">
				{#each settingsStore.customApis as api, i}
					<div class="flex items-center gap-2 rounded-md bg-secondary px-3 py-2">
						<span class="grow text-sm">{api.name}</span>
						<span class="max-w-50 truncate text-xs text-muted-foreground">{api.api}</span>
						<Button variant="ghost" size="icon" onclick={() => removeCustomApi(i)}>
							<svg
								class="h-4 w-4"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="2"
							>
								<path
									d="M3 6h18M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"
								/>
							</svg>
						</Button>
					</div>
				{/each}
			</div>
		{/if}
	</div>
</div>
