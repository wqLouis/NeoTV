import { t, type Dictionary } from "intlayer";

const commonContent = {
  key: "common",
  content: {
    // Browse page
    browseTitle: t({
      en: "Browse",
      "zh-Hans": "浏览",
    }),
    // Favourites page
    favouritesTitle: t({
      en: "Favourites",
      "zh-Hans": "收藏",
    }),
    noFavourites: t({
      en: "No favourite videos yet",
      "zh-Hans": "暂无收藏的视频",
    }),
    addFavouritesHint: t({
      en: "Browse and add videos to your favourites",
      "zh-Hans": "浏览并添加视频到收藏夹",
    }),
    // History page
    historyTitle: t({
      en: "History",
      "zh-Hans": "历史",
    }),
    noHistory: t({
      en: "No watch history",
      "zh-Hans": "暂无观看记录",
    }),
    clearHistory: t({
      en: "Clear History",
      "zh-Hans": "清空历史",
    }),
    // Player
    selectSource: t({
      en: "Select Source",
      "zh-Hans": "选择播放源",
    }),
    noSourcesAvailable: t({
      en: "No sources available",
      "zh-Hans": "暂无可用播放源",
    }),
    loadingSources: t({
      en: "Loading sources...",
      "zh-Hans": "加载播放源中...",
    }),
    // Video source overlay
    playNow: t({
      en: "Play Now",
      "zh-Hans": "立即播放",
    }),
    addToFavourites: t({
      en: "Add to Favourites",
      "zh-Hans": "添加到收藏",
    }),
    removeFromFavourites: t({
      en: "Remove from Favourites",
      "zh-Hans": "取消收藏",
    }),
    // Toast
    addedToFavourites: t({
      en: "Added to favourites",
      "zh-Hans": "已添加到收藏",
    }),
    removedFromFavourites: t({
      en: "Removed from favourites",
      "zh-Hans": "已取消收藏",
    }),
  },
} satisfies Dictionary;

export default commonContent;
