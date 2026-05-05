# NeoTV 📺

**一个免费、跨平台的在线视频搜索与观看应用。**

![GitHub License](https://img.shields.io/github/license/wqLouis/NeoTV)

NeoTV 是一个基于LibreTV App的免费的开源在线视频搜索与观看平台，旨在提供简洁、高效、无广告的观影体验。基于 [Tauri 2.0](https://tauri.app/) 框架开发，实现了对 Windows, macOS, Linux, 以及 Android 的原生支持。

<img width="3200" height="2136" alt="Screenshot_2026-05-05-22-19-40-251_com neotv app" src="https://github.com/user-attachments/assets/147ba1ec-059d-4182-b690-6c14396df15d" />

<img width="3200" height="2136" alt="Screenshot_2026-05-05-22-19-51-520_com neotv app" src="https://github.com/user-attachments/assets/a3288bec-cdf7-4525-a685-a44591e8c5c9" />

---

## ✨ 主要特性

- 🎬 **海量资源聚合**: 搜索来自多个在线视频源的电影、电视剧、动漫及综艺节目。
- 🖥️ **原生跨平台体验**: 基于 Tauri 2.0 构建，提供接近原生的性能和流畅体验。支持 Windows, macOS, Linux, 和 Android。
- 📱 **移动端优化**: UI 界面为移动设备深度优化，确保在手机和平板上也能舒适操作。
- 🔍 **豆瓣内容集成**: 浏览豆瓣高分电影和电视剧推荐，支持豆瓣排行榜和类型筛选。
- 🚀 **智能选源**: 自动测试并选择最佳播放源，支持 HLS 视频播放。
- 🔐 **开源与安全**: 项目完全开源，代码透明，无任何跟踪或广告。

---

## 🛠️ 技术栈

- **核心框架**: [Tauri 2.0](https://tauri.app/)
- **前端**: Svelte 5, TailwindCSS, TypeScript
- **后端**: Rust
- **视频播放**: hls.js

---

## 🚀 快速开始

### 从源码编译

```bash
# 安装依赖
bun install

# 运行开发模式
bun run dev

# 构建 web 前端
bun run build

# 构建 Android APK
./scripts/build-android.sh
```

### 前置要求

- Node.js 18+
- Rust 1.77+
- Android SDK (仅用于 Android 构建)

---

## ⚠️ 免责声明

NeoTV 仅作为视频搜索工具，不存储、上传或分发任何视频内容。所有视频均来自第三方 API 接口提供的搜索结果。如有侵权内容，请联系相应的内容提供方。

本项目开发者不对使用本项目产生的任何后果负责。使用本项目时，您必须遵守当地的法律法规。
