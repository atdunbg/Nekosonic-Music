use ncm_api_rs::{create_client, ApiClient, Query};
use serde::Deserialize;
use tauri::State;
use tokio::sync::Mutex;          // 异步 Mutex
use std::sync::Mutex as StdMutex; // 同步 Mutex 用于 cookie

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
        .filter_map(|c| c.split(';').next())   // 取第一个键值对
        .map(|s| s.trim().to_string())
        .collect::<Vec<_>>()
        .join("; ")
}

impl ApiController {

    pub fn new() -> Self {
    let cookie_path = std::env::temp_dir().join("netease_cookies.json");
    let saved_cookie = fs::read_to_string(&cookie_path)
        .map(|s| s.trim().to_string())
        .ok();  // 注意这里返回 Option<String>
    // eprintln!("[api] 启动时加载 cookie: {:?}", saved_cookie);

    let client = create_client(None); // 不依赖客户端存储，我们自己管理
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
            // eprintln!("[api] 请求携带 cookie: {}", c);
            query = query.cookie(c);
        }
    }
    query
}
    /// 保存 cookie 到文件
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

// 搜索歌曲
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

// 获取热搜词
#[tauri::command]
pub async fn get_hot_search(state: State<'_, ApiController>) -> Result<String, String> {
    let client = state.client.lock().await;
    let q = state.build_query();
    client.search_hot_detail(&q).await
        .map(|r| r.body.to_string())
        .map_err(|e| e.to_string())
}


// 获取歌曲链接
#[tauri::command]
pub async fn get_song_url(id: u64, state: State<'_, ApiController>) -> Result<String, String> {
    let client = state.client.lock().await;
    let q = state.build_query()
        .param("id", &id.to_string())
        .param("level", "standard");
    let resp = client.song_url_v1(&q).await.map_err(|e| e.to_string())?;
    resp.body["data"][0]["url"].as_str()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .ok_or_else(|| "暂无播放源".into())
}


// 获取歌词
#[tauri::command]
pub async fn get_lyric(id: u64, state: State<'_, ApiController>) -> Result<String, String> {
    let client = state.client.lock().await;
    let q = state.build_query().param("id", &id.to_string());
    client.lyric(&q).await
        .map(|r| r.body.to_string())
        .map_err(|e| e.to_string())
}


// 获取歌单详情
#[tauri::command]
pub async fn get_playlist_detail(id: u64, state: State<'_, ApiController>) -> Result<String, String> {
    let client = state.client.lock().await;
    let q = state.build_query().param("id", &id.to_string());
    client.playlist_detail(&q).await
        .map(|r| r.body.to_string())
        .map_err(|e| e.to_string())
}

// 登录
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

// 登出
#[tauri::command]
pub async fn logout(state: State<'_, ApiController>) -> Result<(), String> {
    // 清除内存中的 cookie
    *state.cookie.lock().map_err(|e| e.to_string())? = None;
    // 删除持久化文件
    let _ = fs::remove_file(&state.cookie_path);
    Ok(())
}

// 获取二维码key
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

// 创建二维码, 功能暂时有问题
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
    // 提取 qrurl 字段（网易云新的返回格式）
    let qrurl = resp.body["data"]["qrurl"]
        .as_str()
        .ok_or("未获取到二维码链接")?
        .to_string();
    Ok(qrurl)
}

// 检查二维码状态
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

// 获取登录状态
#[tauri::command]
pub async fn get_login_status(state: State<'_, ApiController>) -> Result<String, String> {
    let client = state.client.lock().await;
    let q = state.build_query();
    client.user_account(&q).await
        .map(|r| r.body.to_string())
        .map_err(|e| e.to_string())
}

// 用户歌单
#[tauri::command]
pub async fn user_playlist(uid: u64, state: State<'_, ApiController>) -> Result<String, String> {
    let client = state.client.lock().await;
    let q = state.build_query().param("uid", &uid.to_string());
    let resp = client.user_playlist(&q).await.map_err(|e| e.to_string())?;
    Ok(resp.body.to_string())
}

// 每日推荐歌曲
#[tauri::command]
pub async fn recommend_songs(state: State<'_, ApiController>) -> Result<String, String> {
    let client = state.client.lock().await;
    let q = state.build_query();
    let resp = client.recommend_songs(&q).await.map_err(|e| e.to_string())?;
    Ok(resp.body.to_string())
}

// 推荐歌单（需要登录）
#[tauri::command]
pub async fn recommend_resource(state: State<'_, ApiController>) -> Result<String, String> {
    let client = state.client.lock().await;
    let q = state.build_query();
    let resp = client.recommend_resource(&q).await.map_err(|e| e.to_string())?;
    Ok(resp.body.to_string())
}

#[tauri::command]
pub async fn personal_fm(state: State<'_, ApiController>) -> Result<String, String> {
    let client = state.client.lock().await;
    let q = state.build_query();
    let resp = client.personal_fm(&q).await.map_err(|e| e.to_string())?;
    Ok(resp.body.to_string())
}

#[tauri::command]
pub async fn get_song_detail(id: u64, state: State<'_, ApiController>) -> Result<String, String> {
    let client = state.client.lock().await;
    let q = state.build_query().param("ids", &id.to_string());
    let resp = client.song_detail(&q).await.map_err(|e| e.to_string())?;
    Ok(resp.body.to_string())
}