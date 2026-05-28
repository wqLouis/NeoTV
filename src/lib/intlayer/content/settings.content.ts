import { t, type Dictionary } from "intlayer";

const settingsContent = {
  key: "settings",
  content: {
    title: t({
      en: "Settings",
      "zh-Hans": "设置",
    }),
    export: t({
      en: "Export",
      "zh-Hans": "导出",
    }),
    import: t({
      en: "Import",
      "zh-Hans": "导入",
    }),
    reset: t({
      en: "Reset",
      "zh-Hans": "重置",
    }),
    apiSource: t({
      en: "API Source Selection",
      "zh-Hans": "API 源选择",
    }),
    sourceSpeedTest: t({
      en: "Source Speed Test",
      "zh-Hans": "源测速",
    }),
    startSpeedTest: t({
      en: "Start Test",
      "zh-Hans": "开始测速",
    }),
    testing: t({
      en: "Testing...",
      "zh-Hans": "测速中...",
    }),
    optimize: t({
      en: "Optimize",
      "zh-Hans": "优化源",
    }),
    optimizing: t({
      en: "Optimizing...",
      "zh-Hans": "优化中...",
    }),
    speedTestHint: t({
      en: "Click 'Start Test' to check speed and latency of all selected sources",
      "zh-Hans": "点击'开始测速'检测所有已选源的速度和延迟",
    }),
    appearance: t({
      en: "Appearance",
      "zh-Hans": "外观",
    }),
    tvNavMode: t({
      en: "TV Navigation Mode",
      "zh-Hans": "TV 导航模式",
    }),
    tvNavModeHint: t({
      en: "Use arrow keys to navigate the interface",
      "zh-Hans": "使用方向键在界面中导航",
    }),
    contentFilter: t({
      en: "Content Filter",
      "zh-Hans": "内容过滤",
    }),
    yellowFilter: t({
      en: "Adult Content Filter",
      "zh-Hans": "黄色内容过滤",
    }),
    yellowFilterHint: t({
      en: "Filter adult content in search results",
      "zh-Hans": "搜索结果中过滤成人内容",
    }),
    commentaryFilter: t({
      en: "Commentary Filter",
      "zh-Hans": "解说过滤",
    }),
    commentaryFilterHint: t({
      en: "Auto-filter commentary and movie review videos",
      "zh-Hans": "自动过滤解说、电影解说等视频",
    }),
    adFilter: t({
      en: "Ad Filter",
      "zh-Hans": "广告过滤",
    }),
    adFilterHint: t({
      en: "Filter ad segments during video playback",
      "zh-Hans": "过滤视频播放中的广告片段",
    }),
    playback: t({
      en: "Playback Settings",
      "zh-Hans": "播放设置",
    }),
    autoplay: t({
      en: "Autoplay",
      "zh-Hans": "自动播放",
    }),
    autoplayHint: t({
      en: "Start playback automatically after page loads",
      "zh-Hans": "页面加载后自动开始播放",
    }),
    autoplayNext: t({
      en: "Auto Play Next Episode",
      "zh-Hans": "自动连播",
    }),
    autoplayNextHint: t({
      en: "Automatically play the next episode after current ends",
      "zh-Hans": "播放完毕后自动播放下一集",
    }),
    preloader: t({
      en: "Preloader Settings",
      "zh-Hans": "预加载设置",
    }),
    cacheSize: t({
      en: "Cache Size",
      "zh-Hans": "缓存大小",
    }),
    cacheSizeHint: t({
      en: "Maximum video preloader cache size",
      "zh-Hans": "视频预加载缓存最大占用",
    }),
    workerCount: t({
      en: "Concurrent Downloads",
      "zh-Hans": "并发下载数",
    }),
    workerCountHint: t({
      en: "Number of concurrent download segments for preloading",
      "zh-Hans": "预加载时并发下载的分段数",
    }),
    cacheManagement: t({
      en: "Cache Management",
      "zh-Hans": "缓存管理",
    }),
    cacheStats: t({
      en: "Cache Statistics",
      "zh-Hans": "缓存统计",
    }),
    hits: t({
      en: "Hits",
      "zh-Hans": "命中",
    }),
    misses: t({
      en: "Misses",
      "zh-Hans": "未命中",
    }),
    hitRate: t({
      en: "Hit Rate",
      "zh-Hans": "命中率",
    }),
    total: t({
      en: "Total",
      "zh-Hans": "总计",
    }),
    memory: t({
      en: "Memory",
      "zh-Hans": "内存",
    }),
    disk: t({
      en: "Disk",
      "zh-Hans": "磁盘",
    }),
    imageCache: t({
      en: "Image Cache",
      "zh-Hans": "图片缓存",
    }),
    speedCache: t({
      en: "Speed Cache",
      "zh-Hans": "速度缓存",
    }),
    speedCacheHint: t({
      en: "Cached source speed test results by network environment",
      "zh-Hans": "按网络环境保存的源测速结果",
    }),
    clearCache: t({
      en: "Clear Cache",
      "zh-Hans": "清除缓存",
    }),
    loading: t({
      en: "Loading...",
      "zh-Hans": "加载中...",
    }),
    // Toast messages
    cacheCleared: t({
      en: "Cache cleared",
      "zh-Hans": "缓存已清除",
    }),
    clearCacheFailed: t({
      en: "Failed to clear cache",
      "zh-Hans": "清除缓存失败",
    }),
    speedCacheCleared: t({
      en: "Speed cache cleared",
      "zh-Hans": "速度缓存已清除",
    }),
    speedCacheClearFailed: t({
      en: "Failed to clear speed cache",
      "zh-Hans": "清除速度缓存失败",
    }),
    speedTestComplete: t({
      en: "Speed test complete",
      "zh-Hans": "测速完成",
    }),
    speedTestFailed: t({
      en: "Speed test failed",
      "zh-Hans": "测速失败",
    }),
    optimizeSuccess: t({
      en: "Optimized: kept {kept} available sources, removed {removed} invalid sources",
      "zh-Hans": "已优化：保留 {kept} 个可用源，移除 {removed} 个无效源",
    }),
    allSourcesAvailable: t({
      en: "All {count} sources are available",
      "zh-Hans": "所有 {count} 个源均可正常使用",
    }),
    optimizeFailed: t({
      en: "Optimization failed",
      "zh-Hans": "优化失败",
    }),
    configExported: t({
      en: "Configuration exported",
      "zh-Hans": "配置导出成功",
    }),
    configImportSuccess: t({
      en: "Configuration imported successfully",
      "zh-Hans": "配置导入成功",
    }),
    configImportFailed: t({
      en: "Configuration import failed, please check the file format",
      "zh-Hans": "配置导入失败，请检查文件格式",
    }),
    selectSourceWarning: t({
      en: "Please select at least one API source first",
      "zh-Hans": "请先选择至少一个 API 源",
    }),
    // Theme selector
    theme: t({ en: "Theme", "zh-Hans": "主题" }),
    light: t({ en: "Light", "zh-Hans": "浅色" }),
    dark: t({ en: "Dark", "zh-Hans": "深色" }),
    system: t({ en: "System", "zh-Hans": "跟随系统" }),
    gridDensity: t({ en: "Grid Density", "zh-Hans": "每行显示数量" }),
    compact: t({ en: "Compact", "zh-Hans": "紧凑" }),
    standard: t({ en: "Standard", "zh-Hans": "标准" }),
    loose: t({ en: "Loose", "zh-Hans": "宽松" }),
    // API selector
    apiSelectorHint: t({ en: "Select video sources to use, at least one required", "zh-Hans": "选择要使用的视频源，至少选择一个" }),
    selectAll: t({ en: "Select All", "zh-Hans": "全选" }),
    reverseSelect: t({ en: "Reverse", "zh-Hans": "反选" }),
    customApi: t({ en: "Custom API Sources", "zh-Hans": "自定义 API 源" }),
    addedCount: t({ en: "{n} added", "zh-Hans": "已添加 {n} 个" }),
    name: t({ en: "Name", "zh-Hans": "名称" }),
    apiUrl: t({ en: "API URL", "zh-Hans": "API 地址" }),
    add: t({ en: "Add", "zh-Hans": "添加" }),
    // Player
    selectSource: t({ en: "Select Source", "zh-Hans": "选择播放源" }),
    noSourcesAvailable: t({ en: "No sources available", "zh-Hans": "暂无可用播放源" }),
    loadingSources: t({ en: "Loading sources...", "zh-Hans": "加载播放源中..." }),
    playNow: t({ en: "Play Now", "zh-Hans": "立即播放" }),
    addToFavourites: t({ en: "Add to Favourites", "zh-Hans": "添加到收藏" }),
    removeFromFavourites: t({ en: "Remove from Favourites", "zh-Hans": "取消收藏" }),
    addedToFavourites: t({ en: "Added to favourites", "zh-Hans": "已添加到收藏" }),
    removedFromFavourites: t({ en: "Removed from favourites", "zh-Hans": "已取消收藏" }),
    // Language
    language: t({ en: "Language", "zh-Hans": "语言" }),
    english: t({ en: "English", "zh-Hans": "English" }),
    chinese: t({ en: "简体中文", "zh-Hans": "简体中文" }),
  },
} satisfies Dictionary;

export default settingsContent;
