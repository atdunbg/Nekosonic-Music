use ncm_api_rs::{create_client, ApiClient, Query};
use serde::Deserialize;
use tauri::{Manager, State};
use tokio::sync::Mutex;
use std::sync::Mutex as StdMutex;
use std::sync::atomic::Ordering;

use std::fs;
use std::path::PathBuf;

pub struct ApiController {
    client: Mutex<ApiClient>,
    cookie: StdMutex<Option<String>>,
    cookie_path: PathBuf,
}

fn cookies_to_key_values(cookies: &[String]) -> String {
    cookies
        .iter()
        .filter_map(|c| c.split(';').next())
        .map(|s| s.trim().to_string())
        .collect::<Vec<_>>()
        .join("; ")
}

impl ApiController {

    pub fn new(app_data_dir: PathBuf) -> Self {
    let _ = fs::create_dir_all(&app_data_dir);
    let cookie_path = app_data_dir.join("netease_cookies.json");
    let saved_cookie = fs::read_to_string(&cookie_path)
        .map(|s| s.trim().to_string())
        .ok();

    let client = create_client(None);
    ApiController {
        client: Mutex::new(client),
        cookie: StdMutex::new(saved_cookie),
        cookie_path,
    }
}

fn build_query(&self) -> Query {
    let mut query = Query::new();
    if let Ok(cookie_guard) = self.cookie.lock() {
        if let Some(c) = cookie_guard.as_ref() {
            query = query.cookie(c);
        }
    }
    query
}
    fn save_cookie(&self, cookie_str: &str) {
        let _ = fs::write(&self.cookie_path, cookie_str);
    }
}

#[derive(Deserialize)]
pub struct SearchQuery { pub keyword: String }

#[derive(Deserialize)]
pub struct LoginQuery { pub phone: String, pub password: String }

#[derive(Deserialize)]
pub struct QrKeyQuery { pub key: String }

/// 搜索歌曲
#[tauri::command]
pub async fn search_songs(query: SearchQuery, state: State<'_, ApiController>) -> Result<String, String> {
    let client = state.client.lock().await;
    let q = state.build_query()
        .param("keywords", &query.keyword)
        .param("type", "1")
        .param("limit", "30");
    client.cloudsearch(&q).await
        .map(|r| r.body.to_string())
        .map_err(|e| e.to_string())
}

/// 获取热搜词列表
#[tauri::command]
pub async fn get_hot_search(state: State<'_, ApiController>) -> Result<String, String> {
    let client = state.client.lock().await;
    let q = state.build_query();
    client.search_hot_detail(&q).await
        .map(|r| r.body.to_string())
        .map_err(|e| e.to_string())
}

#[derive(Deserialize)]
pub struct PlaylistTrackAllQuery { pub id: u64, pub limit: Option<i64>, pub offset: Option<i64> }

/// 获取歌单全部歌曲
#[tauri::command]
pub async fn playlist_track_all(query: PlaylistTrackAllQuery, state: State<'_, ApiController>) -> Result<String, String> {
    let client = state.client.lock().await;
    let q = state.build_query()
        .param("id", &query.id.to_string())
        .param("limit", &query.limit.unwrap_or(1000).to_string())
        .param("offset", &query.offset.unwrap_or(0).to_string());
    client.playlist_track_all(&q).await
        .map(|r| r.body.to_string())
        .map_err(|e| e.to_string())
}

#[derive(Deserialize)]
pub struct SongUrlQuery { pub id: u64, pub level: Option<String> }

/// 获取歌曲播放地址
#[tauri::command]
pub async fn get_song_url(query: SongUrlQuery, state: State<'_, ApiController>) -> Result<String, String> {
    let client = state.client.lock().await;
    let level = query.level.as_deref().unwrap_or("standard");
    let q = state.build_query()
        .param("id", &query.id.to_string())
        .param("level", level);
    let resp = client.song_url_v1(&q).await.map_err(|e| e.to_string())?;
    resp.body["data"][0]["url"].as_str()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .ok_or_else(|| "暂无播放源".into())
}

/// 获取歌词
#[tauri::command]
pub async fn get_lyric(id: u64, state: State<'_, ApiController>) -> Result<String, String> {
    let client = state.client.lock().await;
    let q = state.build_query().param("id", &id.to_string());
    client.lyric(&q).await
        .map(|r| r.body.to_string())
        .map_err(|e| e.to_string())
}

/// 获取歌单详情
#[tauri::command]
pub async fn get_playlist_detail(id: u64, state: State<'_, ApiController>) -> Result<String, String> {
    let client = state.client.lock().await;
    let q = state.build_query().param("id", &id.to_string());
    client.playlist_detail(&q).await
        .map(|r| r.body.to_string())
        .map_err(|e| e.to_string())
}

/// 手机号密码登录
#[tauri::command]
pub async fn login(query: LoginQuery, state: State<'_, ApiController>) -> Result<String, String> {
    let client = state.client.lock().await;
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
    let client = state.client.lock().await;
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
    let client = state.client.lock().await;
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
    let client = state.client.lock().await;
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
    let client = state.client.lock().await;
    let q = state.build_query();
    client.user_account(&q).await
        .map(|r| r.body.to_string())
        .map_err(|e| e.to_string())
}

/// 获取用户歌单列表
#[tauri::command]
pub async fn user_playlist(uid: u64, state: State<'_, ApiController>) -> Result<String, String> {
    let client = state.client.lock().await;
    let q = state.build_query().param("uid", &uid.to_string());
    let resp = client.user_playlist(&q).await.map_err(|e| e.to_string())?;
    Ok(resp.body.to_string())
}

/// 获取每日推荐歌曲
#[tauri::command]
pub async fn recommend_songs(state: State<'_, ApiController>) -> Result<String, String> {
    let client = state.client.lock().await;
    let q = state.build_query();
    let resp = client.recommend_songs(&q).await.map_err(|e| e.to_string())?;
    Ok(resp.body.to_string())
}

/// 获取推荐歌单
#[tauri::command]
pub async fn recommend_resource(state: State<'_, ApiController>) -> Result<String, String> {
    let client = state.client.lock().await;
    let q = state.build_query();
    let resp = client.recommend_resource(&q).await.map_err(|e| e.to_string())?;
    Ok(resp.body.to_string())
}

/// 获取私人漫游歌曲
#[tauri::command]
pub async fn personal_fm(state: State<'_, ApiController>) -> Result<String, String> {
    let client = state.client.lock().await;
    let q = state.build_query();
    let resp = client.personal_fm(&q).await.map_err(|e| e.to_string())?;
    Ok(resp.body.to_string())
}

/// 获取歌曲详情
#[tauri::command]
pub async fn get_song_detail(id: String, state: State<'_, ApiController>) -> Result<String, String> {
    let client = state.client.lock().await;
    let q = state.build_query().param("ids", &id);
    let resp = client.song_detail(&q).await.map_err(|e| e.to_string())?;
    Ok(resp.body.to_string())
}

#[derive(Deserialize)]
pub struct UserRecordQuery { pub uid: u64, pub r#type: String }

#[derive(Deserialize)]
pub struct LikeSongQuery { pub id: u64, pub like: String }

/// 获取喜欢的歌曲ID列表
#[tauri::command]
pub async fn likelist(uid: u64, state: State<'_, ApiController>) -> Result<String, String> {
    let client = state.client.lock().await;
    let q = state.build_query().param("uid", &uid.to_string());
    client.likelist(&q).await
        .map(|r| r.body.to_string())
        .map_err(|e| e.to_string())
}

/// 获取用户播放记录
#[tauri::command]
pub async fn user_record(query: UserRecordQuery, state: State<'_, ApiController>) -> Result<String, String> {
    let client = state.client.lock().await;
    let q = state.build_query()
        .param("uid", &query.uid.to_string())
        .param("type", &query.r#type);
    client.user_record(&q).await
        .map(|r| r.body.to_string())
        .map_err(|e| e.to_string())
}

/// 喜欢/取消喜欢歌曲
#[tauri::command]
pub async fn like_song(query: LikeSongQuery, state: State<'_, ApiController>) -> Result<String, String> {
    let client = state.client.lock().await;
    let q = state.build_query()
        .param("id", &query.id.to_string())
        .param("like", &query.like);
    client.like(&q).await
        .map(|r| r.body.to_string())
        .map_err(|e| e.to_string())
}

/// 上报最近播放歌曲
#[tauri::command]
pub async fn record_recent_song(limit: u64, state: State<'_, ApiController>) -> Result<String, String> {
    let client = state.client.lock().await;
    let q = state.build_query().param("limit", &limit.to_string());
    client.record_recent_song(&q).await
        .map(|r| r.body.to_string())
        .map_err(|e| e.to_string())
}

#[derive(Deserialize)]
pub struct PlaylistSubscribeQuery { pub id: u64, pub subscribe: Option<bool> }

/// 收藏/取消收藏歌单
#[tauri::command]
pub async fn playlist_subscribe(query: PlaylistSubscribeQuery, state: State<'_, ApiController>) -> Result<String, String> {
    let client = state.client.lock().await;
    let t = if query.subscribe.unwrap_or(true) { "1" } else { "0" };
    let q = state.build_query()
        .param("id", &query.id.to_string())
        .param("t", t);
    client.playlist_subscribe(&q).await
        .map(|r| r.body.to_string())
        .map_err(|e| e.to_string())
}

/// 退出应用
#[tauri::command]
pub async fn exit_app(app_handle: tauri::AppHandle) {
    crate::ALLOW_EXIT.store(true, Ordering::SeqCst);
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.close();
    }
}
