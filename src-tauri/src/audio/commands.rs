//! Tauri 命令
//!
//! 暴露给前端调用的所有 `#[tauri::command]` 函数。
//! `AppAudio` 是 Tauri 管理的状态，内部包装 `AudioController` 的互斥锁。

use std::sync::Mutex as StdMutex;
use tauri::State;

use crate::audio::controller::AudioController;
use crate::audio::device::list_output_devices;

/// Tauri 管理的音频状态，内部包装 `AudioController` 的互斥锁
pub struct AppAudio(pub StdMutex<AudioController>);

/// Tauri 命令：播放网络音频
#[tauri::command]
pub fn play_audio(state: State<'_, AppAudio>, url: String) -> Result<(), String> {
    let ctrl = state.0.lock().map_err(|e| e.to_string())?;
    ctrl.play_url(&url);
    Ok(())
}

/// Tauri 命令：播放本地音频文件
#[tauri::command]
pub fn play_local_audio(state: State<'_, AppAudio>, path: String) -> Result<(), String> {
    let ctrl = state.0.lock().map_err(|e| e.to_string())?;
    ctrl.play_local(&path);
    Ok(())
}

/// Tauri 命令：暂停当前播放
#[tauri::command]
pub fn pause_audio(state: State<'_, AppAudio>) {
    if let Ok(ctrl) = state.0.lock() { ctrl.pause(); }
}

/// Tauri 命令：恢复播放
#[tauri::command]
pub fn resume_audio(state: State<'_, AppAudio>) {
    if let Ok(ctrl) = state.0.lock() { ctrl.resume(); }
}

/// Tauri 命令：停止当前播放
#[tauri::command]
pub fn stop_audio(state: State<'_, AppAudio>) {
    if let Ok(ctrl) = state.0.lock() { ctrl.stop(); }
}

/// Tauri 命令：获取所有可用的音频输出设备列表
#[tauri::command]
pub fn get_output_devices() -> Vec<String> {
    list_output_devices()
}

/// Tauri 命令：设置音频输出设备，传入 None 使用系统默认设备
#[tauri::command]
pub fn set_output_device(state: State<'_, AppAudio>, device: Option<String>) {
    if let Ok(ctrl) = state.0.lock() { ctrl.set_device(device); }
}

/// Tauri 命令：跳转到指定播放位置（秒）
#[tauri::command]
pub fn seek_audio(state: State<'_, AppAudio>, time: f64) {
    if let Ok(ctrl) = state.0.lock() { ctrl.seek(time); }
}

/// Tauri 命令：获取当前播放位置（秒）
#[tauri::command]
pub fn get_audio_position(state: State<'_, AppAudio>) -> f64 {
    if let Ok(ctrl) = state.0.lock() { ctrl.get_position() } else { 0.0 }
}

/// Tauri 命令：设置播放音量
#[tauri::command]
pub fn set_volume(state: State<'_, AppAudio>, vol: f32) {
    if let Ok(ctrl) = state.0.lock() { ctrl.set_volume(vol); }
}

#[tauri::command]
pub fn is_audio_playing(state: State<'_, AppAudio>) -> bool {
    if let Ok(ctrl) = state.0.lock() { ctrl.get_is_playing() } else { false }
}
