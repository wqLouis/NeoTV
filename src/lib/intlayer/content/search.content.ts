import { t, type Dictionary } from "intlayer";

const searchContent = {
  key: "search",
  content: {
    placeholder: t({
      en: "Search videos...",
      "zh-Hans": "搜索视频...",
    }),
    webSearch: t({
      en: "Web Search",
      "zh-Hans": "全网搜索",
    }),
    doubanFilter: t({
      en: "Douban Filter",
      "zh-Hans": "豆瓣筛选",
    }),
    // Douban type buttons
    movie: t({
      en: "Movie",
      "zh-Hans": "电影",
    }),
    tv: t({
      en: "TV Series",
      "zh-Hans": "电视剧",
    }),
    // Quick filters
    hot: t({
      en: "Hot",
      "zh-Hans": "热门",
    }),
    new: t({
      en: "New",
      "zh-Hans": "最新",
    }),
    top: t({
      en: "Top Rated",
      "zh-Hans": "高分",
    }),
    // Filter labels
    genre: t({
      en: "Genre",
      "zh-Hans": "类型",
    }),
    country: t({
      en: "Country",
      "zh-Hans": "地区",
    }),
    rating: t({
      en: "Rating",
      "zh-Hans": "评分",
    }),
    filter: t({
      en: "Filter",
      "zh-Hans": "筛选",
    }),
    all: t({
      en: "All",
      "zh-Hans": "全部",
    }),
    // Search history
    searchHistory: t({
      en: "Search History",
      "zh-Hans": "搜索历史",
    }),
    clear: t({
      en: "Clear",
      "zh-Hans": "清空",
    }),
    // Results
    resultsFound: t({
      en: "results found",
      "zh-Hans": "个结果",
    }),
    noResults: t({
      en: "No results found",
      "zh-Hans": "没有找到匹配的结果",
    }),
    tryDifferent: t({
      en: "Try different keywords or change the data source",
      "zh-Hans": "请尝试其他关键词或更换数据源",
    }),
    emptyState: t({
      en: "Enter keywords to search",
      "zh-Hans": "输入关键词搜索视频",
    }),
    // Validation
    selectApiWarning: t({
      en: "Please select at least one API source",
      "zh-Hans": "请至少选择一个API源",
    }),
    // Countries
    mainland: t({
      en: "Mainland China",
      "zh-Hans": "中国大陆",
    }),
    usa: t({
      en: "USA",
      "zh-Hans": "美国",
    }),
    japan: t({
      en: "Japan",
      "zh-Hans": "日本",
    }),
    korea: t({
      en: "South Korea",
      "zh-Hans": "韩国",
    }),
    hongkong: t({
      en: "Hong Kong",
      "zh-Hans": "香港",
    }),
    taiwan: t({
      en: "Taiwan",
      "zh-Hans": "台湾",
    }),
    uk: t({
      en: "UK",
      "zh-Hans": "英国",
    }),
    france: t({
      en: "France",
      "zh-Hans": "法国",
    }),
    germany: t({
      en: "Germany",
      "zh-Hans": "德国",
    }),
    italy: t({
      en: "Italy",
      "zh-Hans": "意大利",
    }),
    spain: t({
      en: "Spain",
      "zh-Hans": "西班牙",
    }),
    india: t({
      en: "India",
      "zh-Hans": "印度",
    }),
    thailand: t({
      en: "Thailand",
      "zh-Hans": "泰国",
    }),
    other: t({
      en: "Other",
      "zh-Hans": "其他",
    }),
    // Rating options
    allRating: t({
      en: "All",
      "zh-Hans": "全部",
    }),
    above9: t({
      en: "Above 9",
      "zh-Hans": "9分以上",
    }),
    above8: t({
      en: "Above 8",
      "zh-Hans": "8分以上",
    }),
    above7: t({
      en: "Above 7",
      "zh-Hans": "7分以上",
    }),
  },
} satisfies Dictionary;

export default searchContent;
