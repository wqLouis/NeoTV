<script lang="ts">
	import { settingsStore } from '$lib/stores/settings.svelte';
	import { API_SITES } from '$lib/api/constants';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Switch } from '$lib/components/ui/switch';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Separator } from '$lib/components/ui/separator';
	import { themeStore } from '$lib/stores/theme.svelte';

	interface BuiltinApiEntry {
		key: string;
		api: string;
		name: string;
		detail?: string;
		adult?: boolean;
	}

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

	let newCustomName = $state('');
	let newCustomUrl = $state('');

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

	function exportConfig() {
		const config = settingsStore.exportConfig();
		const blob = new Blob([config], { type: 'application/json' });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = 'libretv-config.json';
		a.click();
		URL.revokeObjectURL(url);
	}

	let importFile: HTMLInputElement;

	function triggerImport() {
		importFile?.click();
	}

	async function handleImport(e: Event) {
		const file = (e.target as HTMLInputElement).files?.[0];
		if (!file) return;

		const text = await file.text();
		const success = settingsStore.importConfig(text);
		if (success) {
			alert('配置导入成功');
		} else {
			alert('配置导入失败，请检查文件格式');
		}
	}
</script>

<div class="container mx-auto px-4 py-6">
	<div class="mb-6 flex items-center justify-between">
		<h1 class="text-2xl font-bold">设置</h1>
		<div class="flex gap-2">
			<Button variant="outline" size="sm" onclick={exportConfig}>导出</Button>
			<Button variant="outline" size="sm" onclick={triggerImport}>导入</Button>
			<input
				type="file"
				accept=".json"
				class="hidden"
				bind:this={importFile}
				onchange={handleImport}
			/>
			<Button variant="outline" size="sm" onclick={() => settingsStore.reset()}>重置</Button>
		</div>
	</div>

	<div class="space-y-6">
		<Card>
			<CardHeader>
				<CardTitle>API 源选择</CardTitle>
			</CardHeader>
			<CardContent class="space-y-4">
				<p class="text-sm text-muted-foreground">选择要使用的视频源，至少选择一个</p>
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
						<Input placeholder="API 地址" bind:value={newCustomUrl} class="flex-grow" />
						<Button onclick={addCustomApi}>添加</Button>
					</div>
					{#if settingsStore.customApis.length > 0}
						<div class="space-y-2">
							{#each settingsStore.customApis as api, i}
								<div class="flex items-center gap-2 rounded-md bg-secondary px-3 py-2">
									<span class="flex-grow text-sm">{api.name}</span>
									<span class="max-w-[200px] truncate text-xs text-muted-foreground">{api.api}</span
									>
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
			</CardContent>
		</Card>

		<Card>
			<CardHeader>
				<CardTitle>外观</CardTitle>
			</CardHeader>
			<CardContent class="space-y-4">
				<div class="flex gap-2">
					<button
						class="flex flex-1 flex-col items-center gap-2 rounded-lg border p-4 transition-colors
							{themeStore.current === 'light' ? 'border-primary bg-primary/10' : 'border-border hover:bg-accent'}"
						onclick={() => themeStore.setTheme('light')}
					>
						<svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z"
							/>
						</svg>
						<span class="text-sm">浅色</span>
					</button>
					<button
						class="flex flex-1 flex-col items-center gap-2 rounded-lg border p-4 transition-colors
							{themeStore.current === 'dark' ? 'border-primary bg-primary/10' : 'border-border hover:bg-accent'}"
						onclick={() => themeStore.setTheme('dark')}
					>
						<svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z"
							/>
						</svg>
						<span class="text-sm">深色</span>
					</button>
					<button
						class="flex flex-1 flex-col items-center gap-2 rounded-lg border p-4 transition-colors
							{themeStore.current === 'system'
							? 'border-primary bg-primary/10'
							: 'border-border hover:bg-accent'}"
						onclick={() => themeStore.setTheme('system')}
					>
						<svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"
							/>
						</svg>
						<span class="text-sm">跟随系统</span>
					</button>
				</div>
			</CardContent>
		</Card>

		<Card>
			<CardHeader>
				<CardTitle>内容过滤</CardTitle>
			</CardHeader>
			<CardContent class="space-y-4">
				<div class="flex items-center justify-between">
					<div>
						<Label>启用豆瓣</Label>
						<p class="text-sm text-muted-foreground">在首页显示豆瓣推荐内容</p>
					</div>
					<Switch
						checked={settingsStore.doubanEnabled}
						onCheckedChange={(v: boolean) => settingsStore.setDoubanEnabled(v)}
					/>
				</div>
				<Separator />
				<div class="flex items-center justify-between">
					<div>
						<Label>黄色内容过滤</Label>
						<p class="text-sm text-muted-foreground">搜索结果中过滤成人内容</p>
					</div>
					<Switch
						checked={settingsStore.yellowFilterEnabled}
						onCheckedChange={(v: boolean) => settingsStore.setYellowFilterEnabled(v)}
					/>
				</div>
				<Separator />
				<div class="flex items-center justify-between">
					<div>
						<Label>广告过滤</Label>
						<p class="text-sm text-muted-foreground">过滤视频播放中的广告片段</p>
					</div>
					<Switch
						checked={settingsStore.adFilteringEnabled}
						onCheckedChange={(v: boolean) => settingsStore.setAdFilteringEnabled(v)}
					/>
				</div>
			</CardContent>
		</Card>

		<Card>
			<CardHeader>
				<CardTitle>播放设置</CardTitle>
			</CardHeader>
			<CardContent class="space-y-4">
				<div class="flex items-center justify-between">
					<div>
						<Label>自动播放</Label>
						<p class="text-sm text-muted-foreground">页面加载后自动开始播放</p>
					</div>
					<Switch
						checked={settingsStore.autoplayEnabled}
						onCheckedChange={(v: boolean) => settingsStore.setAutoplayEnabled(v)}
					/>
				</div>
				<Separator />
				<div class="flex items-center justify-between">
					<div>
						<Label>自动连播</Label>
						<p class="text-sm text-muted-foreground">播放完毕后自动播放下一集</p>
					</div>
					<Switch
						checked={settingsStore.autoplayEnabled}
						onCheckedChange={(v: boolean) => settingsStore.setAutoplayEnabled(v)}
					/>
				</div>
			</CardContent>
		</Card>
	</div>
</div>
