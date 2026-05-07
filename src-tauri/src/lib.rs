use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    menu::{MenuBuilder, MenuItemBuilder},
    Manager, LogicalSize, Emitter,
};

mod api;
mod audio;
use api::ApiController;
use audio::AppAudio;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            // 窗口最小尺寸
            window.set_min_size(Some(LogicalSize::new(1280.0, 700.0)))?;

            // 注入控制器
            let api_controller = ApiController::new();
            app.manage(api_controller);

            let audio_controller = audio::AudioController::new(app.handle().clone());
            let app_audio = AppAudio(std::sync::Mutex::new(audio_controller));
            app.manage(app_audio);

            // 托盘菜单
            let show = MenuItemBuilder::with_id("show", "显示窗口").build(app)?;
            let play_pause = MenuItemBuilder::with_id("play_pause", "播放/暂停").build(app)?;
            let next = MenuItemBuilder::with_id("next", "下一首").build(app)?;
            let prev = MenuItemBuilder::with_id("prev", "上一首").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "退出").build(app)?;

            let menu = MenuBuilder::new(app)
                .item(&show)
                .separator()
                .item(&play_pause)
                .item(&next)
                .item(&prev)
                .separator()
                .item(&quit)
                .build()?;

            // 托盘图标（使用应用默认图标）
            let icon = app.default_window_icon().cloned().unwrap();

            let _tray = TrayIconBuilder::with_id("main-tray")
                .tooltip("Nekosonic")
                .icon(icon)
                .menu(&menu)
                .on_menu_event(|app, event| {
                    let window = app.get_webview_window("main").unwrap();
                    match event.id().as_ref() {
                        "show" => {
                            window.show().unwrap();
                            window.set_focus().unwrap();
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
                            app.exit(0);
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
                        let window = app.get_webview_window("main").unwrap();
                        window.show().unwrap();
                        window.set_focus().unwrap();
                    }
                })
                .build(app)?;

            // 点击关闭按钮时隐藏到托盘
            let window_clone = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api: close_api, .. } = event {
                    close_api.prevent_close();          // 阻止窗口关闭
                    let _ = window_clone.hide();        // 隐藏到托盘
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

            audio::play_audio,
            audio::pause_audio,
            audio::resume_audio,
            audio::stop_audio,
            audio::get_output_devices,
            audio::set_output_device,
            audio::seek_audio,
            audio::set_volume
        ])
        .run(tauri::generate_context!())
        .expect("error while running Nekosonic");
}