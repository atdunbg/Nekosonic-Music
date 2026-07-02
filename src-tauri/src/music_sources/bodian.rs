// 波点音乐（酷我旗下子品牌）
// 参考 SPlayer bodian.ts / UnblockNeteaseMusic bodian.js
//
// 搜索：复用 kuwo 的 search.kuwo.cn/r.s 接口（无需签名）
// URL：bd-api.kuwo.cn/api/play/music/v2/audioUrl，需 sign 签名 + 先调一次广告接口
//
// sign 算法（来自 bodian.js generateSign）：
//   1. url 末尾追加 &timestamp={ms}
//   2. 提取 ? 后的 query 部分
//   3. 去除非字母数字字符后按字符排序
//   4. 拼接 "kuwotest" + sorted_chars + url.pathname
//   5. md5

use async_trait::async_trait;
use md5::compute as md5_compute;
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{ExternalSong, MusicSource, build_client, parse_kuwo_abslist, parse_kuwo_json, truncate_chars};

const UA: &str = "Dart/2.19 (dart:io)";

pub struct BodianSource {
    label: String,
    /// 随机设备 ID（参考 SPlayer，每次实例化生成）
    device_id: String,
}

impl BodianSource {
    pub fn new(label: String) -> Self {
        Self {
            label,
            device_id: gen_device_id(),
        }
    }
}

/// 生成随机设备 ID（参考 SPlayer getRandomDeviceId）
/// 用系统时间纳秒作为种子，每次实例化不同
fn gen_device_id() -> String {
    let ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    // 取纳秒的低 11 位数字，范围 0~100000000000
    let n = (ns % 100_000_000_001) as u64;
    n.to_string()
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

/// 生成 bodian 风格签名 URL
/// 输入：原始 url（含 query），如 http://bd-api.kuwo.cn/api/play/music/v2/audioUrl?&br=320kmp3&musicId=123
/// 输出：追加 &timestamp=...&sign=... 的完整 url
///
/// 算法（参考 UnblockNeteaseMusic bodian.js generateSign）：
///   1. str += &timestamp={ms}
///   2. query = str 中 ? 之后的部分
///   3. filteredChars = query 去除非字母数字后按字符排序
///   4. dataToEncrypt = "kuwotest" + filteredChars + url.pathname
///   5. sign = md5(dataToEncrypt)
fn generate_sign(url: &str) -> String {
    let ts = now_ms();
    let with_ts = format!("{}&timestamp={}", url, ts);
    // 提取 pathname（? 之前的部分去掉 scheme://host）
    let (scheme_host, pathname, query_with_ts) = split_url(&with_ts);
    let _ = scheme_host; // 不需要
    // 提取 ? 之后的 query 部分（含 timestamp）
    let q = query_with_ts;
    // 去除非字母数字，按字符排序
    let mut chars: Vec<char> = q.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    chars.sort();
    let sorted: String = chars.iter().collect();
    let data = format!("kuwotest{}{}", sorted, pathname);
    let hash = md5_compute(data.as_bytes());
    let sign = format!("{:x}", hash);
    format!("{}&sign={}", with_ts, sign)
}

/// 将 URL 拆分为 (scheme://host, pathname, query)
/// 例：http://bd-api.kuwo.cn/api/play/music/v2/audioUrl?&br=320kmp3&musicId=123&timestamp=...
/// → ("http://bd-api.kuwo.cn", "/api/play/music/v2/audioUrl", "&br=320kmp3&musicId=123&timestamp=...")
fn split_url(url: &str) -> (String, String, String) {
    // 分离 query
    let (path_part, query) = url.split_once('?').map(|(p, q)| (p.to_string(), q.to_string()))
        .unwrap_or((url.to_string(), String::new()));
    // path_part 形如 http://bd-api.kuwo.cn/api/play/music/v2/audioUrl
    // 找到 scheme:// 后的第三个 /（即 host 之后的第一个 /）
    let scheme_end = path_part.find("://").map(|i| i + 3).unwrap_or(0);
    let after_scheme = &path_part[scheme_end..];
    let path_start = after_scheme.find('/').map(|i| scheme_end + i).unwrap_or(path_part.len());
    let scheme_host = path_part[..path_start].to_string();
    let pathname = path_part[path_start..].to_string();
    (scheme_host, pathname, query)
}

fn client() -> Result<reqwest::Client, String> {
    build_client(UA, 10)
}

#[async_trait]
impl MusicSource for BodianSource {
    fn id(&self) -> &str {
        "bodian"
    }
    fn label(&self) -> &str {
        &self.label
    }

    async fn search(&self, keyword: &str, _page: u32, limit: u32) -> Result<Vec<ExternalSong>, String> {
        let client = client()?;
        let kw = urlencoding::encode(keyword);
        let url = format!(
            "http://search.kuwo.cn/r.s?&correct=1&vipver=1&stype=comprehensive&encoding=utf8&rformat=json&mobi=1&show_copyright_off=1&searchapi=6&all={}",
            kw
        );
        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("bodian 搜索请求失败: {}", e))?;
        let text = resp
            .text()
            .await
            .map_err(|e| format!("bodian 读取响应失败: {}", e))?;

        // 复用 kuwo 的 JSON 解析（含单引号 JSON 兜底）和 abslist 解析逻辑
        let json: Value = parse_kuwo_json(&text, "bodian")?;

        let abslist = json["content"][1]["musicpage"]["abslist"]
            .as_array()
            .ok_or_else(|| format!("bodian 搜索结果格式异常, body(前300): {}", truncate_chars(&text, 300)))?;

        let list = parse_kuwo_abslist(&abslist[..abslist.len().min(limit as usize)], "bodian", &self.label);
        if list.is_empty() {
            return Err("bodian 搜索无结果".into());
        }
        Ok(list)
    }

    async fn get_url(&self, song_id: &str, quality: &str) -> Result<String, String> {
        let client = client()?;
        // 1. 先调一次广告接口（参考 SPlayer sendAdFreeRequest）
        //    模拟"观看广告"获取免费播放权限
        let _ = client
            .post("http://bd-api.kuwo.cn/api/service/advert/watch?uid=-1&token=&timestamp=1724306124436&sign=15a676d66285117ad714e8c8371691da")
            .header("user-agent", "Dart/2.19 (dart:io)")
            .header("plat", "ar")
            .header("channel", "aliopen")
            .header("devid", &self.device_id)
            .header("ver", "3.9.0")
            .header("host", "bd-api.kuwo.cn")
            .header("qimei36", "1e9970cbcdc20a031dee9f37100017e1840e")
            .header("content-type", "application/json; charset=utf-8")
            .body(r#"{"type":5,"subType":5,"musicId":0,"adToken":""}"#)
            .send()
            .await;

        // 2. 构造带签名的 audioUrl 请求
        let br = if quality.contains("lossless") || quality.contains("hires") {
            "2000kflac"
        } else {
            "320kmp3"
        };
        let base = format!(
            "http://bd-api.kuwo.cn/api/play/music/v2/audioUrl?&br={}&musicId={}",
            br, song_id
        );
        let signed_url = generate_sign(&base);

        let resp = client
            .get(&signed_url)
            .header("user-agent", "Dart/2.19 (dart:io)")
            .header("plat", "ar")
            .header("channel", "aliopen")
            .header("devid", &self.device_id)
            .header("ver", "3.9.0")
            .header("host", "bd-api.kuwo.cn")
            .header("qimei36", "1e9970cbcdc20a031dee9f37100017e1840e")
            .header("X-Forwarded-For", "1.0.1.114")
            .send()
            .await
            .map_err(|e| format!("bodian 获取URL失败: {}", e))?;

        let text = resp
            .text()
            .await
            .map_err(|e| format!("bodian 读取URL失败: {}", e))?;

        // 解析 bd-api 返回的 JSON
        let json: Value = serde_json::from_str(&text)
            .map_err(|e| format!("bodian 解析URL JSON失败: {} | body: {}", e, text))?;

        if json["code"].as_i64() == Some(200) {
            if let Some(url) = json["data"]["audioUrl"].as_str().filter(|s| !s.is_empty()) {
                return Ok(url.to_string());
            }
        }

        Err(format!("bodian 无法获取URL (code={}): {}",
            json["code"].as_i64().unwrap_or(-1),
            json["msg"].as_str().unwrap_or("未知错误")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_url() {
        let url = "http://bd-api.kuwo.cn/api/play/music/v2/audioUrl?&br=320kmp3&musicId=123&timestamp=1700000000000";
        let (host, path, query) = split_url(url);
        assert_eq!(host, "http://bd-api.kuwo.cn");
        assert_eq!(path, "/api/play/music/v2/audioUrl");
        assert_eq!(query, "&br=320kmp3&musicId=123&timestamp=1700000000000");
    }

    #[test]
    fn test_generate_sign_format() {
        let url = "http://bd-api.kuwo.cn/api/play/music/v2/audioUrl?&br=320kmp3&musicId=123";
        let signed = generate_sign(url);
        // 应包含 timestamp 和 sign
        assert!(signed.contains("&timestamp="));
        assert!(signed.contains("&sign="));
        // 原始 URL 应保留
        assert!(signed.starts_with("http://bd-api.kuwo.cn/api/play/music/v2/audioUrl?&br=320kmp3&musicId=123"));
    }

    /// 真实接口测试，需手动运行: cargo test -- --ignored --nocapture
    #[tokio::test]
    #[ignore]
    async fn test_bodian_get_url_real() {
        // 测试：搜索「唯一 邓紫棋」
        let src = BodianSource::new("波点音乐".into());
        let kw = "唯一 邓紫棋";
        let songs = src.search(kw, 1, 10).await.expect("搜索应成功");
        println!("bodian 搜索「{}」到 {} 首", kw, songs.len());
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
