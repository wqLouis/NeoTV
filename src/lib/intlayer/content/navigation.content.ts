import { t, type Dictionary } from "intlayer";

const navigationContent = {
  key: "navigation",
  content: {
    search: t({
      en: "Search",
      "zh-Hans": "搜索",
    }),
    home: t({
      en: "Home",
      "zh-Hans": "首页",
    }),
    browse: t({
      en: "Browse",
      "zh-Hans": "浏览",
    }),
    history: t({
      en: "History",
      "zh-Hans": "历史",
    }),
    favourites: t({
      en: "Favourites",
      "zh-Hans": "收藏",
    }),
    settings: t({
      en: "Settings",
      "zh-Hans": "设置",
    }),
  },
} satisfies Dictionary;

export default navigationContent;
