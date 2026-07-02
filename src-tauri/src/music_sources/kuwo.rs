//! 酷我音乐音源
//! 搜索: search.kuwo.cn/r.s（返回单引号 JSON，需预处理）
//! URL: mobi.kuwo.cn/mobi.s + kwDES 加密（能绕过 VIP 限制）
//! 移植自 UnblockNeteaseMusic/server 的 kuwo.js

use crate::music_sources::kwdes;
use crate::music_sources::{ExternalSong, MusicSource, build_client, parse_kuwo_abslist, parse_kuwo_json, truncate_chars};
use serde_json::Value;

const UA: &str = "okhttp/3.10.0";

pub struct KuwoSource {
    label: String,
}

impl KuwoSource {
    pub fn new(label: String) -> Self {
        Self { label }
    }
}

#[async_trait::async_trait]
impl MusicSource for KuwoSource {
    fn id(&self) -> &str {
        "kuwo"
    }

    fn label(&self) -> &str {
        &self.label
    }

    async fn search(&self, keyword: &str, _page: u32, limit: u32) -> Result<Vec<ExternalSong>, String> {
        let url = format!(
            "http://search.kuwo.cn/r.s?correct=1&vipver=1&stype=comprehensive&encoding=utf8&rformat=json&mobi=1&show_copyright_off=1&searchapi=6&all={}",
            urlencoding::encode(keyword)
        );

        let client = build_client(UA, 10)?;
        let resp = client
            .get(&url)
            .header("X-Forwarded-For", "1.0.1.114")
            .send()
            .await
            .map_err(|e| format!("kuwo 搜索请求失败: {}", e))?;

        let text = resp
            .text()
            .await
            .map_err(|e| format!("kuwo 搜索读取失败: {}", e))?;

        let json: Value = parse_kuwo_json(&text, "kuwo")?;

        let abs_list = json["content"][1]["musicpage"]["abslist"]
            .as_array()
            .ok_or_else(|| format!("kuwo 搜索结果格式异常, body(前300): {}", truncate_chars(&text, 300)))?;

        Ok(parse_kuwo_abslist(&abs_list[..abs_list.len().min(limit as usize)], "kuwo", &self.label))
    }

    async fn get_url(&self, song_id: &str, _quality: &str) -> Result<String, String> {
        // 用 kwDES 加密的 mobi.s 接口获取播放 URL（能绕过 VIP 限制）
        let query = format!(
            "corp=kuwo&source=kwplayer_ar_5.1.0.0_B_jiakong_vh.apk&p2p=1&type=convert_url2&sig=0&format=mp3&rid={}",
            song_id
        );
        let encrypted = kwdes::encrypt_query(&query);
        let url = format!("http://mobi.kuwo.cn/mobi.s?f=kuwo&q={}", encrypted);

        let client = build_client(UA, 10)?;
        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("kuwo mobi.s 请求失败: {}", e))?;

        let text = resp
            .text()
            .await
            .map_err(|e| format!("kuwo mobi.s 读取失败: {}", e))?;

        // 解析 key=value 格式响应（每行一个），提取 url 和 bitrate
        let mut url_val = String::new();
        let mut bitrate: u64 = 0;
        for line in text.lines() {
            let line = line.trim();
            if let Some(pos) = line.find('=') {
                let key = line[..pos].trim();
                let val = line[pos + 1..].trim();
                match key {
                    "url" => url_val = val.to_string(),
                    "bitrate" => bitrate = val.parse::<u64>().unwrap_or(0),
                    _ => {}
                }
            }
        }

        // 广告音频检测：bitrate < 32 通常是"酷我音乐以为你免费开放播放权限"提示音
        // 正常歌曲 bitrate >= 128
        if bitrate > 0 && bitrate < 32 {
            return Err(format!(
                "kuwo 返回广告音频 (bitrate={}), song_id={}",
                bitrate, song_id
            ));
        }

        if url_val.starts_with("http") {
            Ok(url_val)
        } else {
            Err(format!(
                "kuwo mobi.s 未返回有效 URL: {}",
                truncate_chars(&text, 200)
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 真实接口测试：搜索 + 获取 VIP 歌曲 URL
    /// 手动运行: cargo test -- --ignored --nocapture
    #[tokio::test]
    #[ignore]
    async fn test_kuwo_search_and_url() {
        let src = KuwoSource::new("酷我音乐".into());
        let kw = "唯一 邓紫棋";
        let songs = src.search(kw, 1, 10).await.expect("搜索应成功");
        println!("kuwo 搜索「{}」到 {} 首", kw, songs.len());
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
