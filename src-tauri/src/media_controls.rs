use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Listener};
use souvlaki::{
    MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback,
    MediaPosition, PlatformConfig, SeekDirection,
};

struct MediaState {
    controls: MediaControls,
}

pub fn start_media_controls(app_handle: AppHandle, hwnd: Option<*mut std::ffi::c_void>) {
    let config = PlatformConfig {
        dbus_name: "nekosonic",
        display_name: "Nekosonic",
        hwnd,
    };

    let mut controls = match MediaControls::new(config) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to create media controls: {e}");
            return;
        }
    };

    let ah = app_handle.clone();
    if let Err(e) = controls.attach(move |event: MediaControlEvent| {
        let cmd = match &event {
            MediaControlEvent::Play => "Play",
            MediaControlEvent::Pause => "Pause",
            MediaControlEvent::Toggle => "PlayPause",
            MediaControlEvent::Next => "Next",
            MediaControlEvent::Previous => "Previous",
            MediaControlEvent::Stop => "Stop",
            MediaControlEvent::Raise => "Raise",
            MediaControlEvent::Quit => "Quit",
            MediaControlEvent::SetVolume(v) => {
                let _ = ah.emit("mpris-command", format!("SetVolume:{v}"));
                return;
            }
            MediaControlEvent::Seek(dir) => {
                let offset_us = match dir {
                    SeekDirection::Forward => 5_000_000i64,
                    SeekDirection::Backward => -5_000_000i64,
                };
                let _ = ah.emit("mpris-command", format!("Seek:{offset_us}"));
                return;
            }
            MediaControlEvent::SeekBy(dir, duration) => {
                let offset_us: i64 = match dir {
                    SeekDirection::Forward => duration.as_micros() as i64,
                    SeekDirection::Backward => -(duration.as_micros() as i64),
                };
                let _ = ah.emit("mpris-command", format!("Seek:{offset_us}"));
                return;
            }
            MediaControlEvent::SetPosition(pos) => {
                let pos_us = pos.0.as_micros() as i64;
                let _ = ah.emit("mpris-command", format!("SetPosition:{pos_us}"));
                return;
            }
            MediaControlEvent::OpenUri(_) => return,
        };
        let _ = ah.emit("mpris-command", cmd);
    }) {
        eprintln!("Failed to attach media control handler: {e}");
        return;
    }

    let state = Arc::new(Mutex::new(MediaState { controls }));
    let state_for_listener = state.clone();

    app_handle.listen("playback-state", move |event| {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(event.payload()) {
            let mut s = match state_for_listener.lock() {
                Ok(s) => s,
                Err(_) => return,
            };

            if let Some(status) = data.get("status").and_then(|v| v.as_str()) {
                let position_us = data.get("positionUs").and_then(|v| v.as_i64()).unwrap_or(0);
                let progress = if position_us > 0 {
                    Some(MediaPosition(std::time::Duration::from_micros(position_us as u64)))
                } else {
                    None
                };
                let playback = match status {
                    "playing" => MediaPlayback::Playing { progress },
                    "paused" => MediaPlayback::Paused { progress },
                    _ => MediaPlayback::Stopped,
                };
                let _ = s.controls.set_playback(playback);
            }

            let title = data.get("title").and_then(|v| v.as_str()).unwrap_or("");
            let album = data.get("album").and_then(|v| v.as_str()).unwrap_or("");
            let artists = data
                .get("artists")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|a| a.as_str().map(|s| s.to_owned()))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let artist_str = artists.join(", ");
            let cover_url = data.get("coverUrl").and_then(|v| v.as_str()).unwrap_or("");
            let duration_us = data.get("durationUs").and_then(|v| v.as_i64()).unwrap_or(0);

            let metadata = MediaMetadata {
                title: if title.is_empty() { None } else { Some(title) },
                album: if album.is_empty() { None } else { Some(album) },
                artist: if artist_str.is_empty() { None } else { Some(&artist_str) },
                cover_url: if cover_url.is_empty() { None } else { Some(cover_url) },
                duration: if duration_us > 0 {
                    Some(std::time::Duration::from_micros(duration_us as u64))
                } else {
                    None
                },
            };
            let _ = s.controls.set_metadata(metadata);
        }
    });

    std::mem::forget(state);
}
