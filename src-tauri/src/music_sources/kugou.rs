// 酷狗音乐（内置音源）
// 搜索：mobilecdn.kugou.com/api/v3/search/song
// URL：trackercdn.kugou.com/i/v2/ （key = md5(hash + "kgcloudv2")）

use async_trait::async_trait;
use md5::compute as md5_compute;
use serde_json::Value;

use super::{ExternalSong, MusicSource, build_client, truncate_chars};

const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36";

pub struct KugouSource {
    label: String,
}

impl KugouSource {
    pub fn new(label: String) -> Self {
        Self { label }
    }
}

fn client() -> Result<reqwest::Client, String> {
    build_client(UA, 8)
}

/// 酷狗 V2 接口 key 算法：md5(hash + "kgcloudv2")
fn make_key(hash: &str) -> String {
    let data = format!("{}kgcloudv2", hash);
    let digest = md5_compute(data.as_bytes());
    format!("{:x}", digest)
}

#[async_trait]
impl MusicSource for KugouSource {
    fn id(&self) -> &str {
        "kugou"
    }
    fn label(&self) -> &str {
        &self.label
    }

    async fn search(&self, keyword: &str, _page: u32, limit: u32) -> Result<Vec<ExternalSong>, String> {
        let client = client()?;
        let limit_str = limit.to_string();
        let resp = client
            .get("http://mobilecdn.kugou.com/api/v3/search/song")
            .query(&[
                ("keyword", keyword),
                ("page", "1"),
                ("pagesize", &limit_str),
                ("showtype", "20"),
            ])
            .send()
            .await
            .map_err(|e| format!("kugou 搜索请求失败: {}", e))?;
        let text = resp
            .text()
            .await
            .map_err(|e| format!("kugou 读取响应失败: {}", e))?;
        let json: Value =
            serde_json::from_str(&text).map_err(|e| format!("kugou 解析JSON失败: {}", e))?;

        let info = json["data"]["info"]
            .as_array()
            .ok_or_else(|| format!("kugou 搜索无结果: {}", truncate_chars(&text, 200)))?;

        let list: Vec<ExternalSong> = info
            .iter()
            .map(|s| {
                let hash = s["hash"].as_str().unwrap_or("").to_string();
                // V3 接口字段是 songname/album_name/singername（带下划线），老接口是 songname/albumname/singername
                let name = s["songname"]
                    .as_str()
                    .or_else(|| s["filename"].as_str())
                    .unwrap_or("")
                    .to_string();
                let artist = s["singername"]
                    .as_str()
                    .or_else(|| s["singer_name"].as_str())
                    .unwrap_or("")
                    .to_string();
                let album = s["albumname"]
                    .as_str()
                    .or_else(|| s["album_name"].as_str())
                    .unwrap_or("")
                    .to_string();
                let duration = s["duration"].as_u64().map(|d| d * 1000).unwrap_or(0);
                // kugou V3 搜索接口不返回封面字段，需通过 album_id 单独调接口获取
                // 这里先留空，前端会 fallback 到默认封面图标
                let pic = s["albumpic"]
                    .as_str()
                    .or_else(|| s["album_img"].as_str())
                    .or_else(|| s["img"].as_str())
                    .or_else(|| s["pic"].as_str())
                    .unwrap_or("")
                    .to_string();
                ExternalSong {
                    id: hash,
                    source: "kugou".into(),
                    source_label: self.label.clone(),
                    name,
                    artist,
                    album,
                    duration,
                    pic_url: pic,
                }
            })
            .collect();

        if list.is_empty() {
            return Err("kugou 搜索无结果".into());
        }
        Ok(list)
    }

    async fn get_url(&self, song_id: &str, _quality: &str) -> Result<String, String> {
        let client = client()?;
        let hash = song_id;
        let key = make_key(hash);
        let filename = format!("{}.mp3", hash);
        let resp = client
            .get("http://trackercdn.kugou.com/i/v2/")
            .query(&[
                ("key", key.as_str()),
                ("hash", hash),
                ("br", "hq"),
                ("appid", "1005"),
                ("pid", "2"),
                ("behavior", "play"),
                ("cmd", "25"),
                ("filename", &filename),
            ])
            .send()
            .await
            .map_err(|e| format!("kugou 获取URL请求失败: {}", e))?;
        let text = resp
            .text()
            .await
            .map_err(|e| format!("kugou 读取URL响应失败: {}", e))?;
        let json: Value =
            serde_json::from_str(&text).map_err(|e| format!("kugou 解析URL JSON失败: {} | body: {}", e, truncate_chars(&text, 300)))?;
        // 响应中 url 是数组（新接口）或字符串（旧接口）
        let url = json["url"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
            .or_else(|| json["url"].as_str())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| format!("kugou URL 为空: status={} | body: {}", json["status"].as_u64().unwrap_or(0), truncate_chars(&text, 300)))?;
        Ok(url.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 真实接口测试，需手动运行: cargo test -- --ignored --nocapture
    #[tokio::test]
    #[ignore]
    async fn test_kugou_search_and_url() {
        // 测试：搜索「唯一 邓紫棋」
        let src = KugouSource::new("酷狗音乐".into());
        let kw = "唯一 邓紫棋";
        let songs = src.search(kw, 1, 10).await.expect("搜索应成功");
        println!("kugou 搜索「{}」到 {} 首", kw, songs.len());
        for s in &songs {
            println!("  - {} - {} (id={})", s.name, s.artist, s.id);
        }
        // 尝试获取前 5 首的 URL
        for s in songs.iter().take(5) {
            match src.get_url(&s.id, "standard").await {
                Ok(url) => println!("  ✅ {} -> {}", s.name, url),
                Err(e) => println!("  ❌ {} -> {}", s.name, e),
            }
        }
    }
}
