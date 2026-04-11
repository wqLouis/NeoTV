<script lang="ts">
	import {
		settingsStore,
		GRID_DENSITY_CLASSES,
		type GridDensity
	} from '$lib/stores/settings.svelte';
	import { API_SITES } from '$lib/api/constants';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Switch } from '$lib/components/ui/switch';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Separator } from '$lib/components/ui/separator';
	import { themeStore } from '$lib/stores/theme.svelte';
	import { toast } from 'svelte-sonner';
	import {
		testSourceSpeed,
		testAllSourcesSpeed,
		formatLatency,
		formatSpeed,
		getSpeedLevel,
		type SpeedTestResult
	} from '$lib/utils/speedTest';
	import { Gauge, Zap, XCircle, CheckCircle, Loader2 } from 'lucide-svelte';

	interface BuiltinApiEntry {
		key: string;
		api: string;
		name: string;
		detail?: string;
		adult?: boolean;
	}

	let cacheStats = $state<{
		hits: number;
		misses: number;
		total: number;
		hit_rate: number;
		mem_count: number;
		mem_size: number;
		disk_count: number;
		disk_size: number;
	} | null>(null);

	async function loadCacheStats() {
		try {
			const { invoke } = await import('@tauri-apps/api/core');
			cacheStats = await invoke('cache_stats');
		} catch {
			cacheStats = null;
		}
	}

	function formatSize(bytes: number): string {
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
		return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
	}

	async function clearCache() {
		try {
			const { invoke } = await import('@tauri-apps/api/core');
			await invoke('cache_clear');
			toast.success('缓存已清除');
			loadCacheStats();
		} catch {
			toast.error('清除缓存失败');
		}
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

	let speedTestResults = $state<SpeedTestResult[]>([]);
	let isTestingSpeed = $state(false);

	async function runSpeedTest() {
		if (isTestingSpeed) return;
		isTestingSpeed = true;
		speedTestResults = [];

		try {
			const allApis = [...settingsStore.selectedApis];
			for (let i = 0; i < settingsStore.customApis.length; i++) {
				allApis.push(`custom_${i}`);
			}

			if (allApis.length === 0) {
				toast.warning('请先选择至少一个 API 源');
				isTestingSpeed = false;
				return;
			}

			const results = await testAllSourcesSpeed(
				settingsStore.selectedApis,
				settingsStore.customApis
			);
			results.sort((a, b) => {
				if (a.status === 'success' && b.status !== 'success') return -1;
				if (b.status === 'success' && a.status !== 'success') return 1;
				return (a.latency_ms || Infinity) - (b.latency_ms || Infinity);
			});
			speedTestResults = results;
			toast.success('测速完成');
		} catch (e) {
			toast.error('测速失败');
		} finally {
			isTestingSpeed = false;
		}
	}

	let isOptimizing = $state(false);

	async function optimizeSources() {
		if (isOptimizing) return;
		isOptimizing = true;

		try {
			const allBuiltinKeys = builtinApis.map((e) => e.key);

			const results = await testAllSourcesSpeed(allBuiltinKeys, settingsStore.customApis);

			const successResults = results.filter((r) => r.status === 'success');
			const failedResults = results.filter((r) => r.status !== 'success');

			successResults.sort((a, b) => (a.latency_ms || Infinity) - (b.latency_ms || Infinity));

			const optimizedApis = successResults.map((r) => r.source_id);

			settingsStore.setSelectedApis(optimizedApis);

			const removedCount = failedResults.length;
			const keptCount = successResults.length;

			if (removedCount > 0) {
				toast.success(`已优化：保留 ${keptCount} 个可用源，移除 ${removedCount} 个无效源`);
			} else {
				toast.success(`所有 ${keptCount} 个源均可正常使用`);
			}

			speedTestResults = results;
		} catch (e) {
			toast.error('优化失败');
		} finally {
			isOptimizing = false;
		}
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

	$effect(() => {
		loadCacheStats();
	});
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
			</CardContent>
		</Card>

		<Card>
			<CardHeader>
				<div class="flex items-center justify-between">
					<CardTitle>源测速</CardTitle>
					<div class="flex gap-2">
						<Button
							variant="outline"
							size="sm"
							onclick={runSpeedTest}
							disabled={isTestingSpeed || isOptimizing}
						>
							{#if isTestingSpeed}
								<Loader2 class="mr-2 h-4 w-4 animate-spin" />
								测速中...
							{:else}
								<Zap class="mr-2 h-4 w-4" />
								开始测速
							{/if}
						</Button>
						<Button
							variant="default"
							size="sm"
							onclick={optimizeSources}
							disabled={isTestingSpeed || isOptimizing}
						>
							{#if isOptimizing}
								<Loader2 class="mr-2 h-4 w-4 animate-spin" />
								优化中...
							{:else}
								<Zap class="mr-2 h-4 w-4" />
								优化源
							{/if}
						</Button>
					</div>
				</div>
			</CardHeader>
			<CardContent class="space-y-3">
				{#if speedTestResults.length > 0}
					<div class="space-y-2">
						{#each speedTestResults as result}
							<div class="flex items-center justify-between rounded-md bg-secondary px-3 py-2">
								<div class="flex items-center gap-3">
									{#if result.status === 'success'}
										<CheckCircle class="h-4 w-4 text-green-500" />
									{:else}
										<XCircle class="h-4 w-4 text-red-500" />
									{/if}
									<span class="text-sm font-medium">{result.source_name}</span>
								</div>
								<div class="flex items-center gap-4 text-sm">
									{#if result.status === 'success'}
										<div class="flex items-center gap-1">
											<Gauge class="h-4 w-4 text-muted-foreground" />
											<span
												class={result.latency_ms < 500
													? 'text-green-500'
													: result.latency_ms < 1000
														? 'text-yellow-500'
														: 'text-red-500'}
											>
												{formatLatency(result.latency_ms)}
											</span>
										</div>
										<div class="flex items-center gap-1">
											<Zap class="h-4 w-4 text-muted-foreground" />
											<span
												class={getSpeedLevel(result.download_speed_kbps) === 'fast'
													? 'text-green-500'
													: getSpeedLevel(result.download_speed_kbps) === 'medium'
														? 'text-yellow-500'
														: 'text-red-500'}
											>
												{formatSpeed(result.download_speed_kbps)}
											</span>
										</div>
									{:else}
										<span class="text-xs text-muted-foreground">{result.error || '测速失败'}</span>
									{/if}
								</div>
							</div>
						{/each}
					</div>
				{:else}
					<p class="py-4 text-center text-sm text-muted-foreground">
						点击"开始测速"检测所有已选源的速度和延迟
					</p>
				{/if}
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

				<Separator />

				<div>
					<Label class="mb-3 block">每行显示数量</Label>
					<div class="flex gap-2">
						{#each ['compact', 'standard', 'loose'] as const as density}
							<button
								class="flex flex-1 flex-col items-center gap-1 rounded-lg border p-3 transition-colors
									{settingsStore.gridDensity === density
									? 'border-primary bg-primary/10'
									: 'border-border hover:bg-accent'}"
								onclick={() => settingsStore.setGridDensity(density)}
							>
								<span class="text-sm font-medium">
									{density === 'compact' ? '紧凑' : density === 'standard' ? '标准' : '宽松'}
								</span>
								<span class="text-xs text-muted-foreground">
									{density === 'compact' ? '8列' : density === 'standard' ? '6列' : '5列'}
								</span>
							</button>
						{/each}
					</div>
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
						<Label>解说过滤</Label>
						<p class="text-sm text-muted-foreground">自动过滤解说、电影解说等视频</p>
					</div>
					<Switch
						checked={settingsStore.commentaryFilterEnabled}
						onCheckedChange={(v: boolean) => settingsStore.setCommentaryFilterEnabled(v)}
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
						checked={settingsStore.autoplayNextEpisode}
						onCheckedChange={(v: boolean) => settingsStore.setAutoplayNextEpisode(v)}
					/>
				</div>
				<Separator />
				<div class="flex items-center justify-between">
					<div>
						<Label>自动整合源</Label>
						<p class="text-sm text-muted-foreground">后台测试所有源，自动选择可播放的</p>
					</div>
					<Switch
						checked={settingsStore.autoIntegrateSources}
						onCheckedChange={(v: boolean) => settingsStore.setAutoIntegrateSources(v)}
					/>
				</div>
			</CardContent>
		</Card>

		<Card>
			<CardHeader>
				<CardTitle>缓存管理</CardTitle>
			</CardHeader>
			<CardContent class="space-y-4">
				{#if cacheStats}
					<div class="mb-4 rounded-lg bg-muted p-3">
						<div class="text-sm font-medium">缓存统计</div>
						<div class="mt-2 grid grid-cols-2 gap-2 text-xs text-muted-foreground">
							<div>命中: {cacheStats.hits}</div>
							<div>未命中: {cacheStats.misses}</div>
							<div>命中率: {(cacheStats.hit_rate * 100).toFixed(1)}%</div>
							<div>总计: {cacheStats.total}</div>
						</div>
					</div>
				{/if}
				<div class="flex items-center justify-between">
					<div>
						<Label>图片缓存</Label>
						<p class="text-sm text-muted-foreground">
							{#if cacheStats}
								内存: {cacheStats.mem_count} 项 ({formatSize(cacheStats.mem_size)}) | 磁盘: {cacheStats.disk_count}
								项 ({formatSize(cacheStats.disk_size)})
							{:else}
								加载中...
							{/if}
						</p>
					</div>
					<Button variant="outline" size="sm" onclick={clearCache}>清除缓存</Button>
				</div>
			</CardContent>
		</Card>
	</div>
</div>
