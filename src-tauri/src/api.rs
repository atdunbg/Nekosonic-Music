use ncm_api_rs::{create_client, ApiClient, Query};
use serde::Deserialize;
use serde_json::json;
use tauri::{Manager, State, Emitter};
use tokio::sync::Mutex as AsyncMutex;
use std::sync::Mutex as StdMutex;
use std::sync::atomic::Ordering;

use std::fs;
use std::path::PathBuf;
use std::io::Write;
use std::hash::{Hash, Hasher};
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::tag::Accessor;
use base64::Engine;

/// 统一的 API 调用宏，封装了「获取客户端 → 构建请求 → 发送 → 提取响应体」的完整流程。
///
/// 消除了每个 Tauri 命令中重复的 `client.lock().unwrap().clone()` + `build_query()` + `.map(|r| r.body.to_string())` 样板代码。
///
/// 提供三种调用方式：
///
/// 1. 无额外参数 — 仅使用 cookie 中的默认参数构建请求
///    ```
///    api_call!(state, get_playlist_detail)
///    // 等价于:
///    // let client = state.client.lock().unwrap().clone();
///    // let q = state.build_query();
///    // client.get_playlist_detail(&q).await.map(|r| r.body.to_string()).map_err(|e| e.to_string())
///    ```
///
/// 2. 附加参数 — 在默认参数基础上追加键值对
///    ```
///    api_call!(state, song_url_v1, params: [("id", id), ("level", "standard")])
///    // 等价于:
///    // let q = state.build_query().param("id", id).param("level", "standard");
///    // client.song_url_v1(&q).await...
///    ```
///
/// 3. 预构建查询 — 直接传入已构建好的 Query 对象，跳过 build_query()
///    ```
///    api_call!(state, playlist_track_all, query: my_query)
///    // 等价于:
///    // client.playlist_track_all(&my_query).await...
///    ```
macro_rules! api_call {
    ($state:expr, $method:ident) => {{
        let client = $state.client.lock().await.clone();
        let q = $state.build_query();
        client.$method(&q).await
            .map(|r| r.body.to_string())
            .map_err(|e| e.to_string())
    }};
    ($state:expr, $method:ident, params: [$(($key:expr, $val:expr)),* $(,)?]) => {{
        let client = $state.client.lock().await.clone();
        let mut q = $state.build_query();
        $(q = q.param($key, $val);)*
        client.$method(&q).await
            .map(|r| r.body.to_string())
            .map_err(|e| e.to_string())
    }};
    ($state:expr, $method:ident, query: $q:expr) => {{
        let client = $state.client.lock().await.clone();
        client.$method(&$q).await
            .map(|r| r.body.to_string())
            .map_err(|e| e.to_string())
    }};
}

pub struct ApiController {
    client: AsyncMutex<ApiClient>,
    cookie: StdMutex<Option<String>>,
    cookie_path: PathBuf,
}

/// 将 Cookie 字符串列表转换为 `key=value; key=value` 格式
fn cookies_to_key_values(cookies: &[String]) -> String {
    cookies
        .iter()
        .filter_map(|c| c.split(';').next())
        .map(|s| s.trim().to_string())
        .collect::<Vec<_>>()
        .join("; ")
}

impl ApiController {

    /// 创建新的 API 控制器，从本地文件恢复已保存的 Cookie
    pub fn new(app_data_dir: PathBuf) -> Self {
    let _ = fs::create_dir_all(&app_data_dir);
    let cookie_path = app_data_dir.join("netease_cookies.json");
    let saved_cookie = fs::read_to_string(&cookie_path)
        .map(|s| s.trim().to_string())
        .ok();

    let client = create_client(None);
    ApiController {
        client: AsyncMutex::new(client),
        cookie: StdMutex::new(saved_cookie),
        cookie_path,
    }
}

    /// 构建带当前 Cookie 的 API 查询对象
    fn build_query(&self) -> Query {
    let mut query = Query::new();
    if let Ok(cookie_guard) = self.cookie.lock() {
        if let Some(c) = cookie_guard.as_ref() {
            query = query.cookie(c);
        }
    }
    query
}
    /// 将 Cookie 字符串持久化到本地文件并同步到 API 客户端
    async fn save_cookie(&self, cookie_str: &str) {
        let _ = fs::write(&self.cookie_path, cookie_str);
        let mut client = self.client.lock().await;
        client.set_cookie(cookie_str.to_string());
    }
}

/// 搜索查询参数
#[derive(Deserialize)]
pub struct SearchQuery { pub keyword: String }

/// 多类型搜索查询参数
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudSearchQuery {
    pub keyword: String,
    pub search_type: Option<i64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// 搜索建议查询参数
#[derive(Deserialize)]
pub struct SearchSuggestQuery { pub keyword: String }

/// 手机号登录查询参数
#[derive(Deserialize)]
pub struct LoginQuery { pub phone: String, pub password: String }

/// 二维码登录密钥查询参数
#[derive(Deserialize)]
pub struct QrKeyQuery { pub key: String }

/// 搜索歌曲
#[tauri::command]
pub async fn search_songs(query: SearchQuery, state: State<'_, ApiController>) -> Result<String, String> {
    api_call!(state, cloudsearch, params: [("keywords", &query.keyword), ("type", "1"), ("limit", "30")])
}

/// 多类型搜索（歌曲/歌手/专辑）
#[tauri::command]
pub async fn cloudsearch(query: CloudSearchQuery, state: State<'_, ApiController>) -> Result<String, String> {
    api_call!(state, cloudsearch, params: [
        ("keywords", &query.keyword),
        ("type", &query.search_type.unwrap_or(1).to_string()),
        ("limit", &query.limit.unwrap_or(30).to_string()),
        ("offset", &query.offset.unwrap_or(0).to_string())
    ])
}

/// 搜索建议
#[tauri::command]
pub async fn search_suggest(query: SearchSuggestQuery, state: State<'_, ApiController>) -> Result<String, String> {
    api_call!(state, search_suggest, params: [("keywords", &query.keyword)])
}

/// 获取热搜词列表
#[tauri::command]
pub async fn get_hot_search(state: State<'_, ApiController>) -> Result<String, String> {
    api_call!(state, search_hot_detail)
}

/// 歌单全部曲目查询参数
#[derive(Deserialize)]
pub struct PlaylistTrackAllQuery { pub id: u64, pub limit: Option<i64>, pub offset: Option<i64> }

/// 获取歌单全部歌曲
#[tauri::command]
pub async fn playlist_track_all(query: PlaylistTrackAllQuery, state: State<'_, ApiController>) -> Result<String, String> {
    api_call!(state, playlist_track_all, params: [("id", &query.id.to_string()), ("limit", &query.limit.unwrap_or(1000).to_string()), ("offset", &query.offset.unwrap_or(0).to_string())])
}

/// 歌曲播放地址查询参数
#[derive(Deserialize)]
pub struct SongUrlQuery { pub id: u64, pub level: Option<String>, pub fm_mode: Option<bool> }

/// 获取歌曲播放地址（返回完整 data 对象，包含 url、freeTrialInfo 等）
#[tauri::command]
pub async fn get_song_url(query: SongUrlQuery, state: State<'_, ApiController>) -> Result<String, String> {
    let client = state.client.lock().await.clone();
    let level = query.level.as_deref().unwrap_or("standard");

    let resp = if query.fm_mode.unwrap_or(false) {
        let mut fm_cookie = state.cookie.lock().ok().and_then(|g| g.clone()).unwrap_or_default();
        if !fm_cookie.contains("os=") {
            fm_cookie = format!("{}; os=android; appver=8.10.05", fm_cookie);
        }
        let data = serde_json::json!({
            "ids": format!("[{}]", query.id),
            "level": level,
            "encodeType": "flac",
            "feeProcess": "true"
        });
        let option = ncm_api_rs::request::RequestOption {
            crypto: ncm_api_rs::request::CryptoType::default(),
            cookie: Some(fm_cookie),
            ua: None,
            proxy: None,
            real_ip: None,
            random_cn_ip: false,
            e_r: None,
            domain: None,
            check_token: false,
        };
        client.request(
            "/api/song/enhance/player/url/v1",
            data,
            option,
        ).await.map_err(|e| e.to_string())?
    } else {
        let q = state.build_query()
            .param("id", &query.id.to_string())
            .param("level", level);
        client.song_url_v1(&q).await.map_err(|e| e.to_string())?
    };

    let data = &resp.body["data"][0];
    let url = data["url"].as_str().filter(|s| !s.is_empty());
    if url.is_none() {
        return Err("暂无播放源".into());
    }
    Ok(data.to_string())
}

/// 获取歌词
#[tauri::command]
pub async fn get_lyric(id: u64, state: State<'_, ApiController>) -> Result<String, String> {
    api_call!(state, lyric, params: [("id", &id.to_string())])
}

/// 获取歌单详情
#[tauri::command]
pub async fn get_playlist_detail(id: u64, state: State<'_, ApiController>) -> Result<String, String> {
    api_call!(state, playlist_detail, params: [("id", &id.to_string())])
}

/// 手机号密码登录
#[tauri::command]
pub async fn login(query: LoginQuery, state: State<'_, ApiController>) -> Result<String, String> {
    let client = state.client.lock().await.clone();
    let q = Query::new()
        .param("phone", &query.phone)
        .param("password", &query.password);
    let resp = client.login_cellphone(&q).await.map_err(|e| e.to_string())?;

    if !resp.cookie.is_empty() {
        let cookie_str = cookies_to_key_values(&resp.cookie);
        *state.cookie.lock().map_err(|e| e.to_string())? = Some(cookie_str.clone());
        state.save_cookie(&cookie_str).await;
    }

    Ok(resp.body.to_string())
}

/// 退出登录
#[tauri::command]
pub async fn logout(state: State<'_, ApiController>) -> Result<(), String> {
    *state.cookie.lock().map_err(|e| e.to_string())? = None;
    let _ = fs::remove_file(&state.cookie_path);
    Ok(())
}

/// 获取二维码登录密钥
#[tauri::command]
pub async fn get_qr_key(state: State<'_, ApiController>) -> Result<String, String> {
    let client = state.client.lock().await.clone();
    let q = state.build_query();
    let resp = client.login_qr_key(&q).await.map_err(|e| e.to_string())?;
    resp.body["unikey"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "缺少 unikey".into())
}

/// 生成二维码图片
#[tauri::command]
pub async fn create_qr(
    query: QrKeyQuery,
    state: State<'_, ApiController>,
) -> Result<String, String> {
    let client = state.client.lock().await.clone();
    let q = state
        .build_query()
        .param("key", &query.key)
        .param("qrimg", "true");
    let resp = client.login_qr_create(&q).await.map_err(|e| e.to_string())?;
    let qrurl = resp.body["data"]["qrurl"]
        .as_str()
        .ok_or("未获取到二维码链接")?
        .to_string();
    Ok(qrurl)
}

/// 检查二维码扫码状态
#[tauri::command]
pub async fn check_qr_status(query: QrKeyQuery, state: State<'_, ApiController>) -> Result<String, String> {
    let client = state.client.lock().await.clone();
    let q = state.build_query().param("key", &query.key);
    let resp = client.login_qr_check(&q).await.map_err(|e| e.to_string())?;
    if resp.body["code"].as_u64() == Some(803) && !resp.cookie.is_empty() {
        let cookie_str = cookies_to_key_values(&resp.cookie);
        *state.cookie.lock().map_err(|e| e.to_string())? = Some(cookie_str.clone());
        state.save_cookie(&cookie_str).await;
    }
    Ok(resp.body.to_string())
}

/// 获取当前登录状态
#[tauri::command]
pub async fn get_login_status(state: State<'_, ApiController>) -> Result<String, String> {
    api_call!(state, user_account)
}

/// 获取用户歌单列表
#[tauri::command]
pub async fn user_playlist(uid: u64, state: State<'_, ApiController>) -> Result<String, String> {
    api_call!(state, user_playlist, params: [("uid", &uid.to_string())])
}

/// 获取每日推荐歌曲
#[tauri::command]
pub async fn recommend_songs(state: State<'_, ApiController>) -> Result<String, String> {
    api_call!(state, recommend_songs)
}

/// 获取推荐歌单
#[tauri::command]
pub async fn recommend_resource(state: State<'_, ApiController>) -> Result<String, String> {
    api_call!(state, recommend_resource)
}

/// 个性化推荐歌单（无需登录）
/// 对应 /personalized
#[tauri::command]
pub async fn personalized(limit: Option<u32>, state: State<'_, ApiController>) -> Result<String, String> {
    api_call!(state, personalized, params: [
        ("limit", &limit.unwrap_or(30).to_string())
    ])
}

/// 推荐新歌
/// 对应 /personalized/newsong
#[tauri::command]
pub async fn personalized_newsong(limit: Option<u32>, state: State<'_, ApiController>) -> Result<String, String> {
    api_call!(state, personalized_newsong, params: [
        ("limit", &limit.unwrap_or(10).to_string())
    ])
}

/// 热门歌手
/// 对应 /top/artists
#[tauri::command]
pub async fn top_artists(limit: Option<u32>, offset: Option<u32>, state: State<'_, ApiController>) -> Result<String, String> {
    api_call!(state, top_artists, params: [
        ("limit", &limit.unwrap_or(30).to_string()),
        ("offset", &offset.unwrap_or(0).to_string())
    ])
}

/// 新歌速递
/// 对应 /top/song，type: 全部:0 / 华语:7 / 欧美:96 / 韩国:16 / 日本:8
#[tauri::command]
pub async fn top_song(area_type: Option<u32>, state: State<'_, ApiController>) -> Result<String, String> {
    api_call!(state, top_song, params: [
        ("type", &area_type.unwrap_or(0).to_string())
    ])
}

/// 最新专辑（新碟上架）
/// 对应 /album/newest
#[tauri::command]
pub async fn album_newest(state: State<'_, ApiController>) -> Result<String, String> {
    api_call!(state, album_newest)
}

/// 私人漫游模式查询参数
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonalFmModeQuery {
    pub mode: Option<String>,
    pub sub_mode: Option<String>,
    pub limit: Option<i64>,
}

/// 私人漫游（带模式）
#[tauri::command]
pub async fn personal_fm_mode(query: PersonalFmModeQuery, state: State<'_, ApiController>) -> Result<String, String> {
    api_call!(state, personal_fm_mode, params: [
        ("mode", query.mode.as_deref().unwrap_or("DEFAULT")),
        ("submode", query.sub_mode.as_deref().unwrap_or("")),
        ("limit", &query.limit.unwrap_or(3).to_string())
    ])
}

/// FM 不喜欢查询参数
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FmTrashQuery {
    pub id: u64,
    pub time: Option<i64>,
}

/// FM 不喜欢（减少推荐）
#[tauri::command]
pub async fn fm_trash(query: FmTrashQuery, state: State<'_, ApiController>) -> Result<String, String> {
    api_call!(state, fm_trash, params: [
        ("id", &query.id.to_string()),
        ("time", &query.time.unwrap_or(25).to_string())
    ])
}

/// 获取私人漫游歌曲
#[tauri::command]
pub async fn personal_fm(state: State<'_, ApiController>) -> Result<String, String> {
    api_call!(state, personal_fm)
}

/// 听歌打卡查询参数
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScrobbleQuery {
    pub id: u64,
    pub sourceid: Option<String>,
    pub time: u64,
    pub alg: Option<String>,
    pub source: Option<String>,
    pub bitrate: Option<u64>,
}

/// 听歌打卡
#[tauri::command]
pub async fn scrobble(query: ScrobbleQuery, state: State<'_, ApiController>) -> Result<String, String> {
    let client = state.client.lock().await.clone();
    let cookie = state.cookie.lock().ok().and_then(|g| g.clone()).unwrap_or_default();
    let option = ncm_api_rs::request::RequestOption {
        crypto: ncm_api_rs::request::CryptoType::Weapi,
        cookie: Some(cookie),
        ua: None,
        proxy: None,
        real_ip: None,
        random_cn_ip: false,
        e_r: None,
        domain: None,
        check_token: false,
    };
    let data = json!({
        "logs": serde_json::to_string(&json!([{
            "action": "play",
            "json": {
                "download": 0,
                "end": "playend",
                "id": query.id.to_string(),
                "sourceId": query.sourceid.as_deref().unwrap_or(""),
                "time": query.time as i64,
                "type": "song",
                "wifi": 0,
                "source": query.source.as_deref().unwrap_or("list"),
                "alg": query.alg.as_deref().unwrap_or(""),
                "bitrate": query.bitrate.unwrap_or(0),
                "mainsite": 1,
                "content": ""
            }
        }])).unwrap_or_default()
    });
    let result = client.request("/api/feedback/weblog", data.clone(), option)
        .await
        .map(|r| r.body.to_string())
        .map_err(|e| e.to_string());
    result
}

/// 获取歌曲详情
#[tauri::command]
pub async fn get_song_detail(id: String, state: State<'_, ApiController>) -> Result<String, String> {
    api_call!(state, song_detail, params: [("ids", &id)])
}

/// 用户播放记录查询参数
#[derive(Deserialize)]
pub struct UserRecordQuery { pub uid: u64, pub r#type: String }

/// 喜欢/取消喜欢歌曲查询参数
#[derive(Deserialize)]
pub struct LikeSongQuery { pub id: u64, pub like: String }

/// 获取喜欢的歌曲ID列表
#[tauri::command]
pub async fn likelist(uid: u64, state: State<'_, ApiController>) -> Result<String, String> {
    api_call!(state, likelist, params: [("uid", &uid.to_string())])
}

/// 获取用户播放记录
#[tauri::command]
pub async fn user_record(query: UserRecordQuery, state: State<'_, ApiController>) -> Result<String, String> {
    api_call!(state, user_record, params: [("uid", &query.uid.to_string()), ("type", &query.r#type)])
}

/// 喜欢/取消喜欢歌曲
#[tauri::command]
pub async fn like_song(query: LikeSongQuery, state: State<'_, ApiController>) -> Result<String, String> {
    api_call!(state, like, params: [("id", &query.id.to_string()), ("like", &query.like)])
}

/// 上报最近播放歌曲
#[tauri::command]
pub async fn record_recent_song(limit: u64, state: State<'_, ApiController>) -> Result<String, String> {
    api_call!(state, record_recent_song, params: [("limit", &limit.to_string())])
}

/// 歌单收藏/取消收藏查询参数
#[derive(Deserialize)]
pub struct PlaylistSubscribeQuery { pub id: u64, pub subscribe: Option<bool> }

/// 收藏/取消收藏歌单
#[tauri::command]
pub async fn playlist_subscribe(query: PlaylistSubscribeQuery, state: State<'_, ApiController>) -> Result<String, String> {
    let t = if query.subscribe.unwrap_or(true) { "1" } else { "0" };
    api_call!(state, playlist_subscribe, params: [("id", &query.id.to_string()), ("t", t)])
}

/// 退出应用
#[tauri::command]
pub async fn exit_app(app_handle: tauri::AppHandle) {
    crate::ALLOW_EXIT.store(true, Ordering::SeqCst);
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.close();
    }
}

/// 本地歌曲信息结构体
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalSongInfo {
    pub id: u64,
    pub name: String,
    pub artist: String,
    pub album: String,
    pub duration: u64,
    pub cover: Option<String>,
    pub filename: String,
    pub file_size: u64,
    pub path: String,
    pub local: bool,
}

/// 下载歌曲查询参数
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadSongQuery {
    pub id: u64,
    pub name: String,
    pub artist: String,
    pub album: Option<String>,
    pub duration: Option<u64>,
    pub cover_url: Option<String>,
    pub level: Option<String>,
    pub download_path: Option<String>,
}

/// 下载歌曲到本地，支持进度回调，并保存元数据文件
#[tauri::command]
pub async fn download_song(
    app_handle: tauri::AppHandle,
    query: DownloadSongQuery,
    state: State<'_, ApiController>,
) -> Result<String, String> {
    let level = query.level.as_deref().unwrap_or("standard");

    let q = state.build_query()
        .param("id", &query.id.to_string())
        .param("level", level);
    let client = state.client.lock().await.clone();
    let resp = client.song_url_v1(&q).await.map_err(|e| e.to_string())?;
    let data = &resp.body["data"][0];
    let url = data["url"].as_str().filter(|s| !s.is_empty());
    let is_vip = data.get("freeTrialInfo").is_some_and(|v| !v.is_null());
    if is_vip {
        return Err("VIP歌曲无法下载".into());
    }
    if url.is_none() {
        return Err("暂无下载源，可能需要 VIP 权限".into());
    }
    let url = url.unwrap();
    let ext = if url.contains(".flac") { "flac" } else { "mp3" };
    drop(client);

    let download_dir = resolve_download_dir(&app_handle, query.download_path.as_deref());
    let _ = fs::create_dir_all(&download_dir);

    let safe_name = sanitize_filename(&query.name);
    let safe_artist = sanitize_filename(&query.artist);
    let filename = format!("{} - {}.{}", safe_artist, safe_name, ext);
    let filepath = download_dir.join(&filename);

    if filepath.exists() {
        return Err("文件已存在".into());
    }

    let resp = reqwest::get(url).await.map_err(|e| format!("下载失败: {}", e))?;
    let total_size = resp.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    let temp_path = filepath.with_extension(format!("{}.tmp", ext));
    let mut file = fs::File::create(&temp_path).map_err(|e| format!("创建文件失败: {}", e))?;

    let mut stream = resp.bytes_stream();
    use futures_util::StreamExt;
    let mut chunk_count: u64 = 0;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("读取失败: {}", e))?;
        file.write_all(&chunk).map_err(|e| format!("写入失败: {}", e))?;
        downloaded += chunk.len() as u64;
        chunk_count += 1;
        if chunk_count % 8 == 0 || downloaded == total_size {
            let progress = if total_size > 0 {
                (downloaded as f64 / total_size as f64) * 100.0
            } else {
                0.0
            };
            let _ = app_handle.emit("download-progress", json!({
                "id": query.id,
                "progress": progress,
                "name": query.name,
            }));
        }
    }
    drop(file);

    fs::rename(&temp_path, &filepath).map_err(|e| format!("重命名失败: {}", e))?;

    let meta = json!({
        "id": query.id,
        "name": query.name,
        "artist": query.artist,
        "album": query.album,
        "duration": query.duration,
        "coverUrl": query.cover_url,
        "filename": filename,
    });
    let meta_path = download_dir.join(format!("{}.json", query.id));
    let mut meta_file = fs::File::create(&meta_path).map_err(|e| format!("创建元数据失败: {}", e))?;
    meta_file.write_all(serde_json::to_string_pretty(&meta).unwrap().as_bytes())
        .map_err(|e| format!("写入元数据失败: {}", e))?;

    let _ = app_handle.emit("download-progress", json!({
        "id": query.id,
        "progress": 100.0,
        "name": query.name,
    }));

    Ok(filename)
}

/// 扫描指定目录下的音频文件，优先使用元数据文件补充信息
/// `downloaded_only` 为 true 时，只返回有对应 .json 元数据的文件（即通过应用下载的）
fn scan_dir_for_songs(dir: &PathBuf, downloaded_only: bool) -> Result<Vec<LocalSongInfo>, String> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let audio_exts = ["mp3", "flac", "wav", "ogg", "aac", "m4a", "wma", "opus"];

    let mut meta_map: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();
    let entries = fs::read_dir(dir).map_err(|e| format!("读取目录失败: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "json") {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(filename) = meta["filename"].as_str() {
                        meta_map.insert(filename.to_string(), meta);
                    }
                }
            }
        }
    }

    let mut songs: Vec<LocalSongInfo> = Vec::new();
    let entries = fs::read_dir(dir).map_err(|e| format!("读取目录失败: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !audio_exts.contains(&ext.to_lowercase().as_str()) {
            continue;
        }

        let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        let file_size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

        // 下载音乐模式：只显示有 .json 元数据的文件
        if downloaded_only && !meta_map.contains_key(&filename) {
            continue;
        }

        let (title, artist, album, duration_ms, cover_b64) = read_audio_metadata(&path);

        if let Some(meta) = meta_map.get(&filename) {
            let meta_title = meta["name"].as_str().unwrap_or("");
            let meta_artist = meta["artist"].as_str().unwrap_or("");
            let meta_album = meta["album"].as_str().unwrap_or("");
            let meta_duration = meta["duration"].as_u64().unwrap_or(0);
            let meta_cover_url = meta["coverUrl"].as_str().unwrap_or("");

            let final_title = if title.is_empty() { meta_title.to_string() } else { title };
            let final_artist = if artist.is_empty() { meta_artist.to_string() } else { artist };
            let final_album = if album.is_empty() { meta_album.to_string() } else { album };
            let final_duration = if duration_ms == 0 { meta_duration } else { duration_ms };
            let final_cover = cover_b64.or_else(|| {
                if meta_cover_url.is_empty() { None } else { Some(meta_cover_url.to_string()) }
            });

            songs.push(LocalSongInfo {
                id: meta["id"].as_u64().unwrap_or(0),
                name: final_title,
                artist: final_artist,
                album: final_album,
                duration: final_duration,
                cover: final_cover,
                filename,
                file_size,
                path: path.to_string_lossy().to_string(),
                local: true,
            });
        } else {
            let stem = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
            let (parsed_artist, parsed_name) = parse_filename(&stem);
            let final_title = if title.is_empty() { parsed_name } else { title };
            let final_artist = if artist.is_empty() { parsed_artist } else { artist };

            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            filename.hash(&mut hasher);
            let hash_id = hasher.finish();

            songs.push(LocalSongInfo {
                id: hash_id,
                name: final_title,
                artist: final_artist,
                album,
                duration: duration_ms,
                cover: cover_b64,
                filename,
                file_size,
                path: path.to_string_lossy().to_string(),
                local: true,
            });
        }
    }

    Ok(songs)
}

/// 列出本地已下载的歌曲（下载目录）
#[tauri::command]
pub async fn list_local_songs(app_handle: tauri::AppHandle, download_path: Option<String>) -> Result<Vec<LocalSongInfo>, String> {
    let download_dir = resolve_download_dir(&app_handle, download_path.as_deref());
    tokio::task::spawn_blocking(move || {
        scan_dir_for_songs(&download_dir, true) // 只显示下载的歌曲
    }).await.map_err(|e| format!("扫描任务失败: {}", e))?
}

/// 扫描多个本地文件夹中的音频文件
#[tauri::command]
pub async fn scan_local_folders(paths: Vec<String>) -> Result<Vec<LocalSongInfo>, String> {
    tokio::task::spawn_blocking(move || {
        let mut all_songs: Vec<LocalSongInfo> = Vec::new();
        let mut seen_paths: std::collections::HashSet<String> = std::collections::HashSet::new();

        for p in &paths {
            let dir = PathBuf::from(p);
            let songs = scan_dir_for_songs(&dir, false)?; // 本地音乐：显示所有音频
            for song in songs {
                if seen_paths.insert(song.path.clone()) {
                    all_songs.push(song);
                }
            }
        }

        Ok(all_songs)
    }).await.map_err(|e| format!("扫描任务失败: {}", e))?
}

/// 读取音频文件的元数据（标题、艺术家、专辑、时长、封面）
fn read_audio_metadata(path: &PathBuf) -> (String, String, String, u64, Option<String>) {
    match lofty::read_from_path(path) {
        Ok(tagged_file) => {
            let properties = tagged_file.properties();
            let duration_ms = properties.duration().as_millis() as u64;

            let tag = tagged_file.primary_tag();
            let (title, artist, album) = if let Some(t) = tag {
                let title = t.title().map(|s| s.to_string()).unwrap_or_default();
                let artist = t.artist().map(|s| s.to_string()).unwrap_or_default();
                let album = t.album().map(|s| s.to_string()).unwrap_or_default();
                (title, artist, album)
            } else {
                (String::new(), String::new(), String::new())
            };

            let cover_b64 = if let Some(t) = tag {
                if let Some(pic) = t.pictures().first() {
                    let data = pic.data();
                    let mime = pic.mime_type().map(|m| m.to_string()).unwrap_or_else(|| "image/jpeg".to_string());
                    let b64 = base64::engine::general_purpose::STANDARD.encode(data);
                    Some(format!("data:{};base64,{}", mime, b64))
                } else {
                    None
                }
            } else {
                None
            };

            (title, artist, album, duration_ms, cover_b64)
        }
        Err(e) => {
            eprintln!("[api] 读取音频元数据失败 {}: {}", path.display(), e);
            (String::new(), String::new(), String::new(), 0, None)
        }
    }
}

/// 解析文件名，提取艺术家和歌曲名称（支持 "艺术家 - 歌名" 格式）
fn parse_filename(stem: &str) -> (String, String) {
    if let Some(pos) = stem.find(" - ") {
        let artist = &stem[..pos];
        let name = &stem[pos + 3..];
        (artist.trim().to_string(), name.trim().to_string())
    } else if let Some(pos) = stem.find('-') {
        let artist = &stem[..pos];
        let name = &stem[pos + 1..];
        (artist.trim().to_string(), name.trim().to_string())
    } else {
        ("".to_string(), stem.trim().to_string())
    }
}

/// 删除本地歌曲查询参数
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteLocalSongQuery {
    pub id: u64,
    pub filename: String,
    pub download_path: Option<String>,
}

/// 删除本地已下载的歌曲文件及其元数据
#[tauri::command]
pub async fn delete_local_song(
    app_handle: tauri::AppHandle,
    query: DeleteLocalSongQuery,
) -> Result<(), String> {
    let download_dir = resolve_download_dir(&app_handle, query.download_path.as_deref());
    tokio::task::spawn_blocking(move || {
        let file_path = download_dir.join(&query.filename);
        let meta_path = download_dir.join(format!("{}.json", query.id));

        if file_path.exists() {
            fs::remove_file(&file_path).map_err(|e| format!("删除文件失败: {}", e))?;
        }
        if meta_path.exists() {
            fs::remove_file(&meta_path).map_err(|e| format!("删除元数据失败: {}", e))?;
        }
        Ok(())
    }).await.map_err(|e| format!("删除任务失败: {}", e))?
}

/// 检查指定歌曲是否已下载到本地
#[tauri::command]
pub async fn check_local_song(app_handle: tauri::AppHandle, id: u64, download_path: Option<String>) -> Result<bool, String> {
    let download_dir = resolve_download_dir(&app_handle, download_path.as_deref());
    tokio::task::spawn_blocking(move || {
        let meta_path = download_dir.join(format!("{}.json", id));
        Ok(meta_path.exists())
    }).await.map_err(|e| format!("检查任务失败: {}", e))?
}

/// 解析下载目录，优先使用自定义路径，否则使用默认目录
fn resolve_download_dir(app_handle: &tauri::AppHandle, custom_path: Option<&str>) -> PathBuf {
    if let Some(path) = custom_path {
        if !path.is_empty() {
            return PathBuf::from(path);
        }
    }
    get_default_download_dir(app_handle)
}

/// 获取默认下载目录，优先使用应用数据目录下的 downloads 子目录
fn get_default_download_dir(app_handle: &tauri::AppHandle) -> PathBuf {
    if let Ok(dir) = app_handle.path().app_data_dir() {
        let download_dir = dir.join("downloads");
        return download_dir;
    }
    let music_dir = dirs::audio_dir().unwrap_or_else(|| {
        std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
    });
    music_dir.join("Nekosonic")
}

/// 获取默认下载路径字符串，供前端使用
#[tauri::command]
pub async fn get_default_download_path(app_handle: tauri::AppHandle) -> String {
    get_default_download_dir(&app_handle).to_string_lossy().to_string()
}

/// 清理文件名中的非法字符，将 `/ \ : * ? " < > |` 替换为下划线
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c == '/' || c == '\\' || c == ':' || c == '*' || c == '?'
                || c == '"' || c == '<' || c == '>' || c == '|'
            {
                '_'
            } else {
                c
            }
        })
        .collect::<String>()
        .trim()
        .to_string()
}

/// 读取本地图片文件并转为 base64 data URL，供前端壁纸等场景使用
#[tauri::command]
pub async fn read_image_as_data_url(path: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let file_path = PathBuf::from(&path);
        if !file_path.exists() {
            return Err(format!("文件不存在: {}", path));
        }
        let bytes = fs::read(&file_path).map_err(|e| format!("读取文件失败: {}", e))?;
        let mime = match file_path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase().as_str() {
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "webp" => "image/webp",
            "gif" => "image/gif",
            "bmp" => "image/bmp",
            "svg" => "image/svg+xml",
            _ => "image/jpeg", // 默认 jpeg
        };
        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
        Ok(format!("data:{};base64,{}", mime, b64))
    }).await.map_err(|e| format!("任务失败: {}", e))?
}

/// 在系统文件管理器中显示指定文件（选中）
#[tauri::command]
pub fn show_item_in_folder(path: String) -> Result<(), String> {
    let p = PathBuf::from(&path);
    if !p.exists() {
        return Err(format!("文件不存在: {}", path));
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .args(["/select,", &p.to_string_lossy()])
            .spawn()
            .map_err(|e| format!("打开文件夹失败: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-R", &p.to_string_lossy()])
            .spawn()
            .map_err(|e| format!("打开文件夹失败: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        let uri = format!("file://{}", p.to_string_lossy());
        // 优先使用 freedesktop DBus FileManager1 接口（支持选中文件，Nautilus/Dolphin 等均实现）
        let dbus_ok = std::process::Command::new("dbus-send")
            .args([
                "--session",
                "--print-reply",
                "--dest=org.freedesktop.FileManager1",
                "/org/freedesktop/FileManager1",
                "org.freedesktop.FileManager1.ShowItems",
                &format!("array:string:{}", uri),
                "string:",
            ])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        // fallback：仅打开父目录（无法选中文件）
        if !dbus_ok {
            let parent = p.parent().unwrap_or(&p).to_string_lossy().to_string();
            std::process::Command::new("xdg-open")
                .arg(&parent)
                .spawn()
                .map_err(|e| format!("打开文件夹失败: {}", e))?;
        }
    }
    Ok(())
}

/// 获取歌手详情
#[tauri::command]
pub async fn artist_detail(id: u64, state: State<'_, ApiController>) -> Result<String, String> {
    api_call!(state, artist_detail, params: [("id", &id.to_string())])
}

/// 获取歌手歌曲列表
#[tauri::command]
pub async fn artist_songs(query: ArtistSongsQuery, state: State<'_, ApiController>) -> Result<String, String> {
    let mut q = state.build_query().param("id", &query.id.to_string());
    if let Some(ref order) = query.order {
        q = q.param("order", order);
    }
    if let Some(limit) = query.limit {
        q = q.param("limit", &limit.to_string());
    }
    if let Some(offset) = query.offset {
        q = q.param("offset", &offset.to_string());
    }
    api_call!(state, artist_songs, query: q)
}

/// 歌手歌曲查询参数
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistSongsQuery {
    pub id: u64,
    pub order: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// 获取歌手专辑列表
#[tauri::command]
pub async fn artist_album(id: u64, limit: Option<u32>, offset: Option<u32>, state: State<'_, ApiController>) -> Result<String, String> {
    let mut q = state.build_query().param("id", &id.to_string());
    if let Some(limit) = limit {
        q = q.param("limit", &limit.to_string());
    }
    if let Some(offset) = offset {
        q = q.param("offset", &offset.to_string());
    }
    api_call!(state, artist_album, query: q)
}

/// 获取歌手简介
#[tauri::command]
pub async fn artist_desc(id: u64, state: State<'_, ApiController>) -> Result<String, String> {
    api_call!(state, artist_desc, params: [("id", &id.to_string())])
}

/// 关注/取消关注歌手
#[derive(Deserialize)]
pub struct ArtistSubQuery { pub id: u64, pub sub: Option<bool> }

#[tauri::command]
pub async fn artist_sub(query: ArtistSubQuery, state: State<'_, ApiController>) -> Result<String, String> {
    let t = if query.sub.unwrap_or(true) { "1" } else { "0" };
    api_call!(state, artist_sub, params: [("id", &query.id.to_string()), ("t", t)])
}

/// 获取已关注的歌手列表
#[derive(Deserialize)]
pub struct ArtistSublistQuery { pub limit: Option<u32>, pub offset: Option<u32> }

#[tauri::command]
pub async fn artist_sublist(query: ArtistSublistQuery, state: State<'_, ApiController>) -> Result<String, String> {
    let q = state.build_query()
        .param("limit", &query.limit.unwrap_or(100).to_string())
        .param("offset", &query.offset.unwrap_or(0).to_string());
    api_call!(state, artist_sublist, query: &q)
}

/// 获取专辑详情
#[tauri::command]
pub async fn album_detail(id: u64, state: State<'_, ApiController>) -> Result<String, String> {
    api_call!(state, album, params: [("id", &id.to_string())])
}

/// 获取最新评论
#[tauri::command]
pub async fn comment_new(query: CommentNewQuery, state: State<'_, ApiController>) -> Result<String, String> {
    let mut q = state.build_query()
        .param("type", &query.r#type.to_string())
        .param("id", &query.id.to_string());
    if let Some(sort_type) = query.sort_type {
        q = q.param("sortType", &sort_type.to_string());
    }
    if let Some(page_no) = query.page_no {
        q = q.param("pageNo", &page_no.to_string());
    }
    if let Some(page_size) = query.page_size {
        q = q.param("pageSize", &page_size.to_string());
    }
    if let Some(cursor) = query.cursor {
        q = q.param("cursor", &cursor.to_string());
    }
    api_call!(state, comment_new, query: q)
}

/// 最新评论查询参数
#[derive(Deserialize)]
pub struct CommentNewQuery {
    pub r#type: u8,
    pub id: u64,
    #[serde(rename = "sortType")]
    pub sort_type: Option<u8>,
    #[serde(rename = "pageNo")]
    pub page_no: Option<u32>,
    #[serde(rename = "pageSize")]
    pub page_size: Option<u32>,
    pub cursor: Option<u64>,
}

/// 获取热门评论
#[tauri::command]
pub async fn comment_hot(query: CommentHotQuery, state: State<'_, ApiController>) -> Result<String, String> {
    let mut q = state.build_query()
        .param("type", &query.r#type.to_string())
        .param("id", &query.id.to_string());
    if let Some(limit) = query.limit {
        q = q.param("limit", &limit.to_string());
    }
    if let Some(offset) = query.offset {
        q = q.param("offset", &offset.to_string());
    }
    if let Some(before) = query.before {
        q = q.param("before", &before.to_string());
    }
    api_call!(state, comment_hot, query: q)
}

/// 热门评论查询参数
#[derive(Deserialize)]
pub struct CommentHotQuery {
    pub r#type: u8,
    pub id: u64,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub before: Option<u64>,
}

/// 获取评论楼层（子评论）
#[tauri::command]
pub async fn comment_floor(query: CommentFloorQuery, state: State<'_, ApiController>) -> Result<String, String> {
    let mut q = state.build_query()
        .param("parentCommentId", &query.parent_comment_id.to_string())
        .param("type", &query.r#type.to_string())
        .param("id", &query.id.to_string());
    if let Some(limit) = query.limit {
        q = q.param("limit", &limit.to_string());
    }
    if let Some(time) = query.time {
        q = q.param("time", &time.to_string());
    }
    api_call!(state, comment_floor, query: q)
}

/// 评论楼层查询参数
#[derive(Deserialize)]
pub struct CommentFloorQuery {
    #[serde(rename = "parentCommentId")]
    pub parent_comment_id: u64,
    pub r#type: u8,
    pub id: u64,
    pub limit: Option<u32>,
    pub time: Option<u64>,
}

/// 点赞/取消点赞评论
#[tauri::command]
pub async fn comment_like(query: CommentLikeQuery, state: State<'_, ApiController>) -> Result<String, String> {
    api_call!(state, comment_like, params: [("t", &query.t.to_string()), ("type", &query.r#type.to_string()), ("id", &query.id.to_string()), ("cid", &query.cid.to_string())])
}

/// 评论点赞查询参数
#[derive(Deserialize)]
pub struct CommentLikeQuery {
    pub t: u8,
    pub r#type: u8,
    pub id: u64,
    pub cid: u64,
}

// ==================== 云盘 ====================

/// 获取云盘列表
#[tauri::command]
pub async fn user_cloud(limit: Option<u32>, offset: Option<u32>, state: State<'_, ApiController>) -> Result<String, String> {
    api_call!(state, user_cloud, params: [("limit", &limit.unwrap_or(30).to_string()), ("offset", &offset.unwrap_or(0).to_string())])
}

/// 获取云盘歌曲详情
#[tauri::command]
pub async fn user_cloud_detail(id: String, state: State<'_, ApiController>) -> Result<String, String> {
    api_call!(state, user_cloud_detail, params: [("id", &id)])
}

/// 删除云盘歌曲
#[tauri::command]
pub async fn user_cloud_del(id: u64, state: State<'_, ApiController>) -> Result<String, String> {
    api_call!(state, user_cloud_del, params: [("id", &id.to_string())])
}

/// 查询 NOS LBS 获取上传节点域名
///
/// 通过 `http://wannos.127.net/lbs` 查询指定 bucket 的上传节点，
/// 从返回的 `nosup-<region><n>.127.net` 中提取区域标识，
/// 构造 multipart upload 所需的 `<bucket>.nos-<region>.163yun.com` 域名。
async fn query_nos_upload_host(bucket: &str) -> Result<String, String> {
    let lbs_url = format!("http://wannos.127.net/lbs?version=1.0&bucketname={}", bucket);
    let http = reqwest::Client::new();
    let resp = http.get(&lbs_url).send().await
        .map_err(|e| format!("LBS查询请求失败: {}", e))?;

    let lbs_data: serde_json::Value = resp.json().await
        .map_err(|e| format!("LBS响应解析失败: {}", e))?;

    let nosup_url = lbs_data["upload"][0].as_str()
        .ok_or_else(|| format!("LBS响应缺少upload字段: {}", lbs_data))?;

    // 从 "nosup-jd1.127.net" 中提取区域 "jd"
    let region = extract_nos_region(nosup_url)?;
    Ok(format!("https://{}.nos-{}.163yun.com", bucket, region))
}

/// 从 nosup 上传节点 URL 中提取 NOS 区域标识
///
/// 例如: `http://nosup-jd1.127.net` → `jd`
///       `http://nosup-hz1.127.net` → `hz`
fn extract_nos_region(nosup_url: &str) -> Result<String, String> {
    let start = nosup_url.find("nosup-")
        .ok_or_else(|| format!("无法从LBS响应中解析区域: {}", nosup_url))?;
    let after = &nosup_url[start + 6..];
    let dot = after.find('.')
        .ok_or_else(|| format!("无法从LBS响应中解析区域: {}", nosup_url))?;
    let region_with_num = &after[..dot];
    // 去掉末尾数字: "jd1" → "jd"
    let region: String = region_with_num
        .chars()
        .take_while(|c| !c.is_ascii_digit())
        .collect();
    if region.is_empty() {
        return Err(format!("区域标识为空: {}", nosup_url));
    }
    Ok(region)
}

/// 云盘上传：完整流程（检查 → [获取Token → LBS查询 → NOS上传] → 提交信息 → 发布）
///
/// 关键发现：upload check 返回的 songId 是十六进制字符串（如 MD5 摘要），不是数字 ID。
/// needUpload=false 表示文件已在 NOS 上，无需重复上传，直接走 info+pub 即可。
/// 参考 ydq/netease-cloud-disk-music-upload 实现。
#[tauri::command]
pub async fn cloud_upload(file_path: String, app_handle: tauri::AppHandle, state: State<'_, ApiController>) -> Result<String, String> {
    let path = PathBuf::from(&file_path);
    if !path.exists() {
        return Err("文件不存在".to_string());
    }

    let file_bytes = fs::read(&path).map_err(|e| format!("读取文件失败: {}", e))?;
    let file_size = file_bytes.len() as i64;
    let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();

    // 计算 MD5
    let md5_hex = format!("{:x}", md5::compute(&file_bytes));

    // 读取音频元数据
    let (song_name, artist, album, _duration, _cover) = read_audio_metadata(&path);
    let bitrate = {
        match lofty::read_from_path(&path) {
            Ok(tf) => {
                let br = tf.properties().audio_bitrate().unwrap_or(0) * 1000;
                if br > 0 { br.to_string() } else { "999000".to_string() }
            }
            Err(_) => "999000".to_string()
        }
    };

    // 文件扩展名
    let ext = if filename.contains('.') {
        filename.rsplit('.').next().unwrap_or("mp3").to_string()
    } else {
        "mp3".to_string()
    };

    let client = state.client.lock().await.clone();

    // Step 1: 上传检查
    let check_q = state.build_query()
        .param("bitrate", &bitrate)
        .param("length", &file_size.to_string())
        .param("md5", &md5_hex)
        .param("ext", &ext)
        .param("songId", "0");

    let check_res = client.cloud_upload_check(&check_q).await
        .map_err(|e| format!("上传检查失败: {}", e))?;
    let check_data = &check_res.body;

    // songId 是十六进制字符串（非数字），需要保持字符串传递
    let song_id = check_data["songId"].as_str().unwrap_or("0").to_string();
    let need_upload = check_data["needUpload"].as_bool().unwrap_or(true);

    let mut resource_id = String::new();

    // Step 2-4: 仅在 needUpload=true 时执行 NOS 上传
    if need_upload {
        // Step 2: 获取 NOS 上传 Token
        let token_q = state.build_query()
            .param("filename", &filename)
            .param("md5", &md5_hex);

        let token_res = client.cloud_upload_token_alloc(&token_q).await
            .map_err(|e| format!("获取上传Token失败: {}", e))?;
        let token_data = &token_res.body;

        resource_id = token_data["result"]["resourceId"].as_str().unwrap_or("").to_string();
        let object_key_raw = token_data["result"]["objectKey"].as_str().unwrap_or("").to_string();
        let object_key = object_key_raw.replace('/', "%2F");
        let token_str = token_data["result"]["token"].as_str().unwrap_or("").to_string();

        if token_str.is_empty() {
            return Err(format!("获取上传Token为空, 响应: {}", token_data));
        }

        // Step 3: 查询 LBS 获取正确的 NOS 上传节点
        let bucket = "jd-musicrep-privatecloud-audio-public";
        let nos_host = query_nos_upload_host(bucket).await
            .unwrap_or_else(|_| format!("https://{}.nos-jd.163yun.com", bucket));

        let content_type = match ext.as_str() {
            "flac" => "audio/flac",
            "wav" => "audio/wav",
            "ogg" => "audio/ogg",
            "aac" | "m4a" => "audio/aac",
            _ => "audio/mpeg",
        };

        // Step 4: 上传文件到 NOS（multipart upload）
        let http_client = reqwest::Client::new();

        // 4a: 初始化 multipart upload
        let init_url = format!("{}/{}?uploads", nos_host, object_key);
        let init_res = http_client.post(&init_url)
            .header("x-nos-token", &token_str)
            .header("X-Nos-Meta-Content-Type", content_type)
            .send()
            .await
            .map_err(|e| format!("初始化NOS上传失败: {}", e))?;

        let init_status = init_res.status();
        let init_xml = init_res.text().await.map_err(|e| format!("读取NOS响应失败: {}", e))?;

        if !init_status.is_success() {
            return Err(format!("初始化NOS上传失败: HTTP {} 响应: {}", init_status, init_xml));
        }

        // 解析 UploadId
        let upload_id = init_xml
            .split("<UploadId>")
            .nth(1)
            .and_then(|s| s.split("</UploadId>").next())
            .unwrap_or_default()
            .to_string();

        if upload_id.is_empty() {
            return Err(format!("获取UploadId失败, NOS响应: {}", init_xml));
        }

        // 4b: 分块上传（每块 10MB）
        let block_size = 10 * 1024 * 1024;
        let file_size_usize = file_bytes.len();
        let mut offset = 0usize;
        let mut block_index = 1u32;
        let mut etags = Vec::new();
        let total_blocks = ((file_size_usize + block_size - 1) / block_size).max(1);

        while offset < file_size_usize {
            let end = (offset + block_size).min(file_size_usize);
            let chunk = file_bytes[offset..end].to_vec();

            let part_url = format!(
                "{}/{}?partNumber={}&uploadId={}",
                nos_host, object_key, block_index, upload_id
            );

            let part_res = http_client.put(&part_url)
                .header("x-nos-token", &token_str)
                .header("Content-Type", content_type)
                .body(chunk)
                .send()
                .await
                .map_err(|e| format!("上传分块{}失败: {}", block_index, e))?;

            if let Some(etag) = part_res.headers().get("etag") {
                etags.push(etag.to_str().unwrap_or_default().to_string());
            }

            // 发送上传进度
            let progress = (block_index as f64 / total_blocks as f64 * 100.0).min(100.0);
            let _ = app_handle.emit("cloud-upload-progress", json!({
                "filename": filename,
                "progress": progress,
                "uploaded": end,
                "total": file_size_usize,
            }));

            offset = end;
            block_index += 1;
        }

        // 4c: 完成 multipart upload
        let mut complete_xml = String::from("<CompleteMultipartUpload>");
        for (i, etag) in etags.iter().enumerate() {
            complete_xml.push_str(&format!(
                "<Part><PartNumber>{}</PartNumber><ETag>{}</ETag></Part>",
                i + 1, etag
            ));
        }
        complete_xml.push_str("</CompleteMultipartUpload>");

        let complete_url = format!("{}/{}?uploadId={}", nos_host, object_key, upload_id);
        let complete_res = http_client.post(&complete_url)
            .header("Content-Type", "text/plain;charset=UTF-8")
            .header("X-Nos-Meta-Content-Type", content_type)
            .header("x-nos-token", &token_str)
            .body(complete_xml)
            .send()
            .await
            .map_err(|e| format!("完成NOS上传失败: {}", e))?;

        if !complete_res.status().is_success() {
            let status = complete_res.status();
            let body = complete_res.text().await.unwrap_or_default();
            return Err(format!("完成NOS上传失败: HTTP {} 响应: {}", status, body));
        }
    }

    // Step 5: 提交歌曲信息（songId 使用 check 返回的字符串值）
    let info_q = state.build_query()
        .param("md5", &md5_hex)
        .param("songId", &song_id)
        .param("filename", &filename)
        .param("song", &song_name)
        .param("album", &album)
        .param("artist", &artist)
        .param("bitrate", &bitrate)
        .param("resourceId", &resource_id);

    let info_res = client.cloud_upload_info(&info_q).await
        .map_err(|e| format!("提交歌曲信息失败: {}", e))?;
    let info_data = &info_res.body;

    // info 可能返回新的 songId，优先使用
    let final_song_id = info_data["songId"].as_str()
        .filter(|s| s != &"0" && !s.is_empty())
        .unwrap_or(&song_id)
        .to_string();

    // Step 6: 发布
    let pub_q = state.build_query().param("songId", &final_song_id);
    let pub_res = client.cloud_publish(&pub_q).await
        .map_err(|e| format!("发布失败: {}", e))?;

    let _ = app_handle.emit("cloud-upload-complete", &filename);
    Ok(pub_res.body.to_string())
}
