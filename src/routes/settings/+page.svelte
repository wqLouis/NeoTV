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
	import { Select, SelectTrigger, SelectContent, SelectItem } from '$lib/components/ui/select';
	import { toast } from 'svelte-sonner';
	import { invoke } from '@tauri-apps/api/core';
	import {
		formatLatency,
		formatSpeed,
		getSpeedLevel,
		type SpeedTestResult
	} from '$lib/utils/speedTest';
	import { Gauge, Zap, XCircle, CheckCircle, Loader2 } from '@lucide/svelte';
	import PageHeader from '$lib/components/business/PageHeader.svelte';
	import ApiSelector from '$lib/components/business/ApiSelector.svelte';
	import ThemeSelector from '$lib/components/business/ThemeSelector.svelte';

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

	let preloaderCacheSize = $state(settingsStore.preloaderCacheSizeMB.toString());
	let preloaderWorkerCount = $state(settingsStore.preloaderWorkerCount.toString());

	function handleCacheSizeChange(v: string) {
		preloaderCacheSize = v;
		const size = parseInt(v, 10);
		settingsStore.setPreloaderCacheSizeMB(size);
		invoke('preloader_set_max_cache_size', { bytes: size * 1024 * 1024 });
	}

	function handleWorkerCountChange(v: string) {
		preloaderWorkerCount = v;
		const count = parseInt(v, 10);
		settingsStore.setPreloaderWorkerCount(count);
		invoke('preloader_set_workers', { count });
	}

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

	async function clearSpeedCache() {
		try {
			await invoke('speed_cache_clear_all');
			toast.success('速度缓存已清除');
		} catch {
			toast.error('清除速度缓存失败');
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

	async function testSingleSource(sourceId: string, customUrl?: string): Promise<SpeedTestResult> {
		return await invoke<SpeedTestResult>('test_source_speed', {
			sourceId,
			customUrl
		});
	}

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

			const results: SpeedTestResult[] = [];
			for (const apiKey of allApis) {
				if (apiKey.startsWith('custom_')) {
					const idx = parseInt(apiKey.replace('custom_', ''), 10);
					const customApi = settingsStore.customApis[idx];
					if (customApi) {
						const result = await testSingleSource('custom', customApi.api);
						result.source_name = customApi.name;
						results.push(result);
					}
				} else {
					const result = await testSingleSource(apiKey);
					results.push(result);
				}
			}

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

			const results: SpeedTestResult[] = [];
			for (const apiKey of allBuiltinKeys) {
				const result = await testSingleSource(apiKey);
				results.push(result);
			}

			const successResults = results.filter((r) => r.status === 'success');
			const failedResults = results.filter((r) => r.status !== 'error');

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

<div class="container mx-auto h-full px-4 py-6">
	<PageHeader title="设置">
		{#snippet actions()}
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
		{/snippet}
	</PageHeader>

	<div class="space-y-6">
		<Card>
			<CardHeader>
				<CardTitle>API 源选择</CardTitle>
			</CardHeader>
			<CardContent>
				<ApiSelector />
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
				<ThemeSelector />
				<Separator />
				<div class="flex items-center justify-between">
					<div>
						<Label>TV 导航模式</Label>
						<p class="text-sm text-muted-foreground">使用方向键在界面中导航</p>
					</div>
					<Switch
						checked={settingsStore.tvNavModeEnabled}
						onCheckedChange={(v: boolean) => settingsStore.setTvNavModeEnabled(v)}
					/>
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
			</CardContent>
		</Card>

		<Card>
			<CardHeader>
				<CardTitle>预加载设置</CardTitle>
			</CardHeader>
			<CardContent class="space-y-4">
				<div class="flex items-center justify-between">
					<div>
						<Label>缓存大小</Label>
						<p class="text-sm text-muted-foreground">视频预加载缓存最大占用</p>
					</div>
					<Select
						type="single"
						bind:value={preloaderCacheSize}
						onValueChange={(v) => {
							if (v) {
								handleCacheSizeChange(v);
							}
						}}
					>
						<SelectTrigger class="w-32">
							<span>{settingsStore.preloaderCacheSizeMB} MB</span>
						</SelectTrigger>
						<SelectContent>
							<SelectItem value="256">256 MB</SelectItem>
							<SelectItem value="512">512 MB</SelectItem>
							<SelectItem value="1024">1 GB</SelectItem>
							<SelectItem value="2048">2 GB</SelectItem>
						</SelectContent>
					</Select>
				</div>
				<Separator />
				<div class="flex items-center justify-between">
					<div>
						<Label>并发下载数</Label>
						<p class="text-sm text-muted-foreground">预加载时并发下载的分段数</p>
					</div>
					<Select
						type="single"
						bind:value={preloaderWorkerCount}
						onValueChange={(v) => {
							if (v) {
								handleWorkerCountChange(v);
							}
						}}
					>
						<SelectTrigger class="w-24">
							<span>{settingsStore.preloaderWorkerCount}</span>
						</SelectTrigger>
						<SelectContent>
							<SelectItem value="2">2</SelectItem>
							<SelectItem value="4">4</SelectItem>
							<SelectItem value="6">6</SelectItem>
							<SelectItem value="8">8</SelectItem>
							<SelectItem value="12">12</SelectItem>
						</SelectContent>
					</Select>
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
				<Separator />
				<div class="flex items-center justify-between">
					<div>
						<Label>速度缓存</Label>
						<p class="text-sm text-muted-foreground">按网络环境保存的源测速结果</p>
					</div>
					<Button variant="outline" size="sm" onclick={clearSpeedCache}>清除速度缓存</Button>
				</div>
			</CardContent>
		</Card>
	</div>
</div>
