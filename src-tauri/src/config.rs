use once_cell::sync::Lazy;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum ApiType {
    Json,
    Html,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ApiSourceInfo {
    pub api_base_url: String,
    pub name: String,
    pub detail_base_url: Option<String>,
    pub api_type: ApiType,
    pub search_path: Option<String>,
    pub detail_path: Option<String>,
}

pub struct ApiPathConfig {
    pub search: String,
}

static API_PATH_DEFAULTS: Lazy<ApiPathConfig> = Lazy::new(|| ApiPathConfig {
    search: "/api.php/provide/vod/?ac=videolist&wd=".to_string(),
});

pub static API_SITES_CONFIG: Lazy<HashMap<String, ApiSourceInfo>> = Lazy::new(|| {
    let mut m = HashMap::new();

    m.insert(
        "dyttzy".to_string(),
        ApiSourceInfo {
            api_base_url: "http://caiji.dyttzyapi.com".to_string(),
            name: "电影天堂资源".to_string(),
            detail_base_url: Some("http://caiji.dyttzyapi.com".to_string()),
            api_type: ApiType::Json,
            search_path: None,
            detail_path: None,
        },
    );
    m.insert(
        "ruyi".to_string(),
        ApiSourceInfo {
            api_base_url: "https://cj.rycjapi.com".to_string(),
            name: "如意资源".to_string(),
            detail_base_url: None,
            api_type: ApiType::Json,
            search_path: None,
            detail_path: None,
        },
    );
    m.insert(
        "bfzy".to_string(),
        ApiSourceInfo {
            api_base_url: "https://bfzyapi.com".to_string(),
            name: "暴风资源".to_string(),
            detail_base_url: None,
            api_type: ApiType::Json,
            search_path: None,
            detail_path: None,
        },
    );
    m.insert(
        "tyyszy".to_string(),
        ApiSourceInfo {
            api_base_url: "https://tyyszy.com".to_string(),
            name: "天涯资源".to_string(),
            detail_base_url: None,
            api_type: ApiType::Json,
            search_path: None,
            detail_path: None,
        },
    );
    m.insert(
        "xiaomaomi".to_string(),
        ApiSourceInfo {
            api_base_url: "https://zy.xiaomaomi.cc".to_string(),
            name: "小猫咪资源".to_string(),
            detail_base_url: None,
            api_type: ApiType::Json,
            search_path: None,
            detail_path: None,
        },
    );
    m.insert(
        "ffzy".to_string(),
        ApiSourceInfo {
            api_base_url: "http://ffzy5.tv".to_string(),
            name: "非凡影视".to_string(),
            detail_base_url: Some("http://ffzy5.tv".to_string()),
            api_type: ApiType::Html,
            search_path: None,
            detail_path: Some("/index.php/vod/detail/id/{id}.html".to_string()),
        },
    );
    m.insert(
        "heimuer".to_string(),
        ApiSourceInfo {
            api_base_url: "https://json.heimuer.xyz".to_string(),
            name: "黑木耳".to_string(),
            detail_base_url: Some("https://heimuer.tv".to_string()),
            api_type: ApiType::Html,
            search_path: None,
            detail_path: Some("/index.php/vod/detail/id/{id}.html".to_string()),
        },
    );
    m.insert(
        "zy360".to_string(),
        ApiSourceInfo {
            api_base_url: "https://360zy.com".to_string(),
            name: "360资源".to_string(),
            detail_base_url: None,
            api_type: ApiType::Json,
            search_path: None,
            detail_path: None,
        },
    );
    m.insert(
        "wolong".to_string(),
        ApiSourceInfo {
            api_base_url: "https://wolongzyw.com".to_string(),
            name: "卧龙资源".to_string(),
            detail_base_url: None,
            api_type: ApiType::Json,
            search_path: None,
            detail_path: None,
        },
    );
    m.insert(
        "hwba".to_string(),
        ApiSourceInfo {
            api_base_url: "https://cjhwba.com".to_string(),
            name: "华为吧资源".to_string(),
            detail_base_url: None,
            api_type: ApiType::Json,
            search_path: None,
            detail_path: None,
        },
    );
    m.insert(
        "jisu".to_string(),
        ApiSourceInfo {
            api_base_url: "https://jszyapi.com".to_string(),
            name: "极速资源".to_string(),
            detail_base_url: Some("https://jszyapi.com".to_string()),
            api_type: ApiType::Json,
            search_path: None,
            detail_path: None,
        },
    );
    m.insert(
        "dbzy".to_string(),
        ApiSourceInfo {
            api_base_url: "https://dbzy.com".to_string(),
            name: "豆瓣资源".to_string(),
            detail_base_url: None,
            api_type: ApiType::Json,
            search_path: None,
            detail_path: None,
        },
    );
    m.insert(
        "mozhua".to_string(),
        ApiSourceInfo {
            api_base_url: "https://mozhuazy.com".to_string(),
            name: "魔爪资源".to_string(),
            detail_base_url: None,
            api_type: ApiType::Json,
            search_path: None,
            detail_path: None,
        },
    );
    m.insert(
        "mdzy".to_string(),
        ApiSourceInfo {
            api_base_url: "https://www.mdzyapi.com".to_string(),
            name: "魔都资源".to_string(),
            detail_base_url: None,
            api_type: ApiType::Json,
            search_path: None,
            detail_path: None,
        },
    );
    m.insert(
        "zuid".to_string(),
        ApiSourceInfo {
            api_base_url: "https://api.zuidapi.com".to_string(),
            name: "最大资源".to_string(),
            detail_base_url: None,
            api_type: ApiType::Json,
            search_path: None,
            detail_path: None,
        },
    );
    m.insert(
        "yinghua".to_string(),
        ApiSourceInfo {
            api_base_url: "https://m3u8.apiyhzy.com".to_string(),
            name: "樱花资源".to_string(),
            detail_base_url: None,
            api_type: ApiType::Json,
            search_path: None,
            detail_path: None,
        },
    );
    m.insert(
        "baidu".to_string(),
        ApiSourceInfo {
            api_base_url: "https://api.apibdzy.com".to_string(),
            name: "百度云资源".to_string(),
            detail_base_url: None,
            api_type: ApiType::Json,
            search_path: None,
            detail_path: None,
        },
    );
    m.insert(
        "wujin".to_string(),
        ApiSourceInfo {
            api_base_url: "https://api.wujinapi.me".to_string(),
            name: "无尽资源".to_string(),
            detail_base_url: None,
            api_type: ApiType::Json,
            search_path: None,
            detail_path: None,
        },
    );
    m.insert(
        "wwzy".to_string(),
        ApiSourceInfo {
            api_base_url: "https://wwzy.tv".to_string(),
            name: "旺旺短剧".to_string(),
            detail_base_url: None,
            api_type: ApiType::Json,
            search_path: None,
            detail_path: None,
        },
    );
    m.insert(
        "ikun".to_string(),
        ApiSourceInfo {
            api_base_url: "https://ikunzyapi.com".to_string(),
            name: "iKun资源".to_string(),
            detail_base_url: None,
            api_type: ApiType::Json,
            search_path: None,
            detail_path: None,
        },
    );

    m
});

pub fn get_api_source(source_id: &str) -> Option<ApiSourceInfo> {
    API_SITES_CONFIG.get(source_id).cloned()
}

pub fn get_search_path(source_info: &ApiSourceInfo) -> String {
    source_info
        .search_path
        .clone()
        .unwrap_or_else(|| API_PATH_DEFAULTS.search.clone())
}
