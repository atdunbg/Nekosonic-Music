//! 音频流式下载
//!
//! 负责从网络 URL 流式下载音频数据到共享缓冲区，并通过事件通知下载进度。

use std::io::Read;
use tauri::{AppHandle, Emitter};

use crate::audio::buffer::SharedBuffer;

/// 流式下载音频数据到共享缓冲区，支持下载进度事件通知
pub fn download_audio_streaming(
    url: &str,
    buffer: &SharedBuffer,
    app_handle: &AppHandle,
) -> Result<(), String> {
    let resp = reqwest::blocking::get(url).map_err(|e| format!("下载失败: {}", e))?;
    if !resp.status().is_success() {
        return Err(format!("HTTP 错误: {}", resp.status()));
    }
    let total_size = resp.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut reader = resp;
    loop {
        if buffer.is_cancelled() {
            return Err("下载已取消".to_string());
        }
        let mut chunk = [0u8; 8192];
        let read_size = reader
            .read(&mut chunk)
            .map_err(|e| format!("读取失败: {}", e))?;
        if read_size == 0 {
            break;
        }
        buffer.write_chunk(&chunk[..read_size]);
        downloaded += read_size as u64;
        let progress = if total_size > 0 {
            (downloaded as f64 / total_size as f64) * 100.0
        } else {
            0.0
        };
        let _ = app_handle.emit("cache-progress", progress);
    }
    Ok(())
}
