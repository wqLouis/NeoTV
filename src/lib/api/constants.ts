export interface ApiSite {
	api: string;
	name: string;
	detail?: string;
	adult?: boolean;
}

export const API_SITES: Record<string, ApiSite> = {
	dyttzy: {
		api: 'http://caiji.dyttzyapi.com',
		name: '电影天堂资源',
		detail: 'http://caiji.dyttzyapi.com'
	},
	ruyi: { api: 'https://cj.rycjapi.com', name: '如意资源' },
	bfzy: { api: 'https://bfzyapi.com', name: '暴风资源' },
	tyyszy: { api: 'https://tyyszy.com', name: '天涯资源' },
	xiaomaomi: { api: 'https://zy.xiaomaomi.cc', name: '小猫咪资源' },
	ffzy: { api: 'http://ffzy5.tv', name: '非凡影视', detail: 'http://ffzy5.tv' },
	heimuer: { api: 'https://json.heimuer.xyz', name: '黑木耳', detail: 'https://heimuer.tv' },
	zy360: { api: 'https://360zy.com', name: '360资源' },
	wolong: { api: 'https://wolongzyw.com', name: '卧龙资源' },
	hwba: { api: 'https://cjhwba.com', name: '华为吧资源' },
	jisu: { api: 'https://jszyapi.com', name: '极速资源', detail: 'https://jszyapi.com' },
	dbzy: { api: 'https://dbzy.com', name: '豆瓣资源' },
	mozhua: { api: 'https://mozhuazy.com', name: '魔爪资源' },
	mdzy: { api: 'https://www.mdzyapi.com', name: '魔都资源' },
	zuid: { api: 'https://api.zuidapi.com', name: '最大资源' },
	yinghua: { api: 'https://m3u8.apiyhzy.com', name: '樱花资源' },
	baidu: { api: 'https://api.apibdzy.com', name: '百度云资源' },
	wujin: { api: 'https://api.wujinapi.me', name: '无尽资源' },
	wwzy: { api: 'https://wwzy.tv', name: '旺旺短剧' },
	ikun: { api: 'https://ikunzyapi.com', name: 'iKun资源' },
	testSource: { api: 'https://www.example.com', name: '空内容测试源', adult: true }
};

export const DOUBAN_CHART_GENRE_IDS: Record<string, number> = {
	剧情: 11,
	喜剧: 24,
	动作: 5,
	爱情: 13,
	科幻: 17,
	动画: 25,
	悬疑: 10,
	惊悚: 19,
	恐怖: 20,
	纪录片: 1,
	短片: 23,
	情色: 6,
	同性: 26,
	音乐: 14,
	歌舞: 7,
	家庭: 28,
	儿童: 8,
	传记: 2,
	历史: 4,
	战争: 22,
	犯罪: 3,
	西部: 27,
	奇幻: 16,
	冒险: 15,
	灾难: 12,
	武侠: 29,
	古装: 30,
	运动: 18,
	黑色电影: 31
};

export const AGGREGATED_SEARCH_CONFIG = {
	enabled: true,
	timeout: 8000,
	maxResults: 10000,
	parallelRequests: true,
	showSourceBadges: true
};

export const API_CONFIG = {
	search: {
		path: '/api.php/provide/vod/?ac=videolist&wd=',
		pagePath: '/api.php/provide/vod/?ac=videolist&wd={query}&pg={page}',
		maxPages: 50,
		headers: {
			'User-Agent':
				'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36',
			Accept: 'application/json'
		}
	},
	detail: {
		path: '/api.php/provide/vod/?ac=videolist&ids=',
		headers: {
			'User-Agent':
				'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36',
			Accept: 'application/json'
		}
	}
};

export const DOUBAN_NEW_SEARCH_API_BASE = 'https://movie.douban.com/j/new_search_subjects';
export const DOUBAN_CHART_TOP_LIST_BASE = 'https://movie.douban.com/j/chart/top_list';
export const DOUBAN_TAGS_BASE = 'https://movie.douban.com/j/search_tags';

export const YELLOW_FILTER_BANNED = [
	'伦理片',
	'福利',
	'里番动漫',
	'门事件',
	'萝莉少女',
	'制服诱惑',
	'国产传媒',
	'cosplay',
	'黑丝诱惑',
	'无码',
	'日本无码',
	'有码',
	'日本有码',
	'SWAG',
	'网红主播',
	'色情片',
	'同性片',
	'福利视频',
	'福利片'
];

export const SITE_CONFIG = {
	name: 'LibreTV',
	url: 'https://libretv.is-an.org',
	description: '免费在线视频搜索与观看平台',
	logo: './image/retrotv_5520.png',
	version: '1.0.3'
};
