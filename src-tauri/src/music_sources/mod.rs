// 音源模块：可扩展架构（内置 + 自定义 HTTP API）
//
// 设计要点：
// - MusicSource trait 定义统一接口（search + get_url）
// - 内置源：bodian（波点，首选）/ kugou（酷狗，次选）
// - 自定义源：通过 SourceConfig 配置 HTTP API 模板，由 CustomHttpSource 实现
// - 调用方传入 Vec<SourceConfig>，模块根据 kind 路由到对应实现
// - 多源聚合搜索：并发请求所有启用源，按配置顺序合并结果

pub mod bodian;
pub mod custom;
pub mod kugou;
pub mod kwdes;
pub mod kuwo;

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 音源配置（前端持久化、传给后端都用这个结构）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    /// 唯一 ID
    /// - builtin: 'bodian' / 'kugou' / 'kuwo' / 'bilibili'
    /// - custom: 'custom-xxxx'（前端生成）
    pub id: String,
    /// 显示名称
    pub label: String,
    /// 是否启用
    pub enabled: bool,
    /// 类型: "builtin" | "custom"
    pub kind: String,

    // ===== 自定义 HTTP API 源字段（kind == "custom" 时使用）=====
    /// 搜索 URL 模板，支持 {keyword} / {page} / {limit} 占位符
    #[serde(default)]
    pub search_url: Option<String>,
    /// 获取播放 URL 接口，支持 {id} / {quality} 占位符
    #[serde(default)]
    pub url_api: Option<String>,
    /// JSON 路径：搜索结果数组（如 "data.songs"）
    #[serde(default)]
    pub search_path: Option<String>,
    /// JSON 路径：URL 字符串字段（如 "data.url"）
    #[serde(default)]
    pub url_path: Option<String>,
    /// 字段映射（自定义源 JSON 字段名）
    #[serde(default)]
    pub id_field: Option<String>,
    #[serde(default)]
    pub name_field: Option<String>,
    #[serde(default)]
    pub artist_field: Option<String>,
    #[serde(default)]
    pub album_field: Option<String>,
    #[serde(default)]
    pub duration_field: Option<String>,
    #[serde(default)]
    pub pic_field: Option<String>,
}

impl SourceConfig {
    /// 默认内置音源列表（顺序即默认优先级，波点为首选，参考 SPlayer）
    /// kuwo 使用 kwDES + mobi.s 接口，能绕过 VIP 限制播放完整歌曲
    #[allow(dead_code)]
    pub fn builtin_defaults() -> Vec<SourceConfig> {
        vec![
            SourceConfig::builtin("bodian", "波点音乐", true),
            SourceConfig::builtin("kugou", "酷狗音乐", true),
            SourceConfig::builtin("kuwo", "酷我音乐", true),
        ]
    }

    #[allow(dead_code)]
    fn builtin(id: &str, label: &str, enabled: bool) -> SourceConfig {
        SourceConfig {
            id: id.into(),
            label: label.into(),
            enabled,
            kind: "builtin".into(),
            search_url: None,
            url_api: None,
            search_path: None,
            url_path: None,
            id_field: None,
            name_field: None,
            artist_field: None,
            album_field: None,
            duration_field: None,
            pic_field: None,
        }
    }
}

/// 外部歌曲（搜索结果）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalSong {
    /// 源内 ID（字符串，便于通用）
    pub id: String,
    /// 来源 source id（"bodian" / "kugou" / 自定义源 id）
    pub source: String,
    /// 来源显示名（"波点音乐" / ...）
    pub source_label: String,
    pub name: String,
    pub artist: String,
    pub album: String,
    /// 时长（毫秒）
    pub duration: u64,
    pub pic_url: String,
}

/// 音源统一接口
#[async_trait]
pub trait MusicSource: Send + Sync {
    fn id(&self) -> &str;
    fn label(&self) -> &str;
    /// 搜索
    async fn search(&self, keyword: &str, page: u32, limit: u32) -> Result<Vec<ExternalSong>, String>;
    /// 获取播放 URL
    async fn get_url(&self, song_id: &str, quality: &str) -> Result<String, String>;
}

/// 构建 HTTP 客户端（统一 UA / 超时，复用连接池）
pub(crate) fn build_client(ua: &str, timeout_secs: u64) -> Result<Client, String> {
    Client::builder()
        .user_agent(ua)
        .timeout(Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| format!("构建 HTTP 客户端失败: {}", e))
}

/// 安全截断字符串到指定字符数（避免 UTF-8 多字节边界 panic）
pub(crate) fn truncate_chars(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

/// 解析 kuwo r.s 接口返回的 JSON（bodian 与 kuwo 共用）
///
/// 该接口返回格式不稳定：
/// - 有时返回标准双引号 JSON
/// - 有时返回单引号 JSON（key/value 用 ' 包裹）
///
/// 先尝试直接解析标准 JSON；失败再用 `fix_single_quote_json` 修复单引号格式。
/// 不能无脑 fix，因为 `fix_single_quote_json` 遇到标准 JSON 值内部的单引号
/// （如歌名 "Don't Stop"）会误判为定界符，破坏正确的 JSON。
pub(crate) fn parse_kuwo_json(text: &str, source_tag: &str) -> Result<serde_json::Value, String> {
    match serde_json::from_str(text) {
        Ok(j) => Ok(j),
        Err(_) => {
            let fixed = fix_single_quote_json(text);
            serde_json::from_str(&fixed).map_err(|e| {
                format!(
                    "{} 搜索JSON解析失败: {} | body(前200): {}",
                    source_tag,
                    e,
                    truncate_chars(text, 200)
                )
            })
        }
    }
}

/// 从 kuwo r.s 接口的 abslist 数组构造 ExternalSong 列表（bodian 与 kuwo 共用）
pub(crate) fn parse_kuwo_abslist(
    abslist: &[serde_json::Value],
    source_id: &str,
    source_label: &str,
) -> Vec<ExternalSong> {
    abslist
        .iter()
        .map(|item| {
            let musicrid = item["MUSICRID"].as_str().unwrap_or("");
            let id = musicrid
                .strip_prefix("MUSIC_")
                .or_else(|| musicrid.strip_prefix("music_"))
                .unwrap_or(musicrid)
                .to_string();
            let name = item["SONGNAME"].as_str().unwrap_or("").to_string();
            let artist = item["ARTIST"].as_str().unwrap_or("").to_string();
            let album = item["ALBUM"].as_str().unwrap_or("").to_string();
            // DURATION 兼容字符串/数字两种格式
            let duration = item["DURATION"]
                .as_str()
                .and_then(|d| d.parse::<u64>().ok())
                .or_else(|| item["DURATION"].as_u64())
                .map(|d| d * 1000)
                .unwrap_or(0);
            // web_albumpic_short 是相对路径（如 "120/s4s61/37/4021488843.jpg"）
            // 需拼接 kuwo 图片 CDN 前缀；albumpic 字段有时是完整 URL
            let pic_raw = item["web_albumpic_short"]
                .as_str()
                .or_else(|| item["albumpic"].as_str())
                .or_else(|| item["pic"].as_str())
                .unwrap_or("");
            let pic_url = if pic_raw.is_empty() {
                String::new()
            } else if pic_raw.starts_with("http") {
                pic_raw.to_string()
            } else {
                format!("https://img1.kuwo.cn/star/albumcover/{}", pic_raw)
            };
            ExternalSong {
                id,
                source: source_id.to_string(),
                source_label: source_label.to_string(),
                name,
                artist,
                album,
                duration,
                pic_url,
            }
        })
        .collect()
}

/// 修复 kuwo r.s 接口返回的单引号 JSON
/// kuwo 用 `'` 代替 `"` 作为字符串定界符，但值内部可能也含单引号
/// 用状态机遍历：只在「作为定界符的位置」把 `'` 替换成 `"`
///
/// 规则：遇到 `'` 时判断它是否为定界符：
/// - 如果当前在字符串内部（已遇到开定界符 `'`），则看下一个非空白字符
///   是 `:` `}` `,` `]` 之一 → 这是闭定界符；否则是值内部的字符，保留
/// - 如果不在字符串内部，遇到 `'` 一定是开定界符
fn fix_single_quote_json(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(n);
    let mut in_str = false;
    let mut i = 0;

    while i < n {
        let c = chars[i];
        if c == '\'' {
            if !in_str {
                out.push('"');
                in_str = true;
            } else {
                let mut j = i + 1;
                while j < n && (chars[j] == ' ' || chars[j] == '\t' || chars[j] == '\n' || chars[j] == '\r') {
                    j += 1;
                }
                let next = if j < n { Some(chars[j]) } else { None };
                match next {
                    Some(':') | Some('}') | Some(',') | Some(']') | None => {
                        out.push('"');
                        in_str = false;
                    }
                    _ => {
                        out.push('\'');
                    }
                }
            }
        } else if c == '"' && in_str {
            out.push('\\');
            out.push('"');
        } else {
            out.push(c);
        }
        i += 1;
    }

    out
}

/// 根据 SourceConfig 构造对应的音源实例
pub fn get_source(cfg: &SourceConfig) -> Option<Box<dyn MusicSource>> {
    match cfg.kind.as_str() {
        "builtin" => match cfg.id.as_str() {
            "bodian" => Some(Box::new(bodian::BodianSource::new(cfg.label.clone()))),
            "kugou" => Some(Box::new(kugou::KugouSource::new(cfg.label.clone()))),
            "kuwo" => Some(Box::new(kuwo::KuwoSource::new(cfg.label.clone()))),
            _ => None,
        },
        "custom" => Some(Box::new(custom::CustomHttpSource::new(cfg.clone()))),
        _ => None,
    }
}

/// 多源聚合搜索：并发请求所有启用的源，按配置顺序合并结果
pub async fn search_multi(
    keyword: &str,
    sources: &[SourceConfig],
    limit: u32,
) -> Vec<ExternalSong> {
    use futures_util::future::join_all;

    let tasks: Vec<_> = sources
        .iter()
        .filter(|s| s.enabled)
        .filter_map(|cfg| {
            let src = get_source(cfg)?;
            let kw = keyword.to_string();
            Some(async move {
                let label = src.label().to_string();
                let sid = src.id().to_string();
                let _ = label;
                match src.search(&kw, 1, limit).await {
                    Ok(list) => list,
                    Err(e) => {
                        eprintln!("[music_sources] {} 搜索失败: {}", sid, e);
                        Vec::new()
                    }
                }
            })
        })
        .collect();

    let results = join_all(tasks).await;
    let mut all = Vec::new();
    for r in results {
        all.extend(r);
    }
    all
}

/// 网易云 fallback：按 sources 顺序尝试 (search + get_url)
/// 用于 get_song_url 在网易云无 URL 时降级
/// 返回 (url, source_label)
///
/// 匹配策略：
/// 1. 用「歌名 歌手」在源里搜索
/// 2. 只选原版（排除 Live/DJ/翻唱/伴奏/remix 等非原版）
/// 3. 原版 get_url 失败（VIP/广告/付费）则换下一个源继续尝试
/// 4. 所有源都失败就报错，不塞非原版替代品
pub async fn fallback_get_url(
    song_name: &str,
    artist: &str,
    quality: &str,
    sources: &[SourceConfig],
) -> Result<(String, String), String> {
    let target_lower = song_name.to_lowercase();
    // 歌手列表：按 "/" 分割（兼容前端 " / " 和 api.rs 的 "/" 格式）
    let artists: Vec<String> = if artist.is_empty() {
        Vec::new()
    } else {
        artist
            .split('/')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect()
    };
    // 搜索关键词只用第一个歌手（多歌手全拼接会搜不到结果）
    let first_artist = artists.first().cloned().unwrap_or_default();

    // 收集每个源的失败原因，便于排障
    let mut details: Vec<String> = Vec::new();

    for cfg in sources.iter().filter(|s| s.enabled) {
        let src = match get_source(cfg) {
            Some(s) => s,
            None => continue,
        };
        let kw = if first_artist.is_empty() {
            song_name.to_string()
        } else {
            format!("{} {}", song_name, first_artist)
        };
        let results = match src.search(&kw, 1, 20).await {
            Ok(r) => r,
            Err(e) => {
                details.push(format!("{}搜索失败:{}", src.id(), e));
                continue;
            }
        };

        // 版本标签提取：从歌名中识别版本关键词
        // 用于「版本一致性匹配」——目标是原版就只匹配原版，目标是DJ版就只匹配DJ版
        // 避免 VIP 原版歌曲误匹配到同名 DJ 版，也避免 VIP DJ 版误匹配到同名原版
        let extract_version = |name: &str| -> Vec<&'static str> {
            let n = name.to_lowercase();
            let mut tags = Vec::new();
            // 括号通常表示版本标注（如 "Rise (Redux)" "RISE (Live)"）
            // 目标是原版（无括号）时，带括号的搜索结果视为不同版本
            if n.contains('(') || n.contains('（') {
                tags.push("paren");
            }
            if n.contains("live") || n.contains("现场") || n.contains("演唱会") {
                tags.push("live");
            }
            if n.contains("remix") || n.contains("混音") {
                tags.push("remix");
            }
            if n.contains("dj") || n.contains("电音") {
                tags.push("dj");
            }
            if n.contains("伴奏") || n.contains("karaoke") || n.contains("inst") {
                tags.push("accompaniment");
            }
            if n.contains("翻唱") {
                tags.push("cover");
            }
            tags
        };
        // 版本一致性：目标歌名和搜索结果歌名的版本标签必须相同
        // - 目标是原版（无标签）→ 搜索结果也必须是原版（无标签）
        // - 目标是 DJ 版 → 搜索结果也必须是 DJ 版
        let version_match = |result_name: &str| -> bool {
            extract_version(&target_lower) == extract_version(result_name)
        };

        // 歌手匹配：任一网易云歌手名与外部源歌手双向包含匹配
        // 双向是因为网易云可能用中文名"邓紫棋"，外部源用英文名"G.E.M."
        // 去掉歌手字段中的标点（如 "G.E.M." → "gem"），避免点号/空格影响匹配
        let normalize_artist = |a: &str| -> String {
            a.to_lowercase()
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '&' || *c == '、')
                .collect()
        };
        let artist_match = |a: &str| -> bool {
            if artists.is_empty() {
                return false;
            }
            let a_norm = normalize_artist(a);
            if a_norm.is_empty() {
                return false;
            }
            artists.iter().any(|art| {
                let art_norm = normalize_artist(art);
                !art_norm.is_empty() && (a_norm.contains(&art_norm) || art_norm.contains(&a_norm))
            })
        };

        // 匹配优先级（逐级放宽，但每级都要求版本一致）：
        // 1. 歌名完全匹配 + 版本一致 + 歌手匹配（最严格，优先选同名同版本同歌手）
        // 2. 歌名包含目标歌名 + 版本一致 + 歌手匹配（处理歌名带后缀，必须歌手匹配防误匹配）
        // 3. 歌名完全匹配 + 版本一致（最后兜底，不检查歌手——宁可播放同名同版本也不返回失败）
        let best = results.iter().find(|s| {
            s.name.to_lowercase() == target_lower && version_match(&s.name) && artist_match(&s.artist)
        })
        .or_else(|| results.iter().find(|s| {
            let n = s.name.to_lowercase();
            n.contains(&target_lower) && version_match(&s.name) && artist_match(&s.artist)
        }))
        .or_else(|| results.iter().find(|s| {
            s.name.to_lowercase() == target_lower && version_match(&s.name)
        }));

        if let Some(song) = best {
            eprintln!("[music_sources] fallback 匹配: [{}] {} - {} (id={})", src.id(), song.name, song.artist, song.id);
            match src.get_url(&song.id, quality).await {
                Ok(url) if !url.is_empty() => {
                    return Ok((url, src.label().to_string()));
                }
                Ok(_) => {
                    details.push(format!("{}返回空URL", src.id()));
                }
                Err(e) => {
                    details.push(format!("{}取URL失败:{}", src.id(), e));
                }
            }
        } else {
            details.push(format!("{}未匹配到同版本", src.id()));
        }
    }
    Err(format!("所有音源均无可用 URL; 尝试明细: {}", details.join(", ")))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 真实接口测试：验证「唯一 邓紫棋」的 fallback 能找到可播放 URL
    /// 手动运行: cargo test -- --ignored --nocapture
    #[tokio::test]
    #[ignore]
    async fn test_fallback_weiyi_dengziqi() {
        let sources = SourceConfig::builtin_defaults();
        match fallback_get_url("唯一", "邓紫棋", "standard", &sources).await {
            Ok((url, label)) => {
                println!("✅ fallback 成功: [{}] {}", label, url);
                assert!(url.starts_with("http"));
            }
            Err(e) => {
                panic!("❌ fallback 失败: {}", e);
            }
        }
    }
}
