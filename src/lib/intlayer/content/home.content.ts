import { t, type Dictionary } from "intlayer";

const homeContent = {
  key: "home",
  content: {
    movie: t({
      en: "Movie",
      "zh-Hans": "电影",
    }),
    tv: t({
      en: "TV Series",
      "zh-Hans": "电视剧",
    }),
    hotMovies: t({
      en: "Hot Movies",
      "zh-Hans": "热门电影",
    }),
    latestMovies: t({
      en: "Latest Movies",
      "zh-Hans": "最新电影",
    }),
    hotTv: t({
      en: "Hot TV Series",
      "zh-Hans": "热门电视剧",
    }),
    latestTv: t({
      en: "Latest TV Series",
      "zh-Hans": "最新电视剧",
    }),
  },
} satisfies Dictionary;

export default homeContent;
