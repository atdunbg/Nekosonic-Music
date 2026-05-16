use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem},
    Manager, Emitter,
};
use std::sync::atomic::{AtomicBool, Ordering};

mod api;
mod audio;
use api::ApiController;
use audio::AppAudio;

static ALLOW_EXIT: AtomicBool = AtomicBool::new(false);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {

            let app_data_dir = app.path().app_data_dir().expect("无法获取应用数据目录");
            let api_controller = ApiController::new(app_data_dir);
            app.manage(api_controller);

            let audio_controller = audio::AudioController::new(app.handle().clone());
            let app_audio = AppAudio(std::sync::Mutex::new(audio_controller));
            app.manage(app_audio);

            let show = MenuItemBuilder::with_id("show", "显示窗口").build(app)?;
            let _sep1 = PredefinedMenuItem::separator(app)?;
            let prev = MenuItemBuilder::with_id("prev", "上一首").build(app)?;
            let play_pause = MenuItemBuilder::with_id("play_pause", "播放/暂停").build(app)?;
            let next = MenuItemBuilder::with_id("next", "下一首").build(app)?;
            let _sep2 = PredefinedMenuItem::separator(app)?;
            let quit = MenuItemBuilder::with_id("quit", "退出").build(app)?;

            let menu = MenuBuilder::new(app)
                .item(&show)
                .separator()
                .items(&[&prev, &play_pause, &next])
                .separator()
                .item(&quit)
                .build()?;

            let icon = app.default_window_icon().cloned().unwrap();

            let _tray = TrayIconBuilder::with_id("main-tray")
                .tooltip("Nekosonic Music")
                .icon(icon)
                .show_menu_on_left_click(false)
                .menu(&menu)
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.unminimize();
                                let _ = window.set_focus();
                                let _ = app.emit("window-shown", ());
                            }
                        }
                        "play_pause" => {
                            let _ = app.emit("tray-play-pause", ());
                        }
                        "next" => {
                            let _ = app.emit("tray-next", ());
                        }
                        "prev" => {
                            let _ = app.emit("tray-prev", ());
                        }
                        "quit" => {
                            ALLOW_EXIT.store(true, Ordering::SeqCst);
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.close();
                            }
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                            let _ = app.emit("window-shown", ());
                        }
                    }
                })
                .build(app)?;

            let window = app.get_webview_window("main").unwrap();
            let window_clone = window.clone();
            let app_handle = app.handle().clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api: close_api, .. } = event {
                    if ALLOW_EXIT.load(Ordering::SeqCst) {
                        return;
                    }
                    close_api.prevent_close();
                    let _ = window_clone.hide();
                    let _ = app_handle.emit("window-hidden", ());
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            api::login,
            api::logout,

            api::search_songs,
            api::get_song_url,
            api::get_hot_search,
            api::get_playlist_detail,
            api::get_lyric,
            api::user_playlist,
            api::recommend_resource,
            api::recommend_songs,
            api::personal_fm,
            api::get_song_detail,
            api::get_qr_key,
            api::create_qr,
            api::check_qr_status,
            api::get_login_status,
            api::likelist,
            api::user_record,
            api::like_song,
            api::record_recent_song,
            api::playlist_subscribe,
            api::playlist_track_all,
            api::exit_app,

            audio::play_audio,
            audio::play_local_audio,
            audio::pause_audio,
            audio::resume_audio,
            audio::stop_audio,
            audio::get_output_devices,
            audio::set_output_device,
            audio::seek_audio,
            audio::set_volume,

            api::download_song,
            api::list_local_songs,
            api::delete_local_song,
            api::check_local_song,
            api::get_default_download_path,

            api::artist_detail,
            api::artist_songs,
            api::artist_album,
            api::artist_desc,
            api::album_detail,
            api::comment_new,
            api::comment_hot,
            api::comment_floor,
            api::comment_like,
        ])
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
                let _ = window.unminimize();
                let _ = app.emit("window-shown", ());
            }
        }))
        .run(tauri::generate_context!())
        .expect("error while running Nekosonic");
}
