// 自定义 HTTP API 音源
// 用户通过模板 URL + JSON 路径配置搜索/获取播放地址的接口
//
// 模板占位符：
//   搜索 URL：{keyword} {page} {limit}
//   URL 接口：{id} {quality}
//
// JSON 路径用 "." 分隔，如 "data.songs"
// 字段映射指定结果数组每项里各字段对应的 key 名

use async_trait::async_trait;
use serde_json::Value;

use super::{ExternalSong, MusicSource, SourceConfig, build_client};

const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36";

pub struct CustomHttpSource {
    cfg: SourceConfig,
}

impl CustomHttpSource {
    pub fn new(cfg: SourceConfig) -> Self {
        Self { cfg }
    }
}

fn client() -> Result<reqwest::Client, String> {
    build_client(UA, 10)
}

/// 按 "a.b.c" 路径访问嵌套 JSON
fn json_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let mut cur = value;
    for k in path.split('.') {
        if k.is_empty() {
            continue;
        }
        cur = cur.get(k)?;
    }
    Some(cur)
}

/// 取节点的字符串值（兼容字符串/数字）
fn value_to_string(v: &Value) -> String {
    v.as_str()
        .map(String::from)
        .unwrap_or_else(|| v.to_string())
}

#[async_trait]
impl MusicSource for CustomHttpSource {
    fn id(&self) -> &str {
        &self.cfg.id
    }
    fn label(&self) -> &str {
        &self.cfg.label
    }

    async fn search(&self, keyword: &str, page: u32, limit: u32) -> Result<Vec<ExternalSong>, String> {
        let client = client()?;
        let tmpl = self
            .cfg
            .search_url
            .as_deref()
            .ok_or("自定义音源缺少 search_url")?;
        let url = tmpl
            .replace("{keyword}", &urlencoding::encode(keyword))
            .replace("{page}", &page.to_string())
            .replace("{limit}", &limit.to_string());

        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("custom [{}] 请求失败: {}", self.cfg.id, e))?;
        let text = resp
            .text()
            .await
            .map_err(|e| format!("custom [{}] 读取失败: {}", self.cfg.id, e))?;
        let json: Value = serde_json::from_str(&text)
            .map_err(|e| format!("custom [{}] JSON 解析失败: {}", self.cfg.id, e))?;

        let arr_path = self.cfg.search_path.as_deref().unwrap_or("data");
        let arr = json_path(&json, arr_path).and_then(|v| v.as_array()).ok_or_else(|| {
            format!(
                "custom [{}] 找不到数组路径: {}（响应: {}）",
                self.cfg.id, arr_path, &text[..text.len().min(200)]
            )
        })?;

        let id_f = self.cfg.id_field.as_deref().unwrap_or("id");
        let name_f = self.cfg.name_field.as_deref().unwrap_or("name");
        let artist_f = self.cfg.artist_field.as_deref().unwrap_or("artist");
        let album_f = self.cfg.album_field.as_deref().unwrap_or("album");
        let dur_f = self.cfg.duration_field.as_deref().unwrap_or("duration");
        let pic_f = self.cfg.pic_field.as_deref().unwrap_or("pic");

        let list: Vec<ExternalSong> = arr
            .iter()
            .map(|s| {
                let id = json_path(s, id_f).map(value_to_string).unwrap_or_default();
                let name = json_path(s, name_f).and_then(|v| v.as_str()).unwrap_or("").to_string();
                let artist = json_path(s, artist_f).and_then(|v| v.as_str()).unwrap_or("").to_string();
                let album = json_path(s, album_f).and_then(|v| v.as_str()).unwrap_or("").to_string();
                let duration = json_path(s, dur_f)
                    .and_then(|v| {
                        v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                    })
                    .unwrap_or(0);
                let pic = json_path(s, pic_f).and_then(|v| v.as_str()).unwrap_or("").to_string();
                ExternalSong {
                    id,
                    source: self.cfg.id.clone(),
                    source_label: self.cfg.label.clone(),
                    name,
                    artist,
                    album,
                    duration,
                    pic_url: pic,
                }
            })
            .collect();
        Ok(list)
    }

    async fn get_url(&self, song_id: &str, quality: &str) -> Result<String, String> {
        let client = client()?;
        let tmpl = self.cfg.url_api.as_deref().ok_or("自定义音源缺少 url_api")?;
        let url = tmpl
            .replace("{id}", &urlencoding::encode(song_id))
            .replace("{quality}", quality);

        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("custom [{}] URL 请求失败: {}", self.cfg.id, e))?;
        let text = resp
            .text()
            .await
            .map_err(|e| format!("custom [{}] URL 读取失败: {}", self.cfg.id, e))?;

        // 先尝试 JSON 路径取 URL；不是 JSON 则视为纯 URL 文本
        if let Ok(json) = serde_json::from_str::<Value>(&text) {
            let url_path = self.cfg.url_path.as_deref().unwrap_or("data.url");
            if let Some(v) = json_path(&json, url_path) {
                if let Some(s) = v.as_str() {
                    if !s.is_empty() {
                        return Ok(s.to_string());
                    }
                }
            }
            return Err(format!(
                "custom [{}] URL 字段为空或路径错误: {}",
                self.cfg.id, text
            ));
        }

        let s = text.trim();
        if s.is_empty() {
            return Err("custom URL 为空".into());
        }
        Ok(s.to_string())
    }
}
