use ncm_api_rs::{create_client, ApiClient, Query};
use serde::Deserialize;
use serde_json::json;
use tauri::{Manager, State, Emitter};
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
        let client = $state.client.lock().unwrap().clone();
        let q = $state.build_query();
        client.$method(&q).await
            .map(|r| r.body.to_string())
            .map_err(|e| e.to_string())
    }};
    ($state:expr, $method:ident, params: [$(($key:expr, $val:expr)),* $(,)?]) => {{
        let client = $state.client.lock().unwrap().clone();
        let mut q = $state.build_query();
        $(q = q.param($key, $val);)*
        client.$method(&q).await
            .map(|r| r.body.to_string())
            .map_err(|e| e.to_string())
    }};
    ($state:expr, $method:ident, query: $q:expr) => {{
        let client = $state.client.lock().unwrap().clone();
        client.$method(&$q).await
            .map(|r| r.body.to_string())
            .map_err(|e| e.to_string())
    }};
}

pub struct ApiController {
    client: StdMutex<ApiClient>,
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
        client: StdMutex::new(client),
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
    fn save_cookie(&self, cookie_str: &str) {
        let _ = fs::write(&self.cookie_path, cookie_str);
        if let Ok(mut client) = self.client.lock() {
            client.set_cookie(cookie_str.to_string());
        }
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
    let client = state.client.lock().unwrap().clone();
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
    let client = state.client.lock().unwrap().clone();
    let q = Query::new()
        .param("phone", &query.phone)
        .param("password", &query.password);
    let resp = client.login_cellphone(&q).await.map_err(|e| e.to_string())?;

    if !resp.cookie.is_empty() {
        let cookie_str = cookies_to_key_values(&resp.cookie);
        *state.cookie.lock().map_err(|e| e.to_string())? = Some(cookie_str.clone());
        state.save_cookie(&cookie_str);
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
    let client = state.client.lock().unwrap().clone();
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
    let client = state.client.lock().unwrap().clone();
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
    let client = state.client.lock().unwrap().clone();
    let q = state.build_query().param("key", &query.key);
    let resp = client.login_qr_check(&q).await.map_err(|e| e.to_string())?;
    if resp.body["code"].as_u64() == Some(803) && !resp.cookie.is_empty() {
        let cookie_str = cookies_to_key_values(&resp.cookie);
        *state.cookie.lock().map_err(|e| e.to_string())? = Some(cookie_str.clone());
        state.save_cookie(&cookie_str);
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
}

/// 听歌打卡
#[tauri::command]
pub async fn scrobble(query: ScrobbleQuery, state: State<'_, ApiController>) -> Result<String, String> {
    api_call!(state, scrobble, params: [("id", &query.id.to_string()), ("sourceid", query.sourceid.as_deref().unwrap_or("")), ("time", &query.time.to_string())])
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
    let client = state.client.lock().unwrap().clone();
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

/// 列出本地已下载的歌曲，优先使用元数据文件补充信息
#[tauri::command]
pub fn list_local_songs(app_handle: tauri::AppHandle, download_path: Option<String>) -> Result<Vec<LocalSongInfo>, String> {
    let download_dir = resolve_download_dir(&app_handle, download_path.as_deref());
    if !download_dir.exists() {
        return Ok(Vec::new());
    }

    let audio_exts = ["mp3", "flac", "wav", "ogg", "aac", "m4a", "wma", "opus"];

    let mut meta_map: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();
    let entries = fs::read_dir(&download_dir).map_err(|e| format!("读取目录失败: {}", e))?;

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
    let entries = fs::read_dir(&download_dir).map_err(|e| format!("读取目录失败: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !audio_exts.contains(&ext.to_lowercase().as_str()) {
            continue;
        }

        let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        let file_size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

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
pub fn delete_local_song(
    app_handle: tauri::AppHandle,
    query: DeleteLocalSongQuery,
) -> Result<(), String> {
    let download_dir = resolve_download_dir(&app_handle, query.download_path.as_deref());
    let file_path = download_dir.join(&query.filename);
    let meta_path = download_dir.join(format!("{}.json", query.id));

    if file_path.exists() {
        fs::remove_file(&file_path).map_err(|e| format!("删除文件失败: {}", e))?;
    }
    if meta_path.exists() {
        fs::remove_file(&meta_path).map_err(|e| format!("删除元数据失败: {}", e))?;
    }
    Ok(())
}

/// 检查指定歌曲是否已下载到本地
#[tauri::command]
pub fn check_local_song(app_handle: tauri::AppHandle, id: u64, download_path: Option<String>) -> Result<bool, String> {
    let download_dir = resolve_download_dir(&app_handle, download_path.as_deref());
    let meta_path = download_dir.join(format!("{}.json", id));
    Ok(meta_path.exists())
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
pub fn get_default_download_path(app_handle: tauri::AppHandle) -> String {
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
